use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct VirtualSdcard {
    pub file_path: Option<String>,
    pub progress: f32,
    pub is_active: bool,
    pub file_position: i32,
    pub file_size: i32,
}