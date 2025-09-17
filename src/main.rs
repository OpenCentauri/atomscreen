// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fs, path::PathBuf, process::exit, rc::Rc, sync::Arc};

use clap::Parser;
use moonraker_rs::moonraker_connection::{GeneralEvent, MoonrakerEvent, PrinterEvent};
use slint::{Model, ModelRc, PlatformError, SharedString, VecModel};

use crate::{application_error::ApplicationError, hardware::init_display};


mod config;
mod application_error;
mod hardware;

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

    let moonraker_connection = Arc::new(moonraker_rs::moonraker_connection::MoonrakerConnection::new("localhost", 7125));
    let moonraker_connection_clone = moonraker_connection.clone();

    let mut receiver = moonraker_connection.get_listener();
    let ui = init_display(&config.display)?;
    let ui_weak = ui.as_weak();
    
    tokio::spawn(async move {
        moonraker_connection.connection_loop().await;
    });

    tokio::spawn(async move {
        loop {
            match receiver.recv().await
            {
                Ok(event) => {
                    // Forward event to Slint UI
                    //println!("Received Moonraker event: {:?}", event);
                    //connection_ref.get_listener();

                    if let GeneralEvent::MoonrakerEvent(moonraker_event) = &*event
                    {
                        if let MoonrakerEvent::NotifyStatusUpdate(status_update) = moonraker_event
                        {
                            if let PrinterEvent::Extruder(extruder_event) = status_update
                            {
                                println!("Extruder event: {:?}", extruder_event);
                                let extruder = Heater { target: extruder_event.target as i32, temperature: extruder_event.temperature as i32};
                                
                                ui_weak.upgrade_in_event_loop(move |ui| ui.global::<State>().set_extruder(extruder)).unwrap();
                            }

                            if let PrinterEvent::HeaterBed(heater_bed_event) = status_update
                            {
                                let bed = Heater { target: heater_bed_event.target as i32, temperature: heater_bed_event.temperature as i32};

                                ui_weak.upgrade_in_event_loop(move |ui| ui.global::<State>().set_heated_bed(bed)).unwrap();
                            }

                            if let PrinterEvent::TemperatureSensor(temperature_sensor_event) = status_update
                            {
                                let sensor_event = TemperatureSensor { name: SharedString::from(&temperature_sensor_event.name), temperature: temperature_sensor_event.sensor.temperature as i32 };

                                ui_weak.upgrade_in_event_loop(move |ui| {
                                    let temperature_sensors = ui.global::<State>().get_temperature_sensors();
                                    let current_sensors = temperature_sensors.as_any().downcast_ref::<VecModel<TemperatureSensor>>();

                                    let mut entries = match &current_sensors {
                                        Some(model) => model.iter().collect::<Vec<TemperatureSensor>>(),
                                        None => vec![],
                                    };

                                    let index = entries.iter().position(|sensor| sensor.name == sensor_event.name);

                                    match index {
                                        Some(index) => {
                                            entries[index].temperature = sensor_event.temperature as i32
                                        },
                                        None => entries.push(sensor_event)
                                    }

                                    ui.global::<State>().set_temperature_sensors(ModelRc::new(Rc::new(VecModel::from(entries))));
                                }).unwrap();
                            }
                        }

                    }
                },
                Err(e) => {
                    eprintln!("Error receiving Moonraker event: {}", e);
                    break;
                }
            }
        }
    });


    

    ui.run().unwrap();
    Ok(())
}
