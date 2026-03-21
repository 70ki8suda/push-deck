use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

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
pub struct ConfigRecoveryState {
    pub config_path: PathBuf,
    pub backup_path: PathBuf,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceEndpointDescriptor {
    pub endpoint_id: String,
    pub display_name: String,
    pub is_push_3: bool,
}

impl DeviceEndpointDescriptor {
    pub fn push_3(endpoint_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            display_name: display_name.into(),
            is_push_3: true,
        }
    }

    pub fn other(endpoint_id: impl Into<String>, display_name: impl Into<String>) -> Self {
        Self {
            endpoint_id: endpoint_id.into(),
            display_name: display_name.into(),
            is_push_3: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ConfigLoadState {
    Loaded,
    CreatedDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum DeviceConnectionState {
    WaitingForDevice,
    Connected { endpoint: DeviceEndpointDescriptor },
}

impl DeviceConnectionState {
    pub fn endpoint(&self) -> Option<&DeviceEndpointDescriptor> {
        match self {
            Self::Connected { endpoint } => Some(endpoint),
            Self::WaitingForDevice => None,
        }
    }
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

    pub fn with_app_state(&self, app_state: AppState) -> Self {
        Self {
            app_state,
            capabilities: self.capabilities.clone(),
        }
    }

    pub fn with_shortcut_capability(&self, shortcut: ShortcutCapabilityState) -> Self {
        Self {
            app_state: self.app_state,
            capabilities: RuntimeCapabilities { shortcut },
        }
    }
}

fn shortcut_capability_store() -> &'static Mutex<ShortcutCapabilityState> {
    static STORE: OnceLock<Mutex<ShortcutCapabilityState>> = OnceLock::new();
    STORE.get_or_init(|| Mutex::new(ShortcutCapabilityState::Unavailable))
}

pub fn recorded_shortcut_capability() -> ShortcutCapabilityState {
    *shortcut_capability_store()
        .lock()
        .expect("shortcut capability store lock poisoned")
}

pub fn runtime_state_snapshot(app_state: AppState) -> RuntimeState {
    RuntimeState::new(app_state, recorded_shortcut_capability())
}

pub fn record_shortcut_capability(shortcut: ShortcutCapabilityState) -> ShortcutCapabilityState {
    let mut capability = shortcut_capability_store()
        .lock()
        .expect("shortcut capability store lock poisoned");
    *capability = shortcut;
    *capability
}
