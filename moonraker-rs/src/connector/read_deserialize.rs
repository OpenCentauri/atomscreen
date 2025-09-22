use serde::Deserialize;

use crate::printer_objects::*;

#[derive(Debug, Deserialize)]
pub struct JsonRpcMethodResponse
{
    pub jsonrpc: String,
    pub result: Option<serde_json::Value>,
    pub error: Option<serde_json::Value>,
    pub id: u32,
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
    pub jsonrpc: String,
    pub method: String,
    pub params: MoonrakerEventParameters,
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
            },
            "notify_proc_stat_update" => {
                let parsed_params = serde_json::from_value(helper.params[0].take())
                    .map_err(serde::de::Error::custom)?;
                MoonrakerEventParameters::NotifyProcessStatisticsUpdate(parsed_params)
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
pub enum MoonrakerEventParameters
{
    NotifyStatusUpdate(MoonrakerEventNotifyStatusUpdate),
    NotifyProcessStatisticsUpdate(MoonrakerNotifyProcStatUpdate),
}

#[derive(Debug)]
pub struct MoonrakerEventNotifyStatusUpdate
{
    pub events: Vec<OptionalPrinterEvent>,
}

#[derive(Deserialize, Debug)]
pub struct MoonrakerNotifyProcStatUpdate
{
    pub moonraker_stats: MoonrakerStats,
    pub throttled_state: Option<ThrottledState>,
    pub cpu_temp : Option<f32>,
    pub network : serde_json::Value,
    pub system_cpu_usage : SystemCpuUsage,
    pub system_memory : SystemMemory,
    pub system_uptime : Option<f32>,
    pub websocket_connections : i32,
}

#[derive(Deserialize, Debug)]
pub struct MoonrakerStats 
{
    pub time: f32,
    pub cpu_usage: f32,
    pub memory: Option<i32>,
    pub mem_units : Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct ThrottledState
{
    pub bits: i32,
    pub flags: Vec<String>,
}

#[derive(Deserialize, Debug)]
pub struct SystemCpuUsage 
{
    pub cpu: f32,
}

#[derive(Deserialize, Debug)]
pub struct SystemMemory
{
    pub total: i32,
    pub available: i32,
    pub used: i32,
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