use std::collections::BTreeMap;
use std::io::Write;
use std::path::PathBuf;

use crate::io_error;

#[derive(Debug, Clone)]
pub struct ExternalCommand {
    pub name: PathBuf,
    pub args: Vec<String>,
    pub envs: BTreeMap<String, String>,
    pub stdin: ExternalCommandInput,
    pub stdout: ExternalCommandOutput,
    pub stderr: ExternalCommandOutput,
}

impl ExternalCommand {
    pub fn execute(&self) -> std::io::Result<std::process::Output> {
        let mut cmd = std::process::Command::new(&self.name);
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
            let name = self.name.display();
            io_error(e, &format!("failed to execute command '{name}'"))
        })?;

        self.stdin.handle_input(child.stdin.take()).map_err(|e| {
            let name = self.name.display();
            io_error(e, &format!("failed to write stdin to command '{name}'"))
        })?;
        let output = child.wait_with_output().map_err(|e| {
            let name = self.name.display();
            io_error(e, &format!("failed to wait for command '{name}'"))
        })?;
        self.stdout.handle_output(&output.stdout).map_err(|e| {
            let name = self.name.display();
            io_error(e, &format!("failed to handle stdout from command '{name}'"))
        })?;
        self.stderr.handle_output(&output.stderr).map_err(|e| {
            let name = self.name.display();
            io_error(e, &format!("failed to handle stderr from command '{name}'"))
        })?;

        Ok(output)
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ExternalCommand {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        Ok(Self {
            name: value.to_member("name")?.required()?.try_into()?,
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

#[derive(Debug, Default, Clone)]
pub enum ExternalCommandInput {
    #[default]
    Null,
    Text {
        text: String,
    },
    File {
        path: PathBuf,
    },
}

impl ExternalCommandInput {
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

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ExternalCommandInput {
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

#[derive(Debug, Default, Clone)]
pub enum ExternalCommandOutput {
    #[default]
    Null,
    File {
        path: PathBuf,
        append: bool,
        skip_if_empty: bool,
    },
}

impl ExternalCommandOutput {
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
                    .append(*append)
                    .open(path)?;
                file.write_all(output)?;
                Ok(())
            }
        }
    }
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ExternalCommandOutput {
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

#[derive(Debug, Clone)]
pub struct ShellCommand(ExternalCommand);

impl ShellCommand {
    pub fn execute(&self) -> std::io::Result<std::process::Output> {
        self.0.execute()
    }

    pub fn get(&self) -> &ExternalCommand {
        &self.0
    }

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
            name: value
                .to_member("name")?
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
