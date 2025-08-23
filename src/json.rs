use std::collections::{BTreeMap, HashMap};
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
    let value = nojson::RawJson::parse_jsonc(&text)
        .and_then(|(json, _)| f(json.value()))
        .map_err(|e| LoadJsonFileError::Json {
            path: path.as_ref().to_path_buf(),
            text: text.clone(),
            error: e,
        })?;
    Ok(value)
}

pub fn load_jsonc_str<F, T>(name: &str, text: &str, f: F) -> Result<T, LoadJsonFileError>
where
    F: for<'text, 'raw> FnOnce(
        nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<T, nojson::JsonParseError>,
{
    let value = nojson::RawJson::parse_jsonc(&text)
        .and_then(|(json, _)| f(json.value()))
        .map_err(|e| LoadJsonFileError::Json {
            path: PathBuf::from(name),
            text: text.to_owned(),
            error: e,
        })?;
    Ok(value)
}

// TODO: Support nest for resolver
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

impl std::fmt::Display for LoadJsonFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LoadJsonFileError::Io { path, error } => {
                write!(f, "failed to read file '{}': {error}", path.display())
            }
            LoadJsonFileError::Json { path, error, text } => {
                format_json_error(f, path, error, text)
            }
        }
    }
}

impl std::error::Error for LoadJsonFileError {
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

    let prev_line = if line_num.get() == 1 {
        None
    } else {
        text.lines().nth(line_num.get() - 2)
    };

    let (display_line, display_column) = format_line_around_position(line, column_num.get());
    let prev_display_line = prev_line.map(|prev| {
        let (truncated, _) = format_line_around_position(prev, column_num.get());
        truncated
    });
    writeln!(f, "{error}")?;
    writeln!(f, "--> {}:{line_num}:{column_num}", path.display())?;
    if let Some(prev) = prev_display_line {
        writeln!(f, "     |{prev}")?;
    }
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

fn collect_references<'text, 'raw>(
    value: nojson::RawJsonValue<'text, 'raw>,
    references: &mut BTreeMap<usize, nojson::RawJsonValue<'text, 'raw>>,
) {
    if let Ok(array) = value.to_array() {
        for value in array {
            collect_references(value, references);
        }
    } else if let Ok(object) = value.to_object() {
        for (i, (name, value)) in object.enumerate() {
            if i == 0
                && name.to_unquoted_string_str().is_ok_and(|s| s == "ref")
                && value.kind() == nojson::JsonValueKind::String
            {
                references.insert(value.position(), value);
                break;
            } else {
                collect_references(value, references);
            }
        }
    }
}

#[derive(Debug)]
pub struct VariableResolver<'text, 'raw> {
    pub definitions: HashMap<String, VariableDefinition<'text, 'raw>>,
    pub references: BTreeMap<usize, nojson::RawJsonValue<'text, 'raw>>,
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for VariableResolver<'text, 'raw> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let definitions = value
            .to_member("variables")?
            .map(TryFrom::try_from)?
            .unwrap_or_default();
        let mut references = BTreeMap::new();
        collect_references(value, &mut references);
        Ok(Self {
            definitions,
            references,
        })
    }
}

impl<'text, 'raw> VariableResolver<'text, 'raw> {
    /*
        pub fn contains_ref(&self, value: nojson::RawJsonValue<'text, 'raw>) -> bool {
            match value.kind() {
                nojson::JsonValueKind::Null
                | nojson::JsonValueKind::Boolean
                | nojson::JsonValueKind::Integer
                | nojson::JsonValueKind::Float
                | nojson::JsonValueKind::String => false,
                nojson::JsonValueKind::Array => value
                    .to_array()
                    .expect("infallible")
                    .any(|v| self.contains_ref(v)),
                nojson::JsonValueKind::Object => {
                    if let Some(v) = value.to_member("ref").expect("infallible").get() {}
                }
            }
        }
    */
    /*
        pub fn resolve(
            &self,
            value: nojson::RawJsonValue<'text, 'raw>,
        ) -> Option<nojson::RawJsonOwned> {
            let mut resolved = String::new();
            if sefl.resolve_value(value, &mut resolved) {
                Some(resolved)
            } else {
                None
            }
        }

        fn resolve_value(
            &self,
            _value: nojson::RawJsonValue<'text, 'raw>,
            _resolved: &mut String,
        ) -> bool {
            /*
                        match value.kind() {
                            nojson::JsonValueKind::Null => false,
                        }
            */
            todo!()
        }
    */
}

#[derive(Debug)]
pub enum VariableDefinition<'text, 'raw> {
    Const {
        value: nojson::RawJsonValue<'text, 'raw>,
    },
    Env {
        default: Option<nojson::RawJsonValue<'text, 'raw>>,
        is_json: bool,
    },
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for VariableDefinition<'text, 'raw> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let ty = value.to_member("type")?.required()?;
        match ty.to_unquoted_string_str()?.as_ref() {
            "const" => Ok(Self::Const {
                value: value.to_member("value")?.required()?,
            }),
            "env" => Ok(Self::Env {
                default: value.to_member("default")?.get(),
                is_json: value
                    .to_member("is_json")?
                    .map(bool::try_from)?
                    .unwrap_or_default(),
            }),
            _ => Err(ty.invalid("unknown variable type")),
        }
    }
}
