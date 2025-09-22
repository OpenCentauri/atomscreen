use std::sync::Arc;

use fastwebsockets::{Frame, Payload, WebSocketWrite};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use serde::Serialize;
use tokio::{io::WriteHalf, sync::{broadcast::Receiver, Mutex}};
use std::fmt::Debug;

use crate::error::Error;

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest
{
    pub jsonrpc: String,
    pub method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    pub id: u32,
}

pub struct MoonrakerRequest
{
    pub id : u32,
    pub method : String,
    pub args : Option<serde_json::Value>
}

pub enum OutboundMessage
{
    EndLoop,
    RawFrame(Mutex<Option<Frame<'static>>>),
    MoonrakerRequest(MoonrakerRequest),
}

impl Debug for OutboundMessage
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OutboundMessage::EndLoop => write!(f, "OutboundMessage::EndLoop"),
            OutboundMessage::RawFrame(_) => write!(f, "OutboundMessage::RawFrame"),
            OutboundMessage::MoonrakerRequest(req) => write!(f, "OutboundMessage::MoonrakerRequest id: {}, method: {}", req.id, req.method),
        }
    }
}

pub(crate) async fn moonraker_writer_connection_loop(
    outbound_receiver : Receiver<Arc<OutboundMessage>>,
    websocket_writer : WebSocketWrite<WriteHalf<TokioIo<Upgraded>>>) {
    let mut data = MoonrakerConnectionWriteLoop::new(outbound_receiver, websocket_writer);
    data.connection_loop().await;
}

struct MoonrakerConnectionWriteLoop {
    outbound_receiver : Receiver<Arc<OutboundMessage>>,
    websocket_writer : WebSocketWrite<WriteHalf<TokioIo<Upgraded>>>,
}

impl MoonrakerConnectionWriteLoop {
    pub fn new(
        outbound_receiver : Receiver<Arc<OutboundMessage>>,
        websocket_writer : WebSocketWrite<WriteHalf<TokioIo<Upgraded>>>) -> Self {
            Self {
                outbound_receiver,
                websocket_writer,
            }
    }

    pub async fn connection_loop(&mut self)
    {
        loop {
            let message = self.outbound_receiver.recv().await.unwrap();

            if let Err(e) = match &*message
            {
                OutboundMessage::EndLoop => break,
                OutboundMessage::RawFrame(frame) => self.handle_raw_frame(frame).await,
                OutboundMessage::MoonrakerRequest(request) => self.handle_moonraker_request(request).await,
            }
            {
                // TODO: Handle error
                eprintln!("Failed to process outbound message: {:?}", e);
            }
        }
    }

    pub async fn handle_raw_frame(&mut self, frame : &Mutex<Option<Frame<'static>>>) -> Result<(), Error>
    {
        println!("Got raw frame to send");

        if let Some(frame) = frame.lock().await.take()
        {
            self.websocket_writer.write_frame(frame).await?;
        }

        Ok(())
    }

    pub async fn handle_moonraker_request(&mut self, request : &MoonrakerRequest) -> Result<(), Error>
    {
        let request = JsonRpcRequest {
            jsonrpc: "2.0".to_string(),
            method: request.method.clone(),
            params: request.args.clone(),
            id: request.id,
        };

        let data = serde_json::to_string(&request).unwrap();
        
        #[cfg(debug_assertions)]
        println!("Sending request: {}", data);

        let bytes = data.as_bytes().to_vec();

        self.websocket_writer.write_frame(Frame::text(Payload::Owned(bytes))).await?;
        Ok(())
    }
}