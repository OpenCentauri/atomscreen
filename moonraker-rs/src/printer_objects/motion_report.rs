use optional_struct::*;
use serde::Deserialize;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct MotionReport {
    pub live_position: [f32; 4],
    pub live_velocity: f32,
    pub live_extruder_velocity: f32,
    pub steppers: Vec<String>,
    pub trapq: Vec<String>,
}

impl MotionReport {
    pub fn overlay(&mut self, motion: OptionalMotionReport) {
        if let Some(live_position) = motion.live_position {
            self.live_position = live_position;
        }
        if let Some(live_velocity) = motion.live_velocity {
            self.live_velocity = live_velocity;
        }
        if let Some(live_extruder_velocity) = motion.live_extruder_velocity {
            self.live_extruder_velocity = live_extruder_velocity;
        }
        if let Some(steppers) = motion.steppers {
            self.steppers = steppers;
        }
        if let Some(trapq) = motion.trapq {
            self.trapq = trapq;
        }
    }
}
