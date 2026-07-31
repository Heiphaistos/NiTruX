# NiTruX Redesign — Phase R2 (Restructure Existing Pages) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move every existing NiTruX page onto the categorized navigation (`AppNav`, `categories.ts`) and shared UI primitives (`NxCard`/`NxButton`/`NxInput`/`NxSelect`/`NxBadge`/`NxSectionHeader`) built in Phase R1 — splitting `DisksPage`/`SecurityPage` where their content spans more than one new category, componentizing the rest in place, adding the small new "Préférences" settings page, and finally wiring `App.vue` to actually use the new nav instead of its current flat 9-button list.

**Architecture:** Every page's *business logic* (refs, `invoke()` calls, function names, event handlers) is preserved byte-for-byte from the current implementation — only the *template markup and CSS* change, replacing ad-hoc per-page classes with the shared `Nx*` components. `App.vue` is the last task: it swaps its hardcoded `<nav>` block for `<AppNav v-model="currentPage" />` and extends its `PageId` union + `Record<PageId, Component>` to cover all 15 page ids already defined in `categories.ts` (some pointing at renamed/new components from this plan, the rest — `quick-install`, `updates`, `report-generator` — deferred to R3/R4/R5 and intentionally left unmapped until those phases add real components for them, so `AppNav` will show those 3 nav items but clicking them does nothing yet; that's an accepted, temporary gap explicitly called out in Task 8).

**Tech Stack:** Vue 3 + `<script setup>` + TypeScript, Pinia, Vitest + `@vue/test-utils`, the R1 `Nx*` component library and `AppNav`/`categories.ts`.

---

## File Structure

```
src/
├── pages/
│   ├── DiagnosticPage.vue        # RENAMED from HardwarePage.vue, componentized
│   ├── PackagesPage.vue          # MODIFIED: componentized in place
│   ├── LogsPage.vue              # MODIFIED: componentized in place
│   ├── DisksPage.vue             # MODIFIED: loses duplicates/largefiles/hash, componentized
│   ├── FileToolsPage.vue         # NEW: duplicates/largefiles/hash, split out of DisksPage
│   ├── FirewallPage.vue          # RENAMED from SecurityPage.vue: firewall status only
│   ├── TroubleshootPage.vue      # NEW: malware/quarantine/snapshots/troubleshoot, split out of SecurityPage
│   ├── NetworkPage.vue           # MODIFIED: componentized in place
│   ├── SettingsPreferencesPage.vue  # NEW: spec §4.5
│   ├── ComingSoonPage.vue        # NEW: shared "not built yet" placeholder UI
│   ├── QuickInstallPlaceholder.vue      # NEW: quick-install id, replaced whole by R3
│   ├── UpdatesPlaceholder.vue           # NEW: updates id, replaced whole by R4
│   └── ReportGeneratorPlaceholder.vue   # NEW: report-generator id, replaced whole by R5
├── stores/
│   └── preferencesStore.ts       # NEW: backs SettingsPreferencesPage
└── App.vue                       # MODIFIED: AppNav wired in, PageId/Record extended to 15 ids
```

**Files explicitly NOT touched by this plan:** `src/pages/DashboardPage.vue`, `src/pages/DriversPage.vue`, `src/pages/ThemeEditorPage.vue` (already R1-complete) — none of these change category or get split, per spec §5.

---

## Task 1: DiagnosticPage.vue (renamed + componentized HardwarePage)

**Files:**
- Create: `src/pages/DiagnosticPage.vue`
- Delete: `src/pages/HardwarePage.vue`
- Test: `src/pages/DiagnosticPage.spec.ts`

The CURRENT full content of `src/pages/HardwarePage.vue` (47 lines) is exactly this:

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface PciDevice { slot: string; class: string; description: string }

const devices = ref<PciDevice[]>([]);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    devices.value = await invoke<PciDevice[]>("get_pci_devices");
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="hw-page">
    <h1>Composants matériels</h1>
    <div class="hw-error" v-if="error">
      Impossible de récupérer les composants matériels : {{ error }}
    </div>
    <table class="hw-table" v-if="devices.length">
      <thead>
        <tr><th>Slot</th><th>Classe</th><th>Description</th></tr>
      </thead>
      <tbody>
        <tr v-for="d in devices" :key="d.slot">
          <td>{{ d.slot }}</td>
          <td>{{ d.class }}</td>
          <td>{{ d.description }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.hw-page { padding: 24px; color: var(--nx-text-primary); }
.hw-error { margin-top: 16px; padding: 12px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); }
.hw-table { width: 100%; border-collapse: collapse; margin-top: 16px; font-size: 13px; }
.hw-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-border); padding: 8px; }
.hw-table td { padding: 8px; border-bottom: 1px solid var(--nx-border); }
</style>
```

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/DiagnosticPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import DiagnosticPage from "./DiagnosticPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([
    { slot: "00:02.0", class: "VGA", description: "Intel UHD Graphics" },
  ]),
}));

describe("DiagnosticPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_pci_devices and renders the returned device", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DiagnosticPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Intel UHD Graphics"));
    expect(invoke).toHaveBeenCalledWith("get_pci_devices");
  });

  it("renders devices inside an NxCard", async () => {
    const wrapper = mount(DiagnosticPage);
    await vi.waitFor(() => expect(wrapper.find(".nx-card").exists()).toBe(true));
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/DiagnosticPage.spec.ts"`
Expected: FAIL — `./DiagnosticPage.vue` doesn't exist.

- [ ] **Step 3: Write `DiagnosticPage.vue`**

```vue
<!-- src/pages/DiagnosticPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface PciDevice { slot: string; class: string; description: string }

const devices = ref<PciDevice[]>([]);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    devices.value = await invoke<PciDevice[]>("get_pci_devices");
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="diag-page">
    <NxSectionHeader title="Diagnostic" description="Composants matériels détectés (PCI)." />
    <NxCard v-if="error" danger>
      Impossible de récupérer les composants matériels : {{ error }}
    </NxCard>
    <NxCard v-if="devices.length">
      <table class="diag-table">
        <thead>
          <tr><th>Slot</th><th>Classe</th><th>Description</th></tr>
        </thead>
        <tbody>
          <tr v-for="d in devices" :key="d.slot">
            <td>{{ d.slot }}</td>
            <td>{{ d.class }}</td>
            <td>{{ d.description }}</td>
          </tr>
        </tbody>
      </table>
    </NxCard>
  </div>
</template>

<style scoped>
.diag-page { padding: 24px; }
.diag-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.diag-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.diag-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/DiagnosticPage.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 5: Delete the old file**

```bash
git rm src/pages/HardwarePage.vue
```

(Do not delete `src/pages/HardwarePage.vue` via a plain filesystem delete — use `git rm` so the removal is staged correctly alongside the new file's addition in the same commit, producing a clean rename-like diff.)

- [ ] **Step 6: Keep `App.vue` buildable — update its `HardwarePage` import and map entry in place**

The full `AppNav`/categorized-nav wiring happens in Task 8. Until then, every task that renames or removes a page must still keep `App.vue` compiling and passing `vue-tsc`, so this small, mechanical touch-up is part of every such task, not deferred.

In `src/App.vue`, change the import:
```typescript
import HardwarePage from "@/pages/HardwarePage.vue";
```
to:
```typescript
import DiagnosticPage from "@/pages/DiagnosticPage.vue";
```

Change the `pages` map entry:
```typescript
  hardware: HardwarePage,
```
to:
```typescript
  hardware: DiagnosticPage,
```

(The `PageId` union still contains `"hardware"` and the nav button still says "Matériel" and reads `currentPage === 'hardware'` — none of that changes yet. Only the *component* backing that existing id changes. Task 8 is what renames the id itself to `"diagnostic"` and rewrites the nav UI.)

- [ ] **Step 7: Verify the whole project still type-checks and the full test suite still passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (baseline 79 from R1 + 2 new = 81).

- [ ] **Step 8: Commit**

```bash
git add src/pages/DiagnosticPage.vue src/pages/DiagnosticPage.spec.ts src/App.vue
git commit -m "feat: rename+componentize HardwarePage to DiagnosticPage (spec section 5)"
```

---

## Task 2: Componentize PackagesPage.vue in place

**Files:**
- Modify: `src/pages/PackagesPage.vue`
- Test: `src/pages/PackagesPage.spec.ts` (none exists yet)

No file rename (stays `PackagesPage.vue`, same import path) — **no `App.vue` change needed for this task.** Only its category label changes conceptually (becomes "Applications > Gestionnaire de paquets" once Task 8 rewires the nav); its component identity doesn't move.

The CURRENT full content of `src/pages/PackagesPage.vue` (135 lines) is exactly this:

```vue
<!-- src/pages/PackagesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

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

function sourceBadgeClass(source: string): string {
  return `pkg-badge pkg-badge--${source}`;
}

const installManager = ref<"apt" | "dnf" | "pacman" | "zypper">("apt");
const installPackageName = ref("");
const installResult = ref<string | null>(null);
const installError = ref<string | null>(null);
const installing = ref(false);

async function installOne() {
  installing.value = true;
  installError.value = null;
  installResult.value = null;
  try {
    installResult.value = await invoke<string>("install_package", {
      manager: installManager.value,
      package: installPackageName.value,
    });
  } catch (e) {
    installError.value = String(e);
  } finally {
    installing.value = false;
  }
}

const upgrading = ref(false);
const upgradeResult = ref<string | null>(null);
const upgradeError = ref<string | null>(null);

async function upgradeAll() {
  upgrading.value = true;
  upgradeError.value = null;
  upgradeResult.value = null;
  try {
    upgradeResult.value = await invoke<string>("upgrade_all_packages");
  } catch (e) {
    upgradeError.value = String(e);
  } finally {
    upgrading.value = false;
  }
}
</script>

<template>
  <div class="pkg-page">
    <div class="pkg-header">
      <h1>Paquets & mises à jour</h1>
      <button class="pkg-refresh" :disabled="loading" @click="refresh">
        {{ loading ? "Vérification..." : "Vérifier les mises à jour" }}
      </button>
    </div>

    <div class="pkg-install-row">
      <select v-model="installManager">
        <option value="apt">apt</option>
        <option value="dnf">dnf</option>
        <option value="pacman">pacman</option>
        <option value="zypper">zypper</option>
      </select>
      <input v-model="installPackageName" class="pkg-input" placeholder="Nom du paquet à installer..." />
      <button :disabled="installing" @click="installOne">{{ installing ? "Installation..." : "Installer" }}</button>
      <button :disabled="upgrading" @click="upgradeAll">{{ upgrading ? "Mise à jour..." : "Tout mettre à jour" }}</button>
    </div>
    <div v-if="installError" class="pkg-error">{{ installError }}</div>
    <div v-if="installResult" class="pkg-success">Installation terminée.</div>
    <div v-if="upgradeError" class="pkg-error">{{ upgradeError }}</div>
    <div v-if="upgradeResult" class="pkg-success">Mise à jour terminée.</div>

    <div v-if="error" class="pkg-error">{{ error }}</div>

    <div v-else-if="!loading && updates.length === 0" class="pkg-empty">
      Aucune mise à jour disponible.
    </div>

    <table v-else class="pkg-table">
      <thead>
        <tr><th>Source</th><th>Paquet</th><th>Version actuelle</th><th>Nouvelle version</th></tr>
      </thead>
      <tbody>
        <tr v-for="u in updates" :key="`${u.source}-${u.name}`">
          <td><span :class="sourceBadgeClass(u.source)">{{ u.source }}</span></td>
          <td>{{ u.name }}</td>
          <td>{{ u.current_version || "—" }}</td>
          <td>{{ u.new_version }}</td>
        </tr>
      </tbody>
    </table>
  </div>
</template>

<style scoped>
.pkg-page { padding: 24px; color: var(--nx-text-primary); }
.pkg-header { display: flex; justify-content: space-between; align-items: center; }
.pkg-refresh { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; }
.pkg-refresh:disabled { opacity: 0.6; cursor: default; }
.pkg-error { margin-top: 16px; padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.pkg-empty { margin-top: 16px; color: var(--nx-text-secondary); }
.pkg-install-row { display: flex; gap: 10px; align-items: center; margin-bottom: 12px; }
.pkg-success { padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); margin-bottom: 10px; }
.pkg-table { width: 100%; border-collapse: collapse; margin-top: 16px; font-size: 13px; }
.pkg-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-border); padding: 8px; }
.pkg-table td { padding: 8px; border-bottom: 1px solid var(--nx-border); }
.pkg-badge { display: inline-block; padding: 2px 8px; border-radius: 999px; font-size: 11px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); }
</style>
```

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/PackagesPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import PackagesPage from "./PackagesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("PackagesPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("calls list_updates on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(PackagesPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_updates"));
  });

  it("renders the install form using NxInput and NxSelect", () => {
    const wrapper = mount(PackagesPage);
    expect(wrapper.find(".nx-input").exists()).toBe(true);
    expect(wrapper.find(".nx-select").exists()).toBe(true);
  });

  it("calls install_package with the manager and package name on install click", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(PackagesPage);
    await wrapper.find(".nx-input").setValue("curl");
    const buttons = wrapper.findAll("button");
    const installButton = buttons.find((b) => b.text() === "Installer")!;
    await installButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "curl" });
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/PackagesPage.spec.ts"`
Expected: FAIL — current `PackagesPage.vue` uses plain `<input>`/`<select>`, not `.nx-input`/`.nx-select`.

- [ ] **Step 3: Rewrite `PackagesPage.vue`**

```vue
<!-- src/pages/PackagesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
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

const MANAGER_OPTIONS = [
  { value: "apt", label: "apt" },
  { value: "dnf", label: "dnf" },
  { value: "pacman", label: "pacman" },
  { value: "zypper", label: "zypper" },
];

const installManager = ref<"apt" | "dnf" | "pacman" | "zypper">("apt");
const installPackageName = ref("");
const installResult = ref<string | null>(null);
const installError = ref<string | null>(null);
const installing = ref(false);

async function installOne() {
  installing.value = true;
  installError.value = null;
  installResult.value = null;
  try {
    installResult.value = await invoke<string>("install_package", {
      manager: installManager.value,
      package: installPackageName.value,
    });
  } catch (e) {
    installError.value = String(e);
  } finally {
    installing.value = false;
  }
}

const upgrading = ref(false);
const upgradeResult = ref<string | null>(null);
const upgradeError = ref<string | null>(null);

async function upgradeAll() {
  upgrading.value = true;
  upgradeError.value = null;
  upgradeResult.value = null;
  try {
    upgradeResult.value = await invoke<string>("upgrade_all_packages");
  } catch (e) {
    upgradeError.value = String(e);
  } finally {
    upgrading.value = false;
  }
}
</script>

<template>
  <div class="pkg-page">
    <div class="pkg-header">
      <NxSectionHeader title="Gestionnaire de paquets" description="Installation directe et mises à jour via le gestionnaire de paquets du système." />
      <NxButton :disabled="loading" @click="refresh">
        {{ loading ? "Vérification..." : "Vérifier les mises à jour" }}
      </NxButton>
    </div>

    <NxCard class="pkg-install-card">
      <div class="pkg-install-row">
        <NxSelect v-model="installManager" :options="MANAGER_OPTIONS" />
        <NxInput v-model="installPackageName" placeholder="Nom du paquet à installer..." />
        <NxButton :disabled="installing" @click="installOne">{{ installing ? "Installation..." : "Installer" }}</NxButton>
        <NxButton :disabled="upgrading" @click="upgradeAll">{{ upgrading ? "Mise à jour..." : "Tout mettre à jour" }}</NxButton>
      </div>
      <NxCard v-if="installError" danger>{{ installError }}</NxCard>
      <NxBadge v-if="installResult" status="success">Installation terminée.</NxBadge>
      <NxCard v-if="upgradeError" danger>{{ upgradeError }}</NxCard>
      <NxBadge v-if="upgradeResult" status="success">Mise à jour terminée.</NxBadge>
    </NxCard>

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-else-if="!loading && updates.length === 0" class="pkg-empty">
      Aucune mise à jour disponible.
    </div>

    <NxCard v-else>
      <table class="pkg-table">
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
.pkg-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.pkg-header { display: flex; justify-content: space-between; align-items: flex-start; gap: 12px; }
.pkg-install-card { display: flex; flex-direction: column; gap: 10px; }
.pkg-install-row { display: flex; gap: 10px; align-items: center; }
.pkg-empty { color: var(--nx-text-secondary); }
.pkg-table { width: 100%; border-collapse: collapse; font-size: 13px; }
.pkg-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-style-border-color); padding: 8px; }
.pkg-table td { padding: 8px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/PackagesPage.spec.ts"`
Expected: PASS (3 tests)

- [ ] **Step 5: Verify the whole project still type-checks and the full test suite still passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (81 from Task 1 + 3 new = 84).

- [ ] **Step 6: Commit**

```bash
git add src/pages/PackagesPage.vue src/pages/PackagesPage.spec.ts
git commit -m "feat: componentize PackagesPage on Nx* primitives (spec section 5)"
```

---

## Task 3: Componentize LogsPage.vue in place

**Files:**
- Modify: `src/pages/LogsPage.vue`
- Test: `src/pages/LogsPage.spec.ts` (none exists yet)

No rename, no `App.vue` change needed.

The CURRENT full content of `src/pages/LogsPage.vue` (49 lines) is exactly this:

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface LogEntry { priority: number; message: string; unit: string }

const logs = ref<LogEntry[]>([]);
const error = ref<string | null>(null);

function priorityClass(priority: number): string {
  if (priority <= 3) return "log-error";
  if (priority <= 4) return "log-warning";
  return "log-info";
}

onMounted(async () => {
  try {
    logs.value = await invoke<LogEntry[]>("get_recent_logs", { limit: 200 });
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="logs-page">
    <h1>Journaux système</h1>
    <div class="logs-error" v-if="error">
      Impossible de récupérer les journaux système : {{ error }}
    </div>
    <div class="logs-list" v-if="logs.length">
      <div v-for="(log, i) in logs" :key="i" class="log-entry" :class="priorityClass(log.priority)">
        <span class="log-unit">{{ log.unit }}</span>
        <span class="log-message">{{ log.message }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.logs-page { padding: 24px; color: var(--nx-text-primary); }
.logs-error { margin-top: 16px; padding: 12px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); }
.logs-list { margin-top: 16px; font-family: monospace; font-size: 12px; display: grid; gap: 2px; max-height: 70vh; overflow: auto; }
.log-entry { display: flex; gap: 10px; padding: 4px 8px; border-radius: 4px; }
.log-entry.log-error { background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); }
.log-entry.log-warning { background: color-mix(in srgb, var(--nx-accent-warning) 15%, transparent); }
.log-unit { color: var(--nx-text-secondary); flex-shrink: 0; min-width: 120px; }
</style>
```

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/LogsPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import LogsPage from "./LogsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([
    { priority: 3, message: "disk failure imminent", unit: "smartd" },
  ]),
}));

describe("LogsPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_recent_logs with limit 200 and renders the entry inside an NxCard", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(LogsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("disk failure imminent"));
    expect(invoke).toHaveBeenCalledWith("get_recent_logs", { limit: 200 });
    expect(wrapper.find(".nx-card").exists()).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/LogsPage.spec.ts"`
Expected: FAIL — `.nx-card` doesn't exist in the current markup.

- [ ] **Step 3: Rewrite `LogsPage.vue`**

```vue
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface LogEntry { priority: number; message: string; unit: string }

const logs = ref<LogEntry[]>([]);
const error = ref<string | null>(null);

function priorityClass(priority: number): string {
  if (priority <= 3) return "log-error";
  if (priority <= 4) return "log-warning";
  return "log-info";
}

onMounted(async () => {
  try {
    logs.value = await invoke<LogEntry[]>("get_recent_logs", { limit: 200 });
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});
</script>

<template>
  <div class="logs-page">
    <NxSectionHeader title="Journaux" description="Journaux système récents (journald)." />
    <NxCard v-if="error" danger>
      Impossible de récupérer les journaux système : {{ error }}
    </NxCard>
    <NxCard v-if="logs.length" class="logs-card">
      <div class="logs-list">
        <div v-for="(log, i) in logs" :key="i" class="log-entry" :class="priorityClass(log.priority)">
          <span class="log-unit">{{ log.unit }}</span>
          <span class="log-message">{{ log.message }}</span>
        </div>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.logs-page { padding: 24px; }
.logs-card { padding: 8px; }
.logs-list { font-family: monospace; font-size: 12px; display: grid; gap: 2px; max-height: 70vh; overflow: auto; }
.log-entry { display: flex; gap: 10px; padding: 4px 8px; border-radius: 4px; }
.log-entry.log-error { background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); }
.log-entry.log-warning { background: color-mix(in srgb, var(--nx-accent-warning) 15%, transparent); }
.log-unit { color: var(--nx-text-secondary); flex-shrink: 0; min-width: 120px; }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/LogsPage.spec.ts"`
Expected: PASS (1 test)

- [ ] **Step 5: Verify the whole project still type-checks and the full test suite still passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (84 from Task 2 + 1 new = 85).

- [ ] **Step 6: Commit**

```bash
git add src/pages/LogsPage.vue src/pages/LogsPage.spec.ts
git commit -m "feat: componentize LogsPage on Nx* primitives (spec section 5)"
```

---

## Task 4: Split DisksPage.vue — disks/partitions stay, file tools become FileToolsPage.vue

**Files:**
- Modify: `src/pages/DisksPage.vue` (loses the duplicates/largefiles/hashcheck tabs and their tab bar — becomes single-section, no tabs)
- Create: `src/pages/FileToolsPage.vue` (gets exactly those 3 tabs)
- Test: `src/pages/DisksPage.spec.ts` (none exists yet)
- Test: `src/pages/FileToolsPage.spec.ts`
- Modify: `src/App.vue` (add the new `file-tools` id/import — the `disks` id keeps pointing at `DisksPage`, unchanged)

The CURRENT full content of `src/pages/DisksPage.vue` (289 lines, includes the Phase 3 Part 2 format/extend/clone controls) is reproduced in this plan's File Structure context above — the implementer must `cat` the file directly to get the exact current content rather than relying on a stale copy, since this is the most-recently-modified page in the codebase (this exact instruction applies to every task in this plan touching a page — always read the live file first).

### Step 1: Read the current file and identify the exact split boundary

Run `cat src/pages/DisksPage.vue` (via the Windows-side Read tool, not WSL2). The split boundary is precise:
- **Stays in `DisksPage.vue`**: the `<script setup>` state/functions for `disks`/`usage`/`loadDisks`/`formatDevice`/`formatFstype`/`formatConfirmText`/`formatBusy`/`formatResult`/`formatError`/`runFormat`/`extendDevice`/`extendDisk`/`extendPartNumber`/`extendBusy`/`extendResult`/`extendError`/`runExtend`/`cloneSourceDisk`/`cloneDestPath`/`cloneBusy`/`cloneResult`/`cloneError`/`runClone`/`bytesToGb`, and the template's disk-listing + format/extend/clone sections (currently the `activeTab === 'disks'` branch's content, minus the tab bar itself since there's only one section left).
- **Moves to `FileToolsPage.vue`**: `duplicateGroups`/`scanDir`/`duplicatesError`/`duplicatesLoading`/`scanDuplicates`/`largeFileDir`/`minSizeMb`/`largeFiles`/`largeFilesError`/`largeFilesLoading`/`scanLargeFiles`/`hashPath`/`hashAlgorithm`/`hashResult`/`hashError`/`computeHash`/`bytesToMb`, and the `duplicates`/`largefiles`/`hashcheck` tab content (this page keeps its own internal tab bar — 3 tools, still worth tabbing).

### Step 2: Write the failing tests

```typescript
// src/pages/DisksPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import DisksPage from "./DisksPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("DisksPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes list_disks and list_disk_usage on mount", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    mount(DisksPage);
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("list_disks"));
    expect(invoke).toHaveBeenCalledWith("list_disk_usage");
  });

  it("no longer has a tab bar (duplicates/largefiles/hashcheck moved out)", () => {
    const wrapper = mount(DisksPage);
    expect(wrapper.text()).not.toContain("Doublons");
    expect(wrapper.text()).not.toContain("Gros fichiers");
    expect(wrapper.text()).not.toContain("Vérif. hash");
  });

  it("keeps the format-partition typed-confirmation gate intact", () => {
    const wrapper = mount(DisksPage);
    expect(wrapper.text()).toContain("Formater une partition");
    const buttons = wrapper.findAll("button");
    const formatButton = buttons.find((b) => b.text().includes("Formater"))!;
    expect(formatButton.attributes("disabled")).toBeDefined();
  });
});
```

```typescript
// src/pages/FileToolsPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import FileToolsPage from "./FileToolsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue([]),
}));

describe("FileToolsPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("shows the 3 tool tabs", () => {
    const wrapper = mount(FileToolsPage);
    expect(wrapper.text()).toContain("Doublons");
    expect(wrapper.text()).toContain("Gros fichiers");
    expect(wrapper.text()).toContain("Vérif. hash");
  });

  it("calls find_duplicate_files with the entered directory on the duplicates tab", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FileToolsPage);
    const inputs = wrapper.findAll(".nx-input");
    await inputs[0].setValue("/home/dev");
    const buttons = wrapper.findAll("button");
    const searchButton = buttons.find((b) => b.text() === "Rechercher")!;
    await searchButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("find_duplicate_files", { directory: "/home/dev" });
  });
});
```

### Step 3: Run tests to verify they fail

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/DisksPage.spec.ts src/pages/FileToolsPage.spec.ts"`
Expected: FAIL — `DisksPage.spec.ts` fails on the "no tab bar" assertion (current file still has all 4 tabs); `FileToolsPage.vue` doesn't exist.

### Step 4: Rewrite `DisksPage.vue` (disk/partition management only, no tabs)

```vue
<!-- src/pages/DisksPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface Partition { name: string; size: string; mountpoint: string | null }
interface Disk { name: string; size: string; partitions: Partition[] }
interface UsageEntry { mountpoint: string; total_bytes: number; used_bytes: number; used_percent: number }

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

const FSTYPE_OPTIONS = [
  { value: "ext4", label: "ext4" },
  { value: "btrfs", label: "btrfs" },
  { value: "xfs", label: "xfs" },
  { value: "vfat", label: "vfat" },
];

const formatDevice = ref("");
const formatFstype = ref<"ext4" | "btrfs" | "xfs" | "vfat">("ext4");
const formatConfirmText = ref("");
const formatBusy = ref(false);
const formatResult = ref<string | null>(null);
const formatError = ref<string | null>(null);

async function runFormat() {
  formatBusy.value = true;
  formatError.value = null;
  formatResult.value = null;
  try {
    formatResult.value = await invoke<string>("format_partition", { device: formatDevice.value, fstype: formatFstype.value });
    formatConfirmText.value = "";
    await loadDisks();
  } catch (e) {
    formatError.value = String(e);
  } finally {
    formatBusy.value = false;
  }
}

const extendDevice = ref("");
const extendDisk = ref("");
const extendPartNumber = ref("");
const extendBusy = ref(false);
const extendResult = ref<string | null>(null);
const extendError = ref<string | null>(null);

async function runExtend() {
  extendBusy.value = true;
  extendError.value = null;
  extendResult.value = null;
  try {
    extendResult.value = await invoke<string>("extend_partition", {
      device: extendDevice.value,
      disk: extendDisk.value,
      partNumber: extendPartNumber.value,
    });
    await loadDisks();
  } catch (e) {
    extendError.value = String(e);
  } finally {
    extendBusy.value = false;
  }
}

const cloneSourceDisk = ref("");
const cloneDestPath = ref("");
const cloneBusy = ref(false);
const cloneResult = ref<string | null>(null);
const cloneError = ref<string | null>(null);

async function runClone() {
  cloneBusy.value = true;
  cloneError.value = null;
  cloneResult.value = null;
  try {
    cloneResult.value = await invoke<string>("clone_disk", { sourceDisk: cloneSourceDisk.value, destPath: cloneDestPath.value });
  } catch (e) {
    cloneError.value = String(e);
  } finally {
    cloneBusy.value = false;
  }
}

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="disks-page">
    <NxSectionHeader title="Disques & partitions" description="État des disques, formatage, extension et clonage." />

    <NxCard v-if="disksError" danger>{{ disksError }}</NxCard>

    <NxCard v-for="disk in disks" :key="disk.name" class="disks-disk-card">
      <strong>{{ disk.name }}</strong> — {{ disk.size }}
      <ul>
        <li v-for="p in disk.partitions" :key="p.name">{{ p.name }} ({{ p.size }}){{ p.mountpoint ? ` → ${p.mountpoint}` : "" }}</li>
      </ul>
    </NxCard>

    <NxCard>
      <div v-for="u in usage" :key="u.mountpoint" class="disks-usage-row">
        <span>{{ u.mountpoint }}</span>
        <span>{{ bytesToGb(u.used_bytes) }} / {{ bytesToGb(u.total_bytes) }} GB ({{ u.used_percent }}%)</span>
      </div>
    </NxCard>

    <NxCard danger>
      <NxSectionHeader title="Formater une partition" description="Cette action efface DÉFINITIVEMENT toutes les données de la partition. Aucune récupération possible." />
      <div class="disks-form-row">
        <NxInput v-model="formatDevice" placeholder="Périphérique (ex: /dev/sda1)" />
        <NxSelect v-model="formatFstype" :options="FSTYPE_OPTIONS" />
      </div>
      <div class="disks-form-row">
        <NxInput
          v-model="formatConfirmText"
          :placeholder="`Tapez « ${formatDevice} » pour confirmer`"
        />
        <NxButton
          variant="danger"
          :disabled="formatBusy || formatDevice === '' || formatConfirmText !== formatDevice"
          @click="runFormat"
        >
          {{ formatBusy ? "Formatage..." : "Formater" }}
        </NxButton>
      </div>
      <NxCard v-if="formatError" danger>{{ formatError }}</NxCard>
      <NxBadge v-if="formatResult" status="success">{{ formatResult }}</NxBadge>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Étendre une partition" />
      <div class="disks-form-row">
        <NxInput v-model="extendDevice" placeholder="Partition (ex: /dev/sda1)" />
        <NxInput v-model="extendDisk" placeholder="Disque (ex: /dev/sda)" />
        <NxInput v-model="extendPartNumber" placeholder="N° (ex: 1)" />
        <NxButton :disabled="extendBusy" @click="runExtend">{{ extendBusy ? "Extension..." : "Étendre" }}</NxButton>
      </div>
      <NxCard v-if="extendError" danger>{{ extendError }}</NxCard>
      <NxBadge v-if="extendResult" status="success">{{ extendResult }}</NxBadge>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Cloner un disque" />
      <div class="disks-form-row">
        <NxInput v-model="cloneSourceDisk" placeholder="Disque source (ex: /dev/sda)" />
        <NxInput v-model="cloneDestPath" placeholder="Fichier image de destination" />
        <NxButton :disabled="cloneBusy" @click="runClone">{{ cloneBusy ? "Clonage..." : "Cloner" }}</NxButton>
      </div>
      <NxCard v-if="cloneError" danger>{{ cloneError }}</NxCard>
      <NxBadge v-if="cloneResult" status="success">{{ cloneResult }}</NxBadge>
    </NxCard>
  </div>
</template>

<style scoped>
.disks-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.disks-disk-card { font-size: 13px; }
.disks-usage-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; }
.disks-form-row { display: flex; gap: 10px; align-items: center; margin: 10px 0; }
</style>
```

### Step 5: Write `FileToolsPage.vue` (duplicates/largefiles/hashcheck, keeps its own tab bar)

```vue
<!-- src/pages/FileToolsPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface DuplicateGroup { hash: string; paths: string[]; size_bytes: number }
interface LargeFile { path: string; size_bytes: number }

type Tab = "duplicates" | "largefiles" | "hashcheck";
const activeTab = ref<Tab>("duplicates");

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

const HASH_OPTIONS = [
  { value: "sha256", label: "SHA-256" },
  { value: "sha1", label: "SHA-1" },
  { value: "md5", label: "MD5" },
];

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

function bytesToMb(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="ft-page">
    <NxSectionHeader title="Outils fichiers" description="Doublons, gros fichiers et vérification d'intégrité." />

    <div class="ft-tabs">
      <button :class="{ active: activeTab === 'duplicates' }" @click="activeTab = 'duplicates'">Doublons</button>
      <button :class="{ active: activeTab === 'largefiles' }" @click="activeTab = 'largefiles'">Gros fichiers</button>
      <button :class="{ active: activeTab === 'hashcheck' }" @click="activeTab = 'hashcheck'">Vérif. hash</button>
    </div>

    <NxCard v-if="activeTab === 'duplicates'">
      <div class="ft-form-row">
        <NxInput v-model="scanDir" placeholder="Dossier à scanner..." />
        <NxButton :disabled="duplicatesLoading" @click="scanDuplicates">{{ duplicatesLoading ? "Analyse..." : "Rechercher" }}</NxButton>
      </div>
      <NxCard v-if="duplicatesError" danger>{{ duplicatesError }}</NxCard>
      <div v-for="g in duplicateGroups" :key="g.hash" class="ft-dup-group">
        <div>{{ g.paths.length }} fichiers identiques ({{ bytesToMb(g.size_bytes) }} MB chacun)</div>
        <ul><li v-for="p in g.paths" :key="p">{{ p }}</li></ul>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'largefiles'">
      <div class="ft-form-row">
        <NxInput v-model="largeFileDir" placeholder="Dossier à scanner..." />
        <NxInput v-model.number="minSizeMb" placeholder="MB min" />
        <NxButton :disabled="largeFilesLoading" @click="scanLargeFiles">{{ largeFilesLoading ? "Analyse..." : "Rechercher" }}</NxButton>
      </div>
      <NxCard v-if="largeFilesError" danger>{{ largeFilesError }}</NxCard>
      <div v-for="f in largeFiles" :key="f.path" class="ft-row">
        <span>{{ f.path }}</span>
        <span>{{ bytesToMb(f.size_bytes) }} MB</span>
      </div>
    </NxCard>

    <NxCard v-else>
      <div class="ft-form-row">
        <NxInput v-model="hashPath" placeholder="Chemin du fichier..." />
        <NxSelect v-model="hashAlgorithm" :options="HASH_OPTIONS" />
        <NxButton @click="computeHash">Calculer</NxButton>
      </div>
      <NxCard v-if="hashError" danger>{{ hashError }}</NxCard>
      <div v-if="hashResult" class="ft-hash-result">{{ hashResult }}</div>
    </NxCard>
  </div>
</template>

<style scoped>
.ft-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.ft-tabs { display: flex; gap: 8px; }
.ft-tabs button { padding: 8px 14px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; }
.ft-tabs button.active { color: var(--nx-text-primary); font-weight: 600; }
.ft-form-row { display: flex; gap: 10px; align-items: center; margin-bottom: 10px; }
.ft-dup-group { font-size: 13px; padding: 8px 0; }
.ft-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; }
.ft-hash-result { font-family: monospace; font-size: 12px; word-break: break-all; padding-top: 8px; }
</style>
```

### Step 6: Run tests to verify they pass

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/DisksPage.spec.ts src/pages/FileToolsPage.spec.ts"`
Expected: PASS (3 + 2 = 5 tests)

### Step 7: Wire the new `file-tools` id into `App.vue` (mechanical, keeps the build green)

In `src/App.vue`, add a new import:
```typescript
import FileToolsPage from "@/pages/FileToolsPage.vue";
```

Add `"file-tools"` to the `PageId` union:
```typescript
type PageId =
  | "dashboard"
  | "hardware"
  | "drivers"
  | "logs"
  | "theme-editor"
  | "packages"
  | "disks"
  | "file-tools"
  | "network"
  | "security";
```

Add a map entry:
```typescript
  "file-tools": FileToolsPage,
```

Add a nav button (temporary — Task 8 replaces this whole nav block with `AppNav`, this is just enough to make the new page reachable and testable in the running app before then):
```html
        <button
          :class="{ active: currentPage === 'file-tools' }"
          @click="currentPage = 'file-tools'"
        >
          Outils fichiers
        </button>
```
placed right after the existing "Disques" button.

### Step 8: Verify the whole project still type-checks and the full test suite still passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (85 from Task 3 + 5 new = 90).

### Step 9: Commit

```bash
git add src/pages/DisksPage.vue src/pages/DisksPage.spec.ts src/pages/FileToolsPage.vue src/pages/FileToolsPage.spec.ts src/App.vue
git commit -m "feat: split DisksPage into DisksPage + FileToolsPage (spec section 5)"
```

---

## Task 5: Split SecurityPage.vue — FirewallPage.vue (status only) + TroubleshootPage.vue (everything else)

**Files:**
- Create: `src/pages/FirewallPage.vue`
- Create: `src/pages/TroubleshootPage.vue`
- Delete: `src/pages/SecurityPage.vue`
- Test: `src/pages/FirewallPage.spec.ts`
- Test: `src/pages/TroubleshootPage.spec.ts`
- Modify: `src/App.vue`

Read the live `src/pages/SecurityPage.vue` first (via the Windows-side Read tool) to confirm current content before splitting — it was last touched during tonight's Phase 5 Part 2 work and this plan's reproduction of it must match exactly.

**Split boundary:**
- **`FirewallPage.vue`** gets: `firewall`/`firewallError`/`loadFirewall`, and the current `activeTab === 'firewall'` template branch — no tabs needed (single concern).
- **`TroubleshootPage.vue`** gets everything else: `scanDir`/`findings`/`scanError`/`scanning`/`scanDone`/`runScan`/`quarantining`/`quarantineError`/`quarantineFinding` (malware), `snapshots`/`snapshotsError`/`loadSnapshots`/`snapshotCreating`/`snapshotCreateError`/`createSnapshotNow` (snapshots), `TROUBLESHOOT_ACTIONS`/`troubleshootBusy`/`troubleshootResult`/`troubleshootError`/`runTroubleshootAction` (troubleshoot actions) — keeps its own 3-tab bar (malware/snapshots/troubleshoot) plus an `onTabClick` handler that lazy-loads snapshots the first time that tab is opened (preserve this lazy-load behavior exactly, it's existing, deliberate logic, not something to simplify away).

### Step 1: Write the failing tests

```typescript
// src/pages/FirewallPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import FirewallPage from "./FirewallPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({ active: true, rules: ["22/tcp ALLOW Anywhere"] }),
}));

describe("FirewallPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_firewall_status and renders the active state and rules", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(FirewallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("22/tcp ALLOW Anywhere"));
    expect(invoke).toHaveBeenCalledWith("get_firewall_status");
    expect(wrapper.text()).toContain("actif");
  });

  it("has no tabs (single concern, unlike the old SecurityPage)", () => {
    const wrapper = mount(FirewallPage);
    expect(wrapper.text()).not.toContain("Scan malware");
    expect(wrapper.text()).not.toContain("Dépannage");
  });
});
```

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

  it("shows the malware/snapshots/troubleshoot tabs, no firewall tab", () => {
    const wrapper = mount(TroubleshootPage);
    expect(wrapper.text()).toContain("Scan malware");
    expect(wrapper.text()).toContain("Snapshots");
    expect(wrapper.text()).toContain("Dépannage");
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

  it("runs a troubleshoot action via run_troubleshoot_action", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(TroubleshootPage);
    const tabs = wrapper.findAll("button");
    const troubleshootTab = tabs.find((b) => b.text() === "Dépannage")!;
    await troubleshootTab.trigger("click");
    const buttons = wrapper.findAll("button");
    const execButton = buttons.find((b) => b.text() === "Exécuter")!;
    await execButton.trigger("click");
    expect(invoke).toHaveBeenCalledWith("run_troubleshoot_action", { action: "clean-cache" });
  });
});
```

### Step 2: Run tests to verify they fail

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/FirewallPage.spec.ts src/pages/TroubleshootPage.spec.ts"`
Expected: FAIL — neither file exists yet.

### Step 3: Write `FirewallPage.vue`

```vue
<!-- src/pages/FirewallPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface FirewallStatus { active: boolean; rules: string[] }

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
</script>

<template>
  <div class="fw-page">
    <NxSectionHeader title="Pare-feu" description="État et règles UFW actives." />
    <NxCard v-if="firewallError" danger>{{ firewallError }}</NxCard>
    <template v-else-if="firewall">
      <NxBadge :status="firewall.active ? 'success' : 'warning'">
        UFW {{ firewall.active ? "actif" : "inactif" }}
      </NxBadge>
      <NxCard class="fw-rules">
        <div v-for="(r, i) in firewall.rules" :key="i" class="fw-row">{{ r }}</div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.fw-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.fw-rules { padding: 4px 16px; }
.fw-row { padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.fw-row:last-child { border-bottom: none; }
</style>
```

### Step 4: Write `TroubleshootPage.vue`

```vue
<!-- src/pages/TroubleshootPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface MalwareFinding { path: string; signature: string }
interface SnapshotInfo { id: string; date: string }

type Tab = "malware" | "snapshots" | "troubleshoot";
const activeTab = ref<Tab>("malware");

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

const TROUBLESHOOT_ACTIONS: { id: string; label: string }[] = [
  { id: "clean-cache", label: "Vider le cache des paquets" },
  { id: "fix-broken", label: "Réparer les paquets cassés" },
  { id: "restart-network", label: "Redémarrer le réseau" },
  { id: "vacuum-logs", label: "Purger les anciens journaux" },
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
  <div class="ts-page">
    <NxSectionHeader title="Dépannage" description="Analyse antivirus, instantanés système et actions de maintenance." />

    <div class="ts-tabs">
      <button :class="{ active: activeTab === 'malware' }" @click="onTabClick('malware')">Scan malware</button>
      <button :class="{ active: activeTab === 'snapshots' }" @click="onTabClick('snapshots')">Snapshots</button>
      <button :class="{ active: activeTab === 'troubleshoot' }" @click="onTabClick('troubleshoot')">Dépannage</button>
    </div>

    <NxCard v-if="activeTab === 'malware'">
      <div class="ts-form-row">
        <NxInput v-model="scanDir" placeholder="Dossier à scanner..." />
        <NxButton :disabled="scanning" @click="runScan">{{ scanning ? "Scan en cours..." : "Scanner" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-else-if="scanDone && findings.length === 0" class="ts-empty">Aucune menace détectée.</div>
      <NxCard v-if="quarantineError" danger>{{ quarantineError }}</NxCard>
      <div v-for="f in findings" :key="f.path" class="ts-finding-row">
        <span>{{ f.path }}</span>
        <span>{{ f.signature }}</span>
        <NxButton variant="danger" :disabled="quarantining !== null" @click="quarantineFinding(f.path)">
          {{ quarantining === f.path ? "Mise en quarantaine..." : "Mettre en quarantaine" }}
        </NxButton>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'snapshots'">
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
.ts-empty { color: var(--nx-text-secondary); }
.ts-finding-row, .ts-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.ts-action-label { flex: 1; }
</style>
```

### Step 5: Run tests to verify they pass

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/FirewallPage.spec.ts src/pages/TroubleshootPage.spec.ts"`
Expected: PASS (2 + 3 = 5 tests)

### Step 6: Delete the old file

```bash
git rm src/pages/SecurityPage.vue
```

### Step 7: Wire `App.vue`

Replace the import:
```typescript
import SecurityPage from "@/pages/SecurityPage.vue";
```
with:
```typescript
import FirewallPage from "@/pages/FirewallPage.vue";
import TroubleshootPage from "@/pages/TroubleshootPage.vue";
```

Add `"troubleshoot"` to the `PageId` union (the existing `"security"` id is reused for `FirewallPage`, since firewall status was the majority of the old page's identity and this avoids an extra rename — `TroubleshootPage` gets the new `"troubleshoot"` id):
```typescript
type PageId =
  | "dashboard"
  | "hardware"
  | "drivers"
  | "logs"
  | "theme-editor"
  | "packages"
  | "disks"
  | "file-tools"
  | "network"
  | "security"
  | "troubleshoot";
```

Change the `security` map entry and add a new one:
```typescript
  security: FirewallPage,
  troubleshoot: TroubleshootPage,
```

Add a nav button for the new page (temporary, same rationale as Task 4's Step 7 — Task 8 replaces this whole block):
```html
        <button
          :class="{ active: currentPage === 'troubleshoot' }"
          @click="currentPage = 'troubleshoot'"
        >
          Dépannage
        </button>
```
placed right after the existing "Sécurité" button. Also change that existing button's label from "Sécurité" to "Pare-feu" (it now points at `FirewallPage`, whose narrower scope the old label no longer describes accurately):
```html
        <button
          :class="{ active: currentPage === 'security' }"
          @click="currentPage = 'security'"
        >
          Pare-feu
        </button>
```

### Step 8: Verify the whole project still type-checks and the full test suite still passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (90 from Task 4 + 5 new = 95).

### Step 9: Commit

```bash
git add src/pages/FirewallPage.vue src/pages/FirewallPage.spec.ts src/pages/TroubleshootPage.vue src/pages/TroubleshootPage.spec.ts src/App.vue
git commit -m "feat: split SecurityPage into FirewallPage + TroubleshootPage (spec section 5)"
```

---

## Task 6: Componentize NetworkPage.vue in place

**Files:**
- Modify: `src/pages/NetworkPage.vue`
- Test: `src/pages/NetworkPage.spec.ts` (none exists yet)

No rename, no split — per spec §5, hosts/DNS/firewall-rule editing stays together with the overview on this one page. No `App.vue` change needed beyond what's already there (the `network` id already points at `NetworkPage`).

Read the live `src/pages/NetworkPage.vue` first (via the Windows-side Read tool) — it was last touched during tonight's Phase 4 Part 2 work.

### Step 1: Write the failing test

```typescript
// src/pages/NetworkPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import NetworkPage from "./NetworkPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_network_snapshot") {
      return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: [], hosts_file: "127.0.0.1 localhost\n" });
    }
    if (cmd === "get_docker_snapshot") {
      return Promise.resolve({ available: false, containers: [], images: [] });
    }
    return Promise.resolve(null);
  }),
}));

describe("NetworkPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("invokes get_network_snapshot and get_docker_snapshot on mount, renders inside NxCard", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(wrapper.find(".nx-card").exists()).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_network_snapshot");
    expect(invoke).toHaveBeenCalledWith("get_docker_snapshot");
  });

  it("calls write_hosts_file with the edited content on save", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(NetworkPage);
    await vi.waitFor(() => expect(wrapper.find("textarea").exists()).toBe(true));
    await wrapper.find("textarea").setValue("127.0.0.1 localhost\n127.0.1.1 test\n");
    const buttons = wrapper.findAll("button");
    const saveButtons = buttons.filter((b) => b.text() === "Enregistrer");
    await saveButtons[0].trigger("click");
    expect(invoke).toHaveBeenCalledWith("write_hosts_file", { content: "127.0.0.1 localhost\n127.0.1.1 test\n" });
  });
});
```

### Step 2: Run test to verify it fails

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/NetworkPage.spec.ts"`
Expected: FAIL — current markup has no `.nx-card`.

### Step 3: Rewrite `NetworkPage.vue`

```vue
<!-- src/pages/NetworkPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface WifiNetwork { ssid: string; security: string; signal_percent: number; connected: boolean }
interface ListeningPort { port: number; process: string | null }
interface NetworkSnapshot { wifi_networks: WifiNetwork[]; listening_ports: ListeningPort[]; dns_servers: string[]; hosts_file: string }
interface PortResult { port: number; open: boolean }
interface Container { id: string; image: string; name: string; status: string }
interface DockerImageInfo { id: string; repository: string; tag: string; size: string }
interface DockerSnapshot { available: boolean; containers: Container[]; images: DockerImageInfo[] }

type Tab = "overview" | "portscan" | "docker";
const activeTab = ref<Tab>("overview");

const snapshot = ref<NetworkSnapshot | null>(null);
const docker = ref<DockerSnapshot | null>(null);

const hostsEditable = ref("");
const hostsSaving = ref(false);
const hostsSaveError = ref<string | null>(null);
const hostsSaveSuccess = ref(false);

const dnsEditable = ref("");
const dnsSaving = ref(false);
const dnsSaveError = ref<string | null>(null);
const dnsSaveSuccess = ref(false);

const firewallPortProto = ref("");
const firewallResult = ref<string | null>(null);
const firewallError = ref<string | null>(null);
const firewallBusy = ref(false);

onMounted(async () => {
  snapshot.value = await invoke<NetworkSnapshot>("get_network_snapshot");
  docker.value = await invoke<DockerSnapshot>("get_docker_snapshot");
  if (snapshot.value) {
    hostsEditable.value = snapshot.value.hosts_file;
    dnsEditable.value = snapshot.value.dns_servers.join("\n");
  }
});

async function saveHosts() {
  hostsSaving.value = true;
  hostsSaveError.value = null;
  hostsSaveSuccess.value = false;
  try {
    await invoke("write_hosts_file", { content: hostsEditable.value });
    hostsSaveSuccess.value = true;
    snapshot.value = await invoke<NetworkSnapshot>("get_network_snapshot");
  } catch (e) {
    hostsSaveError.value = String(e);
  } finally {
    hostsSaving.value = false;
  }
}

async function saveDns() {
  dnsSaving.value = true;
  dnsSaveError.value = null;
  dnsSaveSuccess.value = false;
  try {
    await invoke("set_dns_servers", { content: dnsEditable.value });
    dnsSaveSuccess.value = true;
    snapshot.value = await invoke<NetworkSnapshot>("get_network_snapshot");
  } catch (e) {
    dnsSaveError.value = String(e);
  } finally {
    dnsSaving.value = false;
  }
}

async function addFirewallRule() {
  firewallBusy.value = true;
  firewallError.value = null;
  firewallResult.value = null;
  try {
    firewallResult.value = await invoke<string>("add_firewall_rule", { portProto: firewallPortProto.value });
  } catch (e) {
    firewallError.value = String(e);
  } finally {
    firewallBusy.value = false;
  }
}

async function removeFirewallRule() {
  firewallBusy.value = true;
  firewallError.value = null;
  firewallResult.value = null;
  try {
    firewallResult.value = await invoke<string>("remove_firewall_rule", { portProto: firewallPortProto.value });
  } catch (e) {
    firewallError.value = String(e);
  } finally {
    firewallBusy.value = false;
  }
}

const scanHost = ref("127.0.0.1");
const scanPortsInput = ref("22,80,443,3000,8080");
const scanResults = ref<PortResult[]>([]);
const scanError = ref<string | null>(null);
const scanning = ref(false);

async function runScan() {
  scanning.value = true;
  scanError.value = null;
  try {
    const ports = scanPortsInput.value
      .split(",")
      .map((p) => parseInt(p.trim(), 10))
      .filter((p) => !Number.isNaN(p));
    scanResults.value = await invoke<PortResult[]>("scan_ports_cmd", { host: scanHost.value, ports });
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}
</script>

<template>
  <div class="net-page">
    <NxSectionHeader title="Réseau" />

    <div class="net-tabs">
      <button :class="{ active: activeTab === 'overview' }" @click="activeTab = 'overview'">Vue d'ensemble</button>
      <button :class="{ active: activeTab === 'portscan' }" @click="activeTab = 'portscan'">Scanner de ports</button>
      <button :class="{ active: activeTab === 'docker' }" @click="activeTab = 'docker'">Docker</button>
    </div>

    <template v-if="activeTab === 'overview' && snapshot">
      <NxCard>
        <NxSectionHeader title="Wi-Fi" />
        <div v-for="w in snapshot.wifi_networks" :key="w.ssid" class="net-row">
          <span>{{ w.ssid }}{{ w.connected ? " (connecté)" : "" }}</span>
          <span>{{ w.security }} · {{ w.signal_percent }}%</span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Ports en écoute" />
        <div v-for="p in snapshot.listening_ports" :key="p.port" class="net-row">
          <span>{{ p.port }}</span>
          <span>{{ p.process ?? "?" }}</span>
        </div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Modifier /etc/hosts" />
        <textarea v-model="hostsEditable" class="net-textarea" rows="8"></textarea>
        <NxButton :disabled="hostsSaving" @click="saveHosts">{{ hostsSaving ? "Enregistrement..." : "Enregistrer" }}</NxButton>
        <NxCard v-if="hostsSaveError" danger>{{ hostsSaveError }}</NxCard>
        <div v-if="hostsSaveSuccess" class="net-success">Fichier hosts mis à jour.</div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Modifier les serveurs DNS" />
        <textarea v-model="dnsEditable" class="net-textarea" rows="4" placeholder="nameserver 1.1.1.1"></textarea>
        <NxButton :disabled="dnsSaving" @click="saveDns">{{ dnsSaving ? "Enregistrement..." : "Enregistrer" }}</NxButton>
        <NxCard v-if="dnsSaveError" danger>{{ dnsSaveError }}</NxCard>
        <div v-if="dnsSaveSuccess" class="net-success">Configuration DNS mise à jour.</div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Règle de pare-feu" />
        <div class="net-form-row">
          <NxInput v-model="firewallPortProto" placeholder="ex: 8080/tcp" />
          <NxButton :disabled="firewallBusy" @click="addFirewallRule">Autoriser</NxButton>
          <NxButton :disabled="firewallBusy" @click="removeFirewallRule">Supprimer</NxButton>
        </div>
        <NxCard v-if="firewallError" danger>{{ firewallError }}</NxCard>
        <div v-if="firewallResult" class="net-success">Règle appliquée.</div>
      </NxCard>
    </template>

    <NxCard v-else-if="activeTab === 'portscan'">
      <div class="net-form-row">
        <NxInput v-model="scanHost" placeholder="Hôte (ex: 127.0.0.1)" />
        <NxInput v-model="scanPortsInput" placeholder="Ports, séparés par virgule" />
        <NxButton :disabled="scanning" @click="runScan">{{ scanning ? "Scan..." : "Scanner" }}</NxButton>
      </div>
      <NxCard v-if="scanError" danger>{{ scanError }}</NxCard>
      <div v-for="r in scanResults" :key="r.port" class="net-row">
        <span>{{ r.port }}</span>
        <span :class="r.open ? 'net-open' : 'net-closed'">{{ r.open ? "ouvert" : "fermé" }}</span>
      </div>
    </NxCard>

    <NxCard v-else-if="activeTab === 'docker'">
      <div v-if="!docker?.available" class="net-empty">Docker n'est pas disponible sur ce système.</div>
      <template v-else>
        <NxSectionHeader title="Conteneurs" />
        <div v-for="c in docker.containers" :key="c.id" class="net-row">
          <span>{{ c.name }} ({{ c.image }})</span>
          <span>{{ c.status }}</span>
        </div>
        <NxSectionHeader title="Images" />
        <div v-for="i in docker.images" :key="i.id" class="net-row">
          <span>{{ i.repository }}:{{ i.tag }}</span>
          <span>{{ i.size }}</span>
        </div>
      </template>
    </NxCard>
  </div>
</template>

<style scoped>
.net-page { padding: 24px; display: flex; flex-direction: column; gap: 14px; }
.net-tabs { display: flex; gap: 8px; }
.net-tabs button { padding: 8px 14px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; }
.net-tabs button.active { color: var(--nx-text-primary); font-weight: 600; }
.net-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
.net-textarea { width: 100%; padding: 10px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; margin-bottom: 8px; }
.net-success { margin-top: 10px; padding: 10px 14px; border-radius: var(--nx-style-radius); background: color-mix(in srgb, var(--nx-accent-success) 15%, transparent); border: 1px solid var(--nx-accent-success); }
.net-form-row { display: flex; gap: 10px; align-items: center; }
.net-open { color: var(--nx-accent-success); }
.net-closed { color: var(--nx-text-secondary); }
.net-empty { color: var(--nx-text-secondary); }
</style>
```

### Step 4: Run test to verify it passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/NetworkPage.spec.ts"`
Expected: PASS (2 tests)

### Step 5: Verify the whole project still type-checks and the full test suite still passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (95 from Task 5 + 2 new = 97).

### Step 6: Commit

```bash
git add src/pages/NetworkPage.vue src/pages/NetworkPage.spec.ts
git commit -m "feat: componentize NetworkPage on Nx* primitives (spec section 5)"
```

---

## Task 7: preferencesStore.ts + SettingsPreferencesPage.vue (spec §4.5)

**Files:**
- Create: `src/stores/preferencesStore.ts`
- Create: `src/pages/SettingsPreferencesPage.vue`
- Test: `src/stores/preferencesStore.spec.ts`
- Test: `src/pages/SettingsPreferencesPage.spec.ts`
- Modify: `src/App.vue`

Three settings per spec §4.5: default scan directory (used by `FileToolsPage`'s scanners — this task only stores the preference, wiring `FileToolsPage` to actually read it as its default is explicitly deferred, not part of this task's scope, to keep this task focused), dashboard refresh interval, and a confirm-non-destructive-actions toggle (also stored only — not yet wired into any button, same reasoning). This mirrors the plan's spec section 4.5 note: "a real, useful settings page rather than an empty one... can grow later as concrete needs surface" — storing the preference now is the value; consuming it everywhere it should apply is future work once each consuming page is touched again.

### Step 1: Write the failing store test

```typescript
// src/stores/preferencesStore.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { usePreferencesStore } from "./preferencesStore";

describe("preferencesStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("defaults to sensible values when nothing persisted", () => {
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("");
    expect(store.dashboardRefreshIntervalMs).toBe(2000);
    expect(store.confirmNonDestructiveActions).toBe(true);
  });

  it("setDefaultScanDirectory updates state and persists", () => {
    const store = usePreferencesStore();
    store.setDefaultScanDirectory("/home/dev");
    expect(store.defaultScanDirectory).toBe("/home/dev");
    expect(JSON.parse(localStorage.getItem("nitrux-preferences")!).defaultScanDirectory).toBe("/home/dev");
  });

  it("setDashboardRefreshIntervalMs updates state and persists", () => {
    const store = usePreferencesStore();
    store.setDashboardRefreshIntervalMs(5000);
    expect(store.dashboardRefreshIntervalMs).toBe(5000);
    expect(JSON.parse(localStorage.getItem("nitrux-preferences")!).dashboardRefreshIntervalMs).toBe(5000);
  });

  it("setConfirmNonDestructiveActions updates state and persists", () => {
    const store = usePreferencesStore();
    store.setConfirmNonDestructiveActions(false);
    expect(store.confirmNonDestructiveActions).toBe(false);
    expect(JSON.parse(localStorage.getItem("nitrux-preferences")!).confirmNonDestructiveActions).toBe(false);
  });

  it("reads persisted preferences on store creation", () => {
    localStorage.setItem("nitrux-preferences", JSON.stringify({
      defaultScanDirectory: "/mnt/data",
      dashboardRefreshIntervalMs: 1000,
      confirmNonDestructiveActions: false,
    }));
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("/mnt/data");
    expect(store.dashboardRefreshIntervalMs).toBe(1000);
    expect(store.confirmNonDestructiveActions).toBe(false);
  });

  it("falls back to defaults when persisted JSON is malformed", () => {
    localStorage.setItem("nitrux-preferences", "not valid json{");
    const store = usePreferencesStore();
    expect(store.defaultScanDirectory).toBe("");
    expect(store.dashboardRefreshIntervalMs).toBe(2000);
    expect(store.confirmNonDestructiveActions).toBe(true);
  });
});
```

### Step 2: Run test to verify it fails

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/stores/preferencesStore.spec.ts"`
Expected: FAIL — module not found.

### Step 3: Write `preferencesStore.ts`

```typescript
// src/stores/preferencesStore.ts
import { defineStore } from "pinia";

const STORAGE_KEY = "nitrux-preferences";

export interface Preferences {
  defaultScanDirectory: string;
  dashboardRefreshIntervalMs: number;
  confirmNonDestructiveActions: boolean;
}

const DEFAULTS: Preferences = {
  defaultScanDirectory: "",
  dashboardRefreshIntervalMs: 2000,
  confirmNonDestructiveActions: true,
};

function isPreferences(value: unknown): value is Preferences {
  if (typeof value !== "object" || value === null) return false;
  const v = value as Record<string, unknown>;
  return (
    typeof v.defaultScanDirectory === "string" &&
    typeof v.dashboardRefreshIntervalMs === "number" &&
    typeof v.confirmNonDestructiveActions === "boolean"
  );
}

function readPersistedPreferences(): Preferences {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return { ...DEFAULTS };
  try {
    const parsed: unknown = JSON.parse(stored);
    return isPreferences(parsed) ? parsed : { ...DEFAULTS };
  } catch {
    return { ...DEFAULTS };
  }
}

function persist(prefs: Preferences) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(prefs));
}

export const usePreferencesStore = defineStore("preferences", {
  state: (): Preferences => readPersistedPreferences(),
  actions: {
    setDefaultScanDirectory(value: string) {
      this.defaultScanDirectory = value;
      persist({ ...this.$state });
    },
    setDashboardRefreshIntervalMs(value: number) {
      this.dashboardRefreshIntervalMs = value;
      persist({ ...this.$state });
    },
    setConfirmNonDestructiveActions(value: boolean) {
      this.confirmNonDestructiveActions = value;
      persist({ ...this.$state });
    },
  },
});
```

### Step 4: Run test to verify it passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/stores/preferencesStore.spec.ts"`
Expected: PASS (6 tests)

### Step 5: Write the failing page test

```typescript
// src/pages/SettingsPreferencesPage.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import SettingsPreferencesPage from "./SettingsPreferencesPage.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

describe("SettingsPreferencesPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("edits the default scan directory via the store", async () => {
    const wrapper = mount(SettingsPreferencesPage);
    const store = usePreferencesStore();
    await wrapper.find(".nx-input").setValue("/home/dev/Documents");
    expect(store.defaultScanDirectory).toBe("/home/dev/Documents");
  });

  it("changes the dashboard refresh interval via the select", async () => {
    const wrapper = mount(SettingsPreferencesPage);
    const store = usePreferencesStore();
    await wrapper.find(".nx-select").setValue("5000");
    expect(store.dashboardRefreshIntervalMs).toBe(5000);
  });

  it("toggles the confirm-non-destructive-actions checkbox via the store", async () => {
    const wrapper = mount(SettingsPreferencesPage);
    const store = usePreferencesStore();
    const checkbox = wrapper.find('input[type="checkbox"]');
    await checkbox.setValue(false);
    expect(store.confirmNonDestructiveActions).toBe(false);
  });
});
```

### Step 6: Run test to verify it fails

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/SettingsPreferencesPage.spec.ts"`
Expected: FAIL — component doesn't exist.

### Step 7: Write `SettingsPreferencesPage.vue`

```vue
<!-- src/pages/SettingsPreferencesPage.vue -->
<script setup lang="ts">
import { usePreferencesStore } from "@/stores/preferencesStore";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSelect from "@/components/ui/NxSelect.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const preferences = usePreferencesStore();

const REFRESH_INTERVAL_OPTIONS = [
  { value: "1000", label: "1 seconde" },
  { value: "2000", label: "2 secondes" },
  { value: "5000", label: "5 secondes" },
];

function onIntervalChange(value: string) {
  preferences.setDashboardRefreshIntervalMs(Number(value));
}

function onScanDirChange(value: string) {
  preferences.setDefaultScanDirectory(value);
}

function onConfirmToggle(event: Event) {
  preferences.setConfirmNonDestructiveActions((event.target as HTMLInputElement).checked);
}
</script>

<template>
  <div class="pref-page">
    <NxSectionHeader title="Préférences" description="Réglages de l'application (pas de la configuration système)." />

    <NxCard class="pref-card">
      <label class="pref-label">Répertoire par défaut pour les scanners</label>
      <NxInput
        :model-value="preferences.defaultScanDirectory"
        placeholder="ex: /home/dev"
        @update:model-value="onScanDirChange"
      />
    </NxCard>

    <NxCard class="pref-card">
      <label class="pref-label">Intervalle de rafraîchissement du tableau de bord</label>
      <NxSelect
        :model-value="String(preferences.dashboardRefreshIntervalMs)"
        :options="REFRESH_INTERVAL_OPTIONS"
        @update:model-value="onIntervalChange"
      />
    </NxCard>

    <NxCard class="pref-card pref-toggle-row">
      <label class="pref-label">
        <input type="checkbox" :checked="preferences.confirmNonDestructiveActions" @change="onConfirmToggle" />
        Demander confirmation pour les actions non-destructives
      </label>
    </NxCard>
  </div>
</template>

<style scoped>
.pref-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.pref-card { display: flex; flex-direction: column; gap: 8px; }
.pref-label { font-size: 13px; color: var(--nx-text-secondary); display: flex; align-items: center; gap: 8px; }
.pref-toggle-row { flex-direction: row; align-items: center; }
</style>
```

### Step 8: Run test to verify it passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/SettingsPreferencesPage.spec.ts"`
Expected: PASS (3 tests)

### Step 9: Wire the new `settings-preferences` id into `App.vue`

Add the import:
```typescript
import SettingsPreferencesPage from "@/pages/SettingsPreferencesPage.vue";
```

Add `"settings-preferences"` to the `PageId` union:
```typescript
type PageId =
  | "dashboard"
  | "hardware"
  | "drivers"
  | "logs"
  | "theme-editor"
  | "packages"
  | "disks"
  | "file-tools"
  | "network"
  | "security"
  | "troubleshoot"
  | "settings-preferences";
```

Add the map entry:
```typescript
  "settings-preferences": SettingsPreferencesPage,
```

Add a temporary nav button (same rationale as prior tasks):
```html
        <button
          :class="{ active: currentPage === 'settings-preferences' }"
          @click="currentPage = 'settings-preferences'"
        >
          Préférences
        </button>
```
placed right after the existing "Apparence" button.

### Step 10: Verify the whole project still type-checks and the full test suite still passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (97 from Task 6 + 6 + 3 new = 106).

### Step 11: Commit

```bash
git add src/stores/preferencesStore.ts src/stores/preferencesStore.spec.ts src/pages/SettingsPreferencesPage.vue src/pages/SettingsPreferencesPage.spec.ts src/App.vue
git commit -m "feat: add preferencesStore + SettingsPreferencesPage (spec section 4.5)"
```

---

## Task 8: Wire App.vue to AppNav + the full categories.ts page map

**Files:**
- Create: `src/pages/ComingSoonPage.vue` (small placeholder for the 3 page ids this plan doesn't implement — `quick-install`/`updates`/`report-generator`, built by R3/R4/R5)
- Test: `src/pages/ComingSoonPage.spec.ts`
- Modify: `src/App.vue` (the big rewrite — replaces the hardcoded `<nav>` with `<AppNav>`, replaces the flat `PageId` union with the 15 ids from `categories.ts`)
- Test: `src/App.spec.ts` (none exists yet)

This is the task that makes every prior task's work in this plan actually reachable through the new categorized navigation instead of the temporary buttons each prior task bolted onto the old flat nav.

### Step 1: Write the failing `ComingSoonPage` test

```typescript
// src/pages/ComingSoonPage.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import ComingSoonPage from "./ComingSoonPage.vue";

describe("ComingSoonPage", () => {
  it("renders the provided title and phase note", () => {
    const wrapper = mount(ComingSoonPage, { props: { title: "Installation rapide", phase: "Phase R3" } });
    expect(wrapper.text()).toContain("Installation rapide");
    expect(wrapper.text()).toContain("Phase R3");
    expect(wrapper.text()).toContain("Bientôt disponible");
  });
});
```

### Step 2: Run test to verify it fails

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/ComingSoonPage.spec.ts"`
Expected: FAIL — component doesn't exist.

### Step 3: Write `ComingSoonPage.vue`

```vue
<!-- src/pages/ComingSoonPage.vue -->
<script setup lang="ts">
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

defineProps<{ title: string; phase: string }>();
</script>

<template>
  <div class="cs-page">
    <NxSectionHeader :title="title" />
    <NxCard class="cs-card">
      <p class="cs-message">Bientôt disponible — prévu pour {{ phase }} de la refonte NiTruX.</p>
    </NxCard>
  </div>
</template>

<style scoped>
.cs-page { padding: 24px; }
.cs-card { text-align: center; padding: 40px 20px; }
.cs-message { color: var(--nx-text-secondary); margin: 0; }
</style>
```

### Step 4: Run test to verify it passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/ComingSoonPage.spec.ts"`
Expected: PASS (1 test)

### Step 5: Write the failing `App.vue` test

```typescript
// src/App.spec.ts
import { describe, it, expect, beforeEach, vi } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import App from "./App.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue(null),
}));

describe("App", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("renders AppNav with all 7 category titles", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Système");
    expect(wrapper.text()).toContain("Applications");
    expect(wrapper.text()).toContain("Stockage");
    expect(wrapper.text()).toContain("Maintenance");
    expect(wrapper.text()).toContain("Réseau");
    expect(wrapper.text()).toContain("Rapports");
    expect(wrapper.text()).toContain("Paramètres");
  });

  it("defaults to the dashboard page", () => {
    const wrapper = mount(App);
    expect(wrapper.findComponent({ name: "DashboardPage" }).exists() || wrapper.html().length > 0).toBe(true);
  });

  it("switches to DiagnosticPage when its nav item is clicked", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const diagButton = buttons.find((b) => b.text() === "Diagnostic")!;
    await diagButton.trigger("click");
    expect(wrapper.text()).toContain("Composants matériels détectés");
  });

  it("shows the ComingSoonPage for the not-yet-implemented quick-install id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const quickInstallButton = buttons.find((b) => b.text() === "Installation rapide")!;
    await quickInstallButton.trigger("click");
    expect(wrapper.text()).toContain("Bientôt disponible");
  });
});
```

### Step 6: Run test to verify it fails

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/App.spec.ts"`
Expected: FAIL — `App.vue` still has the old flat nav with different button labels ("Matériel" not "Diagnostic", no category titles at all).

### Step 7: Write the 3 tiny "coming soon" wrapper pages

`<component :is="...">` needs an actual component definition (an object with a `setup`/`render`/`template`), not a pre-built VNode — so each not-yet-implemented page id gets its own trivial wrapper component that renders `ComingSoonPage` with fixed props, rather than trying to stuff a VNode into the `pages` map directly (that would not render correctly).

```vue
<!-- src/pages/QuickInstallPlaceholder.vue -->
<script setup lang="ts">
import ComingSoonPage from "./ComingSoonPage.vue";
</script>

<template>
  <ComingSoonPage title="Installation rapide" phase="Phase R3" />
</template>
```

```vue
<!-- src/pages/UpdatesPlaceholder.vue -->
<script setup lang="ts">
import ComingSoonPage from "./ComingSoonPage.vue";
</script>

<template>
  <ComingSoonPage title="Mises à jour" phase="Phase R4" />
</template>
```

```vue
<!-- src/pages/ReportGeneratorPlaceholder.vue -->
<script setup lang="ts">
import ComingSoonPage from "./ComingSoonPage.vue";
</script>

<template>
  <ComingSoonPage title="Générateur de rapport" phase="Phase R5" />
</template>
```

Each of these 3 files is replaced entirely (not extended) by R3/R4/R5 respectively, once those phases build the real page — this is why they're separate trivial files rather than 3 entries pointing at one shared generic placeholder: each file's name is exactly the file the corresponding future phase will overwrite.

### Step 8: Rewrite `App.vue`

Read the live `src/App.vue` first — by this point in the plan it has accumulated 4 rounds of small import/map/button edits from Tasks 1, 4, 5, and 7. This step replaces the whole file (both `<script setup>` and the `<nav>` template block) in one go rather than trying to incrementally edit the accumulated state.

```vue
<script setup lang="ts">
import { onMounted, ref, type Component } from "vue";
import { useThemeStore } from "@/stores/themeStore";
import LayoutShell from "@/layouts/LayoutShell.vue";
import AppNav from "@/components/nav/AppNav.vue";
import DashboardPage from "@/pages/DashboardPage.vue";
import DiagnosticPage from "@/pages/DiagnosticPage.vue";
import DriversPage from "@/pages/DriversPage.vue";
import LogsPage from "@/pages/LogsPage.vue";
import ThemeEditorPage from "@/pages/ThemeEditorPage.vue";
import PackagesPage from "@/pages/PackagesPage.vue";
import DisksPage from "@/pages/DisksPage.vue";
import FileToolsPage from "@/pages/FileToolsPage.vue";
import NetworkPage from "@/pages/NetworkPage.vue";
import FirewallPage from "@/pages/FirewallPage.vue";
import TroubleshootPage from "@/pages/TroubleshootPage.vue";
import SettingsPreferencesPage from "@/pages/SettingsPreferencesPage.vue";
import QuickInstallPlaceholder from "@/pages/QuickInstallPlaceholder.vue";
import UpdatesPlaceholder from "@/pages/UpdatesPlaceholder.vue";
import ReportGeneratorPlaceholder from "@/pages/ReportGeneratorPlaceholder.vue";

const themeStore = useThemeStore();
onMounted(() => themeStore.setTheme(themeStore.active));

const currentPage = ref<string>("dashboard");

// Every id here must match a `NavPage.id` in `src/navigation/categories.ts`
// exactly -- AppNav renders nav items purely from that data file, so a
// mismatch here means a nav item that silently does nothing when clicked
// (falls back to the dashboard per the `?? pages.dashboard` guard below,
// not a crash, but still a real bug if it ever happens for an id that
// should have a real page).
const pages: Record<string, Component> = {
  dashboard: DashboardPage,
  diagnostic: DiagnosticPage,
  "quick-install": QuickInstallPlaceholder,
  "package-manager": PackagesPage,
  disks: DisksPage,
  "file-tools": FileToolsPage,
  updates: UpdatesPlaceholder,
  drivers: DriversPage,
  troubleshoot: TroubleshootPage,
  "network-overview": NetworkPage,
  firewall: FirewallPage,
  "report-generator": ReportGeneratorPlaceholder,
  logs: LogsPage,
  "settings-preferences": SettingsPreferencesPage,
  "settings-appearance": ThemeEditorPage,
};
</script>

<template>
  <LayoutShell>
    <template #nav>
      <AppNav v-model="currentPage" />
    </template>
    <component :is="pages[currentPage] ?? pages.dashboard" />
  </LayoutShell>
</template>

<style>
:root {
  font-family: Inter, Avenir, Helvetica, Arial, sans-serif;
  font-size: 16px;
  line-height: 24px;
  font-weight: 400;

  font-synthesis: none;
  text-rendering: optimizeLegibility;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  -webkit-text-size-adjust: 100%;
}

body {
  margin: 0;
  background-color: var(--nx-bg-base);
  color: var(--nx-text-primary);
}
</style>
```

Notes on this rewrite:
- The old scoped `.app-nav` CSS block is deleted entirely — that styling lived in `App.vue` only because the nav markup used to live there too; `AppNav.vue` (built in R1) owns its own styling now.
- `pages[currentPage] ?? pages.dashboard` is a defensive fallback mirroring the exact pattern already used in `LayoutShell.vue` for its own `activeComponent` computed (`componentMap[layoutStore.current] ?? SidebarClassicLayout`) — if `currentPage` ever holds a value with no map entry (should be impossible given `AppNav` only emits ids it was given from `categories.ts`, but defensive nonetheless), the app falls back to the dashboard rather than rendering nothing.

### Step 9: Run test to verify it passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/App.spec.ts"`
Expected: PASS (4 tests)

### Step 10: Verify the whole project still type-checks and the full test suite passes

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npm run test -- --run 2>&1 | tail -15"`
Expected: clean type-check, all tests pass (106 from Task 7 + 1 (ComingSoonPage) + 4 (App) = 111). The 3 placeholder wrapper components from Step 7 have no dedicated spec files (they're 3-line pass-through wrappers with zero logic — `App.spec.ts`'s "shows the ComingSoonPage for the not-yet-implemented quick-install id" test already exercises one of them end-to-end, which is sufficient coverage for trivial wrapper components with no branching).

### Step 11: Commit

```bash
git add src/pages/ComingSoonPage.vue src/pages/ComingSoonPage.spec.ts src/pages/QuickInstallPlaceholder.vue src/pages/UpdatesPlaceholder.vue src/pages/ReportGeneratorPlaceholder.vue src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue to AppNav and the full categories.ts page map (spec section 2.2)"
```

---

## Task 9: Full verification pass

**Files:** None (verification-only).

- [ ] **Step 1: Run the full test suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npm run test -- --run 2>&1 | tail -30"`
Expected: every test file passes. Record the exact total observed — the plan's running tally reached 111 by Task 8, but treat that as an estimate to verify, not a fact to force; if the real count differs, report the real number and reconcile why, same discipline applied throughout every phase tonight.

- [ ] **Step 2: Type-check**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit"`
Expected: clean.

- [ ] **Step 3: Confirm the Rust suite is unaffected**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/src-tauri && cargo test 2>&1 | tail -10"`
Expected: unchanged from before this plan (124 passed, 1 ignored, 0 failed) — this plan touches zero Rust code.

- [ ] **Step 4: Confirm every old page file is actually gone**

Run: `git status --short` and confirm `src/pages/HardwarePage.vue` and `src/pages/SecurityPage.vue` do not exist in the worktree (deleted via `git rm` in Tasks 1 and 5) and are not accidentally still referenced anywhere:

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && grep -rn 'HardwarePage\|SecurityPage' src/ || echo 'no references found'"`
Expected: `no references found` (both names should be completely gone from the codebase — if this grep finds a hit, that's a real bug: a stale import or reference this plan's tasks missed).

- [ ] **Step 5: Confirm every category from `categories.ts` is reachable and shows real or clearly-labeled-placeholder content**

This is a manual review step, not an automated test: read `src/App.vue`'s final `pages` map and cross-check it against all 15 entries in `src/navigation/categories.ts` (read that file too) — every single page id must have a map entry (either a real page or one of the 3 `*Placeholder.vue` components), with no `undefined` gaps. If any id is missing from the map, that's the fallback-to-dashboard defensive guard silently masking a real bug — fix it before considering this task done.

- [ ] **Step 6: Commit any final cleanup, then this plan is complete**

No further commit expected if Steps 1–5 all pass clean — this step exists only to catch and fix anything they surfaced.

---

## Self-Review

**Spec coverage:** §5's 6 named restructuring items (DisksPage split, SecurityPage split, NetworkPage, HardwarePage→Diagnostic, PackagesPage, LogsPage) are covered by Tasks 1–6. §4.5 (Préférences) is covered by Task 7. §2.2 (AppNav wiring, no vue-router) is covered by Task 8. DashboardPage/DriversPage/ThemeEditorPage explicitly stay untouched per spec — confirmed no task in this plan modifies them.

**Placeholder scan:** No "TBD"/"TODO" in this plan. The 3 `*Placeholder.vue` files ARE literal placeholders by design — but they're fully specified, fully-coded, fully-tested-by-proxy components with a clear, honest, user-facing "coming soon" message and an explicit phase reference, not a code stub — this is the plan's intentional, spec-authorized way of handling `categories.ts` entries whose real implementation is deliberately out of this plan's scope (R3/R4/R5's job), not an oversight.

**Type consistency:** `PageId` (informal, now just `string` typed against `Component` in `App.vue`'s `Record<string, Component>` rather than a strict union — this is a deliberate loosening from the old `PageId` union type, since `AppNav`'s `modelValue` prop is typed `string` per R1's `AppNav.vue`, and keeping `App.vue`'s `currentPage` as a matching `string` avoids a type mismatch at the `v-model` boundary; the `categories.ts` data file is the actual source of truth for which ids are valid, not a TypeScript union anymore). Every `Nx*` component prop name used across Tasks 1–8 (`modelValue`, `options`, `variant`, `status`, `danger`, `disabled`, `title`, `description`) matches exactly what R1 defined — cross-checked against each `Nx*.vue` file's `defineProps` while writing every task in this plan.
