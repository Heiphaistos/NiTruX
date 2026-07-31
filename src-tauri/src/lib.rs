use std::sync::Mutex;

use sysinfo::System;

mod disks;
mod docker;
mod drivers;
mod duplicates;
mod hardware;
mod hashcheck;
mod largefiles;
mod logs;
mod network;
mod packages;
mod portscan;
mod sensors;
mod smart;
mod subprocess;
mod system;

#[tauri::command]
fn list_updates() -> Result<Vec<packages::PackageUpdate>, String> {
    let mut all_updates = Vec::new();
    for manager in packages::detect_package_managers() {
        // Attribute the error to the manager that raised it (e.g. "apt:
        // permission denied") so the frontend can surface something
        // actionable rather than a bare, unattributed message.
        let updates = manager
            .list_upgradable()
            .map_err(|e| format!("{}: {}", manager.id(), e))?;
        all_updates.extend(updates);
    }
    all_updates.extend(packages::universal::list_universal_updates());
    Ok(all_updates)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Held for the app's lifetime so CPU usage deltas can be computed
        // across repeated refreshes (see system::build_snapshot doc comment).
        .manage(Mutex::new(System::new_all()))
        .invoke_handler(tauri::generate_handler![
            system::get_system_snapshot,
            sensors::get_sensor_snapshot,
            hardware::get_pci_devices,
            drivers::get_driver_snapshot,
            logs::get_recent_logs,
            list_updates,
            disks::list_disks,
            disks::list_disk_usage,
            duplicates::find_duplicate_files,
            largefiles::find_large_files_cmd,
            hashcheck::compute_file_hash,
            hashcheck::verify_file_hash,
            smart::get_smart_status,
            network::get_network_snapshot,
            portscan::scan_ports_cmd,
            docker::get_docker_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
