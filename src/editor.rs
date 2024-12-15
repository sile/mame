use std::{collections::BTreeMap, net::SocketAddr, path::PathBuf};

use jsonlrpc::{JsonRpcVersion, RequestId};
use jsonlrpc_mio::{ClientId, RpcServer};
use mio::{Events, Poll, Token};
use orfail::OrFail;
use ratatui::{
    prelude::{Buffer as RenderBuffer, Rect},
    text::Line,
    widgets::{self, Paragraph},
};
use serde::Serialize;

use crate::{
    buffer::{Buffer, BufferId},
    input::InputThread,
    rpc::{Caller, OpenReturnValue, Request, RpcError, RpcResult},
};

#[derive(Debug)]
pub struct Editor {
    poller: Poll,
    events: Events,
    rpc_server: RpcServer<Request>,
    exit: bool,
    buffers: BTreeMap<BufferId, Buffer>,
    current_buffer_id: Option<BufferId>,
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
            exit: false,
            buffers: BTreeMap::new(),
            current_buffer_id: None,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.rpc_server.listen_addr()
    }

    pub fn run(mut self) -> orfail::Result<()> {
        let mut terminal = ratatui::init();
        terminal.clear().or_fail()?;

        let input_thread_handle = InputThread::start(self.rpc_server.listen_addr()).or_fail()?;

        log::info!("Editor started: addr={}", self.rpc_server.listen_addr());

        while !self.exit {
            self.poller.poll(&mut self.events, None).or_fail()?;
            for event in self.events.iter() {
                self.rpc_server
                    .handle_event(&mut self.poller, event)
                    .or_fail()?;
            }
            while let Some((from, request)) = self.rpc_server.try_recv() {
                self.handle_request(from, request).or_fail()?;
            }

            if self.needs_redraw() {
                terminal
                    .draw(|frame| frame.render_widget(&self, frame.area()))
                    .or_fail()?;
            }

            if input_thread_handle.is_finished() {
                todo!();
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
            Request::Open { id, params, .. } => {
                let caller = Caller::new(from, id);
                let result = self.handle_open(params.path);
                self.reply(caller, result).or_fail()?;
            }
            Request::Exit { .. } => {
                self.exit = true;
            }
        }
        Ok(())
    }

    fn reply<T: Serialize>(&mut self, caller: Caller, result: RpcResult<T>) -> orfail::Result<()> {
        match result {
            Ok(result) => {
                #[derive(Serialize)]
                struct Response<T> {
                    jsonrpc: JsonRpcVersion,
                    id: RequestId,
                    result: T,
                }
                let response = Response {
                    jsonrpc: JsonRpcVersion::V2,
                    id: caller.request_id,
                    result,
                };
                self.rpc_server
                    .reply(&mut self.poller, caller.client_id, &response)
                    .or_fail()?;
            }
            Err(error) => {
                todo!("{error:?}");
            }
        }
        Ok(())
    }

    fn handle_open(&mut self, path: PathBuf) -> RpcResult<OpenReturnValue> {
        log::info!("Open file: {}", path.display());
        let new = !path.exists();
        let buffer = if new {
            Buffer::new(&path).map_err(|e| RpcError::file_error(path, e))?
        } else {
            Buffer::open_file(&path).map_err(|e| RpcError::file_error(path, e))?
        };

        // TODO: existence check
        log::info!("New buffer: {:?}", buffer.id);
        self.current_buffer_id = Some(buffer.id.clone());
        self.buffers.insert(buffer.id.clone(), buffer);

        Ok(OpenReturnValue { new })
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

    fn needs_redraw(&self) -> bool {
        if let Some(id) = &self.current_buffer_id {
            return self.buffers.get(id).is_some_and(|b| b.needs_redraw);
        }

        false
    }

    fn current_buffer(&self) -> Option<&Buffer> {
        self.current_buffer_id
            .as_ref()
            .and_then(|id| self.buffers.get(id))
    }
}

impl widgets::Widget for &Editor {
    fn render(self, area: Rect, render_buffer: &mut RenderBuffer) {
        let Some(buffer) = self.current_buffer() else {
            return;
        };

        let text = buffer
            .lines
            .iter()
            .cloned()
            .map(|line| Line::from(line))
            .collect::<Vec<_>>();

        Paragraph::new(text).render(area, render_buffer);
    }
}
