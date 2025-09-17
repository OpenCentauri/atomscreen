
use serde::Deserialize;


#[derive(Debug, Deserialize)]
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


#[derive(Debug, Deserialize, Default)]
pub struct IdleTimeout {
    pub state: IdleTimeoutState,
    pub printing_time: f32,
}
