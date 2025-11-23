use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct ScriptButtonAction {
    pub name: String,
    pub command: Vec<String>,
}

#[derive(Deserialize, Clone)]
pub struct ScriptTaskRaw {
    pub name: String,
    pub command: Vec<String>,
    pub show_stdout: Option<bool>,
    pub stop_on_failure: Option<bool>,
}

#[derive(Deserialize)]
pub struct ScriptInner {
    pub completed_actions: Option<Vec<ScriptButtonAction>>,
    pub tasks: Vec<ScriptTaskRaw>,
    pub hide_pending_tasks: Option<bool>,
    pub index: Option<usize>,
}

#[derive(Clone)]
pub struct Script {
    pub name: String,
    pub completed_actions: Vec<ScriptButtonAction>,
    pub tasks: Vec<ScriptTaskRaw>,
    pub hide_pending_tasks: bool,
}

impl Script {
    pub fn from_inner(inner: ScriptInner, name: String) -> Self {
        Script {
            name,
            completed_actions: inner.completed_actions.unwrap_or(vec![ ScriptButtonAction {
                name: "Back".to_string(),
                command: vec!["__back".into()],
            } ]),
            tasks: inner.tasks,
            hide_pending_tasks: inner.hide_pending_tasks.unwrap_or(false),
        }
    }
}