use std::collections::BTreeMap;

use crate::KeyMatcher;

#[derive(Debug, Clone)]
pub struct KeymapRegistry<T> {
    pub contexts: BTreeMap<String, Keymap<T>>, // TODO: private
}

#[derive(Debug, Clone)]
pub struct Keymap<T> {
    pub bindings: BTreeMap<KeyMatcher, T>, // TODO: private
}

#[derive(Debug, Clone)]
pub struct KeymapManager<T> {
    registry: KeymapRegistry<T>,
    context: String,
}

impl<T> KeymapManager<T> {
    pub fn new(registry: KeymapRegistry<T>, initial_context: &str) -> Option<Self> {
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

    pub fn registry(&self) -> &KeymapRegistry<T> {
        &self.registry
    }

    pub fn keymap(&self) -> &Keymap<T> {
        self.registry.contexts.get(&self.context).expect("bug")
    }
}
