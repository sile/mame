use std::{
    collections::HashMap,
    io::{Read, Write},
    net::SocketAddr,
    os::fd::AsRawFd,
    path::PathBuf,
    process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, Stdio},
};

use jsonlrpc::{JsonRpcVersion, RequestId, ResponseObject};
use mio::{event::Event, unix::SourceFd, Interest, Poll, Token};
use orfail::OrFail;
use serde::Serialize;

use crate::rpc::{self, RpcError, StartLspParams};

#[derive(Debug)]
pub struct LspClientManager {
    editor: SocketAddr,
    min_token: Token,
    max_token: Token,
    next_token: Token,
    clients: HashMap<String, LspClient>,
    token_to_client_id: HashMap<Token, String>,
}

impl LspClientManager {
    pub fn new(editor: SocketAddr, min_token: Token, max_token: Token) -> Self {
        Self {
            editor,
            min_token,
            max_token,
            next_token: min_token,
            clients: HashMap::new(),
            token_to_client_id: HashMap::new(),
        }
    }

    fn next_token(&mut self) -> Token {
        let token = self.next_token;
        self.next_token.0 += 1;
        if self.next_token == self.max_token {
            self.next_token = self.min_token;
        }
        token
    }

    pub fn start(&mut self, poller: &mut Poll, params: &StartLspParams) -> Result<(), RpcError> {
        if self.clients.contains_key(&params.name) {
            return Err(RpcError::other("LSP server name conflicts"));
        }

        let stdin_token = self.next_token();
        let stdout_token = self.next_token();
        let stderr_token = self.next_token();

        for token in [stdin_token, stdout_token, stderr_token] {
            self.token_to_client_id.insert(token, params.name.clone());
        }

        let client = LspClient::start(
            self.editor,
            params.name.clone(),
            poller,
            stdin_token,
            stdout_token,
            stderr_token,
            params,
        )?;
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
    editor: SocketAddr,
    lsp_server: Child,
    name: String,
    stdin: ChildStdin,
    stdout: ChildStdout,
    stderr: ChildStderr,
    stdin_token: Token,
    stdout_token: Token,
    stderr_token: Token,
    send_buf: Vec<u8>,
    send_buf_offset: usize,
    recv_buf: Vec<u8>,
    recv_buf_offset: usize,
    next_request_id: i64,
    ongoing_requests: HashMap<RequestId, &'static str>,
    responses: Vec<ResponseObject>,
}

impl LspClient {
    pub fn start(
        editor: SocketAddr,
        name: String,
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
            .envs(params.env.iter())
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
            unsafe { libc::fcntl(fd, libc::F_SETFL, libc::O_NONBLOCK) }; // TODO: handle error
            poller
                .registry()
                .register(&mut SourceFd(&fd), token, Interest::READABLE)
                .expect("TODO");
        }

        let mut this = Self {
            editor,
            name,
            lsp_server,
            stdin_token,
            stdout_token,
            stderr_token,
            stdin,
            stdout,
            stderr,
            send_buf: Vec::new(),
            send_buf_offset: 0,
            recv_buf: vec![0; 4096],
            recv_buf_offset: 0,
            next_request_id: 0,
            ongoing_requests: HashMap::new(),
            responses: Vec::new(),
        };
        this.send(
            poller,
            InitializeParams::METHOD,
            false,
            &InitializeParams::new(&params.root_dir),
        )
        .or_fail()
        .map_err(|e| RpcError::other(&e.to_string()))?;
        Ok(this)
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
            #[serde(skip_serializing_if = "Option::is_none")]
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
                self.ongoing_requests.insert(RequestId::Number(id), method);
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
        // TODO: log::info!("LSP I/O event: {event:?}");
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
            if let Err(e) = self.flush(poller, false).or_fail() {
                log::error!("LSP server error: {e})");
                let _ = self.lsp_server.kill();
                return Ok(false);
            }
        }

        if event.is_readable() {
            if event.token() == self.stdout_token {
                if let Err(e) = self.read_response().or_fail() {
                    log::error!("LSP server error: {e})");
                    let _ = self.lsp_server.kill();
                    return Ok(false);
                }
            } else if event.token() == self.stderr_token {
                if let Err(e) = self.handle_stderr().or_fail() {
                    log::error!("LSP server error: {e})");
                    let _ = self.lsp_server.kill();
                    return Ok(false);
                }
            } else {
                unreachable!()
            }

            for response in std::mem::take(&mut self.responses) {
                if let Err(e) = self.handle_response(poller, response).or_fail() {
                    log::error!("LSP server error: {e})");
                    let _ = self.lsp_server.kill();
                    return Ok(false);
                }
            }
        }

        Ok(true)
    }

    fn handle_stderr(&mut self) -> orfail::Result<()> {
        let mut buf = vec![0; 4096];
        loop {
            match self.stderr.read(&mut buf) {
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    break;
                }
                Err(e) => {
                    return Err(e).or_fail();
                }
                Ok(size) => {
                    for line in String::from_utf8_lossy(&buf[..size]).lines() {
                        log::debug!("[LSP STDERR] {line}");
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_response(
        &mut self,
        poller: &mut Poll,
        response: ResponseObject,
    ) -> orfail::Result<()> {
        let id = response.id().or_fail()?;
        let method = self.ongoing_requests.remove(id).or_fail()?;
        match method {
            "initialize" => self.handle_initialize_response(poller, response).or_fail(),
            _ => Err(orfail::Failure::new(format!(
                "Unknown LSP response: {id:?}"
            ))),
        }
    }

    fn handle_initialize_response(
        &mut self,
        poller: &mut Poll,
        response: ResponseObject,
    ) -> orfail::Result<()> {
        // TODO: Handle _response
        log::debug!("LSP initialize response: {response:?}");

        self.send(poller, "initialized", true, &serde_json::Value::Null)
            .or_fail()?;

        rpc::cast(
            self.editor,
            &rpc::Request::NotifyLspStarted {
                jsonrpc: JsonRpcVersion::V2,
                params: rpc::NotifyLspStartedParams {
                    name: self.name.clone(),
                },
            },
        )
        .or_fail()?;

        Ok(())
    }

    pub fn notify_did_open(&mut self, poller: &mut Poll) -> orfail::Result<()> {
        // TODO
        Ok(())
    }

    pub fn request_semantic_tokens_full(&mut self, poller: &mut Poll) -> orfail::Result<()> {
        // TODO
        Ok(())
    }

    fn read_response(&mut self) -> orfail::Result<()> {
        loop {
            if self.recv_buf.len() == self.recv_buf_offset {
                self.recv_buf.resize(self.recv_buf_offset * 2, 0);
            }

            match self.stdout.read(&mut self.recv_buf[self.recv_buf_offset..]) {
                Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    return Ok(());
                }
                Err(e) => {
                    return Err(e).or_fail();
                }
                Ok(size) => {
                    assert_ne!(size, 0);
                    self.recv_buf_offset += size;
                }
            }

            let mut content_len = None;
            let mut offset = 0;
            while let Some(line_end) = self.recv_buf[offset..self.recv_buf_offset]
                .windows(2)
                .position(|b| b == b"\r\n")
            {
                if line_end == 0 {
                    let content_len = content_len.or_fail()?;
                    offset += 2;
                    if self.recv_buf[offset..self.recv_buf_offset].len() < content_len {
                        return Ok(());
                    }

                    let content = &self.recv_buf[offset..][..content_len];
                    let response: ResponseObject = serde_json::from_slice(content).or_fail()?;
                    log::debug!("LSP response: {response:?}");
                    self.responses.push(response);

                    self.recv_buf.drain(..offset + content_len);
                    self.recv_buf_offset -= offset + content_len;
                    offset = 0;
                    continue;
                }

                let key = "content-length:";
                let line = std::str::from_utf8(&self.recv_buf[offset..][..line_end]).or_fail()?;
                if line.len() > key.len() && line[..key.len()].eq_ignore_ascii_case(key) {
                    content_len = Some(line[key.len()..].trim().parse::<usize>().or_fail()?);
                }
                offset += line_end + 2;
            }
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InitializeParams {
    #[serde(default)]
    pub root_uri: Option<PathBuf>,
    pub client_info: Option<ClientInfo>,
    pub capabilities: ClientCapabilities,
    #[serde(default)]
    pub workspace_folders: Vec<WorkspaceFolder>,
}

impl InitializeParams {
    pub const METHOD: &'static str = "initialize";

    pub fn new(root_dir: &PathBuf) -> Self {
        let capabilities = ClientCapabilities {
            workspace: WorkspaceCapabilitylies {
                workspace_edit: WorkspaceEditClientCapabilities {
                    document_changes: true,
                },
            },
            general: GeneralClientCapabilities {
                // TODO: position_encodings: vec![PositionEncodingKind::Utf8],
                position_encodings: vec![PositionEncodingKind::Utf16],
            },
        };
        Self {
            root_uri: Some(root_dir.clone()),
            client_info: None, // TODO
            capabilities,
            workspace_folders: vec![],
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct ClientInfo {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientCapabilities {
    #[serde(default)]
    pub workspace: WorkspaceCapabilitylies,
    pub general: GeneralClientCapabilities,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneralClientCapabilities {
    #[serde(default)]
    pub position_encodings: Vec<PositionEncodingKind>,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize)]
pub enum PositionEncodingKind {
    #[serde(rename = "utf-8")]
    Utf8,
    #[default]
    #[serde(rename = "utf-16")]
    Utf16,
    #[serde(rename = "utf-32")]
    Utf32,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceCapabilitylies {
    #[serde(default)]
    pub workspace_edit: WorkspaceEditClientCapabilities,
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceEditClientCapabilities {
    #[serde(default)]
    pub document_changes: bool,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceFolder {
    pub uri: PathBuf,
    pub name: String,
}
