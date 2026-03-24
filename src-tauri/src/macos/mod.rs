use std::error::Error;
use std::fmt::{Display, Formatter};
use crate::config::schema::{ShortcutKey, ShortcutModifier};

#[cfg(target_os = "macos")]
use objc2_app_kit::{NSApplicationActivationPolicy, NSRunningApplication, NSWorkspace};
#[cfg(target_os = "macos")]
use std::process::Command;
#[cfg(test)]
use std::collections::HashSet;

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

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunningAppOption {
    pub bundle_id: String,
    pub app_name: String,
}

pub trait ActionBackend {
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError>;
    fn running_apps(&self) -> Result<Vec<RunningAppOption>, MacosError> {
        Err(MacosError::UnsupportedAction)
    }
    fn shortcut_accessibility_available(&self) -> Result<bool, MacosError> {
        Err(MacosError::UnsupportedAction)
    }
    fn frontmost_target(&self) -> Result<Option<RunningAppOption>, MacosError> {
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

    fn launch_or_focus_script(bundle_id: &str) -> String {
        format!(
            r#"tell application id "{}"
    if running then
        reopen
    else
        launch
    end if
    activate
end tell"#,
            Self::escape_applescript_string(bundle_id)
        )
    }

    fn running_app_option(app: &NSRunningApplication) -> Option<RunningAppOption> {
        if !Self::should_include_running_app(
            app.activationPolicy(),
            app.bundleIdentifier().is_some(),
            app.isTerminated(),
        ) {
            return None;
        }

        let bundle_id = app.bundleIdentifier()?.to_string();
        let app_name = app
            .localizedName()
            .map(|name| name.to_string())
            .unwrap_or_else(|| bundle_id.clone());

        Some(RunningAppOption {
            bundle_id,
            app_name,
        })
    }

    fn should_include_running_app(
        activation_policy: NSApplicationActivationPolicy,
        has_bundle_id: bool,
        is_terminated: bool,
    ) -> bool {
        !is_terminated
            && has_bundle_id
            && activation_policy == NSApplicationActivationPolicy::Regular
    }

    #[cfg(test)]
    fn parse_running_apps(output: &str) -> Vec<RunningAppOption> {
        use std::collections::HashSet;

        let mut apps = Vec::new();
        let mut seen_bundle_ids = HashSet::new();
        let mut current_name: Option<String> = None;
        let mut current_bundle_id: Option<String> = None;
        let mut current_type: Option<String> = None;

        for line in output.lines() {
            let trimmed = line.trim();

            if let Some(name) = parse_lsappinfo_name(trimmed) {
                push_running_app(
                    &mut apps,
                    &mut seen_bundle_ids,
                    current_name.take(),
                    current_bundle_id.take(),
                    current_type.take(),
                );
                current_name = Some(name);
                continue;
            }

            if let Some(bundle_id) = trimmed.strip_prefix("bundleID=\"") {
                current_bundle_id = bundle_id.strip_suffix('\"').map(str::to_string);
                continue;
            }

            if trimmed == "bundleID=[ NULL ]" {
                current_bundle_id = None;
                continue;
            }

            if let Some(app_type) = parse_lsappinfo_type(trimmed) {
                current_type = Some(app_type);
            }
        }

        push_running_app(
            &mut apps,
            &mut seen_bundle_ids,
            current_name,
            current_bundle_id,
            current_type,
        );

        apps
    }
}

#[cfg(all(target_os = "macos", test))]
fn parse_lsappinfo_name(line: &str) -> Option<String> {
    let (_, rest) = line.split_once(") \"")?;
    let (name, _) = rest.split_once('"')?;
    Some(name.to_string())
}

#[cfg(all(target_os = "macos", test))]
fn parse_lsappinfo_type(line: &str) -> Option<String> {
    let (_, rest) = line.split_once("type=\"")?;
    let (app_type, _) = rest.split_once('"')?;
    Some(app_type.to_string())
}

#[cfg(all(target_os = "macos", test))]
fn push_running_app(
    apps: &mut Vec<RunningAppOption>,
    seen_bundle_ids: &mut HashSet<String>,
    name: Option<String>,
    bundle_id: Option<String>,
    app_type: Option<String>,
) {
    let Some(name) = name else {
        return;
    };
    let Some(bundle_id) = bundle_id else {
        return;
    };
    let Some(app_type) = app_type else {
        return;
    };

    if app_type == "BackgroundOnly" {
        return;
    }

    if seen_bundle_ids.insert(bundle_id.clone()) {
        apps.push(RunningAppOption {
            bundle_id,
            app_name: name,
        });
    }
}

#[cfg(target_os = "macos")]
impl ActionBackend for SystemMacosBackend {
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError> {
        let script = Self::launch_or_focus_script(bundle_id);
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

    fn running_apps(&self) -> Result<Vec<RunningAppOption>, MacosError> {
        let apps = NSWorkspace::sharedWorkspace()
            .runningApplications()
            .iter()
            .filter_map(|app| Self::running_app_option(&app))
            .collect();

        Ok(apps)
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

    fn frontmost_target(&self) -> Result<Option<RunningAppOption>, MacosError> {
        Ok(NSWorkspace::sharedWorkspace()
            .frontmostApplication()
            .and_then(|app| Self::running_app_option(&app)))
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

    fn frontmost_target(&self) -> Result<Option<RunningAppOption>, MacosError> {
        Err(MacosError::PlatformUnavailable)
    }
}

#[cfg(test)]
mod tests {
    #[cfg(target_os = "macos")]
    use objc2_app_kit::NSApplicationActivationPolicy;

    use super::{RunningAppOption, SystemMacosBackend};

    #[test]
    fn parse_running_apps_extracts_foreground_entries_from_lsappinfo_output() {
        let output = r#"11) "Arc" ASN:0x0-0x11011:
    bundleID="company.thebrowser.Browser"
    bundle path="/Applications/Arc.app"
    pid = 703 type="Foreground" flavor=3 Version="77482" fileType="APPL" creator="????" Arch=ARM64
12) "Terminal" ASN:0x0-0x12012:
    bundleID="com.apple.Terminal"
    bundle path="/System/Applications/Utilities/Terminal.app"
    pid = 801 type="Foreground" flavor=3 Version="2.14" fileType="APPL" creator="trml" Arch=ARM64
"#;

        assert_eq!(
            SystemMacosBackend::parse_running_apps(output),
            vec![
                RunningAppOption {
                    bundle_id: "company.thebrowser.Browser".to_string(),
                    app_name: "Arc".to_string(),
                },
                RunningAppOption {
                    bundle_id: "com.apple.Terminal".to_string(),
                    app_name: "Terminal".to_string(),
                },
            ]
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn running_app_filter_keeps_regular_apps_only() {
        assert!(SystemMacosBackend::should_include_running_app(
            NSApplicationActivationPolicy::Regular,
            true,
            false
        ));
        assert!(!SystemMacosBackend::should_include_running_app(
            NSApplicationActivationPolicy::Accessory,
            true,
            false
        ));
        assert!(!SystemMacosBackend::should_include_running_app(
            NSApplicationActivationPolicy::Prohibited,
            true,
            false
        ));
        assert!(!SystemMacosBackend::should_include_running_app(
            NSApplicationActivationPolicy::Regular,
            false,
            false
        ));
        assert!(!SystemMacosBackend::should_include_running_app(
            NSApplicationActivationPolicy::Regular,
            true,
            true
        ));
    }
}
