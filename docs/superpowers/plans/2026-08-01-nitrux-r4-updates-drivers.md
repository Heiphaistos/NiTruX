# NiTruX Phase R4 — Maintenance > Mises à jour + Pilotes enrichis — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `UpdatesPlaceholder.vue` "Bientôt disponible" page with a real dedicated Updates page (spec §4.2), and give `DriversPage.vue` a richness pass with real per-device driver mapping (spec §4.3).

**Architecture:** §4.2 is pure frontend promotion — `list_updates` and `upgrade_all_packages` already exist, are already VM-verified (Phase 2 Part 2 / Phase 2 Part 1), and are already reachable (buried) inside `PackagesPage.vue`; this plan gives them a dedicated `UpdatesPage.vue` with no new backend work. §4.3 needs one small backend addition: `lspci` alone (already used by `hardware::get_pci_devices`) doesn't report which kernel driver is bound to each device — `lspci -k` does, via a "Kernel driver in use: X" line per device block. `drivers.rs` gains a new parser for that output and `DriverSnapshot` gains a `devices: Vec<DeviceDriver>` field; `DriversPage.vue` is rewritten on `NxCard`/`NxStatTile` instead of a flat `<ul>`, and states honestly that Linux "driver updates" flow through the normal package manager rather than faking a Windows-style separate update mechanism (per spec §4.3's explicit instruction).

**Tech Stack:** Tauri v2 + Rust (backend), Vue 3.5 + TypeScript + Vitest (frontend), same patterns as Phases R1–R3.

---

## Task 1: Backend — per-device driver mapping (`lspci -k` parsing)

**Files:**
- Modify: `src-tauri/src/drivers.rs`

Real `lspci -k` output looks like this (confirmed on the actual dev environment):

```
5582:00:00.0 SCSI storage controller: Red Hat, Inc. Virtio console (rev 01)
	Subsystem: Red Hat, Inc. Virtio console
	Kernel driver in use: virtio-pci
	Kernel modules: virtio_pci
64f4:00:00.0 System peripheral: Red Hat, Inc. Virtio file system (rev 01)
	Kernel driver in use: virtio-pci
	Kernel modules: virtio_pci
8857:00:00.0 3D controller: Microsoft Corporation Device 008e
	Kernel driver in use: dxgkrnl
	Kernel modules: dxgkrnl
```

Device lines start at column 0 (no leading whitespace); everything else (`Subsystem:`, `Kernel driver in use:`, `Kernel modules:`) is indented with a tab and belongs to the device line immediately above it. Not every device block has a `Kernel driver in use:` line (some devices have no driver bound) — that case must map to `None`, not be dropped or error.

- [ ] **Step 1: Write the failing tests**

Read the live `src-tauri/src/drivers.rs` first (reproduced above in this plan's context section — it currently has `DriverSnapshot { loaded_modules, gpu_driver }`, `parse_lsmod_line`, `detect_gpu_driver`, `run_lsmod`, `get_driver_snapshot`). Add these tests to the existing `#[cfg(test)] mod tests` block (do not remove any existing test):

```rust
    #[test]
    fn parses_a_device_block_with_kernel_driver_in_use() {
        let output = "5582:00:00.0 SCSI storage controller: Red Hat, Inc. Virtio console (rev 01)\n\tSubsystem: Red Hat, Inc. Virtio console\n\tKernel driver in use: virtio-pci\n\tKernel modules: virtio_pci\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].slot, "5582:00:00.0");
        assert_eq!(devices[0].description, "Red Hat, Inc. Virtio console (rev 01)");
        assert_eq!(devices[0].driver.as_deref(), Some("virtio-pci"));
    }

    #[test]
    fn parses_multiple_device_blocks() {
        let output = "5582:00:00.0 SCSI storage controller: Red Hat, Inc. Virtio console (rev 01)\n\tKernel driver in use: virtio-pci\n64f4:00:00.0 System peripheral: Red Hat, Inc. Virtio file system (rev 01)\n\tKernel driver in use: virtio-pci\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 2);
        assert_eq!(devices[1].slot, "64f4:00:00.0");
    }

    #[test]
    fn maps_missing_kernel_driver_line_to_none_rather_than_dropping_the_device() {
        let output = "00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 620\n\tSubsystem: Some Vendor Device\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].driver, None);
    }

    #[test]
    fn skips_malformed_device_header_lines_without_dropping_valid_siblings() {
        let output = "not a valid header line\n\tKernel driver in use: bogus\n00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 620\n\tKernel driver in use: i915\n";
        let devices = parse_lspci_k_output(output);
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].slot, "00:02.0");
        assert_eq!(devices[0].driver.as_deref(), Some("i915"));
    }
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers/src-tauri && cargo test parses_a_device_block parses_multiple_device_blocks maps_missing_kernel_driver skips_malformed_device_header 2>&1 | tail -20"`
Expected: FAIL — `parse_lspci_k_output` and `DeviceDriver` don't exist yet.

- [ ] **Step 3: Implement `DeviceDriver`, `parse_lspci_k_output`, and wire them into `DriverSnapshot`**

Add this above `pub struct DriverSnapshot` (reuses the existing `Serialize`/`Clone` derive pattern already used by every other snapshot struct in this codebase):

```rust
#[derive(Serialize, Clone)]
pub struct DeviceDriver {
    pub slot: String,
    pub description: String,
    /// `None` when no kernel driver is currently bound to this device —
    /// a real, honest state (not every PCI device has one), not an error.
    pub driver: Option<String>,
}

/// Parses `lspci -k` output into one entry per device block. A device
/// block starts with an unindented line (`<slot> <class>: <description>`)
/// and is followed by zero or more tab-indented detail lines, of which
/// only `Kernel driver in use: <name>` is extracted here — `Subsystem:`
/// and `Kernel modules:` lines are ignored (the *bound* driver is what
/// matters for "what's actually running", not every module that
/// could theoretically bind).  Lines that don't parse as a device header
/// (no `slot description: text` shape) are skipped rather than causing
/// the whole parse to fail or misattributing trailing indented lines to
/// the wrong device — mirrors `parse_lspci_line`'s existing tolerance in
/// this same module's PCI-adjacent sibling, `hardware::parse_lspci_line`.
pub fn parse_lspci_k_output(output: &str) -> Vec<DeviceDriver> {
    let mut devices = Vec::new();
    for line in output.lines() {
        if !line.starts_with(char::is_whitespace) {
            // New device header line.
            if let Some((slot, rest)) = line.split_once(' ') {
                if let Some((_class, description)) = rest.split_once(": ") {
                    devices.push(DeviceDriver {
                        slot: slot.to_string(),
                        description: description.to_string(),
                        driver: None,
                    });
                    continue;
                }
            }
            // Malformed header — not a device block; any indented lines
            // that follow have no valid current device to attach to and
            // are skipped below via the `else` branch's guard.
        } else if let Some(current) = devices.last_mut() {
            let trimmed = line.trim_start();
            if let Some(driver_name) = trimmed.strip_prefix("Kernel driver in use: ") {
                current.driver = Some(driver_name.to_string());
            }
        }
    }
    devices
}

fn run_lspci_k() -> Result<Vec<DeviceDriver>, String> {
    let output = subprocess::run_with_timeout("lspci", &["-k"], Duration::from_secs(5))
        .map_err(|e| format!("{e} (paquet requis : pciutils)"))?;
    Ok(parse_lspci_k_output(&output))
}
```

Then update `DriverSnapshot` and `get_driver_snapshot`:

```rust
#[derive(Serialize, Clone)]
pub struct DriverSnapshot {
    pub loaded_modules: Vec<String>,
    pub gpu_driver: String,
    pub devices: Vec<DeviceDriver>,
}
```

```rust
#[tauri::command]
pub fn get_driver_snapshot() -> Result<DriverSnapshot, String> {
    let loaded_modules = run_lsmod()?;
    let gpu_driver = detect_gpu_driver(&loaded_modules);
    let devices = run_lspci_k().unwrap_or_default();
    Ok(DriverSnapshot { loaded_modules, gpu_driver, devices })
}
```

`devices` deliberately uses `.unwrap_or_default()` rather than `?` — `lspci -k` needing the `pciutils` package (same dependency `hardware::get_pci_devices` already declares) shouldn't make the whole driver snapshot fail if it's missing; `loaded_modules`/`gpu_driver` (from `lsmod`, a different, near-universally-present binary) still have value on their own. This mirrors the existing codebase's general pattern of degrading gracefully rather than making one missing optional binary block an entire read-only snapshot (see `malwarescan`'s and `smart`'s handling of missing binaries elsewhere in this codebase).

- [ ] **Step 4: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers/src-tauri && cargo test 2>&1 | tail -15"`
Expected: `128 passed; 0 failed; 1 ignored` (124 baseline + 4 new tests from Step 1).

- [ ] **Step 5: Verify against the real host** (this dev environment has `pciutils` installed and WSL2 exposes real PCI-like devices)

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers/src-tauri && cargo build 2>&1 | tail -5"` then manually sanity-check by running `lspci -k | head -20` directly and confirming the parser's expectations (slot format, tab indentation, `Kernel driver in use:` prefix) still match what Step 1's tests assert — they were written against this exact real output, captured during this plan's own research, so this should already agree; only worth re-confirming if Step 4 surprises you.

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/drivers.rs
git commit -m "feat: add per-device kernel driver mapping via lspci -k (spec section 4.3)"
```

---

## Task 2: Frontend — `DriversPage.vue` richness pass

**Files:**
- Modify: `src/pages/DriversPage.vue`
- Test: `src/pages/DriversPage.spec.ts` (none exists yet)

Currently `DriversPage.vue` is a plain, non-componentized page (raw `<h1>`/`<ul>`, no `Nx*` components — it was explicitly left untouched by R2 per spec §5's restructuring list, and untouched again by R3). This task brings it in line with every other page's `Nx*`-based presentation, and adds the new per-device driver table from Task 1's `devices` field, with an honest note about how Linux driver updates actually work.

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/DriversPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DriversPage from "./DriversPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    loaded_modules: ["i915", "snd_hda_intel"],
    gpu_driver: "i915 (Intel, open-source)",
    devices: [
      { slot: "00:02.0", description: "Intel Corporation UHD Graphics 620", driver: "i915" },
      { slot: "00:1f.3", description: "Intel Corporation Audio Controller", driver: null },
    ],
  }),
}));

describe("DriversPage", () => {
  it("renders the GPU driver, the per-device table, and an honest note about Linux driver updates", async () => {
    const wrapper = mount(DriversPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("i915 (Intel, open-source)"));
    expect(wrapper.text()).toContain("Intel Corporation UHD Graphics 620");
    expect(wrapper.text()).toContain("i915");
    expect(wrapper.text()).toContain("Intel Corporation Audio Controller");
    expect(wrapper.find(".nx-card").exists()).toBe(true);
    expect(wrapper.text().toLowerCase()).toContain("gestionnaire de paquets");
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npx vitest run src/pages/DriversPage.spec.ts"`
Expected: FAIL — current `DriversPage.vue` has no `.nx-card`, doesn't render the per-device table (the `devices` field doesn't exist in its current `DriverSnapshot` interface), and has no update-model note.

- [ ] **Step 3: Rewrite `DriversPage.vue`**

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxBadge from "@/components/ui/NxBadge.vue";

interface DeviceDriver { slot: string; description: string; driver: string | null }
interface DriverSnapshot { loaded_modules: string[]; gpu_driver: string; devices: DeviceDriver[] }

const snapshot = ref<DriverSnapshot | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<DriverSnapshot>("get_driver_snapshot");
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="drv-page">
    <NxSectionHeader title="Pilotes" description="Pilotes noyau actifs par périphérique PCI et modules chargés." />

    <NxCard v-if="error" danger>Impossible de récupérer les pilotes : {{ error }}</NxCard>

    <template v-if="snapshot">
      <div class="drv-stats">
        <NxCard><NxStatTile label="Pilote GPU actif" :value="snapshot.gpu_driver" /></NxCard>
        <NxCard><NxStatTile label="Modules chargés" :value="String(snapshot.loaded_modules.length)" /></NxCard>
        <NxCard><NxStatTile label="Périphériques PCI" :value="String(snapshot.devices.length)" /></NxCard>
      </div>

      <NxCard class="drv-note">
        <p>
          Sur Linux, il n'existe pas de mécanisme séparé de « mise à jour de pilote » comme sur Windows : les pilotes sont
          soit intégrés au noyau, soit installés comme des modules via le <strong>gestionnaire de paquets</strong> du système
          (voir <strong>Maintenance &gt; Mises à jour</strong>). Un pilote à jour est simplement un système à jour.
        </p>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Périphériques & pilotes" />
        <table class="drv-table">
          <thead>
            <tr><th>Emplacement</th><th>Périphérique</th><th>Pilote</th></tr>
          </thead>
          <tbody>
            <tr v-for="d in snapshot.devices" :key="d.slot">
              <td>{{ d.slot }}</td>
              <td>{{ d.description }}</td>
              <td>
                <NxBadge v-if="d.driver" status="success">{{ d.driver }}</NxBadge>
                <NxBadge v-else status="warning">aucun</NxBadge>
              </td>
            </tr>
          </tbody>
        </table>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Modules chargés" :description="String(snapshot.loaded_modules.length)" />
        <div class="drv-modules">
          <NxBadge v-for="mod in snapshot.loaded_modules" :key="mod" status="info">{{ mod }}</NxBadge>
        </div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.drv-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.drv-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; }
.drv-note p { margin: 0; font-size: 13px; color: var(--nx-text-secondary); line-height: 1.5; }
.drv-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.drv-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.drv-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
.drv-modules { display: flex; flex-wrap: wrap; gap: 6px; }
</style>
```

Before pasting this in verbatim, cross-check `NxCard`, `NxStatTile`, `NxSectionHeader`, `NxBadge`'s actual live `defineProps` (read each file in `src/components/ui/`) against the props used here (`danger`, `label`/`value`/`status`, `title`/`description`, `status`) — adapt and note any drift.

- [ ] **Step 4: Run it, confirm it PASSES**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npx vitest run src/pages/DriversPage.spec.ts"`
Expected: PASS (1 test)

- [ ] **Step 5: Commit**

```bash
git add src/pages/DriversPage.vue src/pages/DriversPage.spec.ts
git commit -m "feat: richness pass for DriversPage — Nx* components + per-device driver table (spec section 4.3)"
```

---

## Task 3: Frontend — dedicated `UpdatesPage.vue`

**Files:**
- Create: `src/pages/UpdatesPage.vue`
- Test: `src/pages/UpdatesPage.spec.ts`

`PackagesPage.vue` (spec §5: stays as "Applications > Gestionnaire de paquets", the "raw package manager" view) currently ALSO hosts the update-checking/upgrade-all UI. Per spec §4.2, this task gives that same functionality (`list_updates` + `upgrade_all_packages` — no new backend, no change to `PackagesPage.vue` either) its own dedicated page under **Maintenance > Mises à jour**. Both pages calling the same two existing commands independently is intentional per spec §4.2's own framing ("currently only reachable, if at all, through PackagesPage's tab clutter") — `PackagesPage.vue` is explicitly not modified by this task.

- [ ] **Step 1: Write the failing tests**

```typescript
// src/pages/UpdatesPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import UpdatesPage from "./UpdatesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_updates") {
      return Promise.resolve([
        { name: "curl", current_version: "7.88.1", new_version: "7.89.0", source: "apt" },
      ]);
    }
    if (cmd === "upgrade_all_packages") return Promise.resolve("Mise à jour terminée");
    return Promise.resolve(null);
  }),
}));

describe("UpdatesPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("loads and displays upgradable packages on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    expect(invoke).toHaveBeenCalledWith("list_updates");
    expect(wrapper.text()).toContain("7.89.0");
  });

  it("calls upgrade_all_packages when the button is clicked", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("curl"));
    const button = wrapper.findAll("button").find((b) => b.text() === "Tout mettre à jour")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("upgrade_all_packages"));
  });

  it("shows an empty-state message when there are no updates", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_updates") return Promise.resolve([]);
      return Promise.resolve(null);
    });
    const wrapper = mount(UpdatesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune mise à jour"));
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npx vitest run src/pages/UpdatesPage.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 3: Write `UpdatesPage.vue`**

```vue
<!-- src/pages/UpdatesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface PackageUpdate {
  name: string;
  current_version: string;
  new_version: string;
  source: string;
}

const updates = ref<PackageUpdate[]>([]);
const error = ref<string | null>(null);
const loading = ref(false);

async function refresh() {
  loading.value = true;
  error.value = null;
  try {
    updates.value = await invoke<PackageUpdate[]>("list_updates");
  } catch (e) {
    error.value = String(e);
  } finally {
    loading.value = false;
  }
}

onMounted(refresh);

const upgrading = ref(false);
const upgradeResult = ref<string | null>(null);
const upgradeError = ref<string | null>(null);

async function upgradeAll() {
  upgrading.value = true;
  upgradeError.value = null;
  upgradeResult.value = null;
  try {
    upgradeResult.value = await invoke<string>("upgrade_all_packages");
    await refresh();
  } catch (e) {
    upgradeError.value = String(e);
  } finally {
    upgrading.value = false;
  }
}
</script>

<template>
  <div class="upd-page">
    <div class="upd-header">
      <NxSectionHeader title="Mises à jour" description="Paquets pouvant être mis à jour, tous gestionnaires détectés confondus." />
      <div class="upd-actions">
        <NxButton :disabled="loading" @click="refresh">{{ loading ? "Vérification..." : "Vérifier" }}</NxButton>
        <NxButton :disabled="upgrading || updates.length === 0" @click="upgradeAll">
          {{ upgrading ? "Mise à jour..." : "Tout mettre à jour" }}
        </NxButton>
      </div>
    </div>

    <NxCard v-if="upgradeError" danger>{{ upgradeError }}</NxCard>
    <NxBadge v-if="upgradeResult" status="success">Mise à jour terminée.</NxBadge>

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-else-if="!loading && updates.length === 0" class="upd-empty">
      Aucune mise à jour disponible.
    </div>

    <NxCard v-else>
      <table class="upd-table">
        <thead>
          <tr><th>Source</th><th>Paquet</th><th>Version actuelle</th><th>Nouvelle version</th></tr>
        </thead>
        <tbody>
          <tr v-for="u in updates" :key="`${u.source}-${u.name}`">
            <td><NxBadge status="info">{{ u.source }}</NxBadge></td>
            <td>{{ u.name }}</td>
            <td>{{ u.current_version || "—" }}</td>
            <td>{{ u.new_version }}</td>
          </tr>
        </tbody>
      </table>
    </NxCard>
  </div>
</template>

<style scoped>
.upd-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.upd-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; flex-wrap: wrap; }
.upd-actions { display: flex; gap: 10px; }
.upd-empty { color: var(--nx-text-secondary); }
.upd-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.upd-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.upd-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 4: Run it, confirm it PASSES**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npx vitest run src/pages/UpdatesPage.spec.ts"`
Expected: PASS (3 tests)

- [ ] **Step 5: Commit**

```bash
git add src/pages/UpdatesPage.vue src/pages/UpdatesPage.spec.ts
git commit -m "feat: add dedicated UpdatesPage (spec section 4.2)"
```

---

## Task 4: Wire `App.vue` to the real `UpdatesPage`, retire the placeholder

**Files:**
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`
- Delete: `src/pages/UpdatesPlaceholder.vue` (`git rm`)

Same pattern as R3 Task 4. Read the live `src/App.vue` and `src/App.spec.ts` first.

- [ ] **Step 1: Add a test to `App.spec.ts`**

There is no existing test referencing `UpdatesPlaceholder` by name to replace (R2's `App.spec.ts` only had a dedicated test for the `quick-install` placeholder, which R3 already replaced) — add a new test to the existing `describe("App", ...)` block:

```typescript
  it("shows the real UpdatesPage (not a placeholder) for the updates id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const updatesButton = buttons.find((b) => b.text() === "Mises à jour")!;
    await updatesButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R4");
  });
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npx vitest run src/App.spec.ts"`
Expected: FAIL — `App.vue` still maps `updates` to the placeholder.

- [ ] **Step 3: Update `App.vue`**

Replace the `UpdatesPlaceholder` import with `UpdatesPage`:
```typescript
import UpdatesPage from "@/pages/UpdatesPage.vue";
```
(replaces `import UpdatesPlaceholder from "@/pages/UpdatesPlaceholder.vue";`)

And update the map entry:
```typescript
  updates: UpdatesPage,
```
(replaces `updates: UpdatesPlaceholder,` — every other line unchanged)

- [ ] **Step 4: Delete the placeholder**

```bash
git rm src/pages/UpdatesPlaceholder.vue
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npx vitest run src/App.spec.ts"`
Expected: PASS (5 tests — 4 from before + this task's new one).

- [ ] **Step 6: Commit**

```bash
git add src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue to the real UpdatesPage, retire the placeholder (spec section 4.2)"
```

---

## Task 5: Full verification pass

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npm run test -- --run 2>&1 | tail -20"`
Expected: baseline entering this plan was 119 (end of R3). This plan adds: Task 2 (1 test) + Task 3 (3 tests) + Task 4 (1 new test) = 5 → expected total 124. Report the real observed number.

- [ ] **Step 2: Type-check**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && npx vue-tsc --noEmit"`
Expected: clean.

- [ ] **Step 3: Confirm the Rust suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers/src-tauri && cargo test 2>&1 | tail -10"`
Expected: `128 passed; 0 failed; 1 ignored` (124 baseline + 4 new from Task 1).

- [ ] **Step 4: Confirm `UpdatesPlaceholder.vue` is fully gone**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r4-updates-drivers && grep -rn 'UpdatesPlaceholder' src/ || echo 'no references found'"`
Expected: `no references found`.

- [ ] **Step 5: Manual smoke check**

Confirm `src/App.vue`'s `pages` map still has 15 entries with `updates` now pointing at `UpdatesPage` (not a placeholder), and `PackagesPage.vue` is byte-for-byte untouched by this plan (`git diff master -- src/pages/PackagesPage.vue` should be empty once this branch is checked against `master`).

- [ ] **Step 6: Commit any final cleanup**

No further commit expected if Steps 1–5 all pass clean.

---

## Self-Review

**Spec coverage:** §4.2 (dedicated Updates page, reusing existing commands, no new backend) — Tasks 3–4. §4.3 (per-device driver info, honest note about Linux's update model, `NxCard`/`NxStatTile` richness pass) — Tasks 1–2. `ReportGeneratorPlaceholder.vue` (§4.4) is explicitly NOT touched by this plan — that's R5's job.

**Placeholder scan:** No "TBD"/"TODO". Task 1's `.unwrap_or_default()` choice for `devices` is explicitly justified (graceful degradation matching the existing codebase pattern for optional binaries), not a stub.

**Type consistency:** `DeviceDriver` (Rust, Task 1: `slot: String`, `description: String`, `driver: Option<String>`) maps to TypeScript `DeviceDriver` (Task 2: `slot: string`, `description: string`, `driver: string | null`) — the same `Option<T>` ↔ `T | null` convention already used throughout this codebase (e.g. `PciDevice`, existing `ListeningPort.process`). `PackageUpdate` fields in Task 3's `UpdatesPage.vue` (`name`, `current_version`, `new_version`, `source`) are copied verbatim from the existing, already-correct interface in `PackagesPage.vue` — not redefined from scratch. `Nx*` component prop names (`NxCard`'s `danger`, `NxButton`'s `disabled`, `NxBadge`'s `status`, `NxStatTile`'s `label`/`value`/`status`, `NxSectionHeader`'s `title`/`description`) match their R1 `defineProps` exactly, cross-checked against each file while writing this plan.
