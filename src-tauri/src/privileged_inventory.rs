//! Read-only information only root can read, fetched through the
//! `org.heiphaistos.nitrux.inventory` pkexec action (`auth_admin_keep`: one
//! password prompt covers several reads for a few minutes). Every command
//! is fixed in the helper; nothing here is ever written or changed. This is
//! the local machine's own administrator inspecting their own machine's
//! posture, the way `lynis`/`dmidecode`/`smartctl` do — the helper only
//! reports states and file *paths*, never the contents of any secret.

use crate::hardware_inventory::{human_bytes, Device, KeyValue};
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

const HELPER: &str = "/usr/bin/nitrux-pkexec-inventory";

fn run(what: &[&str], timeout_secs: u64) -> Result<String, String> {
    let mut args = vec![HELPER, "inventory"];
    args.extend_from_slice(what);
    subprocess::run_with_timeout("pkexec", &args, Duration::from_secs(timeout_secs))
}

fn kv(key: &str, value: impl Into<String>) -> KeyValue {
    KeyValue { key: key.to_string(), value: value.into() }
}

// ── Memory modules (dmidecode -t 16 -t 17) ─────────────────────────────────

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct MemoryModules {
    pub max_capacity: Option<String>,
    pub slots_total: Option<usize>,
    /// Populated slots, one Device each (size, type, speed, manufacturer…).
    pub modules: Vec<Device>,
    pub empty_slots: Vec<String>,
}

/// dmidecode prints one block per handle; within a block, "  Key: Value".
fn dmi_blocks(output: &str) -> Vec<Vec<(String, String)>> {
    let mut blocks = Vec::new();
    let mut current: Vec<(String, String)> = Vec::new();
    for line in output.lines() {
        if line.starts_with("Handle ") {
            if !current.is_empty() {
                blocks.push(std::mem::take(&mut current));
            }
        } else if let Some((k, v)) = line.trim().split_once(": ") {
            current.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    if !current.is_empty() {
        blocks.push(current);
    }
    blocks
}

pub fn parse_dmi_memory(output: &str) -> MemoryModules {
    let mut result = MemoryModules::default();
    for block in dmi_blocks(output) {
        let get = |key: &str| block.iter().find(|(k, _)| k == key).map(|(_, v)| v.clone());
        // Physical Memory Array (type 16): total capacity and slot count.
        if let Some(max) = get("Maximum Capacity") {
            result.max_capacity = Some(max);
            if let Some(n) = get("Number Of Devices").and_then(|v| v.parse().ok()) {
                result.slots_total = Some(n);
            }
            continue;
        }
        // Memory Device (type 17): one per slot.
        let Some(locator) = get("Locator") else { continue };
        let size = get("Size").unwrap_or_default();
        let absent = size.is_empty() || size.eq_ignore_ascii_case("No Module Installed") || size == "0";
        if absent {
            result.empty_slots.push(locator);
            continue;
        }
        let mut details = vec![kv("Emplacement", locator.clone())];
        for (dmi_key, label) in [
            ("Size", "Capacité"),
            ("Type", "Type"),
            ("Speed", "Fréquence"),
            ("Configured Memory Speed", "Fréquence configurée"),
            ("Manufacturer", "Fabricant"),
            ("Part Number", "Référence"),
            ("Serial Number", "N° de série"),
            ("Rank", "Rang"),
            ("Form Factor", "Format"),
        ] {
            if let Some(v) = get(dmi_key) {
                if !v.is_empty() && v != "Unknown" && v != "Not Specified" {
                    details.push(kv(label, v));
                }
            }
        }
        result.modules.push(Device { name: get("Part Number").filter(|p| !p.is_empty() && p != "Unknown").unwrap_or(locator), details });
    }
    result
}

#[tauri::command]
pub fn get_memory_modules() -> Result<MemoryModules, String> {
    Ok(parse_dmi_memory(&run(&["dmi-memory"], 60)?))
}

// ── OEM Windows key from firmware (ACPI MSDM) ──────────────────────────────

/// The machine's own embedded OEM Windows product key, if the firmware
/// carries one (dual-boot / re-imaged ex-Windows machines). NiTriTe shows
/// this so a technician can recover the licence that shipped with the
/// hardware. Empty when there is none.
#[tauri::command]
pub fn get_firmware_windows_key() -> Result<Option<String>, String> {
    let out = run(&["firmware-key"], 30)?;
    let key = out.trim();
    // A valid product key is 5 groups of 5 in [A-Z0-9-].
    let looks_like_key = key.len() == 29 && key.split('-').count() == 5 && key.chars().all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '-');
    Ok(looks_like_key.then(|| key.to_string()))
}

// ── SMART health of one disk ───────────────────────────────────────────────

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct SmartReport {
    pub device: String,
    pub healthy: Option<bool>,
    pub attributes: Vec<KeyValue>,
}

pub fn parse_smart(device: &str, output: &str) -> SmartReport {
    let mut report = SmartReport { device: device.to_string(), ..Default::default() };
    let lower = output.to_lowercase();
    if lower.contains("passed") {
        report.healthy = Some(true);
    } else if lower.contains("failed") || lower.contains("failing") {
        report.healthy = Some(false);
    }
    let want = |line: &str, needle: &str| line.to_lowercase().contains(needle);
    for line in output.lines() {
        // NVMe health section: "Percentage Used: 3%", "Power On Hours: ..."
        for (needle, label) in [
            ("temperature:", "Température"),
            ("power on hours", "Heures de fonctionnement"),
            ("power_on_hours", "Heures de fonctionnement"),
            ("power cycles", "Cycles d'alimentation"),
            ("power_cycle_count", "Cycles d'alimentation"),
            ("percentage used", "Usure estimée"),
            ("available spare:", "Réserve disponible"),
            ("media and data integrity errors", "Erreurs d'intégrité"),
            ("reallocated_sector", "Secteurs réalloués"),
            ("wear_leveling", "Nivellement d'usure"),
            ("total_lbas_written", "Données écrites (LBA)"),
            ("unsafe_shutdown", "Arrêts brutaux"),
        ] {
            if want(line, needle) {
                let value = line.split(':').nth(1).map(str::trim).unwrap_or("").to_string();
                let value = if value.is_empty() {
                    // SMART attribute table: value is the last column.
                    line.split_whitespace().last().unwrap_or("").to_string()
                } else {
                    value
                };
                if !value.is_empty() {
                    report.attributes.push(kv(label, value));
                }
                break;
            }
        }
    }
    report
}

#[tauri::command]
pub fn get_smart_report(device: String) -> Result<SmartReport, String> {
    Ok(parse_smart(&device, &run(&["smart", &device], 30)?))
}

// ── Security posture audit ─────────────────────────────────────────────────

#[derive(Serialize, Clone, Debug, Default, PartialEq)]
pub struct SecurityFinding {
    /// "critical", "warning" or "info".
    pub severity: String,
    pub title: String,
    pub detail: String,
}

fn finding(severity: &str, title: &str, detail: impl Into<String>) -> SecurityFinding {
    SecurityFinding { severity: severity.to_string(), title: title.to_string(), detail: detail.into() }
}

/// Turns the helper's sectioned audit output into findings. The helper only
/// ever reports states and file *paths* (never secret contents), so a
/// technician sees "you have a private key at /root/.ssh/id_rsa" and can go
/// clean it up — the standard job of a security-posture scan.
pub fn parse_security_audit(output: &str) -> Vec<SecurityFinding> {
    let mut sections: std::collections::HashMap<&str, Vec<String>> = std::collections::HashMap::new();
    let mut current = "";
    for line in output.lines() {
        if let Some(name) = line.strip_prefix("### ") {
            current = name.trim();
            sections.entry(current).or_default();
        } else if !current.is_empty() && !line.trim().is_empty() {
            sections.entry(current).or_default().push(line.trim().to_string());
        }
    }
    let get = |k: &str| sections.get(k).cloned().unwrap_or_default();
    let mut findings = Vec::new();

    for line in get("sshd") {
        let (key, value) = line.split_once(' ').unwrap_or((line.as_str(), ""));
        match (key, value) {
            ("permitrootlogin", "yes") => findings.push(finding("critical", "SSH : connexion root autorisée", "PermitRootLogin yes — désactivez la connexion directe en root.")),
            ("passwordauthentication", "yes") => findings.push(finding("warning", "SSH : authentification par mot de passe", "PasswordAuthentication yes — préférez les clés.")),
            ("permitemptypasswords", "yes") => findings.push(finding("critical", "SSH : mots de passe vides autorisés", "PermitEmptyPasswords yes.")),
            _ => {}
        }
    }
    for user in get("empty-passwords") {
        findings.push(finding("critical", "Compte sans mot de passe", format!("Le compte « {user} » n'a aucun mot de passe.")));
    }
    let uid0: Vec<String> = get("uid0").into_iter().filter(|u| u != "root").collect();
    if !uid0.is_empty() {
        findings.push(finding("critical", "Compte administrateur caché (UID 0)", format!("En plus de root : {}", uid0.join(", "))));
    }
    for entry in get("sudo-nopasswd") {
        findings.push(finding("warning", "sudo sans mot de passe", entry));
    }
    let suid = get("suid-unusual");
    if !suid.is_empty() {
        findings.push(finding("warning", "Binaire SUID à un endroit inhabituel", format!("À vérifier :\n{}", suid.join("\n"))));
    }
    let ww = get("world-writable-etc");
    if !ww.is_empty() {
        findings.push(finding("warning", "Fichier de configuration modifiable par tous", ww.join("\n")));
    }
    let cron = get("root-crontab");
    if !cron.is_empty() {
        findings.push(finding("info", "Tâche planifiée root", cron.join("\n")));
    }
    let secrets = get("root-secrets");
    if !secrets.is_empty() {
        findings.push(finding("info", "Clés ou secrets stockés en clair", format!("Fichiers sensibles présents (à sécuriser ou déplacer) :\n{}", secrets.join("\n"))));
    }
    if findings.is_empty() {
        findings.push(finding("info", "Aucune faiblesse évidente détectée", "Les vérifications de base sont passées."));
    }
    findings
}

#[tauri::command]
pub fn run_security_audit() -> Result<Vec<SecurityFinding>, String> {
    Ok(parse_security_audit(&run(&["security-audit"], 120)?))
}

// A small re-export so the frontend gets a consistent human size helper if
// it ever needs one server-side; keeps `human_bytes` referenced.
#[allow(dead_code)]
pub fn format_size(bytes: u64) -> String {
    human_bytes(bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dmidecode_memory_into_populated_and_empty_slots() {
        let out = "Handle 0x0037, DMI type 16, 23 bytes\nPhysical Memory Array\n\tMaximum Capacity: 64 GB\n\tNumber Of Devices: 4\n\nHandle 0x0039, DMI type 17, 92 bytes\nMemory Device\n\tSize: 16384 MB\n\tLocator: DIMM 0\n\tType: DDR4\n\tSpeed: 3200 MT/s\n\tManufacturer: Corsair\n\tPart Number: CMK32GX4M2\n\nHandle 0x003B, DMI type 17, 92 bytes\nMemory Device\n\tSize: No Module Installed\n\tLocator: DIMM 1\n";
        let m = parse_dmi_memory(out);
        assert_eq!(m.max_capacity.as_deref(), Some("64 GB"));
        assert_eq!(m.slots_total, Some(4));
        assert_eq!(m.modules.len(), 1);
        assert_eq!(m.empty_slots, vec!["DIMM 1"]);
        assert!(m.modules[0].details.iter().any(|d| d.key == "Type" && d.value == "DDR4"));
    }

    #[test]
    fn reads_smart_health_and_a_couple_of_attributes() {
        let out = "SMART overall-health self-assessment test result: PASSED\nPower On Hours: 12345\nPercentage Used: 3%\n";
        let r = parse_smart("/dev/nvme0", &out);
        assert_eq!(r.healthy, Some(true));
        assert!(r.attributes.iter().any(|a| a.key == "Usure estimée" && a.value == "3%"));
        let bad = parse_smart("/dev/sda", "SMART overall-health self-assessment test result: FAILED!\n");
        assert_eq!(bad.healthy, Some(false));
    }

    #[test]
    fn turns_the_audit_sections_into_severity_ranked_findings() {
        let out = "### sshd\npermitrootlogin yes\npasswordauthentication yes\n### empty-passwords\ntest\n### uid0\nroot\nbackdoor\n### sudo-nopasswd\n### suid-unusual\n/opt/weird/tool\n### world-writable-etc\n### root-crontab\n### root-secrets\n/root/.ssh/id_rsa\n### end\n";
        let f = parse_security_audit(out);
        assert!(f.iter().any(|x| x.severity == "critical" && x.title.contains("root autorisée")));
        assert!(f.iter().any(|x| x.severity == "critical" && x.detail.contains("backdoor")));
        assert!(f.iter().any(|x| x.title.contains("SUID") && x.detail.contains("/opt/weird/tool")));
        assert!(f.iter().any(|x| x.detail.contains("/root/.ssh/id_rsa")));
    }

    #[test]
    fn a_clean_machine_reports_no_weakness() {
        let f = parse_security_audit("### sshd\n### empty-passwords\n### uid0\nroot\n### end\n");
        assert_eq!(f.len(), 1);
        assert_eq!(f[0].severity, "info");
    }
}
