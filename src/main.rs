// Prevent console window in addition to Slint window in Windows release builds when, e.g., starting the app via file manager. Ignored on other platforms.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::{error::Error, fs, path::PathBuf, process::exit};

use clap::Parser;

use crate::hardware::init_display;


mod config;
mod application_error;
mod hardware;

slint::include_modules!();

fn main() -> Result<(), Box<dyn Error>> {
    let args = config::Args::parse();

    let config_path = PathBuf::from(&args.config);
    if !config_path.exists() {
        eprintln!("Config file does not exist: {}", config_path.display());
        exit(2);
    }

    let config_str = fs::read_to_string(&config_path).unwrap();
    let config = toml::from_str::<config::Config>(&config_str).unwrap();

    let ui = init_display(&config.display)?;

    ui.run()?;

    Ok(())
}
