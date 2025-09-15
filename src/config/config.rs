use serde::{Deserialize, Serialize};
use super::{DisplayConfig};

#[derive(Serialize, Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
}