use std::error::Error;
use std::fmt::{Display, Formatter};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacosError {
    PlatformUnavailable,
    AppNotFound { bundle_id: String },
    Backend { message: String },
}

impl Display for MacosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlatformUnavailable => {
                f.write_str("macOS integration is unavailable on this platform")
            }
            Self::AppNotFound { bundle_id } => write!(f, "app not found: {bundle_id}"),
            Self::Backend { message } => f.write_str(message),
        }
    }
}

impl Error for MacosError {}

pub trait LaunchOrFocusBackend {
    fn resolve_bundle_id(&self, app_name: &str) -> Result<Option<String>, MacosError>;
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError>;
}

#[derive(Debug, Default)]
pub struct SystemMacosBackend;

#[cfg(target_os = "macos")]
impl SystemMacosBackend {
    fn run_osascript(script: &str) -> Result<std::process::Output, MacosError> {
        Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|error| MacosError::Backend {
                message: format!("failed to run osascript: {error}"),
            })
    }

    fn escape_applescript_string(value: &str) -> String {
        value.replace('\\', "\\\\").replace('\"', "\\\"")
    }
}

#[cfg(target_os = "macos")]
impl LaunchOrFocusBackend for SystemMacosBackend {
    fn resolve_bundle_id(&self, app_name: &str) -> Result<Option<String>, MacosError> {
        let script = format!(
            "id of application \"{}\"",
            Self::escape_applescript_string(app_name)
        );
        let output = Self::run_osascript(&script)?;

        if output.status.success() {
            let bundle_id =
                String::from_utf8(output.stdout).map_err(|error| MacosError::Backend {
                    message: format!("invalid bundle id output: {error}"),
                })?;
            let bundle_id = bundle_id.trim().to_string();

            if bundle_id.is_empty() {
                Ok(None)
            } else {
                Ok(Some(bundle_id))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Can’t get application id")
                || stderr.contains("Can't get application id")
                || stderr.contains("application id")
            {
                Ok(None)
            } else {
                Err(MacosError::Backend {
                    message: stderr.trim().to_string(),
                })
            }
        }
    }

    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError> {
        let script = format!(
            "tell application id \"{}\" to activate",
            Self::escape_applescript_string(bundle_id)
        );
        let output = Self::run_osascript(&script)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Can’t get application id")
                || stderr.contains("Can't get application id")
                || stderr.contains("not found")
                || stderr.contains("No application")
            {
                Err(MacosError::AppNotFound {
                    bundle_id: bundle_id.to_string(),
                })
            } else {
                Err(MacosError::Backend {
                    message: stderr.trim().to_string(),
                })
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl LaunchOrFocusBackend for SystemMacosBackend {
    fn resolve_bundle_id(&self, _app_name: &str) -> Result<Option<String>, MacosError> {
        Err(MacosError::PlatformUnavailable)
    }

    fn launch_or_focus_bundle_id(&self, _bundle_id: &str) -> Result<(), MacosError> {
        Err(MacosError::PlatformUnavailable)
    }
}
