
use std::collections::HashMap;

use indexmap::IndexMap;
use serde::Deserialize;

use crate::config::ScriptInner;

use super::DisplayConfig;

#[derive(Deserialize)]
pub struct Config {
    pub display: DisplayConfig,
    pub scripts: IndexMap<String, ScriptInner>,
    pub window_title: Option<String>,
}
