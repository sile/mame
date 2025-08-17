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
        // TODO: check if the context exists
        self.context = context.to_owned();
        true
    }

    pub fn context(&self) -> &str {
        &self.context
    }
}
