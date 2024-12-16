use std::{
    os::fd::AsRawFd,
    process::{Child, Command},
};

use mio::{unix::SourceFd, Interest, Poll, Token};

use crate::rpc::{RpcError, StartLspParams};

#[derive(Debug)]
pub struct LspClientManager {}

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
        let lsp_server = Command::new(&params.command).args(&params.args).spawn()?;
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

        Ok(Self {
            lsp_server,
            stdin_token,
            stdout_token,
            stderr_token,
        })
    }

    pub fn stop(mut self, poller: &mut Poll) {
        log::info!("Stops LSP server");

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

        // TODO: Send SIGTERM before SIGKILL
        let _ = self.lsp_server.kill();
    }
}
