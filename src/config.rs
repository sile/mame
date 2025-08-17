use crate::KeymapRegistry;

#[derive(Debug)]
pub struct Config<T> {
    pub context: String,
    pub kaymap_registry: KeymapRegistry<T>, // TODO: private
}

impl<T> Config<T> {
    pub fn parse(
        value: nojson::RawJsonValue<'_, '_>,
        default_context: &str,
    ) -> Result<Self, nojson::JsonParseError> {
        todo!()
    }

    pub fn set_context(&mut self, context: &str) -> bool {
        todo!()
    }

    pub fn context(&self) -> &str {
        todo!()
    }

    pub fn get_entry(&self, key: tuinix::KeyInput) -> Option<&T> {
        todo!()
    }
}
