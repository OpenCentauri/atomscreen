use std::sync::Arc;

use moonraker_rs::{moonraker_connection::{MoonrakerConnection}, requests::PrinterAdministrationRequestHandler};
use slint::ComponentHandle;

use crate::{config::OptionalGcodeCommands, config::GcodeCommands as GcodeCommandsConfig, AppWindow, GcodeCommands};


pub fn run_command(moonraker_connection : &Arc<MoonrakerConnection>, command : &str)
{
    let command = command.to_string();
    let moonraker_connection = Arc::clone(moonraker_connection);
    // TODO: For other callbacks that don't use the slint main thread, maybe don't run them on the slint event loop?
    tokio::spawn(async move {
        if let Err(e) = moonraker_connection.run_gcode_script(&command).await
        {
            moonraker_connection.send_request_error(format!("Failed to send G-code command '{}': {}", command, e));
        }
    });
}

pub fn register_extruder_extrude(ui: &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>, gcode_command_config : &OptionalGcodeCommands)
{
    let command = gcode_command_config.extruder_extrude.clone().unwrap_or(GcodeCommandsConfig::default().extruder_extrude);
    let moonraker_connection = Arc::clone(moonraker_connection);

    ui.global::<GcodeCommands>().set_extruder_extrude_available(!command.is_empty());
    ui.global::<GcodeCommands>().on_extruder_extrude(move || {
        run_command(&moonraker_connection, &command);
    });
}

pub fn register_extruder_retract(ui: &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>, gcode_command_config : &OptionalGcodeCommands)
{
    let command = gcode_command_config.extruder_retract.clone().unwrap_or(GcodeCommandsConfig::default().extruder_retract);
    let moonraker_connection = Arc::clone(moonraker_connection);

    ui.global::<GcodeCommands>().set_extruder_retract_available(!command.is_empty());
    ui.global::<GcodeCommands>().on_extruder_retract(move || {
        run_command(&moonraker_connection, &command);
    });
}

pub fn register_extruder_load_filament(ui: &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>, gcode_command_config : &OptionalGcodeCommands)
{
    let command = gcode_command_config.extruder_load_filament.clone().unwrap_or(GcodeCommandsConfig::default().extruder_load_filament);
    let moonraker_connection = Arc::clone(moonraker_connection);

    ui.global::<GcodeCommands>().set_extruder_load_filament_available(!command.is_empty());
    ui.global::<GcodeCommands>().on_extruder_load_filament(move || {
        run_command(&moonraker_connection, &command);
    });
}

pub fn register_extruder_unload_filament(ui: &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>, gcode_command_config : &OptionalGcodeCommands)
{
    let command = gcode_command_config.extruder_unload_filament.clone().unwrap_or(GcodeCommandsConfig::default().extruder_unload_filament);
    let moonraker_connection = Arc::clone(moonraker_connection);

    ui.global::<GcodeCommands>().set_extruder_unload_filament_available(!command.is_empty());
    ui.global::<GcodeCommands>().on_extruder_unload_filament(move || {
        run_command(&moonraker_connection, &command);
    });
}