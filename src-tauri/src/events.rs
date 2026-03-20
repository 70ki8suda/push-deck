use crate::app_state::RuntimeState;
use crate::display::DisplayFrame;
use serde::{Deserialize, Serialize};
use tauri::{Emitter, Runtime};

pub const RUNTIME_EVENT_NAME: &str = "push-deck:runtime-event";

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RuntimeEvent {
    StateChanged { state: RuntimeState },
    DeviceConnectionChanged {
        connected: bool,
        device_name: Option<String>,
    },
    PadPressed { pad_id: String },
    DisplayFrame { frame: DisplayFrame },
}

pub fn emit_runtime_event<R, E>(emitter: &E, event: RuntimeEvent) -> tauri::Result<()>
where
    R: Runtime,
    E: Emitter<R>,
{
    emitter.emit(RUNTIME_EVENT_NAME, event)
}
