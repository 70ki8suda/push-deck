use push_deck::actions::{dispatch_pad_action, ActionExecutionError, LaunchOrFocusError};
use push_deck::config::schema::PadAction;
use push_deck::macos::{LaunchOrFocusBackend, MacosError};
use std::sync::{Arc, Mutex};

#[test]
fn dispatches_launch_or_focus_action_with_provided_bundle_id() {
    let backend = FakeBackend::new()
        .with_launch_result(Ok(()))
        .with_resolve_result(Ok(Some("ignored.bundle.id".to_string())));
    let action = PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal");

    dispatch_pad_action(&backend, &action).expect("action should dispatch");

    assert_eq!(
        backend.launch_calls(),
        vec!["com.apple.Terminal".to_string()]
    );
    assert!(backend.resolve_calls().is_empty());
}

#[test]
fn returns_error_when_bundle_id_cannot_be_resolved() {
    let backend = FakeBackend::new().with_resolve_result(Ok(None));
    let action = PadAction::launch_or_focus_app("", "Terminal");

    let error = dispatch_pad_action(&backend, &action).expect_err("missing bundle id should fail");

    assert_eq!(
        error,
        ActionExecutionError::LaunchOrFocus(LaunchOrFocusError::BundleIdUnavailable {
            app_name: "Terminal".to_string(),
        })
    );
    assert_eq!(backend.resolve_calls(), vec!["Terminal".to_string()]);
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
    resolve_result: Option<Result<Option<String>, MacosError>>,
    launch_result: Option<Result<(), MacosError>>,
    resolve_calls: Vec<String>,
    launch_calls: Vec<String>,
}

impl FakeBackend {
    fn new() -> Self {
        Self::default()
    }

    fn with_resolve_result(self, result: Result<Option<String>, MacosError>) -> Self {
        self.state.lock().expect("lock state").resolve_result = Some(result);
        self
    }

    fn with_launch_result(self, result: Result<(), MacosError>) -> Self {
        self.state.lock().expect("lock state").launch_result = Some(result);
        self
    }

    fn resolve_calls(&self) -> Vec<String> {
        self.state.lock().expect("lock state").resolve_calls.clone()
    }

    fn launch_calls(&self) -> Vec<String> {
        self.state.lock().expect("lock state").launch_calls.clone()
    }
}

impl LaunchOrFocusBackend for FakeBackend {
    fn resolve_bundle_id(&self, app_name: &str) -> Result<Option<String>, MacosError> {
        let mut state = self.state.lock().expect("lock state");
        state.resolve_calls.push(app_name.to_string());
        state
            .resolve_result
            .clone()
            .unwrap_or_else(|| Ok(Some(format!("resolved.{app_name}"))))
    }

    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError> {
        let mut state = self.state.lock().expect("lock state");
        state.launch_calls.push(bundle_id.to_string());
        state.launch_result.clone().unwrap_or(Ok(()))
    }
}
