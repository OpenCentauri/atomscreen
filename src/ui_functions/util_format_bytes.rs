use slint::{ComponentHandle, SharedString};

use crate::{AppWindow, Utils};

const UNITS: &[&str] = &["B", "KiB", "MiB", "GiB", "TiB", "PiB"];

pub fn register_util_format_bytes(ui : &AppWindow)
{
    ui.global::<Utils>().on_format_bytes(|bytes| {
        let mut bytes = bytes as f64;
        let mut idx = 0;

        while bytes > 1024.0 {
            bytes /= 1024.0;
            idx += 1;
        }        
        SharedString::from(format!("{:.2} {}", bytes, UNITS[idx])) 
    });
}