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
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            active_profile_id: DEFAULT_PROFILE_ID.to_string(),
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
    Red,
    Orange,
    Yellow,
    Green,
    Cyan,
    Blue,
    Purple,
    Pink,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum PadAction {
    Unassigned,
    LaunchOrFocusApp {
        bundle_id: String,
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
        let mut modifiers = self.modifiers.clone();
        modifiers.sort_by_key(|modifier| modifier.sort_order());
        modifiers.dedup();

        if modifiers.len() != self.modifiers.len() {
            return Err(ConfigError::InvalidShortcutModifiers);
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigError {
    InvalidPadId(String),
    DuplicatePadId(String),
    InvalidShortcutModifiers,
}

impl Display for ConfigError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidPadId(pad_id) => write!(f, "invalid padId: {pad_id}"),
            Self::DuplicatePadId(pad_id) => write!(f, "duplicate padId: {pad_id}"),
            Self::InvalidShortcutModifiers => f.write_str("invalid shortcut modifiers"),
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
