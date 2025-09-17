use serde::Deserialize;
use optional_struct::*;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct VirtualSdcard {
    pub file_path: Option<String>,
    pub progress: f32,
    pub is_active: bool,
    pub file_position: i32,
    pub file_size: i32,
}

impl VirtualSdcard {
    pub fn overlay(&mut self, vsd: OptionalVirtualSdcard) {
        if let Some(file_path) = vsd.file_path {
            self.file_path = Some(file_path);
        }
        if let Some(progress) = vsd.progress {
            self.progress = progress;
        }
        if let Some(is_active) = vsd.is_active {
            self.is_active = is_active;
        }
        if let Some(file_position) = vsd.file_position {
            self.file_position = file_position;
        }
        if let Some(file_size) = vsd.file_size {
            self.file_size = file_size;
        }
    }
}
