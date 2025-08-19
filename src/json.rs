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

impl std::fmt::Display for LoadJsonFileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            LoadJsonFileError::Io { path, error } => {
                write!(f, "failed to read file '{}': {error}", path.display())
            }
            LoadJsonFileError::Json { path, error, text } => {
                write!(
                    f,
                    "failed to parse JSON from file '{}': {error}",
                    path.display(),
                )?;
                write!(f, "{}", format_json_error_context(error, text))
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

fn format_json_error_context(error: &nojson::JsonParseError, text: &str) -> String {
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

    format!(
        "\n\nINPUT:{}\n{:4} |{}\n|{:>column$} error",
        if let Some(prev) = prev_display_line {
            format!("\n     |{prev}")
        } else {
            "".to_owned()
        },
        line_num,
        display_line,
        "^",
        column = display_column
    )
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
