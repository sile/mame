use std::collections::BTreeMap;

use crate::KeyMatcher;

pub type ContextName = String;
pub type Action = (); // TODO:

#[derive(Debug, Clone)]
pub struct KeymapRegistry {
    pub contexts: BTreeMap<ContextName, Keymap>,
}

#[derive(Debug, Clone)]
pub struct Keymap {
    pub bindings: BTreeMap<KeyMatcher, Action>,
}

#[derive(Debug, Clone)]
pub struct KeymapManager {
    pub registry: KeymapRegistry,
    pub current_context: ContextName,
}

impl KeymapManager {
    pub fn new(registry: KeymapRegistry, initial_context: ContextName) -> Self {
        Self {
            registry,
            current_context: initial_context,
        }
    }

    pub fn set_context(&mut self, context: ContextName) {
        self.current_context = context;
    }

    pub fn get_current_context(&self) -> &ContextName {
        &self.current_context
    }

    pub fn get_registry(&self) -> &KeymapRegistry {
        &self.registry
    }

    pub fn get_registry_mut(&mut self) -> &mut KeymapRegistry {
        &mut self.registry
    }
}
