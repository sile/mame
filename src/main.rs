use std::path::PathBuf;

use jsonlrpc::RequestId;
use mame::{
    editor::Editor,
    rpc::{self, OpenParams, Request, StartLspParams},
};
use orfail::OrFail;

fn main() -> orfail::Result<()> {
    let port = 4343; // TODO

    // TODO
    // if let Some(logfile) = logfile {
    //     let _ = WriteLogger::init(
    //         self.loglevel,
    //         ConfigBuilder::new()
    //             .set_time_format_rfc3339()
    //             .set_time_offset_to_local()
    //             .unwrap_or_else(|b| b)
    //             .build(),
    //         std::fs::OpenOptions::new()
    //             .create(true)
    //             .append(true)
    //             .truncate(false)
    //             .open(logfile)
    //             .or_fail()?,
    //     );
    // }
    let path = std::env::args().nth(1).map(PathBuf::from);
    let editor = Editor::new(port).or_fail()?;
    let addr = editor.addr();
    // TODO: Logger::start(addr);
    if let Some(path) = path {
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
