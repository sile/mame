pub trait Action:
    for<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>, Error = nojson::JsonParseError>
{
}
