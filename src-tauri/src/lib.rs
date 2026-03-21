pub mod actions;
pub mod app_state;
pub mod commands;
pub mod config;
pub mod device;
pub mod display;
pub mod events;
pub mod macos;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let command_host = commands::DefaultCommandHost::bootstrap_default()
        .expect("failed to initialize command host");

    tauri::Builder::default()
        .manage(command_host)
        .invoke_handler(tauri::generate_handler![
            commands::load_current_config,
            commands::update_pad_binding,
            commands::trigger_test_action,
            commands::restore_default_config,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
