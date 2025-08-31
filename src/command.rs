//! External command execution with configurable I/O handling.
//!
//! This module provides utilities for executing external commands and shell scripts
//! with fine-grained control over stdin, stdout, and stderr. Commands can be configured
//! to read input from text or files, and write output to files with various options.
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use crate::io_error;
use crate::json;

/// Configuration for executing an external command with customizable I/O handling.
#[derive(Debug, Clone)]
pub struct ExternalCommand {
    /// Path to the executable command
    pub command: PathBuf,

    /// Command line arguments to pass to the executable
    pub args: Vec<String>,

    /// Environment variables to set for the command execution
    pub envs: BTreeMap<String, String>,

    /// Configuration for handling stdin input
    pub stdin: CommandInput,

    /// Configuration for handling stdout output
    pub stdout: CommandOutput,

    /// Configuration for handling stderr output
    pub stderr: CommandOutput,
}

impl ExternalCommand {
    /// Executes the external command with configured I/O handling.
    ///
    /// Spawns the process with the specified arguments and environment variables,
    /// handles stdin input, waits for completion, and processes stdout/stderr
    /// according to the configured output settings.
    ///
    /// Returns the complete process output including exit status and captured streams.
    pub fn execute(&self) -> std::io::Result<std::process::Output> {
        let mut cmd = std::process::Command::new(&self.command);
        for arg in &self.args {
            cmd.arg(arg);
        }
        for (k, v) in &self.envs {
            cmd.env(k, v);
        }
        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        let mut child = cmd.spawn().map_err(|e| {
            let name = self.command.display();
            io_error(e, &format!("failed to execute command '{name}'"))
        })?;

        self.stdin.handle_input(child.stdin.take()).map_err(|e| {
            let name = self.command.display();
            io_error(e, &format!("failed to write stdin to command '{name}'"))
        })?;
        let output = child.wait_with_output().map_err(|e| {
            let name = self.command.display();
            io_error(e, &format!("failed to wait for command '{name}'"))
        })?;

        let success = output.status.success();

        self.stdout
            .handle_output(&output.stdout, success)
            .map_err(|e| {
                let name = self.command.display();
                io_error(e, &format!("failed to handle stdout from command '{name}'"))
            })?;
        self.stderr
            .handle_output(&output.stderr, success)
            .map_err(|e| {
                let name = self.command.display();
                io_error(e, &format!("failed to handle stderr from command '{name}'"))
            })?;

        Ok(output)
    }

    /// Returns a command line representation that combines the command and args fields for display purposes.
    pub fn command_line(&self) -> impl '_ + std::fmt::Display {
        CommandLine {
            command: &self.command,
            args: &self.args,
        }
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ExternalCommand {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            command: json::parse_from_flattened_string(value.to_member("command")?.required()?)?,
            args: value
                .to_member("args")?
                .map(|v| {
                    v.to_array()?
                        .map(json::parse_from_flattened_string)
                        .collect()
                })?
                .unwrap_or_default(),
            envs: value
                .to_member("envs")?
                .map(|v| {
                    v.to_object()?
                        .map(|(k, v)| Ok((k.try_into()?, json::parse_from_flattened_string(v)?)))
                        .collect()
                })?
                .unwrap_or_default(),
            stdin: value
                .to_member("stdin")?
                .map(TryFrom::try_from)?
                .unwrap_or_default(),
            stdout: value
                .to_member("stdout")?
                .map(TryFrom::try_from)?
                .unwrap_or_default(),
            stderr: value
                .to_member("stderr")?
                .map(TryFrom::try_from)?
                .unwrap_or_default(),
        })
    }
}

/// Configuration for providing input to a command's stdin.
#[derive(Debug, Default, Clone)]
pub enum CommandInput {
    /// No input provided (default)
    #[default]
    Null,

    /// Input from a text string
    Text {
        /// The text content to write to stdin
        text: String,
    },

    /// Input from a file
    File {
        /// Path to the file whose contents will be piped to stdin
        path: PathBuf,
    },
}

impl CommandInput {
    fn handle_input<W: Write>(&self, writer: Option<W>) -> std::io::Result<()> {
        let Some(mut writer) = writer else {
            return Ok(());
        };
        match self {
            Self::Null => {}
            Self::Text { text } => {
                writer.write_all(text.as_bytes())?;
            }
            Self::File { path } => {
                let mut file = std::fs::File::open(path)?;
                std::io::copy(&mut file, &mut writer)?;
            }
        }
        Ok(())
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for CommandInput {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let ty = value.to_member("type")?.required()?;
        match ty.to_unquoted_string_str()?.as_ref() {
            "null" => Ok(Self::Null),
            "text" => Ok(Self::Text {
                text: json::parse_from_flattened_string(value.to_member("text")?.required()?)?,
            }),
            "file" => Ok(Self::File {
                path: json::parse_from_flattened_string(value.to_member("path")?.required()?)?,
            }),
            _ => Err(ty.invalid("unknown stdin type")),
        }
    }
}

/// Configuration for handling command output (stdout/stderr).
#[derive(Debug, Default, Clone)]
pub enum CommandOutput {
    /// Discard the output (default)
    #[default]
    Null,

    /// Write output to a file
    File {
        /// Path to the output file
        path: PathBuf,

        /// Whether to append to existing file content
        append: bool,

        /// Skip writing if the output is empty
        skip_if_empty: bool,

        /// Skip writing if the command executed successfully (exit code 0)
        skip_if_success: bool,
    },
}

impl CommandOutput {
    fn handle_output(&self, output: &[u8], success: bool) -> std::io::Result<()> {
        match self {
            Self::Null => Ok(()),
            Self::File {
                path,
                append,
                skip_if_empty,
                skip_if_success,
            } => {
                if *skip_if_empty && output.is_empty() {
                    return Ok(());
                }

                if *skip_if_success && success {
                    return Ok(());
                }

                let mut file = std::fs::OpenOptions::new()
                    .create(true)
                    .write(true)
                    .truncate(!*append)
                    .append(*append)
                    .open(path)?;
                file.write_all(output)?;
                Ok(())
            }
        }
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for CommandOutput {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let ty = value.to_member("type")?.required()?;
        match ty.to_unquoted_string_str()?.as_ref() {
            "null" => Ok(Self::Null),
            "file" => Ok(Self::File {
                path: json::parse_from_flattened_string(value.to_member("path")?.required()?)?,
                append: value
                    .to_member("append")?
                    .map(bool::try_from)?
                    .unwrap_or_default(),
                skip_if_empty: value
                    .to_member("skip-if-empty")?
                    .map(bool::try_from)?
                    .unwrap_or_default(),
                skip_if_success: value
                    .to_member("skip-if-success")?
                    .map(bool::try_from)?
                    .unwrap_or_default(),
            }),
            _ => Err(ty.invalid("unknown stdin type")),
        }
    }
}

struct CommandLine<'a> {
    command: &'a PathBuf,
    args: &'a [String],
}

impl<'a> std::fmt::Display for CommandLine<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.command.display())?;
        for arg in self.args {
            if arg.is_empty() || arg.chars().any(|c| c.is_control() || c.is_whitespace()) {
                write!(f, " {arg:?}")?;
            } else {
                write!(f, " {arg}")?;
            }
        }
        Ok(())
    }
}
