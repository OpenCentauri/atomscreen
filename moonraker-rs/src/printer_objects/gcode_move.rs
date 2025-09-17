use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct GcodeMove {
    pub speed_factor: f32,
    pub speed: f32,
    pub extruder_factor: Option<f32>,
    pub absolute_coordinates: bool,
    pub absolute_extrude: bool,
    pub homing_origin: [f32; 4],
    pub position: [f32; 4],
    pub gcode_position: [f32; 4],
}
