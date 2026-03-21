pub mod launch_or_focus;

use crate::config::schema::PadAction;
use crate::macos::ActionBackend;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub use launch_or_focus::{launch_or_focus_app, LaunchOrFocusError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActionExecutionError {
    LaunchOrFocus(LaunchOrFocusError),
    Macos(crate::macos::MacosError),
}

impl Display for ActionExecutionError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::LaunchOrFocus(error) => Display::fmt(error, f),
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
        PadAction::SendShortcut { key, modifiers } => backend
            .send_shortcut(*key, modifiers)
            .map_err(ActionExecutionError::Macos),
    }
}
