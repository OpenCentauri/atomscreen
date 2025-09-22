use optional_struct::*;
use serde::Deserialize;

use crate::printer_objects::TempControl;

#[optional_struct]
#[derive(Debug, Deserialize, Clone)]
pub struct HeaterBed {
    pub temperature: f32,
    pub target: f32,
    pub power: f32,
    pub temp_control: TempControl,
}

impl Default for HeaterBed {
    fn default() -> Self {
        Self {
            temperature: 0.0,
            target: 0.0,
            power: 0.0,
            temp_control: TempControl::default_bed(),
        }
    }
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
        if let Some(temp_control) = heater_bed.temp_control {
            self.temp_control = temp_control;
        }
    }
}
