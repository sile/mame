use std::{
    collections::BTreeMap,
    net::{SocketAddr, TcpStream},
    path::PathBuf,
};

use crossterm::event::Event as TerminalEvent;
use jsonlrpc::{JsonRpcVersion, RequestId, ResponseObject, RpcClient};
use jsonlrpc_mio::ClientId;
use orfail::OrFail;
use serde::{Deserialize, Serialize};

pub fn call<T>(server_addr: SocketAddr, request: &Request) -> orfail::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let stream = TcpStream::connect(server_addr).or_fail()?;
    let mut client = RpcClient::new(stream);
    let response: ResponseObject = client.call(request).or_fail()?;
    let result = response
        .into_std_result()
        .map_err(|e| orfail::Failure::new(serde_json::to_string(&e).expect("unreachable")))?;
    serde_json::from_value(result).or_fail()
}

pub fn cast(server_addr: SocketAddr, request: &Request) -> orfail::Result<()> {
    let stream = TcpStream::connect(server_addr).or_fail()?;
    let mut client = RpcClient::new(stream);
    client.cast(request).or_fail()?;
    Ok(())
}

#[derive(Debug)]
pub struct Caller {
    pub client_id: ClientId,
    pub request_id: RequestId,
}

impl Caller {
    pub fn new(client_id: ClientId, request_id: RequestId) -> Self {
        Self {
            client_id,
            request_id,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "method")]
pub enum Request {
    NotifyTerminalEvent {
        jsonrpc: JsonRpcVersion,
        params: NotifyTerminalEventParams,
    },
    Open {
        jsonrpc: JsonRpcVersion,
        id: RequestId,
        params: OpenParams,
    },
    Exit {
        jsonrpc: JsonRpcVersion,
    },
    StartLsp {
        jsonrpc: JsonRpcVersion,
        id: RequestId,
        params: StartLspParams,
    },

    // Internal
    NotifyLspStarted {
        jsonrpc: JsonRpcVersion,
        params: NotifyLspStartedParams,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyTerminalEventParams {
    pub event: TerminalEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenParams {
    pub path: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenReturnValue {
    pub new: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartLspParams {
    pub name: String,
    pub root_dir: PathBuf,
    pub command: PathBuf,
    pub args: Vec<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartLspReturnValue {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyLspStartedParams {
    pub name: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum RpcError {
    FileError {
        path: PathBuf,
        kind: std::io::ErrorKind,
        reason: String,
    },
    IoError {
        kind: std::io::ErrorKind,
        reason: String,
    },
    Other {
        message: String,
    },
}

impl RpcError {
    pub fn other(message: &str) -> Self {
        Self::Other {
            message: message.to_owned(),
        }
    }

    pub fn file_error(path: PathBuf, error: std::io::Error) -> Self {
        Self::FileError {
            path,
            kind: error.kind(),
            reason: error.to_string(),
        }
    }
}

impl From<std::io::Error> for RpcError {
    fn from(value: std::io::Error) -> Self {
        Self::IoError {
            kind: value.kind(),
            reason: value.to_string(),
        }
    }
}

pub type RpcResult<T> = Result<T, RpcError>;
