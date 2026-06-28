
use std::collections::HashMap;

use serde::Deserialize;

use crate::config::ScriptInner;

use super::DisplayConfig;

#[derive(Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub submenus: Option<HashMap<String, HashMap<String, ScriptInner>>>,
    pub scripts: HashMap<String, ScriptInner>,
    pub window_title: Option<String>,
}
