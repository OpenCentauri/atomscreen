use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct HeaterBed {
    pub temperature: f32,
    pub target: f32,
    pub power: f32,
}