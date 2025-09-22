use fastwebsockets::WebSocketError;
use thiserror::Error;
use tokio::sync::broadcast::error::SendError;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Got an unsupported moonraker message from the websocket")]
    UnsupportedMessage(#[from] serde_json::Error),
    #[error("Failed to write message to websocket")]
    WebsocketWriteError(#[from] WebSocketError),
    #[error("Unknown error")]
    Unknown(String),
}
