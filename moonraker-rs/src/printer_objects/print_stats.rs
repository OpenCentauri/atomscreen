use serde::Deserialize;
use optional_struct::*;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
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

#[derive(Debug, Deserialize, Default, Clone, PartialEq, Eq)]
pub struct PrintStatsInfo {
    pub total_layer: Option<i32>,
    pub current_layer: Option<i32>,
}

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct PrintStats {
    pub filename: String,
    pub total_duration: f32,
    pub print_duration: f32,
    pub filament_used: f32,
    pub state: PrintState,
    pub message: String,
    pub info: PrintStatsInfo,
}

impl PrintStats {
    pub fn overlay(&mut self, print_stats: OptionalPrintStats) {
        if let Some(filename) = print_stats.filename {
            self.filename = filename;
        }
        if let Some(total_duration) = print_stats.total_duration {
            self.total_duration = total_duration;
        }
        if let Some(print_duration) = print_stats.print_duration {
            self.print_duration = print_duration;
        }
        if let Some(filament_used) = print_stats.filament_used {
            self.filament_used = filament_used;
        }
        if let Some(state) = print_stats.state {
            self.state = state;
        }
        if let Some(message) = print_stats.message {
            self.message = message;
        }
        if let Some(info) = print_stats.info {
            self.info = info;
        }
    }
}
