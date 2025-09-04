use std::collections::BTreeMap;

use crate::action::{Action, BindingContextName};
use crate::matcher::InputMatcher;

#[derive(Debug, Clone)]
pub(crate) struct ContextualBindings<A> {
    pub(crate) bindings: BTreeMap<BindingContextName, Vec<Binding<A>>>,
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ContextualBindings<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            bindings: value
                .to_object()?
                .map(|(k, v)| {
                    let context_name = k.try_into()?;
                    let bindings = v.try_into()?;
                    Ok((context_name, bindings))
                })
                .collect::<Result<_, _>>()?,
        })
    }
}

/// A single input binding that maps terminal input patterns to actions within a context.
#[derive(Debug, Clone)]
pub struct Binding<A> {
    /// Input patterns that trigger this binding (keyboard keys, mouse events, etc.)
    pub triggers: Vec<InputMatcher>,

    /// Optional human-readable label for display purposes
    pub label: Option<String>,

    /// Optional action to execute when the binding is triggered
    pub action: Option<A>,

    /// Optional context to switch to when this binding is activated
    pub context: Option<BindingContextName>,
}

impl<A: Action> Binding<A> {
    /// Checks if this binding matches the given terminal input.
    ///
    /// Returns `true` if any of the binding's triggers match the provided input.
    pub fn matches(&self, input: tuinix::TerminalInput) -> bool {
        self.triggers.iter().any(|t| t.matches(input))
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Binding<A> {
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
