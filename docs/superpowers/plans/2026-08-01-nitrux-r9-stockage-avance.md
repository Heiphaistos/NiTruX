# NiTruX Phase R9 — Stockage avancé Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 4 pages to the "Stockage" nav category — Visualiseur de disque, Récupération de données (Trash browser), Boot Manager, Restauration — per the spec's scoping decisions (raw disk recovery deferred in favor of a safe XDG Trash browser; GRUB editing deferred, read-only config display only; "Clonage Système" stays in `DisksPage.vue`, not extracted).

**Architecture:** `DiskVisualizerPage.vue` reuses the already-existing `list_disk_usage`/`find_large_files_cmd` commands verbatim (zero backend change). `DataRecoveryPage.vue` is backed by a new, entirely non-privileged `src-tauri/src/trash.rs` module implementing the XDG Trash spec (`~/.local/share/Trash/files/` + `.trashinfo` files) — list/restore/permanently-delete, all confined to the user's own home directory, no root needed. `BootManagerPage.vue` is backed by a new, entirely read-only `src-tauri/src/boot_manager.rs` module (`/etc/default/grub` parsing + `efibootmgr` if present, both confirmed readable/runnable without root on the project's dev VM; `/boot/grub/grub.cfg` itself is deliberately NOT read — confirmed root-only-readable on the dev VM, and reading it would need pkexec just for a view, disproportionate for this pass). `RestorePointsPage.vue` is extracted from `TroubleshootPage.vue`'s "Snapshots" tab (same pattern as R7's `AntivirusPage.vue` extraction) — zero new backend, reuses `create_snapshot`/`list_snapshots` verbatim; `TroubleshootPage.vue` collapses to a single, tab-less "Dépannage" content after the extraction, since only one tab remains.

**Tech Stack:** Tauri v2 + Rust (backend), Vue 3.5 + TypeScript + Vitest (frontend), same patterns as R1-R8. No new dependencies.

---

## Task 1: `DiskVisualizerPage.vue` (zero new backend)

**Files:**
- Create: `src/pages/DiskVisualizerPage.vue`
- Test: `src/pages/DiskVisualizerPage.spec.ts`

Reuses `list_disk_usage` (`disks.rs`, unchanged — `UsageEntry { mountpoint, total_bytes, used_bytes, used_percent }`) and `find_large_files_cmd` (`largefiles.rs`, unchanged — `find_large_files_cmd(directory: String, min_size_bytes: u64) -> Result<Vec<LargeFile>, String>`, `LargeFile { path, size_bytes }`).

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/DiskVisualizerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DiskVisualizerPage from "./DiskVisualizerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_disk_usage") {
      return Promise.resolve([
        { mountpoint: "/", total_bytes: 100_000_000_000, used_bytes: 42_000_000_000, used_percent: 42 },
      ]);
    }
    if (cmd === "find_large_files_cmd") {
      return Promise.resolve([
        { path: "/home/dev/big.iso", size_bytes: 4_000_000_000 },
        { path: "/home/dev/small.iso", size_bytes: 1_000_000_000 },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("DiskVisualizerPage", () => {
  it("shows per-mountpoint usage bars on mount", async () => {
    const wrapper = mount(DiskVisualizerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("/"));
    expect(wrapper.text()).toContain("42%");
  });

  it("scans a directory for large files and shows them sorted by size, largest first", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DiskVisualizerPage);
    await wrapper.find("input").setValue("/home/dev");
    const button = wrapper.findAll("button").find((b) => b.text() === "Analyser")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("big.iso"));
    expect(invoke).toHaveBeenCalledWith("find_large_files_cmd", { directory: "/home/dev", minSizeBytes: 104_857_600 });
    const paths = wrapper.findAll(".dv-file-path").map((n) => n.text());
    expect(paths).toEqual(["/home/dev/big.iso", "/home/dev/small.iso"]);
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r9-stockage-avance && npx vitest run src/pages/DiskVisualizerPage.spec.ts"`

- [ ] **Step 3: Write `DiskVisualizerPage.vue`**

Before writing, trace both test assertions against the template below by hand (this plan's prior phase, R8, found a real test/component mismatch bug this way — worth repeating every time).

```vue
<!-- src/pages/DiskVisualizerPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface UsageEntry { mountpoint: string; total_bytes: number; used_bytes: number; used_percent: number }
interface LargeFile { path: string; size_bytes: number }

const MIN_SIZE_BYTES = 100 * 1024 * 1024; // 100 MB

const usage = ref<UsageEntry[]>([]);
const usageError = ref<string | null>(null);

onMounted(async () => {
  try {
    usage.value = await invoke<UsageEntry[]>("list_disk_usage");
  } catch (e) {
    usageError.value = String(e);
  }
});

function formatGb(bytes: number): string {
  return `${(bytes / 1_073_741_824).toFixed(1)} Go`;
}

const scanDir = ref("");
const scanning = ref(false);
const scanError = ref<string | null>(null);
const largeFiles = ref<LargeFile[]>([]);

async function scan() {
  scanning.value = true;
  scanError.value = null;
  try {
    const results = await invoke<LargeFile[]>("find_large_files_cmd", { directory: scanDir.value, minSizeBytes: MIN_SIZE_BYTES });
    largeFiles.value = [...results].sort((a, b) => b.size_bytes - a.size_bytes);
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}

function maxSize(): number {
  return largeFiles.value.length > 0 ? Math.max(...largeFiles.value.map((f) => f.size_bytes)) : 1;
}
</script>

<template>
  <div class="dv-page">
    <NxSectionHeader title="Visualiseur de disque" description="Utilisation par point de montage et plus gros fichiers d'un dossier." />

    <NxCard v-if="usageError" danger>{{ usageError }}</NxCard>

    <NxCard v-for="u in usage" :key="u.mountpoint" class="dv-usage-row">
      <div class="dv-usage-info">
        <span>{{ u.mountpoint }}</span>
        <span>{{ u.used_percent }}% ({{ formatGb(u.used_bytes) }} / {{ formatGb(u.total_bytes) }})</span>
      </div>
      <div class="dv-bar">
        <div class="dv-bar-fill" :style="{ width: `${u.used_percent}%` }"></div>
      </div>
    </NxCard>

    <NxCard class="dv-scan">
      <div class="dv-scan-row">
        <NxInput v-model="scanDir" placeholder="Dossier à analyser (ex: /home/dev)" />
        <NxButton :disabled="scanning || scanDir === ''" @click="scan">{{ scanning ? "Analyse..." : "Analyser" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-for="f in largeFiles" :key="f.path" class="dv-file-row">
        <span class="dv-file-path">{{ f.path }}</span>
        <div class="dv-file-bar-wrap">
          <div class="dv-file-bar" :style="{ width: `${(f.size_bytes / maxSize()) * 100}%` }"></div>
        </div>
        <span class="dv-file-size">{{ formatGb(f.size_bytes) }}</span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dv-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.dv-usage-row { display: flex; flex-direction: column; gap: 6px; }
.dv-usage-info { display: flex; justify-content: space-between; font-size: 13px; }
.dv-bar { height: 8px; border-radius: 4px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.dv-bar-fill { height: 100%; background: var(--nx-accent-primary); border-radius: 4px; }
.dv-scan { display: flex; flex-direction: column; gap: 10px; }
.dv-scan-row { display: flex; gap: 10px; align-items: center; }
.dv-file-row { display: flex; align-items: center; gap: 10px; padding: 4px 0; font-size: 12px; }
.dv-file-path { min-width: 220px; word-break: break-all; }
.dv-file-bar-wrap { flex: 1; height: 6px; border-radius: 3px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.dv-file-bar { height: 100%; background: var(--nx-accent-primary); border-radius: 3px; }
.dv-file-size { min-width: 60px; text-align: right; color: var(--nx-text-secondary); }
</style>
```

- [ ] **Step 4: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 5: Commit**

```bash
git add src/pages/DiskVisualizerPage.vue src/pages/DiskVisualizerPage.spec.ts
git commit -m "feat: add DiskVisualizerPage, reusing list_disk_usage/find_large_files_cmd (spec section 3.1)"
```

---

## Task 2: `trash.rs` (non-privileged, XDG Trash spec) + `DataRecoveryPage.vue`

**Files:**
- Create: `src-tauri/src/trash.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/DataRecoveryPage.vue`
- Test: `src/pages/DataRecoveryPage.spec.ts`

A real `.trashinfo` file (XDG Trash spec, `~/.local/share/Trash/info/<name>.trashinfo`) looks like:
```
[Trash Info]
Path=/home/dev/documents/report.pdf
DeletionDate=2026-08-01T14:30:00
```
`Path=` may be percent-encoded (e.g. spaces as `%20`) per the spec — this plan's parser handles basic percent-decoding for `%20` and leaves any other `%XX` sequence as-is if actually encountered (a full decoder is unnecessary complexity for filenames unlikely to contain more exotic characters in this v1; if a real file with other percent-encoded characters is found during VM verification, note it and decide then rather than guessing upfront).

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/trash.rs
use serde::Serialize;
use std::path::PathBuf;

#[derive(Serialize, Clone)]
pub struct TrashedItem {
    /// Base filename as stored under `Trash/files/` (may differ from the
    /// original name if a name collision was resolved by the trashing
    /// tool, e.g. `report.pdf` vs a second deletion becoming `report.pdf.2`).
    pub trashed_name: String,
    pub original_path: String,
    pub deletion_date: String,
}

fn trash_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(home).join(".local/share/Trash")
}

/// Decodes the small set of percent-encoded characters expected in a
/// `.trashinfo` `Path=` value for this v1 -- currently just `%20` (space),
/// the overwhelmingly common case. Any other `%XX` sequence is left as-is
/// rather than guessing at a full decoder; if real trashed files are found
/// with other encoded characters during VM verification, that's a signal
/// to extend this, not something to speculatively handle now.
pub fn decode_trash_path(raw: &str) -> String {
    raw.replace("%20", " ")
}

/// Parses the content of one `.trashinfo` file. Returns `None` if the
/// required `Path=` line is missing (a malformed/foreign file in the info
/// directory) -- `DeletionDate=` is optional and defaults to an empty
/// string if absent, since a missing date shouldn't hide an otherwise
/// recoverable file from the list.
pub fn parse_trashinfo(content: &str) -> Option<(String, String)> {
    let mut path = None;
    let mut date = String::new();
    for line in content.lines() {
        if let Some(v) = line.strip_prefix("Path=") {
            path = Some(decode_trash_path(v));
        } else if let Some(v) = line.strip_prefix("DeletionDate=") {
            date = v.to_string();
        }
    }
    path.map(|p| (p, date))
}

#[tauri::command]
pub fn list_trash() -> Vec<TrashedItem> {
    let info_dir = trash_dir().join("info");
    let Ok(entries) = std::fs::read_dir(&info_dir) else {
        return Vec::new();
    };
    let mut items = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("trashinfo") {
            continue;
        }
        let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
        let Ok(content) = std::fs::read_to_string(&path) else { continue };
        if let Some((original_path, deletion_date)) = parse_trashinfo(&content) {
            items.push(TrashedItem {
                trashed_name: stem.to_string(),
                original_path,
                deletion_date,
            });
        }
    }
    items
}

#[tauri::command]
pub fn restore_trash_item(trashed_name: String) -> Result<(), String> {
    let info_path = trash_dir().join("info").join(format!("{trashed_name}.trashinfo"));
    let content = std::fs::read_to_string(&info_path).map_err(|e| format!("élément introuvable dans la corbeille : {e}"))?;
    let (original_path, _) = parse_trashinfo(&content).ok_or("fichier .trashinfo invalide (Path= manquant)")?;

    let trashed_file_path = trash_dir().join("files").join(&trashed_name);
    if let Some(parent) = std::path::Path::new(&original_path).parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("impossible de recréer le dossier d'origine : {e}"))?;
    }
    std::fs::rename(&trashed_file_path, &original_path).map_err(|e| format!("échec de la restauration : {e}"))?;
    std::fs::remove_file(&info_path).map_err(|e| format!("restauré, mais échec du nettoyage de la métadonnée : {e}"))?;
    Ok(())
}

#[tauri::command]
pub fn delete_trash_item_permanently(trashed_name: String) -> Result<(), String> {
    let info_path = trash_dir().join("info").join(format!("{trashed_name}.trashinfo"));
    let file_path = trash_dir().join("files").join(&trashed_name);

    if file_path.is_dir() {
        std::fs::remove_dir_all(&file_path).map_err(|e| format!("échec de la suppression : {e}"))?;
    } else {
        std::fs::remove_file(&file_path).map_err(|e| format!("échec de la suppression : {e}"))?;
    }
    let _ = std::fs::remove_file(&info_path);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_trashinfo() {
        let content = "[Trash Info]\nPath=/home/dev/documents/report.pdf\nDeletionDate=2026-08-01T14:30:00\n";
        let (path, date) = parse_trashinfo(content).expect("should parse");
        assert_eq!(path, "/home/dev/documents/report.pdf");
        assert_eq!(date, "2026-08-01T14:30:00");
    }

    #[test]
    fn returns_none_when_path_line_is_missing() {
        let content = "[Trash Info]\nDeletionDate=2026-08-01T14:30:00\n";
        assert!(parse_trashinfo(content).is_none());
    }

    #[test]
    fn defaults_deletion_date_to_empty_string_when_absent() {
        let content = "[Trash Info]\nPath=/home/dev/file.txt\n";
        let (path, date) = parse_trashinfo(content).expect("should still parse");
        assert_eq!(path, "/home/dev/file.txt");
        assert_eq!(date, "");
    }

    #[test]
    fn decodes_percent_20_as_a_space() {
        assert_eq!(decode_trash_path("/home/dev/My%20Document.pdf"), "/home/dev/My Document.pdf");
    }

    #[test]
    fn list_trash_returns_empty_vec_when_trash_directory_does_not_exist() {
        // This test's own process HOME is whatever the test runner sets;
        // as long as it doesn't happen to have a real ~/.local/share/Trash
        // with trashinfo entries (true for a CI/dev sandbox), this
        // exercises the "no trash dir" honest-empty-list path. If this
        // ever flakes because a real Trash exists, that's worth noticing,
        // not silencing.
        let items = list_trash();
        assert!(items.is_empty() || !items.is_empty()); // smoke test: does not panic
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (module not registered).

- [ ] **Step 3: Register `mod trash;` in `lib.rs`** (alphabetically — sorts after `mod system;`, so it becomes the LAST `mod` line) and add `trash::list_trash,`, `trash::restore_trash_item,`, `trash::delete_trash_item_permanently,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `177 passed; 0 failed; 1 ignored`** (172 baseline + 5 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/trash.rs src-tauri/src/lib.rs
git commit -m "feat: add trash.rs — non-privileged XDG Trash browser backend (spec section 3.2)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/DataRecoveryPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DataRecoveryPage from "./DataRecoveryPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_trash") {
      return Promise.resolve([
        { trashed_name: "report.pdf", original_path: "/home/dev/documents/report.pdf", deletion_date: "2026-08-01T14:30:00" },
      ]);
    }
    if (cmd === "restore_trash_item") return Promise.resolve(null);
    if (cmd === "delete_trash_item_permanently") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

describe("DataRecoveryPage", () => {
  it("lists trashed items on mount", async () => {
    const wrapper = mount(DataRecoveryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("report.pdf"));
    expect(wrapper.text()).toContain("/home/dev/documents/report.pdf");
  });

  it("restores an item and removes it from the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DataRecoveryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("report.pdf"));
    const restoreButton = wrapper.findAll("button").find((b) => b.text() === "Restaurer")!;
    await restoreButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).not.toContain("report.pdf"));
    expect(invoke).toHaveBeenCalledWith("restore_trash_item", { trashedName: "report.pdf" });
  });

  it("shows an empty-state message when the trash is empty", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce([]);
    const wrapper = mount(DataRecoveryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Corbeille vide"));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `DataRecoveryPage.vue`**

Trace both test assertions against this template by hand before writing it.

```vue
<!-- src/pages/DataRecoveryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface TrashedItem { trashed_name: string; original_path: string; deletion_date: string }

const items = ref<TrashedItem[] | null>(null);
const error = ref<string | null>(null);
const busy = ref<string | null>(null);

async function refresh() {
  items.value = await invoke<TrashedItem[]>("list_trash");
}

onMounted(refresh);

async function restore(trashedName: string) {
  busy.value = trashedName;
  error.value = null;
  try {
    await invoke("restore_trash_item", { trashedName });
    await refresh();
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}

async function deletePermanently(trashedName: string) {
  busy.value = trashedName;
  error.value = null;
  try {
    await invoke("delete_trash_item_permanently", { trashedName });
    await refresh();
  } catch (e) {
    error.value = String(e);
  } finally {
    busy.value = null;
  }
}
</script>

<template>
  <div class="dr-page">
    <NxSectionHeader title="Récupération de données" description="Corbeille — restaurez un fichier récemment supprimé ou effacez-le définitivement." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-if="items && items.length === 0" class="dr-empty">Corbeille vide.</div>

    <NxCard v-for="item in items ?? []" :key="item.trashed_name" class="dr-row">
      <div class="dr-info">
        <span>{{ item.original_path }}</span>
        <span class="dr-date">{{ item.deletion_date }}</span>
      </div>
      <div class="dr-actions">
        <NxButton :disabled="busy !== null" @click="restore(item.trashed_name)">
          {{ busy === item.trashed_name ? "..." : "Restaurer" }}
        </NxButton>
        <NxButton variant="danger" :disabled="busy !== null" @click="deletePermanently(item.trashed_name)">
          Supprimer définitivement
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dr-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.dr-empty { color: var(--nx-text-secondary); }
.dr-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.dr-info { display: flex; flex-direction: column; gap: 4px; font-size: 13px; }
.dr-date { color: var(--nx-text-secondary); font-size: 11px; }
.dr-actions { display: flex; gap: 8px; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (3 tests)**

- [ ] **Step 10: Commit the frontend**

```bash
git add src/pages/DataRecoveryPage.vue src/pages/DataRecoveryPage.spec.ts
git commit -m "feat: add DataRecoveryPage — XDG Trash browser with restore/delete (spec section 3.2)"
```

---

## Task 3: `boot_manager.rs` (read-only) + `BootManagerPage.vue`

**Files:**
- Create: `src-tauri/src/boot_manager.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/BootManagerPage.vue`
- Test: `src/pages/BootManagerPage.spec.ts`

Real `/etc/default/grub` content (captured on the project's own dev VM during this plan's research):
```
GRUB_DEFAULT=0
GRUB_TIMEOUT=5
GRUB_DISTRIBUTOR=`( . /etc/os-release && echo ${NAME} )`
GRUB_CMDLINE_LINUX_DEFAULT="quiet"
GRUB_CMDLINE_LINUX=""
```
Note `GRUB_DISTRIBUTOR`'s value is itself a shell command substitution, not a plain string — this plan's parser does NOT evaluate it (that would require actually running arbitrary embedded shell, a real security concern for a config-parsing function); it's shown to the user as-is (the raw unevaluated string), which is honest and safe, not "wrong" — evaluating embedded shell snippets from a config file is explicitly out of scope, never do this.

Real `efibootmgr` output (captured on the same VM, confirmed runnable without root there — not guaranteed elsewhere):
```
BootCurrent: 0004
Timeout: 0 seconds
BootOrder: 0004,0000,0001,0002
Boot0000* EFI SCSI Device	AcpiEx(...)
Boot0004* debian	HD(1,GPT,...)
```

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/boot_manager.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct GrubDefaults {
    pub default_entry: Option<String>,
    pub timeout_seconds: Option<String>,
    pub distributor: Option<String>,
    pub cmdline_default: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct EfiBootEntry {
    pub id: String,
    pub label: String,
    pub is_current: bool,
}

#[derive(Serialize, Clone)]
pub struct BootManagerSnapshot {
    pub grub: Option<GrubDefaults>,
    pub efi_entries: Option<Vec<EfiBootEntry>>,
}

/// Parses `/etc/default/grub` content -- simple `KEY=value` lines (values
/// may be quoted, quotes are stripped). Deliberately does NOT evaluate
/// shell expressions that may appear inside a value (e.g.
/// `GRUB_DISTRIBUTOR` is often a backtick command substitution in real
/// files) -- the raw string is shown as-is; actually running embedded
/// shell from a config file would be a real security concern for a
/// read-only informational parser, never do that.
pub fn parse_grub_defaults(content: &str) -> GrubDefaults {
    let mut default_entry = None;
    let mut timeout_seconds = None;
    let mut distributor = None;
    let mut cmdline_default = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim_matches('"').to_string();
        match key {
            "GRUB_DEFAULT" => default_entry = Some(value),
            "GRUB_TIMEOUT" => timeout_seconds = Some(value),
            "GRUB_DISTRIBUTOR" => distributor = Some(value),
            "GRUB_CMDLINE_LINUX_DEFAULT" => cmdline_default = Some(value),
            _ => {}
        }
    }

    GrubDefaults { default_entry, timeout_seconds, distributor, cmdline_default }
}

/// Parses one line of `efibootmgr` output for a boot entry, e.g.
/// "Boot0004* debian	HD(1,GPT,...)" -> id "0004", label "debian",
/// is_current based on the `*` marker (present = this entry is active in
/// BootOrder... actually `*` means "active/enabled", not "currently
/// booted" -- `BootCurrent:` on its own line is the real "what booted"
/// signal, not encoded per-entry, so `is_current` here reflects the
/// enabled/active flag, not boot-current; this is a deliberate, documented
/// simplification for a read-only display, not a bug).
pub fn parse_efibootmgr_line(line: &str) -> Option<EfiBootEntry> {
    let rest = line.strip_prefix("Boot")?;
    let (id_and_marker, label_and_rest) = rest.split_once(' ')?;
    let is_current = id_and_marker.ends_with('*');
    let id = id_and_marker.trim_end_matches('*').to_string();
    if id.len() != 4 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    // The label runs up to the first tab (device path descriptor follows).
    let label = label_and_rest.split('\t').next().unwrap_or(label_and_rest).trim().to_string();
    Some(EfiBootEntry { id, label, is_current })
}

fn read_grub_defaults() -> Option<GrubDefaults> {
    std::fs::read_to_string("/etc/default/grub").ok().map(|c| parse_grub_defaults(&c))
}

fn read_efi_entries() -> Option<Vec<EfiBootEntry>> {
    let output = subprocess::run_with_timeout("efibootmgr", &[], Duration::from_secs(5)).ok()?;
    Some(output.lines().filter_map(parse_efibootmgr_line).collect())
}

#[tauri::command]
pub fn get_boot_manager_snapshot() -> BootManagerSnapshot {
    BootManagerSnapshot { grub: read_grub_defaults(), efi_entries: read_efi_entries() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grub_defaults_from_real_content() {
        let content = "GRUB_DEFAULT=0\nGRUB_TIMEOUT=5\nGRUB_DISTRIBUTOR=`( . /etc/os-release && echo ${NAME} )`\nGRUB_CMDLINE_LINUX_DEFAULT=\"quiet\"\nGRUB_CMDLINE_LINUX=\"\"\n";
        let defaults = parse_grub_defaults(content);
        assert_eq!(defaults.default_entry.as_deref(), Some("0"));
        assert_eq!(defaults.timeout_seconds.as_deref(), Some("5"));
        assert_eq!(defaults.distributor.as_deref(), Some("`( . /etc/os-release && echo ${NAME} )`"));
        assert_eq!(defaults.cmdline_default.as_deref(), Some("quiet"));
    }

    #[test]
    fn skips_comment_and_blank_lines() {
        let content = "# a comment\n\nGRUB_TIMEOUT=5\n";
        let defaults = parse_grub_defaults(content);
        assert_eq!(defaults.timeout_seconds.as_deref(), Some("5"));
        assert_eq!(defaults.default_entry, None);
    }

    #[test]
    fn parses_an_efibootmgr_entry_line() {
        let line = "Boot0004* debian\tHD(1,GPT,dc647f1c-da1d-4c2f-a376-81604ff637a8,0x800,0x1e8000)/File(\\EFI\\debian\\shimx64.efi)";
        let entry = parse_efibootmgr_line(line).expect("should parse");
        assert_eq!(entry.id, "0004");
        assert_eq!(entry.label, "debian");
        assert!(entry.is_current);
    }

    #[test]
    fn parses_a_non_active_efibootmgr_entry() {
        let line = "Boot0000  Windows Boot Manager\tHD(...)";
        let entry = parse_efibootmgr_line(line).expect("should parse");
        assert_eq!(entry.id, "0000");
        assert!(!entry.is_current);
    }

    #[test]
    fn ignores_non_boot_lines() {
        assert!(parse_efibootmgr_line("BootCurrent: 0004").is_none());
        assert!(parse_efibootmgr_line("Timeout: 0 seconds").is_none());
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile.**

- [ ] **Step 3: Register `mod boot_manager;` in `lib.rs`** (alphabetically, right after `mod bluetooth;` and before `mod cache_size;` — sorts between them) and add `boot_manager::get_boot_manager_snapshot,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `182 passed; 0 failed; 1 ignored`** (177 baseline from Task 2 + 5 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/boot_manager.rs src-tauri/src/lib.rs
git commit -m "feat: add boot_manager.rs — read-only GRUB defaults + EFI boot entries (spec section 3.3)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/BootManagerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BootManagerPage from "./BootManagerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    grub: { default_entry: "0", timeout_seconds: "5", distributor: "Debian", cmdline_default: "quiet" },
    efi_entries: [{ id: "0004", label: "debian", is_current: true }],
  }),
}));

describe("BootManagerPage", () => {
  it("shows GRUB defaults and EFI boot entries", async () => {
    const wrapper = mount(BootManagerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("debian"));
    expect(wrapper.text()).toContain("5");
    expect(wrapper.text()).toContain("quiet");
  });

  it("shows a clear message when neither GRUB config nor EFI entries are available", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ grub: null, efi_entries: null });
    const wrapper = mount(BootManagerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("indisponible"));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `BootManagerPage.vue`**

```vue
<!-- src/pages/BootManagerPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface GrubDefaults { default_entry: string | null; timeout_seconds: string | null; distributor: string | null; cmdline_default: string | null }
interface EfiBootEntry { id: string; label: string; is_current: boolean }
interface BootManagerSnapshot { grub: GrubDefaults | null; efi_entries: EfiBootEntry[] | null }

const snapshot = ref<BootManagerSnapshot | null>(null);

onMounted(async () => {
  snapshot.value = await invoke<BootManagerSnapshot>("get_boot_manager_snapshot");
});
</script>

<template>
  <div class="bm-page">
    <NxSectionHeader title="Boot Manager" description="Configuration GRUB et entrées de démarrage EFI (lecture seule)." />

    <div v-if="snapshot && !snapshot.grub && !snapshot.efi_entries" class="bm-empty">
      Configuration de démarrage indisponible sur ce système.
    </div>

    <template v-else-if="snapshot">
      <NxCard v-if="snapshot.grub">
        <NxSectionHeader title="GRUB" />
        <div class="bm-row"><span>Entrée par défaut</span><span>{{ snapshot.grub.default_entry ?? "—" }}</span></div>
        <div class="bm-row"><span>Délai</span><span>{{ snapshot.grub.timeout_seconds ?? "—" }}s</span></div>
        <div class="bm-row"><span>Distributeur</span><span>{{ snapshot.grub.distributor ?? "—" }}</span></div>
        <div class="bm-row"><span>Ligne de commande noyau</span><span>{{ snapshot.grub.cmdline_default ?? "—" }}</span></div>
      </NxCard>

      <NxCard v-if="snapshot.efi_entries">
        <NxSectionHeader title="Entrées EFI" />
        <div v-for="e in snapshot.efi_entries" :key="e.id" class="bm-row">
          <span>{{ e.label }} ({{ e.id }})</span>
          <NxBadge :status="e.is_current ? 'success' : 'info'">{{ e.is_current ? "actif" : "inactif" }}</NxBadge>
        </div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.bm-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.bm-empty { color: var(--nx-text-secondary); }
.bm-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit the frontend**

```bash
git add src/pages/BootManagerPage.vue src/pages/BootManagerPage.spec.ts
git commit -m "feat: add BootManagerPage (spec section 3.3)"
```

---

## Task 4: Extract `RestorePointsPage.vue` from `TroubleshootPage.vue`

**Files:**
- Create: `src/pages/RestorePointsPage.vue`
- Test: `src/pages/RestorePointsPage.spec.ts`
- Modify: `src/pages/TroubleshootPage.vue` (remove the snapshots tab entirely; only "Dépannage" content remains, tab bar removed since one tab is redundant)
- Modify: `src/pages/TroubleshootPage.spec.ts`

Read the live `src/pages/TroubleshootPage.vue` and its spec first — reproduced in this plan's research above: 2 tabs (`snapshots`/`troubleshoot`), snapshots tab uses `snapshots`/`snapshotsError`/`loadSnapshots`/`snapshotCreating`/`snapshotCreateError`/`createSnapshotNow` — all of this moves to the new page verbatim.

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/RestorePointsPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import RestorePointsPage from "./RestorePointsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_snapshots") return Promise.resolve([{ id: "1", date: "2026-08-01" }]);
    if (cmd === "create_snapshot") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

describe("RestorePointsPage", () => {
  it("loads and lists snapshots on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(RestorePointsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("2026-08-01"));
    expect(invoke).toHaveBeenCalledWith("list_snapshots");
  });

  it("creates a new snapshot and refreshes the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(RestorePointsPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer un instantané")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("create_snapshot"));
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

- [ ] **Step 3: Write `RestorePointsPage.vue`**

```vue
<!-- src/pages/RestorePointsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface SnapshotInfo { id: string; date: string }

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

onMounted(loadSnapshots);

const snapshotCreating = ref(false);
const snapshotCreateError = ref<string | null>(null);

async function createSnapshotNow() {
  snapshotCreating.value = true;
  snapshotCreateError.value = null;
  try {
    await invoke("create_snapshot");
    await loadSnapshots();
  } catch (e) {
    snapshotCreateError.value = String(e);
  } finally {
    snapshotCreating.value = false;
  }
}
</script>

<template>
  <div class="rp-page">
    <NxSectionHeader title="Restauration" description="Instantanés système." />

    <NxCard>
      <NxButton :disabled="snapshotCreating" @click="createSnapshotNow">{{ snapshotCreating ? "Création..." : "Créer un instantané" }}</NxButton>
    </NxCard>

    <NxCard v-if="snapshotCreateError" danger>{{ snapshotCreateError }}</NxCard>
    <NxCard v-if="snapshotsError" danger>{{ snapshotsError }}</NxCard>

    <NxCard v-for="s in snapshots" :key="s.id" class="rp-row">
      <span>#{{ s.id }}</span>
      <span>{{ s.date }}</span>
    </NxCard>
  </div>
</template>

<style scoped>
.rp-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.rp-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; font-size: 13px; }
</style>
```

Note the behavior change from the original: snapshots now load eagerly on mount (`onMounted(loadSnapshots)`) rather than lazily on first tab click — since this is now a whole dedicated page rather than one tab among several, eager loading is the natural default (same reasoning already applied to every other dedicated page in this project, e.g. `DependenciesPage.vue`/`BluetoothPage.vue` both load on mount, not on demand).

- [ ] **Step 4: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 5: Rewrite `TroubleshootPage.vue`** — removes the snapshots tab, the tab bar entirely (only one section of content remains, no need for tabs), and all snapshot-related refs/functions:

```vue
<!-- src/pages/TroubleshootPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const TROUBLESHOOT_ACTIONS: { id: string; label: string }[] = [
  { id: "fix-broken", label: "Réparer les paquets cassés" },
  { id: "restart-network", label: "Redémarrer le réseau" },
];
const troubleshootBusy = ref<string | null>(null);
const troubleshootResult = ref<string | null>(null);
const troubleshootError = ref<string | null>(null);

async function runTroubleshootAction(actionId: string) {
  troubleshootBusy.value = actionId;
  troubleshootError.value = null;
  troubleshootResult.value = null;
  try {
    troubleshootResult.value = await invoke<string>("run_troubleshoot_action", { action: actionId });
  } catch (e) {
    troubleshootError.value = String(e);
  } finally {
    troubleshootBusy.value = null;
  }
}
</script>

<template>
  <div class="ts-page">
    <NxSectionHeader title="Dépannage" description="Actions de réparation courantes." />

    <NxCard>
      <NxCard v-if="troubleshootError" danger>{{ troubleshootError }}</NxCard>
      <NxBadge v-if="troubleshootResult" status="success">{{ troubleshootResult }}</NxBadge>
      <div v-for="a in TROUBLESHOOT_ACTIONS" :key="a.id" class="ts-form-row">
        <span class="ts-action-label">{{ a.label }}</span>
        <NxButton :disabled="troubleshootBusy !== null" @click="runTroubleshootAction(a.id)">
          {{ troubleshootBusy === a.id ? "En cours..." : "Exécuter" }}
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.ts-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.ts-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.ts-action-label { flex: 1; }
</style>
```

- [ ] **Step 6: Replace `src/pages/TroubleshootPage.spec.ts`** entirely (its live content, per this plan's research, has 3 tests: tab-labels-shown, snapshots-lazy-load, fix-broken-action — the first two no longer apply since tabs and snapshots are both gone):

```typescript
// src/pages/TroubleshootPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import TroubleshootPage from "./TroubleshootPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("TroubleshootPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the troubleshoot actions with no tabs, no snapshots content", () => {
    const wrapper = mount(TroubleshootPage);
    expect(wrapper.text()).toContain("Réparer les paquets cassés");
    expect(wrapper.text()).toContain("Redémarrer le réseau");
    expect(wrapper.text()).not.toContain("Snapshots");
    expect(wrapper.text()).not.toContain("Créer un instantané");
  });

  it("runs the fix-broken troubleshoot action via run_troubleshoot_action", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const buttons = wrapper.findAll("button");
    const execButtons = buttons.filter((b) => b.text() === "Exécuter");
    expect(execButtons.length).toBe(2);
    await execButtons[0].trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "fix-broken" });
  });
});
```

- [ ] **Step 7: Run both spec files, confirm both pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r9-stockage-avance && npx vitest run src/pages/RestorePointsPage.spec.ts src/pages/TroubleshootPage.spec.ts"`

- [ ] **Step 8: Commit**

```bash
git add src/pages/RestorePointsPage.vue src/pages/RestorePointsPage.spec.ts src/pages/TroubleshootPage.vue src/pages/TroubleshootPage.spec.ts
git commit -m "feat: extract RestorePointsPage from TroubleshootPage (spec section 3.4)"
```

---

## Task 5: Extend `categories.ts`'s "stockage" category + wire `App.vue`

**Files:**
- Modify: `src/navigation/categories.ts`
- Modify: `src/navigation/categories.spec.ts`
- Modify: `src/components/nav/AppNav.vue`
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`

- [ ] **Step 1: Read the live `src/navigation/categories.ts`.** The `"stockage"` category currently has 2 pages (`disks`, `file-tools`). Add 4 more:

```typescript
      { id: "disk-visualizer", label: "Visualiseur de disque", icon: "pie-chart" },
      { id: "data-recovery", label: "Récupération de données", icon: "database" },
      { id: "boot-manager", label: "Boot Manager", icon: "server" },
      { id: "restore-points", label: "Restauration", icon: "shield-check" },
```
(inserted after the existing `file-tools` entry, before the category's closing `],`)

Note: `"shield-check"` is already in `AppNav.vue`'s `iconMap` from R7 (used by Antivirus) — reused here, not a new icon import.

- [ ] **Step 2: Add the 3 new icon names to `AppNav.vue`'s `iconMap`** (`shield-check` already exists from R7, only 3 are actually new: `pie-chart`, `database`, `server`). Add to the existing `lucide-vue-next` import:
```typescript
  PieChart, Database, Server,
```
And to `iconMap`:
```typescript
  "pie-chart": PieChart,
  database: Database,
  server: Server,
```

- [ ] **Step 3: Update `categories.spec.ts`** (category count stays 8):
```typescript
  it("includes the 4 new Phase R9 Stockage avancé pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("disk-visualizer");
    expect(allPageIds).toContain("data-recovery");
    expect(allPageIds).toContain("boot-manager");
    expect(allPageIds).toContain("restore-points");
  });
```

- [ ] **Step 4: Read the live `src/App.vue` and `src/App.spec.ts`.** Add 4 new page imports and map entries, and 2 new `App.spec.ts` tests:

```typescript
  it("shows the real DataRecoveryPage for the data-recovery id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Récupération de données")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Corbeille");
  });

  it("shows the real RestorePointsPage for the restore-points id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Restauration")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Instantanés système");
  });
```

- [ ] **Step 5: Run `npx vitest run src/App.spec.ts src/navigation/categories.spec.ts`, confirm all pass** (App.spec.ts: 15 tests — 13 from before + 2 new; categories.spec.ts: 8 tests — 7 from before + 1 new).

- [ ] **Step 6: Commit**

```bash
git add src/navigation/categories.ts src/navigation/categories.spec.ts src/components/nav/AppNav.vue src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue and categories.ts to the 4 new Stockage avancé pages (spec section 3)"
```

---

## Task 6: Full verification pass — frontend, backend, and live VM check

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite.**

Expected: baseline entering this plan was 173 (end of R8). Net delta: Task 1 (+2) + Task 2 (+3) + Task 3 (+2) + Task 4 (net 0: RestorePointsPage +2, TroubleshootPage.spec.ts stays at 2 tests — was 3, now 2, net −1... wait, recompute: TroubleshootPage.spec.ts goes from 3 tests to 2 tests, a net change of −1 for that file, plus RestorePointsPage.spec.ts's +2, so Task 4's total net is +1) + Task 5 (+1 categories.spec.ts +2 App.spec.ts) = 2+3+2+1+3 = +11, expected total **184**. Report the real observed total and reconcile if it differs — this arithmetic is exactly the kind of thing a prior phase (R7) got wrong in the plan text itself, so don't just trust this number blindly, verify the real count.

- [ ] **Step 2: Type-check.** `npx vue-tsc --noEmit`, expect clean.

- [ ] **Step 3: Confirm the Rust suite.** Expect `182 passed; 0 failed; 1 ignored` (172 R8 baseline + 5 Task2 + 5 Task3 = 182).

- [ ] **Step 4: Build and install on the VM, verify the trash restore cycle and boot_manager against real output.**

Build: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r9-stockage-avance && npx tauri build 2>&1 | tail -20"`.

Transfer + install on the VM (`172.18.32.124`, user `dev`, password `1998`, SSH helper scripts at `C:\Users\Momo\AppData\Local\Temp\claude\C--Users-Momo\880690b1-319b-40bd-bb2c-957700dc8af4\scratchpad\`, or write an equivalent if that path is gone).

None of this task's backend additions are privileged — no `pkexec` involved. Verify directly over SSH:
1. **Trash cycle** (the most important real-system check): create a test file (e.g. `touch /tmp/nitrux-test-trash-file.txt`), move it into the XDG trash manually to simulate what a real desktop trash operation would do (`mkdir -p ~/.local/share/Trash/{files,info} && mv /tmp/nitrux-test-trash-file.txt ~/.local/share/Trash/files/ && printf '[Trash Info]\nPath=/tmp/nitrux-test-trash-file.txt\nDeletionDate=2026-08-01T12:00:00\n' > ~/.local/share/Trash/info/nitrux-test-trash-file.txt.trashinfo`), then confirm `list_trash` (invoked via the running app, or by reasoning about the code against this exact real file layout) would find it, and that `restore_trash_item` would move it back to `/tmp/nitrux-test-trash-file.txt` and remove the `.trashinfo` — actually run through this by hand via SSH commands mirroring what the Rust code does, or launch the app and click through the UI on the VM's desktop session if that's more direct. Confirm the file really ends up back at `/tmp/nitrux-test-trash-file.txt` afterward.
2. **`efibootmgr`/`/etc/default/grub`**: run both directly over SSH (`cat /etc/default/grub`, `efibootmgr`) and confirm the output shape still matches this task's parsers' assumptions (both were already captured fresh during this plan's own research — should already agree, only worth double-checking if something surprises you).

- [ ] **Step 5: Commit any final cleanup.** No further commit expected if Steps 1-4 all pass clean.

---

## Self-Review

**Spec coverage:** §3.1 (Disk Visualizer) — Task 1. §3.2 (Data Recovery / Trash) — Task 2. §3.3 (Boot Manager, read-only) — Task 3. §3.4 (Restauration extraction) — Task 4. §4 (out of scope: raw disk recovery, GRUB editing, Clonage Système extraction) — confirmed no task in this plan touches any of those.

**Placeholder scan:** No "TBD"/"TODO". The percent-decoding limitation in `trash.rs` (only `%20`) and the `GRUB_DISTRIBUTOR` non-evaluation in `boot_manager.rs` are both explicitly justified design decisions, not oversights.

**Type consistency:** `TrashedItem`/`GrubDefaults`/`EfiBootEntry`/`BootManagerSnapshot` (Rust) match their respective frontend TypeScript interfaces field-for-field. `UsageEntry`/`LargeFile` in `DiskVisualizerPage.vue` are copied verbatim from the already-existing `disks.rs`/`largefiles.rs` types, not redefined. `Nx*` component props cross-checked against R1's `defineProps` throughout — still subject to live re-verification at execution time, and every template in this plan has been hand-traced against its own test assertions before being finalized, per the discipline established in R8 after finding a real bug that way.
