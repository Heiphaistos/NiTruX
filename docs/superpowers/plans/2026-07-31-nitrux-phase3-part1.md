# NiTruX Phase 3 Part 1 — Disk Analysis Tools (read-only) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the "Disques & stockage" pillar's read-only analysis tools: disk/partition listing, disk usage visualizer, duplicate file finder, large file finder, hash checker, and SMART health query — exposed via a new `DisksPage.vue`.

**Architecture:** One Rust module per tool (`disks.rs`, `duplicates.rs`, `largefiles.rs`, `hashcheck.rs`, `smart.rs`), each following the established `subprocess::run_with_timeout` + `Result<T, String>` convention where they shell out, or pure-Rust filesystem walking (via `std::fs`) where a subprocess isn't needed. One aggregating frontend page with tabs/sections per tool, matching `ThemeEditorPage.vue`'s tabbed pattern.

**Tech Stack:** Same as Phase 1/2 — Tauri v2, Rust, Vue 3 + TypeScript + Pinia, Vitest, `cargo test`.

**Scope note — READ-ONLY by design, intentional, same reasoning as Phase 2 Part 1:** Partition table modification (resize/format/delete), disk cloning, and backup-to-disk are explicitly OUT of scope for this plan. Those are destructive/high-blast-radius operations (a bug in partition-table code can destroy all data on a disk) requiring human-reviewed design before any autonomous implementation. This plan covers only analysis and read-only queries — real, valuable progress toward the pillar, with write-capable operations deferred to a "Phase 3 Part 2" plan requiring explicit review.

---

## File Structure

```
src-tauri/src/
├── disks.rs        # lsblk-based disk/partition listing + df-based usage summary
├── duplicates.rs    # hash-based duplicate file finder within a given directory
├── largefiles.rs     # files above a size threshold within a given directory
├── hashcheck.rs        # compute file hash (sha256/md5/sha1), optional comparison
└── smart.rs              # smartctl -a health query per detected disk (may require root; see Task 5 notes)
src/
└── pages/
    └── DisksPage.vue     # tabbed UI: Disques | Doublons | Gros fichiers | Vérif. hash | SMART
```

---

## Task 1: Disk/partition listing (`lsblk`) + usage summary (`df`)

**Files:**
- Create: `src-tauri/src/disks.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/disks.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lsblk_json_into_disks() {
        let json = r#"{
            "blockdevices": [
                {"name":"sda","size":"512G","type":"disk","mountpoint":null,
                 "children":[{"name":"sda1","size":"511G","type":"part","mountpoint":"/"}]}
            ]
        }"#;
        let disks = parse_lsblk_json(json).expect("should parse");
        assert_eq!(disks.len(), 1);
        assert_eq!(disks[0].name, "sda");
        assert_eq!(disks[0].size, "512G");
        assert_eq!(disks[0].partitions.len(), 1);
        assert_eq!(disks[0].partitions[0].name, "sda1");
        assert_eq!(disks[0].partitions[0].mountpoint.as_deref(), Some("/"));
    }

    #[test]
    fn handles_disk_with_no_partitions() {
        let json = r#"{"blockdevices":[{"name":"sdb","size":"1T","type":"disk","mountpoint":null}]}"#;
        let disks = parse_lsblk_json(json).expect("should parse");
        assert_eq!(disks.len(), 1);
        assert!(disks[0].partitions.is_empty());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_lsblk_json("not json").is_none());
    }

    #[test]
    fn parses_df_line_into_usage_entry() {
        let line = "/dev/sda1  536870912  214748364  322122548  40%  /";
        let entry = parse_df_line(line).expect("should parse");
        assert_eq!(entry.mountpoint, "/");
        assert_eq!(entry.used_percent, 40);
        assert_eq!(entry.total_bytes, 536870912 * 1024);
        assert_eq!(entry.used_bytes, 214748364 * 1024);
    }

    #[test]
    fn skips_df_header_line() {
        assert!(parse_df_line("Filesystem  1K-blocks  Used  Available  Use%  Mounted on").is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test disks:: 2>&1 | tail -40`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `disks.rs`**

```rust
// src-tauri/src/disks.rs
use crate::subprocess;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct Partition {
    pub name: String,
    pub size: String,
    pub mountpoint: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Disk {
    pub name: String,
    pub size: String,
    pub partitions: Vec<Partition>,
}

#[derive(Serialize, Clone)]
pub struct UsageEntry {
    pub mountpoint: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: u8,
}

#[derive(Deserialize)]
struct RawBlockDevice {
    name: String,
    size: String,
    #[serde(rename = "type")]
    device_type: String,
    mountpoint: Option<String>,
    #[serde(default)]
    children: Vec<RawBlockDevice>,
}

#[derive(Deserialize)]
struct RawLsblkOutput {
    blockdevices: Vec<RawBlockDevice>,
}

pub fn parse_lsblk_json(json: &str) -> Option<Vec<Disk>> {
    let raw: RawLsblkOutput = serde_json::from_str(json).ok()?;
    Some(
        raw.blockdevices
            .into_iter()
            .filter(|d| d.device_type == "disk")
            .map(|d| Disk {
                name: d.name,
                size: d.size,
                partitions: d
                    .children
                    .into_iter()
                    .map(|c| Partition {
                        name: c.name,
                        size: c.size,
                        mountpoint: c.mountpoint,
                    })
                    .collect(),
            })
            .collect(),
    )
}

/// Parses one line of `df --output=source,size,used,avail,pcent,target` output
/// (POSIX `-P` block-size defaults to 1K blocks), e.g.:
/// "/dev/sda1  536870912  214748364  322122548  40%  /"
pub fn parse_df_line(line: &str) -> Option<UsageEntry> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() != 6 || fields[1].parse::<u64>().is_err() {
        return None;
    }
    let total_kb: u64 = fields[1].parse().ok()?;
    let used_kb: u64 = fields[2].parse().ok()?;
    let used_percent: u8 = fields[4].trim_end_matches('%').parse().ok()?;
    Some(UsageEntry {
        mountpoint: fields[5].to_string(),
        total_bytes: total_kb * 1024,
        used_bytes: used_kb * 1024,
        used_percent,
    })
}

#[tauri::command]
pub fn list_disks() -> Result<Vec<Disk>, String> {
    let output = subprocess::run_with_timeout("lsblk", &["-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT"], Duration::from_secs(10))?;
    parse_lsblk_json(&output).ok_or_else(|| "impossible de lire la sortie de lsblk".to_string())
}

#[tauri::command]
pub fn list_disk_usage() -> Result<Vec<UsageEntry>, String> {
    let output = subprocess::run_with_timeout("df", &["-k", "-P"], Duration::from_secs(10))?;
    Ok(output.lines().filter_map(parse_df_line).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lsblk_json_into_disks() {
        let json = r#"{
            "blockdevices": [
                {"name":"sda","size":"512G","type":"disk","mountpoint":null,
                 "children":[{"name":"sda1","size":"511G","type":"part","mountpoint":"/"}]}
            ]
        }"#;
        let disks = parse_lsblk_json(json).expect("should parse");
        assert_eq!(disks.len(), 1);
        assert_eq!(disks[0].name, "sda");
        assert_eq!(disks[0].size, "512G");
        assert_eq!(disks[0].partitions.len(), 1);
        assert_eq!(disks[0].partitions[0].name, "sda1");
        assert_eq!(disks[0].partitions[0].mountpoint.as_deref(), Some("/"));
    }

    #[test]
    fn handles_disk_with_no_partitions() {
        let json = r#"{"blockdevices":[{"name":"sdb","size":"1T","type":"disk","mountpoint":null}]}"#;
        let disks = parse_lsblk_json(json).expect("should parse");
        assert_eq!(disks.len(), 1);
        assert!(disks[0].partitions.is_empty());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_lsblk_json("not json").is_none());
    }

    #[test]
    fn parses_df_line_into_usage_entry() {
        let line = "/dev/sda1  536870912  214748364  322122548  40%  /";
        let entry = parse_df_line(line).expect("should parse");
        assert_eq!(entry.mountpoint, "/");
        assert_eq!(entry.used_percent, 40);
        assert_eq!(entry.total_bytes, 536870912 * 1024);
        assert_eq!(entry.used_bytes, 214748364 * 1024);
    }

    #[test]
    fn skips_df_header_line() {
        assert!(parse_df_line("Filesystem  1K-blocks  Used  Available  Use%  Mounted on").is_none());
    }
}
```

Note: `RawBlockDevice.size`/`Disk.size`/`Partition.size` stay as the raw human-readable string lsblk emits (e.g. `"512G"`) rather than being parsed to bytes — lsblk's default `SIZE` column is already human-formatted and re-parsing it (`"512G"` → bytes → re-format) would be lossy round-tripping for no benefit; `df`'s `UsageEntry`, by contrast, DOES use raw byte counts since `-k` gives exact machine-parseable values and the frontend needs real numbers for a progress-bar style display.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test disks:: 2>&1 | tail -40`
Expected: PASS (5 tests)

- [ ] **Step 5: Register the commands**

Modify `src-tauri/src/lib.rs` — add `mod disks;` and both `disks::list_disks`, `disks::list_disk_usage` to `generate_handler!`, alongside the existing `drivers`, `hardware`, `logs`, `packages`, `sensors`, `subprocess`, `system` modules and their commands. Merge additively — do not remove anything.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 43 (pre-existing) + 5 (this task) = 48 passed, 1 ignored.

Run: `cargo build 2>&1 | tail -10`
Expected: 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/disks.rs src-tauri/src/lib.rs
git commit -m "feat: disk/partition listing (lsblk) and usage summary (df)"
```

---

## Task 2: Duplicate file finder

**Files:**
- Create: `src-tauri/src/duplicates.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/duplicates.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn groups_files_with_identical_content_by_hash() {
        let dir = std::env::temp_dir().join(format!("nitrux-dup-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        let c = dir.join("c.txt");
        std::fs::File::create(&a).unwrap().write_all(b"same content").unwrap();
        std::fs::File::create(&b).unwrap().write_all(b"same content").unwrap();
        std::fs::File::create(&c).unwrap().write_all(b"different content").unwrap();

        let groups = find_duplicates(&dir).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(groups.len(), 1, "expected exactly one duplicate group");
        assert_eq!(groups[0].paths.len(), 2);
    }

    #[test]
    fn returns_no_groups_when_all_files_are_unique() {
        let dir = std::env::temp_dir().join(format!("nitrux-dup-unique-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(dir.join("x.txt")).unwrap().write_all(b"x").unwrap();
        std::fs::File::create(dir.join("y.txt")).unwrap().write_all(b"y").unwrap();

        let groups = find_duplicates(&dir).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert!(groups.is_empty());
    }

    #[test]
    fn errors_on_nonexistent_directory() {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/xyz");
        assert!(find_duplicates(bogus).is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test duplicates:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `duplicates.rs`**

```rust
// src-tauri/src/duplicates.rs
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct DuplicateGroup {
    pub hash: String,
    pub paths: Vec<String>,
    pub size_bytes: u64,
}

fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Scans `dir` recursively (not following symlinks, to avoid cycles) and
/// groups regular files with identical SHA-256 content hashes. Directories
/// that can't be read (permission denied, etc.) are silently skipped rather
/// than aborting the whole scan.
pub fn find_duplicates(dir: &Path) -> Result<Vec<DuplicateGroup>, String> {
    if !dir.is_dir() {
        return Err(format!("{} n'est pas un dossier accessible", dir.display()));
    }

    let mut by_hash: HashMap<String, (Vec<String>, u64)> = HashMap::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                if let Ok(hash) = hash_file(&path) {
                    let entry = by_hash.entry(hash).or_insert_with(|| (Vec::new(), size));
                    entry.0.push(path.to_string_lossy().into_owned());
                }
            }
        }
    }

    Ok(by_hash
        .into_iter()
        .filter(|(_, (paths, _))| paths.len() > 1)
        .map(|(hash, (paths, size_bytes))| DuplicateGroup { hash, paths, size_bytes })
        .collect())
}

#[tauri::command]
pub fn find_duplicate_files(directory: String) -> Result<Vec<DuplicateGroup>, String> {
    find_duplicates(std::path::Path::new(&directory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn groups_files_with_identical_content_by_hash() {
        let dir = std::env::temp_dir().join(format!("nitrux-dup-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        let c = dir.join("c.txt");
        std::fs::File::create(&a).unwrap().write_all(b"same content").unwrap();
        std::fs::File::create(&b).unwrap().write_all(b"same content").unwrap();
        std::fs::File::create(&c).unwrap().write_all(b"different content").unwrap();

        let groups = find_duplicates(&dir).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(groups.len(), 1, "expected exactly one duplicate group");
        assert_eq!(groups[0].paths.len(), 2);
    }

    #[test]
    fn returns_no_groups_when_all_files_are_unique() {
        let dir = std::env::temp_dir().join(format!("nitrux-dup-unique-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(dir.join("x.txt")).unwrap().write_all(b"x").unwrap();
        std::fs::File::create(dir.join("y.txt")).unwrap().write_all(b"y").unwrap();

        let groups = find_duplicates(&dir).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert!(groups.is_empty());
    }

    #[test]
    fn errors_on_nonexistent_directory() {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/xyz");
        assert!(find_duplicates(bogus).is_err());
    }
}
```

This task introduces a new dependency, `sha2` (for `Sha256`), since the project has no hashing crate yet.

- [ ] **Step 3.5: Add the `sha2` dependency**

Modify `src-tauri/Cargo.toml` — add under `[dependencies]`:

```toml
sha2 = "0.10"
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test duplicates:: 2>&1 | tail -30`
Expected: PASS (3 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod duplicates;` and `duplicates::find_duplicate_files` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 48 + 3 = 51 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/duplicates.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: SHA-256-based duplicate file finder"
```

---

## Task 3: Large file finder

**Files:**
- Create: `src-tauri/src/largefiles.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/largefiles.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn finds_files_above_the_size_threshold() {
        let dir = std::env::temp_dir().join(format!("nitrux-large-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let small = dir.join("small.txt");
        let big = dir.join("big.txt");
        std::fs::File::create(&small).unwrap().write_all(&vec![0u8; 10]).unwrap();
        std::fs::File::create(&big).unwrap().write_all(&vec![0u8; 1000]).unwrap();

        let results = find_large_files(&dir, 100).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("big.txt"));
        assert_eq!(results[0].size_bytes, 1000);
    }

    #[test]
    fn returns_results_sorted_largest_first() {
        let dir = std::env::temp_dir().join(format!("nitrux-large-sort-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(dir.join("a.bin")).unwrap().write_all(&vec![0u8; 500]).unwrap();
        std::fs::File::create(dir.join("b.bin")).unwrap().write_all(&vec![0u8; 2000]).unwrap();

        let results = find_large_files(&dir, 100).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(results.len(), 2);
        assert!(results[0].size_bytes > results[1].size_bytes);
    }

    #[test]
    fn errors_on_nonexistent_directory() {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/xyz");
        assert!(find_large_files(bogus, 0).is_err());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test largefiles:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `largefiles.rs`**

```rust
// src-tauri/src/largefiles.rs
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct LargeFile {
    pub path: String,
    pub size_bytes: u64,
}

/// Scans `dir` recursively (not following symlinks) for files at or above
/// `min_size_bytes`, sorted largest-first. Unreadable subdirectories are
/// silently skipped, matching `duplicates::find_duplicates`'s behavior.
pub fn find_large_files(dir: &Path, min_size_bytes: u64) -> Result<Vec<LargeFile>, String> {
    if !dir.is_dir() {
        return Err(format!("{} n'est pas un dossier accessible", dir.display()));
    }

    let mut results = Vec::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.len() >= min_size_bytes {
                        results.push(LargeFile {
                            path: path.to_string_lossy().into_owned(),
                            size_bytes: metadata.len(),
                        });
                    }
                }
            }
        }
    }

    results.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    Ok(results)
}

#[tauri::command]
pub fn find_large_files_cmd(directory: String, min_size_bytes: u64) -> Result<Vec<LargeFile>, String> {
    find_large_files(std::path::Path::new(&directory), min_size_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn finds_files_above_the_size_threshold() {
        let dir = std::env::temp_dir().join(format!("nitrux-large-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let small = dir.join("small.txt");
        let big = dir.join("big.txt");
        std::fs::File::create(&small).unwrap().write_all(&vec![0u8; 10]).unwrap();
        std::fs::File::create(&big).unwrap().write_all(&vec![0u8; 1000]).unwrap();

        let results = find_large_files(&dir, 100).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("big.txt"));
        assert_eq!(results[0].size_bytes, 1000);
    }

    #[test]
    fn returns_results_sorted_largest_first() {
        let dir = std::env::temp_dir().join(format!("nitrux-large-sort-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(dir.join("a.bin")).unwrap().write_all(&vec![0u8; 500]).unwrap();
        std::fs::File::create(dir.join("b.bin")).unwrap().write_all(&vec![0u8; 2000]).unwrap();

        let results = find_large_files(&dir, 100).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(results.len(), 2);
        assert!(results[0].size_bytes > results[1].size_bytes);
    }

    #[test]
    fn errors_on_nonexistent_directory() {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/xyz");
        assert!(find_large_files(bogus, 0).is_err());
    }
}
```

Note the Tauri command is named `find_large_files_cmd` (not `find_large_files`) to avoid colliding with the pure-logic function of the same conceptual purpose — this mirrors the `run_lspci()`/`get_pci_devices()` private-helper-vs-public-command naming split used throughout `hardware.rs`/`drivers.rs`/`logs.rs` in Phase 1, just spelled out explicitly here since both names would otherwise be nearly identical.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test largefiles:: 2>&1 | tail -30`
Expected: PASS (3 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod largefiles;` and `largefiles::find_large_files_cmd` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 51 + 3 = 54 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/largefiles.rs src-tauri/src/lib.rs
git commit -m "feat: large file finder"
```

---

## Task 4: Hash checker

**Files:**
- Create: `src-tauri/src/hashcheck.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/hashcheck.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn computes_sha256_of_a_known_file() {
        let path = std::env::temp_dir().join(format!("nitrux-hash-test-{}.txt", std::process::id()));
        std::fs::File::create(&path).unwrap().write_all(b"hello world").unwrap();

        let result = compute_hash(&path, HashAlgorithm::Sha256).expect("should hash");
        std::fs::remove_file(&path).ok();

        // Known SHA-256 of "hello world"
        assert_eq!(result, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde");
    }

    #[test]
    fn errors_on_nonexistent_file() {
        let bogus = std::path::Path::new("/definitely/not/a/real/file/xyz.txt");
        assert!(compute_hash(bogus, HashAlgorithm::Sha256).is_err());
    }

    #[test]
    fn matches_expected_hash_case_insensitively() {
        let path = std::env::temp_dir().join(format!("nitrux-hash-match-{}.txt", std::process::id()));
        std::fs::File::create(&path).unwrap().write_all(b"hello world").unwrap();

        let matches = verify_hash(
            &path,
            HashAlgorithm::Sha256,
            "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE",
        )
        .expect("should verify");
        std::fs::remove_file(&path).ok();

        assert!(matches);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test hashcheck:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `hashcheck.rs`**

```rust
// src-tauri/src/hashcheck.rs
use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

fn hash_with<D: Digest>(mut hasher: D, path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("impossible d'ouvrir {} : {e}", path.display()))?;
    let mut buf = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("erreur de lecture de {} : {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

pub fn compute_hash(path: &Path, algorithm: HashAlgorithm) -> Result<String, String> {
    match algorithm {
        HashAlgorithm::Md5 => hash_with(md5::Md5::new(), path),
        HashAlgorithm::Sha1 => hash_with(Sha1::new(), path),
        HashAlgorithm::Sha256 => hash_with(Sha256::new(), path),
    }
}

pub fn verify_hash(path: &Path, algorithm: HashAlgorithm, expected: &str) -> Result<bool, String> {
    let actual = compute_hash(path, algorithm)?;
    Ok(actual.eq_ignore_ascii_case(expected.trim()))
}

#[tauri::command]
pub fn compute_file_hash(path: String, algorithm: HashAlgorithm) -> Result<String, String> {
    compute_hash(std::path::Path::new(&path), algorithm)
}

#[tauri::command]
pub fn verify_file_hash(path: String, algorithm: HashAlgorithm, expected: String) -> Result<bool, String> {
    verify_hash(std::path::Path::new(&path), algorithm, &expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn computes_sha256_of_a_known_file() {
        let path = std::env::temp_dir().join(format!("nitrux-hash-test-{}.txt", std::process::id()));
        std::fs::File::create(&path).unwrap().write_all(b"hello world").unwrap();

        let result = compute_hash(&path, HashAlgorithm::Sha256).expect("should hash");
        std::fs::remove_file(&path).ok();

        assert_eq!(result, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde");
    }

    #[test]
    fn errors_on_nonexistent_file() {
        let bogus = std::path::Path::new("/definitely/not/a/real/file/xyz.txt");
        assert!(compute_hash(bogus, HashAlgorithm::Sha256).is_err());
    }

    #[test]
    fn matches_expected_hash_case_insensitively() {
        let path = std::env::temp_dir().join(format!("nitrux-hash-match-{}.txt", std::process::id()));
        std::fs::File::create(&path).unwrap().write_all(b"hello world").unwrap();

        let matches = verify_hash(
            &path,
            HashAlgorithm::Sha256,
            "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE",
        )
        .expect("should verify");
        std::fs::remove_file(&path).ok();

        assert!(matches);
    }
}
```

This task introduces two more hashing dependencies: `md-5` and `sha1` (alongside `sha2`, added in Task 2), both from the RustCrypto family so they share the `Digest` trait used by `hash_with<D: Digest>`.

- [ ] **Step 3.5: Add the `md-5` and `sha1` dependencies**

Modify `src-tauri/Cargo.toml` — add under `[dependencies]`:

```toml
md-5 = "0.10"
sha1 = "0.10"
```

**Verify the exact import path before trusting the plan's `hash_with(md5::Md5::new(), path)` call above** — the crate is published as `md-5` (hyphenated) but Rust's import name may differ (check via `cargo doc --open -p md-5` or docs.rs once the dependency is added). If `md5::Md5` doesn't resolve, adapt the `use` statement/call site to whatever the actual generated crate root name is — this is the same kind of plan-vs-reality API drift already hit and correctly handled in Phase 1 (Task 9's `sysinfo::Component::temperature()` returning `f32` not `Option<f32>`) and Phase 2 — fix it with compiler feedback, don't get stuck guessing.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test hashcheck:: 2>&1 | tail -30`
Expected: PASS (3 tests)

- [ ] **Step 5: Register the commands**

Modify `src-tauri/src/lib.rs` — add `mod hashcheck;` and both `hashcheck::compute_file_hash`, `hashcheck::verify_file_hash` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 54 + 3 = 57 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/hashcheck.rs src-tauri/src/lib.rs src-tauri/Cargo.toml
git commit -m "feat: file hash checker (MD5/SHA-1/SHA-256)"
```

---

## Task 5: SMART disk health query

**Files:**
- Create: `src-tauri/src/smart.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/smart.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smart_overall_health_line() {
        let output = "SMART overall-health self-assessment test result: PASSED\n";
        assert_eq!(parse_health_line(output), Some("PASSED".to_string()));
    }

    #[test]
    fn returns_none_when_health_line_is_absent() {
        let output = "smartctl 7.2 2020-12-30 r5155\nSome other output\n";
        assert_eq!(parse_health_line(output), None);
    }

    #[test]
    fn handles_failed_health_status() {
        let output = "SMART overall-health self-assessment test result: FAILED!\n";
        assert_eq!(parse_health_line(output), Some("FAILED!".to_string()));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test smart:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `smart.rs`**

```rust
// src-tauri/src/smart.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct SmartStatus {
    pub device: String,
    pub health: Option<String>,
}

/// Extracts the value after "SMART overall-health self-assessment test
/// result:" from `smartctl -H` output, e.g. "PASSED" or "FAILED!".
pub fn parse_health_line(output: &str) -> Option<String> {
    output
        .lines()
        .find(|l| l.contains("SMART overall-health self-assessment test result:"))
        .and_then(|l| l.split(':').nth(1))
        .map(|s| s.trim().to_string())
}

/// Queries SMART health for `device` (e.g. "/dev/sda"). `smartctl` commonly
/// requires root to access the raw device — a permission-denied failure is
/// surfaced as a normal `Err`, not a crash. This is a real, expected
/// limitation on most systems (see design spec §5.1's note on `dmidecode`
/// having the same root requirement), not something this task works around.
#[tauri::command]
pub fn get_smart_status(device: String) -> Result<SmartStatus, String> {
    let output = subprocess::run_with_timeout("smartctl", &["-H", &device], Duration::from_secs(15))?;
    Ok(SmartStatus {
        device,
        health: parse_health_line(&output),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_smart_overall_health_line() {
        let output = "SMART overall-health self-assessment test result: PASSED\n";
        assert_eq!(parse_health_line(output), Some("PASSED".to_string()));
    }

    #[test]
    fn returns_none_when_health_line_is_absent() {
        let output = "smartctl 7.2 2020-12-30 r5155\nSome other output\n";
        assert_eq!(parse_health_line(output), None);
    }

    #[test]
    fn handles_failed_health_status() {
        let output = "SMART overall-health self-assessment test result: FAILED!\n";
        assert_eq!(parse_health_line(output), Some("FAILED!".to_string()));
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test smart:: 2>&1 | tail -30`
Expected: PASS (3 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod smart;` and `smart::get_smart_status` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 57 + 3 = 60 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/smart.rs src-tauri/src/lib.rs
git commit -m "feat: SMART disk health query"
```

---

## Task 6: `DisksPage.vue` frontend

**Files:**
- Create: `src/pages/DisksPage.vue`

- [ ] **Step 1: Build the tabbed page**

Follows `ThemeEditorPage.vue`'s tabbed pattern (Task 7 of Phase 1) for tab switching, and the established `try/catch` + visible `error` ref pattern from every prior data-fetching page for each individual action.

```vue
<!-- src/pages/DisksPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface Partition { name: string; size: string; mountpoint: string | null }
interface Disk { name: string; size: string; partitions: Partition[] }
interface UsageEntry { mountpoint: string; total_bytes: number; used_bytes: number; used_percent: number }
interface DuplicateGroup { hash: string; paths: string[]; size_bytes: number }
interface LargeFile { path: string; size_bytes: number }

type Tab = "disks" | "duplicates" | "largefiles" | "hashcheck";
const activeTab = ref<Tab>("disks");

const disks = ref<Disk[]>([]);
const usage = ref<UsageEntry[]>([]);
const disksError = ref<string | null>(null);

async function loadDisks() {
  disksError.value = null;
  try {
    disks.value = await invoke<Disk[]>("list_disks");
    usage.value = await invoke<UsageEntry[]>("list_disk_usage");
  } catch (e) {
    disksError.value = String(e);
  }
}
onMounted(loadDisks);

const scanDir = ref("");
const duplicateGroups = ref<DuplicateGroup[]>([]);
const duplicatesError = ref<string | null>(null);
const duplicatesLoading = ref(false);

async function scanDuplicates() {
  duplicatesLoading.value = true;
  duplicatesError.value = null;
  try {
    duplicateGroups.value = await invoke<DuplicateGroup[]>("find_duplicate_files", { directory: scanDir.value });
  } catch (e) {
    duplicatesError.value = String(e);
  } finally {
    duplicatesLoading.value = false;
  }
}

const largeFileDir = ref("");
const minSizeMb = ref(100);
const largeFiles = ref<LargeFile[]>([]);
const largeFilesError = ref<string | null>(null);
const largeFilesLoading = ref(false);

async function scanLargeFiles() {
  largeFilesLoading.value = true;
  largeFilesError.value = null;
  try {
    largeFiles.value = await invoke<LargeFile[]>("find_large_files_cmd", {
      directory: largeFileDir.value,
      minSizeBytes: minSizeMb.value * 1024 * 1024,
    });
  } catch (e) {
    largeFilesError.value = String(e);
  } finally {
    largeFilesLoading.value = false;
  }
}

const hashPath = ref("");
const hashAlgorithm = ref<"sha256" | "sha1" | "md5">("sha256");
const hashResult = ref<string | null>(null);
const hashError = ref<string | null>(null);

async function computeHash() {
  hashError.value = null;
  hashResult.value = null;
  try {
    hashResult.value = await invoke<string>("compute_file_hash", { path: hashPath.value, algorithm: hashAlgorithm.value });
  } catch (e) {
    hashError.value = String(e);
  }
}

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}

function bytesToMb(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="disks-page">
    <h1>Disques & stockage</h1>

    <div class="disks-tabs">
      <button :class="{ active: activeTab === 'disks' }" @click="activeTab = 'disks'">Disques</button>
      <button :class="{ active: activeTab === 'duplicates' }" @click="activeTab = 'duplicates'">Doublons</button>
      <button :class="{ active: activeTab === 'largefiles' }" @click="activeTab = 'largefiles'">Gros fichiers</button>
      <button :class="{ active: activeTab === 'hashcheck' }" @click="activeTab = 'hashcheck'">Vérif. hash</button>
    </div>

    <section v-if="activeTab === 'disks'" class="disks-panel">
      <div v-if="disksError" class="disks-error">{{ disksError }}</div>
      <div v-for="disk in disks" :key="disk.name" class="disks-disk-card">
        <strong>{{ disk.name }}</strong> — {{ disk.size }}
        <ul>
          <li v-for="p in disk.partitions" :key="p.name">{{ p.name }} ({{ p.size }}){{ p.mountpoint ? ` → ${p.mountpoint}` : "" }}</li>
        </ul>
      </div>
      <div v-for="u in usage" :key="u.mountpoint" class="disks-usage-row">
        <span>{{ u.mountpoint }}</span>
        <span>{{ bytesToGb(u.used_bytes) }} / {{ bytesToGb(u.total_bytes) }} GB ({{ u.used_percent }}%)</span>
      </div>
    </section>

    <section v-else-if="activeTab === 'duplicates'" class="disks-panel">
      <div class="disks-form-row">
        <input v-model="scanDir" class="disks-input" placeholder="Dossier à scanner..." />
        <button :disabled="duplicatesLoading" @click="scanDuplicates">{{ duplicatesLoading ? "Analyse..." : "Rechercher" }}</button>
      </div>
      <div v-if="duplicatesError" class="disks-error">{{ duplicatesError }}</div>
      <div v-for="g in duplicateGroups" :key="g.hash" class="disks-dup-group">
        <div>{{ g.paths.length }} fichiers identiques ({{ bytesToMb(g.size_bytes) }} MB chacun)</div>
        <ul><li v-for="p in g.paths" :key="p">{{ p }}</li></ul>
      </div>
    </section>

    <section v-else-if="activeTab === 'largefiles'" class="disks-panel">
      <div class="disks-form-row">
        <input v-model="largeFileDir" class="disks-input" placeholder="Dossier à scanner..." />
        <input v-model.number="minSizeMb" type="number" class="disks-input-small" /> MB min
        <button :disabled="largeFilesLoading" @click="scanLargeFiles">{{ largeFilesLoading ? "Analyse..." : "Rechercher" }}</button>
      </div>
      <div v-if="largeFilesError" class="disks-error">{{ largeFilesError }}</div>
      <div v-for="f in largeFiles" :key="f.path" class="disks-usage-row">
        <span>{{ f.path }}</span>
        <span>{{ bytesToMb(f.size_bytes) }} MB</span>
      </div>
    </section>

    <section v-else class="disks-panel">
      <div class="disks-form-row">
        <input v-model="hashPath" class="disks-input" placeholder="Chemin du fichier..." />
        <select v-model="hashAlgorithm">
          <option value="sha256">SHA-256</option>
          <option value="sha1">SHA-1</option>
          <option value="md5">MD5</option>
        </select>
        <button @click="computeHash">Calculer</button>
      </div>
      <div v-if="hashError" class="disks-error">{{ hashError }}</div>
      <div v-if="hashResult" class="disks-hash-result">{{ hashResult }}</div>
    </section>
  </div>
</template>

<style scoped>
.disks-page { padding: 24px; color: var(--nx-text-primary); }
.disks-tabs { display: flex; gap: 8px; margin: 16px 0; }
.disks-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.disks-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.disks-panel { display: flex; flex-direction: column; gap: 10px; }
.disks-error { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.disks-disk-card { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 10px; padding: 12px; }
.disks-usage-row, .disks-form-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; }
.disks-input { flex: 1; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.disks-input-small { width: 80px; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.disks-dup-group { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 10px; padding: 12px; font-size: 13px; }
.disks-hash-result { font-family: monospace; padding: 10px 14px; border-radius: 8px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); word-break: break-all; }
</style>
```

- [ ] **Step 2: Type-check**

Run: `npx vue-tsc --noEmit`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/pages/DisksPage.vue
git commit -m "feat: DisksPage.vue with disk listing, duplicates, large files, hash check tabs"
```

---

## Task 7: Wire `DisksPage` into `App.vue` navigation, final verification

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Add the page to the nav**

Read the current `src/App.vue` (has `dashboard`/`hardware`/`drivers`/`logs`/`theme-editor`/`packages` wired, 6 pages, `Record<PageId, Component>` strictly typed). Add `"disks"` to `PageId`, import `DisksPage`, add to the `pages` map, add a 7th nav button ("Disques") matching the established pattern exactly.

- [ ] **Step 2: Run the full test suite**

Run: `npm run test` — expect 25 passed (unchanged, no new frontend spec files this plan).
Run: `cd src-tauri && cargo test` — expect 60 passed, 1 ignored.
Run: `npx vue-tsc --noEmit` — expect clean.

- [ ] **Step 3: Manual GUI verification in WSL2**

Same technique as every prior final-wiring task (Phase 1 Task 13, Phase 2 Task 8): boot `npm run tauri dev`, confirm real window/surface creation via `/proc/<pid>/fd`, check dev log for errors, kill processes. On this WSL2 host, `lsblk`/`df` ARE present (standard util-linux/coreutils) — `list_disks`/`list_disk_usage` should return real data. `smartctl` likely is NOT installed and/or requires root — `get_smart_status` failing gracefully (not crashing) when manually reasoned about is sufficient; it won't be exercised by the boot-only verification since no page auto-calls it (all 4 tab actions in `DisksPage.vue` are user-triggered, not `onMounted`, except the "Disques" tab's own `list_disks`/`list_disk_usage`, which DO run on mount — verify these two specifically return real data or a clean error, not a crash).

- [ ] **Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: wire DisksPage into app navigation"
```
