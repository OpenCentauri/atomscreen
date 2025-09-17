use serde::Deserialize;
use optional_struct::*;

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum KlippyState
{
    Ready,
    Startup,
    Error,
    Shutdown,
}

impl Default for KlippyState {
    fn default() -> Self {
        KlippyState::Ready
    }
}

#[optional_struct]
#[derive(Debug, Deserialize, Clone)]
pub struct Webhooks
{
    pub state: KlippyState,
    pub state_message: String,
}

impl Default for Webhooks {
    fn default() -> Self {
        Self {
            state: KlippyState::default(),
            state_message: String::new(),
        }
    }
}

impl Webhooks {
    pub fn overlay(&mut self, webhooks: OptionalWebhooks) {
        if let Some(state) = webhooks.state {
            self.state = state;
        }
        if let Some(state_message) = webhooks.state_message {
            self.state_message = state_message;
        }
    }
}
