use crate::{KeyMatcher, VecMap};

#[derive(Debug, Clone)]
pub struct KeymapRegistry<T> {
    pub contexts: VecMap<String, Keymap<T>>, // TODO: private
}

#[derive(Debug, Clone)]
pub struct Keymap<T> {
    pub bindings: Vec<KeyBinding<T>>,
}

#[derive(Debug, Clone)]
pub struct KeyBinding<A> {
    pub keys: Vec<KeyMatcher>,
    pub label: String,
    pub hidden: bool,
    pub actions: Vec<A>,
}

impl<A> KeyBinding<A>
where
    A: for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>,
{
    pub fn from_json_value(
        primary_key: KeyMatcher,
        value: nojson::RawJsonValue<'_, '_>,
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
