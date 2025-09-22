use optional_struct::*;
use serde::Deserialize;

#[derive(Debug, Deserialize, Default, Clone, PartialEq)]
pub struct ExcludeObjectDefinition {
    pub name: String,
    pub polygon: Vec<[f32; 2]>,
    pub center: [f32; 2],
}

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct ExcludeObject {
    pub objects: Vec<ExcludeObjectDefinition>,
    pub excluded_objects: Vec<String>,
    pub current_object: Option<String>,
}

impl ExcludeObject {
    pub fn overlay(&mut self, exclude_object: OptionalExcludeObject) {
        if let Some(objects) = exclude_object.objects {
            self.objects = objects;
        }
        if let Some(excluded_objects) = exclude_object.excluded_objects {
            self.excluded_objects = excluded_objects;
        }
        if let Some(current_object) = exclude_object.current_object {
            self.current_object = Some(current_object);
        }
    }
}
