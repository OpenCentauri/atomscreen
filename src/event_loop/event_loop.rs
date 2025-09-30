use std::sync::Arc;

use moonraker_rs::{connector::websocket_read::{MoonrakerEvent, PrinterEvent}, moonraker_connection::{MoonrakerConnection, WebsocketEvent}};
use slint::{ComponentHandle, Weak};

use crate::{application_error::ApplicationError, AppState, AppWindow};

pub struct EventLoop
{
    pub ui_weak : Weak<AppWindow>,
    pub moonraker_connection : Arc<MoonrakerConnection>   
}

//pub trait EventLoopListener
//{
//    fn on_printer_event(event_loop : &EventLoop, event : &PrinterEvent) -> Result<(), ApplicationError>;
//}

impl EventLoop
{
    pub fn new(ui_weak : Weak<AppWindow>, moonraker_connection : Arc<MoonrakerConnection>) -> EventLoop
    {
        EventLoop { ui_weak: ui_weak, moonraker_connection: moonraker_connection }
    }

    pub async fn event_loop(&mut self)
    {
        let mut receiver = self.moonraker_connection.get_listener();
        loop 
        {
            match receiver.recv().await
            {
                Ok(message) => 
                {
                    if let Err(e) = match &*message 
                    {
                        WebsocketEvent::Connected => self.on_connected().await,
                        WebsocketEvent::Disconnected => self.on_disconnected().await,
                        WebsocketEvent::MoonrakerEvent(event) => self.on_event(event).await,
                        _ => Ok(()),
                    }
                    {
                        eprintln!("Error handling Moonraker message: {}", e);
                    }
                },
                Err(e) => 
                {
                    eprintln!("Error receiving message from Moonraker: {}", e);
                    break;
                }
            }
        }
    }

    async fn on_connected(&mut self) -> Result<(), ApplicationError>
    {
        self.ui_weak
            .upgrade_in_event_loop(move |ui| ui.global::<AppState>().set_moonraker_connected(true))?;

        Ok(())
    }

    async fn on_disconnected(&mut self) -> Result<(), ApplicationError>
    {
        self.ui_weak
            .upgrade_in_event_loop(move |ui| {
                ui.global::<AppState>().set_moonraker_connected(false);
                ui.global::<AppState>().set_klipper_state("".into());
                ui.global::<AppState>().set_klipper_state_message("".into());
            })?;

        Ok(())
    }

    async fn on_event(&mut self, moonraker_event : &MoonrakerEvent) -> Result<(), ApplicationError>
    {
        match moonraker_event
        {
            MoonrakerEvent::NotifyStatusUpdate(printer_event) => self.on_status_update(printer_event).await,
            _ => Ok(()),
        }
    }

    async fn on_status_update(&mut self, printer_event : &PrinterEvent) -> Result<(), ApplicationError>
    {
        self.handle_temperature_devices_update(printer_event)?;
        self.handle_klipper_state_updates(printer_event)?;
        self.handle_display_status_updates(printer_event)?;

        Ok(())
    }
}