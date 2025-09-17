use serde::Deserialize;
use optional_struct::*;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct DisplayStatus {
    pub message: String,
    pub progress: f32,
}

impl DisplayStatus {
    pub fn overlay(&mut self, display_status: OptionalDisplayStatus) {
        if let Some(message) = display_status.message {
            self.message = message;
        }
        if let Some(progress) = display_status.progress {
            self.progress = progress;
        }
    }
}