
use std::fmt::Display;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::{
    connector::read_deserialize::MoonrakerEventNotifyStatusUpdate, error::Error,
    moonraker_connection::MoonrakerConnection,
};

pub trait SwitchesSensorsDevicesRequestHandler {
    async fn list_power_devices(&self) -> Result<Vec<PowerDevice>, Error>;
    async fn set_power_device_state(
        &self,
        device: &str,
        state: PowerDeviceAction,
    ) -> Result<Value, Error>;
}

impl SwitchesSensorsDevicesRequestHandler for MoonrakerConnection {
    async fn list_power_devices(&self) -> Result<Vec<PowerDevice>, Error> {
        let devices : ListPowerDevicesResult = self.send_request("machine.device_power.devices", None).await?;

        Ok(devices.devices)
    }

    async fn set_power_device_state(
        &self,
        device: &str,
        state: PowerDeviceAction,
    ) -> Result<Value, Error> {
        let args = serde_json::json!({
            "device": device,
            "action": state,
        });
        self.send_request("machine.device_power.post_device", Some(args)).await
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerDeviceState {
    On,
    Off,
    Init,
    Error,
}

impl Display for PowerDeviceState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PowerDeviceState::On => write!(f, "On"),
            PowerDeviceState::Off => write!(f, "Off"),
            PowerDeviceState::Init => write!(f, "Initializing"),
            PowerDeviceState::Error => write!(f, "Error"),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum PowerDeviceAction
{
    On,
    Off,
    Toggle,
}

#[derive(Debug, Deserialize)]
pub struct PowerDevice {
    pub device: String,
    pub status: PowerDeviceState,
    pub locked_while_printing: bool,
    #[serde(alias = "type")]
    pub device_type: String,
}

#[derive(Debug, Deserialize)]
pub struct ListPowerDevicesResult 
{
    pub devices: Vec<PowerDevice>
}