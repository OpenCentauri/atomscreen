// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fs, path::PathBuf, process::exit, sync::Arc};

use clap::Parser;
use moonraker_rs::{
    cache::Cache, connector::{read_deserialize::OptionalPrinterEvent}, printer_objects::{NamedOptionalTemperatureFan, OptionalExtruder, OptionalHeaterBed, OptionalTemperatureFan, TemperatureConfiguration}, 
};

use crate::{config::{MoonrakerConfig, OptionalGcodeCommands, OptionalUiConfig}, event_loop::EventLoop, hardware::init_display, ui_functions::*};

mod application_error;
mod config;
mod hardware;
mod event_loop;
mod ui_functions;

slint::include_modules!();

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = config::Args::parse();

    let config_path = PathBuf::from(&args.config);
    if !config_path.exists() {
        eprintln!("Config file does not exist: {}", config_path.display());
        exit(2);
    }

    let config_str = fs::read_to_string(&config_path).unwrap();
    let config = toml::from_str::<config::Config>(&config_str).unwrap();
    let moonraker_config = config.moonraker.unwrap_or(MoonrakerConfig::default());
    let mut cache = Cache::default();

    if let Some(heater_presets) = config.heater_presets {
        for (name, preset) in heater_presets {
            if name == "extruder" {
                cache.complete_event(OptionalPrinterEvent::Extruder(OptionalExtruder {
                    configuration: Some(TemperatureConfiguration::from(preset)),
                    ..Default::default()
                }));
            } else if name == "heater_bed" {
                cache.complete_event(OptionalPrinterEvent::HeaterBed(OptionalHeaterBed {
                    configuration: Some(TemperatureConfiguration::from(preset)),
                    ..Default::default()
                }));
            } else {
                cache.complete_event(OptionalPrinterEvent::TemperatureFan(NamedOptionalTemperatureFan {
                    name: name,
                    fan: OptionalTemperatureFan {
                        configuration: Some(TemperatureConfiguration::from(preset)),
                        ..Default::default()
                    }
                }));
            }
        }
    }

    let moonraker_connection = Arc::new(
        moonraker_rs::moonraker_connection::MoonrakerConnection::new(
            &moonraker_config.host,
            moonraker_config.port,
            Some(cache),
        ),
    );
    let ui = init_display(&config.display)?;
    ui.global::<Webhooks>().set_moonraker_connected(false);
    let ui_weak = ui.as_weak();
    let mut event_loop = EventLoop::new(ui_weak.clone(), moonraker_connection.clone());

    {
        let moonraker_connection = moonraker_connection.clone();
        tokio::spawn(async move {
            moonraker_connection.connection_loop().await;
        });
    }

    tokio::spawn(async move {
        event_loop.event_loop().await;
    });

    register_filesystem_list_files(&ui, &moonraker_connection);
    register_filesystem_fetch_metadata(&ui, &moonraker_connection);

    register_temperature_set_new_target_temperature(&ui, &moonraker_connection);

    register_util_format_bytes(&ui);
    register_util_prettify_name(&ui);
    register_util_time_in_seconds_to_string(&ui);
    register_util_create_temperature_lists(&ui);
    register_util_convert_temperature_back(&ui);

    register_printer_emergency_stop(&ui, &moonraker_connection);
    register_printer_firmware_restart(&ui, &moonraker_connection);
    register_printer_restart(&ui, &moonraker_connection);

    let gcode_command_config = &config.gcode_commands.unwrap_or_default();
    register_extruder_extrude(&ui, &moonraker_connection, gcode_command_config);
    register_extruder_retract(&ui, &moonraker_connection, gcode_command_config);
    register_extruder_load_filament(&ui, &moonraker_connection, gcode_command_config);
    register_extruder_unload_filament(&ui, &moonraker_connection, gcode_command_config);

    let ui_settings = &config.ui.unwrap_or(OptionalUiConfig::default());
    register_set_ui_settings(&ui, &ui_settings);

    register_execute_quick_action(&ui, &config.quick_actions, &moonraker_connection);

    tokio::task::block_in_place(|| {
        ui.run().unwrap();
    });

    Ok(())
}
