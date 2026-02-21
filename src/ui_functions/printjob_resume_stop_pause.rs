use std::sync::Arc;

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::{PowerDeviceAction, PrinterAdministrationRequestHandler, SwitchesSensorsDevicesRequestHandler}};
use slint::{ComponentHandle, ModelRc, SharedString, VecModel};

use crate::{AppWindow, Filesystem, PowerDevice, PowerDevices, PrintStatus, PrinterAdministration};


pub fn register_printjob_resume(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    
    ui.global::<PrintStatus>().on_resume_print(move || {
        let moonraker_connection = moonraker_connection.clone();

        tokio::spawn(async move {
            if let Err(e) = moonraker_connection.resume_print().await
            {
                moonraker_connection.send_request_error(format!("Failed to resume print: {}", e));
                return;
            }
        });
    });
}

pub fn register_printjob_pause(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    
    ui.global::<PrintStatus>().on_pause_print(move || {
        let moonraker_connection = moonraker_connection.clone();

        tokio::spawn(async move {
            if let Err(e) = moonraker_connection.pause_print().await
            {
                moonraker_connection.send_request_error(format!("Failed to resume print: {}", e));
                return;
            }
        });
    });
}

pub fn register_printjob_stop(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    
    ui.global::<PrintStatus>().on_stop_print(move || {
        let moonraker_connection = moonraker_connection.clone();

        tokio::spawn(async move {
            if let Err(e) = moonraker_connection.stop_print().await
            {
                moonraker_connection.send_request_error(format!("Failed to resume print: {}", e));
                return;
            }
        });
    });
}

pub fn register_printjob_start(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    
    ui.global::<Filesystem>().on_start_file(move |filename| {
        let moonraker_connection = moonraker_connection.clone();
        let filename = filename.to_string();

        tokio::spawn(async move {
            if let Err(e) = moonraker_connection.start_print(&filename).await
            {
                moonraker_connection.send_request_error(format!("Failed to resume print: {}", e));
                return;
            }
        });
    });
}