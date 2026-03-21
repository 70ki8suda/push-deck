use push_deck::app_state::AppState;
use push_deck::commands::{
    CommandError, CommandHost, CurrentConfigResponse, RestoreDefaultConfigResponse,
    TestActionResponse, UpdatePadBindingRequest, UpdatePadBindingResponse,
};
use push_deck::config::schema::{
    Config, PadAction, PadBinding, PadColorId, DEFAULT_PROFILE_ID,
};
use push_deck::config::store::{ConfigStore, ConfigStoreBackend};
use push_deck::macos::{ActionBackend, MacosError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[test]
fn load_current_config_returns_the_active_config_state() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should load the default config");

    let response = host.load_current_config().expect("config should load");

    let CurrentConfigResponse::Ready {
        config,
        runtime_state,
    } = response
    else {
        panic!("expected ready response");
    };

    assert_eq!(config, Config::default());
    assert_eq!(runtime_state.app_state, AppState::WaitingForDevice);
    assert_eq!(
        backend.file_contents(&path("config.json")),
        Some(
            serde_json::to_string_pretty(&Config::default())
                .expect("default config should serialize")
        )
    );
}

#[test]
fn update_pad_binding_persists_the_selected_pad_and_returns_the_updated_config() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should load the default config");

    let binding = PadBinding {
        pad_id: "r0c0".to_string(),
        label: "Launch".to_string(),
        color: PadColorId::Green,
        action: PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal"),
    };

    let response = host
        .update_pad_binding(UpdatePadBindingRequest {
            pad_id: "r0c0".to_string(),
            binding: binding.clone(),
        })
        .expect("update should succeed");

    let UpdatePadBindingResponse {
        config,
        runtime_state,
    } = response;

    assert_eq!(runtime_state.app_state, AppState::WaitingForDevice);
    assert_eq!(
        config.profile(DEFAULT_PROFILE_ID).expect("default profile").pads[0],
        binding
    );
    assert_eq!(
        backend.file_contents(&path("config.json")),
        Some(
            serde_json::to_string_pretty(&config)
                .expect("updated config should serialize")
        )
    );
}

#[test]
fn trigger_test_action_dispatches_the_selected_pad_binding() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let action_backend = TestActionBackend::default().with_launch_result(Ok(()));
    let host = CommandHost::bootstrap(store, action_backend.clone())
        .expect("bootstrap should load the default config");

    host.update_pad_binding(UpdatePadBindingRequest {
        pad_id: "r0c0".to_string(),
        binding: PadBinding {
            pad_id: "r0c0".to_string(),
            label: "Terminal".to_string(),
            color: PadColorId::Blue,
            action: PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal"),
        },
    })
    .expect("update should succeed");

    let response = host
        .trigger_test_action("r0c0")
        .expect("test action should dispatch");

    let TestActionResponse { runtime_state } = response;

    assert_eq!(runtime_state.app_state, AppState::WaitingForDevice);
    assert_eq!(
        action_backend.launch_calls(),
        vec!["com.apple.Terminal".to_string()]
    );
}

#[test]
fn restore_default_config_replaces_recovery_state_and_writes_default_config() {
    let backend = TestConfigStoreBackend::default();
    backend.write_existing(path("config.json"), "{ not json");
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should enter recovery mode");

    let response = host.load_current_config().expect("load should succeed");
    let CurrentConfigResponse::RecoveryRequired { recovery, .. } = response else {
        panic!("expected recovery response");
    };

    assert_eq!(recovery.config_path, path("config.json"));
    assert_eq!(
        backend.file_contents(&recovery.backup_path),
        Some("{ not json".to_string())
    );

    let restored = host
        .restore_default_config()
        .expect("restore should recover the default config");

    let RestoreDefaultConfigResponse {
        config,
        runtime_state,
    } = restored;

    assert_eq!(config, Config::default());
    assert_eq!(runtime_state.app_state, AppState::WaitingForDevice);
    assert_eq!(
        backend.file_contents(&path("config.json")),
        Some(
            serde_json::to_string_pretty(&Config::default())
                .expect("default config should serialize")
        )
    );
}

#[test]
fn restore_default_config_is_rejected_when_not_in_recovery_mode() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should load the default config");

    let error = host
        .restore_default_config()
        .expect_err("restore should be gated to recovery mode");

    assert_eq!(error, CommandError::NotInRecoveryMode);
}

#[derive(Clone, Default)]
struct TestConfigStoreBackend {
    state: Arc<Mutex<TestConfigStoreBackendState>>,
}

#[derive(Default)]
struct TestConfigStoreBackendState {
    files: HashMap<PathBuf, String>,
}

impl TestConfigStoreBackend {
    fn write_existing(&self, path: PathBuf, contents: &str) {
        self.state
            .lock()
            .expect("lock state")
            .files
            .insert(path, contents.to_string());
    }

    fn file_contents(&self, path: &Path) -> Option<String> {
        self.state
            .lock()
            .expect("lock state")
            .files
            .get(path)
            .cloned()
    }
}

impl ConfigStoreBackend for TestConfigStoreBackend {
    fn read_to_string(&self, path: &Path) -> std::io::Result<String> {
        self.state
            .lock()
            .expect("lock state")
            .files
            .get(path)
            .cloned()
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing file"))
    }

    fn write_string(&self, path: &Path, contents: &str) -> std::io::Result<()> {
        self.state
            .lock()
            .expect("lock state")
            .files
            .insert(path.to_path_buf(), contents.to_string());
        Ok(())
    }

    fn rename(&self, from: &Path, to: &Path) -> std::io::Result<()> {
        let mut state = self.state.lock().expect("lock state");
        let contents = state
            .files
            .remove(from)
            .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::NotFound, "missing source"))?;
        state.files.insert(to.to_path_buf(), contents);
        Ok(())
    }

    fn create_dir_all(&self, _path: &Path) -> std::io::Result<()> {
        Ok(())
    }

    fn remove_file(&self, path: &Path) -> std::io::Result<()> {
        self.state.lock().expect("lock state").files.remove(path);
        Ok(())
    }

    fn timestamp_millis(&self) -> u128 {
        1_700_000_000_000
    }
}

#[derive(Clone, Default)]
struct TestActionBackend {
    state: Arc<Mutex<TestActionBackendState>>,
}

#[derive(Default)]
struct TestActionBackendState {
    launch_result: Option<Result<(), MacosError>>,
    launch_calls: Vec<String>,
}

impl TestActionBackend {
    fn with_launch_result(self, result: Result<(), MacosError>) -> Self {
        self.state.lock().expect("lock state").launch_result = Some(result);
        self
    }

    fn launch_calls(&self) -> Vec<String> {
        self.state.lock().expect("lock state").launch_calls.clone()
    }
}

impl ActionBackend for TestActionBackend {
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError> {
        let mut state = self.state.lock().expect("lock state");
        state.launch_calls.push(bundle_id.to_string());
        state.launch_result.clone().unwrap_or(Ok(()))
    }
}

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}
