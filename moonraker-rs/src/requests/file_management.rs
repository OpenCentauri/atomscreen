use serde::Deserialize;

use crate::{error::Error, moonraker_connection::{MoonrakerConnection}};

#[derive(Debug, Deserialize)]
pub struct MoonrakerFile
{
    pub path: String,
    pub modified: f32,
    pub size: i32,
    pub permissions: String
}

#[derive(Debug, Deserialize)]
pub struct MoonrakerFileThumbnail
{
    pub width: i32,
    pub height: i32,
    pub size: i32,
    pub thumbnail_path: String,
}

pub trait FileManagementRequestHandler
{
    async fn list_files(&self, root: &str) -> Result<Vec<MoonrakerFile>, Error>;
    async fn list_gcode_files(&self) -> Result<Vec<MoonrakerFile>, Error>;
    async fn get_thumbnails_for_file(&self, file: &str) -> Result<Vec<MoonrakerFileThumbnail>, Error>;
}

impl FileManagementRequestHandler for MoonrakerConnection
{
    async fn list_files(&self, root: &str) -> Result<Vec<MoonrakerFile>, Error> {
        let args = serde_json::json!({
            "root": root,
        });
        self.send_request("server.files.list", Some(args)).await
    }

    async fn list_gcode_files(&self) -> Result<Vec<MoonrakerFile>, Error> {
        self.list_files("gcodes").await
    }

    async fn get_thumbnails_for_file(&self, filename: &str) -> Result<Vec<MoonrakerFileThumbnail>, Error> {
        let args = serde_json::json!({"filename": filename});
        self.send_request("server.files.thumbnails", Some(args)).await
    }
}