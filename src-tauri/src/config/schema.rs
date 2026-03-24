use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Display, Formatter};

pub const SCHEMA_VERSION: u32 = 1;
pub const DEFAULT_PROFILE_ID: &str = "default";
pub const DEFAULT_PROFILE_NAME: &str = "Default";

const PAD_IDS: [&str; 64] = [
    "r0c0", "r0c1", "r0c2", "r0c3", "r0c4", "r0c5", "r0c6", "r0c7", "r1c0", "r1c1", "r1c2", "r1c3",
    "r1c4", "r1c5", "r1c6", "r1c7", "r2c0", "r2c1", "r2c2", "r2c3", "r2c4", "r2c5", "r2c6", "r2c7",
    "r3c0", "r3c1", "r3c2", "r3c3", "r3c4", "r3c5", "r3c6", "r3c7", "r4c0", "r4c1", "r4c2", "r4c3",
    "r4c4", "r4c5", "r4c6", "r4c7", "r5c0", "r5c1", "r5c2", "r5c3", "r5c4", "r5c5", "r5c6", "r5c7",
    "r6c0", "r6c1", "r6c2", "r6c3", "r6c4", "r6c5", "r6c6", "r6c7", "r7c0", "r7c1", "r7c2", "r7c3",
    "r7c4", "r7c5", "r7c6", "r7c7",
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
    #[serde(rename = "schemaVersion")]
    pub schema_version: u32,
    pub settings: AppSettings,
    pub profiles: Vec<LayoutProfile>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            settings: AppSettings::default(),
            profiles: vec![LayoutProfile::default_profile()],
        }
    }
}

impl Config {
    pub fn from_parts(
        settings: AppSettings,
        profiles: Vec<LayoutProfile>,
    ) -> Result<Self, ConfigError> {
        let active_profile_id = settings.active_profile_id.clone();
        let default_profile = profiles
            .into_iter()
            .find(|profile| profile.id == DEFAULT_PROFILE_ID)
            .map(normalize_profile)
            .transpose()?;

        let mut config = match default_profile {
            Some(profile) => Self {
                schema_version: SCHEMA_VERSION,
                settings,
                profiles: vec![profile],
            },
            None => Self::default(),
        };

        config.settings.active_profile_id = if config
            .profiles
            .iter()
            .any(|profile| profile.id == active_profile_id)
        {
            active_profile_id
        } else {
            DEFAULT_PROFILE_ID.to_string()
        };

        Ok(config)
    }

    pub fn profile(&self, id: &str) -> Option<&LayoutProfile> {
        self.profiles.iter().find(|profile| profile.id == id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub active_profile_id: String,
    #[serde(default)]
    pub push3_color_calibration: Push3ColorCalibration,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            active_profile_id: DEFAULT_PROFILE_ID.to_string(),
            push3_color_calibration: Push3ColorCalibration::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[serde(default)]
pub struct Push3ColorCalibration {
    pub white: u8,
    pub peach: u8,
    pub coral: u8,
    pub red: u8,
    pub orange: u8,
    pub amber: u8,
    pub yellow: u8,
    pub lime: u8,
    pub chartreuse: u8,
    pub green: u8,
    pub mint: u8,
    pub teal: u8,
    pub cyan: u8,
    pub sky: u8,
    pub blue: u8,
    pub indigo: u8,
    pub purple: u8,
    pub magenta: u8,
    pub rose: u8,
    pub pink: u8,
}

impl Default for Push3ColorCalibration {
    fn default() -> Self {
        Self {
            white: 3,
            peach: 8,
            coral: 4,
            red: 5,
            orange: 9,
            amber: 12,
            yellow: 13,
            lime: 16,
            chartreuse: 17,
            green: 21,
            mint: 29,
            teal: 33,
            cyan: 37,
            sky: 41,
            blue: 45,
            indigo: 48,
            purple: 49,
            magenta: 52,
            rose: 56,
            pink: 57,
        }
    }
}

impl Push3ColorCalibration {
    pub fn resolve(&self, color: PadColorId) -> u8 {
        match color {
            PadColorId::Off => 0,
            PadColorId::White => self.white,
            PadColorId::Peach => self.peach,
            PadColorId::Coral => self.coral,
            PadColorId::Red => self.red,
            PadColorId::Orange => self.orange,
            PadColorId::Amber => self.amber,
            PadColorId::Yellow => self.yellow,
            PadColorId::Lime => self.lime,
            PadColorId::Chartreuse => self.chartreuse,
            PadColorId::Green => self.green,
            PadColorId::Mint => self.mint,
            PadColorId::Teal => self.teal,
            PadColorId::Cyan => self.cyan,
            PadColorId::Sky => self.sky,
            PadColorId::Blue => self.blue,
            PadColorId::Indigo => self.indigo,
            PadColorId::Purple => self.purple,
            PadColorId::Magenta => self.magenta,
            PadColorId::Rose => self.rose,
            PadColorId::Pink => self.pink,
        }
    }

    pub fn update(&mut self, logical_color: PadColorId, output_value: u8) {
        match logical_color {
            PadColorId::Off => {}
            PadColorId::White => self.white = output_value,
            PadColorId::Peach => self.peach = output_value,
            PadColorId::Coral => self.coral = output_value,
            PadColorId::Red => self.red = output_value,
            PadColorId::Orange => self.orange = output_value,
            PadColorId::Amber => self.amber = output_value,
            PadColorId::Yellow => self.yellow = output_value,
            PadColorId::Lime => self.lime = output_value,
            PadColorId::Chartreuse => self.chartreuse = output_value,
            PadColorId::Green => self.green = output_value,
            PadColorId::Mint => self.mint = output_value,
            PadColorId::Teal => self.teal = output_value,
            PadColorId::Cyan => self.cyan = output_value,
            PadColorId::Sky => self.sky = output_value,
            PadColorId::Blue => self.blue = output_value,
            PadColorId::Indigo => self.indigo = output_value,
            PadColorId::Purple => self.purple = output_value,
            PadColorId::Magenta => self.magenta = output_value,
            PadColorId::Rose => self.rose = output_value,
            PadColorId::Pink => self.pink = output_value,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LayoutProfile {
    pub id: String,
    pub name: String,
    pub pads: Vec<PadBinding>,
}

impl LayoutProfile {
    fn default_profile() -> Self {
        Self {
            id: DEFAULT_PROFILE_ID.to_string(),
            name: DEFAULT_PROFILE_NAME.to_string(),
            pads: PAD_IDS
                .iter()
                .map(|pad_id| PadBinding::unassigned((*pad_id).to_string()))
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PadBinding {
    pub pad_id: String,
    pub label: String,
    pub color: PadColorId,
    pub action: PadAction,
}

impl PadBinding {
    fn unassigned(pad_id: String) -> Self {
        Self {
            pad_id,
            label: String::new(),
            color: PadColorId::Off,
            action: PadAction::Unassigned,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PadColorId {
    Off,
    White,
    Peach,
    Coral,
    Red,
    Orange,
    Amber,
    Yellow,
    Lime,
    Chartreuse,
    Green,
    Mint,
    Teal,
    Cyan,
    Sky,
    Blue,
    Indigo,
    Purple,
    Magenta,
    Rose,
    Pink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PadAction {
    Unassigned,
    LaunchOrFocusApp {
        #[serde(rename = "bundleId")]
        bundle_id: String,
        #[serde(rename = "appName")]
        app_name: String,
    },
    SendShortcut {
        key: ShortcutKey,
        modifiers: Vec<ShortcutModifier>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutModifier {
    Cmd,
    Shift,
    Opt,
    Ctrl,
}

impl ShortcutModifier {
    fn sort_order(self) -> usize {
        match self {
            Self::Cmd => 0,
            Self::Shift => 1,
            Self::Opt => 2,
            Self::Ctrl => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShortcutKey {
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    #[serde(rename = "0")]
    Num0,
    #[serde(rename = "1")]
    Num1,
    #[serde(rename = "2")]
    Num2,
    #[serde(rename = "3")]
    Num3,
    #[serde(rename = "4")]
    Num4,
    #[serde(rename = "5")]
    Num5,
    #[serde(rename = "6")]
    Num6,
    #[serde(rename = "7")]
    Num7,
    #[serde(rename = "8")]
    Num8,
    #[serde(rename = "9")]
    Num9,
    F1,
    F2,
    F3,
    F4,
    F5,
    F6,
    F7,
    F8,
    F9,
    F10,
    F11,
    F12,
    ArrowUp,
    ArrowDown,
    ArrowLeft,
    ArrowRight,
    Space,
    Tab,
    Enter,
    Escape,
    Delete,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShortcutSpec {
    pub modifiers: Vec<ShortcutModifier>,
    pub key: ShortcutKey,
}

impl ShortcutSpec {
    pub fn validate(&self) -> Result<(), ConfigError> {
        validate_shortcut_modifiers(&self.modifiers).map(|_| ())
    }
}

impl PadAction {
    pub fn launch_or_focus_app(bundle_id: impl Into<String>, app_name: impl Into<String>) -> Self {
        Self::LaunchOrFocusApp {
            bundle_id: bundle_id.into(),
            app_name: app_name.into(),
        }
    }

    fn validate_and_normalize(&mut self) -> Result<(), ConfigError> {
        match self {
            Self::Unassigned => Ok(()),
            Self::LaunchOrFocusApp {
                bundle_id,
                app_name,
            } => {
                if bundle_id.trim().is_empty() || app_name.trim().is_empty() {
                    return Err(ConfigError::InvalidActionPayload(
                        "launch_or_focus_app requires bundleId and appName".to_string(),
                    ));
                }

                Ok(())
            }
            Self::SendShortcut { modifiers, .. } => {
                let normalized = validate_shortcut_modifiers(modifiers)?;
                *modifiers = normalized;
                Ok(())
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidPadId(String),
    DuplicatePadId(String),
    InvalidShortcutModifiers,
    InvalidActionPayload(String),
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPadId(pad_id) => write!(f, "invalid padId: {pad_id}"),
            Self::DuplicatePadId(pad_id) => write!(f, "duplicate padId: {pad_id}"),
            Self::InvalidShortcutModifiers => f.write_str("invalid shortcut modifiers"),
            Self::InvalidActionPayload(message) => {
                write!(f, "invalid action payload: {message}")
            }
        }
    }
}

impl Error for ConfigError {}

fn normalize_profile(profile: LayoutProfile) -> Result<LayoutProfile, ConfigError> {
    let mut provided = HashMap::new();

    for binding in profile.pads {
        if !PAD_IDS.contains(&binding.pad_id.as_str()) {
            return Err(ConfigError::InvalidPadId(binding.pad_id));
        }

        let pad_id = binding.pad_id.clone();
        let mut binding = binding;
        binding.action.validate_and_normalize()?;
        if provided.insert(pad_id.clone(), binding).is_some() {
            return Err(ConfigError::DuplicatePadId(pad_id));
        }
    }

    let pads = PAD_IDS
        .iter()
        .map(|pad_id| {
            provided
                .remove(*pad_id)
                .unwrap_or_else(|| PadBinding::unassigned((*pad_id).to_string()))
        })
        .collect();

    Ok(LayoutProfile {
        id: profile.id,
        name: profile.name,
        pads,
    })
}

fn validate_shortcut_modifiers(
    modifiers: &[ShortcutModifier],
) -> Result<Vec<ShortcutModifier>, ConfigError> {
    let mut normalized = modifiers.to_vec();
    normalized.sort_by_key(|modifier| modifier.sort_order());
    normalized.dedup();

    if normalized.len() != modifiers.len() {
        return Err(ConfigError::InvalidShortcutModifiers);
    }

    Ok(normalized)
}
