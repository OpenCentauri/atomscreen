use std::{process::Command, thread};

use slint::{ComponentHandle, ModelRc, SharedString, VecModel, Weak};

use crate::{AppWindow, State, Task, config::{Script, ScriptButtonAction, ScriptTaskRaw}};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum ScriptTaskState {
    Pending = 0,
    Running = 1,
    Success = 2,
    Failed = 3,
}

#[derive(Clone)]
pub struct ScriptTask {
    pub name: String,
    pub command: Vec<String>,
    pub show_stdout: bool,
    pub stop_on_failure: bool,
    pub state: ScriptTaskState,
    pub message: String,
}

impl ScriptTask {
    pub fn from_raw(raw: ScriptTaskRaw) -> Self {
        ScriptTask {
            name: raw.name,
            command: raw.command,
            show_stdout: raw.show_stdout.unwrap_or(false),
            stop_on_failure: raw.stop_on_failure.unwrap_or(true),
            state: ScriptTaskState::Pending,
            message: String::from(""),
        }
    }
}

pub fn handle_command(command: &[String], ui_weak: &Weak<AppWindow>) -> (i32, String) {
    if command.is_empty() {
        return (0, String::new());
    }

    match command[0].as_str() {
        "__back" => {
            ui_weak.upgrade_in_event_loop(move |ui| {
                ui.global::<State>().set_active_action("".into());
            }).expect("Failed to run in event loop");

            (0, String::new())
        }
        _ => {
            let output = Command::new(&command[0])
                .args(&command[1..])
                .output();

            let output = match output {
                Ok(output) => output,
                Err(e) => {
                    return (-1, format!("Failed to execute command: {}", e));
                }
            };

            return (
                output.status.code().unwrap_or(-1),
                String::from_utf8_lossy(&output.stdout).to_string(),
            );
        }
    }
}

pub fn register_action_button_click(ui: &AppWindow, scripts: &Vec<Script>) {
    let ui_weak = ui.as_weak();
    let scripts = scripts.clone();

    ui.global::<State>().on_action_button_click(move |action_name| {
        let ui_weak = ui_weak.clone();
        let ui_weak_2 = ui_weak.clone();
        let ui = ui_weak.upgrade().expect("Failed to upgrade weak reference to UI");

        let current_action_str = ui.global::<State>().get_active_action();
        let current_action_str = current_action_str.as_str();
        let current_action = match scripts.iter().find(|s| s.name == current_action_str) {
            Some(action) => action,
            None => return,
        };

        let pressed_action_button = match current_action.completed_actions.iter().find(|a| a.name == action_name.as_str()) {
            Some(action) => action,
            None => return,
        };

        let command = pressed_action_button.command.clone();

        thread::spawn(move || {
            let ui_weak = ui_weak_2;
            handle_command(&command, &ui_weak);
        });
    });
}

pub fn publish_script_tasks(ui_weak: Weak<AppWindow>, tasks: &[ScriptTask], hide_pending: bool) {
    let mut ui_tasks = Vec::new();

    for task in tasks {
        if task.state == ScriptTaskState::Pending && hide_pending {
            continue;
        }

        ui_tasks.push(Task {
            name: SharedString::from(task.name.clone()),
            state: task.state as i32,
            message: SharedString::from(task.message.clone())
        })
    }

    ui_weak.upgrade_in_event_loop(move |ui| {
        ui.global::<State>().set_active_action_tasks(ModelRc::new(VecModel::from(ui_tasks)));
    }).expect("Failed to run in event loop");
}

pub fn publish_script_buttons(ui_weak: Weak<AppWindow>, buttons: &[ScriptButtonAction]) {
    let mut ui_buttons = Vec::new();

    for button in buttons {
        ui_buttons.push(SharedString::from(button.name.clone()));
    }

    ui_weak.upgrade_in_event_loop(move |ui| {
        ui.global::<State>().set_active_action_buttons(ModelRc::new(VecModel::from(ui_buttons)));
    }).expect("Failed to run in event loop");
}

pub fn publish_active_action_state(ui_weak: Weak<AppWindow>, action_name: &str, action_state: ScriptTaskState) {
    let action_name = action_name.to_string();
    let action_state = action_state as i32;

    ui_weak.upgrade_in_event_loop(move |ui| {
        ui.global::<State>().set_active_action(SharedString::from(action_name));
        ui.global::<State>().set_active_action_state(action_state);
    }).expect("Failed to run in event loop");
}

pub fn process_script_tasks(ui_weak: Weak<AppWindow>, script: Script) {
    let mut tasks : Vec<ScriptTask> = script.tasks.into_iter().map(|x| ScriptTask::from_raw(x)).collect();
    let hide_pending = script.hide_pending_tasks;
    publish_script_tasks(ui_weak.clone(), &tasks, hide_pending);
    publish_script_buttons(ui_weak.clone(), &[]);
    publish_active_action_state(ui_weak.clone(), &script.name, ScriptTaskState::Running);

    for i in 0..tasks.len() {
        let read_only_task = tasks[i].clone();

        {
            let task = tasks.get_mut(i).unwrap();
            task.state = ScriptTaskState::Running;
        }

        publish_script_tasks(ui_weak.clone(), &tasks, hide_pending);
        
        let (exit_code, output) = handle_command(&read_only_task.command, &ui_weak);

        if read_only_task.show_stdout {
            let task = tasks.get_mut(i).unwrap();
            task.message = output;
        }

        if exit_code == 0 || !read_only_task.stop_on_failure {
            let task = tasks.get_mut(i).unwrap();
            task.state = ScriptTaskState::Success;
        } else {
            {
                let task = tasks.get_mut(i).unwrap();
                task.state = ScriptTaskState::Failed;
            }

            publish_script_tasks(ui_weak.clone(), &tasks, hide_pending);
            publish_active_action_state(ui_weak.clone(), &script.name, ScriptTaskState::Failed);
            publish_script_buttons(ui_weak, &script.completed_actions);
            return;
        }

        publish_script_tasks(ui_weak.clone(), &tasks, hide_pending);
    }

    publish_active_action_state(ui_weak.clone(), &script.name, ScriptTaskState::Success);
    publish_script_buttons(ui_weak, &script.completed_actions);
}

pub fn register_action_start(ui: &AppWindow, scripts: &Vec<Script>) {
    let ui_weak = ui.as_weak();
    let scripts = scripts.clone();

    ui.global::<State>().on_action_start(move |action_name| {
        let ui_weak = ui_weak.clone();

        let action_to_run = match scripts.iter().find(|s| s.name == action_name.as_str()) {
            Some(action) => action.clone(),
            None => return,
        };

        thread::spawn(move || {
            process_script_tasks(ui_weak, action_to_run);
        });
    });
}