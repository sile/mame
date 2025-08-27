//! Formatting utilities for terminal UI display elements.
use crate::terminal::str_cols;

/// Creates a displayable representation of a key input.
pub fn key(key: tuinix::KeyInput) -> impl std::fmt::Display {
    crate::keymatcher::KeyMatcher::Literal(key)
}

/// Creates a displayable padding string with the specified character repeated a given number of times.
pub fn padding(ch: char, count: usize) -> impl std::fmt::Display {
    Padding { ch, count }
}

/// Creates a centered text with padding characters on both sides to fit within the specified width.
///
/// The text is surrounded by spaces and padded with the specified character to fill the total width.
/// If the text (plus surrounding spaces) is too long to fit, returns padding characters for the full width.
pub fn centered(text: &str, padding_ch: char, width: usize) -> impl std::fmt::Display + '_ {
    Centered {
        text,
        padding_ch,
        width,
    }
}

#[derive(Debug)]
struct Padding {
    ch: char,
    count: usize,
}

impl std::fmt::Display for Padding {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for _ in 0..self.count {
            write!(f, "{}", self.ch)?;
        }
        Ok(())
    }
}

#[derive(Debug)]
struct Centered<'a> {
    text: &'a str,
    padding_ch: char,
    width: usize,
}

impl<'a> std::fmt::Display for Centered<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text_cols = str_cols(self.text);

        if text_cols + 4 <= self.width {
            let padding_needed = self.width - text_cols - 2; // 2 for the spaces around text
            let left_padding = padding_needed / 2;
            let right_padding = padding_needed - left_padding;

            write!(f, "{}", padding(self.padding_ch, left_padding))?;
            write!(f, " {} ", self.text)?;
            write!(f, "{}", padding(self.padding_ch, right_padding))?;
        } else {
            write!(f, "{}", padding(self.padding_ch, self.width))?;
        }

        Ok(())
    }
}
