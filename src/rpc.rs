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

use crate::{buffer::BufferId, lsp::SemanticTokenType};

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
    Save {
        jsonrpc: JsonRpcVersion,
        #[serde(default)]
        id: Option<RequestId>,
        #[serde(default)]
        params: SaveParams,
    },
    Mark {
        jsonrpc: JsonRpcVersion,
    },
    Cut {
        jsonrpc: JsonRpcVersion,
    },
    Copy {
        jsonrpc: JsonRpcVersion,
    },
    Paste {
        jsonrpc: JsonRpcVersion,
    },
    Cancel {
        jsonrpc: JsonRpcVersion,
    },
    Exit {
        jsonrpc: JsonRpcVersion,
    },
    StartLsp {
        jsonrpc: JsonRpcVersion,
        id: RequestId,
        params: StartLspParams,
    },
    MoveTo {
        jsonrpc: JsonRpcVersion,
        #[serde(default)]
        id: Option<RequestId>,
        params: MoveToParams,
    },
    MoveDelta {
        jsonrpc: JsonRpcVersion,
        #[serde(default)]
        id: Option<RequestId>,
        params: MoveDeltaParams,
    },

    // Internal
    NotifyLspStarted {
        jsonrpc: JsonRpcVersion,
        params: NotifyLspStartedParams,
    },
    NotifySemanticTokens {
        jsonrpc: JsonRpcVersion,
        params: NotifySemanticTokensParams,
    },
    Command {
        jsonrpc: JsonRpcVersion,
        #[serde(default)]
        id: Option<RequestId>,
        params: CommandParams,
    },
}

impl Request {
    pub fn save() -> Self {
        Self::Save {
            jsonrpc: JsonRpcVersion::V2,
            id: None,
            params: SaveParams {},
        }
    }

    pub fn mark() -> Self {
        Self::Mark {
            jsonrpc: JsonRpcVersion::V2,
        }
    }

    pub fn copy() -> Self {
        Self::Copy {
            jsonrpc: JsonRpcVersion::V2,
        }
    }

    pub fn cut() -> Self {
        Self::Cut {
            jsonrpc: JsonRpcVersion::V2,
        }
    }

    pub fn paste() -> Self {
        Self::Paste {
            jsonrpc: JsonRpcVersion::V2,
        }
    }

    pub fn move_to(row: Option<u32>, col: Option<u32>) -> Self {
        Self::MoveTo {
            jsonrpc: JsonRpcVersion::V2,
            id: None,
            params: MoveToParams { row, col },
        }
    }

    pub fn move_delta(row: i16, col: i16) -> Self {
        Self::MoveDelta {
            jsonrpc: JsonRpcVersion::V2,
            id: None,
            params: MoveDeltaParams { row, col },
        }
    }

    pub fn cancel() -> Self {
        Self::Cancel {
            jsonrpc: JsonRpcVersion::V2,
        }
    }

    pub fn exit() -> Self {
        Self::Exit {
            jsonrpc: JsonRpcVersion::V2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifyTerminalEventParams {
    pub event: TerminalEvent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenParams {
    pub path: PathBuf,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct SaveParams {
    // TODO: Optional buffer name
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SaveReturnValue {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MoveToParams {
    #[serde(default)]
    pub row: Option<u32>,
    #[serde(default)]
    pub col: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveToReturnValue {}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct MoveDeltaParams {
    #[serde(default)]
    pub row: i16,
    #[serde(default)]
    pub col: i16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoveDeltaReturnValue {}

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

impl From<orfail::Failure> for RpcError {
    fn from(value: orfail::Failure) -> Self {
        Self::other(&value.to_string())
    }
}

pub type RpcResult<T> = Result<T, RpcError>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotifySemanticTokensParams {
    pub buffer_id: BufferId,
    // TODO: version
    pub tokens: Vec<SemanticToken>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticToken {
    pub line: usize,
    pub column: usize,
    pub token_len: usize,
    pub token_type: SemanticTokenType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandParams {
    pub path: PathBuf,
    #[serde(default)]
    pub args: Vec<String>,
}
