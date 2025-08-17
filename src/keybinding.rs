use std::collections::BTreeMap;

use crate::KeyMatcher;

#[derive(Debug, Clone)]
pub struct KeymapRegistry<A> {
    pub contexts: BTreeMap<String, Keymap<A>>, // TODO: private
}

impl<'text, 'raw, A> TryFrom<nojson::RawJsonValue<'text, 'raw>> for KeymapRegistry<A>
where
    A: TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>,
{
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            contexts: value.try_into()?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct Keymap<A> {
    pub bindings: Vec<KeyBinding<A>>,
}

impl<'text, 'raw, A> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Keymap<A>
where
    A: TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>,
{
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let mut bindings = Vec::new();
        for (k, v) in value.to_object()? {
            bindings.push(KeyBinding::from_json_value(k.try_into()?, v)?);
        }
        Ok(Self { bindings })
    }
}

#[derive(Debug, Clone)]
pub struct KeyBinding<A> {
    pub keys: Vec<KeyMatcher>,
    pub label: String,
    pub hidden: bool,
    pub actions: Vec<A>,
}

impl<A> KeyBinding<A> {
    pub fn from_json_value<'text, 'raw>(
        primary_key: KeyMatcher,
        value: nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<Self, nojson::JsonParseError>
    where
        A: TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>,
    {
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
            actions: value
                .to_member("actions")?
                .map(TryFrom::try_from)?
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug, Clone)]
pub struct KeymapManager<T> {
    registry: KeymapRegistry<T>,
    context: String,
}

impl<T> KeymapManager<T> {
    pub fn new(registry: KeymapRegistry<T>, initial_context: &str) -> Option<Self> {
        Some(Self {
            registry,
            context: initial_context.to_owned(),
        })
    }

    pub fn set_context(&mut self, context: &str) -> bool {
        if self.registry.contexts.contains_key(context) {
            self.context = context.to_owned();
            true
        } else {
            false
        }
    }

    pub fn context(&self) -> &str {
        &self.context
    }

    pub fn registry(&self) -> &KeymapRegistry<T> {
        &self.registry
    }

    pub fn keymap(&self) -> &Keymap<T> {
        self.registry.contexts.get(&self.context).expect("bug")
    }
}
