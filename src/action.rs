//! Configurable action system with context-aware input bindings.
//!
//! This module provides the core action system that allows defining custom actions
//! and input bindings through JSON/JSONC configuration files. Actions can be organized
//! into different contexts, each with their own set of input bindings that support
//! both keyboard and mouse events.
use std::path::Path;
use std::sync::Arc;

use crate::binding::ContextualBindings;
use crate::json::LoadJsonError;

pub use crate::binding::Binding;
pub use crate::matcher::InputMatcher;

/// Marker trait for types that can be deserialized from JSON as action definitions.
pub trait Action:
    for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>
{
}

/// A context-aware action binding system with configurable input bindings.
///
/// Manages multiple input bindings organized by context, with an optional setup action
/// and the ability to switch between different input contexts at runtime. Supports
/// both keyboard and mouse input events.
#[derive(Debug)]
pub struct ActionBindingSystem<A> {
    context: ContextName,
    setup_action: Option<A>,
    contextual_bindings: ContextualBindings<A>,
    last_input: Option<tuinix::TerminalInput>,
    last_binding: Option<Arc<Binding<A>>>,
}

impl<A: Action> ActionBindingSystem<A> {
    /// Loads an action binding system configuration from a JSONC file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_file(path, |v| Self::try_from(v))
    }

    /// Loads an action binding system configuration from a JSONC string.
    pub fn load_from_str(name: &str, text: &str) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_str(name, text, |v| Self::try_from(v))
    }

    /// Returns the optional setup action that runs during initialization.
    pub fn setup_action(&self) -> Option<&A> {
        self.setup_action.as_ref()
    }

    /// Processes terminal input and returns the matching input binding, if any.
    ///
    /// This method takes terminal input (keyboard or mouse events) and attempts to match it
    /// against the current context's input bindings. If a matching binding is found, it will
    /// be stored as the last binding. To apply any context changes specified in the binding,
    /// call `apply_last_context_switch()` after this method.
    pub fn handle_input(&mut self, input: tuinix::TerminalInput) -> Option<Arc<Binding<A>>> {
        self.last_input = Some(input);
        self.last_binding = None;

        let binding = self
            .contextual_bindings
            .bindings
            .get(&self.context)?
            .iter()
            .find(|b| b.matches(input))?;

        self.last_binding = Some(binding.clone());
        Some(binding.clone())
    }

    /// Applies the context switch from the last matched binding, if any.
    ///
    /// This method should be called after `handle_input()` to apply any context changes
    /// specified in the matched binding. Returns `true` if a context switch was applied,
    /// or `false` if no context switch was needed or no binding was matched.
    pub fn apply_last_context_switch(&mut self) -> bool {
        if let Some(binding) = &self.last_binding
            && let Some(context) = &binding.context
        {
            self.context = context.clone();
            true
        } else {
            false
        }
    }

    /// Sets the current context if it exists, returning true on success.
    pub fn set_current_context(&mut self, context: &ContextName) -> bool {
        if self.contextual_bindings.bindings.contains_key(context) {
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

    /// Returns all input bindings for the currently active context.
    ///
    /// The bindings are returned in the order they appear in the configuration,
    /// which is also the order they are checked during input matching.
    ///
    /// # Panics
    ///
    /// This method never panics. The current context is guaranteed to exist in the
    /// contextual bindings map, as it is validated during initialization and can only
    /// be changed to existing contexts via `set_current_context()`.
    pub fn current_bindings(&self) -> &[Arc<Binding<A>>] {
        &self.contextual_bindings.bindings[&self.context]
    }

    /// Returns an iterator over all contexts and their associated input bindings.
    ///
    /// This provides access to all configured contexts, not just the currently active one.
    pub fn all_bindings(&self) -> impl '_ + Iterator<Item = (&ContextName, &[Arc<Binding<A>>])> {
        self.contextual_bindings
            .bindings
            .iter()
            .map(|(k, v)| (k, &v[..]))
    }

    /// Returns the last terminal input that was processed, if any.
    ///
    /// This tracks the most recent input passed to `handle_input()`, regardless of whether
    /// it resulted in a matching input binding. Returns `None` if no input has been processed yet.
    /// The input can be either a keyboard event or a mouse event.
    pub fn last_input(&self) -> Option<tuinix::TerminalInput> {
        self.last_input
    }

    /// Returns the last input binding that was successfully matched, if any.
    ///
    /// This tracks the most recent binding returned by `handle_input()` when terminal input
    /// was processed and matched against the current context's bindings. Returns `None` if no
    /// input has been processed yet or if the last input didn't match any binding.
    pub fn last_binding(&self) -> Option<Arc<Binding<A>>> {
        self.last_binding.clone()
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ActionBindingSystem<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let setup = value.to_member("setup")?.required()?;
        Ok(Self {
            context: setup.to_member("context")?.required()?.try_into()?,
            setup_action: setup.to_member("action")?.map(A::try_from)?,
            contextual_bindings: value.to_member("bindings")?.required()?.try_into()?,
            last_input: None,
            last_binding: None,
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

        let bindings = value.root().to_member("bindings")?.required()?;
        if !bindings
            .to_object()?
            .any(|(k, _)| k.to_unquoted_string_str().is_ok_and(|k| k == name))
        {
            return Err(value.invalid("undefined context"));
        }

        Ok(Self(name))
    }
}
