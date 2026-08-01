# NiTruX Phase R3 — Applications > Installation rapide — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the `QuickInstallPlaceholder.vue` "Bientôt disponible" page (Applications > Installation rapide) with a real curated app catalog offering one-click, silent, native-package-manager installs with progress feedback — spec `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` §4.1.

**Architecture:** A static curated catalog (`src/data/appCatalog.ts`) drives a grid UI (`src/pages/QuickInstallPage.vue`) built on the R1 `Nx*` component library. Installs reuse the existing, already-VM-verified `install_package` Tauri command (Phase 2 Part 2) — zero new privileged surface. A new thin read-only Tauri command (`detect_native_manager`) lets the page auto-detect which native package manager to use instead of asking the user to pick one, unlike `PackagesPage.vue`'s manual dropdown. Only catalog entries installable via the detected native manager (`apt`/`dnf`/`pacman`/`zypper`) are enabled in v1; Flatpak/Snap entries are shown (browsable) but their install button is disabled with a "Bientôt disponible" badge — installing them is a new privileged surface (flatpak needs no root but snap does, via snapd's own polkit integration) deliberately deferred per spec §4.1's rationale, not added under an unattended pass.

**Tech Stack:** Tauri v2 + Rust (backend), Vue 3.5 + TypeScript + Vitest (frontend), same patterns as Phases R1/R2.

---

## Task 1: Backend — expose `detect_native_manager`

**Files:**
- Modify: `src-tauri/src/lib.rs`

The frontend needs to know which native package manager is present on the host to call `install_package(manager, package)` without asking the user to pick one every time (unlike `PackagesPage.vue`'s existing manual dropdown, which stays as-is — this is a new, separate, auto-detecting flow for the curated catalog only). `packages::detect_package_managers()` already exists and is already tested (`detected_manager_id_matches_binary_name` in `src-tauri/src/packages/mod.rs`) — this task only exposes its first result as a thin Tauri command, mirroring the exact pattern `list_updates` already uses in this same file (a bare aggregator function defined directly in `lib.rs`, not a new module).

- [ ] **Step 1: Add the command**

Read the live `src-tauri/src/lib.rs` first (reproduced above in this plan's context-gathering — it currently has `list_updates` defined right before `pub fn run()`, and a `tauri::generate_handler![...]` list ending with `disk_write::clone_disk`). Add this function right after `list_updates`:

```rust
/// Returns the id of the first detected native package manager
/// ("apt"/"dnf"/"pacman"/"zypper"), or `None` if none is present. Thin
/// wrapper over the already-tested `packages::detect_package_managers()` —
/// no dedicated test here for the same reason `list_updates` has none:
/// it's a pure aggregation over an already-verified primitive, and actually
/// exercising manager detection requires the real host's binaries (already
/// covered by `detected_manager_id_matches_binary_name` in
/// `packages/mod.rs`).
#[tauri::command]
fn detect_native_manager() -> Option<String> {
    packages::detect_package_managers()
        .first()
        .map(|m| m.id().to_string())
}
```

- [ ] **Step 2: Register it in the invoke handler**

In the `tauri::generate_handler![...]` list, add `detect_native_manager,` right after `list_updates,`:

```rust
        .invoke_handler(tauri::generate_handler![
            system::get_system_snapshot,
            sensors::get_sensor_snapshot,
            hardware::get_pci_devices,
            drivers::get_driver_snapshot,
            logs::get_recent_logs,
            list_updates,
            detect_native_manager,
            disks::list_disks,
            ...
```
(the rest of the list — everything from `disks::list_disks` onward — is unchanged; only the `detect_native_manager,` line is new)

- [ ] **Step 3: Verify it compiles and the existing Rust suite is unaffected**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && cargo build --manifest-path src-tauri/Cargo.toml 2>&1 | tail -20 && cargo test --manifest-path src-tauri/Cargo.toml 2>&1 | tail -10"`
Expected: clean build, `124 passed; 0 failed; 1 ignored` (unchanged — this step adds a command with no dedicated test, by design, matching `list_updates`'s precedent).

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: expose detect_native_manager Tauri command (spec section 4.1)"
```

---

## Task 2: Frontend — curated app catalog data

**Files:**
- Create: `src/data/appCatalog.ts`
- Test: `src/data/appCatalog.spec.ts`

- [ ] **Step 1: Write the failing sanity test**

```typescript
// src/data/appCatalog.spec.ts
import { describe, it, expect } from "vitest";
import { appCatalog } from "./appCatalog";

describe("appCatalog", () => {
  it("has no duplicate ids", () => {
    const ids = appCatalog.map((e) => e.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every entry has non-empty required fields and a valid installMethod", () => {
    const validMethods = ["apt", "flatpak", "snap"];
    for (const entry of appCatalog) {
      expect(entry.id.length).toBeGreaterThan(0);
      expect(entry.name.length).toBeGreaterThan(0);
      expect(entry.description.length).toBeGreaterThan(0);
      expect(entry.icon.length).toBeGreaterThan(0);
      expect(entry.category.length).toBeGreaterThan(0);
      expect(entry.packageId.length).toBeGreaterThan(0);
      expect(validMethods).toContain(entry.installMethod);
    }
  });

  it("contains at least one apt entry and at least one flatpak or snap entry (needed to exercise both the enabled and disabled install UI states)", () => {
    expect(appCatalog.some((e) => e.installMethod === "apt")).toBe(true);
    expect(appCatalog.some((e) => e.installMethod === "flatpak" || e.installMethod === "snap")).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npx vitest run src/data/appCatalog.spec.ts"`
Expected: FAIL — module doesn't exist.

- [ ] **Step 3: Write the catalog**

```typescript
// src/data/appCatalog.ts
export type InstallMethod = "apt" | "flatpak" | "snap";

export interface AppCatalogEntry {
  id: string;
  name: string;
  description: string;
  icon: string;
  category: string;
  installMethod: InstallMethod;
  /** Package name/id passed to the install command. For "apt" entries this
   *  is the package name used with whichever native manager is actually
   *  detected on the host (apt/dnf/pacman/zypper) — chosen from apps whose
   *  package name is consistent across those distros' default repos. If a
   *  given host happens to differ, `install_package` surfaces the real
   *  error from the package manager rather than failing silently. */
  packageId: string;
}

export const appCatalog: AppCatalogEntry[] = [
  { id: "firefox", name: "Firefox", description: "Navigateur web libre et rapide.", icon: "🦊", category: "Navigateurs", installMethod: "apt", packageId: "firefox" },
  { id: "chromium", name: "Chromium", description: "Navigateur web open source basé sur le projet Chromium.", icon: "🌐", category: "Navigateurs", installMethod: "apt", packageId: "chromium" },
  { id: "thunderbird", name: "Thunderbird", description: "Client de messagerie complet.", icon: "📧", category: "Communication", installMethod: "apt", packageId: "thunderbird" },
  { id: "discord", name: "Discord", description: "Messagerie vocale et textuelle pour communautés.", icon: "🎮", category: "Communication", installMethod: "flatpak", packageId: "com.discordapp.Discord" },
  { id: "libreoffice", name: "LibreOffice", description: "Suite bureautique complète (texte, tableur, présentation).", icon: "📄", category: "Bureautique", installMethod: "apt", packageId: "libreoffice" },
  { id: "gimp", name: "GIMP", description: "Éditeur d'images professionnel.", icon: "🎨", category: "Média", installMethod: "apt", packageId: "gimp" },
  { id: "inkscape", name: "Inkscape", description: "Éditeur de graphiques vectoriels.", icon: "✏️", category: "Média", installMethod: "apt", packageId: "inkscape" },
  { id: "blender", name: "Blender", description: "Suite de création 3D complète.", icon: "🧊", category: "Média", installMethod: "apt", packageId: "blender" },
  { id: "vlc", name: "VLC", description: "Lecteur multimédia universel.", icon: "🎬", category: "Média", installMethod: "apt", packageId: "vlc" },
  { id: "audacity", name: "Audacity", description: "Éditeur audio multipiste.", icon: "🎵", category: "Média", installMethod: "apt", packageId: "audacity" },
  { id: "obs-studio", name: "OBS Studio", description: "Capture et diffusion vidéo en direct.", icon: "📹", category: "Média", installMethod: "apt", packageId: "obs-studio" },
  { id: "spotify", name: "Spotify", description: "Streaming musical.", icon: "🎧", category: "Média", installMethod: "snap", packageId: "spotify" },
  { id: "steam", name: "Steam", description: "Plateforme de jeux vidéo.", icon: "🕹️", category: "Jeux", installMethod: "flatpak", packageId: "com.valvesoftware.Steam" },
  { id: "keepassxc", name: "KeePassXC", description: "Gestionnaire de mots de passe hors ligne.", icon: "🔐", category: "Utilitaires", installMethod: "apt", packageId: "keepassxc" },
  { id: "htop", name: "htop", description: "Moniteur de processus interactif en terminal.", icon: "📊", category: "Utilitaires", installMethod: "apt", packageId: "htop" },
  { id: "git", name: "Git", description: "Système de contrôle de version.", icon: "🔧", category: "Développement", installMethod: "apt", packageId: "git" },
];
```

- [ ] **Step 4: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npx vitest run src/data/appCatalog.spec.ts"`
Expected: PASS (3 tests)

- [ ] **Step 5: Commit**

```bash
git add src/data/appCatalog.ts src/data/appCatalog.spec.ts
git commit -m "feat: add curated app catalog data for quick-install (spec section 4.1)"
```

---

## Task 3: Frontend — `QuickInstallPage.vue`

**Files:**
- Create: `src/pages/QuickInstallPage.vue`
- Test: `src/pages/QuickInstallPage.spec.ts`

**Files this task reads but does not modify:** `src/data/appCatalog.ts` (Task 2), `src/components/ui/NxCard.vue`, `NxButton.vue`, `NxBadge.vue`, `NxSectionHeader.vue` (all from R1) — read each's `defineProps` first to confirm the prop names below still match.

- [ ] **Step 1: Write the failing tests**

```typescript
// src/pages/QuickInstallPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QuickInstallPage from "./QuickInstallPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "detect_native_manager") return Promise.resolve("apt");
    if (cmd === "install_package") {
      if (args?.package === "fail-me") return Promise.reject("apt: paquet introuvable");
      return Promise.resolve("Installation réussie");
    }
    return Promise.resolve(null);
  }),
}));

describe("QuickInstallPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("detects the native manager on mount and renders the catalog", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    expect(invoke).toHaveBeenCalledWith("detect_native_manager");
  });

  it("installs an apt-method app via install_package using the detected manager", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    const buttons = wrapper.findAll("button");
    const firefoxButton = buttons.find((b) => b.text() === "Installer" && b.element.closest(".qi-card")?.textContent?.includes("Firefox"))!;
    await firefoxButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "firefox" });
  });

  it("disables the install button and shows a not-yet-available badge for flatpak/snap entries", async () => {
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Discord"));
    const discordCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("Discord"))!;
    expect(discordCard.find("button[disabled]").exists()).toBe(true);
    expect(discordCard.text()).toContain("Bientôt disponible");
  });

  it("shows an error message when install_package rejects", async () => {
    // Temporarily monkeypatch the catalog entry under test would require
    // editing appCatalog, which this task must not do — instead this test
    // exercises the real Firefox entry but forces a rejection by using the
    // shared mock's `args?.package === "fail-me"` branch is not reachable
    // for Firefox (whose real packageId is "firefox"), so instead assert
    // the general error-handling path using GIMP by having the mock reject
    // for it specifically.
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "detect_native_manager") return Promise.resolve("apt");
      if (cmd === "install_package" && args?.package === "gimp") return Promise.reject("apt: échec de l'installation");
      return Promise.resolve("ok");
    });
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("GIMP"));
    const gimpCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("GIMP"))!;
    await gimpCard.find("button").trigger("click");
    await vi.waitFor(() => expect(gimpCard.text()).toContain("apt: échec de l'installation"));
  });

  it("filters the catalog by category", async () => {
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    const chips = wrapper.findAll(".qi-chip");
    const jeuxChip = chips.find((c) => c.text() === "Jeux")!;
    await jeuxChip.trigger("click");
    expect(wrapper.text()).toContain("Steam");
    expect(wrapper.text()).not.toContain("Firefox");
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npx vitest run src/pages/QuickInstallPage.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 3: Write `QuickInstallPage.vue`**

```vue
<!-- src/pages/QuickInstallPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { appCatalog, type AppCatalogEntry } from "@/data/appCatalog";

type InstallState = "idle" | "installing" | "success" | "error";

const nativeManager = ref<string | null>(null);
const selectedCategory = ref<string>("Tous");
const installState = ref<Record<string, InstallState>>({});
const installErrors = ref<Record<string, string>>({});

onMounted(async () => {
  nativeManager.value = await invoke<string | null>("detect_native_manager");
});

const categories = computed(() => ["Tous", ...new Set(appCatalog.map((e) => e.category))]);

const filteredCatalog = computed<AppCatalogEntry[]>(() =>
  selectedCategory.value === "Tous" ? appCatalog : appCatalog.filter((e) => e.category === selectedCategory.value),
);

function stateOf(entry: AppCatalogEntry): InstallState {
  return installState.value[entry.id] ?? "idle";
}

async function install(entry: AppCatalogEntry) {
  if (entry.installMethod !== "apt" || !nativeManager.value) return;
  installState.value[entry.id] = "installing";
  delete installErrors.value[entry.id];
  try {
    await invoke<string>("install_package", { manager: nativeManager.value, package: entry.packageId });
    installState.value[entry.id] = "success";
  } catch (e) {
    installState.value[entry.id] = "error";
    installErrors.value[entry.id] = String(e);
  }
}
</script>

<template>
  <div class="qi-page">
    <NxSectionHeader
      title="Installation rapide"
      :description="nativeManager ? `Gestionnaire détecté : ${nativeManager}` : 'Détection du gestionnaire de paquets...'"
    />

    <div class="qi-chips">
      <button
        v-for="cat in categories"
        :key="cat"
        class="qi-chip"
        :class="{ active: selectedCategory === cat }"
        @click="selectedCategory = cat"
      >
        {{ cat }}
      </button>
    </div>

    <div class="qi-grid">
      <NxCard v-for="entry in filteredCatalog" :key="entry.id" class="qi-card">
        <div class="qi-card-header">
          <span class="qi-icon">{{ entry.icon }}</span>
          <div>
            <div class="qi-name">{{ entry.name }}</div>
            <div class="qi-desc">{{ entry.description }}</div>
          </div>
        </div>

        <template v-if="entry.installMethod !== 'apt'">
          <NxBadge status="info">Bientôt disponible ({{ entry.installMethod }})</NxBadge>
          <NxButton disabled>Installer</NxButton>
        </template>
        <template v-else-if="stateOf(entry) === 'success'">
          <NxBadge status="success">Installé</NxBadge>
        </template>
        <template v-else>
          <div v-if="stateOf(entry) === 'installing'" class="qi-progress"><div class="qi-progress-bar"></div></div>
          <NxCard v-if="stateOf(entry) === 'error'" danger class="qi-error">{{ installErrors[entry.id] }}</NxCard>
          <NxButton :disabled="stateOf(entry) === 'installing' || !nativeManager" @click="install(entry)">
            {{ stateOf(entry) === "installing" ? "Installation..." : "Installer" }}
          </NxButton>
        </template>
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.qi-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.qi-chips { display: flex; gap: 8px; flex-wrap: wrap; }
.qi-chip { padding: 6px 14px; border-radius: 99px; border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; font-size: 12px; }
.qi-chip.active { color: var(--nx-text-primary); font-weight: 600; border-color: var(--nx-accent-primary); }
.qi-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; }
.qi-card { display: flex; flex-direction: column; gap: 10px; }
.qi-card-header { display: flex; gap: 10px; align-items: flex-start; }
.qi-icon { font-size: 28px; line-height: 1; }
.qi-name { font-weight: 600; color: var(--nx-text-primary); }
.qi-desc { font-size: 12px; color: var(--nx-text-secondary); }
.qi-progress { width: 100%; height: 4px; border-radius: 2px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.qi-progress-bar { width: 40%; height: 100%; background: var(--nx-accent-primary); border-radius: 2px; animation: qi-slide 1.2s ease-in-out infinite; }
.qi-error { font-size: 12px; padding: 8px 10px; }
@keyframes qi-slide {
  0% { transform: translateX(-100%); }
  100% { transform: translateX(350%); }
}
</style>
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npx vitest run src/pages/QuickInstallPage.spec.ts"`
Expected: PASS (5 tests)

- [ ] **Step 5: Commit**

```bash
git add src/pages/QuickInstallPage.vue src/pages/QuickInstallPage.spec.ts
git commit -m "feat: add QuickInstallPage — curated one-click app catalog (spec section 4.1)"
```

---

## Task 4: Wire `App.vue` to the real `QuickInstallPage`, retire the placeholder

**Files:**
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`
- Delete: `src/pages/QuickInstallPlaceholder.vue` (`git rm`)

Read the live `src/App.vue` and `src/App.spec.ts` first — this task only touches the `quick-install` entry point, nothing else in either file.

- [ ] **Step 1: Update the failing-first `App.spec.ts` assertion**

The existing test named `"shows the ComingSoonPage for the not-yet-implemented quick-install id"` is no longer accurate once this task lands — `quick-install` stops being a placeholder. Replace that one test (leave the other 3 `App.spec.ts` tests untouched) with:

```typescript
  it("shows the real QuickInstallPage (not a placeholder) for the quick-install id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const quickInstallButton = buttons.find((b) => b.text() === "Installation rapide")!;
    await quickInstallButton.trigger("click");
    expect(wrapper.text()).not.toContain("prévu pour Phase R3");
  });
```

(`"prévu pour Phase R3"` is `ComingSoonPage.vue`'s fixed placeholder text — its absence is the signal the real page rendered instead. This assertion doesn't depend on `QuickInstallPage`'s async `detect_native_manager` call resolving within the test, unlike asserting for "Firefox" would — `App.spec.ts`'s `invoke` mock in this file resolves `null` for every command including `detect_native_manager`, which `QuickInstallPage.vue` handles gracefully per Task 3's `v-if="!nativeManager"`-guarded button disabling, not a crash.)

- [ ] **Step 2: Run it to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npx vitest run src/App.spec.ts"`
Expected: FAIL — `App.vue` still maps `quick-install` to the placeholder.

- [ ] **Step 3: Update `App.vue`**

Replace the `QuickInstallPlaceholder` import with `QuickInstallPage`, and update the map entry:

```typescript
import QuickInstallPage from "@/pages/QuickInstallPage.vue";
```
(replaces `import QuickInstallPlaceholder from "@/pages/QuickInstallPlaceholder.vue";`)

```typescript
  "quick-install": QuickInstallPage,
```
(replaces `"quick-install": QuickInstallPlaceholder,` — every other line of the `pages` map and the rest of the file is unchanged)

- [ ] **Step 4: Delete the placeholder file**

```bash
git rm src/pages/QuickInstallPlaceholder.vue
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npx vitest run src/App.spec.ts"`
Expected: PASS (4 tests, same count as before — one test's assertion changed, no test was added or removed)

- [ ] **Step 6: Commit**

```bash
git add src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue to the real QuickInstallPage, retire the placeholder (spec section 4.1)"
```

---

## Task 5: Full verification pass

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npm run test -- --run 2>&1 | tail -20"`
Expected: every test file passes. Baseline entering this plan was 111 (end of R2). This plan adds 3 (`appCatalog.spec.ts`) + 5 (`QuickInstallPage.spec.ts`) = 8, with `App.spec.ts`'s total test count unchanged (one assertion swapped, not added) — expected total 119. Report the real observed number, don't just assert 119.

- [ ] **Step 2: Type-check**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && npx vue-tsc --noEmit"`
Expected: clean.

- [ ] **Step 3: Confirm the Rust suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install/src-tauri && cargo test 2>&1 | tail -10"`
Expected: `125 passed; 0 failed; 1 ignored` — 124 (R2 baseline) + 0 new tests (Task 1's `detect_native_manager` deliberately has none, per its own doc comment) is actually still 124; if the real count differs from 124, investigate why before proceeding — it should NOT differ, since Task 1 adds no test.

- [ ] **Step 4: Confirm `QuickInstallPlaceholder.vue` is fully gone**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r3-quick-install && grep -rn 'QuickInstallPlaceholder' src/ || echo 'no references found'"`
Expected: `no references found`.

- [ ] **Step 5: Manual smoke check of `App.vue`'s page map**

Confirm `src/App.vue`'s `pages` record still has exactly 15 entries (same count as end of R2 — this plan swaps one value, not the key set) and that `"quick-install"` now points at `QuickInstallPage`, not a placeholder.

- [ ] **Step 6: Commit any final cleanup**

No further commit expected if Steps 1–5 all pass clean.

---

## Self-Review

**Spec coverage:** §4.1's every named element is covered — curated catalog data structure (Task 2), v1 apt-only install scope with flatpak/snap shown-but-disabled (Task 3), reuse of the already-VM-verified `install_package` command with zero new privileged surface (Task 1 adds only a read-only detection command), indeterminate progress bar during install (Task 3's `.qi-progress`/`.qi-progress-bar` CSS animation), success/error state on completion (Task 3's `installState`/`installErrors`).

**Placeholder scan:** No "TBD"/"TODO". Task 1's missing dedicated Rust test is explicitly justified (mirrors `list_updates`'s existing precedent for the same reason), not an oversight.

**Type consistency:** `AppCatalogEntry` (Task 2) fields (`id`, `name`, `description`, `icon`, `category`, `installMethod`, `packageId`) are used identically in `QuickInstallPage.vue` (Task 3) — cross-checked. `Nx*` component prop names (`NxCard`'s `danger`, `NxButton`'s `disabled`, `NxBadge`'s `status`, `NxSectionHeader`'s `title`/`description`) match their R1 `defineProps` exactly, same as every prior phase's plans. `detect_native_manager`'s return type `Option<String>` (Rust, Task 1) maps to TypeScript `string | null` (Task 3's `invoke<string | null>`), the same pattern already used elsewhere in this codebase for optional Tauri return values.
