use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MoonrakerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for MoonrakerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".into(),
            port: 7125u16,
        }
    }
}
