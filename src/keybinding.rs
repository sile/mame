use std::collections::BTreeMap;

use crate::KeyMatcher;

#[derive(Debug, Clone)]
pub struct KeyBindingsTable {
    pub contexts: BTreeMap<String, KeyBindings>,
}

#[derive(Debug, Clone)]
pub struct KeyBindings {
    pub bindings: BTreeMap<KeyMatcher, ()>,
}
