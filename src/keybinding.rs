use std::collections::BTreeMap;

use crate::keymatcher::KeyMatcher;
use crate::{Action, Config};

#[derive(Debug, Clone)]
pub struct KeymapRegistry<A> {
    pub contexts: BTreeMap<String, Keymap<A>>, // TODO: private
}

impl<A: Action> KeymapRegistry<A> {
    pub fn validate_actions(
        &self,
        value: nojson::RawJsonValue<'_, '_>,
        config: &Config<A>,
    ) -> Result<(), nojson::JsonParseError> {
        for (k, v) in value.to_object()? {
            let context = k.to_unquoted_string_str()?;
            let keymap = self.contexts.get(context.as_ref()).expect("bug");
            keymap.validate_actions(v, config)?;
        }
        Ok(())
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for KeymapRegistry<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            contexts: value.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Keymap<A> {
    bindings: Vec<Keybinding<A>>,
}

impl<A: Action> Keymap<A> {
    pub fn get_binding(&self, key: tuinix::KeyInput) -> Option<&Keybinding<A>> {
        self.bindings
            .iter()
            .find(|b| b.keys.iter().any(|k| k.matches(key)))
    }

    pub fn bindings(&self) -> impl '_ + Iterator<Item = &Keybinding<A>> {
        self.bindings.iter()
    }

    fn validate_actions(
        &self,
        value: nojson::RawJsonValue<'_, '_>,
        config: &Config<A>,
    ) -> Result<(), nojson::JsonParseError> {
        for (v, binding) in value.to_array().expect("bug").zip(&self.bindings) {
            binding.validate_actions(v, config)?;
        }
        Ok(())
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Keymap<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            bindings: value.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Keybinding<A> {
    keys: Vec<KeyMatcher>,
    pub label: Option<String>,
    pub actions: Vec<A>, // TODO: Wrap by Arc
}

impl<A: Action> Keybinding<A> {
    fn validate_actions(
        &self,
        value: nojson::RawJsonValue<'_, '_>,
        config: &Config<A>,
    ) -> Result<(), nojson::JsonParseError> {
        for (action, value) in self
            .actions
            .iter()
            .zip(value.to_member("actions")?.required()?.to_array()?)
        {
            action.validate(value, config)?;
        }
        Ok(())
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Keybinding<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            keys: value.to_member("keys")?.required()?.try_into()?,
            label: value.to_member("label")?.map(TryFrom::try_from)?,
            actions: value.to_member("actions")?.required()?.try_into()?,
        })
    }
}
