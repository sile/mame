use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

pub fn str_cols(s: &str) -> usize {
    s.width()
}

pub fn char_cols(c: char) -> usize {
    c.width().unwrap_or(0)
}

pub type UnicodeTerminalFrame = tuinix::TerminalFrame<UnicodeCharWidthEstimator>;

#[derive(Debug, Default, Clone, Copy)]
pub struct UnicodeCharWidthEstimator;

impl tuinix::EstimateCharWidth for UnicodeCharWidthEstimator {
    fn estimate_char_width(&self, c: char) -> usize {
        char_cols(c)
    }
}
