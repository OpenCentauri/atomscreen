use slint::{ComponentHandle, Model, ModelRc, SharedString, VecModel};

use crate::{AppWindow, Utils};

pub fn register_create_temperature_lists(ui : &AppWindow)
{
    ui.global::<Utils>().on_create_temperature_lists(|presets| {
        let presets = presets.as_any().downcast_ref::<VecModel<i32>>().unwrap();
        let presets: Vec<i32> = presets.iter().collect();

        let mut list: Vec<SharedString> = Vec::new();
        list.push(SharedString::from("Off"));
        for preset in presets {
            list.push(SharedString::from(format!("{}°C", preset)));
        }

        list.push(SharedString::from("Set"));

        ModelRc::new(VecModel::from(list))
    });
}

pub fn register_convert_temperature_back(ui : &AppWindow)
{
    ui.global::<Utils>().on_convert_temperature_back(|f| {
        let s = f.trim_end_matches("°C");
        match s.parse::<i32>() {
            Ok(v) => v,
            Err(_) => 0,
        }
    });
}