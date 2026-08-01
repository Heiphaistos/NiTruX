# NiTruX Phase R7 — Maintenance avancée Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split `TroubleshootPage.vue`'s malware-scan and cache-cleaning content into their own dedicated pages, and add 3 new pages (Dépendances, Sauvegarde, Désinstalleur) — closing the "Maintenance" category gap identified against NiTriTe Windows's `navigation.ts` (which has 7 Maintenance pages; NiTruX currently has 3).

**Architecture:** Two pages (`AntivirusPage.vue`, `CleanerPage.vue`) are extracted from `TroubleshootPage.vue`'s existing tabs with ZERO backend change — `scan_for_malware`/`quarantine_file`/`run_troubleshoot_action` already exist and are reused verbatim, only their UI moves. `CleanerPage.vue` additionally gains a new, non-privileged cache-size preview (`src-tauri/src/cache_size.rs`, read-only filesystem walks, no root needed). `DependenciesPage.vue` (new `src-tauri/src/dependencies.rs`, `ldd`-based scan, read-only, no root) and `BackupPage.vue` (new `src-tauri/src/backup.rs`, `tar.gz` archive of a user-chosen directory to `$HOME`, no root) are both entirely non-privileged. `UninstallerPage.vue` is the one genuinely new privileged surface: extends the existing `PackageManager` trait with `list_installed()` (mirroring the already-proven `list_upgradable()` pattern across all 4 managers) and adds a new `uninstall-package` pkexec action — a near-exact mirror of the already-shipped, already-VM-verified `install-package` action, with its own dedicated `exec.path` per the project's established polkit-action-collision lesson, and its own live install→uninstall VM verification cycle before merge.

**Tech Stack:** Tauri v2 + Rust (backend), Vue 3.5 + TypeScript + Vitest (frontend), same patterns as R1-R6.

---

## Task 1: Extract `AntivirusPage.vue` from `TroubleshootPage.vue`

**Files:**
- Create: `src/pages/AntivirusPage.vue`
- Test: `src/pages/AntivirusPage.spec.ts`
- Modify: `src/pages/TroubleshootPage.vue` (remove the malware tab)
- Modify: `src/pages/TroubleshootPage.spec.ts` (remove the malware-tab tests, if any target it directly)

Read the live `src/pages/TroubleshootPage.vue` and its spec first — reproduced in this plan's research: it currently has 3 tabs (`malware`/`snapshots`/`troubleshoot`), with the malware tab using `scanDir`/`findings`/`scanError`/`scanning`/`scanDone`/`runScan`/`quarantining`/`quarantineError`/`quarantineFinding` — all of this moves to the new page verbatim, business logic unchanged.

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/AntivirusPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import AntivirusPage from "./AntivirusPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "scan_for_malware") {
      return Promise.resolve([{ path: "/tmp/evil.sh", signature: "Test.Signature" }]);
    }
    if (cmd === "quarantine_file") return Promise.resolve("mis en quarantaine");
    return Promise.resolve(null);
  }),
}));

describe("AntivirusPage", () => {
  it("scans a directory and lists findings", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(AntivirusPage);
    await wrapper.find("input").setValue("/tmp");
    const scanButton = wrapper.findAll("button").find((b) => b.text() === "Scanner")!;
    await scanButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/tmp/evil.sh"));
    expect(invoke).toHaveBeenCalledWith("scan_for_malware", { directory: "/tmp" });
  });

  it("quarantines a finding and removes it from the list", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(AntivirusPage);
    await wrapper.find("input").setValue("/tmp");
    await wrapper.findAll("button").find((b) => b.text() === "Scanner")!.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("/tmp/evil.sh"));
    const quarantineButton = wrapper.findAll("button").find((b) => b.text().includes("quarantaine"))!;
    await quarantineButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).not.toContain("/tmp/evil.sh"));
    expect(invoke).toHaveBeenCalledWith("quarantine_file", { path: "/tmp/evil.sh" });
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r7-maintenance-avancee && npx vitest run src/pages/AntivirusPage.spec.ts"`

- [ ] **Step 3: Write `AntivirusPage.vue`**

```vue
<!-- src/pages/AntivirusPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface MalwareFinding { path: string; signature: string }

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

const quarantining = ref<string | null>(null);
const quarantineError = ref<string | null>(null);

async function quarantineFinding(path: string) {
  quarantining.value = path;
  quarantineError.value = null;
  try {
    await invoke("quarantine_file", { path });
    findings.value = findings.value.filter((f) => f.path !== path);
  } catch (e) {
    quarantineError.value = String(e);
  } finally {
    quarantining.value = null;
  }
}
</script>

<template>
  <div class="av-page">
    <NxSectionHeader title="Antivirus" description="Analyse un dossier à la recherche de signatures de malware connues et met en quarantaine ce qui est trouvé." />

    <NxCard>
      <div class="av-form-row">
        <NxInput v-model="scanDir" placeholder="Dossier à scanner..." />
        <NxButton :disabled="scanning" @click="runScan">{{ scanning ? "Scan en cours..." : "Scanner" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-else-if="scanDone && findings.length === 0" class="av-empty">Aucune menace détectée.</div>
      <NxCard v-if="quarantineError" danger>{{ quarantineError }}</NxCard>
      <div v-for="f in findings" :key="f.path" class="av-finding-row">
        <span>{{ f.path }}</span>
        <span>{{ f.signature }}</span>
        <NxButton variant="danger" :disabled="quarantining !== null" @click="quarantineFinding(f.path)">
          {{ quarantining === f.path ? "Mise en quarantaine..." : "Mettre en quarantaine" }}
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.av-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.av-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.av-empty { color: var(--nx-text-secondary); }
.av-finding-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 4: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 5: Remove the malware tab from `TroubleshootPage.vue`**

Replace the entire file (removes the `malware` tab and all its associated refs/functions — `scanDir`, `findings`, `scanError`, `scanning`, `scanDone`, `runScan`, `quarantining`, `quarantineError`, `quarantineFinding`, the `MalwareFinding` interface, and the malware `<NxCard>` block; the `Tab` type loses `"malware"`; the default `activeTab` changes from `"malware"` to `"snapshots"`; everything else — snapshots and troubleshoot actions — is unchanged):

```vue
<!-- src/pages/TroubleshootPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface SnapshotInfo { id: string; date: string }

type Tab = "snapshots" | "troubleshoot";
const activeTab = ref<Tab>("troubleshoot");

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
  <div class="ts-page">
    <NxSectionHeader title="Dépannage" description="Instantanés système et actions de réparation." />

    <div class="ts-tabs">
      <button :class="{ active: activeTab === 'snapshots' }" @click="onTabClick('snapshots')">Snapshots</button>
      <button :class="{ active: activeTab === 'troubleshoot' }" @click="onTabClick('troubleshoot')">Dépannage</button>
    </div>

    <NxCard v-if="activeTab === 'snapshots'">
      <div class="ts-form-row">
        <NxButton :disabled="snapshotCreating" @click="createSnapshotNow">{{ snapshotCreating ? "Création..." : "Créer un instantané" }}</NxButton>
      </div>
      <NxCard v-if="snapshotCreateError" danger>{{ snapshotCreateError }}</NxCard>
      <NxCard v-if="snapshotsError" danger>{{ snapshotsError }}</NxCard>
      <div v-for="s in snapshots" :key="s.id" class="ts-row">
        <span>#{{ s.id }}</span>
        <span>{{ s.date }}</span>
      </div>
    </NxCard>

    <NxCard v-else>
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
.ts-tabs { display: flex; gap: 8px; }
.ts-tabs button { padding: 8px 14px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; }
.ts-tabs button.active { color: var(--nx-text-primary); font-weight: 600; }
.ts-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.ts-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.ts-action-label { flex: 1; }
</style>
```

- [ ] **Step 6: Replace `src/pages/TroubleshootPage.spec.ts` entirely** (its live content, captured during this plan's research, has 3 tests: one asserting the 3 old tab labels including "Scan malware", one for snapshots lazy-loading, one that clicks the "Dépannage" tab and runs `clean-cache` via the first "Exécuter" button — all 3 need updating for the new 2-tab shape with only `fix-broken`/`restart-network` remaining):

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

  it("shows the snapshots/troubleshoot tabs, no malware or firewall tab, defaults to troubleshoot", () => {
    const wrapper = mount(TroubleshootPage);
    expect(wrapper.text()).toContain("Snapshots");
    expect(wrapper.text()).toContain("Dépannage");
    expect(wrapper.text()).not.toContain("Scan malware");
    expect(wrapper.text()).not.toContain("Pare-feu");
  });

  it("lazy-loads snapshots only the first time the snapshots tab is opened", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const tabs = wrapper.findAll("button");
    const snapshotsTab = tabs.find((b) => b.text() === "Snapshots")!;
    expect(invoke).not.toHaveBeenCalledWith("list_snapshots");
    await snapshotsTab.trigger("click");
    expect(invoke).toHaveBeenCalledWith("list_snapshots");
  });

  it("runs the fix-broken troubleshoot action via run_troubleshoot_action (troubleshoot is now the default tab)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const buttons = wrapper.findAll("button");
    const execButtons = buttons.filter((b) => b.text() === "Exécuter");
    expect(execButtons.length).toBe(2); // fix-broken, restart-network
    await execButtons[0].trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "fix-broken" });
  });
});
```

This preserves the original component's lazy-load behavior exactly (snapshots only fetch on an explicit tab click, same as before the split) — only the default landing tab changed from the now-removed `"malware"` to `"troubleshoot"`, since `"snapshots"` staying lazy means it should not be the tab shown on first render either (defaulting to `"snapshots"` would make the empty-snapshots-until-clicked state the very first thing a user sees, worse than showing the always-ready troubleshoot actions by default).

- [ ] **Step 7: Run both spec files, confirm both pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r7-maintenance-avancee && npx vitest run src/pages/AntivirusPage.spec.ts src/pages/TroubleshootPage.spec.ts"`

- [ ] **Step 8: Commit**

```bash
git add src/pages/AntivirusPage.vue src/pages/AntivirusPage.spec.ts src/pages/TroubleshootPage.vue src/pages/TroubleshootPage.spec.ts
git commit -m "feat: extract AntivirusPage from TroubleshootPage (spec section 2)"
```

---

## Task 2: Extract `CleanerPage.vue` + non-privileged cache-size preview

**Files:**
- Create: `src-tauri/src/cache_size.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/CleanerPage.vue`
- Test: `src/pages/CleanerPage.spec.ts`
- Modify: `src/pages/TroubleshootPage.vue` (remove `clean-cache`/`vacuum-logs` from `TROUBLESHOOT_ACTIONS` — already done as part of Task 1 Step 5's replacement; this task only adds the new page)

### Backend: read-only cache size reporting

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/cache_size.rs
use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct CacheSizeReport {
    pub user_cache_bytes: u64,
    pub package_cache_bytes: Option<u64>,
}

/// Recursively sums file sizes under `path`. Missing directories, and any
/// individual file/subdirectory that can't be read (permission denied,
/// broken symlink, race with concurrent deletion), are silently skipped
/// rather than failing the whole walk -- a best-effort size estimate is
/// more useful here than an all-or-nothing error, since this is purely
/// informational (no cleanup decision is made automatically from it).
pub fn directory_size_bytes(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else { continue };
        if metadata.is_dir() {
            total += directory_size_bytes(&entry.path());
        } else {
            total += metadata.len();
        }
    }
    total
}

fn package_cache_dir() -> Option<&'static str> {
    if Path::new("/var/cache/apt/archives").is_dir() {
        Some("/var/cache/apt/archives")
    } else if Path::new("/var/cache/dnf").is_dir() {
        Some("/var/cache/dnf")
    } else if Path::new("/var/cache/pacman/pkg").is_dir() {
        Some("/var/cache/pacman/pkg")
    } else if Path::new("/var/cache/zypp").is_dir() {
        Some("/var/cache/zypp")
    } else {
        None
    }
}

#[tauri::command]
pub fn get_cache_size_report() -> CacheSizeReport {
    let home = std::env::var("HOME").unwrap_or_default();
    let user_cache_bytes = directory_size_bytes(Path::new(&home).join(".cache").as_path());
    // Package manager cache dirs are typically root-owned and NOT readable
    // by an unprivileged user (e.g. /var/cache/apt/archives is root:root
    // 0700 on Debian) -- `directory_size_bytes`'s permission-denied handling
    // means this naturally reports 0 rather than erroring in that case, so
    // the frontend distinguishes "no known cache dir for this distro"
    // (`None`) from "cache dir exists but unreadable, reports as ~0 bytes"
    // (`Some(0)`) only loosely -- both are honestly represented as "0 o"
    // to the user rather than a confusing permission error for a read-only
    // informational number.
    let package_cache_bytes = package_cache_dir().map(|dir| directory_size_bytes(Path::new(dir)));

    CacheSizeReport { user_cache_bytes, package_cache_bytes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sums_file_sizes_in_a_flat_directory() {
        let dir = std::env::temp_dir().join(format!("nitrux-test-cache-size-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), b"12345").unwrap();
        fs::write(dir.join("b.txt"), b"1234567890").unwrap();
        let total = directory_size_bytes(&dir);
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(total, 15);
    }

    #[test]
    fn recurses_into_subdirectories() {
        let dir = std::env::temp_dir().join(format!("nitrux-test-cache-size-nested-{}", std::process::id()));
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub").join("c.txt"), b"abc").unwrap();
        let total = directory_size_bytes(&dir);
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(total, 3);
    }

    #[test]
    fn returns_zero_for_a_nonexistent_directory_rather_than_erroring() {
        let dir = std::env::temp_dir().join("nitrux-test-cache-size-does-not-exist");
        assert_eq!(directory_size_bytes(&dir), 0);
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (module not registered)

- [ ] **Step 3: Register `mod cache_size;` in `lib.rs`** (alphabetically, between `mod benchmark;` and `mod disk_write;`) and add `cache_size::get_cache_size_report,` to `tauri::generate_handler![...]` (near `report::generate_system_report,`/`optimizations::get_optimization_snapshot,`).

- [ ] **Step 4: Run the full Rust suite, expect `145 passed; 0 failed; 1 ignored`** (142 baseline + 3 new).

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r7-maintenance-avancee/src-tauri && cargo test 2>&1 | tail -15"`

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/cache_size.rs src-tauri/src/lib.rs
git commit -m "feat: add cache_size.rs — read-only cache size reporting (spec section 2)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/CleanerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import CleanerPage from "./CleanerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "get_cache_size_report") {
      return Promise.resolve({ user_cache_bytes: 52_428_800, package_cache_bytes: 104_857_600 });
    }
    if (cmd === "run_troubleshoot_action" && args?.action === "clean-cache") {
      return Promise.resolve("cache vidé");
    }
    return Promise.resolve(null);
  }),
}));

describe("CleanerPage", () => {
  it("loads and displays cache sizes on mount", async () => {
    const wrapper = mount(CleanerPage);
    await vi.waitFor(() => expect(wrapper.text()).toMatch(/50[.,]0 Mo/));
    expect(wrapper.text()).toMatch(/100[.,]0 Mo/);
  });

  it("runs clean-cache when the button is clicked", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(CleanerPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Vider le cache"))!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "clean-cache" }));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `CleanerPage.vue`**

```vue
<!-- src/pages/CleanerPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface CacheSizeReport { user_cache_bytes: number; package_cache_bytes: number | null }

function formatMb(bytes: number): string {
  return `${(bytes / 1_048_576).toFixed(1)} Mo`;
}

const report = ref<CacheSizeReport | null>(null);
const reportError = ref<string | null>(null);

async function loadReport() {
  try {
    report.value = await invoke<CacheSizeReport>("get_cache_size_report");
  } catch (e) {
    reportError.value = String(e);
  }
}

onMounted(loadReport);

const busy = ref<string | null>(null);
const result = ref<string | null>(null);
const actionError = ref<string | null>(null);

async function runAction(action: "clean-cache" | "vacuum-logs") {
  busy.value = action;
  actionError.value = null;
  result.value = null;
  try {
    result.value = await invoke<string>("run_troubleshoot_action", { action });
    if (action === "clean-cache") await loadReport();
  } catch (e) {
    actionError.value = String(e);
  } finally {
    busy.value = null;
  }
}
</script>

<template>
  <div class="cln-page">
    <NxSectionHeader title="Nettoyeur" description="Aperçu de l'espace utilisé par les caches et purge des fichiers temporaires." />

    <NxCard v-if="reportError" danger>{{ reportError }}</NxCard>

    <div class="cln-stats" v-if="report">
      <NxCard><NxStatTile label="Cache utilisateur (~/.cache)" :value="formatMb(report.user_cache_bytes)" /></NxCard>
      <NxCard>
        <NxStatTile label="Cache du gestionnaire de paquets" :value="report.package_cache_bytes !== null ? formatMb(report.package_cache_bytes) : 'inconnu'" />
      </NxCard>
    </div>

    <NxCard>
      <NxCard v-if="actionError" danger>{{ actionError }}</NxCard>
      <NxBadge v-if="result" status="success">{{ result }}</NxBadge>
      <div class="cln-action-row">
        <span class="cln-action-label">Vider le cache des paquets</span>
        <NxButton :disabled="busy !== null" @click="runAction('clean-cache')">{{ busy === "clean-cache" ? "En cours..." : "Exécuter" }}</NxButton>
      </div>
      <div class="cln-action-row">
        <span class="cln-action-label">Purger les anciens journaux</span>
        <NxButton :disabled="busy !== null" @click="runAction('vacuum-logs')">{{ busy === "vacuum-logs" ? "En cours..." : "Exécuter" }}</NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.cln-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.cln-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(220px, 1fr)); gap: 12px; }
.cln-action-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.cln-action-label { flex: 1; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit the frontend**

```bash
git add src/pages/CleanerPage.vue src/pages/CleanerPage.spec.ts
git commit -m "feat: add CleanerPage — cache size preview + clean-cache/vacuum-logs (spec section 2)"
```

---

## Task 3: `dependencies.rs` (read-only ldd scan) + `DependenciesPage.vue`

**Files:**
- Create: `src-tauri/src/dependencies.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/DependenciesPage.vue`
- Test: `src/pages/DependenciesPage.spec.ts`

Real `ldd` output for a binary with a missing shared library looks like:
```
	linux-vdso.so.1 (0x00007ffd...)
	libc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f...)
	libfoo.so.3 => not found
	/lib64/ld-linux-x86-64.so.2 (0x00007f...)
```
Lines with `=> not found` indicate a missing dependency; every other line (found libraries, `linux-vdso.so.1`, the dynamic linker itself) is not a problem.

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/dependencies.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct MissingDependency {
    pub binary: String,
    pub missing_library: String,
}

/// A small, fixed set of common system binaries -- scanning every
/// executable on the system would be slow and mostly redundant (most
/// binaries share the same handful of core libraries); this list mirrors
/// the kind of "did something break after an update" spot-check NiTriTe's
/// own Dépendances page performs, not an exhaustive audit.
const BINARIES_TO_CHECK: &[&str] = &[
    "/bin/bash", "/bin/ls", "/usr/bin/apt", "/usr/bin/systemctl",
    "/usr/bin/python3", "/usr/bin/curl", "/usr/bin/git",
];

/// Parses one line of `ldd <binary>` output, returning the missing library
/// name if this line reports one (`=> not found`), or `None` for a
/// resolved dependency / the dynamic linker line / a malformed line.
pub fn parse_ldd_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.ends_with("=> not found") {
        return None;
    }
    trimmed.split_whitespace().next().map(|s| s.to_string())
}

fn check_binary(binary: &str) -> Vec<MissingDependency> {
    if !std::path::Path::new(binary).exists() {
        return Vec::new();
    }
    let Ok(output) = subprocess::run_with_timeout("ldd", &[binary], Duration::from_secs(5)) else {
        return Vec::new();
    };
    output
        .lines()
        .filter_map(parse_ldd_line)
        .map(|missing_library| MissingDependency { binary: binary.to_string(), missing_library })
        .collect()
}

#[tauri::command]
pub fn scan_missing_dependencies() -> Vec<MissingDependency> {
    BINARIES_TO_CHECK.iter().flat_map(|b| check_binary(b)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_missing_dependency_line() {
        let line = "\tlibfoo.so.3 => not found";
        assert_eq!(parse_ldd_line(line), Some("libfoo.so.3".to_string()));
    }

    #[test]
    fn ignores_a_resolved_dependency_line() {
        let line = "\tlibc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f0000000000)";
        assert_eq!(parse_ldd_line(line), None);
    }

    #[test]
    fn ignores_the_dynamic_linker_line() {
        let line = "\t/lib64/ld-linux-x86-64.so.2 (0x00007f0000000000)";
        assert_eq!(parse_ldd_line(line), None);
    }

    #[test]
    fn ignores_the_vdso_line() {
        let line = "\tlinux-vdso.so.1 (0x00007ffd00000000)";
        assert_eq!(parse_ldd_line(line), None);
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (module not registered).

- [ ] **Step 3: Register `mod dependencies;` in `lib.rs`** (alphabetically, between `mod cache_size;` and `mod disk_write;`, since "dependencies" sorts after "cache_size" and before "disk_write") and add `dependencies::scan_missing_dependencies,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `149 passed; 0 failed; 1 ignored`** (145 baseline + 4 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/dependencies.rs src-tauri/src/lib.rs
git commit -m "feat: add dependencies.rs — read-only ldd-based missing shared library scan (spec section 3.1)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/DependenciesPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DependenciesPage from "./DependenciesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([{ binary: "/usr/bin/git", missing_library: "libfoo.so.3" }]),
}));

describe("DependenciesPage", () => {
  it("lists missing dependencies found on mount", async () => {
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("/usr/bin/git"));
    expect(wrapper.text()).toContain("libfoo.so.3");
  });

  it("shows a clean-system message when nothing is missing", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce([]);
    const wrapper = mount(DependenciesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune dépendance manquante"));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `DependenciesPage.vue`**

```vue
<!-- src/pages/DependenciesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxBadge from "@/components/ui/NxBadge.vue";

interface MissingDependency { binary: string; missing_library: string }

const results = ref<MissingDependency[] | null>(null);

onMounted(async () => {
  results.value = await invoke<MissingDependency[]>("scan_missing_dependencies");
});
</script>

<template>
  <div class="dep-page">
    <NxSectionHeader title="Dépendances" description="Vérifie qu'un ensemble de binaires système courants ont toutes leurs bibliothèques partagées résolues." />

    <div v-if="results && results.length === 0" class="dep-empty">Aucune dépendance manquante détectée.</div>

    <NxCard v-else-if="results">
      <div v-for="(r, i) in results" :key="`${r.binary}-${r.missing_library}-${i}`" class="dep-row">
        <span>{{ r.binary }}</span>
        <NxBadge status="danger">{{ r.missing_library }}</NxBadge>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.dep-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.dep-empty { color: var(--nx-text-secondary); }
.dep-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit the frontend**

```bash
git add src/pages/DependenciesPage.vue src/pages/DependenciesPage.spec.ts
git commit -m "feat: add DependenciesPage (spec section 3.1)"
```

---

## Task 4: `backup.rs` (non-privileged tar.gz archive) + `BackupPage.vue`

**Files:**
- Create: `src-tauri/src/backup.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/BackupPage.vue`
- Test: `src/pages/BackupPage.spec.ts`

No new Cargo dependency — shells out to the system's own `tar` binary (already a baseline assumption on every target distro, same posture as every other `subprocess::run_with_timeout` caller in this codebase), rather than adding a Rust archive crate.

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/backup.rs
use crate::subprocess;
use std::time::Duration;

/// Rejects any source path that is not absolute, or that attempts to
/// escape via `..` -- this runs unprivileged (no pkexec), but a
/// non-absolute or traversal-laden path is still worth rejecting outright
/// as an obvious misuse rather than letting `tar` interpret it.
pub fn validate_source_dir(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("chemin source vide".to_string());
    }
    if !path.starts_with('/') {
        return Err(format!("le chemin source doit être absolu : {path}"));
    }
    if path.contains("..") {
        return Err(format!("le chemin source ne doit pas contenir '..' : {path}"));
    }
    Ok(())
}

pub fn backup_filename(now_epoch_secs: u64) -> String {
    format!("nitrux-backup-{now_epoch_secs}.tar.gz")
}

#[tauri::command]
pub fn create_backup(source_dir: String) -> Result<String, String> {
    validate_source_dir(&source_dir)?;
    let home = std::env::var("HOME").map_err(|_| "variable HOME introuvable".to_string())?;
    let now_epoch_secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let dest_path = format!("{home}/{}", backup_filename(now_epoch_secs));

    subprocess::run_with_timeout("tar", &["-czf", &dest_path, "-C", &source_dir, "."], Duration::from_secs(300))?;

    Ok(dest_path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_well_formed_absolute_source_path() {
        assert!(validate_source_dir("/home/dev/documents").is_ok());
    }

    #[test]
    fn rejects_empty_source_path() {
        assert!(validate_source_dir("").is_err());
    }

    #[test]
    fn rejects_relative_source_path() {
        assert!(validate_source_dir("documents").is_err());
    }

    #[test]
    fn rejects_path_traversal() {
        assert!(validate_source_dir("/home/dev/../../etc").is_err());
    }

    #[test]
    fn backup_filename_includes_the_epoch_timestamp() {
        assert_eq!(backup_filename(1735689600), "nitrux-backup-1735689600.tar.gz");
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile.**

- [ ] **Step 3: Register `mod backup;` in `lib.rs`** (alphabetically, first `mod` line, before `mod benchmark;`) and add `backup::create_backup,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `154 passed; 0 failed; 1 ignored`** (149 baseline + 5 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/backup.rs src-tauri/src/lib.rs
git commit -m "feat: add backup.rs — non-privileged tar.gz archive to \$HOME (spec section 3.2)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/BackupPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BackupPage from "./BackupPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "create_backup") return Promise.resolve(`/home/dev/nitrux-backup-1735689600.tar.gz`);
    return Promise.resolve(null);
  }),
}));

describe("BackupPage", () => {
  it("creates a backup and shows the resulting path", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(BackupPage);
    await wrapper.find("input").setValue("/home/dev");
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer la sauvegarde")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux-backup-1735689600.tar.gz"));
    expect(invoke).toHaveBeenCalledWith("create_backup", { sourceDir: "/home/dev" });
  });

  it("shows an error message when backup creation fails", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce("échec de la sauvegarde");
    const wrapper = mount(BackupPage);
    await wrapper.find("input").setValue("/home/dev");
    const button = wrapper.findAll("button").find((b) => b.text() === "Créer la sauvegarde")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("échec de la sauvegarde"));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `BackupPage.vue`**

```vue
<!-- src/pages/BackupPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const sourceDir = ref("");
const creating = ref(false);
const error = ref<string | null>(null);
const resultPath = ref<string | null>(null);

async function createBackup() {
  creating.value = true;
  error.value = null;
  resultPath.value = null;
  try {
    resultPath.value = await invoke<string>("create_backup", { sourceDir: sourceDir.value });
  } catch (e) {
    error.value = String(e);
  } finally {
    creating.value = false;
  }
}
</script>

<template>
  <div class="bkp-page">
    <NxSectionHeader title="Sauvegarde" description="Archive un dossier vers un fichier .tar.gz horodaté dans votre dossier personnel." />

    <NxCard>
      <div class="bkp-form-row">
        <NxInput v-model="sourceDir" placeholder="Dossier à sauvegarder (ex: /home/dev/documents)" />
        <NxButton :disabled="creating || sourceDir === ''" @click="createBackup">{{ creating ? "Sauvegarde en cours..." : "Créer la sauvegarde" }}</NxButton>
      </div>
      <NxCard v-if="error" danger>{{ error }}</NxCard>
      <NxBadge v-if="resultPath" status="success">Sauvegarde créée : {{ resultPath }}</NxBadge>
    </NxCard>
  </div>
</template>

<style scoped>
.bkp-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.bkp-form-row { display: flex; gap: 10px; align-items: center; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit the frontend**

```bash
git add src/pages/BackupPage.vue src/pages/BackupPage.spec.ts
git commit -m "feat: add BackupPage (spec section 3.2)"
```

---

## Task 5: Backend — `list_installed()` across all 4 `PackageManager` implementations

**Files:**
- Modify: `src-tauri/src/packages/mod.rs` (extend the trait + add `InstalledPackage` + `list_installed_packages` command)
- Modify: `src-tauri/src/packages/apt.rs`
- Modify: `src-tauri/src/packages/dnf.rs`
- Modify: `src-tauri/src/packages/pacman.rs`
- Modify: `src-tauri/src/packages/zypper.rs`
- Modify: `src-tauri/src/lib.rs`

Real `dpkg -l` output (captured on this project's own dev environment during this plan's research):
```
ii  adduser                                3.118ubuntu5                            all          add and remove users and groups
ii  adwaita-icon-theme                     41.0-1ubuntu1                           all          default icon theme of GNOME (small subset)
```
Only lines whose first field is exactly `ii` (installed, not `rc`/removed-config-remains or other transitional states) represent a currently-installed package.

- [ ] **Step 1: Write the failing tests for all 4 managers plus the trait/aggregation**

Add to `src-tauri/src/packages/apt.rs`'s existing `mod tests` block:
```rust
    #[test]
    fn parses_dpkg_l_installed_line() {
        let line = "ii  adduser                                3.118ubuntu5                            all          add and remove users and groups";
        let pkg = parse_dpkg_l_line(line).expect("should parse");
        assert_eq!(pkg.name, "adduser");
        assert_eq!(pkg.version, "3.118ubuntu5");
    }

    #[test]
    fn skips_non_installed_dpkg_status_lines() {
        assert!(parse_dpkg_l_line("rc  old-package  1.0  all  description").is_none());
    }

    #[test]
    fn skips_dpkg_l_header_lines() {
        assert!(parse_dpkg_l_line("Desired=Unknown/Install/Remove/Purge/Hold").is_none());
        assert!(parse_dpkg_l_line("+++-======================================").is_none());
    }
```

Above that block, add to the file (not inside `mod tests`):
```rust
/// Parses one line of `dpkg -l` output. Only lines whose status field is
/// exactly "ii" (installed) count -- other statuses ("rc" = removed but
/// config files remain, "un" = unknown, etc.) are not currently-installed
/// packages and are skipped, same as header/separator lines that don't
/// start with a 2-letter status code at all.
pub fn parse_dpkg_l_line(line: &str) -> Option<super::InstalledPackage> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 3 || fields[0] != "ii" {
        return None;
    }
    Some(super::InstalledPackage { name: fields[1].to_string(), version: fields[2].to_string() })
}
```

Add to `impl PackageManager for Apt`:
```rust
    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("dpkg", &["-l"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_dpkg_l_line).collect())
    }
```

Repeat the same pattern for the other 3 managers, using each one's real listing command and format:

`src-tauri/src/packages/dnf.rs` — `rpm -qa --queryformat '%{NAME} %{VERSION}-%{RELEASE}\n'` (avoids parsing rpm's default human-readable format, asks for exactly the 2 fields needed, space-separated):
```rust
/// Parses one line of `rpm -qa --queryformat '%{NAME} %{VERSION}-%{RELEASE}\n'`
/// output, e.g. "curl 7.76.1-14.fc35" -- deliberately requesting this exact
/// format from rpm rather than parsing its default human-readable listing,
/// which has no fixed field separator.
pub fn parse_rpm_qa_line(line: &str) -> Option<super::InstalledPackage> {
    let mut parts = line.split_whitespace();
    let name = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    if name.is_empty() || version.is_empty() {
        return None;
    }
    Some(super::InstalledPackage { name, version })
}
```
Add tests: `parses_rpm_qa_line` (`"curl 7.76.1-14.fc35"` → name `curl`, version `7.76.1-14.fc35`), `skips_blank_lines` (empty string → `None`).
Add to `impl PackageManager for Dnf`:
```rust
    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("rpm", &["-qa", "--queryformat", "%{NAME} %{VERSION}-%{RELEASE}\n"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_rpm_qa_line).collect())
    }
```

`src-tauri/src/packages/pacman.rs` — `pacman -Q` (already space-separated `name version`, e.g. `"curl 8.4.0-1"`):
```rust
/// Parses one line of `pacman -Q` output, e.g. "curl 8.4.0-1".
pub fn parse_pacman_q_line(line: &str) -> Option<super::InstalledPackage> {
    let mut parts = line.split_whitespace();
    let name = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    Some(super::InstalledPackage { name, version })
}
```
Add tests: `parses_pacman_q_line`, `skips_blank_lines`.
Add to `impl PackageManager for Pacman`:
```rust
    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("pacman", &["-Q"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_pacman_q_line).collect())
    }
```

`src-tauri/src/packages/zypper.rs` — `zypper se --installed-only` (pipe-separated, same shape as `list-updates` but a different status column value `i` for installed):
```rust
/// Parses one line of `zypper se --installed-only` output, e.g.:
/// "i | curl | package | x86_64 | Main Repository"
/// (Status | Name | Type | Arch | Repository -- note this has 5 fields,
/// one fewer than `list-updates`'s 6, and no version column at all; zypper
/// requires a separate `zypper info <pkg>` call per package for version,
/// which this scan deliberately skips for performance -- version is left
/// empty for zypper-sourced installed packages, a real, documented
/// limitation, not a parsing gap, mirroring dnf.rs's existing precedent
/// for `list_upgradable`'s own current_version limitation.)
pub fn parse_zypper_installed_line(line: &str) -> Option<super::InstalledPackage> {
    let fields: Vec<&str> = line.split('|').map(|f| f.trim()).collect();
    if fields.len() != 5 || fields[0] != "i" {
        return None;
    }
    Some(super::InstalledPackage { name: fields[1].to_string(), version: String::new() })
}
```
Add tests: `parses_zypper_installed_line`, `skips_header_and_separator_lines`.
Add to `impl PackageManager for Zypper`:
```rust
    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("zypper", &["se", "--installed-only"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_zypper_installed_line).collect())
    }
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (`InstalledPackage` and the trait method don't exist yet).

- [ ] **Step 3: Add `InstalledPackage`, extend the `PackageManager` trait, and add the aggregating command in `packages/mod.rs`**

```rust
#[derive(Serialize, Clone)]
pub struct InstalledPackage {
    pub name: String,
    pub version: String,
}
```

Change the trait:
```rust
pub trait PackageManager {
    fn id(&self) -> &'static str;
    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String>;
    fn list_installed(&self) -> Result<Vec<InstalledPackage>, String>;
}
```

Add below `detect_package_managers`:
```rust
/// Aggregates installed packages from the FIRST detected native manager
/// only (mirrors `detect_native_manager`'s own "just the first one" choice
/// from Phase R3 -- a host with multiple native managers installed is rare
/// and install/uninstall already only ever targets one at a time via that
/// same detected id).
pub fn list_installed_for_detected_manager() -> Result<Vec<InstalledPackage>, String> {
    let managers = detect_package_managers();
    let Some(manager) = managers.first() else {
        return Err("aucun gestionnaire de paquets détecté".to_string());
    };
    manager.list_installed()
}

#[tauri::command]
pub fn list_installed_packages() -> Result<Vec<InstalledPackage>, String> {
    list_installed_for_detected_manager()
}
```

- [ ] **Step 4: Run the full Rust suite, expect `162 passed; 0 failed; 1 ignored`** (154 baseline + 8 new: 2 apt + 2 dnf + 2 pacman + 2 zypper).

- [ ] **Step 5: Register `packages::list_installed_packages,` in `lib.rs`'s handler list** (near `list_updates,`/`detect_native_manager,`).

- [ ] **Step 6: Run the full Rust suite again to confirm the handler registration didn't break anything, same count.**

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/packages/mod.rs src-tauri/src/packages/apt.rs src-tauri/src/packages/dnf.rs src-tauri/src/packages/pacman.rs src-tauri/src/packages/zypper.rs src-tauri/src/lib.rs
git commit -m "feat: add list_installed() across all 4 PackageManager implementations (spec section 3.3)"
```

---

## Task 6: Backend — `uninstall-package` pkexec action

**Files:**
- Modify: `src-tauri/packaging/nitrux-pkexec-helper`
- Modify: `src-tauri/packaging/org.heiphaistos.nitrux.packages.policy`
- Modify: `src-tauri/tauri.conf.json`
- Modify: `src-tauri/src/packages/install.rs` (add `uninstall_package` command, reusing existing `validate_manager_id`/`validate_package_name`)
- Modify: `src-tauri/src/lib.rs`

This is the one genuinely new privileged surface in R7. Follow the exact same discipline as every prior pkexec action in this project: dedicated `exec.path` (never shared with `install-package`), Rust-side validation AND independent shell-side re-validation, live VM verification before merge.

- [ ] **Step 1: Write the failing Rust test**

Add to `src-tauri/src/packages/install.rs`'s existing `mod tests` block (the file already has `install_package`/`upgrade_all_packages` and their validation tests — `validate_manager_id`/`validate_package_name` are reused verbatim, no new validation function is needed):

```rust
    #[test]
    fn uninstall_package_rejects_unknown_manager_before_ever_shelling_out() {
        let result = uninstall_package("bogus-manager".to_string(), "curl".to_string());
        assert!(result.is_err());
    }

    #[test]
    fn uninstall_package_rejects_malicious_package_name_before_ever_shelling_out() {
        let result = uninstall_package("apt".to_string(), "curl; rm -rf /".to_string());
        assert!(result.is_err());
    }
```

- [ ] **Step 2: Run it, confirm it fails to compile** (`uninstall_package` doesn't exist).

- [ ] **Step 3: Add `uninstall_package` to `src-tauri/src/packages/install.rs`**, right after the existing `install_package` function:

```rust
/// Uninstalls `package` via `manager`, escalating through polkit. Mirrors
/// `install_package` exactly (same validation, same generous timeout for a
/// potentially dependency-heavy operation) -- see that function's doc
/// comment for why this uses its own dedicated `nitrux-pkexec-uninstall-package`
/// exec path rather than sharing `install-package`'s.
#[tauri::command]
pub fn uninstall_package(manager: String, package: String) -> Result<String, String> {
    validate_manager_id(&manager)?;
    validate_package_name(&package)?;
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-uninstall-package", "uninstall-package", &manager, &package],
        Duration::from_secs(300),
    )
}
```

- [ ] **Step 4: Run the full Rust suite, expect `164 passed; 0 failed; 1 ignored`** (162 baseline + 2 new).

- [ ] **Step 5: Add the `uninstall-package` subcommand to `nitrux-pkexec-helper`**

Read the live file first. Add a new case arm to the main `case "$cmd" in ... esac` dispatch, immediately after the existing `install-package)` arm:

```sh
  uninstall-package)
    manager="${2:-}"
    package="${3:-}"
    validate_manager "$manager"
    validate_package_name "$package"
    case "$manager" in
      apt)    exec apt-get remove -y "$package" ;;
      dnf)    exec dnf remove -y "$package" ;;
      pacman) exec pacman -R --noconfirm "$package" ;;
      zypper) exec zypper --non-interactive remove "$package" ;;
    esac
    ;;
```

Also update the file's top-level usage comment block (the one listing every `nitrux-pkexec-<name> <subcommand> <args>` line) to add:
```
#   nitrux-pkexec-uninstall-package uninstall-package <manager> <package>
```
And update the doc comment's list of "11 distinct names" to "12 distinct names", adding `nitrux-pkexec-uninstall-package` to that enumerated list.

- [ ] **Step 6: Add the new action to `org.heiphaistos.nitrux.packages.policy`**

Read the live file first (it currently has 2 `<action>` entries: `install-package`, `upgrade-all`). Add a third, right after `install-package`'s closing `</action>`:

```xml
  <action id="org.heiphaistos.nitrux.uninstall-package">
    <description>Désinstaller un paquet système</description>
    <message>NiTruX veut désinstaller un paquet système</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-uninstall-package</annotate>
  </action>
```

- [ ] **Step 7: Register the new binary mapping in `tauri.conf.json`**

Read the live file first. In BOTH the `linux.deb.files` and `linux.rpm.files` objects, add a new entry right after the existing `"/usr/bin/nitrux-pkexec-install-package": "packaging/nitrux-pkexec-helper",` line:

```json
          "/usr/bin/nitrux-pkexec-uninstall-package": "packaging/nitrux-pkexec-helper",
```

(This must be added in BOTH the `deb` and `rpm` blocks — they currently have identical file mappings, keep them identical.)

- [ ] **Step 8: Register `packages::install::uninstall_package,` in `lib.rs`'s handler list**, right after `packages::install::install_package,`.

- [ ] **Step 9: Run the full Rust suite one more time to confirm nothing broke, same 164 count.**

- [ ] **Step 10: Commit**

```bash
git add src-tauri/src/packages/install.rs src-tauri/packaging/nitrux-pkexec-helper src-tauri/packaging/org.heiphaistos.nitrux.packages.policy src-tauri/tauri.conf.json src-tauri/src/lib.rs
git commit -m "feat: add uninstall-package pkexec action, mirroring install-package (spec section 3.3)"
```

---

## Task 7: `UninstallerPage.vue`

**Files:**
- Create: `src/pages/UninstallerPage.vue`
- Test: `src/pages/UninstallerPage.spec.ts`

Same typed-confirmation-gate UX pattern already used for `format_partition` in `DisksPage.vue`: the uninstall button stays disabled until the user retypes the exact package name.

- [ ] **Step 1: Write the failing tests**

```typescript
// src/pages/UninstallerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import UninstallerPage from "./UninstallerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_installed_packages") {
      return Promise.resolve([
        { name: "curl", version: "7.88.1" },
        { name: "vim", version: "9.0" },
      ]);
    }
    if (cmd === "detect_native_manager") return Promise.resolve("apt");
    if (cmd === "uninstall_package") return Promise.resolve("désinstallation réussie");
    return Promise.resolve(null);
  }),
}));

describe("UninstallerPage", () => {
  it("lists installed packages on mount and filters by search text", async () => {
    const wrapper = mount(UninstallerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    expect(wrapper.text()).toContain("vim");
    await wrapper.find("input[placeholder*='Rechercher']").setValue("curl");
    expect(wrapper.text()).toContain("curl");
    expect(wrapper.text()).not.toContain("vim");
  });

  it("keeps the uninstall button disabled until the exact package name is retyped, then calls uninstall_package", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(UninstallerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    const uninstallButtons = wrapper.findAll("button").filter((b) => b.text() === "Désinstaller");
    await uninstallButtons[0].trigger("click"); // opens the confirmation row for "curl"
    const confirmButton = wrapper.findAll("button").find((b) => b.text() === "Confirmer la désinstallation")!;
    expect(confirmButton.attributes("disabled")).toBeDefined();
    const confirmInput = wrapper.find("input[placeholder*='curl']");
    await confirmInput.setValue("curl");
    expect(confirmButton.attributes("disabled")).toBeUndefined();
    await confirmButton.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("uninstall_package", { manager: "apt", package: "curl" }));
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

- [ ] **Step 3: Write `UninstallerPage.vue`**

```vue
<!-- src/pages/UninstallerPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface InstalledPackage { name: string; version: string }

const packages = ref<InstalledPackage[]>([]);
const loadError = ref<string | null>(null);
const nativeManager = ref<string | null>(null);
const searchText = ref("");

onMounted(async () => {
  try {
    packages.value = await invoke<InstalledPackage[]>("list_installed_packages");
  } catch (e) {
    loadError.value = String(e);
  }
  nativeManager.value = await invoke<string | null>("detect_native_manager");
});

const filteredPackages = computed(() =>
  searchText.value === ""
    ? packages.value
    : packages.value.filter((p) => p.name.toLowerCase().includes(searchText.value.toLowerCase())),
);

const confirmingPackage = ref<string | null>(null);
const confirmText = ref("");
const uninstalling = ref<string | null>(null);
const uninstallError = ref<string | null>(null);
const uninstallResult = ref<string | null>(null);

function startConfirm(name: string) {
  confirmingPackage.value = name;
  confirmText.value = "";
  uninstallError.value = null;
  uninstallResult.value = null;
}

async function confirmUninstall(name: string) {
  if (!nativeManager.value) return;
  uninstalling.value = name;
  uninstallError.value = null;
  try {
    uninstallResult.value = await invoke<string>("uninstall_package", { manager: nativeManager.value, package: name });
    packages.value = packages.value.filter((p) => p.name !== name);
    confirmingPackage.value = null;
  } catch (e) {
    uninstallError.value = String(e);
  } finally {
    uninstalling.value = null;
  }
}
</script>

<template>
  <div class="unin-page">
    <NxSectionHeader title="Désinstalleur" :description="nativeManager ? `Gestionnaire détecté : ${nativeManager}` : 'Détection du gestionnaire...'" />

    <NxCard v-if="loadError" danger>{{ loadError }}</NxCard>
    <NxCard v-if="uninstallError" danger>{{ uninstallError }}</NxCard>
    <NxBadge v-if="uninstallResult" status="success">{{ uninstallResult }}</NxBadge>

    <NxInput v-model="searchText" placeholder="Rechercher un paquet..." />

    <NxCard v-for="pkg in filteredPackages" :key="pkg.name" class="unin-card">
      <div class="unin-row">
        <span>{{ pkg.name }}</span>
        <span class="unin-version">{{ pkg.version || "—" }}</span>
        <NxButton v-if="confirmingPackage !== pkg.name" variant="danger" @click="startConfirm(pkg.name)">Désinstaller</NxButton>
      </div>
      <div v-if="confirmingPackage === pkg.name" class="unin-confirm-row">
        <NxInput v-model="confirmText" :placeholder="`Tapez « ${pkg.name} » pour confirmer`" />
        <NxButton
          variant="danger"
          :disabled="uninstalling !== null || confirmText !== pkg.name"
          @click="confirmUninstall(pkg.name)"
        >
          {{ uninstalling === pkg.name ? "Désinstallation..." : "Confirmer la désinstallation" }}
        </NxButton>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.unin-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.unin-card { display: flex; flex-direction: column; gap: 10px; }
.unin-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.unin-version { color: var(--nx-text-secondary); font-size: 12px; }
.unin-confirm-row { display: flex; gap: 10px; align-items: center; }
</style>
```

- [ ] **Step 4: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 5: Commit**

```bash
git add src/pages/UninstallerPage.vue src/pages/UninstallerPage.spec.ts
git commit -m "feat: add UninstallerPage — typed-confirmation gated package removal (spec section 3.3)"
```

---

## Task 8: Extend `categories.ts`'s "maintenance" category + wire `App.vue`

**Files:**
- Modify: `src/navigation/categories.ts`
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`
- Modify: `src/navigation/categories.spec.ts` (the id-uniqueness/every-category-has-pages tests already generalize correctly; only the explicit id list needs extending, same pattern as R6 Task 10's fix)

- [ ] **Step 1: Read the live `src/navigation/categories.ts`.** The `"maintenance"` category currently has 3 pages (`updates`, `drivers`, `troubleshoot`). Add 5 more, in this order (matching NiTriTe's own `navigation.ts` Maintenance category ordering: updates, drivers, uninstaller, cleaner, backup, scanvirus, dependencies):

```typescript
      { id: "uninstaller", label: "Désinstalleur", icon: "trash-2" },
      { id: "cleaner", label: "Nettoyeur", icon: "sparkles" },
      { id: "backup", label: "Sauvegarde", icon: "save" },
      { id: "antivirus", label: "Antivirus", icon: "shield-check" },
      { id: "dependencies", label: "Dépendances", icon: "package-search" },
```
(inserted right after the existing `troubleshoot` entry, before the category's closing `],`)

- [ ] **Step 2: Add the 5 new icon names to `AppNav.vue`'s `iconMap`** (read the live file first — this is the same file Task 1 of Phase R6 wired up; it already imports several lucide icons). Add these imports to the existing `lucide-vue-next` import statement:
```typescript
  Trash2, Sparkles, Save, ShieldCheck, PackageSearch,
```
And these entries to `iconMap`:
```typescript
  "trash-2": Trash2,
  sparkles: Sparkles,
  save: Save,
  "shield-check": ShieldCheck,
  "package-search": PackageSearch,
```

- [ ] **Step 3: Update `categories.spec.ts`'s length assertion and add-new-ids test**, following R6 Task 10's exact precedent (`toHaveLength(8)` stays 8 — this task adds pages to an EXISTING category, not a new category, so the category *count* doesn't change, only `"maintenance"`'s page count grows from 3 to 8):
```typescript
  it("includes the 5 new Phase R7 Maintenance avancée pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("uninstaller");
    expect(allPageIds).toContain("cleaner");
    expect(allPageIds).toContain("backup");
    expect(allPageIds).toContain("antivirus");
    expect(allPageIds).toContain("dependencies");
  });
```

- [ ] **Step 4: Read the live `src/App.vue` and `src/App.spec.ts`.** Add 5 new page imports and map entries (`AntivirusPage`, `CleanerPage`, `DependenciesPage`, `BackupPage`, `UninstallerPage`), and 2 new `App.spec.ts` tests (following the exact pattern of every prior "shows the real X page" test in this file):

```typescript
  it("shows the real AntivirusPage for the antivirus id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Antivirus")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Analyse un dossier");
  });

  it("shows the real UninstallerPage for the uninstaller id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Désinstalleur")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Détection du gestionnaire");
  });
```

- [ ] **Step 5: Run `npx vitest run src/App.spec.ts src/navigation/categories.spec.ts`, confirm all pass** (App.spec.ts: 11 tests — 9 from before + 2 new; categories.spec.ts: 6 tests — 5 from before + 1 new).

- [ ] **Step 6: Commit**

```bash
git add src/navigation/categories.ts src/navigation/categories.spec.ts src/components/nav/AppNav.vue src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue and categories.ts to the 5 new Maintenance avancée pages (spec section 2-3)"
```

---

## Task 9: Full verification pass — frontend, backend, and live VM check (including the install→uninstall cycle)

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite.**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r7-maintenance-avancee && npm run test -- --run 2>&1 | tail -25"`
Expected: baseline entering this plan was 147 (end of R6). This plan's net delta: Task 1 (+2 `AntivirusPage.spec.ts`; `TroubleshootPage.spec.ts` stays at 3 tests, net 0) + Task 2 (+2) + Task 3 (+2) + Task 4 (+2) + Task 7 (+2) + Task 8 (+1 `categories.spec.ts` + 2 `App.spec.ts`) = +13, expected total **160**. Report the real observed total and reconcile if it differs.

- [ ] **Step 2: Type-check.** `npx vue-tsc --noEmit`, expect clean.

- [ ] **Step 3: Confirm the Rust suite.** Expect `164 passed; 0 failed; 1 ignored` (per the running tally: 142 R6 baseline + 3 Task2 + 4 Task3 + 5 Task4 + 8 Task5 + 2 Task6 = 164). Report the real count.

- [ ] **Step 4: Confirm `TroubleshootPage.vue` no longer references `scan_for_malware`, `clean-cache`, or `vacuum-logs`** — `grep -n 'scan_for_malware\|clean-cache\|vacuum-logs' src/pages/TroubleshootPage.vue` should return nothing (those moved to `AntivirusPage.vue`/`CleanerPage.vue` respectively).

- [ ] **Step 5: Build and install on the VM, then run the critical live install→uninstall cycle.**

Build: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r7-maintenance-avancee && npx tauri build 2>&1 | tail -20"` (expect a clean `.deb`, AppImage failing on `xdg-open` as always).

Transfer + install on the VM (`172.18.32.124`, user `dev`, password `1998` — SSH helper script at `C:\Users\Momo\AppData\Local\Temp\claude\C--Users-Momo\880690b1-319b-40bd-bb2c-957700dc8af4\scratchpad\ssh_run.py`/`ssh_put.py`/`ssh_interactive.py`, or write an equivalent if that path is gone by the time this task runs).

**The critical check**: install a genuinely disposable, harmless package via the ALREADY-EXISTING, ALREADY-VM-VERIFIED `install_package` command (e.g. `cowsay` or `sl` — pick one not already installed on the VM), confirm it's really there (`dpkg -l | grep <pkg>`), then invoke the NEW `uninstall_package` command against that exact package, and confirm it is REALLY gone afterward (`dpkg -l | grep <pkg>` should show nothing, or `rc` status at most). This can be done either by launching the app and clicking through the UI on the VM's desktop session, or by invoking the Tauri commands more directly if that proves easier to verify end-to-end — the point is proving `uninstall_package`/the `uninstall-package` pkexec action actually removes a real package on a real system, not just that the Rust validation logic compiles.

Also spot-check `scan_missing_dependencies` and `create_backup` against the real VM (e.g. `ldd /bin/bash` output shape, and that a `tar.gz` really appears in `$HOME` after calling `create_backup`) — lower-risk than the uninstaller, but still real system interaction worth confirming once, not just trusting the mocked Vitest tests.

- [ ] **Step 6: Commit any final cleanup.** No further commit expected if Steps 1-5 all pass clean.

---

## Self-Review

**Spec coverage:** §2 (TroubleshootPage split into AntivirusPage/CleanerPage, cache-size preview) — Tasks 1-2. §3.1 (Dépendances) — Task 3. §3.2 (Sauvegarde) — Task 4. §3.3 (Désinstalleur: `list_installed()` across 4 managers, new pkexec action, typed-confirmation UI, mandatory live VM install→uninstall verification) — Tasks 5-7, verified in Task 9. §4 (out of scope: bulk uninstall, scheduled backup, restore UI) — confirmed no task in this plan adds any of those.

**Placeholder scan:** No "TBD"/"TODO". Zypper's empty `version` field for installed packages is explicitly justified (mirrors dnf.rs's own documented `current_version` limitation for updates), not an oversight.

**Type consistency:** `InstalledPackage` (Rust: `name: String`, `version: String`) matches `UninstallerPage.vue`'s TypeScript interface exactly. `MissingDependency`/`CacheSizeReport`/`backup_filename`'s return shape all match their respective frontend interfaces field-for-field. `uninstall_package(manager, package)`'s signature exactly mirrors `install_package(manager, package)` — same parameter names, same order, verified by cross-reading `install.rs` before writing Task 6. Every `Nx*` prop used across this plan matches R1's `defineProps`, cross-checked against each file while writing this plan (though as R6 Task 10 showed, the implementer must still re-verify live at execution time rather than trust this plan's snapshot blindly — file contents can have drifted since this plan was written).
