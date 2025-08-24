use std::num::NonZeroUsize;
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
    parent_region: tuinix::TerminalRegion,
}

impl FilePreview {
    pub fn new(spec: &FilePreviewSpec) -> std::io::Result<Self> {
        Ok(Self {
            left_pane: spec
                .left_pane
                .as_ref()
                .map(FilePreviewPane::new)
                .transpose()?
                .flatten(),
            right_pane: spec
                .right_pane
                .as_ref()
                .map(FilePreviewPane::new)
                .transpose()?
                .flatten(),
            parent_region: tuinix::TerminalRegion::default(),
        })
    }

    pub fn set_parent_region(&mut self, region: tuinix::TerminalRegion) {
        self.parent_region = region;

        let pane_region = region.take_bottom(region.size.rows / 3);
        match (&mut self.left_pane, &mut self.right_pane) {
            (None, None) => {}
            (Some(pane), None) => {
                pane.region = pane_region
                    .take_bottom(pane.desired_rows())
                    .take_left(pane.desired_cols());
            }
            (None, Some(pane)) => {
                pane.region = pane_region
                    .take_bottom(pane.desired_rows())
                    .take_right(pane.desired_cols());
            }
            (Some(left_pane), Some(right_pane)) => {}
        }
    }

    pub fn render_left_pane(&self) -> (tuinix::TerminalPosition, UnicodeTerminalFrame) {
        todo!()
    }

    pub fn render_right_pane(&self) -> (tuinix::TerminalPosition, UnicodeTerminalFrame) {
        todo!()
    }
}

#[derive(Debug)]
struct FilePreviewPane {
    path: PathBuf,
    text: String,
    max_rows: NonZeroUsize,
    max_cols: NonZeroUsize,
    region: tuinix::TerminalRegion,
}

impl FilePreviewPane {
    fn new(spec: &FilePreviewPaneSpec) -> std::io::Result<Option<Self>> {
        let content = std::fs::read(&spec.file)
            .map_err(|e| io_error(e, &format!("failed to read file '{}'", spec.file.display())))?;
        let text = String::from_utf8_lossy(&content).into_owned();
        let max_rows = text.lines().count();
        let max_cols = text.lines().map(str_cols).max().unwrap_or_default();
        if let Some((max_rows, max_cols)) =
            NonZeroUsize::new(max_rows).zip(NonZeroUsize::new(max_cols))
        {
            Ok(Some(Self {
                path: spec.file.clone(),
                text,
                max_rows,
                max_cols,
                region: tuinix::TerminalRegion::default(),
            }))
        } else {
            Ok(None)
        }
    }

    fn desired_rows(&self) -> usize {
        self.max_rows.get() + 1
    }

    fn desired_cols(&self) -> usize {
        self.max_cols.get().max(str_cols(self.file_name()) + 3) + 1
    }

    fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
    }
}
