use std::sync::Mutex;

use sysinfo::System;

mod benchmark;
mod disk_write;
mod disks;
mod docker;
mod drivers;
mod duplicates;
mod firewall;
mod hardware;
mod hashcheck;
mod largefiles;
mod logs;
mod malwarescan;
mod network;
mod network_write;
mod optimizations;
mod packages;
mod portscan;
mod report;
mod security_write;
mod sensors;
mod smart;
mod snapshots;
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

/// Returns the id of the first detected native package manager
/// ("apt"/"dnf"/"pacman"/"zypper"), or `None` if none is present. Thin
/// wrapper over the already-tested `packages::detect_package_managers()` —
/// no dedicated test here for the same reason `list_updates` has none:
/// it's a pure aggregation over an already-verified primitive, and actually
/// exercising manager detection requires the real host's binaries (already
/// covered by `detected_manager_id_matches_binary_name` in
/// `packages/mod.rs`).
#[tauri::command]
fn detect_native_manager() -> Option<String> {
    packages::detect_package_managers()
        .first()
        .map(|m| m.id().to_string())
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
            detect_native_manager,
            report::generate_system_report,
            benchmark::run_benchmark,
            disks::list_disks,
            disks::list_disk_usage,
            duplicates::find_duplicate_files,
            largefiles::find_large_files_cmd,
            hashcheck::compute_file_hash,
            hashcheck::verify_file_hash,
            smart::get_smart_status,
            network::get_network_snapshot,
            portscan::scan_ports_cmd,
            docker::get_docker_snapshot,
            firewall::get_firewall_status,
            malwarescan::scan_for_malware,
            snapshots::list_snapshots,
            packages::install::install_package,
            packages::install::upgrade_all_packages,
            network_write::write_hosts_file,
            network_write::set_dns_servers,
            network_write::add_firewall_rule,
            network_write::remove_firewall_rule,
            security_write::run_troubleshoot_action,
            security_write::create_snapshot,
            security_write::quarantine_file,
            disk_write::format_partition,
            disk_write::extend_partition,
            disk_write::clone_disk,
            optimizations::get_optimization_snapshot,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
