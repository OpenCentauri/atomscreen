use std::path::PathBuf;

use optional_struct::optional_struct;
use serde::Deserialize;

use crate::{error::Error, moonraker_connection::MoonrakerConnection};

#[derive(Debug, Deserialize)]
pub struct MoonrakerFile {
    pub path: String,
    pub modified: f32,
    pub size: i32,
    pub permissions: String,
}

#[derive(Debug, Deserialize, PartialEq, Clone)]
pub struct MoonrakerFileThumbnail {
    pub width: i32,
    pub height: i32,
    pub size: i32,
    #[serde(alias = "relative_path")]
    pub thumbnail_path: String,
}

#[optional_struct]
#[derive(Debug, Deserialize, Default)]
pub struct GcodeMetadata {
    pub size: i32,
    pub modified: f32,
    pub uuid: String,
    pub file_processors: Vec<String>,
    pub slicer: String,
    pub slicer_version: String,
    pub gcode_start_byte: i32,
    pub gcode_int_byte: i32,
    pub object_height: f32,
    pub estimated_time: f32,
    pub nozzle_diameter: f32,
    pub layer_height: f32,
    pub first_layer_height: f32,
    pub first_layer_extr_temp: f32,
    pub first_layer_bed_temp: f32,
    pub chamber_temp: f32,
    pub filament_name: String,
    pub filament_colors: Vec<String>,
    pub extruder_colors: Vec<String>,
    pub filament_temps: Vec<i32>,
    pub filament_type: String,
    pub filament_total: f32,
    pub filament_change_count: i32,
    pub filament_weight_total: f32,
    pub filament_weights: Vec<f32>,
    pub mmu_print: i32,
    pub referenced_tools: Vec<i32>,
    pub thumbnails: Vec<MoonrakerFileThumbnail>,
    pub job_id: Option<String>,
    pub print_start_time: Option<f32>,
    pub filename: String,
}

pub trait FileManagementRequestHandler {
    async fn list_files(&self, root: &str) -> Result<Vec<MoonrakerFile>, Error>;
    async fn list_gcode_files(&self) -> Result<Vec<MoonrakerFile>, Error>;
    async fn get_thumbnails_for_file(
        &self,
        file: &str,
    ) -> Result<Vec<MoonrakerFileThumbnail>, Error>;
    async fn get_gcode_metadata_for_file(&self, filename: &str) -> Result<GcodeMetadata, Error>;
}

impl FileManagementRequestHandler for MoonrakerConnection {
    async fn list_files(&self, root: &str) -> Result<Vec<MoonrakerFile>, Error> {
        let args = serde_json::json!({
            "root": root,
        });
        self.send_request("server.files.list", Some(args)).await
    }

    async fn list_gcode_files(&self) -> Result<Vec<MoonrakerFile>, Error> {
        self.list_files("gcodes").await
    }

    async fn get_thumbnails_for_file(
        &self,
        filename: &str,
    ) -> Result<Vec<MoonrakerFileThumbnail>, Error> {
        let args = serde_json::json!({"filename": filename});
        self.send_request("server.files.thumbnails", Some(args))
            .await
    }

    async fn get_gcode_metadata_for_file(&self, filename: &str) -> Result<GcodeMetadata, Error>
    {
        let args = serde_json::json!({"filename": filename});
        let gcode_metadata : OptionalGcodeMetadata = self.send_request("server.files.metadata", Some(args)).await?;

        Ok(GcodeMetadata::from_optional(gcode_metadata))
    }
}

impl GcodeMetadata {
    pub fn from_optional(optional: OptionalGcodeMetadata) -> Self {
        Self {
            size: optional.size.unwrap_or_default(),
            modified: optional.modified.unwrap_or_default(),
            uuid: optional.uuid.unwrap_or_default(),
            file_processors: optional.file_processors.unwrap_or_default(),
            slicer: optional.slicer.unwrap_or_default(),
            slicer_version: optional.slicer_version.unwrap_or_default(),
            gcode_start_byte: optional.gcode_start_byte.unwrap_or_default(),
            gcode_int_byte: optional.gcode_int_byte.unwrap_or_default(),
            object_height: optional.object_height.unwrap_or_default(),
            estimated_time: optional.estimated_time.unwrap_or_default(),
            nozzle_diameter: optional.nozzle_diameter.unwrap_or_default(),
            layer_height: optional.layer_height.unwrap_or_default(),
            first_layer_height: optional.first_layer_height.unwrap_or_default(),
            first_layer_extr_temp: optional.first_layer_extr_temp.unwrap_or_default(),
            first_layer_bed_temp: optional.first_layer_bed_temp.unwrap_or_default(),
            chamber_temp: optional.chamber_temp.unwrap_or_default(),
            filament_name: optional.filament_name.unwrap_or_default(),
            filament_colors: optional.filament_colors.unwrap_or_default(),
            extruder_colors: optional.extruder_colors.unwrap_or_default(),
            filament_temps: optional.filament_temps.unwrap_or_default(),
            filament_type: optional.filament_type.unwrap_or_default(),
            filament_total: optional.filament_total.unwrap_or_default(),
            filament_change_count: optional.filament_change_count.unwrap_or_default(),
            filament_weight_total: optional.filament_weight_total.unwrap_or_default(),
            filament_weights: optional.filament_weights.unwrap_or_default(),
            mmu_print: optional.mmu_print.unwrap_or_default(),
            referenced_tools: optional.referenced_tools.unwrap_or_default(),
            thumbnails: optional.thumbnails.unwrap_or_default(),
            job_id: optional.job_id.clone(),
            print_start_time: optional.print_start_time.clone(),
            filename: optional.filename.unwrap_or_default(),
        }
    }

    pub fn absolute_thumbnails(&self) -> Vec<MoonrakerFileThumbnail>
    {
        let buf = PathBuf::from(&self.filename);
        let base_path = buf
            .parent()
            .and_then(|f| Some(f.to_str().unwrap()))
            .unwrap_or("");

        self.thumbnails
            .iter()
            .map(|f| {
                MoonrakerFileThumbnail {
                    width: f.width,
                    height: f.height,
                    size: f.size,
                    thumbnail_path: if base_path.is_empty() {
                        f.thumbnail_path.clone()
                    } else {
                        format!("{}/{}", base_path, f.thumbnail_path)
                    }
                }
            })
            .collect()
    }
}