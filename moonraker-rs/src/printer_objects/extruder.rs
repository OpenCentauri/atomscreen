use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Extruder {
    pub temperature: f32,
    pub target: f32,
    pub power: f32,
    pub can_extrude: bool,
    pub pressure_advance: f32,
    pub smooth_time: f32,
    pub motion_queue: Option<String>,
}