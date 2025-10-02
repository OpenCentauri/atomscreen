use slint::{ComponentHandle, SharedString};

use crate::{AppWindow, Utils};

pub fn register_util_time_in_seconds_to_string(ui : &AppWindow)
{
    ui.global::<Utils>().on_time_in_seconds_to_string(|f: f32| {
        let total_seconds = f.round() as i32;
        let hours = total_seconds / 3600;
        let minutes = (total_seconds % 3600) / 60;
        let seconds = total_seconds % 60;

        if hours > 0 {
            SharedString::from(format!("{}h{:02}m", hours, minutes))
        } else if total_seconds >= 60 {
            SharedString::from(format!("{}m", minutes))
        } else {
            SharedString::from(format!("{}s", seconds))
        }
    });
} 