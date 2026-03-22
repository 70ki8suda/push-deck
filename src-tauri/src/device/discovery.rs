use crate::app_state::{AppState, DeviceConnectionState, DeviceEndpointDescriptor, RuntimeState};
use crate::events::{emit_runtime_event, RuntimeEvent};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use tauri::{Emitter, Runtime};

#[cfg(target_os = "macos")]
use coremidi::{Destinations, Endpoint, Sources};
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

#[derive(Debug, Default, Clone, Copy)]
pub struct CoreMidiDiscoverySource;

pub struct StartupDiscoverySource<P, S> {
    primary: P,
    fallback: S,
}

impl DeviceDiscoveryBackendError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl SystemDiscoverySource {
    pub fn from_system_profiler_json(
        payload: &str,
    ) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        let value: Value = serde_json::from_str(payload).map_err(|error| {
            DeviceDiscoveryError::backend(format!(
                "failed to parse system_profiler usb json: {error}"
            ))
        })?;

        let mut endpoints = Vec::new();
        collect_system_profiler_devices(&value, &mut endpoints);
        dedupe_endpoints(&mut endpoints);
        Ok(endpoints)
    }
}

impl<P, S> StartupDiscoverySource<P, S> {
    pub fn new(primary: P, fallback: S) -> Self {
        Self { primary, fallback }
    }
}

#[cfg(target_os = "macos")]
impl DeviceDiscoverySource for SystemDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        let output = Command::new("system_profiler")
            .args(["SPUSBDataType", "-json"])
            .output()
            .map_err(|error| {
                DeviceDiscoveryError::backend(format!(
                    "failed to run system_profiler SPUSBDataType -json: {error}"
                ))
            })?;

        if !output.status.success() {
            return Err(DeviceDiscoveryError::backend(
                String::from_utf8_lossy(&output.stderr).trim().to_string(),
            ));
        }

        Self::from_system_profiler_json(&String::from_utf8_lossy(&output.stdout))
    }
}

#[cfg(not(target_os = "macos"))]
impl DeviceDiscoverySource for SystemDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        Ok(vec![])
    }
}

#[cfg(target_os = "macos")]
impl DeviceDiscoverySource for CoreMidiDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        let mut endpoints = Vec::new();
        collect_coremidi_endpoints(&mut endpoints);
        dedupe_endpoints(&mut endpoints);
        Ok(endpoints)
    }
}

#[cfg(not(target_os = "macos"))]
impl DeviceDiscoverySource for CoreMidiDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        Ok(vec![])
    }
}

impl<P, S> DeviceDiscoverySource for StartupDiscoverySource<P, S>
where
    P: DeviceDiscoverySource,
    S: DeviceDiscoverySource,
{
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        match self.primary.discover_devices() {
            Ok(devices) if !devices.is_empty() => Ok(devices),
            Ok(_) | Err(_) => self.fallback.discover_devices(),
        }
    }
}

fn collect_system_profiler_devices(value: &Value, endpoints: &mut Vec<DeviceEndpointDescriptor>) {
    match value {
        Value::Array(items) => {
            for item in items {
                collect_system_profiler_devices(item, endpoints);
            }
        }
        Value::Object(map) => {
            if let Some(name) = map.get("_name").and_then(Value::as_str) {
                let normalized = name.to_ascii_lowercase();
                if normalized.contains("push 3") || normalized == "ableton push 3" {
                    let endpoint_id = map
                        .get("serial_num")
                        .or_else(|| map.get("location_id"))
                        .or_else(|| map.get("product_id"))
                        .and_then(Value::as_str)
                        .unwrap_or(name);
                    endpoints.push(DeviceEndpointDescriptor::push_3(
                        endpoint_id.to_string(),
                        name.to_string(),
                    ));
                }
            }

            for nested in map.values() {
                collect_system_profiler_devices(nested, endpoints);
            }
        }
        _ => {}
    }
}

fn dedupe_endpoints(endpoints: &mut Vec<DeviceEndpointDescriptor>) {
    let mut seen = HashSet::new();
    endpoints.retain(|candidate| seen.insert(candidate.endpoint_id.clone()));
}

#[cfg(target_os = "macos")]
fn collect_coremidi_endpoints(endpoints: &mut Vec<DeviceEndpointDescriptor>) {
    collect_coremidi_sources(endpoints);
    collect_coremidi_destinations(endpoints);
}

#[cfg(target_os = "macos")]
fn collect_coremidi_sources(endpoints: &mut Vec<DeviceEndpointDescriptor>) {
    for source in Sources {
        collect_coremidi_endpoint(&source, endpoints);
    }
}

#[cfg(target_os = "macos")]
fn collect_coremidi_destinations(endpoints: &mut Vec<DeviceEndpointDescriptor>) {
    for destination in Destinations {
        collect_coremidi_endpoint(&destination, endpoints);
    }
}

#[cfg(target_os = "macos")]
fn collect_coremidi_endpoint(endpoint: &Endpoint, endpoints: &mut Vec<DeviceEndpointDescriptor>) {
    let Some(display_name) = endpoint.display_name().or_else(|| endpoint.name()) else {
        return;
    };

    if !is_push_3_display_name(&display_name) {
        return;
    }

    let endpoint_id = endpoint
        .unique_id()
        .map(|unique_id| unique_id.to_string())
        .unwrap_or_else(|| display_name.clone());

    endpoints.push(DeviceEndpointDescriptor::push_3(endpoint_id, display_name));
}

fn is_push_3_display_name(display_name: &str) -> bool {
    display_name.to_ascii_lowercase().contains("push 3")
}

fn is_user_port_display_name(display_name: &str) -> bool {
    display_name.to_ascii_lowercase().contains("user port")
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
        .enumerate()
        .filter(|(_, candidate)| candidate.is_push_3)
        .min_by_key(|(index, candidate)| {
            (!is_user_port_display_name(&candidate.display_name), *index)
        });

    match active_device {
        Some((_, endpoint)) => DeviceDiscoveryResult {
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
