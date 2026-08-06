use std::sync::Mutex;

use sysinfo::System;

mod accounts;
mod backup;
mod benchmark;
mod bluetooth;
mod boot_manager;
mod cache_size;
mod dependencies;
mod disk_write;
mod disks;
mod docker;
mod drivers;
mod duplicates;
mod firewall;
mod hardware;
mod hardware_details;
mod hashcheck;
mod largefiles;
mod logs;
mod malwarescan;
mod network;
mod network_write;
mod optimizations;
mod packages;
mod peripherals;
mod pkexec_bootstrap;
mod portscan;
mod processes;
mod report;
mod scripts;
mod secure_temp;
mod security_write;
mod sensors;
mod smart;
mod snapshots;
mod subprocess;
mod system;
mod system_tools;
mod terminal;
mod trash;
mod update_history;

/// Combines each detected native manager's `list_upgradable()` outcome
/// (already tagged with that manager's id) into one result. A host with two
/// native managers is rare but real (see `detect_package_managers`'s own
/// doc comment) -- if one of them fails, that must not discard updates
/// already found by another that succeeded, matching this codebase's
/// dominant "each source degrades independently" philosophy (`network.rs`,
/// `docker.rs`, ...) rather than the previous all-or-nothing `?` in a loop.
/// A manager's error is only surfaced (attributed, e.g. "apt: permission
/// denied") when EVERY manager failed and there is nothing else to show --
/// in the overwhelmingly common single-manager case this preserves the
/// original, still-useful behavior exactly.
fn aggregate_update_results(results: Vec<(&str, Result<Vec<packages::PackageUpdate>, String>)>) -> Result<Vec<packages::PackageUpdate>, String> {
    let mut all_updates = Vec::new();
    let mut first_error = None;
    for (manager_id, result) in results {
        match result {
            Ok(updates) => all_updates.extend(updates),
            Err(e) => {
                first_error.get_or_insert(format!("{manager_id}: {e}"));
            }
        }
    }
    if all_updates.is_empty() {
        if let Some(e) = first_error {
            return Err(e);
        }
    }
    Ok(all_updates)
}

#[tauri::command]
fn list_updates() -> Result<Vec<packages::PackageUpdate>, String> {
    let native_results: Vec<(&str, Result<Vec<packages::PackageUpdate>, String>)> = packages::detect_package_managers()
        .iter()
        .map(|manager| (manager.id(), manager.list_upgradable()))
        .collect();
    let mut all_updates = aggregate_update_results(native_results)?;
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

/// Environment variables of the app's own process, i.e. exactly what the
/// invoking user's shell environment looked like when NiTruX was launched
/// -- a normal, expected diagnostic view of one's own environment, nothing
/// exposed that the user couldn't already see in their own shell.
#[tauri::command]
fn get_environment_variables() -> Vec<(String, String)> {
    std::env::vars().collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        // Held for the app's lifetime so CPU usage deltas can be computed
        // across repeated refreshes (see system::build_snapshot doc comment).
        .manage(Mutex::new(System::new_all()))
        .manage(terminal::TerminalState::default())
        .invoke_handler(tauri::generate_handler![
            system::get_system_snapshot,
            accounts::get_user_accounts,
            sensors::get_sensor_snapshot,
            hardware::get_pci_devices,
            hardware_details::get_hardware_details,
            drivers::get_driver_snapshot,
            logs::get_recent_logs,
            list_updates,
            detect_native_manager,
            get_environment_variables,
            packages::list_installed_packages,
            peripherals::get_monitors,
            peripherals::get_usb_devices,
            peripherals::get_audio_sinks,
            peripherals::get_printers,
            packages::flatpak::install_flatpak_package,
            report::generate_system_report,
            report::generate_pdf_report,
            benchmark::run_benchmark,
            disks::list_disks,
            disks::list_disk_usage,
            duplicates::find_duplicate_files,
            largefiles::find_large_files_cmd,
            hashcheck::compute_file_hash,
            hashcheck::verify_file_hash,
            smart::get_smart_status,
            network::get_network_snapshot,
            network::get_network_interfaces,
            portscan::scan_ports_cmd,
            processes::get_processes,
            processes::get_systemd_services,
            processes::get_autostart_entries,
            processes::get_scheduled_tasks,
            docker::get_docker_snapshot,
            firewall::get_firewall_status,
            malwarescan::scan_for_malware,
            snapshots::list_snapshots,
            packages::install::install_package,
            packages::install::uninstall_package,
            packages::install::upgrade_all_packages,
            packages::install::install_snap_package,
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
            cache_size::get_cache_size_report,
            dependencies::scan_missing_dependencies,
            backup::create_backup,
            bluetooth::get_bluetooth_status,
            boot_manager::get_boot_manager_snapshot,
            scripts::run_script,
            trash::list_trash,
            trash::restore_trash_item,
            trash::delete_trash_item_permanently,
            update_history::get_update_history,
            system_tools::run_system_tool,
            terminal::spawn_terminal,
            terminal::write_to_terminal,
            terminal::resize_terminal,
            terminal::close_terminal,
            pkexec_bootstrap::is_pkexec_integration_installed,
            pkexec_bootstrap::install_pkexec_integration,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    use super::*;
    use packages::PackageUpdate;

    fn update(name: &str) -> PackageUpdate {
        PackageUpdate { name: name.to_string(), current_version: "1.0".to_string(), new_version: "1.1".to_string(), source: "apt".to_string() }
    }

    #[test]
    fn a_single_failing_manager_surfaces_its_attributed_error() {
        let result = aggregate_update_results(vec![("apt", Err("permission denied".to_string()))]);
        assert_eq!(result, Err("apt: permission denied".to_string()));
    }

    #[test]
    fn one_manager_failing_does_not_discard_another_managers_successful_updates() {
        // The bug this guards against: a naive `?` inside the aggregation
        // loop would propagate dnf's error and discard apt's already-found
        // updates entirely, even though apt succeeded.
        let result = aggregate_update_results(vec![
            ("apt", Ok(vec![update("curl")])),
            ("dnf", Err("dnf not configured".to_string())),
        ]);
        let updates = result.expect("should still return apt's updates despite dnf failing");
        assert_eq!(updates.len(), 1);
        assert_eq!(updates[0].name, "curl");
    }

    #[test]
    fn all_managers_failing_surfaces_the_first_error() {
        let result = aggregate_update_results(vec![
            ("apt", Err("first failure".to_string())),
            ("dnf", Err("second failure".to_string())),
        ]);
        assert_eq!(result, Err("apt: first failure".to_string()));
    }

    #[test]
    fn no_managers_detected_returns_an_empty_list_not_an_error() {
        let result = aggregate_update_results(vec![]);
        assert_eq!(result, Ok(vec![]));
    }
}
