use crate::config::MoonrakerConfig;
use serde::Deserialize;

use super::DisplayConfig;

#[derive(Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub moonraker: Option<MoonrakerConfig>,
}
