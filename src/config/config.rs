use serde::{Deserialize};
use crate::config::MoonrakerConfig;

use super::{DisplayConfig};

#[derive(Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub moonraker: Option<MoonrakerConfig>
}