# NiTruX Phase 5 Part 2 — Security/Maintenance Writes (privileged, pkexec) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add three privileged, curated maintenance/security operations to NiTruX — a "troubleshoot" button running a fixed allowlist of safe one-click fixes, snapshot *creation* (Phase 5 Part 1 only ever listed snapshots), and quarantining a malware finding instead of leaving the user to delete it by hand — via `pkexec`, following the exact architecture established in Phase 2 Part 2 and Phase 4 Part 2.

**Architecture:** Extend the same `nitrux-pkexec-helper` dispatcher script with 3 new subcommands (`troubleshoot`, `create-snapshot`, `quarantine-file`), each backed by its **own dedicated exec path** from the start — `/usr/bin/nitrux-pkexec-troubleshoot`, `/usr/bin/nitrux-pkexec-create-snapshot`, `/usr/bin/nitrux-pkexec-quarantine-file` — one polkit action each. (Phase 2/4 Part 2 originally shared one exec path across all their actions and had to be fixed after a live test showed `pkexec` cannot disambiguate them; this plan applies that lesson from the start instead of repeating the mistake — see the project memory note on this if you need the full story.) The `troubleshoot` subcommand itself takes a single curated action id (`clean-cache`, `fix-broken`, `restart-network`, `vacuum-logs`) validated against a fixed allowlist inside the script — this is one polkit action with one consent message ("NiTruX veut exécuter une action de dépannage système"), not four separate actions, matching the existing `firewall-rule <add|remove>` pattern from Phase 4 Part 2.

**Tech Stack:** Same as every prior phase — Tauri v2, Rust, Vue 3 + TypeScript + Pinia, Vitest, `cargo test`, `pkexec`/polkit, POSIX shell.

**Testing environment:** Unlike Phase 2/4 Part 2, a disposable Debian 13 test VM is now available and its live-verification workflow is proven (SSH as `dev`, `pkttyagent --process $$ &` with `set +m` to avoid SIGTTIN, no GUI session required). This plan still follows the same discipline as before: implement and unit-test everything without ever running a privileged command in WSL2 (the real dev environment) during the Task-by-task implementation pass; the live VM pass happens as an explicit final step performed by the coordinator directly (not delegated to a subagent), exactly like Phase 2/4 Part 2's Task 5b.

**Scope note — what's in, what's deliberately excluded:**
- **In scope — troubleshoot allowlist (4 curated actions, each already a well-understood, reversible, standard Linux maintenance operation):**
  - `clean-cache`: clears the package manager's local download cache (`apt-get clean` / `dnf clean all` / `pacman -Scc --noconfirm` / `zypper clean --all`) — frees disk space, packages simply re-download next time they're needed.
  - `fix-broken`: repairs a broken/interrupted package install (`apt-get install -f -y` / `dpkg --configure -a` chained for apt; `dnf` and `zypper` don't need an equivalent repair step in the same way — see Task 1 for the exact per-manager mapping; `pacman` has no direct equivalent, script no-ops with a clear message for pacman).
  - `restart-network`: `systemctl restart NetworkManager` — a standard "turn it off and on again" fix for a stuck network stack, briefly interrupts connectivity, self-heals.
  - `vacuum-logs`: `journalctl --vacuum-time=7d` — deletes systemd journal entries older than 7 days, standard disk-space maintenance, no different in spirit from log rotation.
- **In scope — snapshot creation:** `timeshift --create --comments "NiTruX manual snapshot"` (or equivalent rsync-mode invocation if Btrfs mode isn't configured — Timeshift picks its own mode based on prior `timeshift --list`/config, this plan does not attempt to configure Timeshift itself, only trigger a creation with whatever mode is already configured on the host).
- **In scope — malware quarantine:** move a single previously-scanned file (path supplied by the caller, which must have come from a real `scan_for_malware` finding — see Task 2's validation) into a root-owned, `chmod 000` quarantine directory (`/var/lib/nitrux/quarantine/`), rather than deleting it outright. Quarantining is the reversible choice — an admin can still recover the file if the scan was a false positive; deletion is not.
- **Explicitly excluded, deferred to a future plan requiring its own dedicated review:** configuring Timeshift itself (schedule, retention, backend selection), snapshot *restore* (far higher blast radius than create), any other troubleshoot action beyond the 4 listed above (no arbitrary shell exec, ever), and un-quarantining/restoring a quarantined file (deferred — for now quarantine is one-way through this UI; recovering a false positive is a manual admin task outside NiTruX until a future plan adds it deliberately).

---

## File Structure

```
src-tauri/
├── packaging/
│   ├── org.heiphaistos.nitrux.security.policy   # NEW polkit policy XML (3 actions, 3 exec paths)
│   └── nitrux-pkexec-helper                      # MODIFIED: add troubleshoot/create-snapshot/quarantine-file
├── src/
│   └── security_write.rs        # run_troubleshoot_action/create_snapshot/quarantine_file commands
└── tauri.conf.json               # modified: bundle the new .policy + 3 new exec-path copies
src/
└── pages/
    └── SecurityPage.vue           # modified: troubleshoot button, "create snapshot" button, quarantine action per finding
```

---

## Task 1: Third polkit policy file + wrapper script subcommands (write only, no live execution)

**Files:**
- Create: `src-tauri/packaging/org.heiphaistos.nitrux.security.policy`
- Modify: `src-tauri/packaging/nitrux-pkexec-helper`

Read the CURRENT `src-tauri/packaging/nitrux-pkexec-helper` in full first — you are adding 3 new `case` branches to the existing dispatcher (which already has `install-package`, `upgrade-all`, `write-hosts`, `set-dns`, `firewall-rule`). Do not alter the existing branches. **Do NOT execute this script with real privileges anywhere, including WSL2.**

- [ ] **Step 1: Write the third polkit policy file**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <vendor>NiTruX</vendor>
  <vendor_url>https://github.com/Heiphaistos/NiTruX</vendor_url>

  <action id="org.heiphaistos.nitrux.troubleshoot">
    <description>Exécuter une action de dépannage système</description>
    <message>NiTruX veut exécuter une action de dépannage système</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-troubleshoot</annotate>
  </action>

  <action id="org.heiphaistos.nitrux.create-snapshot">
    <description>Créer un instantané système</description>
    <message>NiTruX veut créer un instantané système</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-create-snapshot</annotate>
  </action>

  <action id="org.heiphaistos.nitrux.quarantine-file">
    <description>Mettre un fichier en quarantaine</description>
    <message>NiTruX veut mettre un fichier détecté en quarantaine</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-quarantine-file</annotate>
  </action>
</policyconfig>
```

No comment before the `<?xml?>` declaration (invalid per XML 1.0 — bit a prior task in this project). Verify well-formedness via Python `xml.dom.minidom.parse` afterward.

- [ ] **Step 2: Add validation helpers and the 3 new subcommand branches**

Add near the top of the script, alongside the existing `validate_*` helpers:

```sh
validate_troubleshoot_action() {
  case "$1" in
    clean-cache|fix-broken|restart-network|vacuum-logs) : ;;
    *) die "unknown troubleshoot action: $1" ;;
  esac
}

# Quarantine only ever accepts an ABSOLUTE path with no ".." traversal
# component and no shell metacharacters -- this runs as root and moves
# whatever it's given, so the validation here is the only thing standing
# between a caller and moving an arbitrary root-owned file.
validate_quarantine_path() {
  case "$1" in
    '') die "empty quarantine path" ;;
    /*) : ;;
    *) die "quarantine path must be absolute: $1" ;;
  esac
  case "$1" in
    *..*) die "quarantine path must not contain '..': $1" ;;
    *'*'*|*'?'*|*'$'*|*'`'*|*';'*|*'|'*|*'&'*|*'<'*|*'>'*|*'\n'*)
      die "quarantine path contains disallowed characters: $1" ;;
  esac
}
```

Add these branches to the outer `case "$cmd" in`, after the existing `firewall-rule)` branch and before the final `*)` fallback:

```sh
  troubleshoot)
    action="${2:-}"
    validate_troubleshoot_action "$action"
    case "$action" in
      clean-cache)
        if command -v apt-get >/dev/null 2>&1; then apt-get clean; fi
        if command -v dnf >/dev/null 2>&1; then dnf clean all; fi
        if command -v pacman >/dev/null 2>&1; then pacman -Scc --noconfirm; fi
        if command -v zypper >/dev/null 2>&1; then zypper clean --all; fi
        ;;
      fix-broken)
        if command -v apt-get >/dev/null 2>&1; then
          dpkg --configure -a
          apt-get install -f -y
        elif command -v zypper >/dev/null 2>&1; then
          zypper verify -y
        else
          echo "no repair step defined for the detected package manager on this system" >&2
        fi
        ;;
      restart-network)
        exec systemctl restart NetworkManager
        ;;
      vacuum-logs)
        exec journalctl --vacuum-time=7d
        ;;
    esac
    ;;
  create-snapshot)
    exec timeshift --create --comments "NiTruX manual snapshot"
    ;;
  quarantine-file)
    path="${2:-}"
    validate_quarantine_path "$path"
    if [ ! -e "$path" ]; then
      die "quarantine target does not exist: $path"
    fi
    mkdir -p /var/lib/nitrux/quarantine
    chmod 700 /var/lib/nitrux/quarantine
    dest="/var/lib/nitrux/quarantine/$(date +%s)-$(basename "$path")"
    mv "$path" "$dest"
    chmod 000 "$dest"
    echo "quarantined: $dest"
    ;;
```

Note `clean-cache` and `fix-broken` deliberately do NOT use `exec` (they may need to run more than one command per branch, e.g. `dpkg --configure -a` then `apt-get install -f -y`, or multiple package managers' cache-clean commands in sequence) — the script's normal fall-through-to-end-of-script-then-exit-0 behavior handles this correctly since `set -eu` is in effect and each individual command must succeed. `restart-network`, `vacuum-logs`, and `create-snapshot` are single commands and use `exec` like every other single-command branch in this script, for consistency.

Update the usage comment at the top of the script to add the 3 new subcommands, following the existing format.

- [ ] **Step 3: Rigorous static verification (NO live execution)**

Trace through by hand and verify empirically where a prior task's method applies:
1. `validate_troubleshoot_action` with an unlisted action (e.g. `"rm-rf-root"`) → `case` falls to `*) die`. This is a plain enumerated allowlist, not a pattern match, so there's no anchoring subtlety to re-prove here (unlike `validate_port_proto`'s glob pattern in Phase 4 Part 2) — a `case` with literal alternatives requires an exact match already.
2. `validate_quarantine_path` with `/home/dev/../../etc/shadow` → contains `..` → `die`. With `../etc/passwd` (relative, not absolute) → fails the `/*` requirement first → `die`. With `/tmp/evil;rm -rf /` → matches the metacharacter-rejection pattern → `die`. Prove at least the `..`-rejection and the metacharacter-rejection empirically via a disposable scratch `case` test in WSL2 (same method as Phase 4 Part 2 Task 1's `validate_port_proto` check), since `case` glob patterns have subtle matching semantics worth re-confirming rather than assuming.
3. `quarantine-file` with a path to a file that does not exist → the `[ ! -e "$path" ]` check fires before any `mv` is attempted → `die`, nothing moved.
4. Confirm `chmod 000` really does leave the quarantined file unreadable/unexecutable by everyone including its own owner (root can still access files it owns despite `000` on Linux due to DAC being bypassed for the file *owner* only via capabilities — actually verify this claim rather than asserting it: research or reason carefully about whether root retains access to a `chmod 000` file it owns, and adjust the design/comment if the assumption in this task's rationale turns out to be wrong. This affects whether "quarantine" is actually inert or whether root processes could still read/execute it.)

- [ ] **Step 4: Commit**

```bash
git add src-tauri/packaging/org.heiphaistos.nitrux.security.policy src-tauri/packaging/nitrux-pkexec-helper
git commit -m "feat: polkit policy and pkexec helper subcommands for security/maintenance writes"
```

---

## Task 2: Rust commands (`run_troubleshoot_action`, `create_snapshot`, `quarantine_file`)

**Files:**
- Create: `src-tauri/src/security_write.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests first (TDD)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_known_troubleshoot_actions() {
        assert!(validate_troubleshoot_action("clean-cache").is_ok());
        assert!(validate_troubleshoot_action("fix-broken").is_ok());
        assert!(validate_troubleshoot_action("restart-network").is_ok());
        assert!(validate_troubleshoot_action("vacuum-logs").is_ok());
    }

    #[test]
    fn rejects_unknown_troubleshoot_action() {
        assert!(validate_troubleshoot_action("rm-rf-root").is_err());
        assert!(validate_troubleshoot_action("").is_err());
        assert!(validate_troubleshoot_action("clean-cache; rm -rf /").is_err());
    }

    #[test]
    fn accepts_well_formed_absolute_quarantine_path() {
        assert!(validate_quarantine_path("/home/dev/eicar.txt").is_ok());
        assert!(validate_quarantine_path("/tmp/suspicious.bin").is_ok());
    }

    #[test]
    fn rejects_relative_or_empty_quarantine_path() {
        assert!(validate_quarantine_path("").is_err());
        assert!(validate_quarantine_path("relative/path.txt").is_err());
        assert!(validate_quarantine_path("../etc/passwd").is_err());
    }

    #[test]
    fn rejects_quarantine_path_with_traversal_or_metacharacters() {
        assert!(validate_quarantine_path("/home/dev/../../etc/shadow").is_err());
        assert!(validate_quarantine_path("/tmp/evil;rm -rf /").is_err());
        assert!(validate_quarantine_path("/tmp/$(whoami)").is_err());
        assert!(validate_quarantine_path("/tmp/`whoami`").is_err());
    }
}
```

Run: `cd src-tauri && cargo test security_write:: 2>&1 | tail -40` — expect FAIL (module doesn't exist).

- [ ] **Step 2: Implement `security_write.rs`**

```rust
use crate::subprocess;
use std::time::Duration;

const PKEXEC_TROUBLESHOOT: &str = "/usr/bin/nitrux-pkexec-troubleshoot";
const PKEXEC_CREATE_SNAPSHOT: &str = "/usr/bin/nitrux-pkexec-create-snapshot";
const PKEXEC_QUARANTINE_FILE: &str = "/usr/bin/nitrux-pkexec-quarantine-file";

pub fn validate_troubleshoot_action(action: &str) -> Result<(), String> {
    match action {
        "clean-cache" | "fix-broken" | "restart-network" | "vacuum-logs" => Ok(()),
        other => Err(format!("action de dépannage inconnue : {other}")),
    }
}

/// Mirrors the helper script's own `validate_quarantine_path`, checked
/// here too so obviously-malformed input gets a clear error from the app
/// itself rather than an opaque failure from `pkexec`/the helper.
pub fn validate_quarantine_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("chemin de quarantaine vide".to_string());
    }
    if !path.starts_with('/') {
        return Err(format!("le chemin doit être absolu : {path}"));
    }
    if path.contains("..") {
        return Err(format!("le chemin ne doit pas contenir '..' : {path}"));
    }
    const DISALLOWED: [char; 10] = ['*', '?', '$', '`', ';', '|', '&', '<', '>', '\n'];
    if path.chars().any(|c| DISALLOWED.contains(&c)) {
        return Err(format!("le chemin contient des caractères non autorisés : {path}"));
    }
    Ok(())
}

#[tauri::command]
pub fn run_troubleshoot_action(action: String) -> Result<String, String> {
    validate_troubleshoot_action(&action)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_TROUBLESHOOT, "troubleshoot", &action],
        Duration::from_secs(120),
    )
}

#[tauri::command]
pub fn create_snapshot() -> Result<String, String> {
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_CREATE_SNAPSHOT, "create-snapshot"],
        Duration::from_secs(600),
    )
}

#[tauri::command]
pub fn quarantine_file(path: String) -> Result<String, String> {
    validate_quarantine_path(&path)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_QUARANTINE_FILE, "quarantine-file", &path],
        Duration::from_secs(30),
    )
}
```

The 600s timeout on `create_snapshot` is deliberately generous — a real Timeshift snapshot (especially rsync-mode, not Btrfs) can take several minutes depending on how much data is being backed up, similar rationale to `upgrade_all_packages`'s 1800s timeout in Phase 2 Part 2.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cd src-tauri && cargo test security_write:: 2>&1 | tail -40` — expect PASS (7 tests).

- [ ] **Step 4: Register the commands**

Modify `src-tauri/src/lib.rs` — add `mod security_write;` and all 3 commands (`security_write::run_troubleshoot_action`, `security_write::create_snapshot`, `security_write::quarantine_file`) to `generate_handler!`.

- [ ] **Step 5: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20` — expect 108 (current baseline) + 7 = 115 passed, 1 ignored, 0 failed. `cargo build 2>&1 | grep -i warning` → empty.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/security_write.rs src-tauri/src/lib.rs
git commit -m "feat: run_troubleshoot_action, create_snapshot, quarantine_file Tauri commands"
```

---

## Task 3: Bundle the security policy + 3 new exec paths into .deb/.rpm

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add the 4 new entries to both `deb.files` and `rpm.files`**

Read the CURRENT `tauri.conf.json` first (has 2 existing `.policy` files and 5 existing exec-path entries from Phase 2/4 Part 2). ADD:

```json
"/usr/share/polkit-1/actions/org.heiphaistos.nitrux.security.policy": "packaging/org.heiphaistos.nitrux.security.policy",
"/usr/bin/nitrux-pkexec-troubleshoot": "packaging/nitrux-pkexec-helper",
"/usr/bin/nitrux-pkexec-create-snapshot": "packaging/nitrux-pkexec-helper",
"/usr/bin/nitrux-pkexec-quarantine-file": "packaging/nitrux-pkexec-helper"
```

to both the `deb.files` and `rpm.files` maps (in addition to, not replacing, the existing 7 entries in each — the final map should have 3 `.policy` files + 8 exec-path copies = 11 entries per target).

- [ ] **Step 2: Build both bundles and verify**

Run: `npm run tauri build -- --bundles deb,rpm`

Then: `dpkg-deb -c src-tauri/target/release/bundle/deb/NiTruX_*.deb | grep -E "polkit|pkexec"` — expect 3 `.policy` files + 8 `nitrux-pkexec-*` binaries, all with `-rwxr-xr-x` permissions.

Re-verify the shebang byte-check on the UPDATED helper script (same method as every prior phase — extract via `dpkg-deb -x`, `xxd`/`od` the first bytes, confirm `23 21 2f 62 69 6e 2f 73 68 0a` with zero CR bytes anywhere in the file).

Parse all 3 `.policy` files with `xml.dom.minidom` from inside the extracted `.deb` and confirm well-formed.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: bundle security/maintenance polkit policy into .deb/.rpm packages"
```

---

## Task 4: `SecurityPage.vue` — troubleshoot button, snapshot creation, quarantine action

**Files:**
- Modify: `src/pages/SecurityPage.vue`

- [ ] **Step 1: Read the CURRENT `src/pages/SecurityPage.vue` in full before editing**

It currently has (from Phase 5 Part 1, read-only): a firewall status display (`get_firewall_status`), a malware scan trigger + findings list (`scan_for_malware`), and a snapshots list (`list_snapshots`). Add to it, do not replace it:

1. A "Dépannage" section with 4 buttons (one per troubleshoot action: "Vider le cache des paquets", "Réparer les paquets cassés", "Redémarrer le réseau", "Purger les anciens journaux"), each calling `invoke("run_troubleshoot_action", { action: "<id>" })`, with per-button busy/error/success state (follow the exact ref-naming and error-handling pattern established in `NetworkPage.vue`'s `saveHosts`/`saveDns`/`addFirewallRule` functions from Phase 4 Part 2 — read that file for the pattern if unsure).
2. A "Créer un instantané" button next to the existing snapshots list, calling `invoke("create_snapshot")`, and refreshing the snapshots list (re-`invoke("list_snapshots")`) on success.
3. A "Mettre en quarantaine" button next to each malware finding in the existing findings list, calling `invoke("quarantine_file", { path: finding.path })`, removing that finding from the displayed list on success (client-side only — the backend doesn't need to know the UI removed it, it already moved the real file).

- [ ] **Step 2: Type-check**

Run: `npx vue-tsc --noEmit` — expect clean.

- [ ] **Step 3: Run the frontend test suite**

Run: `npm run test -- --run` — expect unchanged count (this task adds no new spec files, matching Phase 4 Part 2's Task 4 precedent).

- [ ] **Step 4: Commit**

```bash
git add src/pages/SecurityPage.vue
git commit -m "feat: troubleshoot/snapshot-create/quarantine UI on SecurityPage"
```

---

## Task 5: Full verification pass + honest gap documentation

**Files:** None (verification-only), plus a documentation append to this plan file.

- [ ] **Step 1: Run the full test suite** (`npm run test`, `cargo test`, `vue-tsc --noEmit`), record exact counts.

- [ ] **Step 2: Do NOT attempt live verification.** That is an explicit follow-up step performed by the coordinator directly against the VM, not by a subagent in this task.

- [ ] **Step 3: Append an honest verification-state summary to this plan file** — same shape as Phase 2/4 Part 2's final sections: what was proven for real (unit tests, real `.deb`/`.rpm` build + inspection, shebang byte-check, the `chmod 000`-vs-root-access research finding from Task 1 Step 3), what remains unproven (the live `pkexec` chain for all 3 new actions), and explicit confirmation this branch stays unmerged/unreleased pending live verification.

- [ ] **Step 4: Commit.**

```bash
git add docs/superpowers/plans/2026-07-31-nitrux-phase5-part2.md
git commit -m "docs: record Phase 5 Part 2 verification coverage and known polkit-GUI test gap"
```

---

## Verification State (as of completion)

**Proven for real (with what evidence):**

- The unit tests for `validate_troubleshoot_action`/`validate_quarantine_path` in `src-tauri/src/security_write.rs` pass: `cargo test security_write::` → `test result: ok. 5 passed; 0 failed; 0 ignored`. Note: this plan's Task 2 text originally said "expect PASS (7 tests)" — the actual test module (`accepts_known_troubleshoot_actions`, `rejects_unknown_troubleshoot_action`, `accepts_well_formed_absolute_quarantine_path`, `rejects_relative_or_empty_quarantine_path`, `rejects_quarantine_path_with_traversal_or_metacharacters`) has 5 tests, not 7. The implementer caught and flagged this discrepancy at the time rather than padding the suite to match a miscounted draft figure — 5 is the correct, honest count, confirmed again here by re-running the suite.
- The full Rust suite passes with no regressions: `cargo test` (from `src-tauri/`) → `test result: ok. 113 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out`. The ignored test is the pre-existing timing-sensitive `system::tests::repeated_refresh_on_shared_system_computes_nonzero_cpu_delta`, unrelated to this plan, same as every prior phase. `src/main.rs` and doc-tests both report 0 tests, as expected.
- The full frontend test suite passes with no regressions: `npm run test -- --run` → 6 test files, 25 tests, all passed, 0 failed. Unchanged count from before this plan — Task 4 added no new spec files, matching the Phase 4 Part 2 precedent.
- `npx vue-tsc --noEmit` produced no output — clean, no type errors.
- A real `.deb` and `.rpm` were built and inspected during Task 3 (per commit `58c8214`'s message and the plan's Task 3 Step 2 checks): the new `org.heiphaistos.nitrux.security.policy` and the updated `nitrux-pkexec-helper` — now installed under 3 additional dedicated exec-path names (`nitrux-pkexec-troubleshoot`, `nitrux-pkexec-create-snapshot`, `nitrux-pkexec-quarantine-file`) alongside the 5 exec paths from the two prior plans — were confirmed present at their correct bundled paths with executable permissions, for a total of 3 `.policy` files + 8 exec-path binaries per package. Unlike Phase 2/4 Part 2, this plan's 3 new polkit actions never shared an exec path in the first place — each got its own dedicated path from Task 1 Step 1 onward, applying the lesson those two prior plans only learned after a live bug.
- The `validate_quarantine_path` rejection logic (`..` traversal, shell metacharacters, non-absolute paths) was empirically proven via disposable scratch `case` tests in WSL2 during Task 1 Step 3, not just reasoned about — mirroring the method used for `validate_port_proto` in Phase 4 Part 2. This is independently reinforced by the Rust-side tests above, which assert the same rejections at the `security_write.rs` validation layer.
- **The CRLF shebang corruption bug:** found during Task 3's build verification (commit `58c8214`). Root cause: this Windows dev machine has `core.autocrlf=true`, which silently converts a committed clean-LF file's shebang (`#!/bin/sh\n`) to CRLF (`#!/bin/sh\r\n`) on working-tree checkout — the committed git blob was always clean LF (confirmed via `git show HEAD:src-tauri/packaging/nitrux-pkexec-helper`), but the *working tree* copy that Tauri's bundler actually reads from at build time could silently drift, which would have broken the script on real Linux systems with `bad interpreter: No such file or directory`. Fixed by adding `.gitattributes` (`src-tauri/packaging/* text eol=lf`, forcing LF regardless of `core.autocrlf`) to this worktree's branch (part of commit `58c8214`) **and** to `master` directly (commit `f2ca2e6`, "fix: pin src-tauri/packaging/* to LF line endings"), since the already-merged/released Phase 2 Part 2 and Phase 4 Part 2 packaging files carried the exact same latent risk on any future checkout or fresh clone, just not yet triggered. Re-verified during this Task 5 pass: `od -An -tx1 -N10` on `src-tauri/packaging/nitrux-pkexec-helper` in the current working tree shows `23 21 2f 62 69 6e 2f 73 68 0a` (`#!/bin/sh\n`, zero CR bytes), and a repo-wide check found 0 CR bytes in both the working-tree file and `git show HEAD:...` for it. Two files (`src-tauri/Cargo.toml`, `src-tauri/packaging/nitrux-pkexec-helper`) show as `modified` in `git status` in this worktree, but `git diff` on both is empty and both are confirmed 0-CR — this is a harmless index/line-ending-attribute bookkeeping artifact left over from before `.gitattributes` was added, not a real content change, and was left untouched rather than force-committed as noise.
- **A known limitation, not a bug:** `chmod 000` on a quarantined file (`nitrux-pkexec-helper`'s `quarantine-file` branch, `src-tauri/packaging/nitrux-pkexec-helper`) does NOT prevent root — or any other process invoked via `pkexec`/`sudo` — from still reading, writing, or executing that file. Linux DAC permission bits (including `000`) are enforced via the kernel's discretionary access control checks, which are bypassed for processes holding `CAP_DAC_OVERRIDE` — a capability every root process has by default. So quarantine as currently implemented only blocks *non-root* actors (a normal user account, a non-privileged process that might otherwise re-execute or re-read the finding) from touching the file; it provides no containment against a compromised or malicious root-level process on the same machine. This was identified during Task 1 Step 3's static review (the task's own text flagged this exact claim as one to verify rather than assume) and is an inherent property of the chosen containment mechanism (permission bits alone), not a defect introduced by this plan's implementation. It is documented here as a known, accepted limitation rather than fixed in this pass — a future plan could strengthen containment (e.g., relocating quarantined files outside any servable/executable path regardless of permissions, changing ownership to a dedicated unprivileged-but-isolated account, or applying AppArmor/SELinux mandatory access control) if that additional complexity is judged worth it later. For NiTruX's actual threat model (protecting an ordinary user from re-triggering a malware finding by accident), permission-bit quarantine is likely sufficient — the gap only matters against an attacker who has already obtained root, at which point quarantine is not the relevant control anyway.

**NOT yet proven (explicit gap):**

- The live `pkexec` → polkit `auth_admin` dialog → `nitrux-pkexec-helper` → actual `apt-get clean`/`dpkg --configure -a`/`systemctl restart NetworkManager`/`journalctl --vacuum-time=7d`/`timeshift --create`/real file quarantine-move execution chain has never been run for real, anywhere. No command in this branch's history has touched a real system's package cache, network service, journal, Timeshift snapshot store, or filesystem via these new commands.
- The built `.deb`/`.rpm` have not been installed on a real system for this plan's 3 new actions specifically. Nothing has confirmed `pkaction --verbose` shows `org.heiphaistos.nitrux.troubleshoot`/`.create-snapshot`/`.quarantine-file` registering correctly with their own distinct exec paths (as opposed to the shared-path ambiguity bug found and fixed live in the two prior plans, `5e10549`/`a0cc352`).
- Correspondingly, `run_troubleshoot_action`, `create_snapshot`, and `quarantine_file` in `src-tauri/src/security_write.rs` have only been exercised through their pure validation logic (`validate_troubleshoot_action`/`validate_quarantine_path`) — the code path that actually spawns `pkexec` against a real `nitrux-pkexec-troubleshoot`/`nitrux-pkexec-create-snapshot`/`nitrux-pkexec-quarantine-file` binary has never executed against a real system.

**Status: this branch (`phase-5-part2`) stays UNMERGED and UNRELEASED pending live verification**, exactly the pattern established by Phase 2 Part 2 and Phase 4 Part 2 — both of which were themselves left unmerged pending live verification at their own Task 5 checkpoints, and were only later live-verified (a disposable Debian 13 VM, SSH + `pkttyagent` for text-mode polkit auth, no GUI session required) and merged/released together as v0.6.0 (commits `0565eba`, `fce3f50`, `62518a4`) after that verification passed. That same live-VM pass is what surfaced the shared-exec-path bug (`5e10549`/`a0cc352`) this plan's Task 1 deliberately designed around from the start.

Live verification of this plan's 3 new actions is the explicit next step: installing the rebuilt `.deb` on the disposable Debian VM (already proven functional tonight for the two prior plans), confirming `pkaction --verbose` registers `org.heiphaistos.nitrux.troubleshoot`/`.create-snapshot`/`.quarantine-file` under their own distinct exec paths, and driving `run_troubleshoot_action` (each of the 4 curated actions), `create_snapshot`, and `quarantine_file` against the real filesystem/package cache/journal/Timeshift store, then restoring the VM to a clean state afterward. This must be performed by the coordinator directly against the VM, not delegated to an autonomous subagent, since it requires actual VM credentials and genuinely mutates a real (if disposable) system's package cache, network service, logs, snapshot store, and filesystem.
