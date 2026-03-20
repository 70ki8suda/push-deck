use push_deck::app_state::{ConfigLoadState, ConfigRecoveryState};
use push_deck::config::schema::Config;
use push_deck::config::store::{ConfigStore, ConfigStoreBackend, ConfigStoreError};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

#[test]
fn missing_config_file_is_initialized_with_default_and_written_to_disk() {
    let backend = TestBackend::default();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());

    let result = store.load().expect("missing config should initialize");

    assert_eq!(result.state, ConfigLoadState::CreatedDefault);
    assert_eq!(result.config, Config::default());
    assert_eq!(
        backend.file_contents(&path("config.json")),
        Some(
            serde_json::to_string_pretty(&Config::default())
                .expect("default config should serialize")
        )
    );
}

#[test]
fn valid_config_file_loads_without_recovery() {
    let backend = TestBackend::default();
    let config = Config::default();
    backend.write_existing(
        path("config.json"),
        &serde_json::to_string_pretty(&config).unwrap(),
    );
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());

    let result = store.load().expect("valid config should load");

    assert_eq!(result.state, ConfigLoadState::Loaded);
    assert_eq!(result.config, config);
}

#[test]
fn broken_json_is_moved_to_timestamped_backup_and_reports_recovery() {
    let backend = TestBackend::default();
    backend.write_existing(path("config.json"), "{ not json");
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());

    let result = store.load().expect("broken config should enter recovery");

    let ConfigLoadState::RecoveryRequired(recovery) = result.state else {
        panic!("expected recovery state");
    };

    assert_eq!(
        recovery,
        ConfigRecoveryState {
            config_path: path("config.json"),
            backup_path: path("config.broken-1700000000000.json"),
            reason: "failed to parse config json".to_string(),
        }
    );
    assert!(backend.file_contents(&path("config.json")).is_none());
    assert_eq!(
        backend.file_contents(&path("config.broken-1700000000000.json")),
        Some("{ not json".to_string())
    );
}

#[test]
fn atomic_save_failure_keeps_existing_file_contents_unchanged() {
    let backend = TestBackend::default();
    backend.write_existing(path("config.json"), "previous contents");
    backend.fail_next_rename();
    let store = ConfigStore::with_backend(path("config.json"), backend.clone());

    let error = store
        .save(&Config::default())
        .expect_err("rename failure should surface");

    assert_eq!(error, ConfigStoreError::AtomicSaveFailed);
    assert_eq!(
        backend.file_contents(&path("config.json")),
        Some("previous contents".to_string())
    );
}

#[test]
fn default_path_uses_the_macos_application_support_location() {
    let path = ConfigStore::config_path_from_home("/Users/tester");

    assert_eq!(
        path,
        PathBuf::from("/Users/tester/Library/Application Support/push-deck/config.json")
    );
}

#[derive(Clone, Default)]
struct TestBackend {
    state: Arc<Mutex<TestBackendState>>,
}

#[derive(Default)]
struct TestBackendState {
    files: HashMap<PathBuf, String>,
    fail_next_rename: bool,
}

impl TestBackend {
    fn write_existing(&self, path: PathBuf, contents: &str) {
        self.state
            .lock()
            .expect("lock state")
            .files
            .insert(path, contents.to_string());
    }

    fn fail_next_rename(&self) {
        self.state.lock().expect("lock state").fail_next_rename = true;
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

impl ConfigStoreBackend for TestBackend {
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
        if state.fail_next_rename {
            state.fail_next_rename = false;
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "rename failed",
            ));
        }

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

fn path(name: &str) -> PathBuf {
    PathBuf::from(name)
}
