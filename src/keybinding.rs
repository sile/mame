use std::collections::BTreeMap;

use crate::action::{Action, ContextName};
use crate::keymatcher::KeyMatcher;

#[derive(Debug, Clone)]
pub struct KeymapRegistry<A> {
    pub contexts: BTreeMap<ContextName, Keymap<A>>,
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for KeymapRegistry<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            contexts: value
                .to_object()?
                .map(|(k, v)| Ok((k.try_into()?, v.try_into()?)))
                .collect::<Result<_, _>>()?,
        })
    }
}

/// A collection of key bindings for a specific context.
#[derive(Debug, Clone)]
pub struct Keymap<A> {
    bindings: Vec<Keybinding<A>>,
}

impl<A: Action> Keymap<A> {
    /// Finds the first binding that matches the given key input.
    pub fn get_binding(&self, key: tuinix::KeyInput) -> Option<&Keybinding<A>> {
        self.bindings
            .iter()
            .find(|b| b.keys.iter().any(|k| k.matches(key)))
    }

    /// Returns an iterator over all keybindings in this keymap.
    pub fn bindings(&self) -> impl '_ + Iterator<Item = &Keybinding<A>> {
        self.bindings.iter()
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Keymap<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            bindings: value.try_into()?,
        })
    }
}

/// A single key binding that maps key combinations to actions within a context.
#[derive(Debug, Clone)]
pub struct Keybinding<A> {
    /// Key combinations that trigger this binding
    pub keys: Vec<KeyMatcher>,

    /// Optional human-readable label for display purposes
    pub label: Option<String>,

    /// Optional action to execute when keys are pressed
    pub action: Option<A>,

    /// Optional context to switch to when this binding is activated
    pub context: Option<ContextName>,
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Keybinding<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            keys: value
                .to_member("keys")?
                .map(TryFrom::try_from)?
                .unwrap_or_default(),
            label: value.to_member("label")?.map(TryFrom::try_from)?,
            action: value.to_member("action")?.map(TryFrom::try_from)?,
            context: value.to_member("context")?.map(TryFrom::try_from)?,
        })
    }
}
