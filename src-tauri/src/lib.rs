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
    subscribe_push3_user_port_runtime_events, CoreMidiDiscoverySource, DeviceDiscoverySource,
    Push3InputSubscription, StartupDiscoverySource,
};
use crate::events::{emit_runtime_event, RuntimeEvent};
use crate::macos::ActionBackend;
use std::error::Error;
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

fn emit_runtime_snapshot<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    host: &DefaultCommandHost,
) -> Result<(), Box<dyn Error>> {
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

pub fn store_push3_input_subscription<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    subscription: Push3InputSubscription<R>,
) -> bool {
    match subscription {
        Push3InputSubscription::Active(connection) => {
            app.manage(std::sync::Mutex::new(Some(connection)));
            true
        }
        Push3InputSubscription::NotConnected => false,
    }
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
