use optional_struct::*;
use serde::Deserialize;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct HeaterBed {
    pub temperature: f32,
    pub target: f32,
    pub power: f32,
}

impl HeaterBed {
    pub fn overlay(&mut self, heater_bed: OptionalHeaterBed) {
        if let Some(temperature) = heater_bed.temperature {
            self.temperature = temperature;
        }
        if let Some(target) = heater_bed.target {
            self.target = target;
        }
        if let Some(power) = heater_bed.power {
            self.power = power;
        }
    }
}
