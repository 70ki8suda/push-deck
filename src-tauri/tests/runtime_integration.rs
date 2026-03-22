use push_deck::app_state::{AppState, DeviceEndpointDescriptor};
use push_deck::commands::{CommandHost, CurrentConfigResponse, UpdatePadBindingRequest};
use push_deck::config::schema::{PadAction, PadBinding, PadColorId};
use push_deck::config::store::{ConfigStore, ConfigStoreBackend};
use push_deck::device::{DeviceDiscoveryError, DeviceDiscoverySource};
use push_deck::macos::{ActionBackend, MacosError};
use push_deck::should_hide_on_close;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[test]
fn first_launch_creates_default_config_and_waits_for_device() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should create the default config");

    host.refresh_runtime(&TestDiscoverySource::waiting())
        .expect("runtime refresh should succeed");

    let response = host.load_current_config().expect("load should succeed");
    let CurrentConfigResponse::Ready {
        config,
        device_name,
        device_connected,
        runtime_state,
    } = response
    else {
        panic!("expected ready response");
    };

    assert_eq!(config, push_deck::config::schema::Config::default());
    assert_eq!(device_name, None);
    assert!(!device_connected);
    assert_eq!(runtime_state.app_state, AppState::WaitingForDevice);
    assert_eq!(runtime_state.capabilities.shortcut, push_deck::app_state::ShortcutCapabilityState::Available);
    assert!(backend.file_contents(&path("config.json")).is_some());
}

#[test]
fn connected_device_reports_ready_with_unavailable_shortcut_capability() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let host = CommandHost::bootstrap(
        store,
        TestActionBackend::default().with_shortcut_capability(false),
    )
    .expect("bootstrap should succeed");

    host.refresh_runtime(&TestDiscoverySource::connected("Ableton Push 3"))
        .expect("runtime refresh should succeed");

    let response = host.load_current_config().expect("load should succeed");
    let CurrentConfigResponse::Ready {
        device_name,
        device_connected,
        runtime_state,
        ..
    } = response
    else {
        panic!("expected ready response");
    };

    assert_eq!(device_name.as_deref(), Some("Ableton Push 3"));
    assert!(device_connected);
    assert_eq!(runtime_state.app_state, AppState::Ready);
    assert_eq!(
        runtime_state.capabilities.shortcut,
        push_deck::app_state::ShortcutCapabilityState::Unavailable
    );
}

#[test]
fn config_recovery_required_survives_runtime_refresh() {
    let backend = TestConfigStoreBackend::default();
    backend.write_existing(path("config.json"), "{ broken");
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should enter recovery mode");

    host.refresh_runtime(&TestDiscoverySource::connected("Ableton Push 3"))
        .expect("runtime refresh should succeed");

    let response = host.load_current_config().expect("load should succeed");
    let CurrentConfigResponse::RecoveryRequired {
        device_name,
        device_connected,
        runtime_state,
        ..
    } = response
    else {
        panic!("expected recovery response");
    };

    assert_eq!(device_name.as_deref(), Some("Ableton Push 3"));
    assert!(device_connected);
    assert_eq!(runtime_state.app_state, AppState::ConfigRecoveryRequired);
}

#[test]
fn save_failed_state_returns_to_prior_stable_state_after_retry() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should succeed");

    host.refresh_runtime(&TestDiscoverySource::waiting())
        .expect("runtime refresh should succeed");

    let request = UpdatePadBindingRequest {
        pad_id: "r0c0".to_string(),
        binding: PadBinding {
            pad_id: "r0c0".to_string(),
            label: "Terminal".to_string(),
            color: PadColorId::Blue,
            action: PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal"),
        },
    };

    backend.fail_next_write();
    host.update_pad_binding(request.clone())
        .expect_err("save should fail while backend rejects writes");

    let failed = host.load_current_config().expect("load should still succeed");
    let CurrentConfigResponse::Ready { runtime_state, .. } = failed else {
        panic!("expected ready response");
    };
    assert_eq!(runtime_state.app_state, AppState::SaveFailed);

    host.update_pad_binding(request)
        .expect("retry should restore the prior stable state");

    let recovered = host.load_current_config().expect("load should succeed");
    let CurrentConfigResponse::Ready { runtime_state, .. } = recovered else {
        panic!("expected ready response");
    };
    assert_eq!(runtime_state.app_state, AppState::WaitingForDevice);
}

#[test]
fn runtime_refresh_clears_save_failed_after_non_recovery_failures() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should succeed");

    host.refresh_runtime(&TestDiscoverySource::waiting())
        .expect("runtime refresh should succeed");

    backend.fail_next_write();
    host.update_pad_binding(UpdatePadBindingRequest {
        pad_id: "r0c0".to_string(),
        binding: PadBinding {
            pad_id: "r0c0".to_string(),
            label: "Terminal".to_string(),
            color: PadColorId::Blue,
            action: PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal"),
        },
    })
    .expect_err("save should fail");

    host.refresh_runtime(&TestDiscoverySource::connected("Ableton Push 3"))
        .expect("runtime refresh should restore the latest stable state");

    let response = host.load_current_config().expect("load should succeed");
    let CurrentConfigResponse::Ready {
        device_name,
        device_connected,
        runtime_state,
        ..
    } = response
    else {
        panic!("expected ready response");
    };

    assert_eq!(device_name.as_deref(), Some("Ableton Push 3"));
    assert!(device_connected);
    assert_eq!(runtime_state.app_state, AppState::Ready);
}

#[test]
fn main_window_close_requests_are_hidden_instead_of_closed() {
    assert!(should_hide_on_close("main"));
    assert!(!should_hide_on_close("preferences"));
}

#[derive(Clone, Default)]
struct TestConfigStoreBackend {
    state: Arc<Mutex<TestConfigStoreBackendState>>,
}

#[derive(Default)]
struct TestConfigStoreBackendState {
    files: HashMap<PathBuf, String>,
    fail_next_write: bool,
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

    fn fail_next_write(&self) {
        self.state.lock().expect("lock state").fail_next_write = true;
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
        let mut state = self.state.lock().expect("lock state");
        if state.fail_next_write {
            state.fail_next_write = false;
            return Err(std::io::Error::new(
                std::io::ErrorKind::PermissionDenied,
                "simulated write failure",
            ));
        }
        state.files.insert(path.to_path_buf(), contents.to_string());
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

#[derive(Clone)]
struct TestDiscoverySource {
    response: Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError>,
}

impl TestDiscoverySource {
    fn waiting() -> Self {
        Self {
            response: Ok(vec![]),
        }
    }

    fn connected(display_name: &str) -> Self {
        Self {
            response: Ok(vec![DeviceEndpointDescriptor::push_3(
                "endpoint-1",
                display_name,
            )]),
        }
    }
}

impl DeviceDiscoverySource for TestDiscoverySource {
    fn discover_devices(&self) -> Result<Vec<DeviceEndpointDescriptor>, DeviceDiscoveryError> {
        self.response.clone()
    }
}

#[derive(Clone, Default)]
struct TestActionBackend {
    state: Arc<Mutex<TestActionBackendState>>,
}

struct TestActionBackendState {
    shortcut_capability: bool,
}

impl Default for TestActionBackendState {
    fn default() -> Self {
        Self {
            shortcut_capability: true,
        }
    }
}

impl TestActionBackend {
    fn with_shortcut_capability(self, is_available: bool) -> Self {
        self.state.lock().expect("lock state").shortcut_capability = is_available;
        self
    }
}

impl ActionBackend for TestActionBackend {
    fn launch_or_focus_bundle_id(&self, _bundle_id: &str) -> Result<(), MacosError> {
        Ok(())
    }

    fn shortcut_accessibility_available(&self) -> Result<bool, MacosError> {
        Ok(self.state.lock().expect("lock state").shortcut_capability)
    }
}

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}
