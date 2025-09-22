use optional_struct::*;
use serde::Deserialize;

#[optional_struct]
#[derive(Debug, Deserialize, Default, Clone)]
pub struct OutputPin {
    pub value: f32,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedOutputPin {
    pub name: String,
    pub pin: OutputPin,
}

#[derive(Debug, Deserialize, Default, Clone)]
pub struct NamedOptionalOutputPin {
    pub name: String,
    pub pin: OptionalOutputPin,
}

impl OutputPin {
    pub fn overlay(&mut self, pin: OptionalOutputPin) {
        if let Some(value) = pin.value {
            self.value = value;
        }
    }
}
