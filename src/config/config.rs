use std::collections::HashMap;

use crate::config::{MoonrakerConfig, OptionalGcodeCommands, OptionalUiConfig};
use serde::Deserialize;

use super::DisplayConfig;

#[derive(Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub moonraker: Option<MoonrakerConfig>,
    pub heater_presets: Option<HashMap<String, Vec<u32>>>,
    pub gcode_commands: Option<OptionalGcodeCommands>,
    pub ui: Option<OptionalUiConfig>,
    pub quick_actions: Option<HashMap<String, Vec<String>>>,
}
