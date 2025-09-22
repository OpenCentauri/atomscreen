use optional_struct::*;
use serde::Deserialize;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct TemperatureFan {
    pub speed: f32,
    pub rpm: Option<f32>,
    pub temperature: f32,
    pub target: f32,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedTemperatureFan {
    pub name: String,
    pub fan: TemperatureFan,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedOptionalTemperatureFan {
    pub name: String,
    pub fan: OptionalTemperatureFan,
}

impl TemperatureFan {
    pub fn overlay(&mut self, fan: OptionalTemperatureFan) {
        if let Some(speed) = fan.speed {
            self.speed = speed;
        }
        if let Some(rpm) = fan.rpm {
            self.rpm = Some(rpm);
        }
        if let Some(temperature) = fan.temperature {
            self.temperature = temperature;
        }
        if let Some(target) = fan.target {
            self.target = target;
        }
    }
}
