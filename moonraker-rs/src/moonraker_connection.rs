use std::cell::RefCell;
use std::sync::Arc;
use std::{error::Error};
use fastwebsockets::{FragmentCollectorRead, Frame, OpCode, Payload, WebSocketWrite};
use fastwebsockets::{handshake, FragmentCollector, WebSocket};
use http_body_util::Empty;
use hyper::{body::Bytes, header, upgrade::Upgraded, Request};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::{ReadHalf, WriteHalf};
use tokio::net::TcpStream;
use tokio::sync::broadcast::{Receiver, Sender};
use tokio::sync::{broadcast, Mutex};
use tokio::time::sleep;
use crate::cache::Cache;

use super::printer_objects::*;
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

#[derive(Debug, Serialize)]
pub struct JsonRpcRequest
{
    jsonrpc: String,
    method: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    params: Option<serde_json::Value>,
    id: u32,
}

#[derive(Debug, Deserialize)]
pub struct JsonRpcMethodResponse
{
    jsonrpc: String,
    result: Option<serde_json::Value>,
    error: Option<serde_json::Value>,
    id: u32,
}


#[derive(Debug)]
pub enum JsonRpcResponse 
{
    MethodResponse(JsonRpcMethodResponse),
    Notification(JsonRpcNotification),
}

impl<'de> Deserialize<'de> for JsonRpcResponse {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        
        if value.get("method").is_some() {
            let notification = JsonRpcNotification::deserialize(value)
                .map_err(serde::de::Error::custom)?;
            Ok(JsonRpcResponse::Notification(notification))
        } else {
            let method_response = JsonRpcMethodResponse::deserialize(value)
                .map_err(serde::de::Error::custom)?;
            Ok(JsonRpcResponse::MethodResponse(method_response))
        }
    }
}

#[derive(Debug)]
pub struct JsonRpcNotification
{
    jsonrpc: String,
    method: String,
    params: MoonrakerEventParameters,
}

impl<'de> Deserialize<'de> for JsonRpcNotification {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct NotificationHelper {
            jsonrpc: String,
            method: String,
            params: Vec<serde_json::Value>,
        }

        let mut helper = NotificationHelper::deserialize(deserializer)?;

        let params = match helper.method.as_str() {
            "notify_status_update" => {
                let parsed_params = serde_json::from_value(helper.params[0].take())
                    .map_err(serde::de::Error::custom)?;
                MoonrakerEventParameters::NotifyStatusUpdate(parsed_params)
            }
            _ => return Err(serde::de::Error::custom("Unknown method")),
        };

        Ok(JsonRpcNotification {
            jsonrpc: helper.jsonrpc,
            method: helper.method,
            params,
        })
    }
}

#[derive(Debug)]
enum MoonrakerEventParameters
{
    NotifyStatusUpdate(MoonrakerEventNotifyStatusUpdate),
}

// TODO: Make struct that represents all types of notifications. Then make a custom deserializer to parse that custom struct.

#[derive(Debug)]
pub struct MoonrakerEventNotifyStatusUpdate
{
    pub events: Vec<OptionalPrinterEvent>,
}

impl<'de> Deserialize<'de> for MoonrakerEventNotifyStatusUpdate {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let helper = serde_json::Map::deserialize(deserializer)?;

        let mut events = Vec::new();

        for (object_name, object_value) in helper {
            let first_part_of_name = object_name.split(" ").next().unwrap_or("");
            let last_part_of_name = object_name.split(" ").last().unwrap_or("");

            //println!("Object name: {} with value {:?}", object_name, object_value);

            let event = match first_part_of_name
            {
                "webhooks" => OptionalPrinterEvent::Webhooks(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "motion_report" => OptionalPrinterEvent::MotionReport(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "gcode_move" => OptionalPrinterEvent::GcodeMove(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "toolhead" => OptionalPrinterEvent::Toolhead(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "extruder" => OptionalPrinterEvent::Extruder(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "heater_bed" => OptionalPrinterEvent::HeaterBed(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "fan" => OptionalPrinterEvent::Fan(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "idle_timeout" => OptionalPrinterEvent::IdleTimeout(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "virtual_sdcard" => OptionalPrinterEvent::VirtualSdcard(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "print_stats" => OptionalPrinterEvent::PrintStats(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "display_status" => OptionalPrinterEvent::DisplayStatus(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                "temperature_sensor" => OptionalPrinterEvent::TemperatureSensor(NamedOptionalTemperatureSensor {
                    name: last_part_of_name.to_string(),
                    sensor: serde_json::from_value(object_value).map_err(serde::de::Error::custom)?,
                }),
                "temperature_fan" => OptionalPrinterEvent::TemperatureFan(NamedOptionalTemperatureFan {
                    name: last_part_of_name.to_string(),
                    fan: serde_json::from_value(object_value).map_err(serde::de::Error::custom)?,
                }),
                "filament_switch_sensor" => OptionalPrinterEvent::FilamentSwitchSensor(NamedOptionalFilamentSwitchSensor {
                    name: last_part_of_name.to_string(),
                    sensor: serde_json::from_value(object_value).map_err(serde::de::Error::custom)?,
                }),
                "output_pin" => OptionalPrinterEvent::OutputPin(NamedOptionalOutputPin {
                    name: last_part_of_name.to_string(),
                    pin: serde_json::from_value(object_value).map_err(serde::de::Error::custom)?,
                }),
                "exclude_object" => OptionalPrinterEvent::ExcludeObject(serde_json::from_value(object_value).map_err(serde::de::Error::custom)?),
                _ => {
                    //eprintln!("Unknown object name: {}", object_name);
                    continue; // Skip unknown object names
                }
            };

            events.push(event);
        }

        Ok(MoonrakerEventNotifyStatusUpdate { events })
    }
}

#[derive(Debug, Deserialize)]
pub struct PrinterObjectListResponse
{
    pub objects: Vec<String>,
}

#[derive(Debug)]
pub struct MoonrakerReply 
{
    id: u32,
    result: serde_json::Value // TODO: Make type safe (by checking method name)
}

#[derive(Debug)]
pub enum MoonrakerEvent
{
    NotifyStatusUpdate(PrinterEvent)
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

#[derive(Debug)]
pub enum OptionalPrinterEvent {
    Webhooks(OptionalWebhooks),
    MotionReport(OptionalMotionReport),
    GcodeMove(OptionalGcodeMove),
    Toolhead(OptionalToolhead),
    Extruder(OptionalExtruder),
    HeaterBed(OptionalHeaterBed),
    Fan(OptionalFan),
    IdleTimeout(OptionalIdleTimeout),
    VirtualSdcard(OptionalVirtualSdcard),
    PrintStats(OptionalPrintStats),
    DisplayStatus(OptionalDisplayStatus),
    TemperatureSensor(NamedOptionalTemperatureSensor),
    TemperatureFan(NamedOptionalTemperatureFan),
    FilamentSwitchSensor(NamedOptionalFilamentSwitchSensor),
    OutputPin(NamedOptionalOutputPin),
    ExcludeObject(OptionalExcludeObject),
}

#[derive(Debug)]
pub enum GeneralEvent {
    Connected,
    Disconnected,
    Error(String),
    MoonrakerEvent(MoonrakerEvent),
    MoonrakerReply(MoonrakerReply),
}

pub struct PrinterObjectsSubscribeParams
{
    pub objects: serde_json::Map<String, Value>,
}

impl PrinterObjectsSubscribeParams
{
    pub fn all_fields(objects: Vec<String>) -> Self {
        let mut map = serde_json::Map::new();
        for object in objects {
            map.insert(object, serde_json::Value::Null);
        }
        Self { objects: map }
    }
}

#[derive(Debug, Deserialize)]
pub struct PrinterObjectsSubscribeResult
{
    pub eventtime: f32,
    pub status: MoonrakerEventNotifyStatusUpdate,
}

pub enum SendingEventType
{
    PrinterObjectList,
    PrinterObjectsSubscribe(PrinterObjectsSubscribeParams),
    RawFrame(Mutex<Option<Frame<'static>>>),
    EndLoop,
}

pub struct SendingEvent
{
    event_type: SendingEventType,
    id: u32,
}

impl SendingEvent
{
    pub fn new(event_type: SendingEventType, id: u32) -> Self {
        Self {
            event_type,
            id,
        }
    }

    pub fn without_id(event_type: SendingEventType) -> Self {
        Self {
            event_type,
            id: 0,
        }
    }
}

/* 
pub struct MoonrakerPrinterState
{
    pub webhooks: Webhooks,
    pub motion_report: MotionReport,
    pub gcode_move: GcodeMove,
    pub toolhead: Toolhead,
    pub extruder: Extruder,
    pub heater_bed: HeaterBed,
    pub fan: Fan,
    pub idle_timeout: IdleTimeout,
    pub virtual_sdcard: VirtualSdcard,
    pub print_stats: PrintStats,
    pub display_status: DisplayStatus,
    pub temperature_sensors: Vec<NamedTemperatureSensor>,
    pub temperature_fans: Vec<NamedTemperatureFan>,
    pub filament_switch_sensors: Vec<FilamentSwitchSensor>,
    pub output_pins: Vec<NamedOutputPin>,
    pub exclude_object: ExcludeObject,
}

impl Default for MoonrakerPrinterState {
    fn default() -> Self {
        Self {
            webhooks: Webhooks::default(),
            motion_report: MotionReport::default(),
            gcode_move: GcodeMove::default(),
            toolhead: Toolhead::default(),
            extruder: Extruder::default(),
            heater_bed: HeaterBed::default(),
            fan: Fan::default(),
            idle_timeout: IdleTimeout::default(),
            virtual_sdcard: VirtualSdcard::default(),
            print_stats: PrintStats::default(),
            display_status: DisplayStatus::default(),
            temperature_sensors: Vec::new(),
            temperature_fans: Vec::new(),
            filament_switch_sensors: Vec::new(),
            output_pins: Vec::new(),
            exclude_object: ExcludeObject::default(),
        }
    }
}


pub struct MoonrakerConnectionBuilder
{
    host: String,
    port: u16,
    on_event: Option<Box<dyn Fn(&GeneralEvent, &MoonrakerPrinterState) + 'static>>,
}

impl MoonrakerConnectionBuilder
{
    pub fn new(host: &str, port: u16) -> Self {
        Self {
            host: host.to_string(),
            port,
            on_event: None,
        }
    }

    pub fn on_event<F>(mut self, callback: F) -> Self
    where
        F: Fn(&GeneralEvent, &MoonrakerPrinterState) + 'static,
    {
        self.on_event = Some(Box::new(callback));
        self
    }

    pub async fn build(self) -> MoonrakerConnection {
        let host = format!("{}:{}", self.host, self.port);
        let url = format!("ws://{}/websocket", host);

        let req = Request::builder()
            .method("GET")
            .uri(url)
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
        let (inbound_event_sender, inbound_event_listener) = broadcast::channel::<Arc<GeneralEvent>>(256);

        // Us -> Moonraker. TX = send requests, RX = ws writer
        let (outbound_event_sender, outbound_event_listener) = broadcast::channel::<Arc<SendingEvent>>(256);

        MoonrakerConnection {
            connection: Mutex::new(None),
            host: host,
            request: req,
            inbound_event_sender: RefCell::new(inbound_event_sender),
            inbound_event_listener: inbound_event_listener,
            outbound_event_sender: outbound_event_sender,
            outbound_event_listener: RefCell::new(outbound_event_listener),
        }
    }
}

*/

pub struct MoonrakerConnection
{
    host: String,
    request: Request<Empty<Bytes>>,
    inbound_event_sender : Sender<Arc<GeneralEvent>>,
    inbound_event_listener: Receiver<Arc<GeneralEvent>>,
    outbound_event_sender : Sender<Arc<SendingEvent>>,
    outbound_event_listener: Receiver<Arc<SendingEvent>>,
    incrementing_id: Mutex<u32>,
}

impl MoonrakerConnection
{
    pub fn new(host: &str, port: u16) -> Self {
        let host = format!("{}:{}", host, port);
        let url = format!("ws://{}/websocket", host);

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
        let (inbound_event_sender, inbound_event_listener) = broadcast::channel::<Arc<GeneralEvent>>(256);

        // Us -> Moonraker. TX = send requests, RX = ws writer
        let (outbound_event_sender, outbound_event_listener) = broadcast::channel::<Arc<SendingEvent>>(256);

        MoonrakerConnection {
            host: host,
            request: req,
            inbound_event_sender: inbound_event_sender,
            inbound_event_listener: inbound_event_listener,
            outbound_event_sender: outbound_event_sender,
            outbound_event_listener: outbound_event_listener,
            incrementing_id: Mutex::new(1),
        }
    }

    pub async fn new_id(&self) -> u32 {
        let mut id = self.incrementing_id.lock().await;
        let current_id = *id;
        *id += 1;
        current_id
    }

    pub async fn connection_loop(&self) {
        loop 
        {
            // TODO: Remove the mutexes here
            // TODO: Kill old threads if they exist
            // TODO: Scope these listeners/senders
            let inbound_sender = self.inbound_event_sender.clone();
            let inbound_sender_2 = self.inbound_event_sender.clone();
            let mut outbound_listener = self.outbound_event_listener.resubscribe();
            let outbound_event_sender = self.outbound_event_sender.clone();

            let _ = inbound_sender.send(Arc::new(GeneralEvent::Disconnected));
            let mut reader;
            let mut writer;

            let cache = Arc::new(Mutex::new(Cache::new()));
            let cache_clone = cache.clone();

            match self.reconnect().await {
                Ok((r, w)) => {
                    reader = r;
                    writer = w;
                    let _ = inbound_sender.send(Arc::new(GeneralEvent::Connected));
                }
                Err(e) => {
                    eprintln!("Error connecting to Moonraker: {}", e);
                    sleep(std::time::Duration::from_secs(2)).await;
                    continue;
                }
            }

            async fn write_method(method: &str, params: Option<serde_json::Value>, id: u32, writer : &mut WebSocketWrite<WriteHalf<TokioIo<Upgraded>>>) -> Result<(), Box<dyn Error + Send + Sync>> {
                let request = JsonRpcRequest {
                    jsonrpc: "2.0".to_string(),
                    method: method.to_string(),
                    params,
                    id,
                };

                let data = serde_json::to_string(&request).unwrap();

                //println!("Sending request: {}", data);

                let bytes = data.as_bytes().to_vec();

                writer.write_frame(Frame::text(Payload::Owned(bytes))).await?;
                Ok(())
            }

            let reader_handle = tokio::spawn(async move {
                loop {
                    let outbound_event_sender_clone = outbound_event_sender.clone();
                    let frame = match reader.read_frame(&mut move |x| {
                        let sender = outbound_event_sender_clone.clone();
                        async move {
                            let _ = sender.send(Arc::new(SendingEvent { event_type: SendingEventType::RawFrame(Mutex::new(Some(x))), id: 0 }));
                            Ok::<(), std::io::Error>(())
                        }
                    }).await {
                        Ok(frame) => frame,
                        Err(e) => {
                            // Assume connection lost
                            let _ = inbound_sender.send(Arc::new(GeneralEvent::Error(e.to_string())));
                            let _ = inbound_sender.send(Arc::new(GeneralEvent::Disconnected));
                            let _ = outbound_event_sender.send(Arc::new(SendingEvent::without_id(SendingEventType::EndLoop)));
                            break;
                        }
                    };

                    match frame.opcode
                    {
                        OpCode::Close => {
                            let _ = inbound_sender.send(Arc::new(GeneralEvent::Disconnected));
                            let _ = outbound_event_sender.send(Arc::new(SendingEvent::without_id(SendingEventType::EndLoop)));
                            break;
                        }

                        OpCode::Text => {
                            let payload = String::from_utf8(frame.payload.to_vec()).expect("Invalid UTF-8 data");

                            if let Ok(data) = serde_json::from_str::<JsonRpcResponse>(&payload)
                            {
                                match data
                                {
                                    JsonRpcResponse::MethodResponse(method_response) => {
                                        if method_response.error.is_some()
                                        {
                                            eprintln!("Received error in method response: {:?}", method_response.error);
                                        }
                                        else 
                                        {
                                            let reply = MoonrakerReply {
                                                id: method_response.id,
                                                result: method_response.result.unwrap_or(serde_json::json!(null)),
                                            };
                                            let _ = inbound_sender.send(Arc::new(GeneralEvent::MoonrakerReply(reply)));
                                        }
                                    }

                                    JsonRpcResponse::Notification(notification) => {
                                        match notification.params
                                        {
                                            MoonrakerEventParameters::NotifyStatusUpdate(status_update) => {
                                                for event in status_update.events
                                                {
                                                    let mut unlocked_cache = cache.lock().await;
                                                    let _ = inbound_sender.send(Arc::new(GeneralEvent::MoonrakerEvent(MoonrakerEvent::NotifyStatusUpdate(unlocked_cache.complete_event(event)))));
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            else 
                            {
                                //eprintln!("Failed to parse JSON-RPC response: {}", payload);
                            }
                        }

                        _ => {
                            eprintln!("Received unsupported frame: {:?}", frame.opcode);
                        }
                    }
                }
            });

            let writer_handle = tokio::spawn(async move {
                loop {
                    let message = outbound_listener.recv().await.unwrap();

                    let result = match &message.event_type
                    {
                        SendingEventType::PrinterObjectList => write_method("printer.objects.list", None, message.id, &mut writer).await,
                        SendingEventType::PrinterObjectsSubscribe(params) => write_method("printer.objects.subscribe", Some(serde_json::json!({"objects": params.objects})), message.id, &mut writer).await,
                        SendingEventType::RawFrame(frame) => {
                            let frame = frame.lock().await.take();
                            println!("Got raw frame to send");

                            let result = match frame 
                            {
                                Some(frame) => {
                                    match writer.write_frame(frame).await {
                                        Ok(_) => Ok(()),
                                        Err(e) => Err(Box::new(e) as Box<dyn Error + Send + Sync>),
                                    }
                                }
                                None => Ok(()),
                            };

                            result
                        },
                        SendingEventType::EndLoop => break,  
                    };

                    // TODO: Erorr handling
                }
            });

            let object_list = match self.get_printer_object_list().await {
                Ok(object_list) => object_list,
                Err(e) => {
                    eprintln!("Error getting printer object list: {}", e);
                    reader_handle.abort();
                    writer_handle.abort();
                    continue;
                }
            };

            // TOOD: Don't subscribe to objects we don't have a use for.
            let initial_objects = self.post_printer_objects_subscribe(object_list.objects.clone()).await;
            if let Err(e) = initial_objects {
                eprintln!("Error subscribing to printer objects: {}", e);
                reader_handle.abort();
                writer_handle.abort();
                continue;
            }

            for event in initial_objects.unwrap().status.events
            {
                let mut unlocked_cache = cache_clone.lock().await;
                let _ = inbound_sender_2.send(Arc::new(GeneralEvent::MoonrakerEvent(MoonrakerEvent::NotifyStatusUpdate(unlocked_cache.complete_event(event)))));
            }

            reader_handle.await.unwrap();
            writer_handle.await.unwrap();
        }
    }

    pub async fn send_request<T>(&self, method: SendingEventType) -> Result<T, Box<dyn Error + Send + Sync>>
    where T : DeserializeOwned
    {
        let mut listener = self.inbound_event_listener.resubscribe();
        let id = self.new_id().await;

        let event = Arc::new(SendingEvent::new(method, id));
        let _ = self.outbound_event_sender.send(event);

        loop {
            let event = listener.recv().await?;

            match &*event
            {
                GeneralEvent::MoonrakerReply(reply) if reply.id == id => {
                    let parsed_result: Result<T, serde_json::Error> = serde_json::from_value(reply.result.clone());

                    match parsed_result {
                        Ok(result) => return Ok(result),
                        Err(e) => return Err(Box::new(e)),
                    }
                },
                _ => continue, // TODO: This should eventually end
            }
        }
    }

    pub async fn get_printer_object_list(&self) -> Result<PrinterObjectListResponse, Box<dyn Error + Send + Sync>> {
        self.send_request(SendingEventType::PrinterObjectList).await
    }

    pub async fn post_printer_objects_subscribe(&self, objects: Vec<String>) -> Result<PrinterObjectsSubscribeResult, Box<dyn Error + Send + Sync>> {
        self.send_request(SendingEventType::PrinterObjectsSubscribe(PrinterObjectsSubscribeParams::all_fields(objects))).await
    }

    pub async fn reconnect(&self) -> Result<(FragmentCollectorRead<ReadHalf<TokioIo<Upgraded>>>, WebSocketWrite<WriteHalf<TokioIo<Upgraded>>>), Box<dyn Error + Send + Sync>> {
        let stream = TcpStream::connect(self.host.clone()).await?;

        let (ws, _) = handshake::client(&SpawnExecutor, self.request.clone(), stream).await?;
        let (rx, tx) = ws.split(tokio::io::split);
        let reader= FragmentCollectorRead::new(rx);

        Ok((reader, tx))
    }

    pub fn get_listener(&self) -> Receiver<Arc<GeneralEvent>> {
        self.inbound_event_listener.resubscribe()
    }
}
