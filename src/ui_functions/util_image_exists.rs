use slint::ComponentHandle;

use crate::{AppWindow, Utils};

pub fn register_util_image_exists(ui : &AppWindow)
{
    ui.global::<Utils>().on_image_exists(|f| {
        return f.size().area() > 0;
    });
}