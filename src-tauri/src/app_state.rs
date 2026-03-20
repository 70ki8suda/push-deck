use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AppState {
    Starting,
    WaitingForDevice,
    Ready,
    ConfigRecoveryRequired,
    SaveFailed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShortcutCapabilityState {
    Available,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    pub shortcut: ShortcutCapabilityState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RuntimeState {
    pub app_state: AppState,
    pub capabilities: RuntimeCapabilities,
}

impl RuntimeState {
    pub fn new(app_state: AppState, shortcut: ShortcutCapabilityState) -> Self {
        Self {
            app_state,
            capabilities: RuntimeCapabilities { shortcut },
        }
    }
}
