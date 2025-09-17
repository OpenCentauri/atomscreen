use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct Fan {
    pub speed: f32,
    pub rpm: Option<i32>,
}

