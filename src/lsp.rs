use std::{
    collections::HashMap,
    os::fd::AsRawFd,
    process::{Child, Command, Stdio},
};

use mio::{event::Event, unix::SourceFd, Interest, Poll, Token};
use orfail::OrFail;

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

#[derive(Debug)]
pub struct LspClient {
    lsp_server: Child,
    stdin_token: Token,
    stdout_token: Token,
    stderr_token: Token,
}

impl LspClient {
    pub fn start(
        poller: &mut Poll,
        stdin_token: Token,
        stdout_token: Token,
        stderr_token: Token,
        params: &StartLspParams,
    ) -> Result<Self, RpcError> {
        let lsp_server = Command::new(&params.command)
            .args(&params.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        log::info!("Started LSP server: {}", params.command.display());

        for (fd, token) in [
            lsp_server
                .stdin
                .as_ref()
                .map(|t| (t.as_raw_fd(), stdin_token)),
            lsp_server
                .stdout
                .as_ref()
                .map(|t| (t.as_raw_fd(), stdout_token)),
            lsp_server
                .stderr
                .as_ref()
                .map(|t| (t.as_raw_fd(), stderr_token)),
        ]
        .into_iter()
        .filter_map(|t| t)
        {
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
        })
    }

    fn handle_event(&mut self, poller: &mut Poll, event: &Event) -> orfail::Result<bool> {
        if let Some(status) = self.lsp_server.try_wait().or_fail()? {
            log::info!("LSP server exited: {status}");

            for fd in [
                self.lsp_server.stdin.as_ref().map(|t| t.as_raw_fd()),
                self.lsp_server.stdout.as_ref().map(|t| t.as_raw_fd()),
                self.lsp_server.stderr.as_ref().map(|t| t.as_raw_fd()),
            ]
            .into_iter()
            .filter_map(|t| t)
            {
                let _ = poller.registry().deregister(&mut SourceFd(&fd));
            }

            return Ok(false);
        }

        todo!()
    }
}
