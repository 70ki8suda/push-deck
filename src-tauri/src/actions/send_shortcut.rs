use crate::app_state::{RuntimeState, ShortcutCapabilityState};
use crate::config::schema::{PadAction, ShortcutModifier};
use crate::events::{emit_runtime_event, RuntimeEvent};
use crate::macos::{ActionBackend, MacosError};
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SendShortcutError {
    InvalidShortcutPayload { message: String },
    AccessibilityPermissionUnavailable,
    NoFrontmostTarget,
    Macos(MacosError),
}

impl Display for SendShortcutError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidShortcutPayload { message } => {
                write!(f, "invalid shortcut payload: {message}")
            }
            Self::AccessibilityPermissionUnavailable => {
                f.write_str("shortcut execution unavailable")
            }
            Self::NoFrontmostTarget => f.write_str("no frontmost app"),
            Self::Macos(error) => Display::fmt(error, f),
        }
    }
}

impl Error for SendShortcutError {}

pub fn send_shortcut_action<B>(backend: &B, action: &PadAction) -> Result<(), SendShortcutError>
where
    B: ActionBackend,
{
    let PadAction::SendShortcut { key, modifiers } = action else {
        return Ok(());
    };

    let modifiers = normalize_shortcut_modifiers(modifiers)?;

    let accessibility_available = backend
        .shortcut_accessibility_available()
        .map_err(SendShortcutError::Macos)?;
    if !accessibility_available {
        return Err(SendShortcutError::AccessibilityPermissionUnavailable);
    }

    if backend
        .frontmost_target()
        .map_err(SendShortcutError::Macos)?
        .is_none()
    {
        return Err(SendShortcutError::NoFrontmostTarget);
    }

    backend
        .send_shortcut(*key, &modifiers)
        .map_err(SendShortcutError::Macos)
}

pub fn emit_shortcut_capability_state<R, E>(
    emitter: &E,
    runtime_state: &RuntimeState,
    shortcut: ShortcutCapabilityState,
) -> tauri::Result<()>
where
    R: tauri::Runtime,
    E: tauri::Emitter<R>,
{
    emit_runtime_event(
        emitter,
        RuntimeEvent::StateChanged {
            state: runtime_state.with_shortcut_capability(shortcut),
        },
    )
}

fn normalize_shortcut_modifiers(
    modifiers: &[ShortcutModifier],
) -> Result<Vec<ShortcutModifier>, SendShortcutError> {
    let mut normalized = modifiers.to_vec();
    normalized.sort_by_key(|modifier| modifier_order(*modifier));
    normalized.dedup();

    if normalized.len() != modifiers.len() {
        return Err(SendShortcutError::InvalidShortcutPayload {
            message: "duplicate shortcut modifiers".to_string(),
        });
    }

    Ok(normalized)
}

fn modifier_order(modifier: ShortcutModifier) -> usize {
    match modifier {
        ShortcutModifier::Cmd => 0,
        ShortcutModifier::Shift => 1,
        ShortcutModifier::Opt => 2,
        ShortcutModifier::Ctrl => 3,
    }
}
