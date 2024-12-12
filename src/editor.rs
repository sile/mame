use std::{net::SocketAddr, thread::JoinHandle};

use jsonlrpc_mio::RpcServer;
use mio::{Events, Poll, Token};
use orfail::OrFail;
use ratatui::DefaultTerminal;

use crate::{input::InputThread, rpc::Request};

#[derive(Debug)]
pub struct Editor {
    poller: Poll,
    events: Events,
    rpc_server: RpcServer<Request>,
    terminal: DefaultTerminal,
    input_thread_handle: JoinHandle<()>,
}

impl Editor {
    pub fn new(port: u16) -> orfail::Result<Self> {
        let mut poller = Poll::new().or_fail()?;
        let rpc_server = RpcServer::start(
            &mut poller,
            SocketAddr::from(([127, 0, 0, 1], port)),
            Token(0),
            Token(usize::MAX),
        )
        .or_fail()?;

        let mut terminal = ratatui::init();
        terminal.clear().or_fail()?;

        let input_thread_handle = InputThread::start(rpc_server.listen_addr()).or_fail()?;

        Ok(Self {
            poller,
            events: Events::with_capacity(1024),
            rpc_server,
            terminal,
            input_thread_handle,
        })
    }

    pub fn run(mut self) -> orfail::Result<()> {
        log::info!("Started editor: addr={}", self.rpc_server.listen_addr());

        loop {
            // TODO: handle key event
            self.poller.poll(&mut self.events, None).or_fail()?;
            for event in self.events.iter() {
                self.rpc_server
                    .handle_event(&mut self.poller, event)
                    .or_fail()?;
                while let Some(request) = self.rpc_server.try_recv() {
                    todo!("{request:?}");
                }
            }
        }
    }
}
