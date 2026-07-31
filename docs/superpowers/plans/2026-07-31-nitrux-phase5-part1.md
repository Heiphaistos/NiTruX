# NiTruX Phase 5 Part 1 — Security/Maintenance Diagnostics (read-only) + Backlog Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Round out the "Réseau, sécurité & maintenance" pillar's remaining read-only diagnostics — UFW firewall status, ClamAV malware scan (report-only, never quarantines/deletes), and Btrfs/Timeshift snapshot listing — plus close two small backlog items flagged during Phase 1's review (hardcoded `BAT0`, non-unique Vue `:key`).

**Architecture:** `firewall.rs` (UFW status), `malwarescan.rs` (ClamAV scan, report-only), `snapshots.rs` (Btrfs/Timeshift listing) — three new modules, each following the established `subprocess::run_with_timeout` + `Result<T, String>` convention. Backlog fixes touch existing `sensors.rs` and `DashboardPage.vue` directly, no new files.

**Tech Stack:** Same as Phase 1-4 — Tauri v2, Rust, Vue 3 + TypeScript + Pinia, Vitest, `cargo test`.

**Scope note — READ-ONLY by design, intentional, same reasoning as every prior phase:** This plan explicitly excludes anything that modifies system state: no quarantine/delete of malware findings, no firewall rule changes, no snapshot creation/restoration/deletion. The malware scanner in particular is designed to be unambiguously safe — it reports what ClamAV finds, nothing more; acting on findings (quarantine, delete) is a privileged, destructive-adjacent operation requiring human-reviewed design, explicitly out of scope here, same as every write-capable operation deferred across Phases 2-4.

---

## File Structure

```
src-tauri/src/
├── firewall.rs      # UFW status (ufw status verbose)
├── malwarescan.rs     # ClamAV scan, report-only
└── snapshots.rs          # Btrfs/Timeshift snapshot listing
src/
└── pages/
    └── SecurityPage.vue      # Tabbed: Pare-feu | Scan malware | Snapshots
```

---

## Task 1: Backlog fix — multi-battery support (`BAT0` → glob)

**Files:**
- Modify: `src-tauri/src/sensors.rs`

This closes the backlog item from Phase 1 Task 9's code review: `read_battery()` only checks `/sys/class/power_supply/BAT0`, missing `BAT1` on multi-battery laptops (some ThinkPads and similar expose two battery bays, only one of which may be populated as `BAT1` depending on configuration).

- [ ] **Step 1: Write the failing test**

```rust
// src-tauri/src/sensors.rs — add to the existing #[cfg(test)] mod tests block
    #[test]
    fn finds_first_available_battery_dir_preferring_lower_numbers() {
        // find_battery_dir scans /sys/class/power_supply for entries starting
        // with "BAT", returning the lexicographically-first match (BAT0 before
        // BAT1) when multiple exist. This test uses a fake base directory
        // populated with fixture subdirectories rather than touching the real
        // /sys, so it works identically on any machine including this dev host.
        let base = std::env::temp_dir().join(format!("nitrux-batt-test-{}", std::process::id()));
        std::fs::create_dir_all(base.join("BAT1")).unwrap();
        std::fs::create_dir_all(base.join("BAT0")).unwrap();
        std::fs::create_dir_all(base.join("AC")).unwrap();

        let found = find_battery_dir_in(&base);
        std::fs::remove_dir_all(&base).ok();

        assert_eq!(found, Some(base.join("BAT0")));
    }

    #[test]
    fn returns_none_when_no_battery_present() {
        let base = std::env::temp_dir().join(format!("nitrux-batt-empty-{}", std::process::id()));
        std::fs::create_dir_all(base.join("AC")).unwrap();

        let found = find_battery_dir_in(&base);
        std::fs::remove_dir_all(&base).ok();

        assert_eq!(found, None);
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test sensors:: 2>&1 | tail -30`
Expected: FAIL — `cannot find function 'find_battery_dir_in'`

- [ ] **Step 3: Implement the fix**

Modify `src-tauri/src/sensors.rs` — replace the hardcoded `BAT0` path logic:

```rust
use std::path::{Path, PathBuf};

/// Scans `base` for the first `BAT*`-prefixed subdirectory, sorted
/// lexicographically (so BAT0 is preferred over BAT1 when both exist).
/// Returns None if no battery directory is present at all (desktops,
/// or laptops where the kernel doesn't expose one this way).
pub fn find_battery_dir_in(base: &Path) -> Option<PathBuf> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(base)
        .ok()?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with("BAT"))
                .unwrap_or(false)
        })
        .collect();
    entries.sort();
    entries.into_iter().next()
}

fn find_battery_dir() -> Option<PathBuf> {
    find_battery_dir_in(Path::new("/sys/class/power_supply"))
}
```

Then update `read_battery()` to use `find_battery_dir()` instead of the hardcoded `Path::new("/sys/class/power_supply/BAT0")`:

```rust
fn read_battery() -> (Option<u8>, Option<bool>) {
    let Some(base) = find_battery_dir() else {
        return (None, None);
    };
    let capacity = fs::read_to_string(base.join("capacity"))
        .ok()
        .and_then(|s| parse_capacity(&s));
    let status = fs::read_to_string(base.join("status"))
        .ok()
        .map(|s| s.trim().eq_ignore_ascii_case("charging"));
    (capacity, status)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test sensors:: 2>&1 | tail -30`
Expected: PASS (all sensors tests, including the 2 new ones)

- [ ] **Step 5: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 77 (pre-existing) + 2 = 79 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/sensors.rs
git commit -m "fix: support BAT1+ multi-battery laptops, not just BAT0"
```

---

## Task 2: Backlog fix — unique Vue keys for temperature readings

**Files:**
- Modify: `src/pages/DashboardPage.vue`

Closes the second backlog item from Phase 1 Task 9's review: `:key="t.label"` on the temperature `v-for` isn't guaranteed unique (two different sensor chips can report components with the same label on real hardware), which can trigger silent Vue key-collision warnings.

- [ ] **Step 1: Fix the key binding**

Modify `src/pages/DashboardPage.vue` — change:

```html
<div class="dash-card" v-for="t in sensors?.temperatures ?? []" :key="t.label">
```

to:

```html
<div class="dash-card" v-for="(t, i) in sensors?.temperatures ?? []" :key="`${t.label}-${i}`">
```

- [ ] **Step 2: Type-check**

Run: `npx vue-tsc --noEmit`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/pages/DashboardPage.vue
git commit -m "fix: ensure unique Vue keys for temperature readings (labels can collide)"
```

---

## Task 3: UFW firewall status (read-only)

**Files:**
- Create: `src-tauri/src/firewall.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/firewall.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ufw_active_status_line() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output);
        assert!(status.active);
        assert_eq!(status.rules.len(), 1);
        assert_eq!(status.rules[0], "22/tcp ALLOW Anywhere");
    }

    #[test]
    fn parses_ufw_inactive_status() {
        let output = "Status: inactive\n";
        let status = parse_ufw_output(output);
        assert!(!status.active);
        assert!(status.rules.is_empty());
    }

    #[test]
    fn parses_multiple_ufw_rules() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n80/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output);
        assert_eq!(status.rules.len(), 2);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test firewall:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `firewall.rs`**

```rust
// src-tauri/src/firewall.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct FirewallStatus {
    pub active: bool,
    pub rules: Vec<String>,
}

/// Parses `ufw status` output. Rule lines are normalized to single-spaced
/// "To Action From" (the raw output is column-padded with variable
/// whitespace, which we collapse for a clean, consistent display string).
pub fn parse_ufw_output(output: &str) -> FirewallStatus {
    let active = output
        .lines()
        .next()
        .map(|l| l.trim() == "Status: active")
        .unwrap_or(false);

    if !active {
        return FirewallStatus { active: false, rules: Vec::new() };
    }

    let rules = output
        .lines()
        .skip_while(|l| !l.starts_with("--"))
        .skip(1)
        .filter(|l| !l.trim().is_empty())
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .collect();

    FirewallStatus { active, rules }
}

#[tauri::command]
pub fn get_firewall_status() -> Result<FirewallStatus, String> {
    let output = subprocess::run_with_timeout("ufw", &["status"], Duration::from_secs(5))?;
    Ok(parse_ufw_output(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_ufw_active_status_line() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output);
        assert!(status.active);
        assert_eq!(status.rules.len(), 1);
        assert_eq!(status.rules[0], "22/tcp ALLOW Anywhere");
    }

    #[test]
    fn parses_ufw_inactive_status() {
        let output = "Status: inactive\n";
        let status = parse_ufw_output(output);
        assert!(!status.active);
        assert!(status.rules.is_empty());
    }

    #[test]
    fn parses_multiple_ufw_rules() {
        let output = "Status: active\n\nTo                         Action      From\n--                         ------      ----\n22/tcp                     ALLOW       Anywhere\n80/tcp                     ALLOW       Anywhere\n";
        let status = parse_ufw_output(output);
        assert_eq!(status.rules.len(), 2);
    }
}
```

Note `get_firewall_status` returns `Result`, not an infallible default like `network.rs`/`docker.rs` — reading UFW status commonly requires root (`ufw status` without privileges may print a permission error or simply fail), and unlike Wi-Fi/Docker absence (a normal, expected state on many machines), a permission or missing-binary failure here is worth surfacing to the user explicitly via the existing try/catch pattern, not silently hiding behind an empty result.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test firewall:: 2>&1 | tail -30`
Expected: PASS (3 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod firewall;` and `firewall::get_firewall_status` to `generate_handler!`, additively alongside all existing modules.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 79 (after Tasks 1-2) + 3 = 82 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/firewall.rs src-tauri/src/lib.rs
git commit -m "feat: UFW firewall status (read-only)"
```

---

## Task 4: ClamAV malware scan (report-only)

**Files:**
- Create: `src-tauri/src/malwarescan.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/malwarescan.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clamscan_infected_line() {
        let line = "/home/user/eicar.txt: Win.Test.EICAR_HDB-1 FOUND";
        let finding = parse_clamscan_line(line).expect("should parse");
        assert_eq!(finding.path, "/home/user/eicar.txt");
        assert_eq!(finding.signature, "Win.Test.EICAR_HDB-1");
    }

    #[test]
    fn skips_clean_clamscan_line() {
        assert!(parse_clamscan_line("/home/user/file.txt: OK").is_none());
    }

    #[test]
    fn skips_summary_lines() {
        assert!(parse_clamscan_line("----------- SCAN SUMMARY -----------").is_none());
        assert!(parse_clamscan_line("Infected files: 0").is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test malwarescan:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `malwarescan.rs`**

```rust
// src-tauri/src/malwarescan.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct MalwareFinding {
    pub path: String,
    pub signature: String,
}

/// Parses one line of `clamscan -r --infected` output, e.g.:
/// "/home/user/eicar.txt: Win.Test.EICAR_HDB-1 FOUND"
/// Clean files aren't printed at all with `--infected`, but this parser
/// also defensively skips "OK" and summary lines in case that flag isn't
/// honored by a given clamscan version.
pub fn parse_clamscan_line(line: &str) -> Option<MalwareFinding> {
    if !line.ends_with("FOUND") {
        return None;
    }
    let (path, rest) = line.split_once(": ")?;
    let signature = rest.trim_end_matches("FOUND").trim();
    Some(MalwareFinding {
        path: path.to_string(),
        signature: signature.to_string(),
    })
}

/// Report-only: this command NEVER deletes, quarantines, or otherwise
/// modifies any file it scans. `clamscan` (without `--remove`/`--move`) is
/// purely read-only against the scanned filesystem. Findings are surfaced
/// to the user for them to act on manually — acting automatically on a
/// malware finding (delete/quarantine) is exactly the kind of
/// destructive-adjacent operation this plan's scope note excludes.
#[tauri::command]
pub fn scan_for_malware(directory: String) -> Result<Vec<MalwareFinding>, String> {
    let output = subprocess::run_with_timeout(
        "clamscan",
        &["-r", "--infected", "--no-summary", &directory],
        Duration::from_secs(120),
    )?;
    Ok(output.lines().filter_map(parse_clamscan_line).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_clamscan_infected_line() {
        let line = "/home/user/eicar.txt: Win.Test.EICAR_HDB-1 FOUND";
        let finding = parse_clamscan_line(line).expect("should parse");
        assert_eq!(finding.path, "/home/user/eicar.txt");
        assert_eq!(finding.signature, "Win.Test.EICAR_HDB-1");
    }

    #[test]
    fn skips_clean_clamscan_line() {
        assert!(parse_clamscan_line("/home/user/file.txt: OK").is_none());
    }

    #[test]
    fn skips_summary_lines() {
        assert!(parse_clamscan_line("----------- SCAN SUMMARY -----------").is_none());
        assert!(parse_clamscan_line("Infected files: 0").is_none());
    }
}
```

Note the 120-second timeout — much longer than every other command in the codebase (which use 5-20s), because scanning a real directory tree for malware signatures is a genuinely slow operation proportional to the number/size of files, unlike every other subprocess call in this project which queries fast, bounded system state. This is a deliberate exception to the "keep timeouts short" pattern, not an oversight.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test malwarescan:: 2>&1 | tail -30`
Expected: PASS (3 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod malwarescan;` and `malwarescan::scan_for_malware` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 82 + 3 = 85 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/malwarescan.rs src-tauri/src/lib.rs
git commit -m "feat: ClamAV malware scan (report-only, never modifies scanned files)"
```

---

## Task 5: Btrfs/Timeshift snapshot listing

**Files:**
- Create: `src-tauri/src/snapshots.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/snapshots.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timeshift_list_line() {
        let line = "1 > 2026-07-30_23-00-01                     O";
        let snap = parse_timeshift_line(line).expect("should parse");
        assert_eq!(snap.id, "1");
        assert_eq!(snap.date, "2026-07-30_23-00-01");
    }

    #[test]
    fn skips_timeshift_header_line() {
        assert!(parse_timeshift_line("Num     Name                               Tags").is_none());
    }

    #[test]
    fn skips_empty_lines() {
        assert!(parse_timeshift_line("").is_none());
        assert!(parse_timeshift_line("   ").is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test snapshots:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `snapshots.rs`**

```rust
// src-tauri/src/snapshots.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct Snapshot {
    pub id: String,
    pub date: String,
}

/// Parses one line of `timeshift --list` output, e.g.:
/// "1 > 2026-07-30_23-00-01                     O"
/// (id, optional ">" marker for the current default, timestamp, tags).
/// Timeshift's list format varies slightly by version in whether the ">"
/// marker is present; this handles both by looking for the first
/// timestamp-shaped token (YYYY-MM-DD_HH-MM-SS) rather than a fixed
/// column position.
pub fn parse_timeshift_line(line: &str) -> Option<Snapshot> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.is_empty() {
        return None;
    }
    let id = fields[0];
    if !id.chars().next()?.is_ascii_digit() {
        return None; // header row or other non-data line
    }
    let date = fields
        .iter()
        .find(|f| f.len() == 19 && f.chars().nth(4) == Some('-'))?
        .to_string();
    Some(Snapshot { id: id.to_string(), date })
}

#[tauri::command]
pub fn list_snapshots() -> Result<Vec<Snapshot>, String> {
    let output = subprocess::run_with_timeout("timeshift", &["--list"], Duration::from_secs(10))?;
    Ok(output.lines().filter_map(parse_timeshift_line).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_timeshift_list_line() {
        let line = "1 > 2026-07-30_23-00-01                     O";
        let snap = parse_timeshift_line(line).expect("should parse");
        assert_eq!(snap.id, "1");
        assert_eq!(snap.date, "2026-07-30_23-00-01");
    }

    #[test]
    fn skips_timeshift_header_line() {
        assert!(parse_timeshift_line("Num     Name                               Tags").is_none());
    }

    #[test]
    fn skips_empty_lines() {
        assert!(parse_timeshift_line("").is_none());
        assert!(parse_timeshift_line("   ").is_none());
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test snapshots:: 2>&1 | tail -30`
Expected: PASS (3 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod snapshots;` and `snapshots::list_snapshots` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 85 + 3 = 88 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/snapshots.rs src-tauri/src/lib.rs
git commit -m "feat: Btrfs/Timeshift snapshot listing"
```

---

## Task 6: `SecurityPage.vue` frontend

**Files:**
- Create: `src/pages/SecurityPage.vue`

- [ ] **Step 1: Build the tabbed page**

Follows the established tabbed pattern and try/catch + visible error ref pattern (all three commands here return `Result`, unlike `network.rs`/`docker.rs`'s infallible design — every action needs its own error ref).

```vue
<!-- src/pages/SecurityPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface FirewallStatus { active: boolean; rules: string[] }
interface MalwareFinding { path: string; signature: string }
interface SnapshotInfo { id: string; date: string }

type Tab = "firewall" | "malware" | "snapshots";
const activeTab = ref<Tab>("firewall");

const firewall = ref<FirewallStatus | null>(null);
const firewallError = ref<string | null>(null);

async function loadFirewall() {
  firewallError.value = null;
  try {
    firewall.value = await invoke<FirewallStatus>("get_firewall_status");
  } catch (e) {
    firewallError.value = String(e);
  }
}
onMounted(loadFirewall);

const scanDir = ref("");
const findings = ref<MalwareFinding[]>([]);
const scanError = ref<string | null>(null);
const scanning = ref(false);
const scanDone = ref(false);

async function runScan() {
  scanning.value = true;
  scanError.value = null;
  scanDone.value = false;
  try {
    findings.value = await invoke<MalwareFinding[]>("scan_for_malware", { directory: scanDir.value });
    scanDone.value = true;
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}

const snapshots = ref<SnapshotInfo[]>([]);
const snapshotsError = ref<string | null>(null);

async function loadSnapshots() {
  snapshotsError.value = null;
  try {
    snapshots.value = await invoke<SnapshotInfo[]>("list_snapshots");
  } catch (e) {
    snapshotsError.value = String(e);
  }
}

function onTabClick(tab: Tab) {
  activeTab.value = tab;
  if (tab === "snapshots" && snapshots.value.length === 0 && !snapshotsError.value) {
    loadSnapshots();
  }
}
</script>

<template>
  <div class="sec-page">
    <h1>Sécurité & maintenance</h1>

    <div class="sec-tabs">
      <button :class="{ active: activeTab === 'firewall' }" @click="onTabClick('firewall')">Pare-feu</button>
      <button :class="{ active: activeTab === 'malware' }" @click="onTabClick('malware')">Scan malware</button>
      <button :class="{ active: activeTab === 'snapshots' }" @click="onTabClick('snapshots')">Snapshots</button>
    </div>

    <section v-if="activeTab === 'firewall'" class="sec-panel">
      <div v-if="firewallError" class="sec-error">{{ firewallError }}</div>
      <template v-else-if="firewall">
        <div class="sec-status" :class="firewall.active ? 'sec-active' : 'sec-inactive'">
          UFW {{ firewall.active ? "actif" : "inactif" }}
        </div>
        <div v-for="(r, i) in firewall.rules" :key="i" class="sec-row">{{ r }}</div>
      </template>
    </section>

    <section v-else-if="activeTab === 'malware'" class="sec-panel">
      <div class="sec-form-row">
        <input v-model="scanDir" class="sec-input" placeholder="Dossier à scanner..." />
        <button :disabled="scanning" @click="runScan">{{ scanning ? "Scan en cours..." : "Scanner" }}</button>
      </div>
      <div v-if="scanError" class="sec-error">{{ scanError }}</div>
      <div v-else-if="scanDone && findings.length === 0" class="sec-empty">Aucune menace détectée.</div>
      <div v-for="f in findings" :key="f.path" class="sec-row sec-finding">
        <span>{{ f.path }}</span>
        <span>{{ f.signature }}</span>
      </div>
    </section>

    <section v-else-if="activeTab === 'snapshots'" class="sec-panel">
      <div v-if="snapshotsError" class="sec-error">{{ snapshotsError }}</div>
      <div v-for="s in snapshots" :key="s.id" class="sec-row">
        <span>#{{ s.id }}</span>
        <span>{{ s.date }}</span>
      </div>
    </section>
  </div>
</template>

<style scoped>
.sec-page { padding: 24px; color: var(--nx-text-primary); }
.sec-tabs { display: flex; gap: 8px; margin: 16px 0; }
.sec-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.sec-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.sec-error { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.sec-status { padding: 10px 14px; border-radius: 8px; margin-bottom: 10px; font-weight: 600; }
.sec-active { background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); }
.sec-inactive { background: color-mix(in srgb, var(--nx-accent-warning) 15%, transparent); border: 1px solid var(--nx-accent-warning); }
.sec-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-border); }
.sec-finding { color: var(--nx-accent-danger); }
.sec-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.sec-input { flex: 1; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.sec-empty { color: var(--nx-text-secondary); margin-top: 10px; }
</style>
```

- [ ] **Step 2: Type-check**

Run: `npx vue-tsc --noEmit`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/pages/SecurityPage.vue
git commit -m "feat: SecurityPage.vue with firewall, malware scan, snapshots tabs"
```

---

## Task 7: Wire `SecurityPage` into `App.vue` navigation, final verification

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Add the page to the nav**

Read the current `src/App.vue` (8 pages: `dashboard`/`hardware`/`drivers`/`logs`/`theme-editor`/`packages`/`disks`/`network`). Add `"security"` to `PageId`, import `SecurityPage`, add to the `pages` map, add a 9th nav button ("Sécurité") matching the established pattern exactly.

- [ ] **Step 2: Run the full test suite**

Run: `npm run test` — expect 25 passed (unchanged).
Run: `cd src-tauri && cargo test` — expect 88 passed, 1 ignored.
Run: `npx vue-tsc --noEmit` — expect clean.

- [ ] **Step 3: Manual GUI verification in WSL2**

Same established technique (boot `npm run tauri dev`, confirm real window/surface via `/proc/<pid>/fd`, check dev log for errors, kill processes). On this WSL2 host, `ufw`/`clamscan`/`timeshift` are almost certainly all absent — use the established scratch-test workaround to prove `get_firewall_status()`, `scan_for_malware()`, and `list_snapshots()` all fail gracefully (return `Err`, don't panic) when their binaries are missing, since none of them auto-run on mount (firewall status does run in `onMounted`, the other two are user-triggered) — specifically verify `get_firewall_status()`'s `Err` path is exercised and doesn't crash the app. Revert the scratch test before committing.

- [ ] **Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: wire SecurityPage into app navigation"
```
