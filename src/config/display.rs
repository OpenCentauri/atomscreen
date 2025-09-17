use crate::{application_error, AppWindow};
use super::DisplayDefaultConfig;
#[cfg(unix)]
use super::DisplayFramebufferConfig;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct DisplayConfig {
    pub default: Option<DisplayDefaultConfig>,
    #[cfg(unix)]
    pub framebuffer: Option<DisplayFramebufferConfig>,
}

pub trait DisplayInit {
    fn init(&self) -> Result<AppWindow, application_error::ApplicationError>;
}