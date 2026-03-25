extern crate self as push_deck;

pub mod actions;
pub mod app_state;
pub mod commands;
pub mod config;
pub mod device;
pub mod display;
pub mod events;
pub mod macos;

use crate::commands::{CurrentConfigResponse, DefaultCommandHost};
use crate::config::store::ConfigStoreBackend;
use crate::device::SystemDiscoverySource;
use crate::device::{
    emit_decoded_pad_input_event,
    push3::DecodedPadInputMessage,
    subscribe_push3_mode_runtime_events, subscribe_push3_user_port_runtime_events,
    CoreMidiDiscoverySource, DeviceDiscoverySource, Push3InputSubscription, Push3ModeSubscription,
    PushModeEvent, StartupDiscoverySource,
};
use crate::events::{emit_runtime_event, RuntimeEvent};
use crate::macos::ActionBackend;
use std::error::Error;
use std::sync::Mutex;
use tauri::Manager;

#[derive(Debug, Default, Clone, Copy)]
struct NullDiscoverySource;

impl crate::device::DeviceDiscoverySource for NullDiscoverySource {
    fn discover_devices(
        &self,
    ) -> Result<Vec<crate::app_state::DeviceEndpointDescriptor>, crate::device::DeviceDiscoveryError>
    {
        Ok(vec![])
    }
}

pub fn should_hide_on_close(window_label: &str) -> bool {
    window_label == "main"
}

pub fn refresh_runtime_with_fallback<S, A, P, F>(
    host: &commands::CommandHost<S, A, impl crate::device::Push3LedBackend>,
    primary: &P,
    fallback: &F,
) -> Result<(), commands::CommandError>
where
    S: ConfigStoreBackend,
    A: ActionBackend,
    P: DeviceDiscoverySource,
    F: DeviceDiscoverySource,
{
    if let Err(error) = host.refresh_runtime(primary) {
        eprintln!("device discovery unavailable at startup: {error}");
        host.refresh_runtime(fallback)?;
    }

    Ok(())
}

fn emit_runtime_snapshot<R, S, A, L>(
    app: &tauri::AppHandle<R>,
    host: &crate::commands::CommandHost<S, A, L>,
) -> Result<(), Box<dyn Error>>
where
    R: tauri::Runtime,
    S: crate::config::store::ConfigStoreBackend,
    A: crate::macos::ActionBackend,
    L: crate::device::Push3LedBackend,
{
    let response = host.load_current_config()?;

    let (device_name, device_connected, runtime_state) = match response {
        CurrentConfigResponse::Ready {
            device_name,
            device_connected,
            runtime_state,
            ..
        }
        | CurrentConfigResponse::RecoveryRequired {
            device_name,
            device_connected,
            runtime_state,
            ..
        } => (device_name, device_connected, runtime_state),
    };

    emit_runtime_event(
        app,
        RuntimeEvent::StateChanged {
            state: runtime_state,
        },
    )
    .map_err(Box::<dyn Error>::from)?;
    emit_runtime_event(
        app,
        RuntimeEvent::DeviceConnectionChanged {
            connected: device_connected,
            device_name,
        },
    )
    .map_err(Box::<dyn Error>::from)?;

    Ok(())
}

fn store_subscription_state<R, C>(
    app: &tauri::AppHandle<R>,
    connection: Option<C>,
) where
    R: tauri::Runtime,
    C: Send + Sync + 'static,
{
    if let Some(state) = app.try_state::<Mutex<Option<C>>>() {
        *state.lock().expect("subscription state lock poisoned") = connection;
    } else {
        app.manage(Mutex::new(connection));
    }
}

pub fn store_push3_input_subscription<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    subscription: Push3InputSubscription<R>,
) -> bool {
    let connection = match subscription {
        Push3InputSubscription::Active(connection) => Some(connection),
        Push3InputSubscription::NotConnected => None,
    };
    let managed = connection.is_some();
    store_subscription_state(app, connection);
    managed
}

pub fn store_push3_mode_subscription<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    subscription: Push3ModeSubscription,
) -> bool {
    let connection = match subscription {
        Push3ModeSubscription::Active(connection) => Some(connection),
        Push3ModeSubscription::NotConnected => None,
    };
    let managed = connection.is_some();
    store_subscription_state(app, connection);
    managed
}

fn fast_resume_push3_input_subscription<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
) -> Result<bool, String> {
    let subscription = subscribe_push3_user_port_runtime_events(app)?;
    Ok(store_push3_input_subscription(app, subscription))
}

pub fn handle_push_mode_event_with<R, RF, FF>(
    app: &tauri::AppHandle<R>,
    host: &crate::commands::CommandHost<
        impl crate::config::store::ConfigStoreBackend,
        impl crate::macos::ActionBackend,
        impl crate::device::Push3LedBackend,
    >,
    event: PushModeEvent,
    mut fast_resume: RF,
    mut fallback_refresh: FF,
) -> Result<(), String>
where
    R: tauri::Runtime,
    RF: FnMut(&tauri::AppHandle<R>) -> Result<bool, String>,
    FF: FnMut() -> Result<(), String>,
{
    match event {
        PushModeEvent::UserModeButtonPressed | PushModeEvent::UserModeEntered => {
            match fast_resume(app) {
                Ok(true) => {
                    host.sync_push3_leds().map_err(|error| error.to_string())?;
                    emit_runtime_snapshot(app, host).map_err(|error| error.to_string())?;
                }
                Ok(false) => fallback_refresh()?,
                Err(error) => {
                    eprintln!("push3 fast resume failed: {error}");
                    fallback_refresh()?;
                }
            }
        }
        PushModeEvent::UserModeButtonReleased | PushModeEvent::UserModeExited => {}
    }

    Ok(())
}

pub fn handle_push_mode_event<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    host: &crate::commands::CommandHost<
        impl crate::config::store::ConfigStoreBackend,
        impl crate::macos::ActionBackend,
        impl crate::device::Push3LedBackend,
    >,
    event: PushModeEvent,
) -> Result<(), String> {
    handle_push_mode_event_with(
        app,
        host,
        event,
        fast_resume_push3_input_subscription,
        || {
            refresh_runtime_with_fallback(host, &CoreMidiDiscoverySource, &SystemDiscoverySource)
                .map_err(|error| error.to_string())?;
            emit_runtime_snapshot(app, host).map_err(|error| error.to_string())
        },
    )
}

pub fn handle_runtime_pad_input_message<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    host: &crate::commands::CommandHost<
        impl crate::config::store::ConfigStoreBackend,
        impl crate::macos::ActionBackend,
        impl crate::device::Push3LedBackend,
    >,
    message: DecodedPadInputMessage,
) -> Result<(), String> {
    emit_decoded_pad_input_event(app, message.clone())
        .map_err(|error| error.to_string())?;

    if let DecodedPadInputMessage::PadPressed { pad_id, .. } = message {
        host.dispatch_pad_press(&pad_id)
            .map_err(|error| error.to_string())?;
    }

    Ok(())
}

fn bootstrap_runtime<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    host: &DefaultCommandHost,
) -> Result<(), Box<dyn Error>> {
    let startup_discovery =
        StartupDiscoverySource::new(CoreMidiDiscoverySource, SystemDiscoverySource);
    refresh_runtime_with_fallback(host, &startup_discovery, &NullDiscoverySource)?;
    emit_runtime_snapshot(app, host)?;

    match subscribe_push3_user_port_runtime_events(app) {
        Ok(subscription) => {
            let _ = store_push3_input_subscription(app, subscription);
        }
        Err(error) => {
            eprintln!("push3 input subscription unavailable at startup: {error}");
        }
    }

    match subscribe_push3_mode_runtime_events(app) {
        Ok(subscription) => {
            let _ = store_push3_mode_subscription(app, subscription);
        }
        Err(error) => {
            eprintln!("push3 mode subscription unavailable at startup: {error}");
        }
    }

    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let command_host = commands::DefaultCommandHost::bootstrap_default()
        .expect("failed to initialize command host");

    tauri::Builder::default()
        .manage(command_host)
        .setup(|app| {
            let host = app.state::<DefaultCommandHost>();
            bootstrap_runtime(&app.handle(), &host)
        })
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                if should_hide_on_close(window.label()) {
                    api.prevent_close();
                    let _ = window.hide();
                }
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::load_current_config,
            commands::refresh_runtime_state,
            commands::load_running_apps,
            commands::update_pad_binding,
            commands::update_push3_color_calibration,
            commands::preview_push3_palette,
            commands::sync_push3_leds,
            commands::trigger_test_action,
            commands::restore_default_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
