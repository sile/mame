use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExternalCommand {
    pub name: PathBuf,
    pub args: Vec<String>,
    pub envs: BTreeMap<String, String>,
    // stdout, stderr, stdin
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
        })
    }
}

#[derive(Debug, Clone)]
pub enum ExternalCommandInput {
    Null,
    Text { text: String },
    File { path: PathBuf },
}

#[derive(Debug, Clone)]
pub enum ExternalCommandOutput {
    Null,
    File { path: PathBuf, skip_if_empty: bool },
}

#[derive(Debug, Clone)]
pub struct ShellCommand(ExternalCommand);

/*
{"type": "shell",
 "command": "echo $MAMEGRE_FILE > $HOME"}
    fn execute_command(&self, buffer: &str) -> orfail::Result<String> {
        let mut cmd = std::process::Command::new(&self.action.command);
        for arg in &self.action.args {
            cmd.arg(arg);
        }
        cmd.arg(self.query.iter().copied().collect::<String>());

        cmd.stdin(std::process::Stdio::piped());
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

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
