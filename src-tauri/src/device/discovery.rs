use crate::app_state::{AppState, DeviceConnectionState, ShortcutCapabilityState, RuntimeState};
use crate::events::{emit_runtime_event, RuntimeEvent};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCandidate {
    pub name: String,
    pub is_push_3: bool,
}

impl DeviceCandidate {
    pub fn push_3(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_push_3: true,
        }
    }

    pub fn other(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            is_push_3: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceDiscoveryResult {
    pub app_state: AppState,
    pub device_state: DeviceConnectionState,
    pub active_device: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DeviceDiscoveryError;

pub trait DeviceDiscoverySource {
    fn discover_devices(&self) -> Vec<DeviceCandidate>;
}

#[derive(Debug)]
pub struct PushDeviceService<S> {
    source: S,
    active_device: Option<String>,
}

impl<S> PushDeviceService<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            active_device: None,
        }
    }

    pub fn active_device(&self) -> Option<&str> {
        self.active_device.as_deref()
    }
}

impl<S> PushDeviceService<S>
where
    S: DeviceDiscoverySource,
{
    pub fn discover(&mut self) -> Result<DeviceDiscoveryResult, DeviceDiscoveryError> {
        let result = discover_push_device(self.source.discover_devices());
        self.active_device = result.active_device.clone();
        Ok(result)
    }
}

pub fn discover_push_device(candidates: Vec<DeviceCandidate>) -> DeviceDiscoveryResult {
    let active_device = candidates
        .into_iter()
        .find(|candidate| candidate.is_push_3)
        .map(|candidate| candidate.name);

    match active_device {
        Some(device_name) => DeviceDiscoveryResult {
            app_state: AppState::Ready,
            device_state: DeviceConnectionState::Connected {
                device_name: device_name.clone(),
            },
            active_device: Some(device_name),
        },
        None => DeviceDiscoveryResult {
            app_state: AppState::WaitingForDevice,
            device_state: DeviceConnectionState::WaitingForDevice,
            active_device: None,
        },
    }
}

pub fn emit_discovery_state<R, E>(
    emitter: &E,
    result: &DeviceDiscoveryResult,
) -> tauri::Result<()>
where
    R: Runtime,
    E: Emitter<R>,
{
    emit_runtime_event(
        emitter,
        RuntimeEvent::StateChanged {
            state: RuntimeState::new(result.app_state, ShortcutCapabilityState::Unavailable),
        },
    )?;

    emit_runtime_event(
        emitter,
        RuntimeEvent::DeviceConnectionChanged {
            connected: matches!(result.device_state, DeviceConnectionState::Connected { .. }),
            device_name: result.active_device.clone(),
        },
    )?;

    Ok(())
}
