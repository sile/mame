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

/// A collection of input bindings for a specific context.
///
/// Each input map contains a list of bindings that define how terminal inputs
/// (keyboard and mouse events) should be handled within that context.
#[derive(Debug, Clone)]
pub struct InputMap<A> {
    bindings: Vec<InputBinding<A>>,
}

impl<A: Action> InputMap<A> {
    /// Finds the first binding that matches the given terminal input.
    ///
    /// Returns the first binding whose triggers match the provided input,
    /// or `None` if no matching binding is found.
    pub fn get_binding(&self, input: tuinix::TerminalInput) -> Option<&InputBinding<A>> {
        self.bindings
            .iter()
            .find(|b| b.triggers.iter().any(|t| t.matches(input)))
    }

    /// Returns an iterator over all input bindings in this input map.
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

/// A single input binding that maps terminal input patterns to actions within a context.
#[derive(Debug, Clone)]
pub struct InputBinding<A> {
    /// Input patterns that trigger this binding (keyboard keys, mouse events, etc.)
    pub triggers: Vec<InputMatcher>,

    /// Optional human-readable label for display purposes
    pub label: Option<String>,

    /// Optional action to execute when the binding is triggered
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
