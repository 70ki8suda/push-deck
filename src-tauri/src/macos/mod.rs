use std::error::Error;
use std::fmt::{Display, Formatter};

use crate::config::schema::{ShortcutKey, ShortcutModifier};

#[cfg(target_os = "macos")]
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MacosError {
    PlatformUnavailable,
    UnsupportedAction,
    AppNotFound { bundle_id: String },
    Backend { message: String },
}

impl Display for MacosError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::PlatformUnavailable => {
                f.write_str("macOS integration is unavailable on this platform")
            }
            Self::UnsupportedAction => f.write_str("action is not supported by this backend"),
            Self::AppNotFound { bundle_id } => write!(f, "app not found: {bundle_id}"),
            Self::Backend { message } => f.write_str(message),
        }
    }
}

impl Error for MacosError {}

pub trait ActionBackend {
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError>;
    fn send_shortcut(
        &self,
        _key: ShortcutKey,
        _modifiers: &[ShortcutModifier],
    ) -> Result<(), MacosError> {
        Err(MacosError::UnsupportedAction)
    }
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

    fn known_app_not_found(stderr: &str) -> bool {
        stderr.contains("Can’t get application id")
            || stderr.contains("Can't get application id")
            || stderr.contains("Application can't be found")
            || stderr.contains("Application can’t be found")
    }
}

#[cfg(target_os = "macos")]
impl ActionBackend for SystemMacosBackend {
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
            if Self::known_app_not_found(&stderr) {
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

    fn send_shortcut(
        &self,
        _key: ShortcutKey,
        _modifiers: &[ShortcutModifier],
    ) -> Result<(), MacosError> {
        Err(MacosError::UnsupportedAction)
    }
}

#[cfg(not(target_os = "macos"))]
impl ActionBackend for SystemMacosBackend {
    fn launch_or_focus_bundle_id(&self, _bundle_id: &str) -> Result<(), MacosError> {
        Err(MacosError::PlatformUnavailable)
    }
}
