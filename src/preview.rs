use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FilePreviewOptions {
    pub path: PathBuf,
    pub skip_if_empty: bool,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewOptions {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            path: value.to_member("path")?.required()?.try_into()?,
            skip_if_empty: value
                .to_member("skip_if_empty")?
                .map(bool::try_from)?
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug)]
pub struct FilePreview {}
