use std::path::{Path, PathBuf};

use ratatui::layout::{Position, Size};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
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

#[derive(Debug)]
pub struct Buffer {
    pub id: BufferId,
    pub lines: Vec<String>,
    pub start_line: usize,
    pub cursor: Cursor,
}

impl Buffer {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = std::path::absolute(path)?;
        Ok(Self {
            id: BufferId::from_path(path),
            lines: Vec::new(),
            start_line: 0,
            cursor: Cursor::default(),
        })
    }

    pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        // TODO: note about canonicalize
        let path = std::path::absolute(path)?;
        let content = std::fs::read_to_string(&path)?;
        Ok(Self {
            id: BufferId::from_path(path),
            lines: content.lines().map(|l| l.to_owned()).collect(),
            start_line: 0,
            cursor: Cursor::default(),
        })
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
