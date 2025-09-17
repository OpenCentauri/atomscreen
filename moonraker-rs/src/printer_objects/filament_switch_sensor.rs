use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct FilamentSwitchSensor {
    pub filament_detected: bool,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Default)]
pub struct NamedFilamentSwitchSensor {
    pub name: String,
    pub sensor: FilamentSwitchSensor,
}