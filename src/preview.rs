use std::path::PathBuf;

use crate::io_error;
use crate::{UnicodeTerminalFrame, str_cols};

#[derive(Debug, Clone)]
pub struct FilePreviewSpec {
    pub left_pane: Option<FilePreviewPaneSpec>,
    pub right_pane: Option<FilePreviewPaneSpec>,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewSpec {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            left_pane: value.to_member("left_pane")?.map(TryFrom::try_from)?,
            right_pane: value.to_member("right_pane")?.map(TryFrom::try_from)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FilePreviewPaneSpec {
    pub file: PathBuf,
    pub skip_if_empty: bool,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewPaneSpec {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            file: value.to_member("fil")?.required()?.try_into()?,
            skip_if_empty: value
                .to_member("skip_if_empty")?
                .map(bool::try_from)?
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug)]
pub struct FilePreview {
    left_pane: Option<FilePreviewPane>,
    right_pane: Option<FilePreviewPane>,
}

impl FilePreview {
    pub fn new(spec: &FilePreviewSpec) -> std::io::Result<Self> {
        Ok(Self {
            left_pane: spec
                .left_pane
                .as_ref()
                .map(FilePreviewPane::new)
                .transpose()?,
            right_pane: spec
                .right_pane
                .as_ref()
                .map(FilePreviewPane::new)
                .transpose()?,
        })
    }

    pub fn render_left_pane(
        &self,
        mut region: tuinix::TerminalRegion,
    ) -> (tuinix::TerminalPosition, UnicodeTerminalFrame) {
        region.size.rows /= 3;
        todo!()
    }

    pub fn render_right_pane(
        &self,
        mut region: tuinix::TerminalRegion,
    ) -> (tuinix::TerminalPosition, UnicodeTerminalFrame) {
        region.size.rows /= 3;
        todo!()
    }
}

#[derive(Debug)]
struct FilePreviewPane {
    path: PathBuf,
    text: String,
}

impl FilePreviewPane {
    fn new(spec: &FilePreviewPaneSpec) -> std::io::Result<Self> {
        let content = std::fs::read(&spec.file)
            .map_err(|e| io_error(e, &format!("failed to read file '{}'", spec.file.display())))?;
        Ok(Self {
            path: spec.file.clone(),
            text: String::from_utf8_lossy(&content).into_owned(),
        })
    }
}
