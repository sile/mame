use std::{collections::BTreeMap, io::Write, net::SocketAddr, path::PathBuf};

use crossterm::{
    event::{KeyCode, KeyModifiers},
    style::{Color, ContentStyle, StyledContent},
    terminal::{EnterAlternateScreen, LeaveAlternateScreen},
};
use jsonlrpc::{JsonRpcVersion, RequestId};
use jsonlrpc_mio::{ClientId, RpcServer};
use mio::{Events, Poll, Token};
use orfail::OrFail;
use serde::Serialize;

use crate::{
    buffer::{Buffer, BufferId, CursorDelta, Position, Size},
    input::InputThread,
    key_mapper::KeyMapper,
    lsp::{LspClientManager, SemanticTokenType},
    rpc::{
        Caller, CommandParams, MoveDeltaParams, MoveDeltaReturnValue, MoveToParams,
        MoveToReturnValue, NotifyLspStartedParams, NotifySemanticTokensParams, OpenReturnValue,
        Request, RpcError, RpcResult, SaveParams, SaveReturnValue, StartLspParams,
        StartLspReturnValue,
    },
};

#[derive(Debug)]
pub struct Editor {
    poller: Poll,
    events: Events,
    rpc_server: RpcServer<Request>,
    lsp_client_manager: LspClientManager,
    exit: bool,
    buffers: BTreeMap<BufferId, Buffer>,
    current_buffer_id: Option<BufferId>, // TODO: non optional
    needs_redraw: bool,
    terminal_size: Size,
    key_mapper: KeyMapper,
    clipboard: Option<String>,
}

impl Editor {
    pub fn new(port: u16) -> orfail::Result<Self> {
        let mut poller = Poll::new().or_fail()?;
        let rpc_server = RpcServer::start(
            &mut poller,
            SocketAddr::from(([127, 0, 0, 1], port)),
            Token(0),
            Token(usize::MAX / 2 - 1),
        )
        .or_fail()?;

        let addr = rpc_server.listen_addr();
        Ok(Self {
            poller,
            events: Events::with_capacity(1024),
            rpc_server,
            lsp_client_manager: LspClientManager::new(
                addr,
                Token(usize::MAX / 2),
                Token(usize::MAX),
            ),
            exit: false,
            buffers: BTreeMap::new(),
            current_buffer_id: None,
            needs_redraw: true,
            terminal_size: Size::default(),
            key_mapper: KeyMapper::new(),
            clipboard: None,
        })
    }

    pub fn addr(&self) -> SocketAddr {
        self.rpc_server.listen_addr()
    }

    pub fn run(mut self) -> orfail::Result<()> {
        crossterm::execute!(std::io::stdout(), EnterAlternateScreen).or_fail()?;
        crossterm::terminal::enable_raw_mode().or_fail()?;

        self.terminal_size = crossterm::terminal::size()
            .map(|(width, height)| Size { width, height })
            .or_fail()?;

        let input_thread_handle = InputThread::start(self.rpc_server.listen_addr()).or_fail()?;

        log::info!("Editor started: addr={}", self.rpc_server.listen_addr());

        while !self.exit {
            self.poller.poll(&mut self.events, None).or_fail()?;
            for event in self.events.iter() {
                self.rpc_server
                    .handle_event(&mut self.poller, event)
                    .or_fail()?;
                if self
                    .lsp_client_manager
                    .handle_event(&mut self.poller, event)
                    .or_fail()?
                {
                    // TODO: optimize
                    // self.needs_redraw = true;
                }
            }
            while let Some((from, request)) = self.rpc_server.try_recv() {
                self.handle_request(from, request).or_fail()?;
            }

            if self.needs_redraw {
                self.render().or_fail()?;
                crossterm::execute!(
                    std::io::stdout(),
                    crossterm::cursor::MoveTo(self.cursor_position().x, self.cursor_position().y)
                )
                .or_fail()?;
                self.needs_redraw = false;
            }

            if input_thread_handle.is_finished() {
                todo!();
            }
        }

        log::info!("Editor exited: addr={}", self.rpc_server.listen_addr());
        crossterm::terminal::disable_raw_mode().or_fail()?;
        crossterm::execute!(std::io::stdout(), LeaveAlternateScreen).or_fail()?; // TODO: call in Drop

        Ok(())
    }

    fn handle_request(&mut self, from: ClientId, request: Request) -> orfail::Result<()> {
        log::debug!("Request: {request:?}");
        match request {
            Request::NotifyTerminalEvent { params, .. } => {
                self.handle_terminal_event(params.event).or_fail()?;
            }
            Request::Open { id, params, .. } => {
                let caller = Caller::new(from, id);
                let result = self.handle_open(params.path);
                self.reply(caller, result).or_fail()?;
            }
            Request::Save { id, params, .. } => {
                let caller = id.map(|id| Caller::new(from, id));
                let result = self.handle_save(params);
                caller
                    .map(|caller| self.reply(caller, result).or_fail())
                    .transpose()?;
            }
            Request::MoveTo { id, params, .. } => {
                let caller = id.map(|id| Caller::new(from, id));
                let result = self.handle_move_to(params);
                caller
                    .map(|caller| self.reply(caller, result).or_fail())
                    .transpose()?;
            }
            Request::MoveDelta { id, params, .. } => {
                let caller = id.map(|id| Caller::new(from, id));
                let result = self.handle_move_delta(params);
                caller
                    .map(|caller| self.reply(caller, result).or_fail())
                    .transpose()?;
            }
            Request::Exit { .. } => {
                self.exit = true;
            }
            Request::Cancel { .. } => {
                self.handle_cancel();
            }
            Request::StartLsp { id, params, .. } => {
                let caller = Caller::new(from, id);
                let result = self.handle_start_lsp(params);
                self.reply(caller, result).or_fail()?;
            }
            Request::NotifyLspStarted { params, .. } => {
                self.handle_notify_lsp_started(params).or_fail()?;
            }
            Request::NotifySemanticTokens { params, .. } => {
                self.handle_notify_semantic_tokens(params).or_fail()?;
            }
            Request::Mark { .. } => {
                self.handle_mark().or_fail()?;
            }
            Request::Copy { .. } => {
                self.handle_copy().or_fail()?;
            }
            Request::Cut { .. } => {
                self.handle_cut().or_fail()?;
            }
            Request::Paste { .. } => {
                self.handle_paste().or_fail()?;
            }
            Request::Command { params, .. } => {
                self.handle_command(params).or_fail()?;
            }
        }
        Ok(())
    }

    fn handle_command(&mut self, params: CommandParams) -> orfail::Result<()> {
        std::thread::spawn(move || {
            // TODO: error handling, etc
            let _ = std::process::Command::new(&params.path)
                .args(&params.args)
                .output();
        });
        Ok(())
    }

    fn handle_paste(&mut self) -> orfail::Result<()> {
        // TODO: remove clone()
        let Some(text) = self.clipboard.clone() else {
            return Ok(());
        };
        self.current_buffer_mut().or_fail()?.insert_text(&text);
        Ok(())
    }

    fn handle_copy(&mut self) -> orfail::Result<()> {
        let text = self.current_buffer().or_fail()?.marked_text();
        self.clipboard = text;
        self.current_buffer_mut().or_fail()?.mark_origin = None;
        Ok(())
    }

    fn handle_cut(&mut self) -> orfail::Result<()> {
        let text = self.current_buffer_mut().or_fail()?.take_marked_text();
        self.clipboard = text;
        self.current_buffer_mut().or_fail()?.mark_origin = None;
        Ok(())
    }

    fn handle_mark(&mut self) -> orfail::Result<()> {
        let buffer = self.current_buffer_mut().or_fail()?;
        buffer.mark();
        Ok(())
    }

    fn handle_cancel(&mut self) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.mark_origin = None;
        }
    }

    fn handle_save(&mut self, _params: SaveParams) -> RpcResult<SaveReturnValue> {
        self.current_buffer_mut()
            .map(|b| b.save())
            .transpose()
            .or_fail()?;
        Ok(SaveReturnValue {})
    }

    fn handle_move_to(&mut self, params: MoveToParams) -> RpcResult<MoveToReturnValue> {
        let terminal_size = self.terminal_size;
        let Some(buffer) = self.current_buffer_mut() else {
            return Ok(MoveToReturnValue {});
        };
        buffer.set_cursor(params.row, params.col, terminal_size);
        self.needs_redraw = true;
        Ok(MoveToReturnValue {})
    }

    fn handle_move_delta(&mut self, params: MoveDeltaParams) -> RpcResult<MoveDeltaReturnValue> {
        let terminal_size = self.terminal_size;
        let Some(buffer) = self.current_buffer_mut() else {
            return Ok(MoveDeltaReturnValue {}); // TODO: error
        };
        buffer.move_cursor(
            CursorDelta::xy(params.col as isize, params.row as isize),
            terminal_size,
        );
        self.needs_redraw = true;
        Ok(MoveDeltaReturnValue {})
    }

    fn handle_notify_semantic_tokens(
        &mut self,
        params: NotifySemanticTokensParams,
    ) -> orfail::Result<()> {
        let buffer = self.buffers.get_mut(&params.buffer_id).or_fail()?;
        buffer.set_semantic_tokens(&params.tokens);
        self.needs_redraw = true;
        Ok(())
    }

    fn handle_notify_lsp_started(&mut self, params: NotifyLspStartedParams) -> orfail::Result<()> {
        // TODO: check name
        for buffer in self.buffers.values_mut() {
            buffer.lsp_server_name = Some(params.name.clone());
        }

        for buffer in self.buffers.values() {
            // self.notify_did_open(buffer).or_fail()?;
            let Some(lsp) = self.lsp_client_manager.clients.get_mut(&params.name) else {
                continue;
            };
            lsp.notify_did_open(&mut self.poller, buffer).or_fail()?;
            lsp.request_semantic_tokens_full(&mut self.poller, buffer)
                .or_fail()?;
        }

        // TODO: semantic tokens

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

        // TODO: buffer existence check (skip reopening if exists)
        log::info!("New buffer: {:?}", buffer.id);
        self.current_buffer_id = Some(buffer.id.clone());
        self.buffers.insert(buffer.id.clone(), buffer);
        self.needs_redraw = true;

        Ok(OpenReturnValue { new })
    }

    fn handle_start_lsp(&mut self, params: StartLspParams) -> RpcResult<StartLspReturnValue> {
        log::info!("Start LSP server: {params:?}");
        self.lsp_client_manager.start(&mut self.poller, &params)?;
        Ok(StartLspReturnValue {})
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

    fn handle_key_event(&mut self, key: crossterm::event::KeyEvent) -> orfail::Result<()> {
        if key.kind != crossterm::event::KeyEventKind::Press {
            return Ok(());
        }
        log::debug!("key: {key:?}");

        let terminal_size = self.terminal_size;
        let Some(buffer) = self.current_buffer_mut() else {
            return Ok(());
        };

        // TODO: remove hard coding mappings
        match key.code {
            KeyCode::Up => {
                buffer.move_cursor(CursorDelta::xy(0, -1), terminal_size);
            }
            KeyCode::Down => {
                buffer.move_cursor(CursorDelta::xy(0, 1), terminal_size);
            }
            KeyCode::Right => {
                buffer.move_cursor(CursorDelta::xy(1, 0), terminal_size);
            }
            KeyCode::Left => {
                buffer.move_cursor(CursorDelta::xy(-1, 0), terminal_size);
            }
            KeyCode::Char(c)
                if !c.is_control()
                    && !key
                        .modifiers
                        .intersects(KeyModifiers::ALT | KeyModifiers::CONTROL) =>
            {
                buffer.insert_char(c);
            }
            KeyCode::Enter => {
                buffer.insert_newline();
            }
            KeyCode::Backspace => {
                buffer.backspace_char();
            }
            _ => {
                if let Some(request) = self.key_mapper.handle_input(&key) {
                    let dummy = ClientId::from(usize::MAX); // TODO
                    self.handle_request(dummy, request).or_fail()?;
                }
            }
        }

        self.needs_redraw = true; // TODO: optimize
        Ok(())
    }

    fn current_buffer(&self) -> Option<&Buffer> {
        self.current_buffer_id
            .as_ref()
            .and_then(|id| self.buffers.get(id))
    }

    fn current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.current_buffer_id
            .as_ref()
            .and_then(|id| self.buffers.get_mut(id))
    }

    fn cursor_position(&self) -> Position {
        self.current_buffer()
            .map(|b| b.cursor_position())
            .unwrap_or_default()
    }

    fn render(&mut self) -> orfail::Result<()> {
        // TODO: optimize
        let Some(buffer) = self.current_buffer() else {
            return Ok(());
        };

        // TODO: footer lines

        crossterm::queue!(
            std::io::stdout(),
            crossterm::cursor::RestorePosition,
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )
        .or_fail()?;
        for (y, row) in (buffer.start_line..)
            .take(self.terminal_size.height as usize)
            .enumerate()
        {
            crossterm::queue!(std::io::stdout(), crossterm::cursor::MoveTo(0, y as u16))
                .or_fail()?;
            for (ty, marked, text) in buffer.line_tokens(row).into_iter() {
                let bg_color = marked.then_some(Color::Blue);
                let fg_color = match ty {
                    None => None,
                    Some(ty) => match ty {
                        SemanticTokenType::Namespace => todo!(),
                        SemanticTokenType::Type => todo!(),
                        SemanticTokenType::Class => None,
                        SemanticTokenType::Enum => todo!(),
                        SemanticTokenType::Interface => todo!(),
                        SemanticTokenType::Struct => todo!(),
                        SemanticTokenType::TypeParameter => todo!(),
                        SemanticTokenType::Parameter => todo!(),
                        SemanticTokenType::Variable => Some(Color::Yellow),
                        SemanticTokenType::Property => todo!(),
                        SemanticTokenType::EnumMember => todo!(),
                        SemanticTokenType::Event => todo!(),
                        SemanticTokenType::Function => Some(Color::Rgb {
                            r: 0x50,
                            g: 0xD0,
                            b: 0x50,
                        }),
                        SemanticTokenType::Method => todo!(),
                        SemanticTokenType::Macro => Some(Color::Blue),
                        SemanticTokenType::Keyword => Some(Color::Magenta),
                        SemanticTokenType::Modifier => todo!(),
                        SemanticTokenType::Comment => Some(Color::Rgb {
                            r: 0xEF,
                            g: 0x75,
                            b: 0x21,
                        }),
                        SemanticTokenType::String => todo!(),
                        SemanticTokenType::Number => None,
                        SemanticTokenType::Regexp => todo!(),
                        SemanticTokenType::Operator => None,
                        SemanticTokenType::Decorator => todo!(),
                    },
                };
                let content = StyledContent::new(
                    ContentStyle {
                        foreground_color: fg_color,
                        background_color: bg_color,
                        ..Default::default()
                    },
                    text,
                );
                crossterm::queue!(
                    std::io::stdout(),
                    crossterm::style::PrintStyledContent(content)
                )
                .or_fail()?;
            }
        }
        crossterm::queue!(std::io::stdout(), crossterm::cursor::RestorePosition,).or_fail()?;
        std::io::stdout().flush().or_fail()?;

        Ok(())
    }
}
