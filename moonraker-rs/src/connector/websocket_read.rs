use std::sync::Arc;

use fastwebsockets::{FragmentCollectorRead, Frame, OpCode};
use hyper::upgrade::Upgraded;
use hyper_util::rt::TokioIo;
use tokio::{
    io::ReadHalf,
    sync::{Mutex, broadcast::Sender},
};

use crate::{
    cache::Cache,
    connector::{
        read_deserialize::{
            JsonRpcResponse, MoonrakerEventParameters, MoonrakerNotifyProcStatUpdate,
        },
        websocket_write::OutboundMessage,
    },
    error::Error,
    moonraker_connection::{MoonrakerConnection, MoonrakerReply, WebsocketEvent},
    printer_objects::*,
};

pub(crate) async fn moonraker_reader_connection_loop(
    inbound_sender: Sender<Arc<WebsocketEvent>>,
    outbound_sender: Sender<Arc<OutboundMessage>>,
    websocket_reader: FragmentCollectorRead<ReadHalf<TokioIo<Upgraded>>>,
    cache: Arc<Mutex<Cache>>,
) {
    let mut data =
        MoonrakerConnectionReadLoop::new(inbound_sender, outbound_sender, websocket_reader, cache);
    data.connection_loop().await;
}

struct MoonrakerConnectionReadLoop {
    inbound_sender: Sender<Arc<WebsocketEvent>>,
    outbound_sender: Sender<Arc<OutboundMessage>>,
    websocket_reader: FragmentCollectorRead<ReadHalf<TokioIo<Upgraded>>>,
    cache: Arc<Mutex<Cache>>,
}

impl MoonrakerConnectionReadLoop {
    pub fn new(
        inbound_sender: Sender<Arc<WebsocketEvent>>,
        outbound_sender: Sender<Arc<OutboundMessage>>,
        websocket_reader: FragmentCollectorRead<ReadHalf<TokioIo<Upgraded>>>,
        cache: Arc<Mutex<Cache>>,
    ) -> Self {
        Self {
            inbound_sender,
            outbound_sender,
            websocket_reader,
            cache,
        }
    }

    pub async fn connection_loop(&mut self) {
        loop {
            let outbound_sender = self.outbound_sender.clone();
            let mut frame = match self
                .websocket_reader
                .read_frame(&mut move |x| {
                    outbound_sender
                        .send(Arc::new(OutboundMessage::RawFrame(Mutex::new(Some(x)))))
                        .expect("Failed to internally send a raw frame event");

                    async move { Ok::<(), std::io::Error>(()) }
                })
                .await
            {
                Ok(frame) => frame,
                Err(e) => {
                    // Assume connection lost
                    self.inbound_sender
                        .send(Arc::new(WebsocketEvent::Error(e.to_string())))
                        .expect("Failed to internally send an error event");
                    self.inbound_sender
                        .send(Arc::new(WebsocketEvent::Disconnected))
                        .expect("Failed to internally send a disconnect event");
                    self.outbound_sender
                        .send(Arc::new(OutboundMessage::EndLoop))
                        .expect("Failed to internally send an endloop event");
                    break;
                }
            };

            if let Err(e) = match frame.opcode {
                OpCode::Close => self.on_frame_close(&mut frame).await,
                OpCode::Text => self.on_frame_text(&mut frame).await,
                _ => Err(Error::Unknown(format!(
                    "Received unsupported websocket frame {:?}",
                    frame.opcode
                ))),
            } {
                // TODO: Erorr handling
                eprintln!("Failed to process websocket event: {:?}", e);
            }
        }
    }

    pub async fn on_frame_close(&mut self, _: &mut Frame<'static>) -> Result<(), Error> {
        self.inbound_sender
            .send(Arc::new(WebsocketEvent::Disconnected))
            .expect("Failed to internally send a disconnect event");
        self.outbound_sender
            .send(Arc::new(OutboundMessage::EndLoop))
            .expect("Failed to internally send an endloop event");

        Ok(())
    }

    pub async fn on_frame_text(&mut self, frame: &mut Frame<'static>) -> Result<(), Error> {
        let payload = String::from_utf8(frame.payload.to_vec()).expect("Invalid UTF-8 data");

        #[cfg(debug_assertions)]
        println!("Received text frame: {}", payload);

        let data = serde_json::from_str::<JsonRpcResponse>(&payload)?;

        match data {
            JsonRpcResponse::MethodResponse(method_response) => {
                if method_response.error.is_some() {
                    eprintln!(
                        "Received error in method response: {:?}",
                        method_response.error
                    );
                } else {
                    let reply = MoonrakerReply {
                        id: method_response.id,
                        result: method_response.result.unwrap_or(serde_json::json!(null)),
                    };
                    let _ = self
                        .inbound_sender
                        .send(Arc::new(WebsocketEvent::MoonrakerReply(reply)));
                }
            }
            JsonRpcResponse::Notification(notification) => {
                match notification.params {
                    MoonrakerEventParameters::NotifyStatusUpdate(status_update) => {
                        for event in status_update.events {
                            let mut unlocked_cache = self.cache.lock().await;
                            self.inbound_sender
                                .send(Arc::new(WebsocketEvent::MoonrakerEvent(
                                    MoonrakerEvent::NotifyStatusUpdate(
                                        unlocked_cache.complete_event(event),
                                    ),
                                )))
                                .expect(
                                    "Failed to internally send a moonraker status update event",
                                );
                        }
                    }
                    MoonrakerEventParameters::NotifyProcessStatisticsUpdate(proc_stat_update) => {
                        self.inbound_sender.send(Arc::new(WebsocketEvent::MoonrakerEvent(MoonrakerEvent::NotifyProcessStatisticsUpdate(proc_stat_update)))).expect("Failed to internally send a moonraker process statistics update event");
                    }
                }
            }
        }

        Ok(())
    }
}

#[derive(Debug)]
pub enum MoonrakerEvent {
    NotifyStatusUpdate(PrinterEvent),
    NotifyProcessStatisticsUpdate(MoonrakerNotifyProcStatUpdate),
}

#[derive(Debug)]
pub enum PrinterEvent {
    Webhooks(Webhooks),
    MotionReport(MotionReport),
    GcodeMove(GcodeMove),
    Toolhead(Toolhead),
    Extruder(Extruder),
    HeaterBed(HeaterBed),
    Fan(Fan),
    IdleTimeout(IdleTimeout),
    VirtualSdcard(VirtualSdcard),
    PrintStats(PrintStats),
    DisplayStatus(DisplayStatus),
    TemperatureSensor(NamedTemperatureSensor),
    TemperatureFan(NamedTemperatureFan),
    FilamentSwitchSensor(NamedFilamentSwitchSensor),
    OutputPin(NamedOutputPin),
    ExcludeObject(ExcludeObject),
}
