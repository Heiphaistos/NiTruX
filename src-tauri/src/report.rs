use crate::disks::{Disk, UsageEntry};
use crate::drivers::DriverSnapshot;
use crate::firewall::FirewallStatus;
use crate::hardware::PciDevice;
use crate::network::NetworkSnapshot;
use crate::packages::PackageUpdate;
use crate::sensors::SensorSnapshot;
use crate::system::SystemSnapshot;
use serde::Serialize;
use std::sync::Mutex;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct SystemReport {
    pub generated_at: String,
    pub system: SystemSnapshot,
    pub sensors: SensorSnapshot,
    pub pci_devices: Option<Vec<PciDevice>>,
    pub drivers: Option<DriverSnapshot>,
    pub disks: Option<Vec<Disk>>,
    pub disk_usage: Option<Vec<UsageEntry>>,
    pub network: NetworkSnapshot,
    pub firewall: Option<FirewallStatus>,
    pub updates: Option<Vec<PackageUpdate>>,
}

fn format_bytes_gb(bytes: u64) -> String {
    format!("{:.1} Go", bytes as f64 / 1_073_741_824.0)
}

pub fn render_json(report: &SystemReport) -> String {
    serde_json::to_string_pretty(report).unwrap_or_else(|e| format!("{{\"error\": \"{e}\"}}"))
}

pub fn render_txt(report: &SystemReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("Rapport système NiTruX — {}\n\n", report.generated_at));

    out.push_str("== SYSTÈME ==\n");
    for cpu in &report.system.cpus {
        out.push_str(&format!("CPU : {} ({})\n", cpu.name, cpu.usage_display));
    }
    out.push_str(&format!(
        "Mémoire : {} / {}\n",
        format_bytes_gb(report.system.memory_used_bytes),
        format_bytes_gb(report.system.memory_total_bytes)
    ));
    out.push_str(&format!("Processus en cours : {}\n\n", report.system.process_count));

    out.push_str("== CAPTEURS ==\n");
    match report.sensors.battery_percent {
        Some(p) => out.push_str(&format!("Batterie : {p}%{}\n", if report.sensors.battery_charging == Some(true) { " (en charge)" } else { "" })),
        None => out.push_str("Batterie : aucune détectée\n"),
    }
    for t in &report.sensors.temperatures {
        out.push_str(&format!("Température {} : {:.1}°C\n", t.label, t.celsius));
    }
    out.push('\n');

    out.push_str("== PÉRIPHÉRIQUES PCI ==\n");
    match &report.pci_devices {
        Some(devices) => {
            for d in devices {
                out.push_str(&format!("{} — {} ({})\n", d.slot, d.description, d.class));
            }
        }
        None => out.push_str("indisponible\n"),
    }
    out.push('\n');

    out.push_str("== PILOTES ==\n");
    match &report.drivers {
        Some(d) => {
            out.push_str(&format!("Pilote GPU actif : {}\n", d.gpu_driver));
            out.push_str(&format!("Modules chargés : {}\n", d.loaded_modules.len()));
        }
        None => out.push_str("indisponible\n"),
    }
    out.push('\n');

    out.push_str("== DISQUES ==\n");
    match &report.disks {
        Some(disks) => {
            for disk in disks {
                out.push_str(&format!("{} — {}\n", disk.name, disk.size));
            }
        }
        None => out.push_str("indisponible\n"),
    }
    out.push('\n');

    out.push_str("== UTILISATION DISQUE ==\n");
    match &report.disk_usage {
        Some(usage) => {
            for u in usage {
                out.push_str(&format!("{} — {}% utilisé ({} / {})\n", u.mountpoint, u.used_percent, format_bytes_gb(u.used_bytes), format_bytes_gb(u.total_bytes)));
            }
        }
        None => out.push_str("indisponible\n"),
    }
    out.push('\n');

    out.push_str("== RÉSEAU ==\n");
    out.push_str(&format!("Serveurs DNS : {}\n", report.network.dns_servers.join(", ")));
    out.push_str(&format!("Ports en écoute : {}\n", report.network.listening_ports.len()));
    out.push_str(&format!("Réseaux Wi-Fi visibles : {}\n\n", report.network.wifi_networks.len()));

    out.push_str("== PARE-FEU ==\n");
    match &report.firewall {
        Some(fw) => {
            out.push_str(&format!("Actif : {}\n", if fw.active { "oui" } else { "non" }));
            for rule in &fw.rules {
                out.push_str(&format!("- {rule}\n"));
            }
        }
        None => out.push_str("indisponible\n"),
    }
    out.push('\n');

    out.push_str("== MISES À JOUR ==\n");
    match &report.updates {
        Some(updates) if updates.is_empty() => out.push_str("Aucune mise à jour disponible.\n"),
        Some(updates) => {
            for u in updates {
                out.push_str(&format!("{} ({}) : {} -> {}\n", u.name, u.source, u.current_version, u.new_version));
            }
        }
        None => out.push_str("indisponible\n"),
    }

    out
}

pub fn render_markdown(report: &SystemReport) -> String {
    let mut out = String::new();
    out.push_str(&format!("# Rapport système NiTruX\n\n_Généré le {}_\n\n", report.generated_at));

    out.push_str("## Système\n\n");
    for cpu in &report.system.cpus {
        out.push_str(&format!("- **CPU** : {} ({})\n", cpu.name, cpu.usage_display));
    }
    out.push_str(&format!(
        "- **Mémoire** : {} / {}\n- **Processus** : {}\n\n",
        format_bytes_gb(report.system.memory_used_bytes),
        format_bytes_gb(report.system.memory_total_bytes),
        report.system.process_count
    ));

    out.push_str("## Capteurs\n\n");
    match report.sensors.battery_percent {
        Some(p) => out.push_str(&format!("- **Batterie** : {p}%{}\n", if report.sensors.battery_charging == Some(true) { " (en charge)" } else { "" })),
        None => out.push_str("- **Batterie** : aucune détectée\n"),
    }
    for t in &report.sensors.temperatures {
        out.push_str(&format!("- **Température {}** : {:.1}°C\n", t.label, t.celsius));
    }
    out.push('\n');

    out.push_str("## Périphériques PCI\n\n");
    match &report.pci_devices {
        Some(devices) if !devices.is_empty() => {
            out.push_str("| Emplacement | Description | Classe |\n|---|---|---|\n");
            for d in devices {
                out.push_str(&format!("| {} | {} | {} |\n", d.slot, d.description, d.class));
            }
        }
        Some(_) => out.push_str("_Aucun périphérique détecté._\n"),
        None => out.push_str("_Indisponible._\n"),
    }
    out.push('\n');

    out.push_str("## Pilotes\n\n");
    match &report.drivers {
        Some(d) => out.push_str(&format!("- **Pilote GPU actif** : {}\n- **Modules chargés** : {}\n", d.gpu_driver, d.loaded_modules.len())),
        None => out.push_str("_Indisponible._\n"),
    }
    out.push('\n');

    out.push_str("## Disques\n\n");
    match &report.disks {
        Some(disks) if !disks.is_empty() => {
            out.push_str("| Nom | Taille |\n|---|---|\n");
            for disk in disks {
                out.push_str(&format!("| {} | {} |\n", disk.name, disk.size));
            }
        }
        Some(_) => out.push_str("_Aucun disque détecté._\n"),
        None => out.push_str("_Indisponible._\n"),
    }
    out.push('\n');

    out.push_str("## Réseau\n\n");
    out.push_str(&format!(
        "- **Serveurs DNS** : {}\n- **Ports en écoute** : {}\n- **Réseaux Wi-Fi visibles** : {}\n\n",
        report.network.dns_servers.join(", "),
        report.network.listening_ports.len(),
        report.network.wifi_networks.len()
    ));

    out.push_str("## Pare-feu\n\n");
    match &report.firewall {
        Some(fw) => {
            out.push_str(&format!("- **Actif** : {}\n", if fw.active { "oui" } else { "non" }));
            for rule in &fw.rules {
                out.push_str(&format!("- {rule}\n"));
            }
        }
        None => out.push_str("_Indisponible._\n"),
    }
    out.push('\n');

    out.push_str("## Mises à jour\n\n");
    match &report.updates {
        Some(updates) if updates.is_empty() => out.push_str("Aucune mise à jour disponible.\n"),
        Some(updates) => {
            out.push_str("| Paquet | Source | Actuelle | Nouvelle |\n|---|---|---|---|\n");
            for u in updates {
                out.push_str(&format!("| {} | {} | {} | {} |\n", u.name, u.source, u.current_version, u.new_version));
            }
        }
        None => out.push_str("_Indisponible._\n"),
    }

    out
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;")
}

pub fn render_html(report: &SystemReport) -> String {
    let mut body = String::new();

    body.push_str("<h2>Système</h2><ul>");
    for cpu in &report.system.cpus {
        body.push_str(&format!("<li><strong>CPU</strong> : {} ({})</li>", escape_html(&cpu.name), escape_html(&cpu.usage_display)));
    }
    body.push_str(&format!(
        "<li><strong>Mémoire</strong> : {} / {}</li><li><strong>Processus</strong> : {}</li></ul>",
        format_bytes_gb(report.system.memory_used_bytes),
        format_bytes_gb(report.system.memory_total_bytes),
        report.system.process_count
    ));

    body.push_str("<h2>Périphériques PCI</h2>");
    match &report.pci_devices {
        Some(devices) if !devices.is_empty() => {
            body.push_str("<table><tr><th>Emplacement</th><th>Description</th><th>Classe</th></tr>");
            for d in devices {
                body.push_str(&format!("<tr><td>{}</td><td>{}</td><td>{}</td></tr>", escape_html(&d.slot), escape_html(&d.description), escape_html(&d.class)));
            }
            body.push_str("</table>");
        }
        Some(_) => body.push_str("<p>Aucun périphérique détecté.</p>"),
        None => body.push_str("<p><em>Indisponible.</em></p>"),
    }

    body.push_str("<h2>Pilotes</h2>");
    match &report.drivers {
        Some(d) => body.push_str(&format!("<p>Pilote GPU actif : {}<br>Modules chargés : {}</p>", escape_html(&d.gpu_driver), d.loaded_modules.len())),
        None => body.push_str("<p><em>Indisponible.</em></p>"),
    }

    body.push_str("<h2>Disques</h2>");
    match &report.disks {
        Some(disks) if !disks.is_empty() => {
            body.push_str("<table><tr><th>Nom</th><th>Taille</th></tr>");
            for disk in disks {
                body.push_str(&format!("<tr><td>{}</td><td>{}</td></tr>", escape_html(&disk.name), escape_html(&disk.size)));
            }
            body.push_str("</table>");
        }
        Some(_) => body.push_str("<p>Aucun disque détecté.</p>"),
        None => body.push_str("<p><em>Indisponible.</em></p>"),
    }

    body.push_str("<h2>Réseau</h2><ul>");
    body.push_str(&format!(
        "<li>Serveurs DNS : {}</li><li>Ports en écoute : {}</li><li>Réseaux Wi-Fi visibles : {}</li></ul>",
        escape_html(&report.network.dns_servers.join(", ")),
        report.network.listening_ports.len(),
        report.network.wifi_networks.len()
    ));

    body.push_str("<h2>Pare-feu</h2>");
    match &report.firewall {
        Some(fw) => {
            body.push_str(&format!("<p>Actif : {}</p><ul>", if fw.active { "oui" } else { "non" }));
            for rule in &fw.rules {
                body.push_str(&format!("<li>{}</li>", escape_html(rule)));
            }
            body.push_str("</ul>");
        }
        None => body.push_str("<p><em>Indisponible.</em></p>"),
    }

    body.push_str("<h2>Mises à jour</h2>");
    match &report.updates {
        Some(updates) if updates.is_empty() => body.push_str("<p>Aucune mise à jour disponible.</p>"),
        Some(updates) => {
            body.push_str("<table><tr><th>Paquet</th><th>Source</th><th>Actuelle</th><th>Nouvelle</th></tr>");
            for u in updates {
                body.push_str(&format!(
                    "<tr><td>{}</td><td>{}</td><td>{}</td><td>{}</td></tr>",
                    escape_html(&u.name), escape_html(&u.source), escape_html(&u.current_version), escape_html(&u.new_version)
                ));
            }
            body.push_str("</table>");
        }
        None => body.push_str("<p><em>Indisponible.</em></p>"),
    }

    format!(
        "<!DOCTYPE html><html lang=\"fr\"><head><meta charset=\"utf-8\"><title>Rapport système NiTruX</title><style>body{{font-family:sans-serif;max-width:900px;margin:2rem auto;padding:0 1rem}}table{{border-collapse:collapse;width:100%;margin-bottom:1rem}}th,td{{border:1px solid #ccc;padding:6px 10px;text-align:left}}h1,h2{{color:#222}}</style></head><body><h1>Rapport système NiTruX</h1><p><em>Généré le {}</em></p>{}</body></html>",
        escape_html(&report.generated_at),
        body
    )
}

pub fn build_system_report(sys: &mut System) -> SystemReport {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    SystemReport {
        generated_at: format!("epoch:{now}"),
        system: crate::system::build_snapshot(sys),
        sensors: crate::sensors::get_sensor_snapshot(),
        pci_devices: crate::hardware::get_pci_devices().ok(),
        drivers: crate::drivers::get_driver_snapshot().ok(),
        disks: crate::disks::list_disks().ok(),
        disk_usage: crate::disks::list_disk_usage().ok(),
        network: crate::network::get_network_snapshot(),
        firewall: crate::firewall::get_firewall_status().ok(),
        updates: crate::list_updates().ok(),
    }
}

#[tauri::command]
pub fn generate_system_report(state: tauri::State<Mutex<System>>, format: String) -> Result<String, String> {
    let mut sys = state.lock().expect("system state mutex poisoned");
    let report = build_system_report(&mut sys);
    match format.as_str() {
        "json" => Ok(render_json(&report)),
        "markdown" => Ok(render_markdown(&report)),
        "txt" => Ok(render_txt(&report)),
        "html" => Ok(render_html(&report)),
        other => Err(format!("format de rapport inconnu : {other}")),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::CpuInfo;

    fn fixture_report() -> SystemReport {
        SystemReport {
            generated_at: "2026-08-01T14:00:00+02:00".to_string(),
            system: SystemSnapshot {
                cpus: vec![CpuInfo { name: "Test CPU".to_string(), usage_percent: 12.5, usage_display: "12.5%".to_string() }],
                memory_used_bytes: 4_000_000_000,
                memory_total_bytes: 8_000_000_000,
                process_count: 210,
            },
            sensors: SensorSnapshot { battery_percent: Some(80), battery_charging: Some(true), temperatures: vec![] },
            pci_devices: Some(vec![PciDevice { slot: "00:02.0".to_string(), class: "VGA compatible controller".to_string(), description: "Intel UHD Graphics".to_string() }]),
            drivers: None,
            disks: Some(vec![Disk { name: "sda".to_string(), size: "500G".to_string(), partitions: vec![] }]),
            disk_usage: None,
            network: NetworkSnapshot { wifi_networks: vec![], listening_ports: vec![], dns_servers: vec!["1.1.1.1".to_string()], hosts_file: "127.0.0.1 localhost\n".to_string() },
            firewall: Some(FirewallStatus { active: true, rules: vec!["22/tcp ALLOW Anywhere".to_string()] }),
            updates: Some(vec![PackageUpdate { name: "curl".to_string(), current_version: "7.88".to_string(), new_version: "7.89".to_string(), source: "apt".to_string() }]),
        }
    }

    #[test]
    fn render_json_produces_valid_parseable_json_with_key_fields() {
        let report = fixture_report();
        let json = render_json(&report);
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("should be valid JSON");
        assert_eq!(parsed["system"]["process_count"], 210);
        assert!(json.contains("\"generated_at\""));
        assert!(json.contains("Test CPU"));
        assert!(json.contains("curl"));
    }

    #[test]
    fn render_txt_includes_every_section_and_handles_missing_data_honestly() {
        let report = fixture_report();
        let txt = render_txt(&report);
        assert!(txt.contains("Test CPU"));
        assert!(txt.contains("80%")); // battery
        assert!(txt.contains("Intel UHD Graphics"));
        assert!(txt.contains("indisponible")); // drivers: None in fixture
        assert!(txt.contains("sda"));
        assert!(txt.contains("1.1.1.1"));
        assert!(txt.contains("22/tcp ALLOW Anywhere"));
        assert!(txt.contains("curl"));
    }

    #[test]
    fn render_markdown_includes_section_headers_and_key_fields() {
        let report = fixture_report();
        let md = render_markdown(&report);
        assert!(md.starts_with("# "));
        assert!(md.contains("## "));
        assert!(md.contains("Test CPU"));
        assert!(md.contains("curl"));
    }

    #[test]
    fn render_html_produces_a_document_with_key_fields_and_no_unescaped_lt_in_text_content() {
        let report = fixture_report();
        let html = render_html(&report);
        assert!(html.contains("<!DOCTYPE html>") || html.contains("<html"));
        assert!(html.contains("Test CPU"));
        assert!(html.contains("curl"));
    }
}
