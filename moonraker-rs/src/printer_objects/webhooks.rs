use serde::Deserialize;

#[derive(Debug, Deserialize)]
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

#[derive(Debug, Deserialize)]
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