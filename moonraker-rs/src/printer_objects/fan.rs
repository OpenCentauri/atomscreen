use serde::Deserialize;
use optional_struct::*;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Fan {
    pub speed: f32,
    pub rpm: Option<i32>,
}

impl Fan {
    pub fn overlay(&mut self, fan: OptionalFan) {
        if let Some(speed) = fan.speed {
            self.speed = speed;
        }
        if let Some(rpm) = fan.rpm {
            self.rpm = Some(rpm);
        }
    }
}

