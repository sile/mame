use std::{net::SocketAddr, thread::JoinHandle};

use jsonlrpc::RpcClient;
use mio::net::TcpStream;
use orfail::OrFail;

use crate::rpc::{NotifyTerminalEventParams, Request};

#[derive(Debug)]
pub struct InputThread {
    rpc_client: RpcClient<TcpStream>,
}

impl InputThread {
    pub fn start(editor_addr: SocketAddr) -> orfail::Result<JoinHandle<()>> {
        let stream = TcpStream::connect(editor_addr).or_fail()?;
        let rpc_client = RpcClient::new(stream);
        let handle = std::thread::spawn(move || Self { rpc_client }.run());
        Ok(handle)
    }

    fn run(mut self) {
        log::debug!("Started input thread");
        loop {
            if let Err(e) = self.run_one().or_fail() {
                log::error!("Input thread error: {e}");
                break;
            }
        }
    }

    fn run_one(&mut self) -> orfail::Result<()> {
        let event = crossterm::event::read().or_fail()?;
        log::trace!("Terminal event: {event:?}");

        let request = Request::NotifyTerminalEvent {
            jsonrpc: jsonlrpc::JsonRpcVersion::V2,
            params: NotifyTerminalEventParams { event },
        };
        self.rpc_client.cast(&request).or_fail()?;
        Ok(())
    }
}
