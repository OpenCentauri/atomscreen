use std::sync::Arc;

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::PrinterAdministrationRequestHandler};
use slint::{ComponentHandle, ModelRc, VecModel};

use crate::{config::{OptionalUiConfig, UiConfig}, AppWindow, PrinterAdministration, UiSettings};


pub fn name_to_id(name : &str) -> i32
{
    match name {
        "files" => 0,
        "temperature" => 1,
        "move" => 2,
        "emergency_stop" => 3,
        "fan" => 4,
        "macros" => 5,
        "console" => 6,
        "settings" => 7,
        _ => panic!("Unknown menu {} for left/right sidebar", name)
    }
}

pub fn register_set_ui_settings(ui : &AppWindow, configuration : &OptionalUiConfig)
{
    let configuration = UiConfig::from_optional(configuration);

    let left_sidebar: Vec<i32> = configuration.left_sidebar.iter().map(|f| name_to_id(f)).collect();
    let right_sidebar: Vec<i32> = configuration.right_sidebar.iter().map(|f| name_to_id(f)).collect();

    ui.global::<UiSettings>().set_dark_mode(configuration.dark_mode);
    ui.global::<UiSettings>().set_left_sidebar(ModelRc::new(VecModel::from(left_sidebar)));
    ui.global::<UiSettings>().set_right_sidebar(ModelRc::new(VecModel::from(right_sidebar)));
}