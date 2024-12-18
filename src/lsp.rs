use std::{
    collections::HashMap,
    io::Write,
    os::fd::AsRawFd,
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
};

use jsonlrpc::JsonRpcVersion;
use mio::{event::Event, unix::SourceFd, Interest, Poll, Token};
use orfail::OrFail;
use serde::Serialize;

use crate::rpc::{RpcError, StartLspParams};

#[derive(Debug)]
pub struct LspClientManager {
    min_token: Token,
    max_token: Token,
    next_token: Token,
    clients: HashMap<String, LspClient>,
    token_to_client_id: HashMap<Token, String>,
}

impl LspClientManager {
    pub fn new(min_token: Token, max_token: Token) -> Self {
        Self {
            min_token,
            max_token,
            next_token: min_token,
            clients: HashMap::new(),
            token_to_client_id: HashMap::new(),
        }
    }

    pub fn start(&mut self, poller: &mut Poll, params: &StartLspParams) -> Result<(), RpcError> {
        if self.clients.contains_key(&params.name) {
            return Err(RpcError::other("LSP server name conflicts"));
        }

        let stdin_token = Token(self.next_token.0);
        let stdout_token = Token(self.next_token.0 + 1);
        let stderr_token = Token(self.next_token.0 + 2);
        self.next_token = Token(self.next_token.0 + 3); // TODO: wrapping handling

        for token in [stdin_token, stdout_token, stderr_token] {
            self.token_to_client_id.insert(token, params.name.clone());
        }

        let client = LspClient::start(poller, stdin_token, stdout_token, stderr_token, params)?;
        self.clients.insert(params.name.clone(), client);

        Ok(())
    }

    pub fn handle_event(&mut self, poller: &mut Poll, event: &Event) -> orfail::Result<()> {
        let Some(id) = self.token_to_client_id.get(&event.token()) else {
            return Ok(());
        };
        let client = self.clients.get_mut(id).expect("infallible");
        if !client.handle_event(poller, event).or_fail()? {
            let client = self.clients.remove(id).expect("infallible");
            self.token_to_client_id.remove(&client.stdin_token);
            self.token_to_client_id.remove(&client.stdout_token);
            self.token_to_client_id.remove(&client.stderr_token);
        }
        Ok(())
    }
}

const SEND_BUF_SIZE_LIMIT: usize = 1024 * 10;

#[derive(Debug)]
pub struct LspClient {
    lsp_server: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
    stdin_token: Token,
    stdout_token: Token,
    stderr_token: Token,
    send_buf: Vec<u8>,
    send_buf_offset: usize,
    next_request_id: i64,
    ongoing_requests: HashMap<i64, &'static str>,
}

impl LspClient {
    pub fn start(
        poller: &mut Poll,
        stdin_token: Token,
        stdout_token: Token,
        stderr_token: Token,
        params: &StartLspParams,
    ) -> Result<Self, RpcError> {
        let mut lsp_server = Command::new(&params.command)
            .args(&params.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        log::info!("Started LSP server: {}", params.command.display());

        let stdin = lsp_server.stdin.take().expect("infallible");
        let stdout = lsp_server.stdout.take().expect("infallible");
        let stderr = lsp_server.stderr.take().expect("infallible");

        for (fd, token) in [
            (stdin.as_raw_fd(), stdin_token),
            (stdout.as_raw_fd(), stdout_token),
            (stderr.as_raw_fd(), stderr_token),
        ] {
            poller
                .registry()
                .register(&mut SourceFd(&fd), token, Interest::READABLE)
                .expect("TODO");
        }

        // TODO: initialize

        Ok(Self {
            lsp_server,
            stdin_token,
            stdout_token,
            stderr_token,
            stdin,
            stdout,
            stderr,
            send_buf: Vec::new(),
            send_buf_offset: 0,
            next_request_id: 0,
            ongoing_requests: HashMap::new(),
        })
    }

    fn send<T: Serialize>(
        &mut self,
        poller: &mut Poll,
        method: &'static str,
        is_notification: bool,
        params: &T,
    ) -> orfail::Result<()> {
        if self.send_buf.len() > SEND_BUF_SIZE_LIMIT {
            log::warn!("Exceeded send buffer limit (drop a LSP request)");
            return Ok(());
        }

        #[derive(Serialize)]
        struct Request<'a, T> {
            jsonrpc: JsonRpcVersion,
            method: &'static str,
            id: Option<i64>,
            params: &'a T,
        }

        let request = Request {
            jsonrpc: JsonRpcVersion::V2,
            method,
            id: if is_notification {
                None
            } else {
                let id = self.next_request_id;
                self.ongoing_requests.insert(id, method);
                self.next_request_id += 1;
                Some(id)
            },
            params,
        };

        let content = serde_json::to_vec(&request).or_fail()?;
        let is_first = self.send_buf.is_empty();
        self.send_buf.extend_from_slice(b"Content-Length:");
        self.send_buf
            .extend_from_slice(content.len().to_string().as_bytes());
        self.send_buf.extend_from_slice(b"\r\n\r\n");
        self.send_buf.extend_from_slice(&content);

        self.flush(poller, is_first).or_fail()
    }

    fn flush(&mut self, poller: &mut Poll, is_first: bool) -> orfail::Result<()> {
        while self.send_buf_offset < self.send_buf.len() {
            match self.stdin.write(&self.send_buf[self.send_buf_offset..]) {
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    if is_first {
                        poller
                            .registry()
                            .reregister(
                                &mut SourceFd(&self.stdin.as_raw_fd()),
                                self.stdin_token,
                                Interest::READABLE | Interest::WRITABLE,
                            )
                            .or_fail()?;
                    }
                    return Ok(());
                }
                Err(e) => {
                    log::error!("Failed to write a LSP request: {e}");
                    return Err(e).or_fail();
                }
                Ok(size) => {
                    self.send_buf_offset += size;
                }
            }
        }
        self.send_buf.clear();
        self.send_buf_offset = 0;
        Ok(())
    }

    fn handle_event(&mut self, poller: &mut Poll, event: &Event) -> orfail::Result<bool> {
        if let Some(status) = self.lsp_server.try_wait().or_fail()? {
            log::info!("LSP server exited: {status}");

            for fd in [
                self.stdin.as_raw_fd(),
                self.stdout.as_raw_fd(),
                self.stderr.as_raw_fd(),
            ] {
                let _ = poller.registry().deregister(&mut SourceFd(&fd));
            }

            return Ok(false);
        }

        if event.is_writable() {
            if self.flush(poller, false).is_err() {
                let _ = self.lsp_server.kill();
                return Ok(false);
            }
        }

        todo!()
    }
}
