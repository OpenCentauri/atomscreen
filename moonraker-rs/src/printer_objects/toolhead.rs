use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Toolhead {
    pub homed_axes: String,
    pub axis_minimum: [f32; 4],
    pub axis_maximum: [f32; 4],
    pub cone_start_z: Option<f32>,
    pub print_time: f32,
    pub stalls: i32,
    pub estimated_print_time: f32,
    pub extruder: String,
    pub position: [f32; 4],
    pub max_velocity: f32,
    pub max_accel: f32,
    pub minimum_cruise_ratio: f32,
    pub square_corner_velocity: f32,
}
