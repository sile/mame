use std::collections::BTreeMap;

use crate::KeyMatcher;

pub type Action = (); // TODO:

#[derive(Debug, Clone)]
pub struct KeymapRegistry {
    pub contexts: BTreeMap<String, Keymap>, // TODO: private
}

#[derive(Debug, Clone)]
pub struct Keymap {
    pub bindings: BTreeMap<KeyMatcher, Action>, // TODO: private
}

#[derive(Debug, Clone)]
pub struct KeymapManager {
    registry: KeymapRegistry,
    context: String,
}

impl KeymapManager {
    pub fn new(registry: KeymapRegistry, initial_context: &str) -> Option<Self> {
        Some(Self {
            registry,
            context: initial_context.to_owned(),
        })
    }

    pub fn set_context(&mut self, context: &str) -> bool {
        if self.registry.contexts.contains_key(context) {
            self.context = context.to_owned();
            true
        } else {
            false
        }
    }

    pub fn context(&self) -> &str {
        &self.context
    }

    pub fn registry(&self) -> &KeymapRegistry {
        &self.registry
    }

    pub fn keymap(&self) -> &Keymap {
        self.registry.contexts.get(&self.context).expect("bug")
    }
}
