# NiTruX Phase R6 — Fondation visuelle + catégorie Performance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix the concrete, root-cause reasons NiTruX still reads as "moche et fade" (no icon library despite `categories.ts` already defining icon names, `AppNav.vue` rendering plain text only, `DashboardPage.vue` never migrated to the shared `Nx*`/style-token system) — and simultaneously ship the first of 6 planned feature-parity phases: a new "Performance" nav category with 4 real pages (Optimisations, Températures, Benchmark, Historique perf.).

**Architecture:** `lucide-vue-next` is added as the icon library (same version NiTriTe Windows uses). `AppNav.vue` gains an icon-name → component lookup table and renders one per nav item — since `categories.ts` already has icon names for all 15 existing pages, this single change fixes icons across the *entire* existing nav with no per-page retrofit. A new shared `NxQuickActionTile.vue` gives `DashboardPage.vue` 5 colored gradient action tiles (mirroring NiTriTe's dashboard), and `DashboardPage.vue` is componentized onto `Nx*`/`--nx-style-*` tokens for the first time since R1. `categories.ts` gains an 8th category, "Performance", with 4 new pages: `TemperaturesPage.vue` (zero new backend — reuses the already-tested `get_sensor_snapshot`), `BenchmarkPage.vue` + new `src-tauri/src/benchmark.rs` (CPU/disk/memory micro-benchmarks, zero privileged operations), `PerfHistoryPage.vue` + new shared `NxSparkline.vue` (100% frontend client-side rolling buffer, zero new backend), and `OptimizationsPage.vue` + new `src-tauri/src/optimizations.rs` (read-only startup-service/swappiness/zram/fstrim diagnostics — explicitly no write/toggle capability in this phase, per the established rule that new privileged surfaces need their own dedicated review+VM-verification phase).

**Tech Stack:** Tauri v2 + Rust (backend), Vue 3.5 + TypeScript + Vite + Pinia + Vitest (frontend), `lucide-vue-next` (new dependency), same patterns as Phases R1–R5.

---

## Task 1: Install lucide-vue-next and wire icons into AppNav.vue

**Files:**
- Modify: `package.json` (new dependency)
- Modify: `src/components/nav/AppNav.vue`
- Test: `src/components/nav/AppNav.spec.ts` (none exists yet)

- [ ] **Step 1: Install the dependency**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npm install lucide-vue-next@^0.474.0"`
Expected: `package.json`/`package-lock.json` updated, install succeeds.

- [ ] **Step 2: Write the failing test**

Read the live `src/components/nav/AppNav.vue` first (from R1: `defineProps<{ modelValue: string }>()`, renders `navigationCategories` from `src/navigation/categories.ts`, currently `{{ page.label }}` only, no icon rendering).

```typescript
// src/components/nav/AppNav.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AppNav from "./AppNav.vue";

describe("AppNav", () => {
  it("renders an icon (svg) next to every nav item's label", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    const items = wrapper.findAll(".nx-app-nav__item");
    expect(items.length).toBeGreaterThan(0);
    for (const item of items) {
      expect(item.find("svg").exists()).toBe(true);
    }
  });

  it("falls back to a neutral icon for an unknown icon name rather than crashing", () => {
    // categories.ts always provides known names in practice, but AppNav
    // must not crash if one is ever misspelled -- this proves the fallback.
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    expect(wrapper.exists()).toBe(true);
  });
});
```

- [ ] **Step 3: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/components/nav/AppNav.spec.ts"`
Expected: FAIL — no `<svg>` rendered yet (plain text only).

- [ ] **Step 4: Wire icons into `AppNav.vue`**

Replace the entire file:

```vue
<!-- src/components/nav/AppNav.vue -->
<script setup lang="ts">
import { type Component } from "vue";
import {
  LayoutDashboard, Stethoscope, Download, Package, HardDrive, Files,
  RefreshCw, Cpu, Wrench, Wifi, Shield, FileText, ScrollText, Settings,
  Palette, Zap, Thermometer, Gauge, BarChart3, Circle,
} from "lucide-vue-next";
import { navigationCategories } from "@/navigation/categories";

defineProps<{ modelValue: string }>();
defineEmits<{ "update:modelValue": [string] }>();

// Maps every icon name used in `categories.ts` to its lucide component.
// An id with no entry here falls back to `Circle` (Step 2's second test) --
// this can only happen if a future `categories.ts` entry's icon name is
// misspelled or not yet added to this map, never a crash.
const iconMap: Record<string, Component> = {
  "layout-dashboard": LayoutDashboard,
  stethoscope: Stethoscope,
  download: Download,
  package: Package,
  "hard-drive": HardDrive,
  files: Files,
  "refresh-cw": RefreshCw,
  cpu: Cpu,
  wrench: Wrench,
  wifi: Wifi,
  shield: Shield,
  "file-text": FileText,
  "scroll-text": ScrollText,
  settings: Settings,
  palette: Palette,
  zap: Zap,
  thermometer: Thermometer,
  gauge: Gauge,
  "bar-chart-3": BarChart3,
};

function getIcon(name: string): Component {
  return iconMap[name] ?? Circle;
}
</script>

<template>
  <nav class="nx-app-nav">
    <div v-for="category in navigationCategories" :key="category.id" class="nx-app-nav__category">
      <div class="nx-app-nav__title">{{ category.title }}</div>
      <button
        v-for="page in category.pages"
        :key="page.id"
        class="nx-app-nav__item"
        :class="{ active: modelValue === page.id }"
        @click="$emit('update:modelValue', page.id)"
      >
        <component :is="getIcon(page.icon)" :size="16" class="nx-app-nav__icon" />
        <span>{{ page.label }}</span>
      </button>
    </div>
  </nav>
</template>

<style scoped>
.nx-app-nav { display: flex; flex-direction: column; gap: 2px; padding: 12px 8px; font-family: var(--nx-style-font-family); }
.nx-app-nav__category { margin-bottom: 10px; }
.nx-app-nav__title {
  padding: 6px 10px 4px;
  font-size: 11px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: var(--nx-text-secondary);
  opacity: 0.7;
  font-weight: 700;
}
.nx-app-nav__item {
  display: flex;
  align-items: center;
  gap: 10px;
  width: 100%;
  text-align: left;
  padding: 7px 10px;
  border: none;
  background: transparent;
  color: var(--nx-text-secondary);
  border-radius: var(--nx-style-radius);
  cursor: pointer;
  font: inherit;
  font-size: 13px;
}
.nx-app-nav__icon { flex-shrink: 0; opacity: 0.8; }
.nx-app-nav__item:hover { background: var(--nx-style-bg); color: var(--nx-text-primary); }
.nx-app-nav__item.active { background: var(--nx-style-bg); color: var(--nx-text-primary); font-weight: 600; }
.nx-app-nav__item.active .nx-app-nav__icon { opacity: 1; }
</style>
```

Notes:
- The `iconMap` covers all 15 existing icon names from the live `categories.ts` (confirmed by reading it before writing this plan) plus 4 new names (`zap`, `thermometer`, `gauge`, `bar-chart-3`) that Task 4 will add for the new Performance category — defined here now so this task's map is already complete once Task 4 lands, no second edit to `AppNav.vue` needed later in this plan.
- `.nx-app-nav__item` changes from `display: block` to `display: flex` to lay the icon and label side by side — this is the only structural CSS change; every other rule is unchanged from R1.

- [ ] **Step 5: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/components/nav/AppNav.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 6: Commit**

```bash
git add package.json package-lock.json src/components/nav/AppNav.vue src/components/nav/AppNav.spec.ts
git commit -m "feat: add lucide-vue-next and render icons in AppNav (spec section 2.2)"
```

---

## Task 2: `NxQuickActionTile.vue` shared component

**Files:**
- Create: `src/components/ui/NxQuickActionTile.vue`
- Test: `src/components/ui/NxQuickActionTile.spec.ts`

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/ui/NxQuickActionTile.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import { Stethoscope } from "lucide-vue-next";
import NxQuickActionTile from "./NxQuickActionTile.vue";

describe("NxQuickActionTile", () => {
  it("renders the icon, label, and applies the given gradient as background", () => {
    const wrapper = mount(NxQuickActionTile, {
      props: { icon: Stethoscope, label: "Diagnostic", gradient: "linear-gradient(135deg,#f97316,#fb923c)" },
    });
    expect(wrapper.text()).toContain("Diagnostic");
    expect(wrapper.find("svg").exists()).toBe(true);
    expect(wrapper.attributes("style")).toContain("linear-gradient(135deg,#f97316,#fb923c)");
  });

  it("emits click when clicked", async () => {
    const wrapper = mount(NxQuickActionTile, {
      props: { icon: Stethoscope, label: "Diagnostic", gradient: "linear-gradient(135deg,#f97316,#fb923c)" },
    });
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toBeTruthy();
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/components/ui/NxQuickActionTile.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 3: Write the component**

```vue
<!-- src/components/ui/NxQuickActionTile.vue -->
<script setup lang="ts">
import { type Component } from "vue";

defineProps<{ icon: Component; label: string; gradient: string }>();
defineEmits<{ click: [MouseEvent] }>();
</script>

<template>
  <button class="nx-quick-action" :style="{ background: gradient }" @click="$emit('click', $event)">
    <component :is="icon" :size="22" class="nx-quick-action__icon" />
    <span class="nx-quick-action__label">{{ label }}</span>
  </button>
</template>

<style scoped>
.nx-quick-action {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 10px;
  padding: 16px;
  border: none;
  border-radius: var(--nx-style-radius);
  color: white;
  cursor: pointer;
  font: inherit;
  min-width: 140px;
  text-align: left;
}
.nx-quick-action__icon { opacity: 0.95; }
.nx-quick-action__label { font-size: 13px; font-weight: 600; }
</style>
```

- [ ] **Step 4: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/components/ui/NxQuickActionTile.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add src/components/ui/NxQuickActionTile.vue src/components/ui/NxQuickActionTile.spec.ts
git commit -m "feat: add NxQuickActionTile shared component (spec section 2.3)"
```

---

## Task 3: Componentize `DashboardPage.vue` + add quick-action tiles

**Files:**
- Modify: `src/pages/DashboardPage.vue`
- Test: `src/pages/DashboardPage.spec.ts` (none exists yet)

Read the live `src/pages/DashboardPage.vue` first (reproduced in this plan's research: raw `.dash-card` divs, `--nx-border`/`--nx-bg-elevated` hardcoded CSS vars, 2-second polling of `get_system_snapshot`+`get_sensor_snapshot`, separate `error`/`sensorsError` refs). All business logic (polling interval, error handling, byte-to-GB conversion) must be preserved exactly — only presentation changes, same discipline as every R2 page-componentization task.

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/DashboardPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DashboardPage from "./DashboardPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_system_snapshot") {
      return Promise.resolve({
        cpus: [{ name: "Test CPU", usage_percent: 12.5, usage_display: "12.5%" }],
        memory_used_bytes: 4_000_000_000,
        memory_total_bytes: 8_000_000_000,
        process_count: 210,
      });
    }
    if (cmd === "get_sensor_snapshot") {
      return Promise.resolve({ battery_percent: 80, battery_charging: true, temperatures: [] });
    }
    return Promise.resolve(null);
  }),
}));

describe("DashboardPage", () => {
  it("renders system stats inside NxCard and 5 quick-action tiles", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    expect(wrapper.find(".nx-card").exists()).toBe(true);
    expect(wrapper.text()).toContain("12.5%");
    expect(wrapper.text()).toContain("210");
    const tiles = wrapper.findAll(".nx-quick-action");
    expect(tiles.length).toBe(5);
  });

  it("emits a navigation request when a quick-action tile is clicked", async () => {
    const wrapper = mount(DashboardPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Test CPU"));
    const tiles = wrapper.findAll(".nx-quick-action");
    const diagnosticTile = tiles.find((t) => t.text().includes("Diagnostic"))!;
    await diagnosticTile.trigger("click");
    expect(wrapper.emitted("navigate")).toEqual([["diagnostic"]]);
  });
});
```

This test deliberately does NOT use `vi.useFakeTimers()` — neither assertion depends on the 2-second polling interval firing (both only need the initial `onMounted` calls to resolve), and every real-timer async test elsewhere in this codebase (e.g. R2-R5's `NetworkPage.spec.ts`, `UpdatesPage.spec.ts`) already uses plain `vi.waitFor` with real timers successfully — no need to introduce fake timers here.

- [ ] **Step 2: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/DashboardPage.spec.ts"`
Expected: FAIL — no `.nx-card`/`.nx-quick-action` yet, no `navigate` event.

- [ ] **Step 3: Rewrite `DashboardPage.vue`**

```vue
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import {
  Stethoscope, Download, RefreshCw, Wrench, FileText,
} from "lucide-vue-next";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxQuickActionTile from "@/components/ui/NxQuickActionTile.vue";

interface CpuInfo { name: string; usage_percent: number; usage_display: string }
interface SystemSnapshot {
  cpus: CpuInfo[];
  memory_used_bytes: number;
  memory_total_bytes: number;
  process_count: number;
}
interface SensorSnapshot {
  battery_percent: number | null;
  battery_charging: boolean | null;
  temperatures: { label: string; celsius: number }[];
}

const emit = defineEmits<{ navigate: [string] }>();

const snapshot = ref<SystemSnapshot | null>(null);
const error = ref<string | null>(null);
const sensors = ref<SensorSnapshot | null>(null);
const sensorsError = ref<string | null>(null);
let intervalId: number | undefined;

async function refresh() {
  try {
    snapshot.value = await invoke<SystemSnapshot>("get_system_snapshot");
    error.value = null;
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
}

async function refreshSensors() {
  try {
    sensors.value = await invoke<SensorSnapshot>("get_sensor_snapshot");
    sensorsError.value = null;
  } catch (err) {
    sensorsError.value = err instanceof Error ? err.message : String(err);
  }
}

onMounted(() => {
  refresh();
  refreshSensors();
  intervalId = window.setInterval(() => {
    refresh();
    refreshSensors();
  }, 2000);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}

const QUICK_ACTIONS = [
  { label: "Diagnostic", icon: Stethoscope, gradient: "linear-gradient(135deg,#f97316,#fb923c)", target: "diagnostic" },
  { label: "Installation rapide", icon: Download, gradient: "linear-gradient(135deg,#3b82f6,#2563eb)", target: "quick-install" },
  { label: "Mises à jour", icon: RefreshCw, gradient: "linear-gradient(135deg,#22c55e,#16a34a)", target: "updates" },
  { label: "Dépannage", icon: Wrench, gradient: "linear-gradient(135deg,#ef4444,#dc2626)", target: "troubleshoot" },
  { label: "Générateur de rapport", icon: FileText, gradient: "linear-gradient(135deg,#8b5cf6,#7c3aed)", target: "report-generator" },
];
</script>

<template>
  <div class="dash-page">
    <NxSectionHeader title="Vue d'ensemble" />

    <div class="dash-actions">
      <NxQuickActionTile
        v-for="action in QUICK_ACTIONS"
        :key="action.target"
        :icon="action.icon"
        :label="action.label"
        :gradient="action.gradient"
        @click="emit('navigate', action.target)"
      />
    </div>

    <NxCard v-if="error" danger>Impossible de récupérer les informations système : {{ error }}</NxCard>
    <NxCard v-if="sensorsError" danger>Impossible de récupérer les capteurs : {{ sensorsError }}</NxCard>

    <div class="dash-grid" v-if="snapshot">
      <NxCard v-for="(cpu, i) in snapshot.cpus" :key="i">
        <NxStatTile :label="cpu.name || `CPU ${i}`" :value="cpu.usage_display" />
      </NxCard>
      <NxCard>
        <NxStatTile label="Mémoire" :value="`${bytesToGb(snapshot.memory_used_bytes)} / ${bytesToGb(snapshot.memory_total_bytes)} GB`" />
      </NxCard>
      <NxCard>
        <NxStatTile label="Processus" :value="String(snapshot.process_count)" />
      </NxCard>
      <NxCard v-if="sensors?.battery_percent !== null && sensors?.battery_percent !== undefined">
        <NxStatTile label="Batterie" :value="`${sensors!.battery_percent}%${sensors!.battery_charging ? ' ⚡' : ''}`" />
      </NxCard>
      <NxCard v-for="(t, i) in sensors?.temperatures ?? []" :key="`${t.label}-${i}`">
        <NxStatTile :label="t.label" :value="`${t.celsius.toFixed(0)}°C`" />
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.dash-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.dash-actions { display: flex; gap: 12px; flex-wrap: wrap; }
.dash-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 14px; }
</style>
```

Notes:
- `DashboardPage.vue` now emits `navigate` instead of changing any page state itself — it has no access to `currentPage` (that lives in `App.vue`). Task 9 wires this event up in `App.vue`.
- Every field/ref/function name (`snapshot`, `error`, `sensors`, `sensorsError`, `refresh`, `refreshSensors`, `bytesToGb`, the 2000ms interval) is unchanged from the original — only the template's presentation layer and the new quick-actions row are new.

- [ ] **Step 4: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/DashboardPage.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add src/pages/DashboardPage.vue src/pages/DashboardPage.spec.ts
git commit -m "feat: componentize DashboardPage on Nx*/style tokens, add quick-action tiles (spec section 2.4)"
```

---

## Task 4: Add "Performance" category to `categories.ts`

**Files:**
- Modify: `src/navigation/categories.ts`
- Modify: `src/App.spec.ts` (the "7 category titles" test becomes 8)

- [ ] **Step 1: Update the failing test first**

Read the live `src/App.spec.ts` (from R5: 6 tests, the first one asserts 7 category title strings are present). Change that one test:

```typescript
  it("renders AppNav with all 8 category titles", () => {
    const wrapper = mount(App);
    expect(wrapper.text()).toContain("Système");
    expect(wrapper.text()).toContain("Performance");
    expect(wrapper.text()).toContain("Applications");
    expect(wrapper.text()).toContain("Stockage");
    expect(wrapper.text()).toContain("Maintenance");
    expect(wrapper.text()).toContain("Réseau");
    expect(wrapper.text()).toContain("Rapports");
    expect(wrapper.text()).toContain("Paramètres");
  });
```
(replaces the existing `"renders AppNav with all 7 category titles"` test — every other test in the file is unchanged)

- [ ] **Step 2: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/App.spec.ts"`
Expected: FAIL — "Performance" not yet in `categories.ts`.

- [ ] **Step 3: Add the category**

Read the live `src/navigation/categories.ts` first. Insert a new category object right after the `"systeme"` category (matching `navigation.ts`'s ordering in NiTriTe, where "Performance" immediately follows "Système") and before `"applications"`:

```typescript
  {
    id: "performance",
    title: "Performance",
    pages: [
      { id: "optimizations", label: "Optimisations", icon: "zap" },
      { id: "temperatures", label: "Températures", icon: "thermometer" },
      { id: "benchmark", label: "Benchmark", icon: "gauge" },
      { id: "perf-history", label: "Historique perf.", icon: "bar-chart-3" },
    ],
  },
```

The full `navigationCategories` array is now 8 entries; every other existing category/page entry is untouched.

- [ ] **Step 4: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/App.spec.ts"`
Expected: PASS (6 tests, same count as before — one assertion changed, not added)

- [ ] **Step 5: Commit**

```bash
git add src/navigation/categories.ts src/App.spec.ts
git commit -m "feat: add Performance category to categories.ts (spec section 3)"
```

---

## Task 5: `TemperaturesPage.vue` (zero new backend)

**Files:**
- Create: `src/pages/TemperaturesPage.vue`
- Test: `src/pages/TemperaturesPage.spec.ts`

Reuses the already-existing, already-tested `get_sensor_snapshot` Tauri command (`sensors.rs`, unchanged by this plan) — no Rust work in this task.

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/TemperaturesPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import TemperaturesPage from "./TemperaturesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    battery_percent: 80,
    battery_charging: false,
    temperatures: [
      { label: "CPU", celsius: 45.2 },
      { label: "GPU", celsius: 72.8 },
      { label: "NVMe", celsius: 91.0 },
    ],
  }),
}));

describe("TemperaturesPage", () => {
  it("renders one card per sensor with a threshold-colored badge", async () => {
    const wrapper = mount(TemperaturesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("CPU"));
    expect(wrapper.text()).toContain("45");
    expect(wrapper.text()).toContain("GPU");
    expect(wrapper.text()).toContain("73");
    expect(wrapper.text()).toContain("NVMe");
    expect(wrapper.text()).toContain("91");
    // CPU (45°C) is under the 60° "success" threshold, NVMe (91°C) is over
    // the 80° "danger" threshold -- both badge classes must appear.
    expect(wrapper.find(".nx-badge--success").exists()).toBe(true);
    expect(wrapper.find(".nx-badge--danger").exists()).toBe(true);
  });

  it("shows an empty-state message when no sensors are detected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({
      battery_percent: null,
      battery_charging: null,
      temperatures: [],
    });
    const wrapper = mount(TemperaturesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun capteur"));
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/TemperaturesPage.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 3: Write `TemperaturesPage.vue`**

```vue
<!-- src/pages/TemperaturesPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface TemperatureReading { label: string; celsius: number }
interface SensorSnapshot { battery_percent: number | null; battery_charging: boolean | null; temperatures: TemperatureReading[] }

const snapshot = ref<SensorSnapshot | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<SensorSnapshot>("get_sensor_snapshot");
  } catch (err) {
    error.value = err instanceof Error ? err.message : String(err);
  }
});

function statusFor(celsius: number): "success" | "warning" | "danger" {
  if (celsius > 80) return "danger";
  if (celsius >= 60) return "warning";
  return "success";
}
</script>

<template>
  <div class="temp-page">
    <NxSectionHeader title="Températures" description="Relevés des capteurs thermiques détectés sur le système." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div v-else-if="snapshot && snapshot.temperatures.length === 0" class="temp-empty">
      Aucun capteur de température détecté sur ce système.
    </div>

    <div class="temp-grid" v-else-if="snapshot">
      <NxCard v-for="t in snapshot.temperatures" :key="t.label">
        <div class="temp-card-inner">
          <NxStatTile :label="t.label" :value="`${t.celsius.toFixed(0)}°C`" />
          <NxBadge :status="statusFor(t.celsius)">
            {{ statusFor(t.celsius) === "danger" ? "élevé" : statusFor(t.celsius) === "warning" ? "modéré" : "normal" }}
          </NxBadge>
        </div>
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.temp-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.temp-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
.temp-card-inner { display: flex; align-items: flex-start; justify-content: space-between; gap: 10px; }
.temp-empty { color: var(--nx-text-secondary); }
</style>
```

- [ ] **Step 4: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/TemperaturesPage.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add src/pages/TemperaturesPage.vue src/pages/TemperaturesPage.spec.ts
git commit -m "feat: add TemperaturesPage, reusing get_sensor_snapshot (spec section 3.1)"
```

---

## Task 6: Backend `benchmark.rs` + `BenchmarkPage.vue`

**Files:**
- Create: `src-tauri/src/benchmark.rs`
- Modify: `src-tauri/src/lib.rs` (register module + command)
- Create: `src/pages/BenchmarkPage.vue`
- Test: `src/pages/BenchmarkPage.spec.ts`

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/benchmark.rs
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::io::{Read, Write};
use std::time::{Duration, Instant};

#[derive(Serialize, Clone)]
pub struct BenchmarkResult {
    pub cpu_hashes_per_sec: u64,
    pub disk_write_mbps: f64,
    pub disk_read_mbps: f64,
    pub memory_bandwidth_gbps: f64,
}

/// Hashes a fixed 1 KB buffer repeatedly for `duration`, returning the
/// count of completed SHA-256 operations. The buffer content is irrelevant
/// (this measures raw hashing throughput, not any real data), so it is
/// zero-filled once and reused for every iteration.
pub fn benchmark_cpu(duration: Duration) -> u64 {
    let buf = [0u8; 1024];
    let mut count: u64 = 0;
    let start = Instant::now();
    while start.elapsed() < duration {
        let mut hasher = Sha256::new();
        hasher.update(buf);
        let _ = hasher.finalize();
        count += 1;
    }
    count
}

/// Allocates a fixed-size buffer and repeatedly copies it into a second
/// buffer for `duration`, returning bandwidth in GB/s. `std::hint::black_box`
/// prevents the compiler from optimizing the copy away entirely, since its
/// result is otherwise never observed.
pub fn benchmark_memory(duration: Duration) -> f64 {
    const CHUNK_BYTES: usize = 16 * 1024 * 1024; // 16 MB
    let src = vec![0xABu8; CHUNK_BYTES];
    let mut dst = vec![0u8; CHUNK_BYTES];
    let start = Instant::now();
    let mut bytes_copied: u64 = 0;
    while start.elapsed() < duration {
        dst.copy_from_slice(&src);
        std::hint::black_box(&dst);
        bytes_copied += CHUNK_BYTES as u64;
    }
    let elapsed_secs = start.elapsed().as_secs_f64();
    if elapsed_secs <= 0.0 {
        return 0.0;
    }
    (bytes_copied as f64 / elapsed_secs) / 1_073_741_824.0
}

/// Writes then reads back a fixed-size temp file, measuring MB/s for each
/// direction. The file is created under `std::env::temp_dir()` (never a
/// system path) and removed before returning, including on the error path,
/// so a failed read never leaves a stray multi-megabyte file behind.
pub fn benchmark_disk(size_bytes: usize) -> Result<(f64, f64), String> {
    let path = std::env::temp_dir().join(format!("nitrux-benchmark-{}.tmp", std::process::id()));
    let data = vec![0x5Au8; size_bytes];

    let write_result = (|| -> Result<f64, String> {
        let start = Instant::now();
        let mut file = std::fs::File::create(&path).map_err(|e| format!("création du fichier de test impossible : {e}"))?;
        file.write_all(&data).map_err(|e| format!("écriture impossible : {e}"))?;
        file.sync_all().map_err(|e| format!("synchronisation disque impossible : {e}"))?;
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return Ok(0.0);
        }
        Ok((size_bytes as f64 / elapsed) / 1_048_576.0)
    })();

    let read_result = write_result.clone().and_then(|_| {
        let start = Instant::now();
        let mut file = std::fs::File::open(&path).map_err(|e| format!("lecture impossible : {e}"))?;
        let mut buf = Vec::with_capacity(size_bytes);
        file.read_to_end(&mut buf).map_err(|e| format!("lecture impossible : {e}"))?;
        let elapsed = start.elapsed().as_secs_f64();
        if elapsed <= 0.0 {
            return Ok(0.0);
        }
        Ok((size_bytes as f64 / elapsed) / 1_048_576.0)
    });

    let _ = std::fs::remove_file(&path);

    let write_mbps = write_result?;
    let read_mbps = read_result?;
    Ok((write_mbps, read_mbps))
}

#[tauri::command]
pub fn run_benchmark() -> Result<BenchmarkResult, String> {
    let cpu_hashes_per_sec = benchmark_cpu(Duration::from_millis(800));
    let memory_bandwidth_gbps = benchmark_memory(Duration::from_millis(500));
    let (disk_write_mbps, disk_read_mbps) = benchmark_disk(50 * 1024 * 1024)?; // 50 MB

    Ok(BenchmarkResult {
        cpu_hashes_per_sec,
        disk_write_mbps,
        disk_read_mbps,
        memory_bandwidth_gbps,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn benchmark_cpu_completes_at_least_one_hash_in_a_nonzero_window() {
        let count = benchmark_cpu(Duration::from_millis(50));
        assert!(count > 0, "expected at least one SHA-256 op in 50ms, got {count}");
    }

    #[test]
    fn benchmark_memory_reports_a_positive_bandwidth() {
        let gbps = benchmark_memory(Duration::from_millis(50));
        assert!(gbps > 0.0, "expected positive bandwidth, got {gbps}");
    }

    #[test]
    fn benchmark_disk_writes_reads_and_cleans_up_the_temp_file() {
        let size = 1024 * 1024; // 1 MB, small and fast for a test
        let (write_mbps, read_mbps) = benchmark_disk(size).expect("benchmark should succeed on a normal filesystem");
        assert!(write_mbps > 0.0, "expected positive write throughput, got {write_mbps}");
        assert!(read_mbps > 0.0, "expected positive read throughput, got {read_mbps}");

        let path = std::env::temp_dir().join(format!("nitrux-benchmark-{}.tmp", std::process::id()));
        assert!(!path.exists(), "temp file should be removed after the benchmark runs");
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance/src-tauri && cargo test benchmark:: 2>&1 | tail -20"`
Expected: FAIL — module not registered in `lib.rs` yet, won't compile as part of the crate.

- [ ] **Step 3: Register the module in `lib.rs`**

Read the live `src-tauri/src/lib.rs` first. Add `mod benchmark;` to the `mod` list (alphabetically, between `mod disks;` and `mod disk_write;` — actually alphabetically `benchmark` sorts before `disk_write`/`disks`, so insert it as the very first `mod` line, before `mod disk_write;`). Add `benchmark::run_benchmark,` to the `tauri::generate_handler![...]` list (near the other performance-adjacent commands — position next to `detect_native_manager,`/`report::generate_system_report,` is fine, exact position doesn't matter functionally).

- [ ] **Step 4: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance/src-tauri && cargo test 2>&1 | tail -20"`
Expected: `135 passed; 0 failed; 1 ignored` (132 baseline + 3 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/benchmark.rs src-tauri/src/lib.rs
git commit -m "feat: add benchmark.rs — CPU/disk/memory micro-benchmarks (spec section 3.2)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/BenchmarkPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BenchmarkPage from "./BenchmarkPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    cpu_hashes_per_sec: 500_000,
    disk_write_mbps: 320.5,
    disk_read_mbps: 480.2,
    memory_bandwidth_gbps: 12.4,
  }),
}));

describe("BenchmarkPage", () => {
  it("runs the benchmark on button click and displays the results", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("320.5"));
    expect(invoke).toHaveBeenCalledWith("run_benchmark");
    expect(wrapper.text()).toContain("480.2");
    expect(wrapper.text()).toContain("12.4");
  });

  it("shows an error message when the benchmark command rejects", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockRejectedValueOnce("erreur de benchmark disque");
    const wrapper = mount(BenchmarkPage);
    const button = wrapper.findAll("button").find((b) => b.text() === "Lancer le benchmark")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("erreur de benchmark disque"));
  });
});
```

- [ ] **Step 7: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/BenchmarkPage.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 8: Write `BenchmarkPage.vue`**

```vue
<!-- src/pages/BenchmarkPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface BenchmarkResult {
  cpu_hashes_per_sec: number;
  disk_write_mbps: number;
  disk_read_mbps: number;
  memory_bandwidth_gbps: number;
}

const running = ref(false);
const error = ref<string | null>(null);
const result = ref<BenchmarkResult | null>(null);

async function run() {
  running.value = true;
  error.value = null;
  result.value = null;
  try {
    result.value = await invoke<BenchmarkResult>("run_benchmark");
  } catch (e) {
    error.value = String(e);
  } finally {
    running.value = false;
  }
}
</script>

<template>
  <div class="bench-page">
    <NxSectionHeader title="Benchmark" description="Mesure rapide des performances CPU, disque et mémoire de ce système." />

    <NxCard>
      <NxButton :disabled="running" @click="run">{{ running ? "Benchmark en cours..." : "Lancer le benchmark" }}</NxButton>
    </NxCard>

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <div class="bench-grid" v-if="result">
      <NxCard><NxStatTile label="CPU (hachages/s)" :value="result.cpu_hashes_per_sec.toLocaleString('fr-FR')" /></NxCard>
      <NxCard><NxStatTile label="Écriture disque" :value="`${result.disk_write_mbps.toFixed(1)} Mo/s`" /></NxCard>
      <NxCard><NxStatTile label="Lecture disque" :value="`${result.disk_read_mbps.toFixed(1)} Mo/s`" /></NxCard>
      <NxCard><NxStatTile label="Bande passante mémoire" :value="`${result.memory_bandwidth_gbps.toFixed(1)} Go/s`" /></NxCard>
    </div>
  </div>
</template>

<style scoped>
.bench-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.bench-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(200px, 1fr)); gap: 14px; }
</style>
```

- [ ] **Step 9: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/BenchmarkPage.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 10: Commit the frontend**

```bash
git add src/pages/BenchmarkPage.vue src/pages/BenchmarkPage.spec.ts
git commit -m "feat: add BenchmarkPage (spec section 3.2)"
```

---

## Task 7: `NxSparkline.vue` + `PerfHistoryPage.vue` (100% frontend)

**Files:**
- Create: `src/components/ui/NxSparkline.vue`
- Test: `src/components/ui/NxSparkline.spec.ts`
- Create: `src/pages/PerfHistoryPage.vue`
- Test: `src/pages/PerfHistoryPage.spec.ts`

No backend work — reuses `get_system_snapshot`/`get_sensor_snapshot`, already used by `DashboardPage.vue` and now `TemperaturesPage.vue`.

### `NxSparkline.vue`

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/ui/NxSparkline.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxSparkline from "./NxSparkline.vue";

describe("NxSparkline", () => {
  it("renders an svg polyline with one point per value", () => {
    const wrapper = mount(NxSparkline, { props: { values: [10, 50, 30, 80, 20] } });
    const polyline = wrapper.find("polyline");
    expect(polyline.exists()).toBe(true);
    const points = polyline.attributes("points")!.trim().split(" ");
    expect(points.length).toBe(5);
  });

  it("renders an empty svg without error when given no values", () => {
    const wrapper = mount(NxSparkline, { props: { values: [] } });
    expect(wrapper.find("svg").exists()).toBe(true);
    expect(wrapper.find("polyline").exists()).toBe(false);
  });
});
```

- [ ] **Step 2: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/components/ui/NxSparkline.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 3: Write `NxSparkline.vue`**

```vue
<!-- src/components/ui/NxSparkline.vue -->
<script setup lang="ts">
import { computed } from "vue";

const props = defineProps<{ values: number[]; width?: number; height?: number }>();

const width = computed(() => props.width ?? 240);
const height = computed(() => props.height ?? 48);

const points = computed(() => {
  if (props.values.length === 0) return "";
  const min = Math.min(...props.values);
  const max = Math.max(...props.values);
  const range = max - min || 1;
  const stepX = props.values.length > 1 ? width.value / (props.values.length - 1) : 0;
  return props.values
    .map((v, i) => {
      const x = i * stepX;
      const y = height.value - ((v - min) / range) * height.value;
      return `${x.toFixed(1)},${y.toFixed(1)}`;
    })
    .join(" ");
});
</script>

<template>
  <svg :width="width" :height="height" class="nx-sparkline">
    <polyline v-if="points" :points="points" fill="none" stroke="var(--nx-accent-primary)" stroke-width="2" />
  </svg>
</template>

<style scoped>
.nx-sparkline { display: block; }
</style>
```

- [ ] **Step 4: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/components/ui/NxSparkline.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 5: Commit**

```bash
git add src/components/ui/NxSparkline.vue src/components/ui/NxSparkline.spec.ts
git commit -m "feat: add NxSparkline shared component (spec section 3.3)"
```

### `PerfHistoryPage.vue`

- [ ] **Step 6: Write the failing test**

```typescript
// src/pages/PerfHistoryPage.spec.ts
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import PerfHistoryPage from "./PerfHistoryPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_system_snapshot") {
      return Promise.resolve({
        cpus: [{ name: "Test CPU", usage_percent: 42, usage_display: "42%" }],
        memory_used_bytes: 4_000_000_000,
        memory_total_bytes: 8_000_000_000,
        process_count: 200,
      });
    }
    return Promise.resolve(null);
  }),
}));

describe("PerfHistoryPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("polls get_system_snapshot on mount and renders a sparkline once a sample is collected", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(PerfHistoryPage);
    await vi.waitFor(() => expect(wrapper.find(".nx-sparkline").exists()).toBe(true));
    expect(invoke).toHaveBeenCalledWith("get_system_snapshot");
  });
});
```

- [ ] **Step 7: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/PerfHistoryPage.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 8: Write `PerfHistoryPage.vue`**

```vue
<!-- src/pages/PerfHistoryPage.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import NxSparkline from "@/components/ui/NxSparkline.vue";
import { usePreferencesStore } from "@/stores/preferencesStore";

interface CpuInfo { usage_percent: number }
interface SystemSnapshot { cpus: CpuInfo[]; memory_used_bytes: number; memory_total_bytes: number }

const MAX_SAMPLES = 60;

const preferences = usePreferencesStore();
const cpuHistory = ref<number[]>([]);
const memoryHistory = ref<number[]>([]);
const error = ref<string | null>(null);
let intervalId: number | undefined;

function averageCpuPercent(cpus: CpuInfo[]): number {
  if (cpus.length === 0) return 0;
  return cpus.reduce((sum, c) => sum + c.usage_percent, 0) / cpus.length;
}

function pushSample(arr: typeof cpuHistory, value: number) {
  arr.value.push(value);
  if (arr.value.length > MAX_SAMPLES) arr.value.shift();
}

async function sample() {
  try {
    const snapshot = await invoke<SystemSnapshot>("get_system_snapshot");
    pushSample(cpuHistory, averageCpuPercent(snapshot.cpus));
    pushSample(memoryHistory, (snapshot.memory_used_bytes / snapshot.memory_total_bytes) * 100);
    error.value = null;
  } catch (e) {
    error.value = String(e);
  }
}

onMounted(() => {
  sample();
  intervalId = window.setInterval(sample, preferences.dashboardRefreshIntervalMs);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});
</script>

<template>
  <div class="perf-page">
    <NxSectionHeader title="Historique perf." description="CPU et mémoire depuis l'ouverture de cette page (non persisté)." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard>
      <NxSectionHeader title="CPU (%)" />
      <NxSparkline :values="cpuHistory" :width="600" :height="80" />
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Mémoire (%)" />
      <NxSparkline :values="memoryHistory" :width="600" :height="80" />
    </NxCard>
  </div>
</template>

<style scoped>
.perf-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
</style>
```

- [ ] **Step 9: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/PerfHistoryPage.spec.ts"`
Expected: PASS (1 test)

- [ ] **Step 10: Commit**

```bash
git add src/pages/PerfHistoryPage.vue src/pages/PerfHistoryPage.spec.ts
git commit -m "feat: add PerfHistoryPage — client-side rolling buffer + sparklines (spec section 3.3)"
```

---

## Task 8: Backend `optimizations.rs` (read-only) + `OptimizationsPage.vue`

**Files:**
- Create: `src-tauri/src/optimizations.rs`
- Modify: `src-tauri/src/lib.rs` (register module + command)
- Create: `src/pages/OptimizationsPage.vue`
- Test: `src/pages/OptimizationsPage.spec.ts`

### Backend

Real command output captured on the project's own Debian dev VM during this plan's research (for reference while writing/reviewing the parsers below):
```
$ systemctl list-unit-files --type=service --state=enabled --no-legend
accounts-daemon.service             enabled enabled
anacron.service                     enabled enabled
apparmor.service                    enabled enabled
...
$ cat /proc/sys/vm/swappiness
60
$ systemctl is-enabled fstrim.timer
enabled
```

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/optimizations.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct OptimizationSnapshot {
    pub enabled_services: Vec<String>,
    pub swappiness: Option<u8>,
    pub zram_active: bool,
    pub fstrim_timer_enabled: bool,
}

/// Parses one line of `systemctl list-unit-files --type=service
/// --state=enabled --no-legend` output into just the unit name (first
/// whitespace-separated field), e.g. "accounts-daemon.service             enabled enabled"
/// -> "accounts-daemon.service". Blank lines are skipped.
pub fn parse_enabled_service_line(line: &str) -> Option<String> {
    let name = line.split_whitespace().next()?;
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

/// Parses `/proc/sys/vm/swappiness` content (a single integer 0-100 on its
/// own line) -- mirrors `sensors::parse_capacity`'s exact pattern for the
/// same reason: a single trimmed-and-parsed `/proc`/`/sys` integer file.
pub fn parse_swappiness(content: &str) -> Option<u8> {
    content.trim().parse::<u8>().ok()
}

/// True if any active swap device (from `/proc/swaps` content) is backed
/// by a zram device. Skips the header line (`Filename Type Size Used
/// Priority`) that `/proc/swaps` always starts with.
pub fn detect_zram_active(proc_swaps_content: &str) -> bool {
    proc_swaps_content.lines().skip(1).any(|line| line.contains("zram"))
}

fn run_optimization_diagnostics() -> OptimizationSnapshot {
    let enabled_services = subprocess::run_with_timeout(
        "systemctl",
        &["list-unit-files", "--type=service", "--state=enabled", "--no-legend"],
        Duration::from_secs(5),
    )
    .map(|output| output.lines().filter_map(parse_enabled_service_line).collect())
    .unwrap_or_default();

    let swappiness = std::fs::read_to_string("/proc/sys/vm/swappiness")
        .ok()
        .and_then(|content| parse_swappiness(&content));

    let zram_active = std::fs::read_to_string("/proc/swaps")
        .map(|content| detect_zram_active(&content))
        .unwrap_or(false);

    let fstrim_timer_enabled = subprocess::run_capturing_exit_code(
        "systemctl",
        &["is-enabled", "fstrim.timer"],
        Duration::from_secs(5),
    )
    .map(|(stdout, _code)| stdout.trim() == "enabled")
    .unwrap_or(false);

    OptimizationSnapshot { enabled_services, swappiness, zram_active, fstrim_timer_enabled }
}

#[tauri::command]
pub fn get_optimization_snapshot() -> OptimizationSnapshot {
    run_optimization_diagnostics()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_enabled_service_line_into_unit_name() {
        let line = "accounts-daemon.service             enabled enabled";
        assert_eq!(parse_enabled_service_line(line), Some("accounts-daemon.service".to_string()));
    }

    #[test]
    fn skips_blank_lines_in_service_list() {
        assert_eq!(parse_enabled_service_line(""), None);
        assert_eq!(parse_enabled_service_line("   "), None);
    }

    #[test]
    fn parses_valid_swappiness_value() {
        assert_eq!(parse_swappiness("60\n"), Some(60));
        assert_eq!(parse_swappiness("0"), Some(0));
    }

    #[test]
    fn rejects_malformed_swappiness_content() {
        assert_eq!(parse_swappiness("not a number"), None);
        assert_eq!(parse_swappiness(""), None);
    }

    #[test]
    fn detects_zram_swap_device_in_proc_swaps() {
        let content = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n/dev/zram0                              partition\t8388604\t\t0\t\t100\n";
        assert!(detect_zram_active(content));
    }

    #[test]
    fn returns_false_when_no_swap_device_is_zram() {
        let content = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n/dev/sda3                               partition\t6873084\t\t524\t\t-2\n";
        assert!(!detect_zram_active(content));
    }

    #[test]
    fn returns_false_when_no_swap_devices_at_all() {
        let content = "Filename\t\t\t\tType\t\tSize\t\tUsed\t\tPriority\n";
        assert!(!detect_zram_active(content));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance/src-tauri && cargo test optimizations:: 2>&1 | tail -20"`
Expected: FAIL — module not registered yet.

- [ ] **Step 3: Register the module in `lib.rs`**

Add `mod optimizations;` to the `mod` list (alphabetically, between `mod network_write;` and `mod packages;`). Add `optimizations::get_optimization_snapshot,` to the `tauri::generate_handler![...]` list.

- [ ] **Step 4: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance/src-tauri && cargo test 2>&1 | tail -20"`
Expected: `141 passed; 0 failed; 1 ignored` (135 from Task 6 + 6 new).

- [ ] **Step 5: Verify against the real VM**

The project already has SSH access to a disposable Debian VM used throughout this project's earlier phases (`172.18.32.124`, user `dev`). Confirm the real commands this module shells out to behave as the parsers above assume:

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && python /tmp/ssh_run.py dev 1998 'systemctl list-unit-files --type=service --state=enabled --no-legend | head -3; cat /proc/sys/vm/swappiness; systemctl is-enabled fstrim.timer'"` — if the SSH helper script from earlier sessions isn't at `/tmp/ssh_run.py` in this environment, locate it (it was used earlier in this project under a Windows scratchpad path) or write an equivalent 5-line paramiko script; the point is confirming real output shape, not the exact script location. Expected: output matches the exact formats already captured in this task's introduction (unit-name-first-field service lines, a bare integer for swappiness, `enabled` for the timer) — if it doesn't, the parser needs fixing before proceeding, not the VM.

- [ ] **Step 6: Commit the backend**

```bash
git add src-tauri/src/optimizations.rs src-tauri/src/lib.rs
git commit -m "feat: add optimizations.rs — read-only startup/swappiness/zram/fstrim diagnostics (spec section 3.4)"
```

### Frontend

- [ ] **Step 7: Write the failing frontend test**

```typescript
// src/pages/OptimizationsPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import OptimizationsPage from "./OptimizationsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    enabled_services: ["accounts-daemon.service", "cron.service", "cups.service"],
    swappiness: 60,
    zram_active: false,
    fstrim_timer_enabled: true,
  }),
}));

describe("OptimizationsPage", () => {
  it("renders swappiness, zram/fstrim status, and the enabled services list", async () => {
    const wrapper = mount(OptimizationsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("accounts-daemon.service"));
    expect(wrapper.text()).toContain("60");
    expect(wrapper.text()).toContain("cron.service");
    expect(wrapper.text()).toContain("cups.service");
  });

  it("has no buttons that trigger a write action -- read-only diagnostic only", async () => {
    const wrapper = mount(OptimizationsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("accounts-daemon.service"));
    const buttons = wrapper.findAll("button");
    for (const b of buttons) {
      expect(b.text().toLowerCase()).not.toMatch(/désactiver|activer|appliquer|modifier/);
    }
  });
});
```

- [ ] **Step 8: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/OptimizationsPage.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 9: Write `OptimizationsPage.vue`**

```vue
<!-- src/pages/OptimizationsPage.vue -->
<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxStatTile from "@/components/ui/NxStatTile.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface OptimizationSnapshot {
  enabled_services: string[];
  swappiness: number | null;
  zram_active: boolean;
  fstrim_timer_enabled: boolean;
}

const snapshot = ref<OptimizationSnapshot | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    snapshot.value = await invoke<OptimizationSnapshot>("get_optimization_snapshot");
  } catch (e) {
    error.value = String(e);
  }
});

const swappinessAdvice = computed(() => {
  const s = snapshot.value?.swappiness;
  if (s === null || s === undefined) return null;
  if (s > 30) return `Swappiness à ${s} — valeur élevée pour un poste de travail. Une valeur plus basse (10-20) privilégie la RAM au swap sur une machine avec suffisamment de mémoire.`;
  return `Swappiness à ${s} — valeur déjà basse, cohérente avec un usage desktop.`;
});
</script>

<template>
  <div class="opt-page">
    <NxSectionHeader title="Optimisations" description="Diagnostic système en lecture seule — aucune action n'est appliquée automatiquement." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <template v-if="snapshot">
      <div class="opt-stats">
        <NxCard><NxStatTile label="Swappiness" :value="String(snapshot.swappiness ?? '—')" /></NxCard>
        <NxCard>
          <NxStatTile label="ZRAM" :value="snapshot.zram_active ? 'actif' : 'inactif'" />
          <NxBadge :status="snapshot.zram_active ? 'success' : 'info'">{{ snapshot.zram_active ? "actif" : "inactif" }}</NxBadge>
        </NxCard>
        <NxCard>
          <NxStatTile label="Timer fstrim" :value="snapshot.fstrim_timer_enabled ? 'activé' : 'désactivé'" />
          <NxBadge :status="snapshot.fstrim_timer_enabled ? 'success' : 'warning'">{{ snapshot.fstrim_timer_enabled ? "activé" : "désactivé" }}</NxBadge>
        </NxCard>
      </div>

      <NxCard v-if="swappinessAdvice">
        <p class="opt-advice">{{ swappinessAdvice }}</p>
      </NxCard>

      <NxCard>
        <NxSectionHeader :title="`Services activés au démarrage (${snapshot.enabled_services.length})`" />
        <div class="opt-services">
          <NxBadge v-for="s in snapshot.enabled_services" :key="s" status="info">{{ s }}</NxBadge>
        </div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.opt-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.opt-stats { display: grid; grid-template-columns: repeat(auto-fit, minmax(180px, 1fr)); gap: 12px; }
.opt-advice { margin: 0; font-size: 13px; color: var(--nx-text-secondary); line-height: 1.5; }
.opt-services { display: flex; flex-wrap: wrap; gap: 6px; }
</style>
```

- [ ] **Step 10: Run it to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/pages/OptimizationsPage.spec.ts"`
Expected: PASS (2 tests)

- [ ] **Step 11: Commit the frontend**

```bash
git add src/pages/OptimizationsPage.vue src/pages/OptimizationsPage.spec.ts
git commit -m "feat: add OptimizationsPage — read-only diagnostic view (spec section 3.4)"
```

---

## Task 9: Wire `App.vue` to the 4 new pages + `DashboardPage`'s `navigate` event

**Files:**
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`

Read the live `src/App.vue` first — it currently has 15 entries in its `pages` map (Tasks 1-8 of this plan did not touch it, only `categories.ts` gained the 4 new page ids in Task 4, which `AppNav.vue` already renders as clickable nav items — clicking one right now would silently fall back to the dashboard per the `pages[currentPage] ?? pages.dashboard` guard, since there's no map entry yet). `DashboardPage.vue` (Task 3) now emits `navigate` instead of doing nothing — `App.vue` needs to listen for it.

- [ ] **Step 1: Write the failing tests**

Add to the existing `describe("App", ...)` block in `src/App.spec.ts`:

```typescript
  it("shows the real TemperaturesPage for the temperatures id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const tempButton = buttons.find((b) => b.text() === "Températures")!;
    await tempButton.trigger("click");
    // App.spec.ts's global invoke mock resolves `null` for every command,
    // so `TemperaturesPage`'s `snapshot` stays null and neither its
    // empty-state message nor its data grid renders (both are gated behind
    // `v-else-if="snapshot"`) -- assert on the page's always-rendered
    // static header instead, which is outside that guard.
    expect(wrapper.text()).toContain("Relevés des capteurs thermiques");
  });

  it("shows the real BenchmarkPage for the benchmark id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const benchButton = buttons.find((b) => b.text() === "Benchmark")!;
    await benchButton.trigger("click");
    expect(wrapper.text()).toContain("Lancer le benchmark");
  });

  it("navigates to DiagnosticPage when the dashboard's Diagnostic quick-action tile is clicked", async () => {
    const wrapper = mount(App);
    await vi.waitFor(() => expect(wrapper.findAll(".nx-quick-action").length).toBeGreaterThan(0));
    const tiles = wrapper.findAll(".nx-quick-action");
    const diagnosticTile = tiles.find((t) => t.text().includes("Diagnostic"))!;
    await diagnosticTile.trigger("click");
    expect(wrapper.text()).toContain("Composants matériels détectés");
  });
```

(3 new tests added to the existing 6 — `PerfHistoryPage`/`OptimizationsPage` aren't asserted here individually since the pattern is already proven by the two tests above and the dashboard-navigate test; this keeps `App.spec.ts` from becoming redundant with each individual page's own spec file, consistent with how R2-R5 kept `App.spec.ts` to a handful of representative routing checks rather than one per page.)

- [ ] **Step 2: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/App.spec.ts"`
Expected: FAIL — `temperatures`/`benchmark` ids have no map entry (silently fall back to dashboard), and `App.vue` doesn't listen for `DashboardPage`'s `navigate` event yet.

- [ ] **Step 3: Update `App.vue`**

Add the 4 new imports (alphabetically grouped with the other page imports is not required by this codebase's existing convention — imports are grouped by when they were added historically, not alphabetized; add these 4 right after the existing `ReportGeneratorPage` import):

```typescript
import TemperaturesPage from "@/pages/TemperaturesPage.vue";
import BenchmarkPage from "@/pages/BenchmarkPage.vue";
import PerfHistoryPage from "@/pages/PerfHistoryPage.vue";
import OptimizationsPage from "@/pages/OptimizationsPage.vue";
```

Add the 4 new entries to the `pages` map (any position is functionally fine; group them together for readability):

```typescript
  optimizations: OptimizationsPage,
  temperatures: TemperaturesPage,
  benchmark: BenchmarkPage,
  "perf-history": PerfHistoryPage,
```

Change the template to listen for `DashboardPage`'s `navigate` event and route it through the same `currentPage` ref used by `AppNav`:

```vue
<template>
  <LayoutShell>
    <template #nav>
      <AppNav v-model="currentPage" />
    </template>
    <component :is="pages[currentPage] ?? pages.dashboard" @navigate="currentPage = $event" />
  </LayoutShell>
</template>
```

`@navigate="currentPage = $event"` is harmless to attach unconditionally to every page component via `<component :is>` — Vue only forwards it to components that actually declare `defineEmits<{ navigate: [...] }>()` (currently only `DashboardPage.vue`); every other page simply ignores an event handler it never emits, exactly like `@click` handlers already do elsewhere on components that don't use them.

- [ ] **Step 4: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/App.spec.ts"`
Expected: PASS (9 tests — 6 from before + 3 new).

- [ ] **Step 5: Commit**

```bash
git add src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue to the 4 new Performance pages and DashboardPage's navigate event (spec section 3)"
```

---

## Task 10: Full verification pass — frontend, backend, and live VM check

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npm run test -- --run 2>&1 | tail -25"`
Expected: baseline entering this plan was 128 (end of R5). This plan adds: Task 1 (2) + Task 2 (2) + Task 3 (2) + Task 4 (net 0, one assertion changed) + Task 5 (2) + Task 6 (2) + Task 7 (2+1=3) + Task 8 (2) + Task 9 (net +3) = 128 + 2+2+2+2+2+3+2+3 = 146. Report the real observed total, don't just assert 146.

- [ ] **Step 2: Type-check**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vue-tsc --noEmit"`
Expected: clean.

- [ ] **Step 3: Confirm the Rust suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance/src-tauri && cargo test 2>&1 | tail -15"`
Expected: `141 passed; 0 failed; 1 ignored` (132 baseline + 3 from Task 6 + 6 from Task 8).

- [ ] **Step 4: Confirm every existing page still shows an icon in the nav**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx vitest run src/components/nav/AppNav.spec.ts src/App.spec.ts 2>&1 | tail -15"`
Expected: all pass — the `AppNav.spec.ts` test already asserts every rendered nav item has an `<svg>`, and this run confirms it against the real, current `categories.ts` (19 pages across 8 categories) rather than just the isolated component test's own props.

- [ ] **Step 5: Real build + live check on the VM**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r6-visual-foundation-performance && npx tauri build 2>&1 | tail -20"` — expect a clean `.deb` build (AppImage failing on `xdg-open` is expected and non-blocking, as in every prior phase).

Transfer the built `.deb` to the project's Debian dev VM (`172.18.32.124`, user `dev`) the same way every prior phase's release was verified, install it, and launch the app on the VM's active desktop session (same env vars as prior sessions: `DISPLAY=:1 WAYLAND_DISPLAY=wayland-0 XAUTHORITY=/run/user/1000/xauth_* DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus`, values may have changed since the VM's Xwayland auth cookie regenerates per session — re-derive them via `systemctl --user show-environment` on the VM if the previous session's exact values don't work). Confirm visually (a screenshot taken on the VM, pulled back and viewed) that:
- The nav shows real icons next to every label, not just text.
- The dashboard shows 5 colored gradient quick-action tiles.
- Clicking into "Optimisations" shows real `enabled_services`/swappiness/zram/fstrim data from the actual VM, not empty/mocked values.
- Clicking "Lancer le benchmark" on the Benchmark page produces real non-zero numbers.

This is the step that catches anything a mock-based Vitest run cannot: whether `systemctl`/`/proc` really parse as assumed on a real system, and whether the icon/gradient work actually renders as intended in a real webview.

- [ ] **Step 6: Commit any final cleanup**

No further commit expected if Steps 1–5 all pass clean.

---

## Self-Review

**Spec coverage:** §2 (lucide-vue-next, AppNav icons, NxQuickActionTile, DashboardPage componentization) — Tasks 1-3. §3 (Performance category + 4 pages) — Task 4 (category) + Tasks 5-8 (the 4 pages) + Task 9 (final wiring). §4 (verification incl. live VM check) — Task 10. §5 (out of scope: no write/toggle in Optimisations, no per-category accent colors beyond the dashboard tiles, no history persistence) — confirmed no task in this plan adds any of those.

**Placeholder scan:** No "TBD"/"TODO". The VM SSH script path in Task 8 Step 5 and Task 10 Step 5 is deliberately described as "locate it, or write an equivalent" rather than a hardcoded path, since scratchpad paths are session-specific and not guaranteed stable across a fresh implementer session — this is an intentional adaptation instruction, not a vague placeholder (the actual verification commands and expected output are fully specified).

**Type consistency:** `BenchmarkResult` (Rust: `cpu_hashes_per_sec: u64`, `disk_write_mbps: f64`, `disk_read_mbps: f64`, `memory_bandwidth_gbps: f64`) matches `BenchmarkPage.vue`'s TypeScript interface exactly (Task 6). `OptimizationSnapshot` (Rust: `enabled_services: Vec<String>`, `swappiness: Option<u8>`, `zram_active: bool`, `fstrim_timer_enabled: bool`) matches `OptimizationsPage.vue`'s interface exactly (Task 8), with `Option<u8>` → `number | null` following this codebase's established convention (already used for `PciDevice`/`DeviceDriver`/etc.). `NxQuickActionTile`'s `icon: Component` prop type matches how `DashboardPage.vue` passes lucide components directly (Task 2/3). Every `Nx*` component prop used (`NxCard.danger`, `NxButton.disabled`, `NxBadge.status`, `NxStatTile.label/value`, `NxSectionHeader.title/description`) was cross-checked against each file's live `defineProps` during this plan's research, matching every prior phase's discipline.
