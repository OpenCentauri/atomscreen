use crate::cache::Cache;
use crate::connector::websocket_read::{MoonrakerEvent, moonraker_reader_connection_loop};
use crate::connector::websocket_write::{
    MoonrakerRequest, OutboundMessage, moonraker_writer_connection_loop,
};
use crate::requests::PrinterAdministrationRequestHandler;
use fastwebsockets::handshake;
use fastwebsockets::{FragmentCollectorRead, WebSocketWrite};
use http_body_util::Empty;
use hyper::{Request, body::Bytes, header, upgrade::Upgraded};
use serde::de::DeserializeOwned;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;
use std::fmt::Debug;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{Mutex, broadcast};
use tokio::time::sleep;

use hyper_util::rt::TokioIo;

struct SpawnExecutor;

impl<Fut> hyper::rt::Executor<Fut> for SpawnExecutor
where
    Fut: Future + Send + 'static,
    Fut::Output: Send + 'static,
{
    fn execute(&self, fut: Fut) {
        tokio::task::spawn(fut);
    }
}

#[derive(Debug)]
pub struct MoonrakerReply {
    pub id: u32,
    pub result: serde_json::Value, // TODO: Make type safe (by checking method name)
}

#[derive(Debug, Clone)]
pub struct MoonrakerErrorReply
{
    pub code: i32,
    pub message: String,
    pub id: u32,
}

#[derive(Debug)]
pub enum WebsocketEvent {
    Connected,
    Disconnected,
    Error(String),
    MoonrakerEvent(MoonrakerEvent),
    MoonrakerReply(MoonrakerReply),
    MoonrakerErrorReply(MoonrakerErrorReply),
}

pub struct PrinterObjectsSubscribeParams {
    pub objects: serde_json::Map<String, Value>,
}

impl PrinterObjectsSubscribeParams {
    pub fn all_fields(objects: Vec<String>) -> Self {
        let mut map = serde_json::Map::new();
        for object in objects {
            map.insert(object, serde_json::Value::Null);
        }
        Self { objects: map }
    }
}

pub struct MoonrakerConnection {
    host: String,
    request: Request<Empty<Bytes>>,
    inbound_event_sender: Sender<Arc<WebsocketEvent>>,
    inbound_event_listener: Receiver<Arc<WebsocketEvent>>,
    outbound_event_sender: Sender<Arc<OutboundMessage>>,
    outbound_event_listener: Receiver<Arc<OutboundMessage>>,
    incrementing_id: Mutex<u32>,
    preconfigured_cache: Option<Cache>,
}

impl MoonrakerConnection {
    pub fn new(host: &str, port: u16, preconfigured_cache : Option<Cache>) -> Self {
        let host = format!("{}:{}", host, port);

        let req = Request::builder()
            .method("GET")
            .uri("/websocket")
            .header("Host", &host)
            .header(header::UPGRADE, "websocket")
            .header(header::CONNECTION, "upgrade")
            .header(
                "Sec-WebSocket-Key",
                fastwebsockets::handshake::generate_key(),
            )
            .header("Sec-WebSocket-Version", "13")
            .body(Empty::<Bytes>::new())
            .unwrap();

        // Moonraker -> us. TX = ws event bus, RX = misc listeners
        let (inbound_event_sender, inbound_event_listener) =
            broadcast::channel::<Arc<WebsocketEvent>>(128);

        // Us -> Moonraker. TX = send requests, RX = ws writer
        let (outbound_event_sender, outbound_event_listener) =
            broadcast::channel::<Arc<OutboundMessage>>(128);

        MoonrakerConnection {
            host: host,
            request: req,
            inbound_event_sender: inbound_event_sender,
            inbound_event_listener: inbound_event_listener,
            outbound_event_sender: outbound_event_sender,
            outbound_event_listener: outbound_event_listener,
            incrementing_id: Mutex::new(1),
            preconfigured_cache
        }
    }

    pub async fn new_id(&self) -> u32 {
        let mut id = self.incrementing_id.lock().await;
        let current_id = *id;
        *id += 1;
        current_id
    }

    pub async fn connection_loop(&self) {
        loop {
            // TODO: Kill old threads if they exist
            let inbound_sender = self.inbound_event_sender.clone();
            inbound_sender
                .send(Arc::new(WebsocketEvent::Disconnected))
                .expect("Failed to internally send a disconnect event");
            let reader;
            let writer;
            let cache = Arc::new(Mutex::new(self.preconfigured_cache.clone().unwrap_or_default()));

            match self.reconnect().await {
                Ok((r, w)) => {
                    reader = r;
                    writer = w;
                    let _ = inbound_sender.send(Arc::new(WebsocketEvent::Connected));
                }
                Err(e) => {
                    eprintln!("Error connecting to Moonraker: {}", e);
                    sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            }

            let reader_handle = {
                let inbound_sender = self.inbound_event_sender.clone();
                let outbound_sender = self.outbound_event_sender.clone();
                let cache = cache.clone();
                tokio::spawn(async move {
                    moonraker_reader_connection_loop(
                        inbound_sender,
                        outbound_sender,
                        reader,
                        cache,
                    )
                    .await;
                })
            };

            let writer_handle = {
                let outbound_receiver = self.outbound_event_listener.resubscribe();
                tokio::spawn(async move {
                    moonraker_writer_connection_loop(outbound_receiver, writer).await;
                })
            };

            let object_list = match self.list_printer_objects().await {
                Ok(object_list) => object_list,
                Err(e) => {
                    eprintln!("Error getting printer object list: {}", e);
                    reader_handle.abort();
                    writer_handle.abort();
                    continue;
                }
            };

            // TODO: Wait until Klippy is ready

            // TOOD: Don't subscribe to objects we don't have a use for.
            let initial_objects = self
                .subscribe_to_printer_objects(object_list.objects.clone())
                .await;
            if let Err(e) = initial_objects {
                eprintln!("Error subscribing to printer objects: {}", e);
                reader_handle.abort();
                writer_handle.abort();
                continue;
            }

            for event in initial_objects.unwrap().status.events {
                let mut unlocked_cache = cache.lock().await;
                let _ = inbound_sender.send(Arc::new(WebsocketEvent::MoonrakerEvent(
                    MoonrakerEvent::NotifyStatusUpdate(unlocked_cache.complete_event(event)),
                )));
            }

            reader_handle.await.unwrap();
            writer_handle.await.unwrap();
            sleep(Duration::from_secs(2)).await;
        }
    }

    pub async fn send_request<T>(
        &self,
        method: &str,
        args: Option<serde_json::Value>,
    ) -> Result<T, crate::error::Error>
    where
        T: DeserializeOwned,
    {
        let mut listener = self.inbound_event_listener.resubscribe();
        let id = self.new_id().await;

        let event = Arc::new(OutboundMessage::MoonrakerRequest(MoonrakerRequest {
            id: id,
            method: method.to_string(),
            args: args,
        }));
        let _ = self.outbound_event_sender.send(event);

        loop {
            let event = listener
                .recv()
                .await
                .expect("Failed to retrieve internal event");
            
                let now = Instant::now();
            match &*event {
                WebsocketEvent::MoonrakerReply(reply) if reply.id == id => {
                    let parsed_result: Result<T, serde_json::Error> =
                        serde_json::from_value(reply.result.clone());

                    match parsed_result {
                        Ok(result) => return Ok(result),
                        Err(e) => return Err(crate::error::Error::UnsupportedMessage(e)),
                    }
                }

                WebsocketEvent::MoonrakerErrorReply(error) if error.id == id => {
                    return Err(crate::error::Error::MoonrakerErrorReply(error.code, error.message.clone()));
                }
                _ => {
                    if now.elapsed().as_secs() > 20 
                    {
                        return Err(crate::error::Error::Timeout);
                    }
                    else {
                        continue;
                    }
                }, // TODO: This should eventually end
            }
        }
    }

    pub async fn download_thumbnail(
        &self,
        thumbnail_filename: &str,
    ) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let url = format!(
            "http://{}/server/files/gcodes/{}",
            self.host, thumbnail_filename
        );

        let body = reqwest::get(&url).await?.bytes().await?;

        Ok(body.to_vec())
    }

    pub async fn reconnect(
        &self,
    ) -> Result<
        (
            FragmentCollectorRead<ReadHalf<TokioIo<Upgraded>>>,
            WebSocketWrite<WriteHalf<TokioIo<Upgraded>>>,
        ),
        Box<dyn Error + Send + Sync>,
    > {
        let stream = TcpStream::connect(self.host.clone()).await?;

        let (ws, _) = handshake::client(&SpawnExecutor, self.request.clone(), stream).await?;
        let (rx, tx) = ws.split(tokio::io::split);
        let reader = FragmentCollectorRead::new(rx);

        Ok((reader, tx))
    }

    pub fn get_listener(&self) -> Receiver<Arc<WebsocketEvent>> {
        self.inbound_event_listener.resubscribe()
    }
}
