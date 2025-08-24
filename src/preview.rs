use std::fmt::Write;
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
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewPaneSpec {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            file: value.to_member("fil")?.required()?.try_into()?,
        })
    }
}

#[derive(Debug)]
pub struct FilePreview {
    left_pane: FilePreviewPane,
    right_pane: FilePreviewPane,
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
                .unwrap_or_default(),
            right_pane: spec
                .right_pane
                .as_ref()
                .map(FilePreviewPane::new)
                .transpose()?
                .unwrap_or_default(),
            parent_region: tuinix::TerminalRegion::default(),
        })
    }

    pub fn set_parent_region(&mut self, region: tuinix::TerminalRegion) {
        self.parent_region = region;

        let pane_region = region.take_bottom(region.size.rows / 3);
        if self.left_pane.desired_cols() + self.right_pane.desired_cols() <= pane_region.size.cols {
            self.left_pane.region = pane_region
                .take_left(self.left_pane.desired_cols())
                .take_bottom(self.left_pane.desired_rows());
            self.right_pane.region = pane_region
                .take_right(self.right_pane.desired_cols())
                .take_bottom(self.right_pane.desired_rows());
        } else if self.right_pane.is_empty() {
            self.left_pane.region = pane_region.take_bottom(self.left_pane.desired_rows());
        } else if self.left_pane.is_empty() {
            self.right_pane.region = pane_region.take_bottom(self.right_pane.desired_rows());
        } else {
            self.left_pane.region = pane_region
                .take_left(pane_region.size.cols / 2)
                .take_bottom(self.left_pane.desired_rows());
            self.right_pane.region = pane_region
                .take_right(pane_region.size.cols / 2)
                .take_bottom(self.right_pane.desired_rows());
        }
    }

    pub fn render_left_pane(&self) -> (tuinix::TerminalPosition, UnicodeTerminalFrame) {
        let region = self.left_pane.region;
        let mut frame = UnicodeTerminalFrame::new(region.size);
        let file_name_cols = str_cols(self.left_pane.file_name());
        if file_name_cols + 4 <= region.size.cols {
            // TODO: align center
            write!(frame, "─ {} ", self.left_pane.file_name()).expect("infallible");
            for _ in file_name_cols + 2..region.size.cols - 2 {
                write!(frame, "─").expect("infallible");
            }
        } else {
            for _ in 0..region.size.cols - 1 {
                write!(frame, "─").expect("infallible");
            }
        }
        writeln!(frame, "┐").expect("infallible");

        for _ in 0..region.size.rows {
            for _ in 0..region.size.cols - 1 {
                write!(frame, " ").expect("infallible");
            }
            writeln!(frame, "│").expect("infallible");
        }

        let text_region = region.drop_top(1).drop_right(1);
        let mut text_frame = UnicodeTerminalFrame::new(text_region.size);
        self.left_pane
            .render_text(&mut text_frame)
            .expect("infallible");
        frame.draw(text_region.position, &text_frame);

        (region.position, frame)
    }

    pub fn render_right_pane(&self) -> (tuinix::TerminalPosition, UnicodeTerminalFrame) {
        let region = self.right_pane.region;
        let mut frame = UnicodeTerminalFrame::new(region.size);
        let file_name_cols = str_cols(self.right_pane.file_name());
        writeln!(frame, "┌").expect("infallible");
        if file_name_cols + 4 <= region.size.cols {
            // TODO: align center
            write!(frame, " {} ─", self.right_pane.file_name()).expect("infallible");
            for _ in file_name_cols + 4..region.size.cols {
                write!(frame, "─").expect("infallible");
            }
        } else {
            for _ in 1..region.size.cols {
                write!(frame, "─").expect("infallible");
            }
        }

        for _ in 0..region.size.rows {
            writeln!(frame, "│").expect("infallible");
        }

        let text_region = region.drop_top(1).drop_left(1);
        let mut text_frame = UnicodeTerminalFrame::new(text_region.size);
        self.right_pane
            .render_text(&mut text_frame)
            .expect("infallible");
        frame.draw(text_region.position, &text_frame);

        (region.position, frame)
    }
}

#[derive(Debug, Default)]
struct FilePreviewPane {
    path: PathBuf,
    text: String,
    max_rows: usize,
    max_cols: usize,
    region: tuinix::TerminalRegion,
}

impl FilePreviewPane {
    fn new(spec: &FilePreviewPaneSpec) -> std::io::Result<Self> {
        let content = if !spec.file.exists() {
            Vec::new()
        } else {
            std::fs::read(&spec.file).map_err(|e| {
                io_error(e, &format!("failed to read file '{}'", spec.file.display()))
            })?
        };
        let text = String::from_utf8_lossy(&content).into_owned();
        let max_rows = text.lines().count();
        let max_cols = text.lines().map(str_cols).max().unwrap_or_default();
        Ok(Self {
            path: spec.file.clone(),
            text,
            max_rows,
            max_cols,
            region: tuinix::TerminalRegion::default(),
        })
    }

    fn is_empty(&self) -> bool {
        self.max_rows == 0 || self.max_cols == 0
    }

    fn desired_rows(&self) -> usize {
        self.max_rows + 1
    }

    fn desired_cols(&self) -> usize {
        self.max_cols.max(str_cols(self.file_name()) + 3) + 1
    }

    fn file_name(&self) -> &str {
        self.path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
    }

    fn render_text(&self, frame: &mut UnicodeTerminalFrame) -> std::fmt::Result {
        for line in self.text.lines().take(frame.size().rows) {
            write!(frame, "{}", line.trim_end())?;
        }
        Ok(())
    }
}
