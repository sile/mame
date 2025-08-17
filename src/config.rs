use crate::KeymapRegistry;

#[derive(Debug)]
pub struct Config<A> {
    pub context: String,
    pub keymap_registry: KeymapRegistry<A>, // TODO: private
}

impl<A> Config<A> {
    pub fn set_context(&mut self, context: &str) -> bool {
        if self.keymap_registry.contexts.contains_key(context) {
            self.context = context.to_owned();
            true
        } else {
            false
        }
    }

    pub fn context(&self) -> &str {
        &self.context
    }
}

impl<'text, 'raw, A> TryFrom<nojson::RawJsonValue<'text, 'raw>> for Config<A>
where
    A: TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>,
{
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

        Ok(Self {
            context,
            keymap_registry,
        })
    }
}
