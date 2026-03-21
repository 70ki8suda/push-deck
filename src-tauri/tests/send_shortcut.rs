#[path = "../src/config/schema.rs"]
mod schema;

mod config {
    pub mod schema {
        pub use crate::schema::*;
    }
}

#[path = "../src/macos/mod.rs"]
mod macos;

#[path = "../src/actions/send_shortcut.rs"]
mod send_shortcut;

#[path = "../src/actions/mod.rs"]
mod actions;

use actions::{dispatch_pad_action, ActionExecutionError, SendShortcutError};
use macos::{ActionBackend, MacosError};
use push_deck::app_state::{
    record_shortcut_capability, recorded_shortcut_capability, ShortcutCapabilityState,
};
use schema::{PadAction, ShortcutKey, ShortcutModifier};
use std::sync::{Arc, Mutex, OnceLock};

#[test]
fn dispatches_shortcut_when_permission_and_frontmost_target_are_present() {
    let backend = FakeBackend::new()
        .with_accessibility_permission(true)
        .with_frontmost_target(Some("Ableton Live".to_string()))
        .with_send_result(Ok(()));
    let action = PadAction::SendShortcut {
        key: ShortcutKey::K,
        modifiers: vec![ShortcutModifier::Cmd, ShortcutModifier::Shift],
    };

    dispatch_pad_action(&backend, &action).expect("shortcut should dispatch");

    assert_eq!(
        backend.accessibility_queries(),
        vec![true],
        "permission should be checked first"
    );
    assert_eq!(
        backend.frontmost_target_queries(),
        vec![true],
        "frontmost target should be queried after permission"
    );
    assert_eq!(
        backend.send_calls(),
        vec![(
            ShortcutKey::K,
            vec![ShortcutModifier::Cmd, ShortcutModifier::Shift]
        )]
    );
}

#[test]
fn returns_error_when_accessibility_permission_is_missing() {
    let backend = FakeBackend::new()
        .with_accessibility_permission(false)
        .with_frontmost_target(Some("Ableton Live".to_string()));
    let action = PadAction::SendShortcut {
        key: ShortcutKey::K,
        modifiers: vec![ShortcutModifier::Cmd],
    };

    let error = dispatch_pad_action(&backend, &action).expect_err("permission should fail");

    assert_eq!(
        error,
        ActionExecutionError::SendShortcut(SendShortcutError::AccessibilityPermissionUnavailable)
    );
    assert_eq!(backend.accessibility_queries(), vec![true]);
    assert!(
        backend.frontmost_target_queries().is_empty(),
        "frontmost target should not be queried when permission is missing"
    );
    assert!(backend.send_calls().is_empty());
}

#[test]
fn returns_error_when_no_frontmost_target_exists() {
    let backend = FakeBackend::new()
        .with_accessibility_permission(true)
        .with_frontmost_target(None);
    let action = PadAction::SendShortcut {
        key: ShortcutKey::K,
        modifiers: vec![ShortcutModifier::Cmd],
    };

    let error =
        dispatch_pad_action(&backend, &action).expect_err("frontmost app should be required");

    assert_eq!(
        error,
        ActionExecutionError::SendShortcut(SendShortcutError::NoFrontmostTarget)
    );
    assert_eq!(backend.accessibility_queries(), vec![true]);
    assert_eq!(backend.frontmost_target_queries(), vec![true]);
    assert!(backend.send_calls().is_empty());
}

#[test]
fn rejects_invalid_shortcut_payload_before_backend_execution() {
    let backend = FakeBackend::new()
        .with_accessibility_permission(true)
        .with_frontmost_target(Some("Ableton Live".to_string()));
    let action = PadAction::SendShortcut {
        key: ShortcutKey::K,
        modifiers: vec![ShortcutModifier::Cmd, ShortcutModifier::Cmd],
    };

    let error = dispatch_pad_action(&backend, &action).expect_err("invalid payload should fail");

    assert!(
        matches!(
            error,
            ActionExecutionError::SendShortcut(SendShortcutError::InvalidShortcutPayload { .. })
        ),
        "unexpected error: {error:?}"
    );
    assert!(backend.accessibility_queries().is_empty());
    assert!(backend.frontmost_target_queries().is_empty());
    assert!(backend.send_calls().is_empty());
}

#[test]
fn reports_shortcut_capability_as_available_when_accessibility_is_granted() {
    let _guard = capability_test_guard();
    record_shortcut_capability(ShortcutCapabilityState::Unavailable);
    let backend = FakeBackend::new().with_accessibility_permission(true);

    let capability =
        send_shortcut::shortcut_capability_state(&backend).expect("capability should resolve");

    assert_eq!(capability, ShortcutCapabilityState::Available);
    assert_eq!(backend.accessibility_queries(), vec![true]);
    assert_eq!(recorded_shortcut_capability(), ShortcutCapabilityState::Available);
}

#[test]
fn reports_shortcut_capability_as_unavailable_when_accessibility_is_missing() {
    let _guard = capability_test_guard();
    record_shortcut_capability(ShortcutCapabilityState::Unavailable);
    let backend = FakeBackend::new().with_accessibility_permission(false);

    let capability =
        send_shortcut::shortcut_capability_state(&backend).expect("capability should resolve");

    assert_eq!(capability, ShortcutCapabilityState::Unavailable);
    assert_eq!(backend.accessibility_queries(), vec![true]);
    assert_eq!(
        recorded_shortcut_capability(),
        ShortcutCapabilityState::Unavailable
    );
}

fn capability_test_guard() -> std::sync::MutexGuard<'static, ()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
        .lock()
        .expect("capability test lock poisoned")
}

#[derive(Clone, Default)]
struct FakeBackend {
    state: Arc<Mutex<FakeBackendState>>,
}

#[derive(Default)]
struct FakeBackendState {
    accessibility_permission: Option<bool>,
    frontmost_target: Option<Option<String>>,
    send_result: Option<Result<(), MacosError>>,
    accessibility_queries: Vec<bool>,
    frontmost_target_queries: Vec<bool>,
    send_calls: Vec<(ShortcutKey, Vec<ShortcutModifier>)>,
}

impl FakeBackend {
    fn new() -> Self {
        Self::default()
    }

    fn with_accessibility_permission(self, granted: bool) -> Self {
        self.state
            .lock()
            .expect("lock state")
            .accessibility_permission = Some(granted);
        self
    }

    fn with_frontmost_target(self, target: Option<String>) -> Self {
        self.state.lock().expect("lock state").frontmost_target = Some(target);
        self
    }

    fn with_send_result(self, result: Result<(), MacosError>) -> Self {
        self.state.lock().expect("lock state").send_result = Some(result);
        self
    }

    fn accessibility_queries(&self) -> Vec<bool> {
        self.state
            .lock()
            .expect("lock state")
            .accessibility_queries
            .clone()
    }

    fn frontmost_target_queries(&self) -> Vec<bool> {
        self.state
            .lock()
            .expect("lock state")
            .frontmost_target_queries
            .clone()
    }

    fn send_calls(&self) -> Vec<(ShortcutKey, Vec<ShortcutModifier>)> {
        self.state.lock().expect("lock state").send_calls.clone()
    }
}

impl ActionBackend for FakeBackend {
    fn launch_or_focus_bundle_id(&self, _bundle_id: &str) -> Result<(), MacosError> {
        Err(MacosError::UnsupportedAction)
    }

    fn shortcut_accessibility_available(&self) -> Result<bool, MacosError> {
        let mut state = self.state.lock().expect("lock state");
        state.accessibility_queries.push(true);
        Ok(state.accessibility_permission.unwrap_or(true))
    }

    fn frontmost_target(&self) -> Result<Option<String>, MacosError> {
        let mut state = self.state.lock().expect("lock state");
        state.frontmost_target_queries.push(true);
        Ok(state.frontmost_target.clone().unwrap_or(None))
    }

    fn send_shortcut(
        &self,
        key: ShortcutKey,
        modifiers: &[ShortcutModifier],
    ) -> Result<(), MacosError> {
        let mut state = self.state.lock().expect("lock state");
        state.send_calls.push((key, modifiers.to_vec()));
        state.send_result.clone().unwrap_or(Ok(()))
    }
}
