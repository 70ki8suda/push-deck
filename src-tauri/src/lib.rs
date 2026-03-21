pub mod actions;
pub mod app_state;
pub mod config;
pub mod device;
pub mod display;
pub mod events;
pub mod macos;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
