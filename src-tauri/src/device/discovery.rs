use crate::app_state::{AppState, DeviceConnectionState, DeviceEndpointDescriptor, RuntimeState};
use crate::events::{emit_runtime_event, RuntimeEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{Emitter, Runtime};

#[cfg(target_os = "macos")]
use std::process::Command;

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

#[derive(Debug, Default, Clone, Copy)]
pub struct SystemDiscoverySource;

impl DeviceDiscoveryBackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

#[cfg(target_os = "macos")]
impl DeviceDiscoverySource for SystemDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        let output = Command::new("system_profiler")
            .args(["SPUSBDataType", "-json"])
            .output()
            .map_err(|error| {
                DeviceDiscoveryError::backend(format!("failed to run system_profiler: {error}"))
            })?;

        if !output.status.success() {
            return Err(DeviceDiscoveryError::backend(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        let payload: Value = serde_json::from_slice(&output.stdout).map_err(|error| {
            DeviceDiscoveryError::backend(format!("failed to parse system_profiler output: {error}"))
        })?;
        let mut endpoints = Vec::new();
        collect_push_three_usb_devices(&payload, &mut endpoints);
        dedupe_endpoints(endpoints)
    }
}

#[cfg(not(target_os = "macos"))]
impl DeviceDiscoverySource for SystemDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        Ok(vec![])
    }
}

fn collect_push_three_usb_devices(value: &Value, endpoints: &mut Vec<DeviceEndpointDescriptor>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_push_three_usb_devices(item, endpoints);
            }
        }
        Value::Object(map) => {
            if let Some(name) = map.get("_name").and_then(Value::as_str) {
                let normalized_name = name.to_ascii_lowercase();
                if normalized_name.contains("push 3") || normalized_name.contains("ableton push")
                {
                    let endpoint_id = map
                        .get("serial_num")
                        .or_else(|| map.get("location_id"))
                        .or_else(|| map.get("manufacturer"))
                        .and_then(Value::as_str)
                        .unwrap_or(name);
                    endpoints.push(DeviceEndpointDescriptor::push_3(
                        endpoint_id.to_string(),
                        name.to_string(),
                    ));
                }
            }

            for nested in map.values() {
                collect_push_three_usb_devices(nested, endpoints);
            }
        }
        _ => {}
    }
}

fn dedupe_endpoints(
    endpoints: Vec<DeviceEndpointDescriptor>,
) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
    let mut unique = Vec::new();
    for endpoint in endpoints {
        if unique
            .iter()
            .all(|candidate: &DeviceEndpointDescriptor| candidate.endpoint_id != endpoint.endpoint_id)
        {
            unique.push(endpoint);
        }
    }
    Ok(unique)
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
