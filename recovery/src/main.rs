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

fn to_shared_string_model(values: impl IntoIterator<Item = String>) -> ModelRc<SharedString> {
    ModelRc::new(VecModel::from(
        values
            .into_iter()
            .map(SharedString::from)
            .collect::<Vec<SharedString>>(),
    ))
}

fn register_action_submenu_enter(
    ui: &AppWindow,
    submenu_scripts: &HashMap<String, Vec<Script>>,
) {
    let ui_weak = ui.as_weak();
    let submenu_scripts = submenu_scripts.clone();

    ui.global::<State>().on_action_submenu_enter(move |submenu_name| {
        let Some(submenu_actions) = submenu_scripts.get(submenu_name.as_str()) else {
            return;
        };

        let submenu_name = submenu_name.to_string();
        let submenu_actions = submenu_actions.clone();
        let ui_weak = ui_weak.clone();

        ui_weak
            .upgrade_in_event_loop(move |ui| {
                ui.global::<State>().set_actions(to_shared_string_model(
                    submenu_actions.into_iter().map(|action| action.name),
                ));
                ui.global::<State>().set_submenus(to_shared_string_model(Vec::new()));
                ui.global::<State>().set_show_back(true);
                ui.global::<State>().set_window_title(SharedString::from(submenu_name));
            })
            .expect("Failed to run in event loop");
    });
}

fn register_action_exit_submenu(
    ui: &AppWindow,
    root_actions: &[Script],
    root_submenus: &[String],
    root_window_title: &str,
) {
    let ui_weak = ui.as_weak();
    let root_actions = root_actions.to_vec();
    let root_submenus = root_submenus.to_vec();
    let root_window_title = root_window_title.to_string();

    ui.global::<State>().on_action_exit_submenu(move || {
        let ui_weak = ui_weak.clone();
        let root_actions = root_actions.clone();
        let root_submenus = root_submenus.clone();
        let root_window_title = root_window_title.clone();

        ui_weak
            .upgrade_in_event_loop(move |ui| {
                ui.global::<State>().set_actions(to_shared_string_model(
                    root_actions.into_iter().map(|action| action.name),
                ));
                ui.global::<State>().set_submenus(to_shared_string_model(root_submenus));
                ui.global::<State>().set_show_back(false);
                ui.global::<State>().set_window_title(SharedString::from(root_window_title));
            })
            .expect("Failed to run in event loop");
    });
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
    let root_actions = to_list(config.scripts);
    let mut submenu_scripts = config
        .submenus
        .unwrap_or_default()
        .into_iter()
        .map(|(submenu_name, scripts)| (submenu_name, to_list(scripts)))
        .collect::<HashMap<String, Vec<Script>>>();
    let mut root_submenus = submenu_scripts.keys().cloned().collect::<Vec<String>>();
    root_submenus.sort();
    let root_window_title = config
        .window_title
        .unwrap_or("Atomscreen Recovery".to_string());

    let mut all_scripts = root_actions
        .iter()
        .cloned()
        .map(|script| (script.name.clone(), script))
        .collect::<HashMap<String, Script>>();

    for scripts in submenu_scripts.values_mut() {
        for script in scripts.iter().cloned() {
            all_scripts.insert(script.name.clone(), script);
        }
    }

    let ui = init_display(&config.display);

    ui.global::<State>().set_actions(to_shared_string_model(
        root_actions.iter().map(|action| action.name.clone()),
    ));
    ui.global::<State>().set_submenus(to_shared_string_model(root_submenus.clone()));
    ui.global::<State>().set_show_back(false);
    ui.global::<State>().set_window_title(SharedString::from(root_window_title.clone()));

    register_action_submenu_enter(&ui, &submenu_scripts);
    register_action_exit_submenu(&ui, &root_actions, &root_submenus, &root_window_title);

    task_exec::register_action_button_click(&ui, &all_scripts);
    task_exec::register_action_start(&ui, &all_scripts);

    ui.run().unwrap();
}
