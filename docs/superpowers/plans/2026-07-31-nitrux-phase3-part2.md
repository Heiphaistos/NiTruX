# NiTruX Phase 3 Part 2 — Disk/Partition Writes (privileged, pkexec) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add the last pillar's privileged, write-capable operations to NiTruX — formatting a partition, extending a partition into trailing free space, and cloning a whole disk to an image file — via `pkexec`, following the exact architecture established in Phase 2/4/5 Part 2 (dedicated exec path per polkit action from the start).

**Architecture:** Extend the same `nitrux-pkexec-helper` dispatcher script with 3 new subcommands (`format-partition`, `extend-partition`, `clone-disk`), each backed by its own dedicated exec path (`/usr/bin/nitrux-pkexec-format-partition`, `/usr/bin/nitrux-pkexec-extend-partition`, `/usr/bin/nitrux-pkexec-clone-disk`), one polkit action each — no shared paths, per the hard-won lesson from Phase 2/4 Part 2's live-tested bug (documented in project memory and referenced in every plan since).

**Tech Stack:** Same as every prior phase — Tauri v2, Rust, Vue 3 + TypeScript + Pinia, Vitest, `cargo test`, `pkexec`/polkit, POSIX shell.

**Testing environment:** Same disposable Debian 13 VM used and proven for all three prior write-capable plans tonight. **This plan is explicitly authorized by the user to be live-tested for real, including actually formatting a partition** ("fait tout dans la vm"). Because the VM's only disk (`/dev/sda`, 127G) hosts its actual boot/root/swap partitions with no spare disk or free space, live format/extend/clone testing targets a **disk image file attached as a loop device** on the VM — this exercises the identical `mkfs`/`parted`/`dd` code paths as a real physical disk with zero risk to the VM's own filesystem, and is standard practice for testing this class of code. The VM remains disposable regardless, but there is no reason to gamble with its boot disk when a loop device proves the same code paths just as validly.

**Scope note — what's in, what's deliberately excluded (this is the highest-blast-radius pillar in the project — irreversible data loss is the central risk, so scope stays conservative):**
- **In scope — `format-partition`:** run `mkfs.<fstype>` against a caller-supplied partition device path, for `fstype` in `{ext4, btrfs, xfs, vfat}` (the 4 most common Linux-relevant filesystems; NTFS formatting is excluded — `mkfs.ntfs` via `ntfs-3g` is less universally available and NTFS is rarely the right choice for a Linux-native partition anyway). Multiple layers of validation before anything destructive runs (detailed in Task 1/2):
  1. The device path must exist and must look like a **partition**, not a whole disk (e.g. `/dev/sda1`, `/dev/nvme0n1p1` — rejects `/dev/sda`, `/dev/nvme0n1`). Formatting a whole disk device directly is almost never what a user means to do and destroys the partition table too, not just one filesystem.
  2. The device must **not currently be mounted anywhere** (checked via `findmnt`/`/proc/mounts` at execution time, inside the privileged script, not just trusted from the caller).
  3. The device must **not be (or be part of) the currently-booted root device** (`findmnt -n -o SOURCE /`), checked independently of the mount check above as defense in depth — even a hypothetical unmounted-but-still-root-adjacent edge case is refused.
  4. `fstype` validated against the fixed allowlist above.
- **In scope — `extend-partition`:** grow a partition to fill immediately-following free space on the same disk (the safe, common "I added space, use it" case), then resize the filesystem to match. Growing a partition and its filesystem is safe in the sense that it does not touch or reduce existing data — the failure modes are "the tool refuses" or "nothing happens," not data loss, provided the resize step only ever grows (never shrinks). v1 supports **ext4 only** for the filesystem-resize step (`resize2fs`) — the partition-table resize itself (`parted resizepart`) is filesystem-agnostic, but this plan only wires up the matching fs-resize for ext4 since that's what the VM and most Linux installs actually use; growing a partition without knowing how to grow its filesystem to match would leave unusable trailing space, so unsupported filesystem types are refused with a clear message rather than silently leaving the partition table and filesystem out of sync.
- **In scope — `clone-disk`:** `dd`-based whole-disk-to-image-file clone (`dd if=<source-disk> of=<dest-image-file> bs=4M status=progress conv=fsync`). Non-destructive to the source by construction — this is the lowest-risk of the three operations and existed mainly to complete the "backup/clone" scope named back when Phase 3 was first planned.
- **Explicitly excluded, deferred indefinitely or to a future plan requiring its own dedicated review:**
  - **Shrinking a partition or filesystem** — the single highest-risk disk operation there is (get the order or size wrong and you silently truncate live data); genuinely out of scope for this plan and likely for NiTruX v1 entirely.
  - **Moving a partition's start offset** — same class of risk as shrink, excluded for the same reason.
  - **Deleting a partition or repartitioning a disk from scratch** — destroys the partition table; not attempted here.
  - **Restoring a disk image back onto a real disk** (the inverse of `clone-disk`) — writing an image file back onto a device is exactly as destructive as `format-partition` but with a much larger blast radius (whole disk, not one partition) and no built-in "this is clearly a partition, not a disk" safety rail; deferred.
  - **NTFS formatting**, for the reason given above.

---

## File Structure

```
src-tauri/
├── packaging/
│   ├── org.heiphaistos.nitrux.disks.policy   # NEW polkit policy XML (3 actions, 3 exec paths)
│   └── nitrux-pkexec-helper                   # MODIFIED: add format-partition/extend-partition/clone-disk
├── src/
│   └── disk_write.rs        # format_partition/extend_partition/clone_disk commands
└── tauri.conf.json           # modified: bundle the new .policy + 3 new exec-path copies
src/
└── pages/
    └── DisksPage.vue          # modified: format button (with typed confirmation), extend button, clone button
```

---

## Task 1: Fourth polkit policy file + wrapper script subcommands (write only, no live execution)

**Files:**
- Create: `src-tauri/packaging/org.heiphaistos.nitrux.disks.policy`
- Modify: `src-tauri/packaging/nitrux-pkexec-helper`

Read the CURRENT `nitrux-pkexec-helper` in full first — you are adding 3 new `case` branches to the existing dispatcher (which already has 8 subcommands across packages/network/security). Do not alter existing branches. **Do NOT execute this script with real privileges anywhere, including WSL2** — this task is static-only; live testing of these specific, high-risk operations happens later in Task 5, performed by the coordinator directly against a disk image file (never a real device), not by a subagent, and not in this task.

- [ ] **Step 1: Write the fourth polkit policy file**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <vendor>NiTruX</vendor>
  <vendor_url>https://github.com/Heiphaistos/NiTruX</vendor_url>

  <action id="org.heiphaistos.nitrux.format-partition">
    <description>Formater une partition</description>
    <message>NiTruX veut formater une partition — cette action efface toutes les données de la partition</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-format-partition</annotate>
  </action>

  <action id="org.heiphaistos.nitrux.extend-partition">
    <description>Étendre une partition</description>
    <message>NiTruX veut étendre une partition dans l'espace libre disponible</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-extend-partition</annotate>
  </action>

  <action id="org.heiphaistos.nitrux.clone-disk">
    <description>Cloner un disque vers un fichier image</description>
    <message>NiTruX veut cloner un disque vers un fichier image</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-clone-disk</annotate>
  </action>
</policyconfig>
```

Note the `format-partition` action's `<message>` explicitly warns about data loss in the polkit consent dialog itself — the only one of the project's 11 actions (by the end of this plan) that carries an explicit destructive-action warning in its polkit message text, because it's the only one where the underlying operation is irreversible by design. No XML comment before `<?xml?>`. Verify well-formed via `xml.dom.minidom.parse`.

- [ ] **Step 2: Add validation helpers**

```sh
# A partition device path must look like "<disk><number>" or an NVMe
# "<nvme-disk>p<number>" -- this is what distinguishes a PARTITION from a
# WHOLE DISK. Formatting a whole disk (e.g. /dev/sda) destroys the
# partition table, not just one filesystem's data -- almost never what is
# intended, and far more destructive than formatting one partition.
validate_partition_device() {
  case "$1" in
    /dev/sd[a-z][0-9]*|/dev/vd[a-z][0-9]*|/dev/nvme[0-9]*n[0-9]*p[0-9]*|/dev/mmcblk[0-9]*p[0-9]*) : ;;
    *) die "not a recognized partition device path (expected e.g. /dev/sda1, /dev/nvme0n1p1): $1" ;;
  esac
  if [ ! -b "$1" ]; then
    die "not a block device: $1"
  fi
}

# Refuses to touch a device that is currently mounted, OR that is (or
# backs) the currently-booted root filesystem -- checked independently of
# the mount check as defense in depth, not merely because "checking twice
# is nice": a device could in principle stop appearing in the live mount
# table while still being relied upon by the running system in a way a
# single check might miss, and this operation is irreversible, so the
# extra check costs nothing and only helps.
validate_partition_not_in_use() {
  if findmnt -n -S "$1" >/dev/null 2>&1; then
    die "device is currently mounted, refusing: $1"
  fi
  root_device=$(findmnt -n -o SOURCE / 2>/dev/null || echo "")
  if [ -n "$root_device" ] && [ "$1" = "$root_device" ]; then
    die "refusing to touch the currently-booted root device: $1"
  fi
}

validate_disk_device() {
  case "$1" in
    /dev/sd[a-z]|/dev/vd[a-z]|/dev/nvme[0-9]*n[0-9]*|/dev/mmcblk[0-9]*|/dev/loop[0-9]*) : ;;
    *) die "not a recognized whole-disk device path: $1" ;;
  esac
  if [ ! -b "$1" ]; then
    die "not a block device: $1"
  fi
}

validate_fstype() {
  case "$1" in
    ext4|btrfs|xfs|vfat) : ;;
    *) die "unsupported filesystem type (expected ext4, btrfs, xfs, or vfat): $1" ;;
  esac
}

# Destination image path for clone-disk: absolute, no traversal, no shell
# metacharacters -- same style as validate_quarantine_path in the security
# actions, since this also runs as root and writes wherever it's told.
validate_image_dest_path() {
  case "$1" in
    '') die "empty destination path" ;;
    /*) : ;;
    *) die "destination path must be absolute: $1" ;;
  esac
  case "$1" in
    *..*) die "destination path must not contain '..': $1" ;;
    *'*'*|*'?'*|*'$'*|*'`'*|*';'*|*'|'*|*'&'*|*'<'*|*'>'*|*'\n'*)
      die "destination path contains disallowed characters: $1" ;;
  esac
}
```

- [ ] **Step 3: Add the 3 new subcommand branches**

```sh
  format-partition)
    device="${2:-}"
    fstype="${3:-}"
    validate_partition_device "$device"
    validate_partition_not_in_use "$device"
    validate_fstype "$fstype"
    case "$fstype" in
      ext4)  exec mkfs.ext4 -F "$device" ;;
      btrfs) exec mkfs.btrfs -f "$device" ;;
      xfs)   exec mkfs.xfs -f "$device" ;;
      vfat)  exec mkfs.vfat -F 32 "$device" ;;
    esac
    ;;
  extend-partition)
    device="${2:-}"
    disk="${3:-}"
    partnum="${4:-}"
    validate_partition_device "$device"
    validate_partition_not_in_use "$device"
    validate_disk_device "$disk"
    case "$partnum" in
      ''|*[!0-9]*) die "partition number must be numeric: $partnum" ;;
    esac
    parted --script "$disk" resizepart "$partnum" 100%
    exec resize2fs "$device"
    ;;
  clone-disk)
    source_disk="${2:-}"
    dest_path="${3:-}"
    validate_disk_device "$source_disk"
    validate_image_dest_path "$dest_path"
    exec dd if="$source_disk" of="$dest_path" bs=4M status=progress conv=fsync
    ;;
```

`extend-partition` takes 3 arguments (the partition device to check/resize the filesystem of, the whole-disk device `parted` needs, and the partition number within that disk) because `parted resizepart` operates in terms of "disk + partition number," not a partition device path directly — the caller (Rust side, Task 2) is responsible for deriving all 3 correctly from a single partition selection.

Note `extend-partition` does NOT use `exec` for the `parted` call (only for the final `resize2fs`) because both steps must run in sequence and `set -eu` already ensures the script aborts if `parted` fails before ever reaching `resize2fs`.

Update the header usage comment to add these 3 new lines, matching the existing format.

- [ ] **Step 4: Rigorous static verification (NO live execution)**

Trace through by hand, and empirically prove the security-critical patterns via disposable WSL2 scratch tests (same method as every prior phase's Task 1):
1. `validate_partition_device` against `/dev/sda` (whole disk, no trailing digit) → must NOT match any of the 4 alternatives (each requires a trailing digit) → falls to `*) die`. Empirically prove this with a scratch `case` test — this is the single most important check in this entire plan, since accepting a whole-disk device here would let `format-partition` wipe an entire disk's partition table instead of one filesystem.
2. `validate_partition_device` against `/dev/sda1` → matches `/dev/sd[a-z][0-9]*` → passes.
3. `validate_partition_device` against a path with injection attempt, e.g. `/dev/sda1;rm -rf /` → prove empirically it's rejected (same `case`-pattern full-string-anchoring property proven in Phase 4 Part 2's Task 1, reconfirm it holds here too since the pattern shape is different).
4. `validate_image_dest_path` — same checks as Phase 5 Part 2's `validate_quarantine_path` (`..`, metacharacters, non-absolute) — prove empirically.
5. `validate_partition_not_in_use` — trace by hand (cannot be proven empirically without a real device/mount, that's fine, static trace is sufficient here): confirm `findmnt -n -S "$1"` is the correct invocation to check whether a specific device is mounted anywhere (verify this against real `findmnt` documentation/behavior — don't assume the flag is right, confirm it), and confirm the root-device comparison happens as a second, independent check.
6. Confirm `set -eu` ensures `parted --script "$disk" resizepart "$partnum" 100%` failing (e.g. no free space to extend into) aborts the script before `resize2fs` ever runs, leaving the filesystem untouched if the partition-table resize didn't happen.

If any of these traces reveal a real gap, do not silently patch around it without flagging it clearly — this is the highest-risk script in the project, get it right or say clearly why you couldn't be sure.

- [ ] **Step 5: Commit**

```bash
git add src-tauri/packaging/org.heiphaistos.nitrux.disks.policy src-tauri/packaging/nitrux-pkexec-helper
git commit -m "feat: polkit policy and pkexec helper subcommands for disk/partition writes"
```

---

## Task 2: Rust commands (`format_partition`, `extend_partition`, `clone_disk`)

**Files:**
- Create: `src-tauri/src/disk_write.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests first (TDD)**

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_well_formed_partition_devices() {
        assert!(validate_partition_device("/dev/sda1").is_ok());
        assert!(validate_partition_device("/dev/nvme0n1p1").is_ok());
        assert!(validate_partition_device("/dev/vdb2").is_ok());
    }

    #[test]
    fn rejects_whole_disk_devices() {
        assert!(validate_partition_device("/dev/sda").is_err());
        assert!(validate_partition_device("/dev/nvme0n1").is_err());
    }

    #[test]
    fn rejects_malformed_or_malicious_partition_device() {
        assert!(validate_partition_device("").is_err());
        assert!(validate_partition_device("/dev/sda1; rm -rf /").is_err());
        assert!(validate_partition_device("sda1").is_err());
        assert!(validate_partition_device("/etc/passwd").is_err());
    }

    #[test]
    fn accepts_supported_fstypes() {
        assert!(validate_fstype("ext4").is_ok());
        assert!(validate_fstype("btrfs").is_ok());
        assert!(validate_fstype("xfs").is_ok());
        assert!(validate_fstype("vfat").is_ok());
    }

    #[test]
    fn rejects_unsupported_fstype() {
        assert!(validate_fstype("ntfs").is_err());
        assert!(validate_fstype("").is_err());
        assert!(validate_fstype("ext4; rm -rf /").is_err());
    }

    #[test]
    fn accepts_well_formed_disk_devices() {
        assert!(validate_disk_device("/dev/sda").is_ok());
        assert!(validate_disk_device("/dev/nvme0n1").is_ok());
    }

    #[test]
    fn rejects_partition_device_as_disk_device() {
        assert!(validate_disk_device("/dev/sda1").is_err());
    }

    #[test]
    fn accepts_well_formed_image_dest_path() {
        assert!(validate_image_dest_path("/home/dev/backup.img").is_ok());
        assert!(validate_image_dest_path("/mnt/backups/disk.img").is_ok());
    }

    #[test]
    fn rejects_malformed_image_dest_path() {
        assert!(validate_image_dest_path("").is_err());
        assert!(validate_image_dest_path("relative.img").is_err());
        assert!(validate_image_dest_path("/tmp/../etc/shadow").is_err());
        assert!(validate_image_dest_path("/tmp/evil;rm -rf /").is_err());
    }

    #[test]
    fn accepts_well_formed_partition_number() {
        assert!(validate_partition_number("1").is_ok());
        assert!(validate_partition_number("12").is_ok());
    }

    #[test]
    fn rejects_malformed_partition_number() {
        assert!(validate_partition_number("").is_err());
        assert!(validate_partition_number("0x1").is_err());
        assert!(validate_partition_number("1; rm -rf /").is_err());
    }
}
```

Run: `cd src-tauri && cargo test disk_write:: 2>&1 | tail -50` — expect FAIL (module doesn't exist).

- [ ] **Step 2: Implement `disk_write.rs`**

```rust
use crate::subprocess;
use std::time::Duration;

const PKEXEC_FORMAT_PARTITION: &str = "/usr/bin/nitrux-pkexec-format-partition";
const PKEXEC_EXTEND_PARTITION: &str = "/usr/bin/nitrux-pkexec-extend-partition";
const PKEXEC_CLONE_DISK: &str = "/usr/bin/nitrux-pkexec-clone-disk";

/// Mirrors the helper script's own device-shape check. A partition device
/// must end in a number distinguishing it from its parent whole-disk
/// device -- accepting a whole-disk path here would let format_partition
/// wipe an entire partition table instead of one filesystem.
pub fn validate_partition_device(device: &str) -> Result<(), String> {
    let looks_like_partition = (device.starts_with("/dev/sd") || device.starts_with("/dev/vd"))
        && device.len() > 8
        && device.chars().last().is_some_and(|c| c.is_ascii_digit());
    let looks_like_nvme_partition = device.starts_with("/dev/nvme")
        && device.contains('n')
        && device.contains('p')
        && device.chars().last().is_some_and(|c| c.is_ascii_digit());
    let looks_like_mmc_partition = device.starts_with("/dev/mmcblk")
        && device.contains('p')
        && device.chars().last().is_some_and(|c| c.is_ascii_digit());
    if !(looks_like_partition || looks_like_nvme_partition || looks_like_mmc_partition) {
        return Err(format!(
            "chemin de partition invalide (attendu par ex. /dev/sda1, /dev/nvme0n1p1) : {device}"
        ));
    }
    if !device.chars().all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '-') {
        return Err(format!("le chemin contient des caractères non autorisés : {device}"));
    }
    Ok(())
}

pub fn validate_disk_device(device: &str) -> Result<(), String> {
    let is_known_whole_disk_shape = (device.starts_with("/dev/sd") || device.starts_with("/dev/vd"))
        && device.len() == 8
        && device.chars().last().is_some_and(|c| c.is_ascii_alphabetic());
    let is_nvme_whole_disk = device.starts_with("/dev/nvme")
        && device.contains('n')
        && !device.contains('p');
    if !(is_known_whole_disk_shape || is_nvme_whole_disk) {
        return Err(format!("chemin de disque invalide (attendu par ex. /dev/sda, /dev/nvme0n1) : {device}"));
    }
    Ok(())
}

pub fn validate_fstype(fstype: &str) -> Result<(), String> {
    match fstype {
        "ext4" | "btrfs" | "xfs" | "vfat" => Ok(()),
        other => Err(format!("système de fichiers non pris en charge : {other}")),
    }
}

pub fn validate_partition_number(n: &str) -> Result<(), String> {
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("numéro de partition invalide : {n}"));
    }
    Ok(())
}

/// Mirrors the helper script's own `validate_image_dest_path`.
pub fn validate_image_dest_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("chemin de destination vide".to_string());
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

/// Formats `device` with `fstype`. IRREVERSIBLE — destroys all data on the
/// target partition. Both arguments are validated here AND independently
/// re-validated inside the privileged helper script, which additionally
/// refuses a mounted or root-adjacent device at execution time (a check
/// that cannot be meaningfully done from this unprivileged process, since
/// mount state can change between validation and execution).
#[tauri::command]
pub fn format_partition(device: String, fstype: String) -> Result<String, String> {
    validate_partition_device(&device)?;
    validate_fstype(&fstype)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_FORMAT_PARTITION, "format-partition", &device, &fstype],
        Duration::from_secs(300),
    )
}

/// Grows `device` (a partition) to fill trailing free space on `disk` at
/// partition number `part_number`, then grows its ext4 filesystem to
/// match. Growing-only: never shrinks, never touches existing data.
#[tauri::command]
pub fn extend_partition(device: String, disk: String, part_number: String) -> Result<String, String> {
    validate_partition_device(&device)?;
    validate_disk_device(&disk)?;
    validate_partition_number(&part_number)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_EXTEND_PARTITION, "extend-partition", &device, &disk, &part_number],
        Duration::from_secs(300),
    )
}

/// Clones `source_disk` to the image file at `dest_path`. Non-destructive
/// to the source; a large, slow, real disk-read operation, hence the very
/// generous timeout (a full disk clone can take a long time depending on
/// disk size — this mirrors the reasoning behind upgrade_all_packages's
/// and create_snapshot's generous timeouts elsewhere in this codebase).
#[tauri::command]
pub fn clone_disk(source_disk: String, dest_path: String) -> Result<String, String> {
    validate_disk_device(&source_disk)?;
    validate_image_dest_path(&dest_path)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_CLONE_DISK, "clone-disk", &source_disk, &dest_path],
        Duration::from_secs(3600),
    )
}
```

Append the test module from Step 1 to the bottom of this file.

- [ ] **Step 3: Run tests to verify they pass**

Run: `cd src-tauri && cargo test disk_write:: 2>&1 | tail -50` — expect PASS (10 tests).

- [ ] **Step 4: Register the commands**

Modify `src-tauri/src/lib.rs` — add `mod disk_write;` (alphabetically) and all 3 commands to `generate_handler!`.

- [ ] **Step 5: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20` — expect 113 (current baseline) + 10 = 123 passed, 1 ignored, 0 failed. `cargo build 2>&1 | grep -i warning` → empty.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/disk_write.rs src-tauri/src/lib.rs
git commit -m "feat: format_partition, extend_partition, clone_disk Tauri commands"
```

---

## Task 3: Bundle the disks policy into .deb/.rpm

**Files:**
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Add the 4 new entries to both `deb.files` and `rpm.files`**

```json
"/usr/share/polkit-1/actions/org.heiphaistos.nitrux.disks.policy": "packaging/org.heiphaistos.nitrux.disks.policy",
"/usr/bin/nitrux-pkexec-format-partition": "packaging/nitrux-pkexec-helper",
"/usr/bin/nitrux-pkexec-extend-partition": "packaging/nitrux-pkexec-helper",
"/usr/bin/nitrux-pkexec-clone-disk": "packaging/nitrux-pkexec-helper"
```

Final map: 4 `.policy` files + 11 exec-path copies = 15 entries per target.

- [ ] **Step 2: Build and verify**

`npm run tauri build -- --bundles deb,rpm`, then `dpkg-deb -c ... | grep -E "polkit|pkexec"` — expect 15 lines, all `-rwxr-xr-x`.

**Re-verify the shebang byte-check** on the updated helper script (extract, `od`/`xxd`, confirm `23 21 2f 62 69 6e 2f 73 68 0a`, zero CR bytes — `.gitattributes` from Phase 5 Part 2 should prevent the CRLF corruption bug from recurring, but verify it actually held, don't just assume the fix generalizes).

Parse all 4 `.policy` files with `xml.dom.minidom` and confirm well-formed.

- [ ] **Step 3: Commit**

```bash
git add src-tauri/tauri.conf.json
git commit -m "feat: bundle disk/partition polkit policy into .deb/.rpm packages"
```

---

## Task 4: `DisksPage.vue` — format/extend/clone UI with an explicit typed-confirmation guard on format

**Files:**
- Modify: `src/pages/DisksPage.vue`

- [ ] **Step 1: Read the CURRENT `src/pages/DisksPage.vue` in full before editing** (Phase 3 Part 1 read-only content — disk/partition list, usage bars). Add to it, do not replace it.

- [ ] **Step 2: Add UI for the 3 new operations**

For `format_partition`: because this is the one genuinely irreversible operation in the entire project, require the user to **type the exact partition device path** (e.g. type `/dev/sda1` to confirm formatting `/dev/sda1`) into a confirmation input before the "Formater" button becomes enabled — a plain "are you sure?" click-through is not enough friction for a data-destroying action; requiring the user to type the specific device name forces them to consciously re-read what they're about to destroy, and makes it much harder to misclick the wrong partition in a list. Implement as a local ref compared against the selected partition's device path; disable the format button until they match exactly.

For `extend_partition` and `clone_disk` (both non-destructive to existing data), a normal button + busy/error/success state following the established pattern from every prior write-capable page (`NetworkPage.vue`, `SecurityPage.vue`) is sufficient — no typed-confirmation friction needed since neither operation can destroy existing data if it works correctly, and if either fails, nothing is lost (extend fails closed per Task 1's `set -eu` ordering; clone simply doesn't produce a usable image file).

- [ ] **Step 3: Type-check + test**

`npx vue-tsc --noEmit` clean; `npm run test -- --run` unchanged count (no new spec files this task, matching precedent).

- [ ] **Step 4: Commit**

```bash
git add src/pages/DisksPage.vue
git commit -m "feat: format/extend/clone UI on DisksPage with typed confirmation for format"
```

---

## Task 5: Full verification pass + live VM verification (this plan's live pass is authorized to actually format/extend/clone against a disk image file, per explicit user instruction)

**Files:** None (verification-only), plus a documentation append to this plan file. This task is performed by the coordinator directly, not delegated to a subagent, for the live-VM portion — the static/unit-test portion may be delegated as in every prior plan.

- [ ] **Step 1: Run the full test suite** (`npm run test`, `cargo test`, `vue-tsc --noEmit`), record exact counts.

- [ ] **Step 2: Live VM verification against a disk image file (not a real device)**

On the VM: create a disk image file (e.g. `dd if=/dev/zero of=/home/dev/nitrux-test-disk.img bs=1M count=512` for a 512MB throwaway image, or `fallocate` if available), attach it as a loop device (`losetup -fP /home/dev/nitrux-test-disk.img` or similar, giving a `/dev/loopN` device), partition it with a small partition table (e.g. `parted` to create one partition spanning most of the image, leaving some trailing free space to exercise `extend-partition` meaningfully), then:
- Install the rebuilt `.deb`, confirm `pkaction --verbose` shows all 4 new actions with correct distinct exec paths.
- Run `format_partition` for real against the loop device's partition (e.g. `/dev/loop0p1`) with `ext4`, confirm via `blkid`/`file -s` that the partition really has an ext4 filesystem afterward.
- Run `extend_partition` for real (this requires the test partition to have been created with trailing free space specifically so there's something to extend into) and confirm via `parted print`/`resize2fs -P` that the partition and filesystem both grew.
- Run `clone_disk` for real against the loop device, confirm the resulting image file exists, has the expected size, and (spot-check) contains a valid partition table (`fdisk -l <image>` or similar).
- Test the rejection paths for real: attempt `format_partition` against a real mounted device (e.g. the VM's actual `/dev/sda2` root) and confirm it's rejected by the helper script itself before any `mkfs` runs; attempt `format_partition` against the whole loop device (not a partition of it) and confirm the device-shape check rejects it.
- Detach the loop device (`losetup -d`) and delete the throwaway image file when done, leaving the VM clean.

- [ ] **Step 3: Append an honest verification-state summary to this plan file** covering both the static/unit-test proof and the live-VM proof (this plan, unusually among the 4 write-capable plans tonight, gets its live verification folded into the same session rather than deferred — record that explicitly).

- [ ] **Step 4: Commit.**

```bash
git add docs/superpowers/plans/2026-07-31-nitrux-phase3-part2.md
git commit -m "docs: record Phase 3 Part 2 verification coverage including live loop-device format/extend/clone test"
```
