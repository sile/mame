//! JSON/JSONC utilities.
use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
use std::path::{Path, PathBuf};

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

    let resolver =
        VariableResolver::new(&json).map_err(|error| LoadJsonError::json(name, text, error))?;
    if resolver.references.is_empty() {
        return f(json.value()).map_err(|error| LoadJsonError::json(name, text, error));
    }

    let text = resolver
        .resolve(json.value())
        .map_err(|error| LoadJsonError::json(name, text, error))?;
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

fn collect_references<'text, 'raw>(
    value: nojson::RawJsonValue<'text, 'raw>,
    references: &mut BTreeMap<usize, nojson::RawJsonValue<'text, 'raw>>,
) {
    if let Ok(array) = value.to_array() {
        for value in array {
            collect_references(value, references);
        }
    } else if let Ok(object) = value.to_object() {
        for (i, (member_name, member_value)) in object.enumerate() {
            if i == 0
                && member_name
                    .to_unquoted_string_str()
                    .is_ok_and(|s| s == "ref")
            {
                references.insert(value.position(), member_value);
                break;
            } else {
                collect_references(member_value, references);
            }
        }
    }
}

#[derive(Debug)]
struct VariableResolver<'text, 'raw> {
    json: &'raw nojson::RawJson<'text>,
    definitions: HashMap<String, VariableDefinition<'text, 'raw>>,
    references: BTreeMap<usize, String>,
    resolved: String,
    resolved_values: BTreeMap<usize, std::ops::Range<usize>>, // original pos => resolved range
    last_position: usize,
}

impl<'text, 'raw> VariableResolver<'text, 'raw> {
    fn new(json: &'raw nojson::RawJson<'text>) -> Result<Self, nojson::JsonParseError> {
        let value = json.value();
        let definitions: HashMap<_, _> = value
            .to_member("variables")?
            .map(TryFrom::try_from)?
            .unwrap_or_default();
        let mut unchecked_references = BTreeMap::new();
        collect_references(value, &mut unchecked_references);

        let mut references = BTreeMap::new();
        for (position, value) in unchecked_references {
            let name = value.to_unquoted_string_str()?;
            if !definitions.contains_key(name.as_ref()) {
                return Err(value.invalid(format!("undefined variable name")));
            }
            references.insert(position, name.into_owned());
        }

        Ok(Self {
            json,
            definitions,
            references,
            resolved: String::new(),
            resolved_values: BTreeMap::new(),
            last_position: 0,
        })
    }
}

impl<'text, 'raw> VariableResolver<'text, 'raw> {
    pub fn resolve(
        mut self,
        value: nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<String, nojson::JsonParseError> {
        self.resolve_value(value)?;
        Ok(self.resolved)
    }

    fn resolve_const(
        &mut self,
        name: &str,
        def: nojson::RawJsonValue<'text, 'raw>,
        value: nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<(), nojson::JsonParseError> {
        let json = if let Some(range) = self.resolved_values.get(&value.position()).cloned() {
            let text = &self.resolved[range];
            nojson::RawJsonOwned::parse_jsonc(text)
                .map_err(|e| self.invalid_json(value, "resolved", text, e))?
                .0
        } else {
            value.extract().into_owned()
        };

        write!(self.resolved, "{json}").expect("infallible");
        self.definitions
            .insert(name.to_owned(), VariableDefinition::Resolved { def, json });
        Ok(())
    }

    fn invalid_json(
        &self,
        value: nojson::RawJsonValue<'text, 'raw>,
        kind: &str,
        text: &str,
        error: nojson::JsonParseError,
    ) -> nojson::JsonParseError {
        let message = format!(
            r#"failed to parse {kind} JSON
```jsonc
// This is the resolved JSON
// ERROR: {error}
{text}
```
"#
        );
        value.invalid(message)
    }

    fn resolve_env(
        &mut self,
        name: &str,
        ref_value: nojson::RawJsonValue<'text, 'raw>,
        def: nojson::RawJsonValue<'text, 'raw>,
        default: Option<nojson::RawJsonValue<'text, 'raw>>,
    ) -> Result<(), nojson::JsonParseError> {
        let json = if let Ok(value) = std::env::var(name)
            && !value.is_empty()
        {
            nojson::RawJsonOwned::parse(&value)
                .or_else(|_| nojson::RawJsonOwned::parse(nojson::Json(value).to_string()))
                .expect("infallible")
        } else if let Some(default) = default {
            if let Some(range) = self.resolved_values.get(&default.position()).cloned() {
                let text = &self.resolved[range];
                nojson::RawJsonOwned::parse_jsonc(text)
                    .map_err(|e| self.invalid_json(default, "resolved", text, e))?
                    .0
            } else {
                default.extract().into_owned()
            }
        } else {
            return Err(ref_value.invalid(format!("environment variable {name} is not set")));
        };

        write!(self.resolved, "{json}").expect("infallible");
        self.definitions
            .insert(name.to_owned(), VariableDefinition::Resolved { def, json });
        Ok(())
    }

    fn resolve_value(
        &mut self,
        value: nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<(), nojson::JsonParseError> {
        let end_position = value.position() + value.as_raw_str().len();
        let after_start_position = self.resolved.len();
        if let Some(variable_name) = self.references.remove(&value.position()) {
            let def = &self.definitions[&variable_name];
            if value.position() < def.position() {
                return Err(value.invalid("variable reference appears before its definition"));
            }
            match def {
                VariableDefinition::Const { def, value } => {
                    self.resolve_const(&variable_name, *def, *value)?;
                }
                VariableDefinition::Env { def, default } => {
                    self.resolve_env(&variable_name, value, *def, *default)?;
                }
                VariableDefinition::Resolved { json, .. } => {
                    write!(self.resolved, "{json}").expect("infallible");
                }
            };
            self.last_position = end_position;
        } else if !self.contains_ref(value) {
        } else if let Ok(array) = value.to_array() {
            for value in array {
                self.resolved
                    .push_str(&self.json.text()[self.last_position..value.position()]);
                self.last_position = value.position();
                self.resolve_value(value)?;
            }
        } else if let Ok(object) = value.to_object() {
            for (_, value) in object {
                self.resolved
                    .push_str(&self.json.text()[self.last_position..value.position()]);
                self.last_position = value.position();
                self.resolve_value(value)?;
            }
        } else {
            panic!("bug");
        }

        self.resolved
            .push_str(&self.json.text()[self.last_position..end_position]);
        self.last_position = end_position;

        let after_end_position = self.resolved.len();
        self.resolved_values.insert(
            value.position(),
            std::ops::Range {
                start: after_start_position,
                end: after_end_position,
            },
        );
        Ok(())
    }

    fn contains_ref(&self, value: nojson::RawJsonValue<'text, 'raw>) -> bool {
        let end_position = value.position() + value.as_raw_str().len();
        self.references
            .range(value.position()..end_position)
            .next()
            .is_some()
    }
}

#[derive(Debug)]
enum VariableDefinition<'text, 'raw> {
    Const {
        def: nojson::RawJsonValue<'text, 'raw>,
        value: nojson::RawJsonValue<'text, 'raw>,
    },
    Env {
        def: nojson::RawJsonValue<'text, 'raw>,
        default: Option<nojson::RawJsonValue<'text, 'raw>>,
    },
    Resolved {
        def: nojson::RawJsonValue<'text, 'raw>,
        json: nojson::RawJsonOwned,
    },
}

impl<'text, 'raw> VariableDefinition<'text, 'raw> {
    fn position(&self) -> usize {
        match self {
            Self::Const { def, .. } | Self::Env { def, .. } | Self::Resolved { def, .. } => {
                def.position()
            }
        }
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for VariableDefinition<'text, 'raw> {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let ty = value.to_member("type")?.required()?;
        match ty.to_unquoted_string_str()?.as_ref() {
            "const" => Ok(Self::Const {
                def: value,
                value: value.to_member("value")?.required()?,
            }),
            "env" => Ok(Self::Env {
                def: value,
                default: value.to_member("default")?.get(),
            }),
            _ => Err(ty.invalid("unknown variable type")),
        }
    }
}
