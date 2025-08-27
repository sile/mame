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

pub trait Action:
    for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>
{
}

#[derive(Debug)]
pub struct ActionConfig<A> {
    context: String,
    setup_action: Option<A>,
    keymap_registry: KeymapRegistry<A>,
}

impl<A: Action> ActionConfig<A> {
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_file(path, |v| ActionConfig::try_from(v))
    }

    pub fn load_str(name: &str, text: &str) -> Result<Self, LoadJsonError> {
        crate::json::load_jsonc_str(name, text, |v| ActionConfig::try_from(v))
    }

    pub fn setup_action(&self) -> Option<&A> {
        self.setup_action.as_ref()
    }

    pub fn set_current_context(&mut self, context: &str) -> bool {
        if self.keymap_registry.contexts.contains_key(context) {
            self.context = context.to_owned();
            true
        } else {
            false
        }
    }

    pub fn current_context(&self) -> &str {
        &self.context
    }

    pub fn current_keymap(&self) -> &Keymap<A> {
        &self.keymap_registry.contexts[&self.context]
    }

    pub fn get_keymap(&self, context: &str) -> Option<&Keymap<A>> {
        self.keymap_registry.contexts.get(context)
    }

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
