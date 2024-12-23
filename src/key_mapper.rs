use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::rpc::Request;

#[derive(Debug)]
pub struct KeyMapper {
    mapping: HashMap<Vec<Key>, Request>,
    pending_keys: Vec<Key>,
}

impl KeyMapper {
    pub fn new() -> Self {
        let mut this = Self {
            mapping: HashMap::new(),
            pending_keys: Vec::new(),
        };
        this.add(
            &[Key::from_char('x').ctrl(), Key::from_char('s').ctrl()],
            Request::save(),
        );

        this.add(
            &[Key::from_char('x').ctrl(), Key::from_char('c').ctrl()],
            Request::exit(),
        );
        this.add(&[Key::new(KeyCode::Esc)], Request::exit()); // TODO
        this
    }

    pub fn add(&mut self, keys: &[Key], request: Request) {
        self.mapping.insert(keys.to_vec(), request);
    }

    pub fn handle_input(&mut self, event: &KeyEvent) -> Option<Request> {
        self.pending_keys.push(Key::from(event.clone()));

        // TODO: optimize
        for i in 0..self.pending_keys.len() {
            if let Some(request) = self.mapping.remove(&self.pending_keys[i..]) {
                self.pending_keys.clear();
                return Some(request);
            }
        }

        None
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
