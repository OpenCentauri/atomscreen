use fastwebsockets::WebSocketError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Got an unsupported moonraker message from the websocket")]
    UnsupportedMessage(#[from] serde_json::Error),
    #[error("Failed to write message to websocket")]
    WebsocketWriteError(#[from] WebSocketError),
    #[error("Moonraker returned an error reply")]
    MoonrakerErrorReply(i32, String),
    #[error("Unknown error")]
    Unknown(String),
    #[error("Internally used")]
    BreakError,
    #[error("Timeout")]
    Timeout,
}
