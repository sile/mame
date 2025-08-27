//! Legend rendering utilities for terminal UI components.
//!
//! This module provides functionality to create bordered legend boxes that display
//! lists of items (typically keybindings or help text) in a terminal interface.
//! The rendered legends use Unicode box-drawing characters and automatically
//! calculate proper sizing based on content width.
use std::fmt::Write;

use crate::fmt::centered;
use crate::terminal::{UnicodeTerminalFrame, str_cols};

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
    pub fn new<I, T>(title: &'a str, items: I) -> Self
    where
        I: Iterator<Item = T>,
        T: std::fmt::Display,
    {
        Self {
            title,
            items: items.map(|x| x.to_string()).collect(),
        }
    }

    /// Renders the legend to the right edge of the frame if it fits.
    pub fn render(&self, frame: &mut UnicodeTerminalFrame) -> std::fmt::Result {
        let rows = self.items.len() + 1; // 1 = "─"
        let cols = std::iter::once(self.title.len() + 4) // 4 = "└ " + " ─"
            .chain(self.items.iter().map(|x| str_cols(x) + 2)) // 2 = "│ "
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
            writeln!(subframe, "│ {item}")?;
        }
        writeln!(subframe, "└{}─", centered(self.title, '─', cols - 2))?;

        frame.draw(position, &subframe);

        Ok(())
    }
}
