//! Configurable action system with context-aware input bindings.
//!
//! This module provides the core action system that allows defining custom actions
//! and input bindings through JSON/JSONC configuration files. Actions can be organized
//! into different contexts, each with their own set of input bindings that support
//! both keyboard and mouse events.
use std::path::Path;

use crate::binding::InputMapRegistry;
use crate::json::LoadJsonError;

pub use crate::binding::{InputBinding, InputMap};
pub use crate::matcher::InputMatcher;

/// Marker trait for types that can be deserialized from JSON as action definitions.
pub trait Action:
    for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>
{
}

/// Configuration for a context-aware action system with input bindings.
///
/// Manages multiple input maps organized by context, with an optional setup action
/// and the ability to switch between different input contexts at runtime. Supports
/// both keyboard and mouse input events.
#[derive(Debug)]
pub struct ActionConfig<A> {
    context: ContextName,
    setup_action: Option<A>,
    input_map_registry: InputMapRegistry<A>,
    last_input: Option<tuinix::TerminalInput>,
}

impl<A: Action> ActionConfig<A> {
    /// Loads an action configuration from a JSONC file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_file(path, |v| ActionConfig::try_from(v))
    }

    /// Loads an action configuration from a JSONC string.
    pub fn load_from_str(name: &str, text: &str) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_str(name, text, |v| ActionConfig::try_from(v))
    }

    /// Returns the optional setup action that runs during initialization.
    pub fn setup_action(&self) -> Option<&A> {
        self.setup_action.as_ref()
    }

    /// Processes terminal input and returns the matching input binding, if any.
    ///
    /// This method takes terminal input (keyboard or mouse events) and attempts to match it
    /// against the current context's input bindings. If a matching binding is found and it
    /// specifies a context change, the active context will be switched before returning the
    /// binding.
    pub fn handle_input(&mut self, input: tuinix::TerminalInput) -> Option<&InputBinding<A>> {
        self.last_input = Some(input);

        let binding = self
            .input_map_registry
            .contexts
            .get(&self.context)?
            .get_binding(input)?;
        if let Some(context) = &binding.context {
            self.context = context.clone();
        }

        Some(binding)
    }

    /// Sets the current context if it exists, returning true on success.
    pub fn set_current_context(&mut self, context: &ContextName) -> bool {
        if self.input_map_registry.contexts.contains_key(context) {
            self.context = context.clone();
            true
        } else {
            false
        }
    }

    /// Returns the name of the currently active context.
    pub fn current_context(&self) -> &ContextName {
        &self.context
    }

    /// Returns the input map for the currently active context.
    pub fn current_input_map(&self) -> &InputMap<A> {
        &self.input_map_registry.contexts[&self.context]
    }

    /// Returns the input map for the specified context, if it exists.
    pub fn get_input_map(&self, context: &ContextName) -> Option<&InputMap<A>> {
        self.input_map_registry.contexts.get(context)
    }

    /// Returns an iterator over all contexts and their associated input maps.
    pub fn input_maps(&self) -> impl '_ + Iterator<Item = (&ContextName, &InputMap<A>)> {
        self.input_map_registry.contexts.iter()
    }

    /// Returns the last terminal input that was processed, if any.
    ///
    /// This tracks the most recent input passed to `handle_input()`, regardless of whether
    /// it resulted in a matching input binding. Returns `None` if no input has been processed yet.
    /// The input can be either a keyboard event or a mouse event.
    pub fn last_input(&self) -> Option<tuinix::TerminalInput> {
        self.last_input
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ActionConfig<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let setup = value.to_member("setup")?.required()?;
        Ok(Self {
            context: setup.to_member("context")?.required()?.try_into()?,
            setup_action: setup.to_member("action")?.map(A::try_from)?,
            input_map_registry: value.to_member("bindings")?.required()?.try_into()?,
            last_input: None,
        })
    }
}

/// A named context identifier for organizing input bindings.
///
/// Contexts allow grouping related input bindings together. Each context
/// can contain bindings for both keyboard and mouse events.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextName(String);

impl ContextName {
    /// Creates a new context name from a string.
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// Returns the context name as a string slice.
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ContextName {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let name: String = value.try_into()?;

        let keybindings = value.root().to_member("keybindings")?.required()?;
        if !keybindings
            .to_object()?
            .any(|(k, _)| k.to_unquoted_string_str().is_ok_and(|k| k == name))
        {
            return Err(value.invalid("undefined context"));
        }

        Ok(Self(name))
    }
}
