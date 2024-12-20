use std::path::PathBuf;

use clap::Args;
use jsonlrpc::RequestId;
use log::LevelFilter;
use orfail::OrFail;
use simplelog::{ConfigBuilder, WriteLogger};

use crate::{
    editor::Editor,
    rpc::{self, OpenParams, Request, StartLspParams},
};

#[derive(Debug, Clone, Args)]
pub struct RunCommand {
    pub path: Option<PathBuf>,

    #[clap(long)]
    pub logfile: Option<PathBuf>,

    #[clap(long, default_value_t=LevelFilter::Info)]
    pub loglevel: LevelFilter,
    // TODO
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

                // TODO
                let request = Request::StartLsp {
                    jsonrpc: jsonlrpc::JsonRpcVersion::V2,
                    id: RequestId::Number(1),
                    params: StartLspParams {
                        name: "erlls".to_owned(),
                        root_dir: PathBuf::from("file:///../../erlang/jsone/"),
                        command: PathBuf::from("erlls"),
                        args: Vec::new(),
                        env: [("RUST_LOG".to_owned(), "debug".to_owned())]
                            .into_iter()
                            .collect(),
                    },
                };
                if rpc::call::<serde_json::Value>(addr, &request).is_err() {
                    // TODO: show error message
                    let request = Request::Exit {
                        jsonrpc: jsonlrpc::JsonRpcVersion::V2,
                    };
                    let _ = rpc::cast(addr, &request);
                }

                log::info!("Initialized");
            });
        }

        editor.run().or_fail()?;
        Ok(())
    }
}
