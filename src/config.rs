use std::path::Path;

use crate::keybinding::KeymapRegistry;
use crate::{Action, Keymap, LoadJsonFileError};

#[derive(Debug)]
pub struct Config<A> {
    context: String,
    keymap_registry: KeymapRegistry<A>,
}

impl<A: Action> Config<A> {
    pub fn load_file<P: AsRef<Path>>(path: P) -> Result<Self, LoadJsonFileError> {
        crate::json::load_jsonc_file(path, |v| Config::try_from(v))
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

    pub fn keymaps(&self) -> impl '_ + Iterator<Item = (&str, &Keymap<A>)> {
        self.keymap_registry
            .contexts
            .iter()
            .map(|(k, v)| (k.as_str(), v))
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Config<A> {
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
            keymap_registry,
        };
        config
            .keymap_registry
            .validate_actions(keybindings, &config)?;

        Ok(config)
    }
}
