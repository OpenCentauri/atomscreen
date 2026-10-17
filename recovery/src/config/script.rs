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
    #[serde(default)]
    pub disabled: bool,
    #[serde(default)]
    pub disabled_reason: String,
}

#[derive(Clone)]
pub struct Script {
    pub name: String,
    pub completed_actions: Vec<ScriptButtonAction>,
    pub tasks: Vec<ScriptTaskRaw>,
    pub hide_pending_tasks: bool,
    pub disabled: bool,
    pub disabled_reason: String,
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
            disabled: inner.disabled,
            disabled_reason: inner.disabled_reason,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Script, ScriptInner};

    #[test]
    fn disabled_fields_default_to_enabled_and_preserve_reason() {
        let legacy: ScriptInner = toml::from_str("tasks = []").unwrap();
        let legacy = Script::from_inner(legacy, "Legacy".into());
        assert!(!legacy.disabled);
        assert!(legacy.disabled_reason.is_empty());
        assert_eq!(legacy.completed_actions[0].command, ["__back"]);

        let disabled: ScriptInner = toml::from_str(
            "tasks = []\ndisabled = true\ndisabled_reason = 'Running OC-patched, root shell already present'",
        ).unwrap();
        let disabled = Script::from_inner(disabled, "Disabled".into());
        assert!(disabled.disabled);
        assert_eq!(disabled.disabled_reason, "Running OC-patched, root shell already present");

        let enabled: ScriptInner = toml::from_str(
            "tasks = []\ndisabled = false\ndisabled_reason = \"Arbitrary note: Å\\nsecond line\"",
        ).unwrap();
        let enabled = Script::from_inner(enabled, "Enabled".into());
        assert!(!enabled.disabled);
        assert_eq!(enabled.disabled_reason, "Arbitrary note: Å\nsecond line");
    }
}