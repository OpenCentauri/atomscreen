use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct MotionReport
{
    pub live_position: [f32; 4],
    pub live_velocity: f32,
    pub live_extruder_velocity: f32,
    pub steppers: Vec<String>,
    pub trapq: Vec<String>,
}