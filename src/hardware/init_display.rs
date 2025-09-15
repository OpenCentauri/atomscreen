use crate::{application_error::ApplicationError, config::{DisplayConfig, DisplayInit}, AppWindow};

pub fn init_display(config: &DisplayConfig) -> Result<AppWindow, ApplicationError> {
    if let Some(default_config) = &config.default
    {
        return default_config.init();
    }

    if let Some(fb_config) = &config.framebuffer
    {
        return fb_config.init();
    }

    Err(ApplicationError::Unknown(String::from("No display driver configured")))
}