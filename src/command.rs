//! External command execution with configurable I/O handling.
//!
//! This module provides utilities for executing external commands and shell scripts
//! with fine-grained control over stdin, stdout, and stderr. Commands can be configured
//! to read input from text or files, and write output to files with various options.
use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use crate::io_error;

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
        self.stdout.handle_output(&output.stdout).map_err(|e| {
            let name = self.command.display();
            io_error(e, &format!("failed to handle stdout from command '{name}'"))
        })?;
        self.stderr.handle_output(&output.stderr).map_err(|e| {
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
            command: value.to_member("command")?.required()?.try_into()?,
            args: value
                .to_member("args")?
                .map(Vec::try_from)?
                .unwrap_or_default(),
            envs: value
                .to_member("envs")?
                .map(BTreeMap::try_from)?
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
                text: value.to_member("text")?.required()?.try_into()?,
            }),
            "file" => Ok(Self::File {
                path: value.to_member("path")?.required()?.try_into()?,
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
    },
}

impl CommandOutput {
    fn handle_output(&self, output: &[u8]) -> std::io::Result<()> {
        match self {
            Self::Null => Ok(()),
            Self::File {
                path,
                append,
                skip_if_empty,
            } => {
                if *skip_if_empty && output.is_empty() {
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
                path: value.to_member("path")?.required()?.try_into()?,
                append: value
                    .to_member("append")?
                    .map(bool::try_from)?
                    .unwrap_or_default(),
                skip_if_empty: value
                    .to_member("skip_if_empty")?
                    .map(bool::try_from)?
                    .unwrap_or_default(),
            }),
            _ => Err(ty.invalid("unknown stdin type")),
        }
    }
}

/// Wrapper for executing shell commands and scripts with configurable I/O handling.
///
/// `ShellCommand` is a specialized version of `ExternalCommand` that automatically
/// configures the shell (using the `SHELL` environment variable or defaulting to "sh")
/// and formats script arguments with the `-c` flag for shell execution.
#[derive(Debug, Clone)]
pub struct ShellCommand(ExternalCommand);

impl ShellCommand {
    /// Executes the shell command with configured I/O handling.
    ///
    /// Delegates to the underlying `ExternalCommand::execute()` method to spawn
    /// the shell process, handle stdin/stdout/stderr according to configuration,
    /// and return the complete process output.
    pub fn execute(&self) -> std::io::Result<std::process::Output> {
        self.0.execute()
    }

    /// Returns a reference to the underlying `ExternalCommand` configuration.
    pub fn get(&self) -> &ExternalCommand {
        &self.0
    }

    /// Returns a mutable reference to the underlying `ExternalCommand` configuration.
    pub fn get_mut(&mut self) -> &mut ExternalCommand {
        &mut self.0
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ShellCommand {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let mut args = vec!["-c".to_owned()];
        for script in value.to_member("script")?.required()?.to_array()? {
            args.push(script.try_into()?);
        }

        Ok(Self(ExternalCommand {
            command: value
                .to_member("shell")?
                .map(PathBuf::try_from)?
                .unwrap_or_else(|| {
                    PathBuf::from(std::env::var("SHELL").unwrap_or_else(|_| "sh".to_owned()))
                }),
            args,
            envs: value
                .to_member("envs")?
                .map(BTreeMap::try_from)?
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
        }))
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
            if arg.is_empty() {
                write!(f, " ''")?;
            } else if arg
                .chars()
                .all(|c| c.is_alphanumeric() || "-_./=".contains(c))
            {
                write!(f, " {arg}")?;
            } else {
                // Shell escaping: replace single quotes with '"'"' to safely quote arguments
                write!(f, " '{}'", arg.replace('\'', r#"'"'"'"#))?;
            }
        }
        Ok(())
    }
}
