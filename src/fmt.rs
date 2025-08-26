//! Formatting utilities for terminal UI display elements.

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
