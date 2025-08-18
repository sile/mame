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
    pub fn get_actions(&self, key: tuinix::KeyInput) -> Option<&[A]> {
        self.bindings.iter().find_map(|b| {
            b.keys
                .iter()
                .any(|k| k.matches(key))
                .then_some(b.actions.as_slice())
        })
    }

    pub fn bindings(&self) -> impl '_ + Iterator<Item = &Keybinding<A>> {
        self.bindings.iter()
    }

    fn validate_actions(
        &self,
        value: nojson::RawJsonValue<'_, '_>,
        config: &Config<A>,
    ) -> Result<(), nojson::JsonParseError> {
        for ((_k, v), binding) in value.to_object().expect("bug").zip(&self.bindings) {
            binding.validate_actions(v, config)?;
        }
        Ok(())
    }
}

impl<'text, 'raw, A: Action> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Keymap<A> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let mut bindings = Vec::new();
        for (k, v) in value.to_object()? {
            bindings.push(Keybinding::from_json_value(k.try_into()?, v)?);
        }
        Ok(Self { bindings })
    }
}

#[derive(Debug, Clone)]
pub struct Keybinding<A> {
    pub keys: Vec<KeyMatcher>,
    pub label: String,
    pub hidden: bool,
    pub actions: Vec<A>,
}

impl<A: Action> Keybinding<A> {
    pub fn from_json_value<'text, 'raw>(
        primary_key: KeyMatcher,
        value: nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<Self, nojson::JsonParseError> {
        let mut keys = vec![primary_key];
        if let Some(aliases) = value.to_member("aliases")?.get() {
            for alias_key in aliases.to_array()? {
                keys.push(alias_key.try_into()?);
            }
        }
        Ok(Self {
            keys,
            label: value
                .to_member("label")?
                .map(TryFrom::try_from)?
                .unwrap_or_else(|| primary_key.to_string()),
            hidden: value
                .to_member("hidden")?
                .map(TryFrom::try_from)?
                .unwrap_or_default(),
            actions: value.to_member("actions")?.required()?.try_into()?,
        })
    }

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
