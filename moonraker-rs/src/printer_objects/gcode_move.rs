use optional_struct::*;
use serde::Deserialize;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct GcodeMove {
    pub speed_factor: f32,
    pub speed: f32,
    pub extruder_factor: f32,
    pub absolute_coordinates: bool,
    pub absolute_extrude: bool,
    pub homing_origin: [f32; 4],
    pub position: [f32; 4],
    pub gcode_position: [f32; 4],
}

impl GcodeMove {
    pub fn overlay(&mut self, gcode_move: OptionalGcodeMove) {
        if let Some(speed_factor) = gcode_move.speed_factor {
            self.speed_factor = speed_factor;
        }
        if let Some(speed) = gcode_move.speed {
            self.speed = speed;
        }
        if let Some(extruder_factor) = gcode_move.extruder_factor {
            self.extruder_factor = extruder_factor;
        }
        if let Some(absolute_coordinates) = gcode_move.absolute_coordinates {
            self.absolute_coordinates = absolute_coordinates;
        }
        if let Some(absolute_extrude) = gcode_move.absolute_extrude {
            self.absolute_extrude = absolute_extrude;
        }
        if let Some(homing_origin) = gcode_move.homing_origin {
            self.homing_origin = homing_origin;
        }
        if let Some(position) = gcode_move.position {
            self.position = position;
        }
        if let Some(gcode_position) = gcode_move.gcode_position {
            self.gcode_position = gcode_position;
        }
    }
}
