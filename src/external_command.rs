use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExternalCommand {
    pub command: PathBuf,
    pub args: Vec<String>,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ExternalCommand {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            command: value.to_member("command")?.required()?.try_into()?,
            args: value
                .to_member("args")?
                .map(Vec::try_from)?
                .unwrap_or_default(),
        })
    }
}

impl nojson::DisplayJson for ExternalCommand {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("command", &self.command)?;
            if !self.args.is_empty() {
                f.member("args", &self.args)?;
            }
            Ok(())
        })
    }
}
