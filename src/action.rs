use crate::Config;

pub trait Action:
    Sized + for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>
{
    #[expect(unused_variables)]
    fn validate(
        &self,
        value: nojson::RawJsonValue<'_, '_>,
        config: &Config<Self>,
    ) -> Result<(), nojson::JsonParseError> {
        Ok(())
    }
}
