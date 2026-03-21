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
    AccessibilityPermissionUnavailable,
    NoFrontmostTarget,
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
            Self::AccessibilityPermissionUnavailable => {
                f.write_str("accessibility permission is unavailable")
            }
            Self::NoFrontmostTarget => f.write_str("no frontmost app"),
            Self::Backend { message } => f.write_str(message),
        }
    }
}

impl Error for MacosError {}

pub trait ActionBackend {
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError>;
    fn shortcut_accessibility_available(&self) -> Result<bool, MacosError> {
        Err(MacosError::UnsupportedAction)
    }
    fn frontmost_target(&self) -> Result<Option<String>, MacosError> {
        Err(MacosError::UnsupportedAction)
    }
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

    fn shortcut_accessibility_available(&self) -> Result<bool, MacosError> {
        let script = r#"tell application "System Events" to get name of every application process"#;
        let output = Self::run_osascript(script)?;

        if output.status.success() {
            Ok(true)
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Not authorized")
                || stderr.contains("not authorized")
                || stderr.contains("Accessibility")
            {
                Ok(false)
            } else {
                Err(MacosError::Backend {
                    message: stderr.trim().to_string(),
                })
            }
        }
    }

    fn frontmost_target(&self) -> Result<Option<String>, MacosError> {
        let script = r#"tell application "System Events" to get name of first application process whose frontmost is true"#;
        let output = Self::run_osascript(script)?;

        if output.status.success() {
            let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if stdout.is_empty() {
                Ok(None)
            } else {
                Ok(Some(stdout))
            }
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Can’t get")
                || stderr.contains("Can't get")
                || stderr.contains("No application process")
            {
                Ok(None)
            } else {
                Err(MacosError::Backend {
                    message: stderr.trim().to_string(),
                })
            }
        }
    }

    fn send_shortcut(
        &self,
        key: ShortcutKey,
        modifiers: &[ShortcutModifier],
    ) -> Result<(), MacosError> {
        let key_expression = match key {
            ShortcutKey::A => r#"keystroke "a""#,
            ShortcutKey::B => r#"keystroke "b""#,
            ShortcutKey::C => r#"keystroke "c""#,
            ShortcutKey::D => r#"keystroke "d""#,
            ShortcutKey::E => r#"keystroke "e""#,
            ShortcutKey::F => r#"keystroke "f""#,
            ShortcutKey::G => r#"keystroke "g""#,
            ShortcutKey::H => r#"keystroke "h""#,
            ShortcutKey::I => r#"keystroke "i""#,
            ShortcutKey::J => r#"keystroke "j""#,
            ShortcutKey::K => r#"keystroke "k""#,
            ShortcutKey::L => r#"keystroke "l""#,
            ShortcutKey::M => r#"keystroke "m""#,
            ShortcutKey::N => r#"keystroke "n""#,
            ShortcutKey::O => r#"keystroke "o""#,
            ShortcutKey::P => r#"keystroke "p""#,
            ShortcutKey::Q => r#"keystroke "q""#,
            ShortcutKey::R => r#"keystroke "r""#,
            ShortcutKey::S => r#"keystroke "s""#,
            ShortcutKey::T => r#"keystroke "t""#,
            ShortcutKey::U => r#"keystroke "u""#,
            ShortcutKey::V => r#"keystroke "v""#,
            ShortcutKey::W => r#"keystroke "w""#,
            ShortcutKey::X => r#"keystroke "x""#,
            ShortcutKey::Y => r#"keystroke "y""#,
            ShortcutKey::Z => r#"keystroke "z""#,
            ShortcutKey::Num0 => r#"keystroke "0""#,
            ShortcutKey::Num1 => r#"keystroke "1""#,
            ShortcutKey::Num2 => r#"keystroke "2""#,
            ShortcutKey::Num3 => r#"keystroke "3""#,
            ShortcutKey::Num4 => r#"keystroke "4""#,
            ShortcutKey::Num5 => r#"keystroke "5""#,
            ShortcutKey::Num6 => r#"keystroke "6""#,
            ShortcutKey::Num7 => r#"keystroke "7""#,
            ShortcutKey::Num8 => r#"keystroke "8""#,
            ShortcutKey::Num9 => r#"keystroke "9""#,
            ShortcutKey::F1 => "key code 122",
            ShortcutKey::F2 => "key code 120",
            ShortcutKey::F3 => "key code 99",
            ShortcutKey::F4 => "key code 118",
            ShortcutKey::F5 => "key code 96",
            ShortcutKey::F6 => "key code 97",
            ShortcutKey::F7 => "key code 98",
            ShortcutKey::F8 => "key code 100",
            ShortcutKey::F9 => "key code 101",
            ShortcutKey::F10 => "key code 109",
            ShortcutKey::F11 => "key code 103",
            ShortcutKey::F12 => "key code 111",
            ShortcutKey::ArrowUp => "key code 126",
            ShortcutKey::ArrowDown => "key code 125",
            ShortcutKey::ArrowLeft => "key code 123",
            ShortcutKey::ArrowRight => "key code 124",
            ShortcutKey::Space => r#"keystroke space"#,
            ShortcutKey::Tab => "key code 48",
            ShortcutKey::Enter => "key code 36",
            ShortcutKey::Escape => "key code 53",
            ShortcutKey::Delete => "key code 117",
        };

        let modifiers = if modifiers.is_empty() {
            String::new()
        } else {
            let modifier_list = modifiers
                .iter()
                .map(|modifier| match modifier {
                    ShortcutModifier::Cmd => "command down",
                    ShortcutModifier::Shift => "shift down",
                    ShortcutModifier::Opt => "option down",
                    ShortcutModifier::Ctrl => "control down",
                })
                .collect::<Vec<_>>()
                .join(", ");
            format!(" using {{{modifier_list}}}")
        };

        let script = format!(
            "tell application \"System Events\" to {}{}",
            key_expression, modifiers
        );
        let output = Self::run_osascript(&script)?;

        if output.status.success() {
            Ok(())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("Not authorized") || stderr.contains("not authorized") {
                Err(MacosError::AccessibilityPermissionUnavailable)
            } else {
                Err(MacosError::Backend {
                    message: stderr.trim().to_string(),
                })
            }
        }
    }
}

#[cfg(not(target_os = "macos"))]
impl ActionBackend for SystemMacosBackend {
    fn launch_or_focus_bundle_id(&self, _bundle_id: &str) -> Result<(), MacosError> {
        Err(MacosError::PlatformUnavailable)
    }

    fn shortcut_accessibility_available(&self) -> Result<bool, MacosError> {
        Err(MacosError::PlatformUnavailable)
    }

    fn frontmost_target(&self) -> Result<Option<String>, MacosError> {
        Err(MacosError::PlatformUnavailable)
    }
}
