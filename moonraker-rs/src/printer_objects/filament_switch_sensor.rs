use optional_struct::*;
use serde::Deserialize;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct FilamentSwitchSensor {
    pub filament_detected: bool,
    pub enabled: bool,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedFilamentSwitchSensor {
    pub name: String,
    pub sensor: FilamentSwitchSensor,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedOptionalFilamentSwitchSensor {
    pub name: String,
    pub sensor: OptionalFilamentSwitchSensor,
}

impl FilamentSwitchSensor {
    pub fn overlay(&mut self, sensor: OptionalFilamentSwitchSensor) {
        if let Some(filament_detected) = sensor.filament_detected {
            self.filament_detected = filament_detected;
        }
        if let Some(enabled) = sensor.enabled {
            self.enabled = enabled;
        }
    }
}
