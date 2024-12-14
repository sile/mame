use std::path::PathBuf;

use clap::Args;
use jsonlrpc::RequestId;
use log::LevelFilter;
use orfail::OrFail;
use simplelog::{ConfigBuilder, WriteLogger};

use crate::{
    editor::Editor,
    rpc::{self, OpenParams, Request},
};

#[derive(Debug, Clone, Args)]
pub struct RunCommand {
    pub path: Option<PathBuf>,

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
        let addr = editor.addr();
        if let Some(path) = self.path {
            std::thread::spawn(move || {
                let request = Request::Open {
                    jsonrpc: jsonlrpc::JsonRpcVersion::V2,
                    id: RequestId::Number(0),
                    params: OpenParams { path },
                };
                if rpc::call::<serde_json::Value>(addr, &request).is_err() {
                    // TODO: show error message
                    let request = Request::Exit {
                        jsonrpc: jsonlrpc::JsonRpcVersion::V2,
                    };
                    let _ = rpc::cast(addr, &request);
                }
            });
        }

        editor.run().or_fail()?;
        Ok(())
    }
}
