//! Configurable action system with context-aware keybindings.
//!
//! This module provides the core action system that allows defining custom actions
//! and keybindings through JSON/JSONC configuration files. Actions can be organized
//! into different contexts, each with their own set of keybindings.
use std::path::Path;

use crate::json::LoadJsonError;
use crate::keybinding::KeymapRegistry;

pub use crate::keybinding::{Keybinding, Keymap};
pub use crate::keymatcher::KeyMatcher;

/// Marker trait for types that can be deserialized from JSON as action definitions.
pub trait Action:
    for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>
{
}

/// Configuration for a context-aware action system with keybindings.
///
/// Manages multiple keymaps organized by context, with an optional setup action
/// and the ability to switch between different input contexts at runtime.
#[derive(Debug)]
pub struct ActionConfig<A> {
    context: String,
    setup_action: Option<A>,
    keymap_registry: KeymapRegistry<A>,
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

    /// Sets the current context if it exists, returning true on success.
    pub fn set_current_context(&mut self, context: &str) -> bool {
        if self.keymap_registry.contexts.contains_key(context) {
            self.context = context.to_owned();
            true
        } else {
            false
        }
    }

    /// Returns the name of the currently active context.
    pub fn current_context(&self) -> &str {
        &self.context
    }

    /// Returns the keymap for the currently active context.
    pub fn current_keymap(&self) -> &Keymap<A> {
        &self.keymap_registry.contexts[&self.context]
    }

    /// Returns the keymap for the specified context, if it exists.
    pub fn get_keymap(&self, context: &str) -> Option<&Keymap<A>> {
        self.keymap_registry.contexts.get(context)
    }

    /// Returns an iterator over all contexts and their associated keymaps.
    pub fn keymaps(&self) -> impl '_ + Iterator<Item = (&str, &Keymap<A>)> {
        self.keymap_registry
            .contexts
            .iter()
            .map(|(k, v)| (k.as_str(), v))
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ActionConfig<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let setup = value.to_member("setup")?.required()?;
        let context_value = setup.to_member("context")?.required()?;
        let context = context_value.try_into()?;

        let keybindings = value.to_member("keybindings")?.required()?;
        let keymap_registry: KeymapRegistry<A> = keybindings.try_into()?;
        if !keymap_registry.contexts.contains_key(&context) {
            return Err(context_value.invalid("undefined keybindings context"));
        }

        let config = Self {
            context,
            setup_action: setup.to_member("action")?.map(A::try_from)?,
            keymap_registry,
        };
        config.keymap_registry.validate(keybindings, &config)?;

        Ok(config)
    }
}
