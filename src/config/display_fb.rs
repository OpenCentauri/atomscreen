use crate::{application_error::ApplicationError, config::DisplayInit, hardware::FramebufferPlatform, AppWindow};
use evdev::Device;
use linuxfb::Framebuffer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DisplayFramebufferConfig {
    pub fb_path: String,
    pub event_path: Option<String>,
}

impl DisplayInit for DisplayFramebufferConfig
{
    fn init(&self) -> Result<crate::AppWindow, crate::application_error::ApplicationError> {
        let fb = match Framebuffer::new(&self.fb_path)
        {
            Ok(fb) => fb,
            Err(e) => return Err(ApplicationError::Unknown(String::from("Failed to initialise framebuffer")))
        };

        let mut touch_device = None;

        if let Some(event_path) = &self.event_path
        {
            touch_device = Some(Device::open(event_path)?);
        }

        slint::platform::set_platform(Box::new(
            FramebufferPlatform::new(
                fb,
                touch_device
            )
        )).expect("set platform");       
        
        let ui = AppWindow::new()?;

        Ok(ui)
    }
}