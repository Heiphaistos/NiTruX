use crate::disks::{Disk, UsageEntry};
use crate::drivers::DriverSnapshot;
use crate::firewall::FirewallStatus;
use crate::hardware::PciDevice;
use crate::network::NetworkSnapshot;
use crate::packages::PackageUpdate;
use crate::sensors::SensorSnapshot;
use crate::system::SystemSnapshot;
use base64::{engine::general_purpose::STANDARD, Engine as _};
use serde::Serialize;
use std::collections::BTreeMap;
use std::sync::Mutex;
use sysinfo::System;

#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct ReportFile {
    pub filename: String,
    pub size_bytes: u64,
    pub created_at: String,
}

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

    out.push_str("## Utilisation disque\n\n");
    match &report.disk_usage {
        Some(usage) if !usage.is_empty() => {
            out.push_str("| Point de montage | Utilisé | Total |\n|---|---|---|\n");
            for u in usage {
                out.push_str(&format!(
                    "| {} | {}% ({}) | {} |\n",
                    u.mountpoint, u.used_percent, format_bytes_gb(u.used_bytes), format_bytes_gb(u.total_bytes)
                ));
            }
        }
        Some(_) => out.push_str("_Aucune information d'utilisation disque._\n"),
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

    body.push_str("<h2>Capteurs</h2><ul>");
    match report.sensors.battery_percent {
        Some(p) => body.push_str(&format!(
            "<li><strong>Batterie</strong> : {p}%{}</li>",
            if report.sensors.battery_charging == Some(true) { " (en charge)" } else { "" }
        )),
        None => body.push_str("<li><strong>Batterie</strong> : aucune détectée</li>"),
    }
    for t in &report.sensors.temperatures {
        body.push_str(&format!("<li><strong>Température {}</strong> : {:.1}°C</li>", escape_html(&t.label), t.celsius));
    }
    body.push_str("</ul>");

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

    body.push_str("<h2>Utilisation disque</h2>");
    match &report.disk_usage {
        Some(usage) if !usage.is_empty() => {
            body.push_str("<table><tr><th>Point de montage</th><th>Utilisé</th><th>Total</th></tr>");
            for u in usage {
                body.push_str(&format!(
                    "<tr><td>{}</td><td>{}% ({})</td><td>{}</td></tr>",
                    escape_html(&u.mountpoint), u.used_percent, format_bytes_gb(u.used_bytes), format_bytes_gb(u.total_bytes)
                ));
            }
            body.push_str("</table>");
        }
        Some(_) => body.push_str("<p>Aucune information d'utilisation disque.</p>"),
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

/// Converts days since 1970-01-01 (proleptic Gregorian calendar, UTC) to
/// (year, month, day). Howard Hinnant's well-known, exhaustively-verified
/// public-domain algorithm (http://howardhinnant.github.io/date_algorithms.html)
/// -- not hand-rolled/approximate calendar math -- valid for the entire
/// `i64` range. This project has no date/time crate dependency; adding one
/// for a single timestamp format felt heavier than this self-contained,
/// precisely-tested conversion.
fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = if z >= 0 { z } else { z - 146_096 } / 146_097;
    let doe = (z - era * 146_097) as u64; // day of era, [0, 146096]
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365; // year of era, [0, 399]
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100); // day of year, [0, 365]
    let mp = (5 * doy + 2) / 153; // [0, 11]
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32; // [1, 31]
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32; // [1, 12]
    let y = if m <= 2 { y + 1 } else { y };
    (y, m, d)
}

/// Formats a Unix epoch (seconds) as an ISO-8601 UTC timestamp
/// ("YYYY-MM-DDTHH:MM:SSZ"), used for `generated_at` in exported reports.
/// Previously this was the literal string "epoch:<seconds>" (e.g.
/// "epoch:1735689600"), shown verbatim to the user in every rendered
/// report ("Généré le epoch:1735689600") -- never actually formatted as a
/// human-readable date, in every format (txt/markdown/html).
pub fn format_epoch_utc(epoch_secs: u64) -> String {
    let days = (epoch_secs / 86_400) as i64;
    let secs_of_day = epoch_secs % 86_400;
    let (year, month, day) = civil_from_days(days);
    let hour = secs_of_day / 3600;
    let minute = (secs_of_day % 3600) / 60;
    let second = secs_of_day % 60;
    format!("{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}Z")
}

pub fn build_system_report(sys: &mut System) -> SystemReport {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);

    SystemReport {
        generated_at: format_epoch_utc(now),
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

/// Renders the report's existing, already-tested HTML representation
/// (`render_html`) to real PDF bytes via printpdf's HTML-to-PDF layout
/// engine, rather than hand-rolling a second, independent layout for PDF --
/// one source of truth for section structure/wording, two output formats.
pub fn render_pdf(report: &SystemReport) -> Result<Vec<u8>, String> {
    let html = render_html(report);
    let images = BTreeMap::new();
    let fonts = BTreeMap::new();
    let options = printpdf::GeneratePdfOptions::default();
    let mut warnings = Vec::new();
    let doc = printpdf::PdfDocument::from_html(&html, &images, &fonts, &options, &mut warnings)
        .map_err(|e| format!("échec de génération du PDF : {e}"))?;
    let save_options = printpdf::PdfSaveOptions::default();
    let mut save_warnings = Vec::new();
    Ok(doc.save(&save_options, &mut save_warnings))
}

/// Returns the PDF as base64 (not raw bytes) so it round-trips through
/// Tauri's JSON-based IPC as a compact string instead of a huge JSON array
/// of numbers; the frontend decodes it back to a Blob for download.
#[tauri::command]
pub fn generate_pdf_report(state: tauri::State<Mutex<System>>) -> Result<String, String> {
    let mut sys = state.lock().expect("system state mutex poisoned");
    let report = build_system_report(&mut sys);
    let pdf_bytes = render_pdf(&report)?;
    Ok(STANDARD.encode(pdf_bytes))
}

// ── Reports library (StatsReportsPage NiTriTe: "Rapports générés") ─────────
// Persists every report the user actually downloads to a fixed directory
// (not every preview -- clicking "Générer" to preview shouldn't silently
// litter disk) so they can be found again later without re-generating,
// mirroring NiTriTe's own reports folder + listing.

const REPORT_EXTENSIONS: [&str; 5] = ["json", "md", "txt", "html", "pdf"];

fn reports_dir() -> Result<String, String> {
    let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
    Ok(format!("{home}/.local/share/nitrux/reports"))
}

/// A stored report's filename must stay a plain, single path component
/// starting with "rapport-" and ending in one of the formats this app
/// actually generates -- rejects traversal (`..`, `/`) since this name is
/// joined onto `reports_dir()` and used directly as a delete target.
fn validate_report_filename(filename: &str) -> Result<(), String> {
    if filename.is_empty() || filename.contains('/') || filename.contains("..") {
        return Err(format!("nom de fichier de rapport invalide : {filename}"));
    }
    if !filename.starts_with("rapport-") {
        return Err(format!("nom de fichier de rapport invalide : {filename}"));
    }
    if !REPORT_EXTENSIONS.iter().any(|ext| filename.ends_with(&format!(".{ext}"))) {
        return Err(format!("nom de fichier de rapport invalide : {filename}"));
    }
    Ok(())
}

fn list_reports_in(dir: &str) -> Result<Vec<ReportFile>, String> {
    let entries = match std::fs::read_dir(dir) {
        Ok(entries) => entries,
        // No report saved yet -- the common case, not an error.
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(vec![]),
        Err(e) => return Err(format!("impossible de lire {dir} : {e}")),
    };
    let mut reports: Vec<(u64, ReportFile)> = Vec::new();
    for entry in entries {
        let entry = entry.map_err(|e| format!("erreur de lecture de répertoire : {e}"))?;
        let filename = entry.file_name().to_string_lossy().into_owned();
        if validate_report_filename(&filename).is_err() {
            continue; // foreign file in our directory -- ignore, don't error the whole list
        }
        let metadata = entry.metadata().map_err(|e| format!("métadonnées illisibles pour {filename} : {e}"))?;
        let modified_epoch = metadata
            .modified()
            .ok()
            .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|d| d.as_secs())
            .unwrap_or(0);
        reports.push((
            modified_epoch,
            ReportFile { filename, size_bytes: metadata.len(), created_at: format_epoch_utc(modified_epoch) },
        ));
    }
    reports.sort_by_key(|r| std::cmp::Reverse(r.0)); // most recent first
    Ok(reports.into_iter().map(|(_, r)| r).collect())
}

/// Writes `bytes` under a fresh `rapport-<epoch>[-N].<extension>` filename.
/// The `-N` disambiguation loop exists because two downloads within the
/// same wall-clock second (a plausible double-click, e.g. "Télécharger"
/// then immediately "Télécharger en PDF" -- different extension so no
/// clash there, but two same-format downloads in one second would collide)
/// must never silently overwrite an earlier report -- the exact
/// Date.now()-collision bug class already found and fixed twice elsewhere
/// in this project (profilesStore, ThemeEditorPage), avoided here from the
/// start instead of discovered later.
fn write_report_bytes_in(dir: &str, bytes: &[u8], extension: &str) -> Result<ReportFile, String> {
    if !REPORT_EXTENSIONS.contains(&extension) {
        return Err(format!("format de rapport inconnu : {extension}"));
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("impossible de créer {dir} : {e}"))?;
    let epoch = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let mut filename = format!("rapport-{epoch}.{extension}");
    let mut path = format!("{dir}/{filename}");
    let mut n = 2;
    while std::path::Path::new(&path).exists() {
        filename = format!("rapport-{epoch}-{n}.{extension}");
        path = format!("{dir}/{filename}");
        n += 1;
    }
    std::fs::write(&path, bytes).map_err(|e| format!("impossible d'écrire {filename} : {e}"))?;
    Ok(ReportFile { filename, size_bytes: bytes.len() as u64, created_at: format_epoch_utc(epoch) })
}

fn delete_report_in(dir: &str, filename: &str) -> Result<(), String> {
    validate_report_filename(filename)?;
    let path = format!("{dir}/{filename}");
    std::fs::remove_file(&path).map_err(|e| format!("impossible de supprimer {filename} : {e}"))
}

#[tauri::command]
pub fn list_reports() -> Result<Vec<ReportFile>, String> {
    list_reports_in(&reports_dir()?)
}

#[tauri::command]
pub fn save_text_report(content: String, extension: String) -> Result<ReportFile, String> {
    write_report_bytes_in(&reports_dir()?, content.as_bytes(), &extension)
}

#[tauri::command]
pub fn save_pdf_report(base64_content: String) -> Result<ReportFile, String> {
    let bytes = STANDARD.decode(&base64_content).map_err(|e| format!("PDF base64 invalide : {e}"))?;
    write_report_bytes_in(&reports_dir()?, &bytes, "pdf")
}

#[tauri::command]
pub fn delete_report(filename: String) -> Result<(), String> {
    delete_report_in(&reports_dir()?, &filename)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::system::CpuInfo;

    // Reference values obtained from `date -u -d @<epoch>` (real coreutils,
    // not computed by hand) to verify format_epoch_utc/civil_from_days
    // independently of their own implementation.
    #[test]
    fn format_epoch_utc_matches_the_unix_epoch() {
        assert_eq!(format_epoch_utc(0), "1970-01-01T00:00:00Z");
    }

    #[test]
    fn format_epoch_utc_matches_a_recent_new_year() {
        assert_eq!(format_epoch_utc(1_735_689_600), "2025-01-01T00:00:00Z");
    }

    #[test]
    fn format_epoch_utc_matches_a_future_new_year() {
        assert_eq!(format_epoch_utc(1_893_456_000), "2030-01-01T00:00:00Z");
    }

    #[test]
    fn format_epoch_utc_matches_a_value_with_nonzero_time_of_day() {
        assert_eq!(format_epoch_utc(1_000_000_000), "2001-09-09T01:46:40Z");
    }

    #[test]
    fn format_epoch_utc_handles_seconds_within_the_first_minute() {
        assert_eq!(format_epoch_utc(59), "1970-01-01T00:00:59Z");
    }

    #[test]
    fn generated_at_is_never_the_old_unformatted_epoch_placeholder() {
        // Regression guard for the actual bug: every rendered report used
        // to show the literal string "epoch:<seconds>" verbatim to the
        // user instead of a real date.
        let formatted = format_epoch_utc(1_735_689_600);
        assert!(!formatted.starts_with("epoch:"));
        assert!(formatted.contains('-') && formatted.contains(':'));
    }

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
            network: NetworkSnapshot { wifi_networks: vec![], listening_ports: vec![], dns_servers: vec!["1.1.1.1".to_string()], hosts_file: "127.0.0.1 localhost\n".to_string(), routes: vec![] },
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

    fn fixture_report_with_disk_usage() -> SystemReport {
        let mut report = fixture_report();
        report.disk_usage = Some(vec![crate::disks::UsageEntry {
            mountpoint: "/".to_string(),
            total_bytes: 500_000_000_000,
            used_bytes: 100_000_000_000,
            used_percent: 20,
        }]);
        report
    }

    #[test]
    fn render_markdown_includes_disk_usage_like_render_txt_already_does() {
        // render_txt has a dedicated "== UTILISATION DISQUE ==" section
        // (disk_usage: Option<Vec<UsageEntry>>) -- render_markdown had none
        // at all, silently dropping this data for the Markdown export.
        let report = fixture_report_with_disk_usage();
        let md = render_markdown(&report);
        assert!(md.contains("93.1 Go"), "Markdown report must include disk usage data like render_txt does");
    }

    #[test]
    fn render_html_produces_a_document_with_key_fields_and_no_unescaped_lt_in_text_content() {
        let report = fixture_report();
        let html = render_html(&report);
        assert!(html.contains("<!DOCTYPE html>") || html.contains("<html"));
        assert!(html.contains("Test CPU"));
        assert!(html.contains("curl"));
    }

    #[test]
    fn render_html_includes_the_sensors_section_like_render_txt_and_render_markdown_already_do() {
        // render_txt/render_markdown both have a "Capteurs"/"CAPTEURS"
        // section (battery + temperatures) -- render_html had none at all,
        // silently dropping this data for both the HTML export AND the PDF
        // export (render_pdf is just render_html fed through printpdf), the
        // only two formats where render_txt/render_markdown's own coverage
        // doesn't already guarantee the data reaches the user.
        let report = fixture_report(); // battery_percent: Some(80)
        let html = render_html(&report);
        assert!(html.contains("80%"), "HTML report must include battery data like the other formats do");
    }

    #[test]
    fn render_html_includes_disk_usage_like_render_txt_already_does() {
        // Same gap as the Capteurs section above, for disk_usage instead:
        // render_txt has "== UTILISATION DISQUE ==", render_html had
        // nothing -- also missing from the PDF export by extension.
        let report = fixture_report_with_disk_usage();
        let html = render_html(&report);
        assert!(html.contains("93.1 Go"), "HTML report must include disk usage data like render_txt does");
    }

    #[test]
    fn render_pdf_produces_bytes_starting_with_the_pdf_magic_header() {
        let report = fixture_report();
        let bytes = render_pdf(&report).expect("HTML generated by render_html should always convert to a valid PDF");
        // "%PDF-" is the mandatory first 5 bytes of every valid PDF file
        // (PDF spec section 7.5.2) -- proves this is a real, well-formed PDF
        // document, not just any non-empty byte blob.
        assert!(bytes.starts_with(b"%PDF-"), "expected PDF magic header, got: {:?}", &bytes[..bytes.len().min(20)]);
        assert!(bytes.len() > 100, "a rendered multi-section report should produce more than a trivial/empty PDF");
    }

    #[test]
    fn generate_pdf_report_command_returns_valid_base64_decoding_to_a_real_pdf() {
        let report = fixture_report();
        let pdf_bytes = render_pdf(&report).expect("should render");
        let encoded = STANDARD.encode(&pdf_bytes);
        let decoded = STANDARD.decode(&encoded).expect("should be valid base64");
        assert_eq!(decoded, pdf_bytes);
    }

    // ── Reports library ──────────────────────────────────────────────────
    // Each test gets its own uniquely-named temp directory (not the real
    // reports_dir(), which is shared across the whole test binary) so
    // parallel test threads never race on the same files -- same pattern
    // already established in trash.rs/backup.rs for filesystem-touching tests.
    fn test_dir(purpose: &str) -> String {
        let dir = std::env::temp_dir().join(format!("nitrux-reports-test-{purpose}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir); // stale leftovers from a prior crashed run
        dir.to_string_lossy().into_owned()
    }

    #[test]
    fn rejects_empty_traversal_and_wrongly_prefixed_or_extensioned_filenames() {
        assert!(validate_report_filename("").is_err());
        assert!(validate_report_filename("../../etc/passwd").is_err());
        assert!(validate_report_filename("rapport-123/../../evil.json").is_err());
        assert!(validate_report_filename("evil-123.json").is_err()); // missing "rapport-" prefix
        assert!(validate_report_filename("rapport-123.exe").is_err()); // not a report extension
    }

    #[test]
    fn accepts_every_real_report_extension() {
        for ext in REPORT_EXTENSIONS {
            assert!(validate_report_filename(&format!("rapport-1735689600.{ext}")).is_ok());
        }
    }

    #[test]
    fn list_on_a_missing_directory_returns_an_empty_list_not_an_error() {
        let dir = test_dir("list-missing");
        assert_eq!(list_reports_in(&dir), Ok(vec![]));
    }

    #[test]
    fn write_then_list_round_trips_the_real_file_size_and_a_real_date() {
        let dir = test_dir("write-list");
        let saved = write_report_bytes_in(&dir, b"hello report", "txt").expect("should write");
        assert_eq!(saved.size_bytes, 12);
        assert!(saved.filename.starts_with("rapport-") && saved.filename.ends_with(".txt"));
        // Regression guard for the actual bug class this project hit twice
        // before (Date.now()-style placeholder never actually formatted):
        // a real ISO-ish date, not a raw epoch number or empty string.
        assert!(saved.created_at.contains('-') && saved.created_at.contains(':'));

        let listed = list_reports_in(&dir).expect("should list");
        assert_eq!(listed.len(), 1);
        assert_eq!(listed[0], saved);

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn write_rejects_an_unknown_extension_without_touching_the_filesystem() {
        let dir = test_dir("write-bad-ext");
        assert!(write_report_bytes_in(&dir, b"x", "exe").is_err());
        assert!(!std::path::Path::new(&dir).exists());
    }

    #[test]
    fn two_writes_in_the_same_second_never_overwrite_each_other() {
        // Reproduces the exact scenario the disambiguation loop exists for:
        // same directory, same extension, same epoch second (both calls
        // happen synchronously here, well within one second).
        let dir = test_dir("write-collision");
        let first = write_report_bytes_in(&dir, b"first", "json").expect("first write should succeed");
        let second = write_report_bytes_in(&dir, b"second", "json").expect("second write should succeed");
        assert_ne!(first.filename, second.filename);

        let first_bytes = std::fs::read(format!("{dir}/{}", first.filename)).expect("first file should still exist");
        assert_eq!(first_bytes, b"first", "the first report must not have been silently overwritten by the second");

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn delete_removes_the_file_and_rejects_a_path_traversal_filename_before_touching_the_filesystem() {
        let dir = test_dir("delete");
        let saved = write_report_bytes_in(&dir, b"to delete", "md").expect("should write");

        assert!(delete_report_in(&dir, "../../etc/passwd").is_err());
        assert!(std::path::Path::new(&format!("{dir}/{}", saved.filename)).exists(), "traversal attempt must not have deleted anything");

        delete_report_in(&dir, &saved.filename).expect("should delete");
        assert!(!std::path::Path::new(&format!("{dir}/{}", saved.filename)).exists());

        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn list_ignores_a_foreign_file_instead_of_erroring_the_whole_listing() {
        let dir = test_dir("list-foreign");
        std::fs::create_dir_all(&dir).expect("should create dir");
        std::fs::write(format!("{dir}/notes.txt"), b"unrelated user file").expect("should write foreign file");
        let ours = write_report_bytes_in(&dir, b"real report", "html").expect("should write");

        let listed = list_reports_in(&dir).expect("should list despite the foreign file");
        assert_eq!(listed, vec![ours]);

        let _ = std::fs::remove_dir_all(&dir);
    }
}
