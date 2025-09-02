//! Legend rendering utilities for terminal UI components.
//!
//! This module provides functionality to create bordered legend boxes that display
//! lists of items (typically input bindings or help text) in a terminal interface.
//! The rendered legends use Unicode box-drawing characters and automatically
//! calculate proper sizing based on content width.
use std::fmt::Write;

use crate::fmt::horizontal_border;
use crate::terminal::UnicodeTerminalFrame;

/// A bordered legend box that displays a list of items with a title.
///
/// Renders as a Unicode box with vertical borders containing the items and
/// a bottom border with the centered title. Automatically sizes to fit content.
#[derive(Debug)]
pub struct Legend<'a> {
    title: &'a str,
    items: Vec<String>,
    size: tuinix::TerminalSize,
}

impl<'a> Legend<'a> {
    /// Creates a new legend with the given title and items.
    pub fn new<I>(title: &'a str, items: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        let items = items.collect::<Vec<_>>();
        let rows = items.len() + 1; // 1 = "─"
        let border_cols = if title.is_empty() {
            2 // 2 = "└─"
        } else {
            calculate_cols(title) + 4 // 4 = "└ " + " ─"
        };
        let cols = std::iter::once(border_cols)
            .chain(items.iter().map(|x| calculate_cols(x) + 1)) // 1 = "│"
            .max()
            .expect("infallible");
        let size = tuinix::TerminalSize::rows_cols(rows, cols);
        Self { title, items, size }
    }

    /// Renders the legend to the right edge of the frame if it fits.
    pub fn render(&self, frame: &mut UnicodeTerminalFrame) -> std::fmt::Result {
        let Some(position) = frame
            .size()
            .cols
            .checked_sub(self.size.cols)
            .map(tuinix::TerminalPosition::col)
            .filter(|_| self.size.rows < frame.size().rows)
        else {
            return Ok(());
        };

        let mut subframe = UnicodeTerminalFrame::new(self.size);
        for item in &self.items {
            writeln!(subframe, "│{item}")?;
        }
        writeln!(
            subframe,
            "└{}─",
            horizontal_border(self.title, self.size.cols - 2)
        )?;

        frame.draw(position, &subframe);

        Ok(())
    }

    /// Returns the size (rows and columns) required to render this legend.
    ///
    /// The size is calculated during construction based on the content width
    /// (longest item or title width) and the number of items plus the border row.
    pub fn size(&self) -> tuinix::TerminalSize {
        self.size
    }
}

fn calculate_cols(s: &str) -> usize {
    let mut frame = UnicodeTerminalFrame::new(tuinix::TerminalSize::rows_cols(1, usize::MAX));
    let _ = frame.write_str(s);
    frame.cursor().col
}
