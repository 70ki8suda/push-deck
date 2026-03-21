#[path = "../src/config/schema.rs"]
mod schema;

mod config {
    pub mod schema {
        pub use crate::schema::*;
    }
}

#[path = "../src/macos/mod.rs"]
mod macos;

#[path = "../src/actions/mod.rs"]
mod actions;

use actions::{dispatch_pad_action, ActionExecutionError, LaunchOrFocusError};
use schema::PadAction;
use macos::{ActionBackend, MacosError};
use std::sync::{Arc, Mutex};

#[test]
fn dispatches_launch_or_focus_action_with_provided_bundle_id() {
    let backend = FakeBackend::new().with_launch_result(Ok(()));
    let action = PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal");

    dispatch_pad_action(&backend, &action).expect("action should dispatch");

    assert_eq!(backend.launch_calls(), vec!["com.apple.Terminal".to_string()]);
}

#[test]
fn returns_error_when_bundle_id_cannot_be_resolved() {
    let backend = FakeBackend::new();
    let action = PadAction::launch_or_focus_app("", "Terminal");

    let error = dispatch_pad_action(&backend, &action).expect_err("missing bundle id should fail");

    assert_eq!(
        error,
        ActionExecutionError::LaunchOrFocus(LaunchOrFocusError::BundleIdUnavailable {
            app_name: "Terminal".to_string(),
        })
    );
    assert!(backend.launch_calls().is_empty());
}

#[test]
fn surfaces_app_not_found_from_backend() {
    let backend = FakeBackend::new().with_launch_result(Err(MacosError::AppNotFound {
        bundle_id: "com.apple.Nonexistent".to_string(),
    }));
    let action = PadAction::launch_or_focus_app("com.apple.Nonexistent", "Missing App");

    let error = dispatch_pad_action(&backend, &action).expect_err("missing app should fail");

    assert_eq!(
        error,
        ActionExecutionError::LaunchOrFocus(LaunchOrFocusError::AppNotFound {
            bundle_id: "com.apple.Nonexistent".to_string(),
        })
    );
    assert_eq!(error.to_string(), "app not found: com.apple.Nonexistent");
    assert_eq!(
        backend.launch_calls(),
        vec!["com.apple.Nonexistent".to_string()]
    );
}

#[derive(Clone, Default)]
struct FakeBackend {
    state: Arc<Mutex<FakeBackendState>>,
}

#[derive(Default)]
struct FakeBackendState {
    launch_result: Option<Result<(), MacosError>>,
    launch_calls: Vec<String>,
}

impl FakeBackend {
    fn new() -> Self {
        Self::default()
    }

    fn with_launch_result(self, result: Result<(), MacosError>) -> Self {
        self.state.lock().expect("lock state").launch_result = Some(result);
        self
    }

    fn launch_calls(&self) -> Vec<String> {
        self.state.lock().expect("lock state").launch_calls.clone()
    }
}

impl ActionBackend for FakeBackend {
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError> {
        let mut state = self.state.lock().expect("lock state");
        state.launch_calls.push(bundle_id.to_string());
        state.launch_result.clone().unwrap_or(Ok(()))
    }
}
