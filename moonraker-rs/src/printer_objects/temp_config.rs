use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TemperatureConfiguration
{
    pub presets : Vec<u32>
}

impl TemperatureConfiguration {
    pub fn default_hotend() -> Self {
        Self {
            presets: vec![200, 240, 260],
        }
    }
    pub fn default_bed() -> Self {
        Self {
            presets: vec![40, 60, 70],
        }
    }
    pub fn default_fan() -> Self {
        Self {
            presets: vec![40, 50, 60],
        }
    }
    pub fn from(presets: Vec<u32>) -> Self {
        Self { presets }
    }
}