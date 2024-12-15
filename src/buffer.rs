use std::path::{Path, PathBuf};

#[derive(Debug)]
pub struct Buffer {
    pub path: Option<PathBuf>,
    pub lines: Vec<String>,
}

impl Buffer {
    pub fn from_file<P: AsRef<Path>>(path: P) -> orfail::Result<Self> {
        //
        todo!()
    }
}
