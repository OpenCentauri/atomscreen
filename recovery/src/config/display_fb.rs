use crate::{
    config::DisplayInit,
    AppWindow,
};
use driver::init_framebuffer;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DisplayFramebufferConfig {
    pub fb_path: String,
    pub event_path: Option<String>,
    pub buffering: Option<String>,
}

impl DisplayInit for DisplayFramebufferConfig {
    fn init(&self) -> AppWindow {
        let double_buffering = match self.buffering.clone().unwrap_or(String::from("double")).to_lowercase().as_str()
        {
            "double" => true,
            "single" => false,
            _ => panic!("Config value {:?} is unsupported for buffering. Supported is \"Single\" or \"Double\"", self.buffering)
        };

        slint::platform::set_platform(init_framebuffer(self.fb_path.clone(), self.event_path.clone(), double_buffering)).expect("Failed to set platform");
        let ui = AppWindow::new().expect("Failed to initialize window");

        ui
    }
}
