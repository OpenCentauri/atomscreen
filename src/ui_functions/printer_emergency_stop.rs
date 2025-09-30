use std::sync::Arc;

use moonraker_rs::{moonraker_connection::MoonrakerConnection, requests::PrinterAdministrationRequestHandler};
use slint::ComponentHandle;

use crate::{AppWindow, PrinterAdministration};


pub fn register_printer_emergency_stop(ui : &AppWindow, moonraker_connection : &Arc<MoonrakerConnection>)
{
    let moonraker_connection = moonraker_connection.clone();
    
    ui.global::<PrinterAdministration>().on_emergency_stop(move || {
        let moonraker_connection = moonraker_connection.clone();

        slint::spawn_local(async move {
            if let Err(e) = moonraker_connection.emergency_stop().await
            {
                moonraker_connection.send_request_error(format!("Failed to send emergency stop command: {}", e));
            }
        }).unwrap();
    });
}