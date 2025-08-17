use crate::KeymapRegistry;

#[derive(Debug)]
pub struct Config<T> {
    pub keybindings: KeymapRegistry<T>, // TODO: private
}

impl<T> Config<T> {
    pub fn parse(
        value: nojson::RawJsonValue<'_, '_>,
        main_context: &str,
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

    pub fn entries(&self) -> impl '_ + Iterator<Item = (String, &T)> {
        std::iter::empty()
    }
}
