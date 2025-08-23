use std::collections::{BTreeMap, HashMap};
use std::fmt::Write;
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
    load_jsonc_str(&path.as_ref().display().to_string(), &text, f)
}

pub fn load_jsonc_str<F, T>(name: &str, text: &str, f: F) -> Result<T, LoadJsonFileError>
where
    F: for<'text, 'raw> FnOnce(
        nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<T, nojson::JsonParseError>,
{
    let (json, _) = nojson::RawJson::parse_jsonc(text)
        .map_err(|error| LoadJsonFileError::json(name, text, error))?;

    let resolver =
        VariableResolver::new(&json).map_err(|error| LoadJsonFileError::json(name, text, error))?;
    if resolver.references.is_empty() {
        return f(json.value()).map_err(|error| LoadJsonFileError::json(name, text, error));
    }

    let text = resolver
        .resolve(json.value())
        .map_err(|error| LoadJsonFileError::json(name, text, error))?;
    let value = nojson::RawJson::parse_jsonc(&text)
        .and_then(|(json, _)| f(json.value()))
        .map_err(|error| LoadJsonFileError::json(name, &text, error))?;
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

impl LoadJsonFileError {
    fn json(path: &str, text: &str, error: nojson::JsonParseError) -> Self {
        Self::Json {
            path: PathBuf::from(path),
            text: text.to_owned(),
            error,
        }
    }
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
            if i == 0 && name.to_unquoted_string_str().is_ok_and(|s| s == "ref") {
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
    json: &'raw nojson::RawJson<'text>,
    definitions: HashMap<String, VariableDefinition<'text, 'raw>>,
    references: BTreeMap<usize, String>,
    resolved: String,
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
        value: nojson::RawJsonValue<'text, 'raw>,
    ) -> Result<(), nojson::JsonParseError> {
        todo!("{}", value.as_raw_str())
    }

    fn resolve_env(
        &mut self,
        name: &str,
        def: nojson::RawJsonValue<'text, 'raw>,
        default: Option<nojson::RawJsonValue<'text, 'raw>>,
        is_json: bool,
    ) -> Result<(), nojson::JsonParseError> {
        let json = if let Ok(value) = std::env::var(name)
            && !value.is_empty()
        {
            if is_json {
                nojson::RawJsonOwned::parse(value)?
            } else {
                nojson::RawJsonOwned::parse(nojson::Json(value).to_string())?
            }
        } else if let Some(default) = default {
            default.extract().into_owned()
        } else {
            return Err(def.invalid("environment variable is not set"));
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
        if let Some(variable_name) = self.references.remove(&value.position()) {
            let def = &self.definitions[&variable_name];
            if value.position() < def.position() {
                return Err(value.invalid("variable reference appears before its definition"));
            }
            match def {
                VariableDefinition::Const { value, .. } => {
                    self.resolve_const(*value)?;
                    todo!()
                }
                VariableDefinition::Env {
                    def,
                    default,
                    is_json,
                } => {
                    self.resolve_env(&variable_name, *def, *default, *is_json)?;
                }
                VariableDefinition::Resolved { json, .. } => {
                    write!(self.resolved, "{json}").expect("infallible");
                }
            };
        } else if !self.contains_ref(value) {
            let end_position = value.position() + value.as_raw_str().len();
            self.resolved
                .push_str(&self.json.text()[self.last_position..end_position]);
        } else if let Ok(array) = value.to_array() {
            for value in array {
                self.resolve_value(value)?;
            }
        } else if let Ok(object) = value.to_object() {
            for (_, value) in object {
                self.resolve_value(value)?;
            }
        } else {
            panic!("bug");
        }
        self.last_position = end_position;
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
pub enum VariableDefinition<'text, 'raw> {
    Const {
        def: nojson::RawJsonValue<'text, 'raw>,
        value: nojson::RawJsonValue<'text, 'raw>,
    },
    Env {
        def: nojson::RawJsonValue<'text, 'raw>,
        default: Option<nojson::RawJsonValue<'text, 'raw>>,
        is_json: bool,
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
                is_json: value
                    .to_member("is_json")?
                    .map(bool::try_from)?
                    .unwrap_or_default(),
            }),
            _ => Err(ty.invalid("unknown variable type")),
        }
    }
}
