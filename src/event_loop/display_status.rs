use moonraker_rs::connector::websocket_read::PrinterEvent;
use slint::{ComponentHandle, SharedString};

use crate::{application_error::ApplicationError, event_loop::EventLoop, DisplayStatus, PrintStatus};

impl EventLoop {
    pub fn handle_display_status_updates(
        &self,
        printer_event: &PrinterEvent,
    ) -> Result<(), ApplicationError> {
        if let PrinterEvent::DisplayStatus(display_status) = printer_event {
            let message = SharedString::from(&display_status.message);
            let progress = display_status.progress;

            self.ui_weak.upgrade_in_event_loop(move |ui| {
                ui.global::<DisplayStatus>().set_message(message);
                ui.global::<PrintStatus>().set_progress(progress);
            })?;
        }
        
        Ok(())
    }
}
