pub type UnicodeTerminalFrame = tuinix::TerminalFrame<UnicodeCharWidthEstimator>;

#[derive(Debug, Default, Clone, Copy)]
pub struct UnicodeCharWidthEstimator;

impl tuinix::EstimateCharWidth for UnicodeCharWidthEstimator {
    fn estimate_char_width(&self, c: char) -> usize {
        unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
    }
}
