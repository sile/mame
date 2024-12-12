use std::net::SocketAddr;

use jsonlrpc_mio::RpcServer;
use mio::{Events, Poll, Token};
use orfail::OrFail;

#[derive(Debug)]
pub struct Editor {
    poller: Poll,
    events: Events,
    rpc_server: RpcServer,
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
        Ok(Self {
            poller,
            events: Events::with_capacity(1024),
            rpc_server,
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
