//! File preview utilities for terminal UI components.
//!
//! This module provides functionality to create bordered file preview panes that display
//! file contents in a terminal interface. The previews support side-by-side layout with
//! automatic sizing and use Unicode box-drawing characters for borders.
use std::fmt::Write;
use std::path::PathBuf;

use crate::fmt::{horizontal_border, padding};
use crate::io_error;
use crate::terminal::{UnicodeTerminalFrame, str_cols};

/// Configuration for a file preview layout with optional left and right panes.
///
/// Specifies which files to display in a side-by-side preview arrangement.
/// Either pane can be omitted to display only a single file preview.
#[derive(Debug, Clone)]
pub struct FilePreviewSpec {
    /// Configuration for the left preview pane
    pub left_pane: Option<FilePreviewPaneSpec>,

    /// Configuration for the right preview pane
    pub right_pane: Option<FilePreviewPaneSpec>,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewSpec {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            left_pane: value.to_member("left-pane")?.map(TryFrom::try_from)?,
            right_pane: value.to_member("right-pane")?.map(TryFrom::try_from)?,
        })
    }
}

/// Configuration for a single file preview pane.
///
/// Specifies the file to display within a preview pane. The pane will
/// show the file's contents with a bordered layout including the filename.
#[derive(Debug, Clone)]
pub struct FilePreviewPaneSpec {
    /// Path to the file to display in this preview pane
    pub file: PathBuf,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewPaneSpec {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            file: value.to_member("file")?.required()?.try_into()?,
        })
    }
}

/// A dual-pane file preview component for terminal display.
///
/// Renders file contents in bordered panes with automatic layout management.
/// Supports side-by-side display of two files or single-file preview when
/// one pane is empty. Panes are positioned in the bottom third of the parent region.
#[derive(Debug)]
pub struct FilePreview {
    left_pane: FilePreviewPane,
    right_pane: FilePreviewPane,
}

impl FilePreview {
    /// Creates a new file preview from the given specification.
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
        })
    }

    /// Renders the file preview component to the terminal frame.
    ///
    /// Calculates optimal positioning for both panes and draws them with their
    /// content and borders. The preview is positioned in the bottom third of the frame.
    pub fn render(&mut self, frame: &mut UnicodeTerminalFrame) -> std::fmt::Result {
        self.calculate_pane_regions(frame.size().to_region());

        let (position, subframe) = self.render_left_pane()?;
        frame.draw(position, &subframe);

        let (position, subframe) = self.render_right_pane()?;
        frame.draw(position, &subframe);

        Ok(())
    }

    fn calculate_pane_regions(&mut self, region: tuinix::TerminalRegion) {
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

    fn render_left_pane(
        &self,
    ) -> Result<(tuinix::TerminalPosition, UnicodeTerminalFrame), std::fmt::Error> {
        let region = self.left_pane.region;
        let mut frame = UnicodeTerminalFrame::new(region.size);

        let cols = region.size.cols;
        let file_name = self.left_pane.file_name();
        writeln!(frame, "─{}┐", horizontal_border(file_name, cols - 2))?;

        for _ in 0..region.size.rows {
            write!(frame, "{}│", padding(' ', cols - 1))?;
        }

        let text_region = region.size.to_region().drop_top(1).drop_right(1);
        let mut text_frame = UnicodeTerminalFrame::new(text_region.size);
        self.left_pane.render_text(&mut text_frame)?;
        frame.draw(text_region.position, &text_frame);

        Ok((region.position, frame))
    }

    fn render_right_pane(
        &self,
    ) -> Result<(tuinix::TerminalPosition, UnicodeTerminalFrame), std::fmt::Error> {
        let region = self.right_pane.region;
        let mut frame = UnicodeTerminalFrame::new(region.size);

        let cols = region.size.cols;
        let file_name = self.right_pane.file_name();
        write!(frame, "┌{}─", horizontal_border(file_name, cols - 2))?;

        for _ in 0..region.size.rows {
            writeln!(frame, "│")?;
        }

        let text_region = region.size.to_region().drop_top(1).drop_left(1);
        let mut text_frame = UnicodeTerminalFrame::new(text_region.size);
        self.right_pane.render_text(&mut text_frame)?;
        frame.draw(text_region.position, &text_frame);

        Ok((region.position, frame))
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
            writeln!(frame, "{}", line.trim_end())?;
        }
        Ok(())
    }
}
