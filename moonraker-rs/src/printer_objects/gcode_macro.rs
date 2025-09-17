use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct GcodeMacro {
    // No fields yet; to be implemented later
}

impl Default for GcodeMacro {
    fn default() -> Self {
        Self {}
    }
}

#[derive(Debug, Deserialize, Default)]
pub struct NamedGcodeMacro {
    pub name: String,
    pub macro_obj: GcodeMacro,
}
