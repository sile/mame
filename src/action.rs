//! Configurable action system with context-aware input bindings.
//!
//! This module provides the core action system that allows defining custom actions
//! and input bindings through JSON/JSONC configuration files. Actions can be organized
//! into different contexts, each with their own set of input bindings that support
//! both keyboard and mouse events.
//!
//! The main component is [`BindingConfig`], which serves as a stateless configuration
//! container that holds binding definitions loaded from JSON/JSONC files. It provides
//! read-only access to the configured bindings and contexts without managing runtime
//! state or input processing.
//!
//! # Key Components
//!
//! - [`BindingConfig`] - Configuration container for context-aware action bindings
//! - [`BindingContextName`] - Named context identifier for organizing input bindings
//! - [`Action`] - Marker trait for types that can be deserialized from JSON as actions
//! - [`Binding`] - Individual input binding with matcher, action, and optional context switch
//! - [`InputMatcher`] - Input matching logic for keyboard and mouse events
use std::path::Path;

use crate::binding::ContextualBindings;
use crate::json::LoadJsonError;

pub use crate::binding::Binding;
pub use crate::matcher::InputMatcher;

/// Marker trait for types that can be deserialized from JSON as action definitions.
pub trait Action:
    for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>
{
}

/// A configuration container for context-aware action bindings.
///
/// Holds multiple input bindings organized by context, with an optional setup action
/// and setup context for initialization. This is a stateless configuration structure
/// that provides read-only access to binding definitions loaded from JSON/JSONC files.
/// Supports both keyboard and mouse input event definitions.
#[derive(Debug)]
pub struct BindingConfig<A> {
    initial_context: BindingContextName,
    setup_action: Option<A>,
    contextual_bindings: ContextualBindings<A>,
}

impl<A: Action> BindingConfig<A> {
    /// Loads an action binding configuration from a JSONC file.
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_file(path, |v| Self::try_from(v))
    }

    /// Loads an action binding configuration from a JSONC string.
    pub fn load_from_str(name: &str, text: &str) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_str(name, text, |v| Self::try_from(v))
    }

    /// Returns the initial context name.
    pub fn initial_context(&self) -> &BindingContextName {
        &self.initial_context
    }

    /// Returns the optional setup action that runs during initialization.
    pub fn setup_action(&self) -> Option<&A> {
        self.setup_action.as_ref()
    }

    /// Returns the input bindings for the specified context, if it exists.
    ///
    /// The bindings are returned in the order they appear in the configuration.
    pub fn get_bindings(&self, context: &BindingContextName) -> Option<&[Binding<A>]> {
        self.contextual_bindings
            .bindings
            .get(context)
            .map(|bindings| &bindings[..])
    }

    /// Returns an iterator over all contexts and their associated input bindings.
    ///
    /// This provides access to all configured contexts, not just the currently active one.
    pub fn all_bindings(&self) -> impl '_ + Iterator<Item = (&BindingContextName, &[Binding<A>])> {
        self.contextual_bindings
            .bindings
            .iter()
            .map(|(k, v)| (k, &v[..]))
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for BindingConfig<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let setup = value.to_member("setup")?.required()?;
        Ok(Self {
            initial_context: setup.to_member("context")?.required()?.try_into()?,
            setup_action: setup.to_member("action")?.map(A::try_from)?,
            contextual_bindings: value.to_member("bindings")?.required()?.try_into()?,
        })
    }
}

/// A named context identifier for organizing input bindings.
///
/// Contexts allow grouping related input bindings together. Each context
/// can contain bindings for both keyboard and mouse events.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BindingContextName(String);

impl BindingContextName {
    /// Creates a new context name from a string.
    pub fn new(name: &str) -> Self {
        Self(name.to_owned())
    }

    /// Returns the context name as a string slice.
    pub fn get(&self) -> &str {
        &self.0
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for BindingContextName {
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
