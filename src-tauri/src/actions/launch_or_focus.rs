use crate::config::schema::PadAction;
use crate::macos::{ActionBackend, MacosError};
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
    B: ActionBackend,
{
    let PadAction::LaunchOrFocusApp {
        bundle_id,
        app_name,
    } = action
    else {
        return Ok(());
    };

    if bundle_id.trim().is_empty() {
        return Err(LaunchOrFocusError::BundleIdUnavailable {
            app_name: app_name.clone(),
        });
    }

    backend
        .launch_or_focus_bundle_id(bundle_id)
        .map_err(|error| match error {
            MacosError::AppNotFound { bundle_id } => LaunchOrFocusError::AppNotFound { bundle_id },
            other => LaunchOrFocusError::Macos(other),
        })
}
