use std::{net::SocketAddr, path::PathBuf, thread::JoinHandle};

use jsonlrpc_mio::{ClientId, RpcServer};
use mio::{Events, Poll, Token};
use orfail::OrFail;
use ratatui::DefaultTerminal;

use crate::{
    input::InputThread,
    rpc::{Caller, Request},
};

#[derive(Debug)]
pub struct Editor {
    poller: Poll,
    events: Events,
    rpc_server: RpcServer<Request>,
    terminal: DefaultTerminal,
    input_thread_handle: JoinHandle<()>,
    exit: bool,
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
            exit: false,
        })
    }

    pub fn run(mut self) -> orfail::Result<()> {
        log::info!("Editor started: addr={}", self.rpc_server.listen_addr());

        while !self.exit {
            // TODO: handle key event
            self.poller.poll(&mut self.events, None).or_fail()?;
            for event in self.events.iter() {
                self.rpc_server
                    .handle_event(&mut self.poller, event)
                    .or_fail()?;
            }
            while let Some((from, request)) = self.rpc_server.try_recv() {
                self.handle_request(from, request).or_fail()?;
            }
        }

        log::info!("Editor exited: addr={}", self.rpc_server.listen_addr());
        Ok(())
    }

    fn handle_request(&mut self, from: ClientId, request: Request) -> orfail::Result<()> {
        match request {
            Request::NotifyTerminalEvent { params, .. } => {
                self.handle_terminal_event(params.event).or_fail()?;
            }
            Request::Open { id, params, .. } => self
                .handle_open(Caller::new(from, id), params.path)
                .or_fail()?,
            Request::Exit { .. } => {
                self.exit = true;
            }
        }
        Ok(())
    }

    fn handle_open(&mut self, caller: Caller, path: PathBuf) -> orfail::Result<()> {
        todo!()
    }

    fn handle_terminal_event(&mut self, event: crossterm::event::Event) -> orfail::Result<()> {
        match event {
            crossterm::event::Event::FocusGained => todo!(),
            crossterm::event::Event::FocusLost => todo!(),
            crossterm::event::Event::Key(key_event) => {
                self.handle_key_event(key_event).or_fail()?
            }
            crossterm::event::Event::Mouse(_mouse_event) => todo!(),
            crossterm::event::Event::Paste(_) => todo!(),
            crossterm::event::Event::Resize(_, _) => todo!(),
        }
        Ok(())
    }

    fn handle_key_event(&mut self, event: crossterm::event::KeyEvent) -> orfail::Result<()> {
        if event.kind != crossterm::event::KeyEventKind::Press {
            return Ok(());
        }
        log::info!("key: {event:?}");
        Ok(())
    }
}
