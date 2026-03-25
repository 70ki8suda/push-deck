use push_deck::app_state::{AppState, DeviceEndpointDescriptor};
use push_deck::commands::{
    CommandHost, CurrentConfigResponse, UpdatePadBindingRequest,
    UpdatePush3ColorCalibrationRequest,
};
use push_deck::config::schema::{Config, PadAction, PadBinding, PadColorId};
use push_deck::config::store::{ConfigStore, ConfigStoreBackend};
use push_deck::device::push3::DecodedPadInputMessage;
use push_deck::device::{
    DeviceDiscoveryError, DeviceDiscoverySource, PushModeEvent, StartupDiscoverySource,
};
use push_deck::macos::{ActionBackend, MacosError};
use push_deck::device::output::{Push3LedBackend, Push3LedError};
use push_deck::{
    handle_push_mode_event_with, handle_runtime_pad_input_message, refresh_runtime_with_fallback,
    run_on_background_thread, should_hide_on_close,
};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

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
    assert_eq!(
        runtime_state.capabilities.shortcut,
        push_deck::app_state::ShortcutCapabilityState::Available
    );
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

    let failed = host
        .load_current_config()
        .expect("load should still succeed");
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
fn main_window_close_requests_are_hidden_instead_of_closed() {
    assert!(should_hide_on_close("main"));
    assert!(!should_hide_on_close("preferences"));
}

#[test]
fn discovery_backend_failures_fall_back_to_waiting_for_device() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should succeed");

    refresh_runtime_with_fallback(
        &host,
        &TestDiscoverySource::failing("system_profiler unavailable"),
        &TestDiscoverySource::waiting(),
    )
    .expect("fallback refresh should keep startup alive");

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

    assert_eq!(device_name, None);
    assert!(!device_connected);
    assert_eq!(runtime_state.app_state, AppState::WaitingForDevice);
}

#[test]
fn startup_discovery_uses_system_profiler_after_coremidi_returns_no_devices() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let host = CommandHost::bootstrap(store, TestActionBackend::default())
        .expect("bootstrap should succeed");

    let startup = StartupDiscoverySource::new(
        TestDiscoverySource::waiting(),
        TestDiscoverySource::connected("Ableton Push 3 User Port"),
    );

    refresh_runtime_with_fallback(&host, &startup, &TestDiscoverySource::waiting())
        .expect("startup discovery should succeed");

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

    assert_eq!(device_name.as_deref(), Some("Ableton Push 3 User Port"));
    assert!(device_connected);
    assert_eq!(runtime_state.app_state, AppState::Ready);
}

#[test]
fn runtime_pad_press_dispatches_the_bound_action() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let action_backend = TestActionBackend::default();
    let host = CommandHost::bootstrap(store, action_backend.clone())
        .expect("bootstrap should succeed");
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

    let app = tauri::test::mock_app();

    handle_runtime_pad_input_message(
        &app.handle(),
        &host,
        DecodedPadInputMessage::PadPressed {
            pad_id: "r0c0".to_string(),
            velocity: 0x40,
        },
    )
    .expect("pad press should be handled");

    assert_eq!(
        action_backend.launch_calls(),
        vec!["com.apple.Terminal".to_string()]
    );
}

#[test]
fn user_mode_button_press_triggers_fast_resume_and_led_resync() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");
    let app = tauri::test::mock_app();
    let resume_calls = Arc::new(Mutex::new(0usize));
    let fallback_calls = Arc::new(Mutex::new(0usize));

    handle_push_mode_event_with(
        &app.handle(),
        &host,
        PushModeEvent::UserModeButtonPressed,
        {
            let resume_calls = resume_calls.clone();
            move |_| {
                *resume_calls.lock().expect("lock resume calls") += 1;
                Ok(true)
            }
        },
        {
            let fallback_calls = fallback_calls.clone();
            move || {
                *fallback_calls.lock().expect("lock fallback calls") += 1;
                Ok(())
            }
        },
    )
    .expect("fast resume should succeed");

    assert_eq!(*resume_calls.lock().expect("lock resume calls"), 1);
    assert_eq!(*fallback_calls.lock().expect("lock fallback calls"), 0);
    assert_eq!(led_backend.synced_configs().len(), 1);
    assert_eq!(led_backend.synced_configs()[0], Config::default());
}

#[test]
fn failed_fast_resume_falls_back_without_panicking() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");
    let app = tauri::test::mock_app();
    let fallback_calls = Arc::new(Mutex::new(0usize));

    handle_push_mode_event_with(
        &app.handle(),
        &host,
        PushModeEvent::UserModeEntered,
        |_| Err("user port unavailable".to_string()),
        {
            let fallback_calls = fallback_calls.clone();
            move || {
                *fallback_calls.lock().expect("lock fallback calls") += 1;
                Ok(())
            }
        },
    )
    .expect("fallback should keep execution alive");

    assert_eq!(*fallback_calls.lock().expect("lock fallback calls"), 1);
    assert!(led_backend.synced_configs().is_empty());
}

#[test]
fn user_mode_button_press_retries_fast_resume_before_fallback() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");
    let app = tauri::test::mock_app();
    let attempts = Arc::new(Mutex::new(0usize));
    let fallback_calls = Arc::new(Mutex::new(0usize));

    handle_push_mode_event_with(
        &app.handle(),
        &host,
        PushModeEvent::UserModeButtonPressed,
        {
            let attempts = attempts.clone();
            move |_| {
                let mut attempts = attempts.lock().expect("lock attempts");
                *attempts += 1;
                Ok(*attempts >= 3)
            }
        },
        {
            let fallback_calls = fallback_calls.clone();
            move || {
                *fallback_calls.lock().expect("lock fallback calls") += 1;
                Ok(())
            }
        },
    )
    .expect("event handler should succeed");

    assert_eq!(*attempts.lock().expect("lock attempts"), 3);
    assert_eq!(*fallback_calls.lock().expect("lock fallback calls"), 0);
    assert_eq!(led_backend.synced_configs().len(), 1);
}

#[test]
fn background_task_dispatch_returns_before_the_task_finishes() {
    let (started_tx, started_rx) = mpsc::channel();
    let (finished_tx, finished_rx) = mpsc::channel();
    let started_at = Instant::now();

    let handle = run_on_background_thread(move || {
        started_tx.send(()).expect("task should signal start");
        std::thread::sleep(Duration::from_millis(150));
        finished_tx.send(()).expect("task should signal finish");
    });

    started_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("task should start promptly");
    assert!(
        finished_rx.recv_timeout(Duration::from_millis(25)).is_err(),
        "background task should still be running while the caller continues",
    );
    assert!(
        started_at.elapsed() < Duration::from_millis(100),
        "dispatch should return without waiting for task completion",
    );

    handle.join().expect("background task should finish cleanly");
    finished_rx
        .recv_timeout(Duration::from_millis(50))
        .expect("task should eventually finish");
}

#[test]
fn update_pad_binding_syncs_the_saved_config_to_the_led_backend() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");

    host.update_pad_binding(UpdatePadBindingRequest {
        pad_id: "r0c0".to_string(),
        binding: PadBinding {
            pad_id: "r0c0".to_string(),
            label: "Terminal".to_string(),
            color: PadColorId::Green,
            action: PadAction::launch_or_focus_app("com.apple.Terminal", "Terminal"),
        },
    })
    .expect("update should succeed");

    assert_eq!(led_backend.synced_configs().len(), 1);
    assert_eq!(
        led_backend.synced_configs()[0]
            .profile("default")
            .expect("default profile")
            .pads[0]
            .color,
        PadColorId::Green
    );
}

#[test]
fn refresh_runtime_syncs_leds_when_a_push_device_is_connected() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");

    host.refresh_runtime(&TestDiscoverySource::connected("Ableton Push 3 User Port"))
        .expect("runtime refresh should succeed");

    assert_eq!(led_backend.synced_configs().len(), 1);
    assert_eq!(led_backend.disconnect_calls(), 0);
}

#[test]
fn refresh_runtime_drops_led_connection_when_the_push_disconnects() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");

    host.refresh_runtime(&TestDiscoverySource::connected("Ableton Push 3 User Port"))
        .expect("runtime refresh should succeed");
    host.refresh_runtime(&TestDiscoverySource::waiting())
        .expect("runtime refresh should succeed");

    assert_eq!(led_backend.synced_configs().len(), 1);
    assert_eq!(led_backend.disconnect_calls(), 1);
}

#[test]
fn update_push3_color_calibration_syncs_the_saved_config_to_the_led_backend() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");

    host.update_push3_color_calibration(UpdatePush3ColorCalibrationRequest {
        logical_color: PadColorId::Red,
        output_value: 9,
    })
    .expect("calibration update should succeed");

    assert_eq!(led_backend.synced_configs().len(), 1);
    assert_eq!(
        led_backend.synced_configs()[0]
            .settings
            .push3_color_calibration
            .red,
        9
    );
}

#[test]
fn preview_push3_palette_forwards_the_selected_page_to_the_led_backend() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");

    host.preview_push3_palette(1)
        .expect("preview should succeed");

    assert_eq!(led_backend.preview_pages(), vec![1]);
}

#[test]
fn sync_push3_leds_resends_the_current_layout_to_the_led_backend() {
    let backend = TestConfigStoreBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend);
    let led_backend = TestLedBackend::default();
    let host = CommandHost::bootstrap_with_led_backend(
        store,
        TestActionBackend::default(),
        led_backend.clone(),
    )
    .expect("bootstrap should succeed");

    host.sync_push3_leds().expect("sync should succeed");

    assert_eq!(led_backend.synced_configs().len(), 1);
    assert_eq!(
        led_backend.synced_configs()[0],
        Config::default()
    );
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

    fn failing(message: &str) -> Self {
        Self {
            response: Err(DeviceDiscoveryError::backend(message)),
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
    launch_calls: Vec<String>,
}

impl Default for TestActionBackendState {
    fn default() -> Self {
        Self {
            shortcut_capability: true,
            launch_calls: vec![],
        }
    }
}

impl TestActionBackend {
    fn with_shortcut_capability(self, is_available: bool) -> Self {
        self.state.lock().expect("lock state").shortcut_capability = is_available;
        self
    }

    fn launch_calls(&self) -> Vec<String> {
        self.state.lock().expect("lock state").launch_calls.clone()
    }
}

impl ActionBackend for TestActionBackend {
    fn launch_or_focus_bundle_id(&self, bundle_id: &str) -> Result<(), MacosError> {
        self.state
            .lock()
            .expect("lock state")
            .launch_calls
            .push(bundle_id.to_string());
        Ok(())
    }

    fn shortcut_accessibility_available(&self) -> Result<bool, MacosError> {
        Ok(self.state.lock().expect("lock state").shortcut_capability)
    }
}

#[derive(Clone, Default)]
struct TestLedBackend {
    state: Arc<Mutex<TestLedBackendState>>,
}

#[derive(Default)]
struct TestLedBackendState {
    synced_configs: Vec<Config>,
    preview_pages: Vec<u8>,
    disconnect_calls: usize,
}

impl TestLedBackend {
    fn synced_configs(&self) -> Vec<Config> {
        self.state.lock().expect("lock state").synced_configs.clone()
    }

    fn disconnect_calls(&self) -> usize {
        self.state.lock().expect("lock state").disconnect_calls
    }

    fn preview_pages(&self) -> Vec<u8> {
        self.state.lock().expect("lock state").preview_pages.clone()
    }
}

impl Push3LedBackend for TestLedBackend {
    fn sync_config(&self, config: &Config) -> Result<(), Push3LedError> {
        self.state
            .lock()
            .expect("lock state")
            .synced_configs
            .push(config.clone());
        Ok(())
    }

    fn preview_palette(&self, page: u8) -> Result<(), Push3LedError> {
        self.state
            .lock()
            .expect("lock state")
            .preview_pages
            .push(page);
        Ok(())
    }

    fn disconnect(&self) {
        self.state.lock().expect("lock state").disconnect_calls += 1;
    }
}

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}
