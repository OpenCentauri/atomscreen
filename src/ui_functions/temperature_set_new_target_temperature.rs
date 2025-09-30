use std::sync::Arc;

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::PrinterAdministrationRequestHandler};
use slint::ComponentHandle;

use crate::{AppWindow, TemperatureSensors};

pub fn register_temperature_set_new_target_temperature(ui : &AppWindow, moonraker_connection: &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    
    ui.global::<TemperatureSensors>().on_set_new_target_temperature(move |heater_name, target| {
        println!("Set new target temperature for {}: {}", heater_name, target);
        let moonraker_connection = moonraker_connection.clone();

        slint::spawn_local(async move {
            let heater_name = heater_name.to_string();

            let command = match heater_name.as_str() {
                "extruder" | "heater_bed" => format!("SET_HEATER_TEMPERATURE HEATER={} TARGET={}", heater_name, target),
                _ => format!("SET_TEMPERATURE_FAN_TARGET TEMPERATURE_FAN={} TARGET={}", heater_name, target),
            };

            let moonraker_connection = moonraker_connection.clone();
            // TODO: Remove except
            moonraker_connection.run_gcode_script(&command).await.expect("Failed to set heater temperature");
        })
        .unwrap();
    });
}