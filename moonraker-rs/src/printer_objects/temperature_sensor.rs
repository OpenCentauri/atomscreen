use serde::Deserialize;
use optional_struct::*;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct TemperatureSensor {
    pub temperature: f32,
    pub measured_min_temp: f32,
    pub measured_max_temp: f32,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedTemperatureSensor {
    pub name: String,
    pub sensor: TemperatureSensor,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedOptionalTemperatureSensor {
    pub name: String,
    pub sensor: OptionalTemperatureSensor,
}

impl TemperatureSensor {
    pub fn overlay(&mut self, sensor: OptionalTemperatureSensor) {
        if let Some(temperature) = sensor.temperature {
            self.temperature = temperature;
        }
        if let Some(measured_min_temp) = sensor.measured_min_temp {
            self.measured_min_temp = measured_min_temp;
        }
        if let Some(measured_max_temp) = sensor.measured_max_temp {
            self.measured_max_temp = measured_max_temp;
        }
    }
}
