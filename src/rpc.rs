use std::{
    net::{SocketAddr, TcpStream},
    path::PathBuf,
};

use crossterm::event::Event as TerminalEvent;
use jsonlrpc::{JsonRpcVersion, RequestId, ResponseObject, RpcClient};
use jsonlrpc_mio::ClientId;
use orfail::OrFail;
use serde::{Deserialize, Serialize};

pub fn call<T>(server_addr: SocketAddr, request: Request) -> orfail::Result<T>
where
    T: for<'de> Deserialize<'de>,
{
    let stream = TcpStream::connect(server_addr).or_fail()?;
    let mut client = RpcClient::new(stream);
    let response: ResponseObject = client.call(&request).or_fail()?;
    let result = response
        .into_std_result()
        .map_err(|e| orfail::Failure::new(serde_json::to_string(&e).expect("unreachable")))?;
    serde_json::from_value(result).or_fail()
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
pub struct OpenResult;
