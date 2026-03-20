use crate::app_state::RuntimeState;
use crate::display::DisplayFrame;
use serde::{Deserialize, Serialize};

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
