use std::path::PathBuf;

use clap::Args;
use log::LevelFilter;
use orfail::OrFail;
use simplelog::{ConfigBuilder, WriteLogger};

use crate::editor::Editor;

#[derive(Debug, Clone, Args)]
pub struct RunCommand {
    #[clap(long)]
    pub logfile: Option<PathBuf>,

    #[clap(long, default_value_t=LevelFilter::Info)]
    pub loglevel: LevelFilter,
}

impl RunCommand {
    pub fn run(self, port: u16) -> orfail::Result<()> {
        if let Some(logfile) = self.logfile {
            let _ = WriteLogger::init(
                self.loglevel,
                ConfigBuilder::new()
                    .set_time_format_rfc3339()
                    .set_time_offset_to_local()
                    .unwrap_or_else(|b| b)
                    .build(),
                std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .truncate(false)
                    .open(logfile)
                    .or_fail()?,
            );
        }

        let editor = Editor::new(port).or_fail()?;
        editor.run().or_fail()?;
        Ok(())
    }
}
