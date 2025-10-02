use optional_struct::optional_struct;
use serde::Deserialize;

#[optional_struct]
#[derive(Deserialize, Debug)]
pub struct UiConfig {
    pub dark_mode: bool,
    pub left_sidebar: Vec<String>,
    pub right_sidebar: Vec<String>,
}

impl UiConfig {
    pub fn from_optional(ui_config : &OptionalUiConfig) -> UiConfig {
        UiConfig {
            dark_mode: ui_config.dark_mode.unwrap_or(UiConfig::default().dark_mode),
            left_sidebar: ui_config.left_sidebar.clone().unwrap_or(UiConfig::default().left_sidebar),
            right_sidebar: ui_config.right_sidebar.clone().unwrap_or(UiConfig::default().right_sidebar),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            dark_mode: true,
            left_sidebar: vec![
                "files".into(),
                "temperature".into(),
                "move".into(),
                "emergency_stop".into(),
            ],
            right_sidebar: vec![
                "fan".into(),
                "macros".into(),
                "console".into(),
                "settings".into(),
            ],
        }
    }
}
