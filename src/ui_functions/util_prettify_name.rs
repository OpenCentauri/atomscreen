use slint::{ComponentHandle, SharedString};

use crate::{AppWindow, Utils};

pub fn register_util_prettify_name(ui : &AppWindow)
{
    ui.global::<Utils>().on_prettify_name(|name| {
        let name = name.to_string();

        let new_name = name.split("_").map(|s| {
            let mut chars = s.chars();
            match chars.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + chars.as_str(),
            }
        }).collect::<Vec<String>>().join(" ");
        
        SharedString::from(new_name)
    });
}