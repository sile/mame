use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::rpc::Request;

#[derive(Debug)]
pub struct KeyMapper {
    mapping: HashMap<Vec<Key>, Request>,
}

impl KeyMapper {
    pub fn new() -> Self {
        let mut this = Self {
            mapping: HashMap::new(),
        };
        this.add(
            &[Key::from_char('x').ctrl(), Key::from_char('c').ctrl()],
            Request::exit(),
        );
        this
    }

    pub fn add(&mut self, keys: &[Key], request: Request) {
        self.mapping.insert(keys.to_vec(), request);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Key {
    pub code: KeyCode,
    pub alt: bool,
    pub ctrl: bool,
}

impl Key {
    pub fn from_char(c: char) -> Self {
        Self::new(KeyCode::Char(c))
    }

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
