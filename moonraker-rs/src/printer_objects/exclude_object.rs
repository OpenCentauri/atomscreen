use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct ExcludeObjectDefinition {
    pub name: String,
    pub polygon: Vec<[f32; 2]>,
    pub center: [f32; 2],
}


#[derive(Debug, Deserialize, Default)]
pub struct ExcludeObject {
    pub objects: Vec<ExcludeObjectDefinition>,
    pub excluded_objects: Vec<String>,
    pub current_object: Option<String>,
}
