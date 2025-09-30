use crate::{
    application_error::ApplicationError, config::DisplayInit, hardware::FramebufferPlatform,
    AppWindow,
};
use evdev::Device;
use linuxfb::Framebuffer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DisplayFramebufferConfig {
    pub fb_path: String,
    pub event_path: Option<String>,
    pub buffering: Option<String>,
}

impl DisplayInit for DisplayFramebufferConfig {
    fn init(&self) -> Result<crate::AppWindow, crate::application_error::ApplicationError> {
        let fb = match Framebuffer::new(&self.fb_path) {
            Ok(fb) => fb,
            Err(e) => {
                return Err(ApplicationError::Unknown(String::from(
                    "Failed to initialise framebuffer",
                )))
            }
        };

        let double_buffering = match self.buffering.clone().unwrap_or(String::from("double")).to_lowercase().as_str()
        {
            "double" => true,
            "single" => false,
            _ => panic!("Config value {:?} is unsupported for buffering. Supported is \"Single\" or \"Double\"", self.buffering)
        };

        let mut touch_device = None;

        if let Some(event_path) = &self.event_path {
            touch_device = Some(Device::open(event_path)?);
        }

        slint::platform::set_platform(Box::new(FramebufferPlatform::new(fb, touch_device, double_buffering)))
            .expect("set platform");

        let ui = AppWindow::new()?;

        Ok(ui)
    }
}
