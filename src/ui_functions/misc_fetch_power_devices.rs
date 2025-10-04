use std::sync::Arc;

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::{PrinterAdministrationRequestHandler, SwitchesSensorsDevicesRequestHandler}};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{AppWindow, PowerDevice, PowerDevices, PrinterAdministration};


pub fn register_misc_fetch_power_devices(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    let ui_weak = ui.as_weak();
    
    ui.global::<PowerDevices>().on_fetch_power_devices(move || {
        let moonraker_connection = moonraker_connection.clone();
        let ui_weak = ui_weak.clone();

        slint::spawn_local(async move {
            let devices = match moonraker_connection.list_power_devices().await
            {
                Ok(d) => d,
                Err(e) => {
                    moonraker_connection.send_request_error(format!("Failed to fetch power devices: {}", e));
                    return;
                }
            };

            let ui = ui_weak.upgrade().unwrap();

            let devices: Vec<PowerDevice> = devices.iter().map(|power_device| {
                PowerDevice {
                    device: SharedString::from(&power_device.device),
                    status: SharedString::from(&power_device.status.to_string()),
                    locked_while_printing: power_device.locked_while_printing,
                    device_type: SharedString::from(&power_device.device_type),
                }
            }).collect();

            ui.global::<PowerDevices>().set_power_devices(ModelRc::new(VecModel::from(devices)));
        }).unwrap();
    });
}