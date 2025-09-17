use serde::Deserialize;

#[derive(Debug, Deserialize, Default)]
pub struct OutputPin {
    pub value: f32,
}

#[derive(Debug, Deserialize, Default)]
pub struct NamedOutputPin {
    pub name: String,
    pub pin: OutputPin,
}
