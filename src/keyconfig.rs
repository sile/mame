use crate::{KeyLabels, KeymapRegistry};

#[derive(Debug)]
pub struct KeyConfig<T> {
    pub keybindings: KeymapRegistry<T>, // TODO: private
    pub keylabels: KeyLabels,           // TODO: private
}

impl<T> KeyConfig<T> {
    pub fn parse(
        value: nojson::RawJsonValue<'_, '_>,
        main_context: &str,
    ) -> Result<Self, nojson::JsonParseError> {
        todo!()
    }
}
