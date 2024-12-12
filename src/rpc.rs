use crossterm::event::Event as TerminalEvent;
use jsonlrpc::JsonRpcVersion;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum Request {
    NotifyTerminalEvent {
        jsonrpc: JsonRpcVersion,
        params: NotifyTerminalEventParams,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyTerminalEventParams {
    pub event: TerminalEvent,
}
