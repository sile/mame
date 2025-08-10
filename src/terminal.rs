pub fn str_cols(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

pub fn char_cols(c: char) -> usize {
    unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
}

pub type UnicodeTerminalFrame = tuinix::TerminalFrame<UnicodeCharWidthEstimator>;

#[derive(Debug, Default, Clone, Copy)]
pub struct UnicodeCharWidthEstimator;

impl tuinix::EstimateCharWidth for UnicodeCharWidthEstimator {
    fn estimate_char_width(&self, c: char) -> usize {
        char_cols(c)
    }
}
