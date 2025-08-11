use tuinix::{KeyCode, KeyInput};

use crate::VecMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum KeyMatcher {
    Literal(KeyInput),
    PrintableChar,
}

impl KeyMatcher {
    pub fn matches(self, key: KeyInput) -> bool {
        match self {
            KeyMatcher::Literal(k) => k == key,
            KeyMatcher::PrintableChar => {
                if let KeyInput {
                    ctrl: false,
                    alt: false,
                    code: KeyCode::Char(ch),
                } = key
                {
                    !ch.is_control()
                } else {
                    false
                }
            }
        }
    }
}

impl std::str::FromStr for KeyMatcher {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "<PRINTABLE>" {
            return Ok(KeyMatcher::PrintableChar);
        }

        // Handle modifier key combinations like "C-c", "M-x"
        let mut alt = false;
        let mut ctrl = false;
        let mut remaining = s;

        loop {
            if let Some(rest) = remaining.strip_prefix("M-")
                && !alt
            {
                remaining = rest;
                alt = true;
            } else if let Some(rest) = remaining.strip_prefix("C-")
                && !ctrl
            {
                remaining = rest;
                ctrl = true;
            } else {
                break;
            }
        }

        // Handle special keys in angle brackets
        let key = |code| KeyInput { ctrl, alt, code };
        match remaining {
            "<UP>" => return Ok(KeyMatcher::Literal(key(KeyCode::Up))),
            "<DOWN>" => return Ok(KeyMatcher::Literal(key(KeyCode::Down))),
            "<LEFT>" => return Ok(KeyMatcher::Literal(key(KeyCode::Left))),
            "<RIGHT>" => return Ok(KeyMatcher::Literal(key(KeyCode::Right))),
            "<ENTER>" => return Ok(KeyMatcher::Literal(key(KeyCode::Enter))),
            "<ESCAPE>" => return Ok(KeyMatcher::Literal(key(KeyCode::Escape))),
            "<BACKSPACE>" => return Ok(KeyMatcher::Literal(key(KeyCode::Backspace))),
            "<TAB>" => return Ok(KeyMatcher::Literal(key(KeyCode::Tab))),
            "<BACKTAB>" => return Ok(KeyMatcher::Literal(key(KeyCode::BackTab))),
            "<DELETE>" => return Ok(KeyMatcher::Literal(key(KeyCode::Delete))),
            "<INSERT>" => return Ok(KeyMatcher::Literal(key(KeyCode::Insert))),
            "<HOME>" => return Ok(KeyMatcher::Literal(key(KeyCode::Home))),
            "<END>" => return Ok(KeyMatcher::Literal(key(KeyCode::End))),
            "<PAGEUP>" => return Ok(KeyMatcher::Literal(key(KeyCode::PageUp))),
            "<PAGEDOWN>" => return Ok(KeyMatcher::Literal(key(KeyCode::PageDown))),
            _ => {}
        }

        // Handle character input
        let mut chars = remaining.chars();
        if let Some(ch) = chars.next()
            && chars.next().is_none()
        {
            let code = KeyCode::Char(ch);
            Ok(KeyMatcher::Literal(key(code)))
        } else if let Some(hex_str) = remaining.strip_prefix("0x") {
            // Handle hex notation for control chars such as 0x7f
            match u32::from_str_radix(hex_str, 16) {
                Ok(code_point) => {
                    if let Some(ch) = char::from_u32(code_point) {
                        let code = KeyCode::Char(ch);
                        Ok(KeyMatcher::Literal(key(code)))
                    } else {
                        Err(format!("invalid Unicode code point: 0x{:x}", code_point))
                    }
                }
                Err(_) => Err(format!("invalid hex notation: {}", remaining)),
            }
        } else {
            Err(format!("invalid key input format: {s:?}"))
        }
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for KeyMatcher {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        value
            .to_unquoted_string_str()?
            .parse()
            .map_err(|e| value.invalid(e))
    }
}

impl std::fmt::Display for KeyMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PrintableChar => write!(f, "<PRINTABLE>"),
            Self::Literal(key) => {
                if key.alt {
                    write!(f, "M-")?;
                }
                if key.ctrl {
                    write!(f, "C-")?;
                }

                match key.code {
                    KeyCode::Up => write!(f, "<UP>"),
                    KeyCode::Down => write!(f, "<DOWN>"),
                    KeyCode::Left => write!(f, "<LEFT>"),
                    KeyCode::Right => write!(f, "<RIGHT>"),
                    KeyCode::Enter => write!(f, "<ENTER>"),
                    KeyCode::Escape => write!(f, "<ESCAPE>"),
                    KeyCode::Backspace => write!(f, "<BACKSPACE>"),
                    KeyCode::Tab => write!(f, "<TAB>"),
                    KeyCode::BackTab => write!(f, "<BACK_TAB>"),
                    KeyCode::Delete => write!(f, "<DELETE>"),
                    KeyCode::Insert => write!(f, "<INSERT>"),
                    KeyCode::Home => write!(f, "<HOME>"),
                    KeyCode::End => write!(f, "<END>"),
                    KeyCode::PageUp => write!(f, "<PAGEUP>"),
                    KeyCode::PageDown => write!(f, "<PAGEDOWN>"),
                    KeyCode::Char(ch) if ch.is_control() => write!(f, "0x{:x}", ch as u32),
                    KeyCode::Char(ch) => write!(f, "{ch}"),
                }
            }
        }
    }
}

impl nojson::DisplayJson for KeyMatcher {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.string(self)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Hash)]
pub struct KeyLabels(VecMap<KeyMatcher, String>);

impl KeyLabels {
    pub fn get_label(&self, k: KeyMatcher) -> String {
        self.0.get(&k).cloned().unwrap_or_else(|| k.to_string())
    }
}

impl nojson::DisplayJson for KeyLabels {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for KeyLabels {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(KeyLabels(value.try_into()?))
    }
}
