use optional_struct::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "PascalCase")]
pub enum IdleTimeoutState {
    Printing,
    Ready,
    Idle,
}

impl Default for IdleTimeoutState {
    fn default() -> Self {
        IdleTimeoutState::Idle
    }
}

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct IdleTimeout {
    pub state: IdleTimeoutState,
    pub printing_time: f32,
}

impl IdleTimeout {
    pub fn overlay(&mut self, idle: OptionalIdleTimeout) {
        if let Some(state) = idle.state {
            self.state = state;
        }
        if let Some(printing_time) = idle.printing_time {
            self.printing_time = printing_time;
        }
    }
}
