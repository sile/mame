use std::path::{Path, PathBuf};

pub fn load_jsonc_file<P: AsRef<Path>, F, T>(path: P, f: F) -> Result<T, LoadJsonFileError>
where
    F: for<'text, 'raw> FnOnce(
        nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<T, nojson::JsonParseError>,
{
    let text = std::fs::read_to_string(&path).map_err(|e| LoadJsonFileError::Io {
        path: path.as_ref().to_path_buf(),
        error: e,
    })?;
    let value = nojson::RawJson::parse(&text)
        .and_then(|json| f(json.value()))
        .map_err(|e| LoadJsonFileError::Json {
            path: path.as_ref().to_path_buf(),
            text: text.clone(),
            error: e,
        })?;
    Ok(value)
}

#[derive(Debug)]
pub enum LoadJsonFileError {
    Io {
        path: PathBuf,
        error: std::io::Error,
    },
    Json {
        path: PathBuf,
        text: String,
        error: nojson::JsonParseError,
    },
}
