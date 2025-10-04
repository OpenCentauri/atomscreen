use std::sync::Arc;

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::{PowerDeviceAction, PrinterAdministrationRequestHandler, SwitchesSensorsDevicesRequestHandler}};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{AppWindow, PowerDevice, PowerDevices, PrinterAdministration};


pub fn register_misc_set_power_device(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    
    ui.global::<PowerDevices>().on_set_power_device_state(move |device, state| {
        let moonraker_connection = moonraker_connection.clone();

        tokio::spawn(async move {
            if let Err(e) = moonraker_connection.set_power_device_state(&device.to_string(), if state { PowerDeviceAction::On } else { PowerDeviceAction::Off }).await
            {
                moonraker_connection.send_request_error(format!("Failed to set power device state for {}: {}", device.to_string(), e));
                return;
            }
        });
    });
}