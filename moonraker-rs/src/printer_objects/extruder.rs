use optional_struct::*;
use serde::Deserialize;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct Extruder {
    pub temperature: f32,
    pub target: f32,
    pub power: f32,
    pub can_extrude: bool,
    pub pressure_advance: f32,
    pub smooth_time: f32,
    pub motion_queue: Option<String>,
}

impl Extruder {
    pub fn overlay(&mut self, extruder: OptionalExtruder) {
        if let Some(temperature) = extruder.temperature {
            self.temperature = temperature;
        }
        if let Some(target) = extruder.target {
            self.target = target;
        }
        if let Some(power) = extruder.power {
            self.power = power;
        }
        if let Some(can_extrude) = extruder.can_extrude {
            self.can_extrude = can_extrude;
        }
        if let Some(pressure_advance) = extruder.pressure_advance {
            self.pressure_advance = pressure_advance;
        }
        if let Some(smooth_time) = extruder.smooth_time {
            self.smooth_time = smooth_time;
        }
        if let Some(motion_queue) = extruder.motion_queue {
            self.motion_queue = Some(motion_queue);
        }
    }
}
