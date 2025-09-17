use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct TemperatureFan {
    pub speed: f32,
    pub rpm: Option<f32>,
    pub temperature: f32,
    pub target: f32,
}

#[derive(Debug, Deserialize, Default)]
pub struct NamedTemperatureFan {
    pub name: String,
    pub fan: TemperatureFan,
}
