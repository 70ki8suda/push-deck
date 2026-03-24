use crate::config::schema::PadAction;
use crate::macos::{ActionBackend, MacosError};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::thread;
use std::time::Duration;

const FOCUS_POLL_INTERVAL: Duration = Duration::from_millis(75);
const FOCUS_POLL_ATTEMPTS: usize = 6;
const COLD_LAUNCH_GRACE_POLLS: usize = 2;

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

    let was_running = backend
        .running_apps()
        .ok()
        .is_some_and(|apps| apps.iter().any(|app| app.bundle_id == *bundle_id));

    run_launch_or_focus(backend, bundle_id)?;
    stabilize_focus(backend, bundle_id, was_running)
}

fn run_launch_or_focus<B>(backend: &B, bundle_id: &str) -> Result<(), LaunchOrFocusError>
where
    B: ActionBackend,
{
    backend
        .launch_or_focus_bundle_id(bundle_id)
        .map_err(|error| match error {
            MacosError::AppNotFound { bundle_id } => LaunchOrFocusError::AppNotFound { bundle_id },
            other => LaunchOrFocusError::Macos(other),
        })
}

fn stabilize_focus<B>(
    backend: &B,
    bundle_id: &str,
    was_running: bool,
) -> Result<(), LaunchOrFocusError>
where
    B: ActionBackend,
{
    if frontmost_matches(backend, bundle_id)? {
        return Ok(());
    }

    for attempt in 0..FOCUS_POLL_ATTEMPTS {
        thread::sleep(FOCUS_POLL_INTERVAL);

        if should_retry_focus(attempt, was_running) {
            run_launch_or_focus(backend, bundle_id)?;
        }

        if frontmost_matches(backend, bundle_id)? {
            return Ok(());
        }
    }

    Ok(())
}

fn frontmost_matches<B>(backend: &B, bundle_id: &str) -> Result<bool, LaunchOrFocusError>
where
    B: ActionBackend,
{
    match backend.frontmost_target() {
        Ok(Some(frontmost)) => Ok(frontmost.bundle_id == bundle_id),
        Ok(None) => Ok(false),
        Err(MacosError::PlatformUnavailable | MacosError::UnsupportedAction) => Ok(true),
        Err(error) => {
            eprintln!("launch/focus verification unavailable: {error}");
            Ok(true)
        }
    }
}

fn should_retry_focus(attempt: usize, was_running: bool) -> bool {
    if was_running {
        attempt == 0 || attempt == 2
    } else {
        attempt == COLD_LAUNCH_GRACE_POLLS
    }
}
