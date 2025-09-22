use serde::Deserialize;

#[derive(Debug, Deserialize, Clone, PartialEq)]
pub struct TempControl
{
    pub min_temp: i32,
    pub max_temp: i32,
    pub step_temp: i32,
}

impl TempControl {
    pub fn default_hotend() -> Self {
        Self {
            min_temp: 180,
            max_temp: 300,
            step_temp: 5,
        }
    }
    pub fn default_bed() -> Self {
        Self {
            min_temp: 30,
            max_temp: 120,
            step_temp: 5,
        }
    }
    pub fn default_fan() -> Self {
        Self {
            min_temp: 30,
            max_temp: 60,
            step_temp: 5,
        }
    }
}