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

pub(crate) fn horizontal_border(text: &str, width: usize) -> impl std::fmt::Display {
    HorizontalBorder { text, width }
}

#[derive(Debug)]
struct HorizontalBorder<'a> {
    text: &'a str,
    width: usize,
}

impl<'a> std::fmt::Display for HorizontalBorder<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text_cols = str_cols(self.text);
        if 0 < text_cols && text_cols + 2 <= self.width {
            let padding_needed = self.width - text_cols - 2; // 2 for the spaces around text
            let left_padding = padding_needed / 2;
            let right_padding = padding_needed - left_padding;

            write!(f, "{}", padding('─', left_padding))?;
            write!(f, " {} ", self.text)?;
            write!(f, "{}", padding('─', right_padding))?;
        } else {
            write!(f, "{}", padding('─', self.width))?;
        }
        Ok(())
    }
}
