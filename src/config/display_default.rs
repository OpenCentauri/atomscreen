use super::DisplayInit;
use crate::{application_error, AppWindow};
use serde::{Deserialize, Serialize};
use slint::ComponentHandle;

#[derive(Serialize, Deserialize)]
pub struct DisplayDefaultConfig {
    pub width: u32,
    pub height: u32,
}

impl DisplayInit for DisplayDefaultConfig {
    fn init(&self) -> Result<AppWindow, application_error::ApplicationError> {
        let app = AppWindow::new()?;

        app.window()
            .set_size(slint::PhysicalSize::new(self.width, self.height));

        Ok(app)
    }
}
