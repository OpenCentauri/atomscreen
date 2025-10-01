use optional_struct::optional_struct;
use serde::Deserialize;

#[optional_struct]
#[derive(Deserialize, Debug)]
pub struct GcodeCommands 
{
    pub extruder_extrude: String,
    pub extruder_retract: String,
    pub extruder_load_filament: String,
    pub extruder_unload_filament: String,
}

impl Default for GcodeCommands {
    fn default() -> Self {
        Self {
            extruder_extrude: "M83\nG1 E25 F300".into(),
            extruder_retract: "M83\nG1 E-25 F300".into(),
            extruder_load_filament: "LOAD_FILAMENT".into(),
            extruder_unload_filament: "UNLOAD_FILAMENT".into(),
        }
    }
}