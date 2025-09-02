use tuinix::{KeyCode, KeyInput};

/// Matches keyboard input against specific patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputMatcher {
    /// Matches an exact key combination
    Key(KeyInput),

    /// Matches any printable character
    PrintableKey,

    /// Matches any key input
    AnyKey,
}

impl InputMatcher {
    /// Returns `true` if the given key input matches this matcher pattern.
    pub fn matches(self, input: tuinix::TerminalInput) -> bool {
        match input {
            tuinix::TerminalInput::Key(key) => match self {
                InputMatcher::Key(k) => k == key,
                InputMatcher::PrintableKey => {
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
                InputMatcher::AnyKey => true,
            },
            _ => todo!(),
        }
    }
}

impl std::str::FromStr for InputMatcher {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "<PRINTABLE_KEY>" {
            return Ok(InputMatcher::PrintableKey);
        }

        if s == "<ANY_KEY>" {
            return Ok(InputMatcher::AnyKey);
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
            "<UP>" => return Ok(InputMatcher::Key(key(KeyCode::Up))),
            "<DOWN>" => return Ok(InputMatcher::Key(key(KeyCode::Down))),
            "<LEFT>" => return Ok(InputMatcher::Key(key(KeyCode::Left))),
            "<RIGHT>" => return Ok(InputMatcher::Key(key(KeyCode::Right))),
            "<ENTER>" => return Ok(InputMatcher::Key(key(KeyCode::Enter))),
            "<ESCAPE>" => return Ok(InputMatcher::Key(key(KeyCode::Escape))),
            "<BACKSPACE>" => return Ok(InputMatcher::Key(key(KeyCode::Backspace))),
            "<TAB>" => return Ok(InputMatcher::Key(key(KeyCode::Tab))),
            "<BACKTAB>" => return Ok(InputMatcher::Key(key(KeyCode::BackTab))),
            "<DELETE>" => return Ok(InputMatcher::Key(key(KeyCode::Delete))),
            "<INSERT>" => return Ok(InputMatcher::Key(key(KeyCode::Insert))),
            "<HOME>" => return Ok(InputMatcher::Key(key(KeyCode::Home))),
            "<END>" => return Ok(InputMatcher::Key(key(KeyCode::End))),
            "<PAGEUP>" => return Ok(InputMatcher::Key(key(KeyCode::PageUp))),
            "<PAGEDOWN>" => return Ok(InputMatcher::Key(key(KeyCode::PageDown))),
            _ => {}
        }

        // Handle character input
        let mut chars = remaining.chars();
        if let Some(ch) = chars.next()
            && chars.next().is_none()
        {
            let code = KeyCode::Char(ch);
            Ok(InputMatcher::Key(key(code)))
        } else if let Some(hex_str) = remaining.strip_prefix("0x") {
            // Handle hex notation for control chars such as 0x7f
            match u32::from_str_radix(hex_str, 16) {
                Ok(code_point) => {
                    if let Some(ch) = char::from_u32(code_point) {
                        let code = KeyCode::Char(ch);
                        Ok(InputMatcher::Key(key(code)))
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

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for InputMatcher {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        value
            .to_unquoted_string_str()?
            .parse()
            .map_err(|e| value.invalid(e))
    }
}

impl std::fmt::Display for InputMatcher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PrintableKey => write!(f, "<PRINTABLE_KEY>"),
            Self::AnyKey => write!(f, "<ANY_KEY>"),
            Self::Key(key) => {
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

impl nojson::DisplayJson for InputMatcher {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.string(self)
    }
}
