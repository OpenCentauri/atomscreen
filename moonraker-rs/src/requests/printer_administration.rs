use serde::Deserialize;
use std::fmt::Debug;

use crate::{
    connector::read_deserialize::MoonrakerEventNotifyStatusUpdate, error::Error,
    moonraker_connection::MoonrakerConnection,
};

pub trait PrinterAdministrationRequestHandler {
    async fn list_printer_objects(&self) -> Result<PrinterObjectListResponse, Error>;
    async fn subscribe_to_printer_objects(
        &self,
        objects: Vec<String>,
    ) -> Result<PrinterObjectsSubscribeResult, Error>;
    async fn run_gcode_script(&self, script: &str) -> Result<String, Error>;
    async fn emergency_stop(&self) -> Result<String, Error>;
    async fn restart(&self) -> Result<String, Error>;
    async fn firmware_restart(&self) -> Result<String, Error>;
}

impl PrinterAdministrationRequestHandler for MoonrakerConnection {
    async fn list_printer_objects(&self) -> Result<PrinterObjectListResponse, Error> {
        self.send_request("printer.objects.list", None).await
    }

    async fn subscribe_to_printer_objects(
        &self,
        objects: Vec<String>,
    ) -> Result<PrinterObjectsSubscribeResult, Error> {
        let mut map = serde_json::Map::new();
        for object in objects {
            map.insert(object, serde_json::Value::Null);
        }

        let args = serde_json::json!({
            "objects": map,
        });

        self.send_request("printer.objects.subscribe", Some(args))
            .await
    }
    
    async fn run_gcode_script(&self, script: &str) -> Result<String, Error> {
        let args = serde_json::json!({ "script": script });
        self.send_request("printer.gcode.script", Some(args)).await
    }

    async fn emergency_stop(&self) -> Result<String, Error> {
        self.send_request("printer.emergency_stop", None).await
    }

    async fn restart(&self) -> Result<String, Error> {
        self.send_request("printer.restart", None).await
    }

    async fn firmware_restart(&self) -> Result<String, Error> {
        self.send_request("printer.firmware_restart", None).await
    }
}

#[derive(Debug, Deserialize)]
pub struct PrinterObjectsSubscribeResult {
    pub eventtime: f32,
    pub status: MoonrakerEventNotifyStatusUpdate,
}

#[derive(Debug, Deserialize)]
pub struct PrinterObjectListResponse {
    pub objects: Vec<String>,
}
