use std::collections::BTreeMap;

use crate::action::{Action, ContextName};
use crate::matcher::InputMatcher;

#[derive(Debug, Clone)]
pub struct ContextualBindings<A> {
    pub bindings: BTreeMap<ContextName, Vec<InputBinding<A>>>,
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ContextualBindings<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let mut next_binding_id = 0;
        Ok(Self {
            bindings: value
                .to_object()?
                .map(|(k, v)| {
                    let context_name = k.try_into()?;
                    let mut bindings: Vec<InputBinding<_>> = v.try_into()?;
                    for b in &mut bindings {
                        b.id.0 = next_binding_id;
                        next_binding_id += 1;
                    }
                    Ok((context_name, bindings))
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

/// A unique identifier for an input binding.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct InputBindingId(usize);

/// A single input binding that maps terminal input patterns to actions within a context.
#[derive(Debug, Clone)]
pub struct InputBinding<A> {
    /// A unique identifier for this input binding.
    ///
    /// This ID is automatically assigned during JSON parsing and is unique across all bindings
    /// in all contexts. It can be used for tracking, debugging, or referencing specific bindings.
    pub id: InputBindingId,

    /// Input patterns that trigger this binding (keyboard keys, mouse events, etc.)
    pub triggers: Vec<InputMatcher>,

    /// Optional human-readable label for display purposes
    pub label: Option<String>,

    /// Optional action to execute when the binding is triggered
    pub action: Option<A>,

    /// Optional context to switch to when this binding is activated
    pub context: Option<ContextName>,
}

impl<A: Action> InputBinding<A> {
    /// Checks if this binding matches the given terminal input.
    ///
    /// Returns `true` if any of the binding's triggers match the provided input.
    pub fn matches(&self, input: tuinix::TerminalInput) -> bool {
        self.triggers.iter().any(|t| t.matches(input))
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for InputBinding<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            id: InputBindingId::default(), // [NOTE] This field will be updated after JSON parsing is complete
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
