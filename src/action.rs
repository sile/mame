use crate::Config;

pub trait Action: Sized {
    fn validate(
        &self,
        value: nojson::RawJsonValue<'_, '_>,
        config: &Config<Self>,
    ) -> Result<(), nojson::JsonParseError>;
}
