use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct TemperatureSensor {
    pub temperature: f32,
    pub measured_min_temp: f32,
    pub measured_max_temp: f32,
}

#[derive(Debug, Deserialize, Default)]
pub struct NamedTemperatureSensor {
    pub name: String,
    pub sensor: TemperatureSensor,
}