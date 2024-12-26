use std::{net::SocketAddr, path::PathBuf, sync::mpsc};

use log::{Metadata, Record};

use crate::rpc::{self, CommandParams};

#[derive(Debug)]
pub struct Logger {
    tx: mpsc::Sender<String>,
}

impl Logger {
    pub fn start(editor: SocketAddr) {
        let (tx, rx) = mpsc::channel();
        let logger = Logger { tx };
        log::set_boxed_logger(Box::new(logger))
            .map(|()| log::set_max_level(log::LevelFilter::Debug))
            .expect("TODO");
        std::thread::spawn(move || {
            while let Ok(msg) = rx.recv() {
                // TODO
                let request = rpc::Request::Command {
                    jsonrpc: jsonlrpc::JsonRpcVersion::V2,
                    id: None,
                    params: CommandParams {
                        path: PathBuf::from("tmux"),
                        args: vec!["display".to_owned(), "-d".to_owned(), "0".to_owned(), msg],
                    },
                };
                rpc::cast(editor, &request).expect("TODO");
            }
        });
    }
}

impl log::Log for Logger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        // TODO
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let msg = format!("{}", record.args());
            let _ = self.tx.send(msg);
        }
    }

    fn flush(&self) {}
}
