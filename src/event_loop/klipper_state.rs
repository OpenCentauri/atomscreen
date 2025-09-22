use moonraker_rs::connector::websocket_read::PrinterEvent;
use slint::{ComponentHandle, SharedString};

use crate::{application_error::ApplicationError, event_loop::EventLoop};

pub trait KlipperStateHandler {
    fn handle_klipper_state_updates(
        &mut self,
        printer_event: &PrinterEvent,
    ) -> Result<(), ApplicationError>;
}

impl KlipperStateHandler for EventLoop {
    fn handle_klipper_state_updates(
        &mut self,
        printer_event: &PrinterEvent,
    ) -> Result<(), ApplicationError> {
        if let PrinterEvent::Webhooks(webhooks) = printer_event {
            let state = SharedString::from(webhooks.state.to_string());
            let state_message = SharedString::from(&webhooks.state_message);

            self.ui_weak.upgrade_in_event_loop(move |ui| {
                ui.global::<crate::AppState>().set_klipper_state(state);
                ui.global::<crate::AppState>().set_klipper_state_message(state_message);
            })?;
        }

        Ok(())
    }
}
