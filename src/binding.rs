use std::collections::BTreeMap;

use crate::action::{Action, ContextName};
use crate::matcher::InputMatcher;

#[derive(Debug, Clone)]
pub struct InputMapRegistry<A> {
    pub contexts: BTreeMap<ContextName, InputMap<A>>,
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for InputMapRegistry<A> {
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
pub struct InputMap<A> {
    bindings: Vec<InputBinding<A>>,
}

impl<A: Action> InputMap<A> {
    /// Finds the first binding that matches the given key input.
    pub fn get_binding(&self, input: tuinix::TerminalInput) -> Option<&InputBinding<A>> {
        self.bindings
            .iter()
            .find(|b| b.triggers.iter().any(|t| t.matches(input)))
    }

    /// Returns an iterator over all keybindings in this keymap.
    pub fn bindings(&self) -> impl '_ + Iterator<Item = &InputBinding<A>> {
        self.bindings.iter()
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for InputMap<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            bindings: value.try_into()?,
        })
    }
}

/// TODO: doc
#[derive(Debug, Clone)]
pub struct InputBinding<A> {
    /// TODO: doc
    pub triggers: Vec<InputMatcher>,

    /// Optional human-readable label for display purposes
    pub label: Option<String>,

    /// TODO: doc
    pub action: Option<A>,

    /// Optional context to switch to when this binding is activated
    pub context: Option<ContextName>,
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for InputBinding<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            triggers: value
                .to_member("triggers")?
                .map(TryFrom::try_from)?
                .unwrap_or_default(),
            label: value.to_member("label")?.map(TryFrom::try_from)?,
            action: value.to_member("action")?.map(TryFrom::try_from)?,
            context: value.to_member("context")?.map(TryFrom::try_from)?,
        })
    }
}
