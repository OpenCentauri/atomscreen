use serde::Deserialize;
use optional_struct::*;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
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

impl Toolhead {
    pub fn overlay(&mut self, toolhead: OptionalToolhead) {
        if let Some(homed_axes) = toolhead.homed_axes {
            self.homed_axes = homed_axes;
        }
        if let Some(axis_minimum) = toolhead.axis_minimum {
            self.axis_minimum = axis_minimum;
        }
        if let Some(axis_maximum) = toolhead.axis_maximum {
            self.axis_maximum = axis_maximum;
        }
        if let Some(cone_start_z) = toolhead.cone_start_z {
            self.cone_start_z = Some(cone_start_z);
        }
        if let Some(print_time) = toolhead.print_time {
            self.print_time = print_time;
        }
        if let Some(stalls) = toolhead.stalls {
            self.stalls = stalls;
        }
        if let Some(estimated_print_time) = toolhead.estimated_print_time {
            self.estimated_print_time = estimated_print_time;
        }
        if let Some(extruder) = toolhead.extruder {
            self.extruder = extruder;
        }
        if let Some(position) = toolhead.position {
            self.position = position;
        }
        if let Some(max_velocity) = toolhead.max_velocity {
            self.max_velocity = max_velocity;
        }
        if let Some(max_accel) = toolhead.max_accel {
            self.max_accel = max_accel;
        }
        if let Some(minimum_cruise_ratio) = toolhead.minimum_cruise_ratio {
            self.minimum_cruise_ratio = minimum_cruise_ratio;
        }
        if let Some(square_corner_velocity) = toolhead.square_corner_velocity {
            self.square_corner_velocity = square_corner_velocity;
        }
    }
}
