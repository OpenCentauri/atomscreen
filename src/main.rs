// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fs, path::PathBuf, process::exit};

use clap::Parser;
use slint::PlatformError;

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

    let handle = tokio::task::spawn_blocking(move || -> Result<(), ApplicationError> {
        let ui = init_display(&config.display)?;
        Ok(ui.run()?)
    });

    let moonraker_connection = moonraker_rs::moonraker_connection::MoonrakerConnection::new("localhost", 7125);

    let mut receiver = moonraker_connection.get_listener();

    tokio::spawn(async move {
        loop {
            match receiver.recv().await
            {
                Ok(event) => {
                    // Forward event to Slint UI
                    //println!("Received Moonraker event: {:?}", event);
                },
                Err(e) => {
                    eprintln!("Error receiving Moonraker event: {}", e);
                    break;
                }
            }
        }
    });

    moonraker_connection.connection_loop().await;

    panic!("Connection loop exited");

    handle.await.unwrap().unwrap();

    Ok(())
}
