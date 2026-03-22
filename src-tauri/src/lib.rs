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
use crate::events::{emit_runtime_event, RuntimeEvent};
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

fn bootstrap_runtime<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    host: &DefaultCommandHost,
) -> Result<(), Box<dyn Error>> {
    host.refresh_runtime(&NullDiscoverySource)?;
    emit_runtime_snapshot(app, host)
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
            commands::update_pad_binding,
            commands::trigger_test_action,
            commands::restore_default_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
