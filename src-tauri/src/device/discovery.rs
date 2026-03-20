use crate::app_state::{AppState, DeviceConnectionState, DeviceEndpointDescriptor, RuntimeState};
use crate::events::{emit_runtime_event, RuntimeEvent};
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceDiscoveryBackendError {
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceDiscoveryError {
    Backend(DeviceDiscoveryBackendError),
}

impl DeviceDiscoveryError {
    pub fn backend(message: impl Into<String>) -> Self {
        Self::Backend(DeviceDiscoveryBackendError::new(message))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceDiscoveryResult {
    pub app_state: AppState,
    pub connection: DeviceConnectionState,
}

pub trait DeviceDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError>;
}

impl DeviceDiscoveryBackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[derive(Debug)]
pub struct PushDeviceService<S> {
    source: S,
    active_endpoint: Option<DeviceEndpointDescriptor>,
}

impl<S> PushDeviceService<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            active_endpoint: None,
        }
    }

    pub fn active_endpoint(&self) -> Option<&DeviceEndpointDescriptor> {
        self.active_endpoint.as_ref()
    }
}

impl<S> PushDeviceService<S>
where
    S: DeviceDiscoverySource,
{
    pub fn discover(&mut self) -> Result<DeviceDiscoveryResult, DeviceDiscoveryError> {
        let result = discover_push_device(self.source.discover_devices()?);
        self.active_endpoint = result.connection.endpoint().cloned();
        Ok(result)
    }
}

pub fn discover_push_device(candidates: Vec<DeviceEndpointDescriptor>) -> DeviceDiscoveryResult {
    let active_device = candidates
        .into_iter()
        .find(|candidate| candidate.is_push_3);

    match active_device {
        Some(endpoint) => DeviceDiscoveryResult {
            app_state: AppState::Ready,
            connection: DeviceConnectionState::Connected { endpoint },
        },
        None => DeviceDiscoveryResult {
            app_state: AppState::WaitingForDevice,
            connection: DeviceConnectionState::WaitingForDevice,
        },
    }
}

pub fn emit_discovery_state<R, E>(
    emitter: &E,
    result: &DeviceDiscoveryResult,
    runtime_state: &RuntimeState,
) -> tauri::Result<()>
where
    R: Runtime,
    E: Emitter<R>,
{
    emit_runtime_event(
        emitter,
        RuntimeEvent::StateChanged {
            state: runtime_state.with_app_state(result.app_state),
        },
    )?;

    emit_runtime_event(
        emitter,
        RuntimeEvent::DeviceConnectionChanged {
            connected: matches!(result.connection, DeviceConnectionState::Connected { .. }),
            device_name: result
                .connection
                .endpoint()
                .map(|endpoint| endpoint.display_name.clone()),
        },
    )?;

    Ok(())
}
