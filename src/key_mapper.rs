use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Debug)]
pub struct KeyMapper {}

impl KeyMapper {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub alt: bool,
    pub ctrl: bool,
}

impl Key {
    pub fn new(code: KeyCode) -> Self {
        Self {
            code,
            alt: false,
            ctrl: false,
        }
    }

    pub fn alt(mut self) -> Self {
        self.alt = true;
        self
    }

    pub fn ctrl(mut self) -> Self {
        self.ctrl = true;
        self
    }
}

impl From<KeyEvent> for Key {
    fn from(value: KeyEvent) -> Self {
        Self {
            code: value.code,
            alt: value.modifiers.contains(KeyModifiers::ALT),
            ctrl: value.modifiers.contains(KeyModifiers::CONTROL),
        }
    }
}
