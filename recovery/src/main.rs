use std::{collections::HashMap, env, fs};
use slint::{ModelRc, SharedString, VecModel};

use crate::config::{DisplayConfig, DisplayInit, Script, ScriptInner};

mod config;
mod task_exec;

slint::include_modules!();

fn init_display(config: &DisplayConfig) -> AppWindow {
    if let Some(default_config) = &config.default {
        return default_config.init();
    }

    #[cfg(unix)]
    if let Some(fb_config) = &config.framebuffer {
        return fb_config.init();
    }

    panic!("No display driver configured");
}

fn to_list(hashmap: HashMap<String, ScriptInner>) -> Vec<Script> {
    let mut list = Vec::new();
    let mut temp_list = Vec::new();

    for (name, inner) in hashmap {
        let index = inner.index.unwrap_or_default();
        temp_list.push((index, Script::from_inner(inner, name)));
    }

    temp_list.sort_by_key(|(index, _)| *index);

    for (_, script) in temp_list {
        list.push(script);
    }

    list
}

fn main() {
    println!("Hello, world!");

    let args = env::args().collect::<Vec<String>>();

    let config_path = match args.get(1) {
        Some(path) => path.clone(),
        None => "config.toml".to_string(),
    };

    let config_str = fs::read_to_string(&config_path).unwrap();
    let config = toml::from_str::<config::Config>(&config_str).unwrap();
    let actions = to_list(config.scripts);

    let ui = init_display(&config.display);

    ui.global::<State>().set_actions(ModelRc::new(VecModel::from(actions.iter().map(|a| SharedString::from(a.name.clone())).collect::<Vec<SharedString>>())));
    ui.global::<State>().set_window_title(SharedString::from(config.window_title.unwrap_or("Atomscreen Recovery".to_string())));

    task_exec::register_action_button_click(&ui, &actions);
    task_exec::register_action_start(&ui, &actions);

    ui.run().unwrap();
}
