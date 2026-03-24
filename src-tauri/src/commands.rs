use crate::actions::{dispatch_pad_action, send_shortcut::shortcut_capability_state, ActionExecutionError};
use crate::app_state::{
    recorded_shortcut_capability, AppState, ConfigRecoveryState, RuntimeState,
};
use crate::config::schema::{Config, PadAction, PadBinding, PadColorId, DEFAULT_PROFILE_ID};
use crate::config::store::{ConfigLoadOutcome, ConfigStore, ConfigStoreBackend, ConfigStoreError};
use crate::device::{
    discover_push_device, CoreMidiDiscoverySource, DeviceDiscoverySource, StartupDiscoverySource,
    SystemDiscoverySource,
};
use crate::device::{NoopPush3LedBackend, Push3LedBackend, SystemPush3LedBackend};
use crate::macos::{ActionBackend, RunningAppOption, SystemMacosBackend};
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::sync::Mutex;

pub type DefaultCommandHost = CommandHost<
    crate::config::store::OsConfigStoreBackend,
    SystemMacosBackend,
    SystemPush3LedBackend,
>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum CurrentConfigResponse {
    Ready {
        config: Config,
        device_name: Option<String>,
        device_connected: bool,
        runtime_state: RuntimeState,
    },
    RecoveryRequired {
        recovery: ConfigRecoveryState,
        device_name: Option<String>,
        device_connected: bool,
        runtime_state: RuntimeState,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePadBindingRequest {
    pub pad_id: String,
    pub binding: PadBinding,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePadBindingResponse {
    pub config: Config,
    pub runtime_state: RuntimeState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePush3ColorCalibrationRequest {
    pub logical_color: PadColorId,
    pub output_value: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UpdatePush3ColorCalibrationResponse {
    pub config: Config,
    pub runtime_state: RuntimeState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PreviewPush3PaletteRequest {
    pub page: u8,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TestActionResponse {
    pub runtime_state: RuntimeState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RestoreDefaultConfigResponse {
    pub config: Config,
    pub runtime_state: RuntimeState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommandError {
    ConfigStore(String),
    RecoveryRequired,
    NotInRecoveryMode,
    InvalidPadBinding { pad_id: String },
    PadNotFound { pad_id: String },
    UnassignedPad { pad_id: String },
    Action(String),
}

impl Display for CommandError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ConfigStore(message) => write!(f, "config store error: {message}"),
            Self::RecoveryRequired => f.write_str("config recovery required"),
            Self::NotInRecoveryMode => f.write_str("not in recovery mode"),
            Self::InvalidPadBinding { pad_id } => {
                write!(f, "pad binding pad_id mismatch: {pad_id}")
            }
            Self::PadNotFound { pad_id } => write!(f, "pad not found: {pad_id}"),
            Self::UnassignedPad { pad_id } => write!(f, "pad is unassigned: {pad_id}"),
            Self::Action(message) => write!(f, "action error: {message}"),
        }
    }
}

impl Error for CommandError {}

impl From<ConfigStoreError> for CommandError {
    fn from(error: ConfigStoreError) -> Self {
        Self::ConfigStore(error.to_string())
    }
}

impl From<ActionExecutionError> for CommandError {
    fn from(error: ActionExecutionError) -> Self {
        Self::Action(error.to_string())
    }
}

#[derive(Debug)]
struct CommandState {
    config: Option<Config>,
    recovery: Option<ConfigRecoveryState>,
    device_name: Option<String>,
    device_connected: bool,
    runtime_state: RuntimeState,
    stable_app_state: AppState,
}

impl CommandState {
    fn new_ready() -> Self {
        let app_state = AppState::WaitingForDevice;
        Self {
            config: Some(Config::default()),
            recovery: None,
            device_name: None,
            device_connected: false,
            runtime_state: RuntimeState::new(app_state, recorded_shortcut_capability()),
            stable_app_state: app_state,
        }
    }

    fn new_recovery(recovery: ConfigRecoveryState) -> Self {
        let app_state = AppState::ConfigRecoveryRequired;
        Self {
            config: None,
            recovery: Some(recovery),
            device_name: None,
            device_connected: false,
            runtime_state: RuntimeState::new(app_state, recorded_shortcut_capability()),
            stable_app_state: app_state,
        }
    }

    fn runtime_snapshot(&self) -> RuntimeState {
        self.runtime_state.clone()
    }

    fn set_config(&mut self, config: Config) {
        self.config = Some(config);
        self.recovery = None;
        self.runtime_state.app_state = self.stable_app_state;
    }

    fn apply_runtime_environment(
        &mut self,
        app_state: AppState,
        shortcut_capability: crate::app_state::ShortcutCapabilityState,
        device_name: Option<String>,
        device_connected: bool,
    ) {
        self.stable_app_state = if self.recovery.is_some() {
            AppState::ConfigRecoveryRequired
        } else {
            app_state
        };
        self.device_name = device_name;
        self.device_connected = device_connected;
        self.runtime_state.capabilities.shortcut = shortcut_capability;

        if self.runtime_state.app_state != AppState::SaveFailed {
            self.runtime_state.app_state = self.stable_app_state;
        }
    }

    fn enter_save_failed(&mut self) {
        self.runtime_state.app_state = AppState::SaveFailed;
    }

    fn clear_save_failed(&mut self) {
        self.runtime_state.app_state = self.stable_app_state;
    }
}

pub struct CommandHost<
    S = crate::config::store::OsConfigStoreBackend,
    A = SystemMacosBackend,
    L = NoopPush3LedBackend,
>
where
    S: ConfigStoreBackend,
    A: ActionBackend,
    L: Push3LedBackend,
{
    store: ConfigStore<S>,
    action_backend: A,
    led_backend: L,
    state: Mutex<CommandState>,
}

impl CommandHost<
    crate::config::store::OsConfigStoreBackend,
    SystemMacosBackend,
    SystemPush3LedBackend,
> {
    pub fn bootstrap_default() -> Result<Self, CommandError> {
        let path = ConfigStore::default_path().map_err(CommandError::from)?;
        let store = ConfigStore::new(path);
        Self::bootstrap_with_led_backend(
            store,
            SystemMacosBackend::default(),
            SystemPush3LedBackend::default(),
        )
    }
}

impl<S, A> CommandHost<S, A, NoopPush3LedBackend>
where
    S: ConfigStoreBackend,
    A: ActionBackend,
{
    pub fn bootstrap(store: ConfigStore<S>, action_backend: A) -> Result<Self, CommandError> {
        Self::bootstrap_with_led_backend(store, action_backend, NoopPush3LedBackend)
    }
}

impl<S, A, L> CommandHost<S, A, L>
where
    S: ConfigStoreBackend,
    A: ActionBackend,
    L: Push3LedBackend,
{
    pub fn bootstrap_with_led_backend(
        store: ConfigStore<S>,
        action_backend: A,
        led_backend: L,
    ) -> Result<Self, CommandError> {
        let state = match store.load().map_err(CommandError::from)? {
            ConfigLoadOutcome::Ready(result) => {
                let mut state = CommandState::new_ready();
                state.set_config(result.config);
                state
            }
            ConfigLoadOutcome::RecoveryRequired(recovery) => CommandState::new_recovery(recovery),
        };

        Ok(Self {
            store,
            action_backend,
            led_backend,
            state: Mutex::new(state),
        })
    }

    pub fn load_current_config(&self) -> Result<CurrentConfigResponse, CommandError> {
        let state = self.state.lock().expect("command state lock poisoned");
        match (&state.config, &state.recovery) {
            (Some(config), None) => Ok(CurrentConfigResponse::Ready {
                config: config.clone(),
                device_name: state.device_name.clone(),
                device_connected: state.device_connected,
                runtime_state: state.runtime_snapshot(),
            }),
            (None, Some(recovery)) => Ok(CurrentConfigResponse::RecoveryRequired {
                recovery: recovery.clone(),
                device_name: state.device_name.clone(),
                device_connected: state.device_connected,
                runtime_state: state.runtime_snapshot(),
            }),
            _ => Err(CommandError::RecoveryRequired),
        }
    }

    pub fn refresh_runtime<D>(&self, discovery_source: &D) -> Result<RuntimeState, CommandError>
    where
        D: DeviceDiscoverySource,
    {
        let discovery = discover_push_device(
            discovery_source
                .discover_devices()
                .map_err(|error| CommandError::Action(format!("device discovery error: {error:?}")))?,
        );
        let shortcut_capability =
            shortcut_capability_state(&self.action_backend).map_err(|error| {
                CommandError::Action(format!("shortcut capability error: {error}"))
            })?;

        let mut state = self.state.lock().expect("command state lock poisoned");
        let device_name = discovery
            .connection
            .endpoint()
            .map(|endpoint| endpoint.display_name.clone());
        let device_connected = device_name.is_some();
        let config = state.config.clone();

        state.apply_runtime_environment(
            discovery.app_state,
            shortcut_capability,
            device_name,
            device_connected,
        );

        let runtime_state = state.runtime_snapshot();
        drop(state);

        if device_connected {
            if let Some(config) = config.as_ref() {
                self.try_sync_leds(config);
            } else {
                self.led_backend.disconnect();
            }
        } else {
            self.led_backend.disconnect();
        }

        Ok(runtime_state)
    }

    pub fn update_pad_binding(
        &self,
        request: UpdatePadBindingRequest,
    ) -> Result<UpdatePadBindingResponse, CommandError> {
        let mut state = self.state.lock().expect("command state lock poisoned");
        if state.recovery.is_some() {
            return Err(CommandError::RecoveryRequired);
        }

        let current_config = state
            .config
            .clone()
            .ok_or(CommandError::RecoveryRequired)?;
        let updated_config = update_binding(current_config, &request)?;

        match self.store.save(&updated_config) {
            Ok(()) => {
                state.clear_save_failed();
                state.set_config(updated_config.clone());
                let runtime_state = state.runtime_snapshot();
                drop(state);
                self.try_sync_leds(&updated_config);
                Ok(UpdatePadBindingResponse {
                    config: updated_config,
                    runtime_state,
                })
            }
            Err(error) => {
                state.enter_save_failed();
                Err(error.into())
            }
        }
    }

    pub fn trigger_test_action(&self, pad_id: &str) -> Result<TestActionResponse, CommandError> {
        let mut state = self.state.lock().expect("command state lock poisoned");
        if state.recovery.is_some() {
            return Err(CommandError::RecoveryRequired);
        }

        let config = state
            .config
            .as_ref()
            .ok_or(CommandError::RecoveryRequired)?;
        let binding = find_pad_binding(config, pad_id)?;

        match binding.action {
            PadAction::Unassigned => Err(CommandError::UnassignedPad {
                pad_id: pad_id.to_string(),
            }),
            _ => {
                dispatch_pad_action(&self.action_backend, &binding.action)?;
                state.runtime_state.capabilities.shortcut = recorded_shortcut_capability();
                Ok(TestActionResponse {
                    runtime_state: state.runtime_snapshot(),
                })
            }
        }
    }

    pub fn dispatch_pad_press(&self, pad_id: &str) -> Result<(), CommandError> {
        let binding = {
            let state = self.state.lock().expect("command state lock poisoned");

            if state.recovery.is_some() {
                return Ok(());
            }

            let Some(config) = state.config.as_ref() else {
                return Ok(());
            };

            find_pad_binding(config, pad_id)?.clone()
        };

        if matches!(binding.action, PadAction::Unassigned) {
            return Ok(());
        }

        dispatch_pad_action(&self.action_backend, &binding.action)?;

        let mut state = self.state.lock().expect("command state lock poisoned");
        state.runtime_state.capabilities.shortcut = recorded_shortcut_capability();

        Ok(())
    }

    pub fn list_running_apps(&self) -> Result<Vec<RunningAppOption>, CommandError> {
        let mut apps = self
            .action_backend
            .running_apps()
            .map_err(|error| CommandError::Action(error.to_string()))?;

        apps.sort_by(|left, right| {
            left.app_name
                .to_ascii_lowercase()
                .cmp(&right.app_name.to_ascii_lowercase())
                .then_with(|| left.bundle_id.cmp(&right.bundle_id))
        });

        apps.dedup_by(|left, right| left.bundle_id == right.bundle_id);

        Ok(apps)
    }

    pub fn update_push3_color_calibration(
        &self,
        request: UpdatePush3ColorCalibrationRequest,
    ) -> Result<UpdatePush3ColorCalibrationResponse, CommandError> {
        let mut state = self.state.lock().expect("command state lock poisoned");
        if state.recovery.is_some() {
            return Err(CommandError::RecoveryRequired);
        }

        let mut updated_config = state
            .config
            .clone()
            .ok_or(CommandError::RecoveryRequired)?;
        updated_config
            .settings
            .push3_color_calibration
            .update(request.logical_color, request.output_value);

        match self.store.save(&updated_config) {
            Ok(()) => {
                state.clear_save_failed();
                state.set_config(updated_config.clone());
                let runtime_state = state.runtime_snapshot();
                drop(state);
                self.try_sync_leds(&updated_config);
                Ok(UpdatePush3ColorCalibrationResponse {
                    config: updated_config,
                    runtime_state,
                })
            }
            Err(error) => {
                state.enter_save_failed();
                Err(error.into())
            }
        }
    }

    pub fn restore_default_config(&self) -> Result<RestoreDefaultConfigResponse, CommandError> {
        let mut state = self.state.lock().expect("command state lock poisoned");
        let recovery = state
            .recovery
            .clone()
            .ok_or(CommandError::NotInRecoveryMode)?;
        let default_config = Config::default();

        match self.store.save(&default_config) {
            Ok(()) => {
                state.set_config(default_config.clone());
                state.stable_app_state = AppState::WaitingForDevice;
                state.runtime_state.app_state = AppState::WaitingForDevice;
                state.runtime_state.capabilities.shortcut = recorded_shortcut_capability();
                state.recovery = None;
                let runtime_state = state.runtime_snapshot();
                drop(state);
                self.try_sync_leds(&default_config);
                Ok(RestoreDefaultConfigResponse {
                    config: default_config,
                    runtime_state,
                })
            }
            Err(error) => {
                state.recovery = Some(recovery);
                state.enter_save_failed();
                Err(error.into())
            }
        }
    }

    fn try_sync_leds(&self, config: &Config) {
        if let Err(error) = self.led_backend.sync_config(config) {
            eprintln!("push3 led sync failed: {error}");
        }
    }

    pub fn preview_push3_palette(&self, page: u8) -> Result<(), CommandError> {
        self.led_backend
            .preview_palette(page)
            .map_err(|error| CommandError::Action(error.to_string()))
    }

    pub fn sync_push3_leds(&self) -> Result<(), CommandError> {
        let config = {
            let state = self.state.lock().expect("command state lock poisoned");
            state.config.clone()
        };

        if let Some(config) = config.as_ref() {
            self.try_sync_leds(config);
        }

        Ok(())
    }
}

fn update_binding(
    current_config: Config,
    request: &UpdatePadBindingRequest,
) -> Result<Config, CommandError> {
    if request.pad_id != request.binding.pad_id {
        return Err(CommandError::InvalidPadBinding {
            pad_id: request.pad_id.clone(),
        });
    }

    let mut profiles = current_config.profiles.clone();
    let profile_index = profiles
        .iter()
        .position(|profile| profile.id == current_config.settings.active_profile_id)
        .unwrap_or(0);
    let profile = profiles
        .get_mut(profile_index)
        .expect("default config always has one profile");

    let pad_index = profile
        .pads
        .iter()
        .position(|pad| pad.pad_id == request.pad_id)
        .ok_or_else(|| CommandError::PadNotFound {
            pad_id: request.pad_id.clone(),
        })?;

    profile.pads[pad_index] = request.binding.clone();

    Config::from_parts(current_config.settings.clone(), profiles)
        .map_err(|error| CommandError::ConfigStore(error.to_string()))
}

fn find_pad_binding<'a>(config: &'a Config, pad_id: &str) -> Result<&'a PadBinding, CommandError> {
    let profile = config
        .profile(&config.settings.active_profile_id)
        .or_else(|| config.profile(DEFAULT_PROFILE_ID))
        .ok_or_else(|| CommandError::PadNotFound {
            pad_id: pad_id.to_string(),
        })?;

    profile
        .pads
        .iter()
        .find(|pad| pad.pad_id == pad_id)
        .ok_or_else(|| CommandError::PadNotFound {
            pad_id: pad_id.to_string(),
        })
}

#[tauri::command]
pub fn load_current_config(
    state: tauri::State<'_, DefaultCommandHost>,
) -> Result<CurrentConfigResponse, String> {
    state.load_current_config().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn refresh_runtime_state(
    state: tauri::State<'_, DefaultCommandHost>,
) -> Result<CurrentConfigResponse, String> {
    let startup_discovery =
        StartupDiscoverySource::new(CoreMidiDiscoverySource, SystemDiscoverySource);

    if let Err(error) = state.refresh_runtime(&startup_discovery) {
        eprintln!("device discovery unavailable during refresh: {error}");
        state
            .refresh_runtime(&SystemDiscoverySource)
            .map_err(|inner| inner.to_string())?;
    }

    state.load_current_config().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_running_apps(
    state: tauri::State<'_, DefaultCommandHost>,
) -> Result<Vec<RunningAppOption>, String> {
    state.list_running_apps().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_pad_binding(
    state: tauri::State<'_, DefaultCommandHost>,
    request: UpdatePadBindingRequest,
) -> Result<UpdatePadBindingResponse, String> {
    state
        .update_pad_binding(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn trigger_test_action(
    state: tauri::State<'_, DefaultCommandHost>,
    pad_id: String,
) -> Result<TestActionResponse, String> {
    state
        .trigger_test_action(&pad_id)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn restore_default_config(
    state: tauri::State<'_, DefaultCommandHost>,
) -> Result<RestoreDefaultConfigResponse, String> {
    state
        .restore_default_config()
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn update_push3_color_calibration(
    state: tauri::State<'_, DefaultCommandHost>,
    request: UpdatePush3ColorCalibrationRequest,
) -> Result<UpdatePush3ColorCalibrationResponse, String> {
    state
        .update_push3_color_calibration(request)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn preview_push3_palette(
    state: tauri::State<'_, DefaultCommandHost>,
    request: PreviewPush3PaletteRequest,
) -> Result<(), String> {
    state
        .preview_push3_palette(request.page)
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn sync_push3_leds(state: tauri::State<'_, DefaultCommandHost>) -> Result<(), String> {
    state.sync_push3_leds().map_err(|error| error.to_string())
}
