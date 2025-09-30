use optional_struct::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone)]
pub struct DisplayStatus {
    pub message: String,
    pub progress: f32,
}

#[derive(Debug, Default, Clone)]
pub struct OptionalDisplayStatus {
    pub message: Option<String>,
    pub progress: Option<f32>,
}

impl<'de> Deserialize<'de> for OptionalDisplayStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Map::deserialize(deserializer)?;

        let message = match value.contains_key("message")
        {
            true => value.get("message").and_then(|v| Some(v.as_str().map(|s| s.to_string()).unwrap_or(String::new()))),
            false => None,
        };

        let progress = match value.contains_key("progress")
        {
            true => value.get("progress").and_then(|v| v.as_f64().map(|f| f as f32)),
            false => None,
        };

         Ok(OptionalDisplayStatus {
            message,
            progress,
        })
    }
}

impl DisplayStatus {
    pub fn overlay(&mut self, display_status: OptionalDisplayStatus) {
        if let Some(message) = display_status.message {
            self.message = message;
        }
        if let Some(progress) = display_status.progress {
            self.progress = progress;
        }
    }
}
