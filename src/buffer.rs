use std::path::{Path, PathBuf};

use orfail::OrFail;
use ratatui::layout::{Position, Size};
use serde::{Deserialize, Serialize};

use crate::{lsp::SemanticTokenType, rpc::SemanticToken};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct BufferId {
    pub name: String,
    pub path: PathBuf,
}

impl BufferId {
    pub fn from_path(path: PathBuf) -> Self {
        Self {
            name: path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("*scratch")
                .to_owned(),
            path,
        }
    }
}

#[derive(
    Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize,
)]
pub struct BufferPosition {
    pub row: usize,
    pub col: usize,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BufferRegion {
    pub start: BufferPosition,
    pub end: BufferPosition,
}

impl BufferRegion {
    fn new(p0: BufferPosition, p1: BufferPosition) -> Self {
        Self {
            start: p0.min(p1),
            end: p0.max(p1),
        }
    }
}

#[derive(Debug)]
pub struct Buffer {
    pub id: BufferId,
    pub lines: Vec<String>,
    pub start_line: usize,
    pub cursor: Cursor,
    pub lsp_server_name: Option<String>,
    pub version: u64,
    pub semantic_tokens: Vec<SemanticToken>,
    pub mark_origin: Option<BufferPosition>,
}

impl Buffer {
    pub fn save(&mut self) -> orfail::Result<()> {
        let content = self.lines.join("\n");
        std::fs::write(&self.id.path, &content).or_fail()
    }

    pub fn marked_region(&self) -> BufferRegion {
        let Some(start) = self.mark_origin else {
            return BufferRegion::default();
        };
        BufferRegion::new(start, self.cursor_buffer_position())
    }

    pub fn line_tokens(&self, linenum: usize) -> Vec<(Option<SemanticTokenType>, &str)> {
        let mut tokens = Vec::new();

        let line = self.lines.get(linenum).map(|s| s.as_str()).unwrap_or("");
        // let Ok(i) = self
        //     .semantic_tokens
        //     .binary_search_by_key(&linenum, |x| x.line)
        // else {
        //     return vec![(None, line)];
        // };

        let mut offset = 0;
        //for token in &self.semantic_tokens[i..] {
        // TODO: optimize
        for token in &self.semantic_tokens {
            if token.line < linenum {
                continue;
            }

            if token.line != linenum {
                tokens.push((None, &line[offset..]));
                break;
            }

            if offset < token.column {
                tokens.push((None, &line[offset..token.column]));
                offset = token.column;
            }
            tokens.push((
                Some(token.token_type),
                &line[token.column..][..token.token_len],
            ));
            offset += token.token_len;
        }

        tokens
    }

    pub fn text(&self) -> String {
        self.lines.join("\n")
    }

    pub fn set_semantic_tokens(&mut self, tokens: &[SemanticToken]) {
        self.semantic_tokens = tokens.to_owned();
    }

    pub fn mark(&mut self) {
        self.mark_origin = Some(self.cursor_buffer_position());
    }

    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        Self::open_file(path)
    }

    pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        // TODO: note about canonicalize
        let path = std::path::absolute(path)?;
        let content = if path.exists() {
            std::fs::read_to_string(&path)?
        } else {
            String::new()
        };
        Ok(Self {
            id: BufferId::from_path(path),
            lines: content.lines().map(|l| l.to_owned()).collect(),
            start_line: 0,
            cursor: Cursor::default(),
            lsp_server_name: None,
            version: 0,
            semantic_tokens: Vec::new(),
            mark_origin: None,
        })
    }

    pub fn set_cursor(&mut self, row: Option<u32>, col: Option<u32>, terminal_size: Size) {
        if let Some(row) = row {
            self.cursor.line = self.lines.len().min(row as usize);
            if self.cursor.line - self.start_line > terminal_size.height as usize {
                self.start_line = self.cursor.line - terminal_size.height as usize / 2;
            }
        }
        if let Some(col) = col {
            self.cursor.column = self
                .lines
                .get(self.cursor.line)
                .map(|s| s.len())
                .unwrap_or_default()
                .min(col as usize);
        }
    }

    pub fn move_cursor(&mut self, delta: CursorDelta, terminal_size: Size) {
        self.cursor.line = self
            .cursor
            .line
            .saturating_add_signed(delta.y)
            .min(self.lines.len().saturating_sub(1));

        if self.cursor.line < self.start_line {
            self.start_line = self.start_line.saturating_sub(1);
        }
        if self.cursor.line - self.start_line > terminal_size.height as usize {
            self.start_line = self.start_line.saturating_add(1);
        }

        // TODO: consider multi byte char (e.g., unicode-width crate)
        // TODO: handle line wrapping
        self.cursor.column = self.cursor.column.saturating_add_signed(delta.x).min(
            self.lines
                .get(self.cursor.line)
                //.map(|l| l.chars().count().saturating_sub(1))
                .map(|l| l.len())
                .unwrap_or_default(),
        );
    }

    pub fn cursor_buffer_position(&self) -> BufferPosition {
        BufferPosition {
            col: self.cursor.column,
            row: self.cursor.line,
        }
    }

    // TODI
    pub fn cursor_position(&self) -> Position {
        Position {
            x: self.cursor.column as u16,
            y: (self.cursor.line.saturating_sub(self.start_line)) as u16,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        let Some(line) = self.lines.get_mut(self.cursor.line) else {
            return;
        };

        // TODO: consider multi byte
        line.insert(self.cursor.column, c);
        self.cursor.column += 1;
    }

    pub fn insert_newline(&mut self) {
        let Some(line) = self.lines.get_mut(self.cursor.line) else {
            return;
        };

        // TODO: consider multi byte
        let new = line.split_off(self.cursor.column);
        self.cursor.line += 1;
        self.cursor.column = 0;
        self.lines.insert(self.cursor.line, new);
    }

    pub fn backspace_char(&mut self) {
        let Some(line) = self.lines.get_mut(self.cursor.line) else {
            return;
        };

        // TODO: consider multi byte
        if let Some(column) = self.cursor.column.checked_sub(1) {
            self.cursor.column = column;
            line.remove(self.cursor.column);
        } else if line.is_empty() {
            self.lines.remove(self.cursor.line);
            self.cursor.line = self.cursor.line.saturating_sub(1);
            self.cursor.column = self
                .lines
                .get(self.cursor.line)
                .map(|l| l.len())
                .unwrap_or_default(); //TODO: multi byte
        } else if let Some(line) = self.cursor.line.checked_sub(1) {
            self.cursor.line = line;
            self.cursor.column = self.lines[line].len(); // TODO: multi byte
            self.backspace_char();
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct CursorDelta {
    pub x: isize,
    pub y: isize,
}

impl CursorDelta {
    pub fn xy(x: isize, y: isize) -> Self {
        Self { x, y }
    }
}
