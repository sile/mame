use std::{
    num::NonZeroUsize,
    path::{Path, PathBuf},
};

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
    pub start_line: NonZeroUsize,
    pub cursor: Cursor,
}

impl Buffer {
    pub fn new<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        let path = std::path::absolute(path)?;
        Ok(Self {
            id: BufferId::from_path(path),
            lines: Vec::new(),
            start_line: NonZeroUsize::MIN,
            cursor: Cursor::new(),
        })
    }

    pub fn open_file<P: AsRef<Path>>(path: P) -> std::io::Result<Self> {
        // TODO: note about canonicalize
        let path = std::path::absolute(path)?;
        let content = std::fs::read_to_string(&path)?;
        Ok(Self {
            id: BufferId::from_path(path),
            lines: content.lines().map(|l| l.to_owned()).collect(),
            start_line: NonZeroUsize::MIN,
            cursor: Cursor::new(),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Cursor {
    pub line: NonZeroUsize,
    pub column: NonZeroUsize,
}

impl Cursor {
    pub fn new() -> Self {
        Self {
            line: NonZeroUsize::MIN,
            column: NonZeroUsize::MIN,
        }
    }
}
