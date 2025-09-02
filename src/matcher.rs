use tuinix::{KeyCode, KeyInput, MouseEvent};

/// Matches terminal input (keyboard and mouse) against specific patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InputMatcher {
    /// Matches an exact key combination
    Key(KeyInput),

    /// Matches any printable character
    PrintableKey,

    /// Matches any key input
    AnyKey,

    /// Matches a specific mouse event
    Mouse(MouseEvent),
}

impl InputMatcher {
    /// Returns `true` if the given terminal input matches this matcher pattern.
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
                _ => false,
            },
            tuinix::TerminalInput::Mouse(m) => {
                matches!(self, InputMatcher::Mouse(e) if e == m.event)
            }
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

        // Handle special keys and mouse events in angle brackets
        let key = |code| InputMatcher::Key(KeyInput { ctrl, alt, code });
        let mouse = |event| InputMatcher::Mouse(event);
        match remaining {
            "<UP>" => return Ok(key(KeyCode::Up)),
            "<DOWN>" => return Ok(key(KeyCode::Down)),
            "<LEFT>" => return Ok(key(KeyCode::Left)),
            "<RIGHT>" => return Ok(key(KeyCode::Right)),
            "<ENTER>" => return Ok(key(KeyCode::Enter)),
            "<ESCAPE>" => return Ok(key(KeyCode::Escape)),
            "<BACKSPACE>" => return Ok(key(KeyCode::Backspace)),
            "<TAB>" => return Ok(key(KeyCode::Tab)),
            "<BACKTAB>" => return Ok(key(KeyCode::BackTab)),
            "<DELETE>" => return Ok(key(KeyCode::Delete)),
            "<INSERT>" => return Ok(key(KeyCode::Insert)),
            "<HOME>" => return Ok(key(KeyCode::Home)),
            "<END>" => return Ok(key(KeyCode::End)),
            "<PAGEUP>" => return Ok(key(KeyCode::PageUp)),
            "<PAGEDOWN>" => return Ok(key(KeyCode::PageDown)),
            "<LEFT_BUTTON_PRESS>" => return Ok(mouse(MouseEvent::LeftPress)),
            "<LEFT_BUTTON_RELEASE>" => return Ok(mouse(MouseEvent::LeftRelease)),
            "<RIGHT_BUTTON_PRESS>" => return Ok(mouse(MouseEvent::RightPress)),
            "<RIGHT_BUTTON_RELEASE>" => return Ok(mouse(MouseEvent::RightRelease)),
            "<MIDDLE_BUTTON_PRESS>" => return Ok(mouse(MouseEvent::MiddlePress)),
            "<MIDDLE_BUTTON_RELEASE>" => return Ok(mouse(MouseEvent::MiddleRelease)),
            "<DRAG>" => return Ok(mouse(MouseEvent::Drag)),
            "<WHEEL_UP>" => return Ok(mouse(MouseEvent::ScrollUp)),
            "<WHEEL_DOWN>" => return Ok(mouse(MouseEvent::ScrollDown)),
            _ => {}
        }

        // Handle character input
        let mut chars = remaining.chars();
        if let Some(ch) = chars.next()
            && chars.next().is_none()
        {
            let code = KeyCode::Char(ch);
            Ok(key(code))
        } else if let Some(hex_str) = remaining.strip_prefix("0x") {
            // Handle hex notation for control chars such as 0x7f
            match u32::from_str_radix(hex_str, 16) {
                Ok(code_point) => {
                    if let Some(ch) = char::from_u32(code_point) {
                        let code = KeyCode::Char(ch);
                        Ok(key(code))
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
                    KeyCode::BackTab => write!(f, "<BACKTAB>"),
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
            Self::Mouse(mouse) => match mouse {
                MouseEvent::LeftPress => write!(f, "<LEFT_BUTTON_PRESS>"),
                MouseEvent::LeftRelease => write!(f, "<LEFT_BUTTON_RELEASE>"),
                MouseEvent::RightPress => write!(f, "<RIGHT_BUTTON_PRESS>"),
                MouseEvent::RightRelease => write!(f, "<RIGHT_BUTTON_RELEASE>"),
                MouseEvent::MiddlePress => write!(f, "<MIDDLE_BUTTON_PRESS>"),
                MouseEvent::MiddleRelease => write!(f, "<MIDDLE_BUTTON_RELEASE>"),
                MouseEvent::Drag => write!(f, "<DRAG>"),
                MouseEvent::ScrollUp => write!(f, "<WHEEL_UP>"),
                MouseEvent::ScrollDown => write!(f, "<WHEEL_DOWN>"),
            },
        }
    }
}

impl nojson::DisplayJson for InputMatcher {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.string(self)
    }
}
