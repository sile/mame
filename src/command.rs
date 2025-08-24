use std::collections::BTreeMap;
use std::path::PathBuf;

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

        //let mut child = cmd.spawn()?;

        todo!()
    }
}

/*
{"type": "shell",
 "command": "echo $MAMEGRE_FILE > $HOME"}
    fn execute_command(&self, buffer: &str) -> orfail::Result<String> {

        let mut child = cmd
            .spawn()
            .or_fail_with(|e| format!("Failed to execute grep command: {e}"))?;

        if let Some(mut stdin) = child.stdin.take() {
            write!(stdin, "{buffer}").or_fail()?;
            stdin.flush().or_fail()?;
        }

        let output = child
            .wait_with_output()
            .or_fail_with(|e| format!("Failed to wait for command: {e}"))?;

        match output.status.code() {
            Some(0 | 1) => {}
            _ => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(orfail::Failure::new(format!(
                    "Grep command failed: {}",
                    stderr.trim()
                )));
            }
        }
        String::from_utf8(output.stdout).or_fail()
    }
*/

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
        skip_if_empty: bool,
    },
}

impl<'text, 'raw> TryFrom<nojson::RawJsonValue<'text, 'raw>> for ExternalCommandOutput {
    type Error = nojson::JsonParseError;

    fn try_from(value: nojson::RawJsonValue<'text, 'raw>) -> Result<Self, Self::Error> {
        let ty = value.to_member("type")?.required()?;
        match ty.to_unquoted_string_str()?.as_ref() {
            "null" => Ok(Self::Null),
            "file" => Ok(Self::File {
                path: value.to_member("path")?.required()?.try_into()?,
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
            name: value.to_member("name")?.required()?.try_into()?,
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
