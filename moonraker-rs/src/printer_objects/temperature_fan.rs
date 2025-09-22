use optional_struct::*;
use serde::Deserialize;

use crate::printer_objects::TempControl;

#[optional_struct]
#[derive(Debug, Deserialize, Clone)]
pub struct TemperatureFan {
    pub speed: f32,
    pub rpm: Option<f32>,
    pub temperature: f32,
    pub target: f32,
    pub temp_control: TempControl
}

impl Default for TemperatureFan {
    fn default() -> Self {
        Self {
            speed: 0.0,
            rpm: None,
            temperature: 0.0,
            target: 0.0,
            temp_control: TempControl::default_fan(),
        }
    }
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
        if let Some(temp_control) = fan.temp_control {
            self.temp_control = temp_control;
        }
    }
}
