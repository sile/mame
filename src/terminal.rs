use tuinix::KeyInput;

use crate::keymatcher::KeyMatcher;

/// Calculates the display width of a string in terminal columns.
///
/// This function uses Unicode width calculation to determine how many columns
/// a string will occupy when displayed in a terminal, properly handling:
/// - Wide characters (CJK characters, emojis) that take 2 columns
/// - Zero-width characters that don't take visual space
/// - Control characters
///
/// # Examples
///
/// ```
/// assert_eq!(mame::str_cols("Hello"), 5);
/// assert_eq!(mame::str_cols("こんにちは"), 10); // Japanese characters are 2 columns each
/// assert_eq!(mame::str_cols("café"), 4);
/// ```
pub fn str_cols(s: &str) -> usize {
    unicode_width::UnicodeWidthStr::width(s)
}

/// Calculates the display width of a character in terminal columns.
///
/// This function determines how many columns a single character will occupy
/// when displayed in a terminal. Returns 0 for characters that have no width
/// (like control characters or zero-width combining characters).
///
/// # Examples
///
/// ```
/// assert_eq!(mame::char_cols('A'), 1);
/// assert_eq!(mame::char_cols('あ'), 2); // Japanese character is 2 columns wide
/// assert_eq!(mame::char_cols('\u{0301}'), 0); // Combining acute accent has no width
/// ```
pub fn char_cols(c: char) -> usize {
    unicode_width::UnicodeWidthChar::width(c).unwrap_or(0)
}

pub fn display_key(key: KeyInput) -> impl std::fmt::Display {
    KeyMatcher::Literal(key)
}

/// A terminal frame that uses Unicode-aware character width estimation.
///
/// This is a type alias for [`tuinix::TerminalFrame`] configured with
/// [`UnicodeCharWidthEstimator`] to properly handle the display width
/// of Unicode characters, including wide characters like CJK text and emojis.
///
/// Use this when you need accurate terminal rendering for international
/// text content.
pub type UnicodeTerminalFrame = tuinix::TerminalFrame<UnicodeCharWidthEstimator>;

/// A character width estimator that uses Unicode width calculation.
///
/// This implementation of [`tuinix::EstimateCharWidth`] provides accurate
/// character width estimation using the Unicode standard, properly handling:
/// - ASCII characters (1 column)
/// - Wide characters like CJK and emojis (2 columns)
/// - Zero-width characters like combining marks (0 columns)
/// - Control characters (0 columns)
///
/// This estimator is more accurate than the default [`tuinix::FixedCharWidthEstimator`]
/// for applications that need to display international text content correctly.
///
/// # Examples
///
/// ```
/// use tuinix::{TerminalFrame, TerminalSize, EstimateCharWidth};
/// # use mame::UnicodeCharWidthEstimator;
///
/// let estimator = UnicodeCharWidthEstimator;
/// assert_eq!(estimator.estimate_char_width('A'), 1);
/// assert_eq!(estimator.estimate_char_width('漢'), 2);
///
/// // Use with a terminal frame
/// let size = TerminalSize::rows_cols(24, 80);
/// let frame = TerminalFrame::with_char_width_estimator(size, estimator);
/// ```
#[derive(Debug, Default, Clone, Copy)]
pub struct UnicodeCharWidthEstimator;

impl tuinix::EstimateCharWidth for UnicodeCharWidthEstimator {
    fn estimate_char_width(&self, c: char) -> usize {
        char_cols(c)
    }
}
