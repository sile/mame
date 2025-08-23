use std::collections::BTreeMap;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct ExternalCommand {
    pub command: PathBuf,
    pub args: Vec<String>,
    pub envs: BTreeMap<String, String>,
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
        })
    }
}

impl nojson::DisplayJson for ExternalCommand {
    fn fmt(&self, f: &mut nojson::JsonFormatter<'_, '_>) -> std::fmt::Result {
        f.object(|f| {
            f.member("command", &self.command)?;
            if !self.args.is_empty() {
                f.member("args", &self.args)?;
            }
            if !self.envs.is_empty() {
                f.member("envs", &self.envs)?;
            }
            Ok(())
        })
    }
}

// TODO: ShellCommand
/*
#[derive(Debug)]
pub enum ExternalCommandStdio {
    Null,
    Text(String),
    File(PathBuf),
}
*/
// TODO: ExternalCommandError, ExternalCommandOutput::{File, String, ...}, AllowStatusCode
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
