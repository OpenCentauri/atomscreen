use crate::AppWindow;

use super::DisplayInit;
use serde::{Deserialize, Serialize};
use slint::ComponentHandle;

#[derive(Serialize, Deserialize)]
pub struct DisplayDefaultConfig {
    pub width: u32,
    pub height: u32,
}

impl DisplayInit for DisplayDefaultConfig {
    fn init(&self) -> AppWindow {
        let app = AppWindow::new().expect("Failed to initialize window");

        app.window()
            .set_size(slint::PhysicalSize::new(self.width, self.height));

        app
    }
}
