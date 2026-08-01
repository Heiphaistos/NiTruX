# NiTruX Phase R5 — Rapports > Générateur de rapport — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `ReportGeneratorPlaceholder.vue` "Bientôt disponible" page with a real system-configuration report generator producing HTML, Markdown, TXT, or JSON output (spec §4.4). This is the **last** phase of the NiTruX redesign — once merged, every `categories.ts` nav entry points at a real page, no placeholders remain.

**Architecture:** A new backend module `src-tauri/src/report.rs` defines `SystemReport` — a struct aggregating 9 already-existing, already-tested, all-read-only snapshot sources (`system::build_snapshot`, `sensors::get_sensor_snapshot`, `hardware::get_pci_devices`, `drivers::get_driver_snapshot`, `disks::list_disks`, `disks::list_disk_usage`, `network::get_network_snapshot`, `firewall::get_firewall_status`, and package updates via `list_updates`) into one struct, built once per report generation. **No new privileged operations** — every data source is already a read-only command shipped in earlier phases. Fallible sources are stored as `Option<T>` (via `.ok()`) rather than defaulted to empty — a report is exactly the kind of artifact where "couldn't determine X" and "confirmed zero X" must stay distinguishable (e.g. "couldn't check for updates" vs. "confirmed 0 updates available" are very different facts for someone troubleshooting a system). Four renderer functions (`render_json`/`render_markdown`/`render_txt`/`render_html`) each take `&SystemReport` and return a `String` — JSON is nearly free via `serde_json::to_string_pretty`, the other three are hand-written template renderers over the same struct, per spec §4.4's explicit architecture. A single Tauri command `generate_system_report(state, format: String) -> Result<String, String>` builds the report and dispatches to the right renderer.

**File-save affordance:** spec §4.4 says "Tauri's file-save dialog." This plan deliberately does NOT add a new Tauri plugin dependency (`tauri-plugin-dialog` + its npm counterpart + new capability permissions) for this — introducing a new dependency and a new capability surface is exactly the kind of change this project's established discipline treats as needing explicit human review, not something to add silently in an unattended pass. Instead: the backend command returns the rendered report as a plain `String` (still exactly matching spec §4.4's own signature, `generate_system_report(format) -> String`), and the frontend triggers a standard client-side download via a `Blob` + a temporary `<a download>` element — zero new dependencies, zero new capability entries, the browser's native "Save As" behavior inside the webview accomplishes the same user-facing outcome (choose where the file goes) without a new plugin. This is a deliberate, conservative interpretation of the spec's UI intent, not a shortcut on functionality — every format is still fully generated and still ends up saved to a user-chosen location.

**Tech Stack:** Tauri v2 + Rust (backend), Vue 3.5 + TypeScript + Vitest (frontend), same patterns as Phases R1–R4.

---

## Task 1: Backend — `SystemReport` aggregation + 4 renderers + Tauri command

**Files:**
- Create: `src-tauri/src/report.rs`
- Modify: `src-tauri/src/lib.rs` (register `mod report;`, register the new command, make `list_updates` crate-visible)

### Context: what's being aggregated

Each of these already exists, is already tested, and is already read-only:

| Source | Function | Return type |
|---|---|---|
| CPU/memory/process | `system::build_snapshot(&mut System)` | `SystemSnapshot` (infallible) |
| Battery/temperature | `sensors::get_sensor_snapshot()` | `SensorSnapshot` (infallible) |
| PCI devices | `hardware::get_pci_devices()` | `Result<Vec<PciDevice>, String>` |
| Drivers | `drivers::get_driver_snapshot()` | `Result<DriverSnapshot, String>` |
| Disks | `disks::list_disks()` | `Result<Vec<Disk>, String>` |
| Disk usage | `disks::list_disk_usage()` | `Result<Vec<UsageEntry>, String>` |
| Network | `network::get_network_snapshot()` | `NetworkSnapshot` (infallible) |
| Firewall | `firewall::get_firewall_status()` | `Result<FirewallStatus, String>` |
| Package updates | `list_updates()` (currently private, defined directly in `lib.rs`) | `Result<Vec<packages::PackageUpdate>, String>` |

`system::build_snapshot` needs a live `&mut System` (the app's shared `tauri::State<Mutex<System>>>`, already managed in `lib.rs`'s `run()` — see the existing `.manage(Mutex::new(System::new_all()))` call and `system::get_system_snapshot`'s own use of that state) — `generate_system_report` needs the same `tauri::State<Mutex<System>>` parameter for the same reason.

- [ ] **Step 1: Make `list_updates` visible to the new module**

Read the live `src-tauri/src/lib.rs` first. Change:
```rust
#[tauri::command]
fn list_updates() -> Result<Vec<packages::PackageUpdate>, String> {
```
to:
```rust
#[tauri::command]
pub(crate) fn list_updates() -> Result<Vec<packages::PackageUpdate>, String> {
```
(one keyword added — `pub(crate)` — everything else about the function is unchanged; this is the only edit `list_updates` itself needs. `pub(crate)`, not `pub`, because it only needs to be visible to `report.rs`, a sibling module in the same crate — no need to widen its visibility further.)

- [ ] **Step 2: Write the failing renderer tests**

Create `src-tauri/src/report.rs` starting with just the struct definitions and a test module (the renderers referenced don't exist yet, so this step's tests will fail to compile — that's the expected "red" state):

```rust
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
        assert_eq!(parsed["process_count_check"], serde_json::Value::Null); // sanity: field doesn't exist, proves we're reading real structure below
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
```

- [ ] **Step 3: Run it, confirm it fails to compile** (renderers don't exist yet)

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator/src-tauri && cargo test report:: 2>&1 | tail -30"`
Expected: compile errors — `render_json`/`render_txt`/`render_markdown`/`render_html` not found.

- [ ] **Step 4: Implement the 4 renderers and `build_system_report`**

Add above the `#[cfg(test)]` block in `report.rs`:

```rust
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
```

`generated_at` uses a plain Unix-epoch-prefixed string (`"epoch:<seconds>"`) rather than pulling in a date-formatting crate (e.g. `chrono`) — this codebase has no existing date/time dependency, and adding one just to pretty-print a timestamp inside a report is exactly the kind of new-dependency scope creep this plan's own design section explicitly avoids elsewhere (see the file-save dialog decision above). The frontend (Task 2) is responsible for presenting this to the user in a friendlier way if desired; the raw value is still fully honest and sortable.

Remove the placeholder `assert_eq!(parsed["process_count_check"], serde_json::Value::Null);` sanity line from Step 2's test once you confirm `render_json` really does produce parseable JSON — it was there only to make the test read the parsed value at least once before Step 4 exists; once `render_json` is real, replace that line with a direct field check instead:
```rust
        assert_eq!(parsed["system"]["process_count"], 210);
```
(same test, `render_json_produces_valid_parseable_json_with_key_fields` — just swap that one assertion line for a real one now that the real structure exists to check against.)

- [ ] **Step 5: Wire `mod report;` and the new command into `lib.rs`**

Add `mod report;` to the `mod` list (alphabetically among the existing ones, next to `mod portscan;`/`mod security_write;` — insert between `mod portscan;` and `mod security_write;`).

Add `report::generate_system_report,` to the `tauri::generate_handler![...]` list, right after `detect_native_manager,` (grouping it near `list_updates`/`detect_native_manager` since it's also package/system-data-adjacent — exact position within the list doesn't matter functionally, but keep related commands visually grouped as the existing list already does).

- [ ] **Step 6: Run the full Rust test suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator/src-tauri && cargo test 2>&1 | tail -20"`
Expected: `132 passed; 0 failed; 1 ignored` (128 baseline + 4 new renderer tests).

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/report.rs src-tauri/src/lib.rs
git commit -m "feat: add SystemReport aggregation + JSON/Markdown/TXT/HTML renderers (spec section 4.4)"
```

---

## Task 2: Frontend — `ReportGeneratorPage.vue`

**Files:**
- Create: `src/pages/ReportGeneratorPage.vue`
- Test: `src/pages/ReportGeneratorPage.spec.ts`

- [ ] **Step 1: Write the failing tests**

```typescript
// src/pages/ReportGeneratorPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import ReportGeneratorPage from "./ReportGeneratorPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "generate_system_report" && args?.format === "json") {
      return Promise.resolve('{"generated_at":"epoch:123"}');
    }
    if (cmd === "generate_system_report" && args?.format === "markdown") {
      return Promise.resolve("# Rapport système NiTruX");
    }
    return Promise.resolve(null);
  }),
}));

describe("ReportGeneratorPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("generates a JSON report by default and shows a preview", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ReportGeneratorPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("generated_at"));
    expect(invoke).toHaveBeenCalledWith("generate_system_report", { format: "json" });
  });

  it("generates a Markdown report when that format is selected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ReportGeneratorPage);
    const select = wrapper.find("select");
    await select.setValue("markdown");
    const button = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Rapport système NiTruX"));
    expect(invoke).toHaveBeenCalledWith("generate_system_report", { format: "markdown" });
  });

  it("enables the download button only after a report has been generated", async () => {
    const wrapper = mount(ReportGeneratorPage);
    const downloadButtonBefore = wrapper.findAll("button").find((b) => b.text() === "Télécharger");
    expect(downloadButtonBefore?.attributes("disabled")).toBeDefined();
    const generateButton = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await generateButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("generated_at"));
    const downloadButtonAfter = wrapper.findAll("button").find((b) => b.text() === "Télécharger")!;
    expect(downloadButtonAfter.attributes("disabled")).toBeUndefined();
  });

  it("shows an error message when generation fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce("erreur de génération");
    const wrapper = mount(ReportGeneratorPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Générer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("erreur de génération"));
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && npx vitest run src/pages/ReportGeneratorPage.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 3: Write `ReportGeneratorPage.vue`**

```vue
<!-- src/pages/ReportGeneratorPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

type ReportFormat = "json" | "markdown" | "txt" | "html";

const FORMAT_OPTIONS: { value: ReportFormat; label: string }[] = [
  { value: "json", label: "JSON" },
  { value: "markdown", label: "Markdown" },
  { value: "txt", label: "Texte brut" },
  { value: "html", label: "HTML" },
];

const FORMAT_MIME: Record<ReportFormat, string> = {
  json: "application/json",
  markdown: "text/markdown",
  txt: "text/plain",
  html: "text/html",
};

const FORMAT_EXTENSION: Record<ReportFormat, string> = {
  json: "json",
  markdown: "md",
  txt: "txt",
  html: "html",
};

const selectedFormat = ref<ReportFormat>("json");
const generating = ref(false);
const error = ref<string | null>(null);
const reportContent = ref<string | null>(null);

async function generate() {
  generating.value = true;
  error.value = null;
  reportContent.value = null;
  try {
    reportContent.value = await invoke<string>("generate_system_report", { format: selectedFormat.value });
  } catch (e) {
    error.value = String(e);
  } finally {
    generating.value = false;
  }
}

function download() {
  if (!reportContent.value) return;
  const blob = new Blob([reportContent.value], { type: FORMAT_MIME[selectedFormat.value] });
  const url = URL.createObjectURL(blob);
  const link = document.createElement("a");
  link.href = url;
  link.download = `nitrux-rapport.${FORMAT_EXTENSION[selectedFormat.value]}`;
  link.click();
  URL.revokeObjectURL(url);
}
</script>

<template>
  <div class="rg-page">
    <NxSectionHeader title="Générateur de rapport" description="Exporte un état complet du système au format de votre choix." />

    <NxCard class="rg-controls">
      <div class="rg-controls-row">
        <NxSelect v-model="selectedFormat" :options="FORMAT_OPTIONS" />
        <NxButton :disabled="generating" @click="generate">{{ generating ? "Génération..." : "Générer" }}</NxButton>
        <NxButton :disabled="!reportContent" @click="download">Télécharger</NxButton>
      </div>
    </NxCard>

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard v-if="reportContent" class="rg-preview">
      <NxSectionHeader title="Aperçu" />
      <pre class="rg-preview-content">{{ reportContent }}</pre>
    </NxCard>
  </div>
</template>

<style scoped>
.rg-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.rg-controls-row { display: flex; gap: 10px; align-items: center; }
.rg-preview-content { max-height: 480px; overflow: auto; font-size: 12px; white-space: pre-wrap; word-break: break-word; margin: 0; }
</style>
```

Before pasting this in verbatim, cross-check `NxCard`, `NxButton`, `NxSelect`, `NxSectionHeader`'s actual live `defineProps` (read each file in `src/components/ui/`) against the props used here (`danger`, `disabled`, `v-model`/`options`, `title`/`description`) — adapt and clearly note any drift in your final report. `NxSelect` was used in `PackagesPage.vue`/`QuickInstallPage.vue` earlier in this redesign — check its exact `options` shape (`{ value, label }[]`) matches what's used above.

- [ ] **Step 4: Run it, confirm it PASSES**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && npx vitest run src/pages/ReportGeneratorPage.spec.ts"`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add src/pages/ReportGeneratorPage.vue src/pages/ReportGeneratorPage.spec.ts
git commit -m "feat: add ReportGeneratorPage — JSON/Markdown/TXT/HTML export with client-side download (spec section 4.4)"
```

---

## Task 3: Wire `App.vue` to the real `ReportGeneratorPage`, retire the placeholder

**Files:**
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`
- Delete: `src/pages/ReportGeneratorPlaceholder.vue` (`git rm`)

Same pattern as R3 Task 4 / R4 Task 4. Read the live `src/App.vue` and `src/App.spec.ts` first.

- [ ] **Step 1: Add a test to `App.spec.ts`**

```typescript
  it("shows the real ReportGeneratorPage (not a placeholder) for the report-generator id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const reportButton = buttons.find((b) => b.text() === "Générateur de rapport")!;
    await reportButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R5");
  });
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && npx vitest run src/App.spec.ts"`

- [ ] **Step 3: Update `App.vue`**

Replace the import:
```typescript
import ReportGeneratorPage from "@/pages/ReportGeneratorPage.vue";
```
(replaces `import ReportGeneratorPlaceholder from "@/pages/ReportGeneratorPlaceholder.vue";`)

And the map entry:
```typescript
  "report-generator": ReportGeneratorPage,
```
(replaces `"report-generator": ReportGeneratorPlaceholder,` — every other line unchanged)

- [ ] **Step 4: Delete the placeholder**

```bash
git rm src/pages/ReportGeneratorPlaceholder.vue
```

Also delete `src/pages/ComingSoonPage.vue` and its spec **only if** nothing else references it anymore — check first:
```bash
wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && grep -rln 'ComingSoonPage' src/ --include='*.vue' --include='*.ts' | grep -v ComingSoonPage"
```
If this prints nothing (no other file imports `ComingSoonPage.vue` anymore, since this was the last of the 3 placeholder wrappers that used it), then `ComingSoonPage.vue` and `ComingSoonPage.spec.ts` are now dead code and should also be removed in this same step:
```bash
git rm src/pages/ComingSoonPage.vue src/pages/ComingSoonPage.spec.ts
```
If the grep DOES print a match, leave `ComingSoonPage.vue` in place and just note in your final report which file still references it — do not guess or force removal.

- [ ] **Step 5: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && npx vitest run src/App.spec.ts"`
Expected: PASS (6 tests — 5 from before + this task's new one). If `ComingSoonPage.spec.ts` was removed in Step 4, the overall suite total in Task 4 will be 1 test lower than a naive addition would suggest — account for this explicitly in Task 4's expected count.

- [ ] **Step 6: Commit**

```bash
git add src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue to the real ReportGeneratorPage, retire the last placeholder (spec section 4.4)"
```

---

## Task 4: Full verification pass — and confirmation the entire redesign (R1–R5) is complete

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && npm run test -- --run 2>&1 | tail -20"`
Expected: baseline entering this plan was 124 (end of R4). This plan adds: Task 2 (4 tests) + Task 3 (1 new test) = 5, **minus 1 test if `ComingSoonPage.spec.ts` was removed in Task 3 Step 4** (it had exactly 1 test, from R2 Task 8). So expected total is either 129 (if `ComingSoonPage.spec.ts` stayed) or 128 (if it was removed, which is the expected outcome since R5 is the last phase to use it). Report the real observed number and reconcile against which case applies.

- [ ] **Step 2: Type-check**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && npx vue-tsc --noEmit"`
Expected: clean.

- [ ] **Step 3: Confirm the Rust suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator/src-tauri && cargo test 2>&1 | tail -10"`
Expected: `132 passed; 0 failed; 1 ignored` (128 baseline + 4 new from Task 1).

- [ ] **Step 4: Confirm `ReportGeneratorPlaceholder.vue` is fully gone**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && grep -rn 'ReportGeneratorPlaceholder' src/ || echo 'no references found'"`
Expected: `no references found`.

- [ ] **Step 5: Confirm the entire redesign is complete — zero placeholders remain**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r5-report-generator && grep -rln 'Placeholder' src/pages/ || echo 'no placeholder files remain'"`
Expected: `no placeholder files remain` — `QuickInstallPlaceholder.vue` (removed in R3), `UpdatesPlaceholder.vue` (removed in R4), and `ReportGeneratorPlaceholder.vue` (removed in this plan) should ALL be gone. This is the signal that every one of the 15 `categories.ts` page ids now points at a real, implemented page — the redesign's original spec (`docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md`) is now fully implemented end to end.

- [ ] **Step 6: Commit any final cleanup**

No further commit expected if Steps 1–5 all pass clean.

---

## Self-Review

**Spec coverage:** §4.4's every named element is covered — new `generate_system_report(format) -> String` command (Task 1, matches the spec's exact signature), aggregation of all 9 named read-only snapshot commands into one `SystemReport` struct (Task 1), 4 renderers sharing a common intermediate struct with JSON via `Serialize` derive being "almost free" (Task 1, `render_json` is a 1-line `serde_json::to_string_pretty` call vs. the ~40-line hand-written renderers for the other 3 formats), "no new privileged operations" (Task 1 — every source is `.ok()`'d from an already-existing read-only command, nothing new is invoked with elevated privileges), "Générer" button + format picker + "voir le rapport" affordance (Task 2's inline `<pre>` preview) + save-to-user-chosen-path (Task 2's Blob-download, with the file-save-dialog deviation explicitly justified in this plan's Architecture section rather than silently substituted).

**Placeholder scan:** No "TBD"/"TODO". The `generated_at: "epoch:<seconds>"` format is a deliberate, explicitly-justified simplification (no new date-formatting dependency), not an oversight — noted inline in Task 1.

**Type consistency:** `SystemReport`'s field types (Task 1) map directly to the already-existing Rust structs (`SystemSnapshot`, `SensorSnapshot`, `PciDevice`, `DriverSnapshot`, `Disk`, `UsageEntry`, `NetworkSnapshot`, `FirewallStatus`, `PackageUpdate`) with zero redefinition — every field is a `.ok()`-wrapped or direct reuse of a type this codebase already tests elsewhere, not a parallel/duplicate type. `ReportFormat` (TypeScript, Task 2: `"json" | "markdown" | "txt" | "html"`) matches the Rust `format: String` command parameter's 4 accepted literal values (`"json"`/`"markdown"`/`"txt"`/`"html"`) exactly — cross-checked against Task 1's `match format.as_str() { ... }` arms. `Nx*` component prop names match their R1 `defineProps` exactly, per every prior phase's same cross-check discipline.
