use std::sync::Mutex;

use sysinfo::System;

// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

mod drivers;
mod hardware;
mod sensors;
mod subprocess;
mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Held for the app's lifetime so CPU usage deltas can be computed
        // across repeated refreshes (see system::build_snapshot doc comment).
        .manage(Mutex::new(System::new_all()))
        .invoke_handler(tauri::generate_handler![
            greet,
            system::get_system_snapshot,
            sensors::get_sensor_snapshot,
            hardware::get_pci_devices,
            drivers::get_driver_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
