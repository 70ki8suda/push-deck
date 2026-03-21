pub mod launch_or_focus;
pub mod send_shortcut;

use crate::config::schema::PadAction;
use crate::macos::ActionBackend;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub use launch_or_focus::{launch_or_focus_app, LaunchOrFocusError};
pub use send_shortcut::{send_shortcut_action, SendShortcutError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionExecutionError {
    LaunchOrFocus(LaunchOrFocusError),
    SendShortcut(SendShortcutError),
    Macos(crate::macos::MacosError),
}

impl Display for ActionExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LaunchOrFocus(error) => Display::fmt(error, f),
            Self::SendShortcut(error) => Display::fmt(error, f),
            Self::Macos(error) => Display::fmt(error, f),
        }
    }
}

impl Error for ActionExecutionError {}

pub fn dispatch_pad_action<B>(backend: &B, action: &PadAction) -> Result<(), ActionExecutionError>
where
    B: ActionBackend,
{
    match action {
        PadAction::LaunchOrFocusApp { .. } => {
            launch_or_focus_app(backend, action).map_err(ActionExecutionError::LaunchOrFocus)
        }
        PadAction::Unassigned => Ok(()),
        PadAction::SendShortcut { .. } => {
            send_shortcut_action(backend, action).map_err(ActionExecutionError::SendShortcut)
        }
    }
}
