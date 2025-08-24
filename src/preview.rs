use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct FilePreviewSpec {
    pub left_pane: Option<FilePreviewPaneSpec>,
    pub right_pane: Option<FilePreviewPaneSpec>,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewSpec {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            left_pane: value.to_member("left_pane")?.map(TryFrom::try_from)?,
            right_pane: value.to_member("right_pane")?.map(TryFrom::try_from)?,
        })
    }
}

#[derive(Debug, Clone)]
pub struct FilePreviewPaneSpec {
    pub file: PathBuf,
    pub skip_if_empty: bool,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for FilePreviewPaneSpec {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            file: value.to_member("fil")?.required()?.try_into()?,
            skip_if_empty: value
                .to_member("skip_if_empty")?
                .map(bool::try_from)?
                .unwrap_or_default(),
        })
    }
}

#[derive(Debug)]
pub struct FilePreview {}
