use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PrintState {
    Standby,
    Printing,
    Paused,
    Complete,
    Error,
    Cancelled,
}

impl Default for PrintState {
    fn default() -> Self {
        PrintState::Standby
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct PrintStatsInfo {
    pub total_layer: Option<i32>,
    pub current_layer: Option<i32>,
}

#[derive(Debug, Deserialize, Default)]
pub struct PrintStats {
    pub filename: String,
    pub total_duration: f32,
    pub print_duration: f32,
    pub filament_used: f32,
    pub state: PrintState,
    pub message: String,
    pub info: PrintStatsInfo,
}
