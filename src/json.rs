//! JSON/JSONC utilities.
use std::borrow::Cow;
use std::fmt::Write;
use std::path::{Path, PathBuf};

/// Flattens a JSON value into a single string.
///
/// If the input is a string, returns it as-is. If the input is an array,
/// concatenates all string elements in the array into a single string.
/// Returns an error for any other JSON value types.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), nojson::JsonParseError> {
/// // Single string value
/// let json = nojson::RawJson::parse(r#""hello world""#)?;
/// let result = mame::json::flatten_string(json.value())?;
/// assert_eq!(result, "hello world");
///
/// // Array of strings
/// let json = nojson::RawJson::parse(r#"["hello", " ", "world"]"#)?;
/// let result = mame::json::flatten_string(json.value())?;
/// assert_eq!(result, "hello world");
///
/// // Empty array
/// let json = nojson::RawJson::parse("[]")?;
/// let result = mame::json::flatten_string(json.value())?;
/// assert_eq!(result, "");
///
/// // Nested arrays are flattened recursively
/// let json = nojson::RawJson::parse(r#"[["hello"], [" ", "world"]]"#)?;
/// let result = mame::json::flatten_string(json.value())?;
/// assert_eq!(result, "hello world");
/// # Ok(())
/// # }
/// ```
pub fn flatten_string<'text, 'raw>(
    value: nojson::RawJsonValue<'text, 'raw>,
) -> Result<Cow<'text, str>, nojson::JsonParseError> {
    if let Ok(s) = value.to_unquoted_string_str() {
        Ok(s)
    } else if let Ok(array) = value.to_array() {
        let mut buf = String::new();
        for value in array {
            flatten_string_to_buf(value, &mut buf)?;
        }
        Ok(Cow::Owned(buf))
    } else {
        Err(value.invalid("expected string or array of strings"))
    }
}

/// Parses a JSON value into a type by first flattening it to a string.
///
/// This function combines string flattening with parsing. It first flattens
/// the JSON value into a single string (handling both single strings and
/// arrays of strings), then parses that string into the target type.
///
/// This function uses [`flatten_string()`] internally to convert the JSON value
/// to a string before parsing.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), nojson::JsonParseError> {
/// // Parse a simple string to integer
/// let json = nojson::RawJson::parse(r#""42""#)?;
/// let result: i32 = mame::json::parse_from_flattened_string(json.value())?;
/// assert_eq!(result, 42);
///
/// // Parse an array of strings to integer
/// let json = nojson::RawJson::parse(r#"["4", "2"]"#)?;
/// let result: i32 = mame::json::parse_from_flattened_string(json.value())?;
/// assert_eq!(result, 42);
/// # Ok(())
/// # }
/// ```
pub fn parse_from_flattened_string<T>(
    value: nojson::RawJsonValue<'_, '_>,
) -> Result<T, nojson::JsonParseError>
where
    T: std::str::FromStr,
    T::Err: Into<Box<dyn Send + Sync + std::error::Error>>,
{
    let s = flatten_string(value)?;
    s.parse().map_err(|e| value.invalid(e))
}

fn flatten_string_to_buf<'text, 'raw>(
    value: nojson::RawJsonValue<'text, 'raw>,
    buf: &mut String,
) -> Result<(), nojson::JsonParseError> {
    if let Ok(s) = value.to_unquoted_string_str() {
        buf.push_str(&s);
        Ok(())
    } else if let Ok(array) = value.to_array() {
        for value in array {
            flatten_string_to_buf(value, buf)?;
        }
        Ok(())
    } else {
        Err(value.invalid("expected string or array of strings"))
    }
}

pub(crate) fn load_jsonc_file<P: AsRef<Path>, F, T>(path: P, f: F) -> Result<T, LoadJsonError>
where
    F: for<'text, 'raw> FnOnce(
        nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<T, nojson::JsonParseError>,
{
    let text = std::fs::read_to_string(&path).map_err(|e| LoadJsonError::Io {
        path: path.as_ref().to_path_buf(),
        error: e,
    })?;
    load_jsonc_str(&path.as_ref().display().to_string(), &text, f)
}

pub(crate) fn load_jsonc_str<F, T>(name: &str, text: &str, f: F) -> Result<T, LoadJsonError>
where
    F: for<'text, 'raw> FnOnce(
        nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<T, nojson::JsonParseError>,
{
    let (json, _) = nojson::RawJson::parse_jsonc(text)
        .map_err(|error| LoadJsonError::json(name, text, error))?;

    let mut preprocessor = Preprocessor::new(&json);
    preprocessor
        .process()
        .map_err(|error| LoadJsonError::json(name, text, error))?;
    let text = preprocessor.processed;
    let value = nojson::RawJson::parse_jsonc(&text)
        .and_then(|(json, _)| f(json.value()))
        .map_err(|error| LoadJsonError::json(name, &text, error))?;
    Ok(value)
}

/// Errors that can occur when loading and parsing JSON/JSONC files.
#[derive(Debug)]
pub enum LoadJsonError {
    /// I/O error occurred while reading a file.
    Io {
        /// Path to the file that couldn't be read
        path: PathBuf,
        /// The underlying I/O error
        error: std::io::Error,
    },
    /// JSON parsing error occurred while processing file content.
    Json {
        /// Path to the file containing invalid JSON
        path: PathBuf,
        /// The text content that failed to parse
        text: String,
        /// The underlying JSON parsing error
        error: nojson::JsonParseError,
    },
}

impl LoadJsonError {
    fn json(path: &str, text: &str, error: nojson::JsonParseError) -> Self {
        Self::Json {
            path: PathBuf::from(path),
            text: text.to_owned(),
            error,
        }
    }
}

impl std::fmt::Display for LoadJsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadJsonError::Io { path, error } => {
                write!(f, "failed to read file '{}': {error}", path.display())
            }
            LoadJsonError::Json { path, error, text } => format_json_error(f, path, error, text),
        }
    }
}

impl std::error::Error for LoadJsonError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        if let Self::Io { error, .. } = self {
            Some(error)
        } else {
            None
        }
    }
}

fn format_json_error(
    f: &mut std::fmt::Formatter<'_>,
    path: &Path,
    error: &nojson::JsonParseError,
    text: &str,
) -> std::fmt::Result {
    let (line_num, column_num) = error
        .get_line_and_column_numbers(text)
        .unwrap_or((std::num::NonZeroUsize::MIN, std::num::NonZeroUsize::MIN));

    let line = error.get_line(text).unwrap_or("");
    let (display_line, display_column) = format_line_around_position(line, column_num.get());
    writeln!(f, "{error}")?;
    writeln!(f, "--> {}:{line_num}:{column_num}", path.display())?;
    writeln!(f, "{line_num:4} |{display_line}")?;
    writeln!(f, "     |{:>column$} error", "^", column = display_column)?;
    Ok(())
}

fn format_line_around_position(line: &str, column_pos: usize) -> (String, usize) {
    const MAX_ERROR_LINE_CHARS: usize = 80;

    let chars: Vec<char> = line.chars().collect();
    let max_context = MAX_ERROR_LINE_CHARS / 2;

    let error_pos = column_pos.saturating_sub(1).min(chars.len());
    let start_pos = error_pos.saturating_sub(max_context);
    let end_pos = (error_pos + max_context + 1).min(chars.len());

    let mut result = String::new();
    let mut new_column_pos = error_pos - start_pos + 1;

    if start_pos > 0 {
        result.push_str("...");
        new_column_pos += 3;
    }

    result.push_str(&chars[start_pos..end_pos].iter().collect::<String>());

    if end_pos < chars.len() {
        result.push_str("...");
    }

    (result, new_column_pos)
}

#[derive(Debug)]
struct Preprocessor<'text, 'raw> {
    json: &'raw nojson::RawJson<'text>,
    processed: String,
    last_position: usize,
}

impl<'text, 'raw> Preprocessor<'text, 'raw> {
    fn new(json: &'raw nojson::RawJson<'text>) -> Self {
        Self {
            json,
            processed: String::new(),
            last_position: 0,
        }
    }

    fn process(&mut self) -> Result<(), nojson::JsonParseError> {
        self.process_value(self.json.value())
    }

    fn process_value(
        &mut self,
        value: nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<(), nojson::JsonParseError> {
        let start_position = value.position();
        let end_position = start_position + value.as_raw_str().len();

        self.processed
            .push_str(&self.json.text()[self.last_position..start_position]);
        self.last_position = value.position();

        if let Some(env_name) = value.to_member("env!")?.get() {
            let default_value = value.to_member("default")?.get();
            self.process_env(env_name, default_value)?;
            self.last_position = end_position;
        } else if let Ok(elements) = value.to_array() {
            for element in elements {
                self.process_value(element)?;
            }
        } else if let Ok(members) = value.to_object() {
            for (_member_name, member_value) in members {
                self.process_value(member_value)?;
            }
        }

        self.processed
            .push_str(&self.json.text()[self.last_position..end_position]);
        self.last_position = end_position;

        Ok(())
    }

    fn process_env(
        &mut self,
        name: nojson::RawJsonValue<'text, 'raw>,
        default: Option<nojson::RawJsonValue<'text, 'raw>>,
    ) -> Result<(), nojson::JsonParseError> {
        let name_str = name.to_unquoted_string_str()?;
        let json = if let Ok(value) = std::env::var(name_str.as_ref())
            && !value.is_empty()
        {
            nojson::RawJsonOwned::parse(&value)
                .or_else(|_| nojson::RawJsonOwned::parse(nojson::Json(value).to_string()))
                .expect("infallible")
        } else if let Some(default) = default {
            default.extract().into_owned()
        } else {
            return Err(name.invalid("environment variable is not set or empty"));
        };
        write!(self.processed, "{json}").expect("infallible");
        Ok(())
    }
}
