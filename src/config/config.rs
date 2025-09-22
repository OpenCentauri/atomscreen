use std::collections::HashMap;

use crate::config::MoonrakerConfig;
use moonraker_rs::printer_objects::TemperatureConfiguration;
use serde::Deserialize;

use super::DisplayConfig;

#[derive(Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub moonraker: Option<MoonrakerConfig>,
    pub heater_presets: Option<HashMap<String, Vec<u32>>>,
}
