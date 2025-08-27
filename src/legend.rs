//! Legend rendering utilities for terminal UI components.
//!
//! This module provides functionality to create bordered legend boxes that display
//! lists of items (typically keybindings or help text) in a terminal interface.
//! The rendered legends use Unicode box-drawing characters and automatically
//! calculate proper sizing based on content width.
use std::fmt::Write;

use crate::fmt::centered;
use crate::terminal::{UnicodeTerminalFrame, str_cols};

/// Renders a bordered legend box containing a list of items with an optional title.
///
/// Creates a Unicode terminal frame with box-drawing characters that displays
/// the provided items in a vertical list. If a title is provided, it's centered
/// in the bottom border; otherwise, the border is a plain horizontal line.
pub fn render_legend<I, T>(title: &str, items: I) -> UnicodeTerminalFrame
where
    I: Iterator<Item = T>,
    T: std::fmt::Display,
{
    let items = items.map(|x| x.to_string()).collect::<Vec<_>>();
    let rows = items.len() + 1; // 1 = "─"
    let cols = std::iter::once(title.len() + 4) // 4 = "└ " + " ─"
        .chain(items.iter().map(|x| str_cols(x) + 2)) // 2 = "│ "
        .max()
        .expect("infallible");

    let mut frame = UnicodeTerminalFrame::new(tuinix::TerminalSize::rows_cols(rows, cols));

    for item in &items {
        writeln!(frame, "│ {item} ").expect("infallible");
    }

    let border_cols = cols - 1; // excluding the initial "└"
    writeln!(frame, "└{}", centered(title, '─', border_cols)).expect("infallible");

    frame
}
