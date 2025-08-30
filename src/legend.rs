//! Legend rendering utilities for terminal UI components.
//!
//! This module provides functionality to create bordered legend boxes that display
//! lists of items (typically keybindings or help text) in a terminal interface.
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
}

impl<'a> Legend<'a> {
    /// Creates a new legend with the given title and items.
    pub fn new<I>(title: &'a str, items: I) -> Self
    where
        I: Iterator<Item = String>,
    {
        Self {
            title,
            items: items.collect(),
        }
    }

    /// Renders the legend to the right edge of the frame if it fits.
    pub fn render(&self, frame: &mut UnicodeTerminalFrame) -> std::fmt::Result {
        let max_cols = frame.size().cols;
        let rows = self.items.len() + 1; // 1 = "─"
        let cols = std::iter::once(self.title.len() + 4) // 4 = "└ " + " ─"
            .chain(self.items.iter().map(|x| calculate_cols(x, max_cols) + 1)) // 1 = "│"
            .max()
            .expect("infallible");
        let Some(position) = frame
            .size()
            .cols
            .checked_sub(cols)
            .map(tuinix::TerminalPosition::col)
            .filter(|_| rows < frame.size().rows)
        else {
            return Ok(());
        };

        let mut subframe = UnicodeTerminalFrame::new(tuinix::TerminalSize::rows_cols(rows, cols));
        for item in &self.items {
            writeln!(subframe, "│{item}")?;
        }
        writeln!(subframe, "└{}─", horizontal_border(self.title, cols - 2))?;

        frame.draw(position, &subframe);

        Ok(())
    }
}

fn calculate_cols(s: &str, max_cols: usize) -> usize {
    let mut frame = UnicodeTerminalFrame::new(tuinix::TerminalSize::rows_cols(1, max_cols));
    let _ = frame.write_str(s);
    frame.cursor().col
}
