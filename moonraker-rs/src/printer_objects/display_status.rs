use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct DisplayStatus {
    pub message: Option<String>,
    pub progress: f32,
}