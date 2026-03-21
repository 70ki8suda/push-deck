use crate::config::schema::PadAction;
use crate::macos::{LaunchOrFocusBackend, MacosError};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchOrFocusError {
    BundleIdUnavailable { app_name: String },
    AppNotFound { bundle_id: String },
    Macos(MacosError),
}

impl Display for LaunchOrFocusError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BundleIdUnavailable { app_name } => {
                write!(f, "bundle id could not be resolved for app: {app_name}")
            }
            Self::AppNotFound { bundle_id } => write!(f, "app not found: {bundle_id}"),
            Self::Macos(error) => Display::fmt(error, f),
        }
    }
}

impl Error for LaunchOrFocusError {}

pub fn launch_or_focus_app<B>(backend: &B, action: &PadAction) -> Result<(), LaunchOrFocusError>
where
    B: LaunchOrFocusBackend,
{
    let PadAction::LaunchOrFocusApp {
        bundle_id,
        app_name,
    } = action
    else {
        return Ok(());
    };

    let resolved_bundle_id = if bundle_id.trim().is_empty() {
        backend
            .resolve_bundle_id(app_name)
            .map_err(LaunchOrFocusError::Macos)?
            .ok_or_else(|| LaunchOrFocusError::BundleIdUnavailable {
                app_name: app_name.clone(),
            })?
    } else {
        bundle_id.clone()
    };

    backend
        .launch_or_focus_bundle_id(&resolved_bundle_id)
        .map_err(|error| match error {
            MacosError::AppNotFound { bundle_id } => LaunchOrFocusError::AppNotFound { bundle_id },
            other => LaunchOrFocusError::Macos(other),
        })
}
