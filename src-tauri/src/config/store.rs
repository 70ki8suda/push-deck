use crate::app_state::{ConfigLoadState, ConfigRecoveryState};
use crate::config::schema::{Config, SCHEMA_VERSION};
use std::ffi::OsStr;
use std::fmt::{Display, Formatter};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigLoadResult {
    pub config: Config,
    pub state: ConfigLoadState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConfigStoreError {
    Io(String),
    Parse(String),
    InvalidSchemaVersion(u32),
    InvalidConfig(String),
    AtomicSaveFailed,
    BackupFailed,
    MissingHomeDirectory,
}

impl Display for ConfigStoreError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Io(message) => write!(f, "io error: {message}"),
            Self::Parse(message) => write!(f, "parse error: {message}"),
            Self::InvalidSchemaVersion(version) => {
                write!(f, "unsupported schema version: {version}")
            }
            Self::InvalidConfig(message) => write!(f, "invalid config: {message}"),
            Self::AtomicSaveFailed => f.write_str("atomic save failed"),
            Self::BackupFailed => f.write_str("backup failed"),
            Self::MissingHomeDirectory => f.write_str("missing HOME directory"),
        }
    }
}

impl std::error::Error for ConfigStoreError {}

pub trait ConfigStoreBackend {
    fn read_to_string(&self, path: &Path) -> io::Result<String>;
    fn write_string(&self, path: &Path, contents: &str) -> io::Result<()>;
    fn rename(&self, from: &Path, to: &Path) -> io::Result<()>;
    fn create_dir_all(&self, path: &Path) -> io::Result<()>;
    fn remove_file(&self, path: &Path) -> io::Result<()>;
    fn timestamp_millis(&self) -> u128;
}

#[derive(Debug, Clone, Default)]
pub struct OsConfigStoreBackend;

impl ConfigStoreBackend for OsConfigStoreBackend {
    fn read_to_string(&self, path: &Path) -> io::Result<String> {
        fs::read_to_string(path)
    }

    fn write_string(&self, path: &Path, contents: &str) -> io::Result<()> {
        fs::write(path, contents)
    }

    fn rename(&self, from: &Path, to: &Path) -> io::Result<()> {
        fs::rename(from, to)
    }

    fn create_dir_all(&self, path: &Path) -> io::Result<()> {
        fs::create_dir_all(path)
    }

    fn remove_file(&self, path: &Path) -> io::Result<()> {
        fs::remove_file(path)
    }

    fn timestamp_millis(&self) -> u128 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time should be after unix epoch")
            .as_millis()
    }
}

#[derive(Debug, Clone)]
pub struct ConfigStore<B = OsConfigStoreBackend> {
    path: PathBuf,
    backend: B,
}

impl ConfigStore<OsConfigStoreBackend> {
    pub fn new(path: impl Into<PathBuf>) -> Self {
        Self {
            path: path.into(),
            backend: OsConfigStoreBackend,
        }
    }

    pub fn default_path() -> Result<PathBuf, ConfigStoreError> {
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .ok_or(ConfigStoreError::MissingHomeDirectory)?;
        Ok(Self::config_path_from_home(home))
    }

    pub fn config_path_from_home(home_dir: impl AsRef<Path>) -> PathBuf {
        home_dir
            .as_ref()
            .join("Library")
            .join("Application Support")
            .join("push-deck")
            .join("config.json")
    }
}

impl<B: ConfigStoreBackend> ConfigStore<B> {
    pub fn with_backend(path: impl Into<PathBuf>, backend: B) -> Self {
        Self {
            path: path.into(),
            backend,
        }
    }

    pub fn load(&self) -> Result<ConfigLoadResult, ConfigStoreError> {
        match self.backend.read_to_string(&self.path) {
            Ok(contents) => self.load_from_contents(contents),
            Err(error) if error.kind() == io::ErrorKind::NotFound => {
                let config = Config::default();
                self.save(&config)?;
                Ok(ConfigLoadResult {
                    config,
                    state: ConfigLoadState::CreatedDefault,
                })
            }
            Err(error) => Err(ConfigStoreError::Io(error.to_string())),
        }
    }

    pub fn save(&self, config: &Config) -> Result<(), ConfigStoreError> {
        let serialized = serde_json::to_string_pretty(config)
            .map_err(|error| ConfigStoreError::Parse(error.to_string()))?;
        let temp_path = self.temp_path();

        if let Some(parent) = self
            .path
            .parent()
            .filter(|path| !path.as_os_str().is_empty())
        {
            self.backend
                .create_dir_all(parent)
                .map_err(|error| ConfigStoreError::Io(error.to_string()))?;
        }

        self.backend
            .write_string(&temp_path, &serialized)
            .map_err(|error| ConfigStoreError::Io(error.to_string()))?;

        match self.backend.rename(&temp_path, &self.path) {
            Ok(()) => Ok(()),
            Err(error) => {
                let _ = self.backend.remove_file(&temp_path);
                if error.kind() == io::ErrorKind::NotFound {
                    Err(ConfigStoreError::AtomicSaveFailed)
                } else {
                    Err(ConfigStoreError::AtomicSaveFailed)
                }
            }
        }
    }

    fn load_from_contents(&self, contents: String) -> Result<ConfigLoadResult, ConfigStoreError> {
        let parsed = match serde_json::from_str::<Config>(&contents) {
            Ok(parsed) => parsed,
            Err(_) => {
                return self.enter_recovery("failed to parse config json".to_string());
            }
        };

        if parsed.schema_version != SCHEMA_VERSION {
            return self.enter_recovery(format!(
                "unsupported schema version: {}",
                parsed.schema_version
            ));
        }

        let config = match Config::from_parts(parsed.settings, parsed.profiles) {
            Ok(config) => config,
            Err(error) => {
                return self.enter_recovery(error.to_string());
            }
        };

        Ok(ConfigLoadResult {
            config,
            state: ConfigLoadState::Loaded,
        })
    }

    fn enter_recovery(&self, reason: String) -> Result<ConfigLoadResult, ConfigStoreError> {
        let backup_path = self.broken_backup_path();

        self.backend
            .rename(&self.path, &backup_path)
            .map_err(|error| ConfigStoreError::Io(error.to_string()))?;

        let state = ConfigLoadState::RecoveryRequired(ConfigRecoveryState {
            config_path: self.path.clone(),
            backup_path: backup_path.clone(),
            reason,
        });

        Ok(ConfigLoadResult {
            config: Config::default(),
            state,
        })
    }

    fn temp_path(&self) -> PathBuf {
        let stamp = self.backend.timestamp_millis();
        let file_name = self
            .path
            .file_name()
            .unwrap_or_else(|| OsStr::new("config.json"));
        self.path
            .with_file_name(format!("{}.tmp-{}", file_name.to_string_lossy(), stamp))
    }

    fn broken_backup_path(&self) -> PathBuf {
        let stamp = self.backend.timestamp_millis();
        let file_name = self
            .path
            .file_name()
            .unwrap_or_else(|| OsStr::new("config.json"));
        let file_name = file_name.to_string_lossy();
        let (stem, ext) = split_name_and_extension(&file_name);
        let backup_name = match ext {
            Some(ext) => format!("{stem}.broken-{stamp}.{ext}"),
            None => format!("{stem}.broken-{stamp}"),
        };
        self.path.with_file_name(backup_name)
    }
}

fn split_name_and_extension(file_name: &str) -> (&str, Option<&str>) {
    match file_name.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() && !ext.is_empty() => (stem, Some(ext)),
        _ => (file_name, None),
    }
}
