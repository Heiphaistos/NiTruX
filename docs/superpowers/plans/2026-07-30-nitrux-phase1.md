# NiTruX Phase 1 — Fondations Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Stand up the NiTruX Tauri v2 shell — branding, the 12-theme × 8-layout engine with a real-time editor, and the "Système & diagnostic" functional pillar (dashboard, hardware, drivers, logs) — as working, testable software.

**Architecture:** Tauri v2 + Rust backend (one command module per data source: `system.rs`, `sensors.rs`, `hardware.rs`, `drivers.rs`, `logs.rs`) exposing typed commands to a Vue 3 + TypeScript + Pinia frontend. Theming is CSS-custom-properties driven (a `Theme` object maps to `--nx-*` variables applied to `:root`). Layout is a component-switch pattern: `LayoutShell.vue` renders one of 8 interchangeable shell components based on the active `layoutStore.current`, each shell exposing the same two named slots (`nav`, `default`).

**Tech Stack:** Tauri v2, Rust (`sysinfo`, `serde`, `serde_json`), Vue 3.5, TypeScript, Pinia, Vite, Vitest, `@vue/test-utils`.

**Scope note:** Spec §5.1 lists benchmark as part of the Système & diagnostic pillar. It is deferred out of this Phase 1 plan (13 tasks is already a full, shippable increment) to a short fast-follow plan once Dashboard/Hardware/Drivers/Logs have landed and been used for a few days — benchmarking is independent of the theme/layout engine and doesn't block anything else in Phase 2.

---

## File Structure

```
NiTruX/
├── src/
│   ├── types/
│   │   ├── theme.ts              # Theme, ThemeColors interfaces
│   │   └── layout.ts             # LayoutId, LayoutDefinition
│   ├── themes/
│   │   └── builtin.ts            # 12 builtin Theme objects
│   ├── layouts/
│   │   ├── registry.ts           # LayoutDefinition[] metadata (8 entries)
│   │   ├── LayoutShell.vue       # dynamic switcher
│   │   ├── SidebarClassicLayout.vue
│   │   ├── WidgetsGridLayout.vue
│   │   ├── CommandFirstLayout.vue
│   │   ├── CompactSidebarLayout.vue
│   │   ├── TopNavLayout.vue
│   │   ├── MasterDetailLayout.vue
│   │   ├── BentoLayout.vue
│   │   └── FloatingDockLayout.vue
│   ├── stores/
│   │   ├── themeStore.ts         # active theme, custom themes, apply-to-DOM
│   │   └── layoutStore.ts        # active layout id, persistence
│   ├── pages/
│   │   ├── ThemeEditorPage.vue   # real-time theme + layout editor
│   │   ├── DashboardPage.vue     # CPU/RAM/processes overview
│   │   ├── HardwarePage.vue      # lspci/dmidecode component tree
│   │   ├── DriversPage.vue       # kernel modules + GPU driver
│   │   └── LogsPage.vue          # journalctl viewer
│   └── App.vue                   # mounts LayoutShell, applies theme on boot
├── src-tauri/
│   ├── icons/                    # copied from NiTriTe
│   ├── src/
│   │   ├── system.rs             # sysinfo snapshot command
│   │   ├── sensors.rs            # temperature + battery commands
│   │   ├── hardware.rs           # lspci/dmidecode parsing + command
│   │   ├── drivers.rs            # lsmod/modinfo + GPU driver detection
│   │   ├── logs.rs               # journalctl command
│   │   └── lib.rs                # command registration
│   └── tauri.conf.json
└── tests/
    └── (vitest specs colocated as *.spec.ts next to source)
```

---

## Task 1: Scaffold Tauri v2 project

**Files:**
- Create: whole project via `create-tauri-app` (package.json, src-tauri/, src/, vite.config.ts, tsconfig.json)

- [ ] **Step 1: Scaffold in WSL2**

Run (inside WSL2 Ubuntu, in `~/projects/` or a path synced to `/mnt/c/Users/Momo/Desktop/NiTruX`):

```bash
cd /mnt/c/Users/Momo/Desktop
npm create tauri-app@latest NiTruX -- --template vue-ts --manager npm
cd NiTruX
npm install
```

- [ ] **Step 2: Add Pinia and Vitest**

```bash
npm install pinia
npm install -D vitest @vue/test-utils jsdom
```

- [ ] **Step 3: Configure Vitest**

Create `vitest.config.ts`:

```ts
import { defineConfig } from "vitest/config";
import vue from "@vitejs/plugin-vue";

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "jsdom",
    globals: true,
  },
});
```

Add to `package.json` scripts:

```json
"test": "vitest run"
```

- [ ] **Step 4: Verify dev build runs**

Run: `npm run tauri dev`
Expected: Tauri window opens showing the default template page (via WSLg), no errors in terminal.

Stop the dev server (Ctrl+C) before continuing.

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "chore: scaffold Tauri v2 + Vue3 + TS project"
```

---

## Task 2: Integrate NiTriTe icon set

**Files:**
- Copy: `C:\Users\Momo\Desktop\Nitrite 2.0\src-tauri\icons\128x128.png` → `src-tauri/icons/128x128.png`
- Copy: `C:\Users\Momo\Desktop\Nitrite 2.0\src-tauri\icons\128x128@2x.png` → `src-tauri/icons/128x128@2x.png`
- Copy: `C:\Users\Momo\Desktop\Nitrite 2.0\src-tauri\icons\32x32.png` → `src-tauri/icons/32x32.png`
- Copy: `C:\Users\Momo\Desktop\Nitrite 2.0\src-tauri\icons\icon.ico` → `src-tauri/icons/icon.ico`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Copy icon files**

```bash
cp "/mnt/c/Users/Momo/Desktop/Nitrite 2.0/src-tauri/icons/128x128.png" src-tauri/icons/128x128.png
cp "/mnt/c/Users/Momo/Desktop/Nitrite 2.0/src-tauri/icons/128x128@2x.png" src-tauri/icons/128x128@2x.png
cp "/mnt/c/Users/Momo/Desktop/Nitrite 2.0/src-tauri/icons/32x32.png" src-tauri/icons/32x32.png
cp "/mnt/c/Users/Momo/Desktop/Nitrite 2.0/src-tauri/icons/icon.ico" src-tauri/icons/icon.ico
```

- [ ] **Step 2: Generate remaining Tauri icon sizes (Linux bundle needs more than 4 files)**

```bash
npx tauri icon src-tauri/icons/128x128@2x.png
```

This regenerates the full icon set (all PNG sizes + `.icns` + `.ico`) from the highest-res source, keeping the NiTriTe artwork.

- [ ] **Step 3: Update app identity in `tauri.conf.json`**

Modify the top-level fields:

```json
{
  "productName": "NiTruX",
  "identifier": "org.heiphaistos.nitrux",
  "app": {
    "windows": [
      {
        "title": "NiTruX",
        "width": 1280,
        "height": 800
      }
    ]
  }
}
```

- [ ] **Step 4: Verify build picks up new icon**

Run: `npm run tauri dev`
Expected: window title bar shows "NiTruX", taskbar/dock icon is the NiTriTe artwork (via WSLg).

- [ ] **Step 5: Commit**

```bash
git add src-tauri/icons src-tauri/tauri.conf.json
git commit -m "feat: brand NiTruX with NiTriTe icon set and app identity"
```

---

## Task 3: Theme types and 12 builtin palettes

**Files:**
- Create: `src/types/theme.ts`
- Create: `src/themes/builtin.ts`
- Test: `src/themes/builtin.spec.ts`

- [ ] **Step 1: Write the failing test**

```ts
// src/themes/builtin.spec.ts
import { describe, it, expect } from "vitest";
import { builtinThemes } from "./builtin";

describe("builtinThemes", () => {
  it("ships exactly 12 themes", () => {
    expect(builtinThemes).toHaveLength(12);
  });

  it("has unique ids", () => {
    const ids = builtinThemes.map((t) => t.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every theme defines all required color keys", () => {
    const requiredKeys = [
      "bgBase", "bgElevated", "bgOverlay", "border",
      "textPrimary", "textSecondary",
      "accentPrimary", "accentSecondary", "accentSuccess", "accentWarning", "accentDanger",
    ];
    for (const theme of builtinThemes) {
      for (const key of requiredKeys) {
        expect(theme.colors, `${theme.id} missing ${key}`).toHaveProperty(key);
      }
    }
  });

  it("every color is a valid hex string", () => {
    const hexRe = /^#[0-9a-fA-F]{6}$/;
    for (const theme of builtinThemes) {
      for (const [key, value] of Object.entries(theme.colors)) {
        expect(value, `${theme.id}.${key} = ${value}`).toMatch(hexRe);
      }
    }
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- src/themes/builtin.spec.ts`
Expected: FAIL — `Cannot find module './builtin'`

- [ ] **Step 3: Write the types**

```ts
// src/types/theme.ts
export interface ThemeColors {
  bgBase: string;
  bgElevated: string;
  bgOverlay: string;
  border: string;
  textPrimary: string;
  textSecondary: string;
  accentPrimary: string;
  accentSecondary: string;
  accentSuccess: string;
  accentWarning: string;
  accentDanger: string;
}

export interface Theme {
  id: string;
  name: string;
  mode: "dark" | "light";
  colors: ThemeColors;
}
```

- [ ] **Step 4: Write the 12 builtin themes**

```ts
// src/themes/builtin.ts
import type { Theme } from "@/types/theme";

export const builtinThemes: Theme[] = [
  {
    id: "catppuccin-mocha", name: "Catppuccin Mocha", mode: "dark",
    colors: {
      bgBase: "#1e1e2e", bgElevated: "#313244", bgOverlay: "#181825", border: "#45475a",
      textPrimary: "#cdd6f4", textSecondary: "#a6adc8",
      accentPrimary: "#89b4fa", accentSecondary: "#f5c2e7",
      accentSuccess: "#94e2d5", accentWarning: "#fab387", accentDanger: "#f38ba8",
    },
  },
  {
    id: "nord", name: "Nord", mode: "dark",
    colors: {
      bgBase: "#2e3440", bgElevated: "#3b4252", bgOverlay: "#242933", border: "#4c566a",
      textPrimary: "#eceff4", textSecondary: "#d8dee9",
      accentPrimary: "#88c0d0", accentSecondary: "#5e81ac",
      accentSuccess: "#a3be8c", accentWarning: "#ebcb8b", accentDanger: "#bf616a",
    },
  },
  {
    id: "adwaita", name: "Adwaita", mode: "light",
    colors: {
      bgBase: "#fafafa", bgElevated: "#ffffff", bgOverlay: "#f0f0f0", border: "#d8d8d8",
      textPrimary: "#241f31", textSecondary: "#5e5c64",
      accentPrimary: "#3584e4", accentSecondary: "#9141ac",
      accentSuccess: "#2ec27e", accentWarning: "#e5a50a", accentDanger: "#e01b24",
    },
  },
  {
    id: "gruvbox", name: "Gruvbox", mode: "dark",
    colors: {
      bgBase: "#282828", bgElevated: "#3c3836", bgOverlay: "#1d2021", border: "#504945",
      textPrimary: "#ebdbb2", textSecondary: "#bdae93",
      accentPrimary: "#fe8019", accentSecondary: "#d3869b",
      accentSuccess: "#b8bb26", accentWarning: "#fabd2f", accentDanger: "#fb4934",
    },
  },
  {
    id: "dracula", name: "Dracula", mode: "dark",
    colors: {
      bgBase: "#282a36", bgElevated: "#44475a", bgOverlay: "#21222c", border: "#6272a4",
      textPrimary: "#f8f8f2", textSecondary: "#bfbfd4",
      accentPrimary: "#bd93f9", accentSecondary: "#ff79c6",
      accentSuccess: "#50fa7b", accentWarning: "#f1fa8c", accentDanger: "#ff5555",
    },
  },
  {
    id: "everforest", name: "Everforest", mode: "dark",
    colors: {
      bgBase: "#2d353b", bgElevated: "#3d484d", bgOverlay: "#232a2e", border: "#4f5b58",
      textPrimary: "#d3c6aa", textSecondary: "#a6b0a0",
      accentPrimary: "#a7c080", accentSecondary: "#dbbc7f",
      accentSuccess: "#83c092", accentWarning: "#e69875", accentDanger: "#e67e80",
    },
  },
  {
    id: "tokyo-night", name: "Tokyo Night", mode: "dark",
    colors: {
      bgBase: "#1a1b26", bgElevated: "#24283b", bgOverlay: "#16161e", border: "#3b4261",
      textPrimary: "#c0caf5", textSecondary: "#9aa5ce",
      accentPrimary: "#7aa2f7", accentSecondary: "#bb9af7",
      accentSuccess: "#9ece6a", accentWarning: "#e0af68", accentDanger: "#f7768e",
    },
  },
  {
    id: "solarized", name: "Solarized", mode: "light",
    colors: {
      bgBase: "#fdf6e3", bgElevated: "#eee8d5", bgOverlay: "#e4ddc4", border: "#93a1a1",
      textPrimary: "#073642", textSecondary: "#586e75",
      accentPrimary: "#268bd2", accentSecondary: "#6c71c4",
      accentSuccess: "#2aa198", accentWarning: "#b58900", accentDanger: "#dc322f",
    },
  },
  {
    id: "rose-pine", name: "Rosé Pine", mode: "dark",
    colors: {
      bgBase: "#191724", bgElevated: "#1f1d2e", bgOverlay: "#26233a", border: "#403d52",
      textPrimary: "#e0def4", textSecondary: "#908caa",
      accentPrimary: "#c4a7e7", accentSecondary: "#ebbcba",
      accentSuccess: "#9ccfd8", accentWarning: "#f6c177", accentDanger: "#eb6f92",
    },
  },
  {
    id: "one-dark", name: "One Dark", mode: "dark",
    colors: {
      bgBase: "#282c34", bgElevated: "#2c313a", bgOverlay: "#21252b", border: "#3e4451",
      textPrimary: "#abb2bf", textSecondary: "#828997",
      accentPrimary: "#61afef", accentSecondary: "#c678dd",
      accentSuccess: "#98c379", accentWarning: "#e5c07b", accentDanger: "#e06c75",
    },
  },
  {
    id: "kanagawa", name: "Kanagawa", mode: "dark",
    colors: {
      bgBase: "#1f1f28", bgElevated: "#2a2a37", bgOverlay: "#16161d", border: "#54546d",
      textPrimary: "#dcd7ba", textSecondary: "#a6a69c",
      accentPrimary: "#7e9cd8", accentSecondary: "#957fb8",
      accentSuccess: "#98bb6c", accentWarning: "#e6c384", accentDanger: "#ff5d62",
    },
  },
  {
    id: "ayu", name: "Ayu", mode: "dark",
    colors: {
      bgBase: "#0a0e14", bgElevated: "#131721", bgOverlay: "#060a10", border: "#232834",
      textPrimary: "#b3b1ad", textSecondary: "#828282",
      accentPrimary: "#39bae6", accentSecondary: "#ffb454",
      accentSuccess: "#c2d94c", accentWarning: "#ffb454", accentDanger: "#f07178",
    },
  },
];
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npm run test -- src/themes/builtin.spec.ts`
Expected: PASS (4 tests)

- [ ] **Step 6: Commit**

```bash
git add src/types/theme.ts src/themes/builtin.ts src/themes/builtin.spec.ts
git commit -m "feat: add theme types and 12 builtin color palettes"
```

---

## Task 4: Theme store — apply theme to DOM via CSS custom properties

**Files:**
- Create: `src/stores/themeStore.ts`
- Test: `src/stores/themeStore.spec.ts`

- [ ] **Step 1: Write the failing test**

```ts
// src/stores/themeStore.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useThemeStore } from "./themeStore";
import { builtinThemes } from "@/themes/builtin";

describe("themeStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    document.documentElement.removeAttribute("style");
  });

  it("defaults to the first builtin theme", () => {
    const store = useThemeStore();
    expect(store.active.id).toBe(builtinThemes[0].id);
  });

  it("applies theme colors as --nx- CSS custom properties on :root", () => {
    const store = useThemeStore();
    const dracula = builtinThemes.find((t) => t.id === "dracula")!;
    store.setTheme(dracula);
    const root = document.documentElement;
    expect(root.style.getPropertyValue("--nx-bg-base").trim()).toBe(dracula.colors.bgBase);
    expect(root.style.getPropertyValue("--nx-accent-primary").trim()).toBe(dracula.colors.accentPrimary);
  });

  it("registers and lists custom themes", () => {
    const store = useThemeStore();
    const custom = {
      id: "my-custom", name: "My Custom", mode: "dark" as const,
      colors: builtinThemes[0].colors,
    };
    store.saveCustomTheme(custom);
    expect(store.customThemes.map((t) => t.id)).toContain("my-custom");
  });

  it("exports the active theme as JSON and re-imports it", () => {
    const store = useThemeStore();
    store.setTheme(builtinThemes[1]);
    const json = store.exportActiveTheme();
    const imported = store.importTheme(json);
    expect(imported.ok).toBe(true);
    expect(store.customThemes.some((t) => t.id === builtinThemes[1].id)).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- src/stores/themeStore.spec.ts`
Expected: FAIL — `Cannot find module './themeStore'`

- [ ] **Step 3: Write the store**

```ts
// src/stores/themeStore.ts
import { defineStore } from "pinia";
import type { Theme } from "@/types/theme";
import { builtinThemes } from "@/themes/builtin";

const CSS_VAR_MAP: Record<keyof Theme["colors"], string> = {
  bgBase: "--nx-bg-base",
  bgElevated: "--nx-bg-elevated",
  bgOverlay: "--nx-bg-overlay",
  border: "--nx-border",
  textPrimary: "--nx-text-primary",
  textSecondary: "--nx-text-secondary",
  accentPrimary: "--nx-accent-primary",
  accentSecondary: "--nx-accent-secondary",
  accentSuccess: "--nx-accent-success",
  accentWarning: "--nx-accent-warning",
  accentDanger: "--nx-accent-danger",
};

function applyToDom(theme: Theme) {
  const root = document.documentElement;
  for (const [key, cssVar] of Object.entries(CSS_VAR_MAP)) {
    root.style.setProperty(cssVar, theme.colors[key as keyof Theme["colors"]]);
  }
  root.dataset.themeMode = theme.mode;
}

export const useThemeStore = defineStore("theme", {
  state: () => ({
    active: builtinThemes[0] as Theme,
    customThemes: [] as Theme[],
  }),
  actions: {
    setTheme(theme: Theme) {
      this.active = theme;
      applyToDom(theme);
    },
    saveCustomTheme(theme: Theme) {
      const existingIndex = this.customThemes.findIndex((t) => t.id === theme.id);
      if (existingIndex >= 0) this.customThemes.splice(existingIndex, 1, theme);
      else this.customThemes.push(theme);
    },
    exportActiveTheme(): string {
      return JSON.stringify(this.active, null, 2);
    },
    importTheme(json: string): { ok: true } | { ok: false; error: string } {
      try {
        const parsed = JSON.parse(json) as Theme;
        if (!parsed.id || !parsed.colors) {
          return { ok: false, error: "Fichier de thème invalide : id ou colors manquant." };
        }
        this.saveCustomTheme(parsed);
        return { ok: true };
      } catch {
        return { ok: false, error: "JSON invalide." };
      }
    },
  },
});
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test -- src/stores/themeStore.spec.ts`
Expected: PASS (4 tests)

- [ ] **Step 5: Apply the default theme on app boot**

Modify `src/App.vue` — add to `<script setup>`:

```ts
import { onMounted } from "vue";
import { useThemeStore } from "@/stores/themeStore";

const themeStore = useThemeStore();
onMounted(() => themeStore.setTheme(themeStore.active));
```

- [ ] **Step 6: Commit**

```bash
git add src/stores/themeStore.ts src/stores/themeStore.spec.ts src/App.vue
git commit -m "feat: theme store applying palettes as CSS custom properties"
```

---

## Task 5: Layout types, registry, and store

**Files:**
- Create: `src/types/layout.ts`
- Create: `src/layouts/registry.ts`
- Create: `src/stores/layoutStore.ts`
- Test: `src/layouts/registry.spec.ts`
- Test: `src/stores/layoutStore.spec.ts`

- [ ] **Step 1: Write the failing tests**

```ts
// src/layouts/registry.spec.ts
import { describe, it, expect } from "vitest";
import { layoutRegistry } from "./registry";

describe("layoutRegistry", () => {
  it("lists exactly 8 layouts", () => {
    expect(layoutRegistry).toHaveLength(8);
  });

  it("has unique ids", () => {
    const ids = layoutRegistry.map((l) => l.id);
    expect(new Set(ids).size).toBe(ids.length);
  });
});
```

```ts
// src/stores/layoutStore.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useLayoutStore } from "./layoutStore";

describe("layoutStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("defaults to sidebar-classic", () => {
    const store = useLayoutStore();
    expect(store.current).toBe("sidebar-classic");
  });

  it("persists the chosen layout to localStorage", () => {
    const store = useLayoutStore();
    store.setLayout("bento");
    expect(localStorage.getItem("nitrux-layout")).toBe("bento");
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `npm run test -- src/layouts/registry.spec.ts src/stores/layoutStore.spec.ts`
Expected: FAIL — modules not found

- [ ] **Step 3: Write the types and registry**

```ts
// src/types/layout.ts
export type LayoutId =
  | "sidebar-classic" | "widgets-grid" | "command-first" | "compact-sidebar"
  | "top-nav" | "master-detail" | "bento" | "floating-dock";

export interface LayoutDefinition {
  id: LayoutId;
  name: string;
  description: string;
}
```

```ts
// src/layouts/registry.ts
import type { LayoutDefinition } from "@/types/layout";

export const layoutRegistry: LayoutDefinition[] = [
  { id: "sidebar-classic", name: "Sidebar classique", description: "Navigation latérale fixe, zone de contenu principale." },
  { id: "widgets-grid", name: "Dashboard modulaire", description: "Grille de cartes réarrangeables sur l'accueil." },
  { id: "command-first", name: "Command palette-first", description: "Recherche/commande centrale, sidebar réduite." },
  { id: "compact-sidebar", name: "Sidebar rétractable", description: "Bande d'icônes, extension au survol." },
  { id: "top-nav", name: "Barre supérieure + onglets", description: "Navigation horizontale, contenu plein écran." },
  { id: "master-detail", name: "Master-detail", description: "Liste étroite à gauche, panneau détail à droite." },
  { id: "bento", name: "Bento grid", description: "Accueil en grille asymétrique." },
  { id: "floating-dock", name: "Dock flottant", description: "Contenu plein-bleed, navigation en dock flottant." },
];
```

- [ ] **Step 4: Write the layout store**

```ts
// src/stores/layoutStore.ts
import { defineStore } from "pinia";
import type { LayoutId } from "@/types/layout";

const STORAGE_KEY = "nitrux-layout";

export const useLayoutStore = defineStore("layout", {
  state: () => ({
    current: (localStorage.getItem(STORAGE_KEY) as LayoutId | null) ?? ("sidebar-classic" as LayoutId),
  }),
  actions: {
    setLayout(id: LayoutId) {
      this.current = id;
      localStorage.setItem(STORAGE_KEY, id);
    },
  },
});
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `npm run test -- src/layouts/registry.spec.ts src/stores/layoutStore.spec.ts`
Expected: PASS (4 tests)

- [ ] **Step 6: Commit**

```bash
git add src/types/layout.ts src/layouts/registry.ts src/stores/layoutStore.ts src/layouts/registry.spec.ts src/stores/layoutStore.spec.ts
git commit -m "feat: layout registry (8 dispositions) and persisted layout store"
```

---

## Task 6: 8 layout shell components + dynamic LayoutShell switcher

**Files:**
- Create: `src/layouts/SidebarClassicLayout.vue`
- Create: `src/layouts/WidgetsGridLayout.vue`
- Create: `src/layouts/CommandFirstLayout.vue`
- Create: `src/layouts/CompactSidebarLayout.vue`
- Create: `src/layouts/TopNavLayout.vue`
- Create: `src/layouts/MasterDetailLayout.vue`
- Create: `src/layouts/BentoLayout.vue`
- Create: `src/layouts/FloatingDockLayout.vue`
- Create: `src/layouts/LayoutShell.vue`
- Test: `src/layouts/LayoutShell.spec.ts`

Every shell component below exposes the same contract: a `nav` named slot and the `default` slot for page content. This keeps `LayoutShell.vue` and every page agnostic of which disposition is active.

- [ ] **Step 1: Write the failing test**

```ts
// src/layouts/LayoutShell.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import { useLayoutStore } from "@/stores/layoutStore";
import { layoutRegistry } from "./registry";
import LayoutShell from "./LayoutShell.vue";

describe("LayoutShell", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("renders default slot content under sidebar-classic", () => {
    const wrapper = mount(LayoutShell, {
      slots: { default: "<div class=\"probe\">content</div>", nav: "<div class=\"nav-probe\" />" },
    });
    expect(wrapper.find(".probe").exists()).toBe(true);
    expect(wrapper.find(".nav-probe").exists()).toBe(true);
  });

  it.each(layoutRegistry.map((l) => l.id))("renders default slot content under %s", async (id) => {
    const store = useLayoutStore();
    store.setLayout(id);
    const wrapper = mount(LayoutShell, {
      slots: { default: "<div class=\"probe\">content</div>", nav: "<div class=\"nav-probe\" />" },
    });
    expect(wrapper.find(".probe").exists()).toBe(true);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- src/layouts/LayoutShell.spec.ts`
Expected: FAIL — `Cannot find module './LayoutShell.vue'`

- [ ] **Step 3: Write the 8 shell components**

```vue
<!-- src/layouts/SidebarClassicLayout.vue -->
<template>
  <div class="nx-layout nx-layout--sidebar-classic">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--sidebar-classic { display: flex; height: 100vh; }
.nx-nav { width: 220px; flex-shrink: 0; background: var(--nx-bg-overlay); border-right: 1px solid var(--nx-border); }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
```

```vue
<!-- src/layouts/WidgetsGridLayout.vue -->
<template>
  <div class="nx-layout nx-layout--widgets-grid">
    <header class="nx-nav"><slot name="nav" /></header>
    <main class="nx-content nx-content--grid"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--widgets-grid { display: flex; flex-direction: column; height: 100vh; }
.nx-nav { padding: 12px 20px; border-bottom: 1px solid var(--nx-border); background: var(--nx-bg-overlay); }
.nx-content--grid { flex: 1; overflow: auto; padding: 20px; display: grid; grid-template-columns: repeat(auto-fill, minmax(240px, 1fr)); gap: 16px; align-content: start; }
</style>
```

```vue
<!-- src/layouts/CommandFirstLayout.vue -->
<template>
  <div class="nx-layout nx-layout--command-first">
    <div class="nx-command-bar"><slot name="nav" /></div>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--command-first { display: flex; flex-direction: column; height: 100vh; align-items: center; }
.nx-command-bar { width: min(640px, 90%); margin-top: 24px; padding: 10px 16px; border-radius: 12px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); }
.nx-content { flex: 1; width: 100%; overflow: auto; padding: 24px; }
</style>
```

```vue
<!-- src/layouts/CompactSidebarLayout.vue -->
<template>
  <div class="nx-layout nx-layout--compact-sidebar">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--compact-sidebar { display: flex; height: 100vh; }
.nx-nav { width: 56px; flex-shrink: 0; overflow: hidden; transition: width 0.15s ease; background: var(--nx-bg-overlay); border-right: 1px solid var(--nx-border); }
.nx-nav:hover { width: 220px; }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
```

```vue
<!-- src/layouts/TopNavLayout.vue -->
<template>
  <div class="nx-layout nx-layout--top-nav">
    <header class="nx-nav"><slot name="nav" /></header>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--top-nav { display: flex; flex-direction: column; height: 100vh; }
.nx-nav { padding: 0 20px; height: 52px; display: flex; align-items: center; background: var(--nx-bg-overlay); border-bottom: 1px solid var(--nx-border); }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
```

```vue
<!-- src/layouts/MasterDetailLayout.vue -->
<template>
  <div class="nx-layout nx-layout--master-detail">
    <aside class="nx-nav"><slot name="nav" /></aside>
    <main class="nx-content"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--master-detail { display: flex; height: 100vh; }
.nx-nav { width: 280px; flex-shrink: 0; overflow: auto; background: var(--nx-bg-elevated); border-right: 1px solid var(--nx-border); }
.nx-content { flex: 1; overflow: auto; padding: 24px; }
</style>
```

```vue
<!-- src/layouts/BentoLayout.vue -->
<template>
  <div class="nx-layout nx-layout--bento">
    <header class="nx-nav"><slot name="nav" /></header>
    <main class="nx-content nx-content--bento"><slot /></main>
  </div>
</template>

<style scoped>
.nx-layout--bento { display: flex; flex-direction: column; height: 100vh; }
.nx-nav { padding: 12px 20px; border-bottom: 1px solid var(--nx-border); background: var(--nx-bg-overlay); }
.nx-content--bento { flex: 1; overflow: auto; padding: 20px; display: grid; grid-template-columns: repeat(4, 1fr); grid-auto-rows: 120px; gap: 14px; }
</style>
```

```vue
<!-- src/layouts/FloatingDockLayout.vue -->
<template>
  <div class="nx-layout nx-layout--floating-dock">
    <main class="nx-content"><slot /></main>
    <nav class="nx-dock"><slot name="nav" /></nav>
  </div>
</template>

<style scoped>
.nx-layout--floating-dock { position: relative; height: 100vh; }
.nx-content { height: 100%; overflow: auto; padding: 24px; padding-bottom: 88px; }
.nx-dock { position: absolute; bottom: 20px; left: 50%; transform: translateX(-50%); padding: 10px 16px; border-radius: 16px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); box-shadow: 0 8px 24px rgba(0,0,0,0.35); }
</style>
```

- [ ] **Step 4: Write the dynamic switcher**

```vue
<!-- src/layouts/LayoutShell.vue -->
<script setup lang="ts">
import { computed } from "vue";
import { useLayoutStore } from "@/stores/layoutStore";
import SidebarClassicLayout from "./SidebarClassicLayout.vue";
import WidgetsGridLayout from "./WidgetsGridLayout.vue";
import CommandFirstLayout from "./CommandFirstLayout.vue";
import CompactSidebarLayout from "./CompactSidebarLayout.vue";
import TopNavLayout from "./TopNavLayout.vue";
import MasterDetailLayout from "./MasterDetailLayout.vue";
import BentoLayout from "./BentoLayout.vue";
import FloatingDockLayout from "./FloatingDockLayout.vue";
import type { LayoutId } from "@/types/layout";

const layoutStore = useLayoutStore();

const componentMap: Record<LayoutId, unknown> = {
  "sidebar-classic": SidebarClassicLayout,
  "widgets-grid": WidgetsGridLayout,
  "command-first": CommandFirstLayout,
  "compact-sidebar": CompactSidebarLayout,
  "top-nav": TopNavLayout,
  "master-detail": MasterDetailLayout,
  "bento": BentoLayout,
  "floating-dock": FloatingDockLayout,
};

const activeComponent = computed(() => componentMap[layoutStore.current]);
</script>

<template>
  <component :is="activeComponent">
    <template #nav><slot name="nav" /></template>
    <slot />
  </component>
</template>
```

- [ ] **Step 5: Run test to verify it passes**

Run: `npm run test -- src/layouts/LayoutShell.spec.ts`
Expected: PASS (9 tests — 1 + 8 parametrized)

- [ ] **Step 6: Commit**

```bash
git add src/layouts/
git commit -m "feat: implement 8 layout shells and dynamic LayoutShell switcher"
```

---

## Task 7: Theme & Layout editor page

**Files:**
- Create: `src/pages/ThemeEditorPage.vue`
- Test: `src/stores/themeStore.editor.spec.ts`

The editor reuses `themeStore` (Task 4) and `layoutStore` (Task 5) directly — this task is the UI wiring plus one additional store action (`updateActiveColor`) needed for live color editing.

- [ ] **Step 1: Write the failing test for live color editing**

```ts
// src/stores/themeStore.editor.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useThemeStore } from "./themeStore";
import { builtinThemes } from "@/themes/builtin";

describe("themeStore.updateActiveColor", () => {
  beforeEach(() => setActivePinia(createPinia()));

  it("mutates a single color of the active theme and re-applies to DOM", () => {
    const store = useThemeStore();
    store.setTheme(builtinThemes[0]);
    store.updateActiveColor("accentPrimary", "#ff00ff");
    expect(store.active.colors.accentPrimary).toBe("#ff00ff");
    expect(document.documentElement.style.getPropertyValue("--nx-accent-primary").trim()).toBe("#ff00ff");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `npm run test -- src/stores/themeStore.editor.spec.ts`
Expected: FAIL — `store.updateActiveColor is not a function`

- [ ] **Step 3: Add the action to `themeStore.ts`**

Modify `src/stores/themeStore.ts` — add inside `actions`:

```ts
    updateActiveColor(key: keyof Theme["colors"], value: string) {
      this.active = { ...this.active, colors: { ...this.active.colors, [key]: value } };
      applyToDom(this.active);
    },
```

- [ ] **Step 4: Run test to verify it passes**

Run: `npm run test -- src/stores/themeStore.editor.spec.ts`
Expected: PASS

- [ ] **Step 5: Build the editor page**

```vue
<!-- src/pages/ThemeEditorPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { useThemeStore } from "@/stores/themeStore";
import { useLayoutStore } from "@/stores/layoutStore";
import { builtinThemes } from "@/themes/builtin";
import { layoutRegistry } from "@/layouts/registry";
import type { Theme } from "@/types/theme";

const themeStore = useThemeStore();
const layoutStore = useLayoutStore();
const activeTab = ref<"theme" | "layout">("theme");
const themeName = ref(themeStore.active.name);
const fileInput = ref<HTMLInputElement | null>(null);

const colorKeys = Object.keys(themeStore.active.colors) as (keyof Theme["colors"])[];

function selectTheme(theme: Theme) {
  themeStore.setTheme(theme);
  themeName.value = theme.name;
}

function handleColorInput(key: keyof Theme["colors"], event: Event) {
  const value = (event.target as HTMLInputElement).value;
  themeStore.updateActiveColor(key, value);
}

function handleSave() {
  themeStore.saveCustomTheme({ ...themeStore.active, id: `custom-${Date.now()}`, name: themeName.value });
}

function handleExport() {
  const json = themeStore.exportActiveTheme();
  const blob = new Blob([json], { type: "application/json" });
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = `${themeName.value.replace(/\s+/g, "_")}.theme.json`;
  a.click();
  URL.revokeObjectURL(url);
}

function handleImportClick() {
  fileInput.value?.click();
}

function handleFileImport(event: Event) {
  const file = (event.target as HTMLInputElement).files?.[0];
  if (!file) return;
  const reader = new FileReader();
  reader.onload = (ev) => themeStore.importTheme(ev.target?.result as string);
  reader.readAsText(file);
  (event.target as HTMLInputElement).value = "";
}
</script>

<template>
  <div class="te-page">
    <div class="te-tabs">
      <button :class="{ active: activeTab === 'theme' }" @click="activeTab = 'theme'">Thème</button>
      <button :class="{ active: activeTab === 'layout' }" @click="activeTab = 'layout'">Disposition</button>
    </div>

    <section v-if="activeTab === 'theme'" class="te-panel">
      <input v-model="themeName" class="te-name-input" placeholder="Nom du thème..." />

      <div class="te-swatches">
        <button
          v-for="theme in builtinThemes"
          :key="theme.id"
          class="te-swatch"
          :style="{ background: theme.colors.bgBase, borderColor: theme.colors.accentPrimary }"
          :title="theme.name"
          @click="selectTheme(theme)"
        />
      </div>

      <div class="te-colors">
        <label v-for="key in colorKeys" :key="key" class="te-color-row">
          <span>{{ key }}</span>
          <input type="color" :value="themeStore.active.colors[key]" @input="handleColorInput(key, $event)" />
        </label>
      </div>

      <div class="te-actions">
        <button @click="handleSave">Sauvegarder</button>
        <button @click="handleExport">Exporter</button>
        <button @click="handleImportClick">Importer</button>
        <input ref="fileInput" type="file" accept=".json" style="display:none" @change="handleFileImport" />
      </div>
    </section>

    <section v-else class="te-panel">
      <div class="te-layouts">
        <button
          v-for="layout in layoutRegistry"
          :key="layout.id"
          class="te-layout-option"
          :class="{ active: layoutStore.current === layout.id }"
          @click="layoutStore.setLayout(layout.id)"
        >
          <strong>{{ layout.name }}</strong>
          <p>{{ layout.description }}</p>
        </button>
      </div>
    </section>
  </div>
</template>

<style scoped>
.te-page { padding: 24px; color: var(--nx-text-primary); }
.te-tabs { display: flex; gap: 8px; margin-bottom: 16px; }
.te-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.te-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.te-name-input { width: 100%; padding: 8px 12px; margin-bottom: 16px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.te-swatches { display: flex; flex-wrap: wrap; gap: 8px; margin-bottom: 20px; }
.te-swatch { width: 32px; height: 32px; border-radius: 8px; border: 2px solid; cursor: pointer; }
.te-colors { display: grid; gap: 8px; margin-bottom: 20px; }
.te-color-row { display: flex; justify-content: space-between; align-items: center; font-size: 13px; color: var(--nx-text-secondary); }
.te-actions { display: flex; gap: 8px; }
.te-actions button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; }
.te-layouts { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; }
.te-layout-option { text-align: left; padding: 14px; border-radius: 10px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; }
.te-layout-option.active { border-color: var(--nx-accent-primary); }
.te-layout-option p { margin: 6px 0 0; font-size: 12px; color: var(--nx-text-secondary); }
</style>
```

- [ ] **Step 6: Commit**

```bash
git add src/pages/ThemeEditorPage.vue src/stores/themeStore.ts src/stores/themeStore.editor.spec.ts
git commit -m "feat: real-time theme and layout editor page"
```

---

## Task 8: Rust backend — system snapshot (CPU/RAM/processes) + Dashboard page

**Files:**
- Create: `src-tauri/src/system.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/Cargo.toml`
- Create: `src/pages/DashboardPage.vue`
- Test: `src-tauri/src/system.rs` (inline `#[cfg(test)]`)

- [ ] **Step 1: Add the `sysinfo` dependency**

Modify `src-tauri/Cargo.toml` — add under `[dependencies]`:

```toml
sysinfo = "0.32"
```

- [ ] **Step 2: Write the failing Rust test**

```rust
// src-tauri/src/system.rs (top of file, test module)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_percent_rounds_to_one_decimal() {
        assert_eq!(format_percent(62.456), "62.5%");
        assert_eq!(format_percent(0.0), "0.0%");
        assert_eq!(format_percent(100.0), "100.0%");
    }

    #[test]
    fn snapshot_has_at_least_one_cpu_and_nonzero_memory() {
        let snap = build_snapshot();
        assert!(!snap.cpus.is_empty());
        assert!(snap.memory_total_bytes > 0);
    }
}
```

- [ ] **Step 3: Run test to verify it fails**

Run: `cd src-tauri && cargo test system:: 2>&1 | tail -30`
Expected: FAIL — `cannot find function 'format_percent'` / `build_snapshot` (module doesn't exist yet)

- [ ] **Step 4: Implement `system.rs`**

```rust
// src-tauri/src/system.rs
use serde::Serialize;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct CpuInfo {
    pub name: String,
    pub usage_percent: f32,
}

#[derive(Serialize, Clone)]
pub struct SystemSnapshot {
    pub cpus: Vec<CpuInfo>,
    pub memory_used_bytes: u64,
    pub memory_total_bytes: u64,
    pub process_count: usize,
}

pub fn format_percent(value: f32) -> String {
    format!("{:.1}%", value)
}

pub fn build_snapshot() -> SystemSnapshot {
    let mut sys = System::new_all();
    sys.refresh_all();

    let cpus = sys
        .cpus()
        .iter()
        .map(|cpu| CpuInfo {
            name: cpu.brand().to_string(),
            usage_percent: cpu.cpu_usage(),
        })
        .collect();

    SystemSnapshot {
        cpus,
        memory_used_bytes: sys.used_memory(),
        memory_total_bytes: sys.total_memory(),
        process_count: sys.processes().len(),
    }
}

#[tauri::command]
pub fn get_system_snapshot() -> SystemSnapshot {
    build_snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_percent_rounds_to_one_decimal() {
        assert_eq!(format_percent(62.456), "62.5%");
        assert_eq!(format_percent(0.0), "0.0%");
        assert_eq!(format_percent(100.0), "100.0%");
    }

    #[test]
    fn snapshot_has_at_least_one_cpu_and_nonzero_memory() {
        let snap = build_snapshot();
        assert!(!snap.cpus.is_empty());
        assert!(snap.memory_total_bytes > 0);
    }
}
```

- [ ] **Step 5: Register the module and command**

Modify `src-tauri/src/lib.rs`:

```rust
mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![system::get_system_snapshot])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

(If `lib.rs` already registers other commands or plugins, add `system::get_system_snapshot` to the existing `generate_handler!` list rather than replacing it.)

- [ ] **Step 6: Run test to verify it passes**

Run: `cd src-tauri && cargo test system:: 2>&1 | tail -30`
Expected: PASS (2 tests)

- [ ] **Step 7: Build the Dashboard page**

```vue
<!-- src/pages/DashboardPage.vue -->
<script setup lang="ts">
import { ref, onMounted, onUnmounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface CpuInfo { name: string; usage_percent: number }
interface SystemSnapshot {
  cpus: CpuInfo[];
  memory_used_bytes: number;
  memory_total_bytes: number;
  process_count: number;
}

const snapshot = ref<SystemSnapshot | null>(null);
let intervalId: number | undefined;

async function refresh() {
  snapshot.value = await invoke<SystemSnapshot>("get_system_snapshot");
}

onMounted(() => {
  refresh();
  intervalId = window.setInterval(refresh, 2000);
});

onUnmounted(() => {
  if (intervalId) window.clearInterval(intervalId);
});

function bytesToGb(bytes: number): string {
  return (bytes / 1024 / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="dash-page" v-if="snapshot">
    <h1>Vue d'ensemble</h1>
    <div class="dash-grid">
      <div class="dash-card" v-for="(cpu, i) in snapshot.cpus" :key="i">
        <div class="dash-label">{{ cpu.name || `CPU ${i}` }}</div>
        <div class="dash-value">{{ cpu.usage_percent.toFixed(1) }}%</div>
      </div>
      <div class="dash-card">
        <div class="dash-label">Mémoire</div>
        <div class="dash-value">{{ bytesToGb(snapshot.memory_used_bytes) }} / {{ bytesToGb(snapshot.memory_total_bytes) }} GB</div>
      </div>
      <div class="dash-card">
        <div class="dash-label">Processus</div>
        <div class="dash-value">{{ snapshot.process_count }}</div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.dash-page { padding: 24px; color: var(--nx-text-primary); }
.dash-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(180px, 1fr)); gap: 14px; margin-top: 16px; }
.dash-card { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 10px; padding: 14px; }
.dash-label { font-size: 12px; color: var(--nx-text-secondary); }
.dash-value { font-size: 22px; font-weight: 700; margin-top: 6px; }
</style>
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/system.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src/pages/DashboardPage.vue
git commit -m "feat: system snapshot backend command and Dashboard page"
```

---

## Task 9: Rust backend — sensors (temperature) + battery, wired into Dashboard

**Files:**
- Create: `src-tauri/src/sensors.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/pages/DashboardPage.vue`

- [ ] **Step 1: Write the failing Rust test**

```rust
// src-tauri/src/sensors.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_battery_capacity_from_sysfs_content() {
        assert_eq!(parse_capacity("87\n"), Some(87));
        assert_eq!(parse_capacity(""), None);
        assert_eq!(parse_capacity("not-a-number"), None);
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test sensors:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `sensors.rs`**

```rust
// src-tauri/src/sensors.rs
use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct SensorSnapshot {
    pub battery_percent: Option<u8>,
    pub battery_charging: Option<bool>,
    pub temperatures: Vec<TemperatureReading>,
}

#[derive(Serialize, Clone)]
pub struct TemperatureReading {
    pub label: String,
    pub celsius: f32,
}

pub fn parse_capacity(content: &str) -> Option<u8> {
    content.trim().parse::<u8>().ok()
}

fn read_battery() -> (Option<u8>, Option<bool>) {
    let base = Path::new("/sys/class/power_supply/BAT0");
    let capacity = fs::read_to_string(base.join("capacity"))
        .ok()
        .and_then(|s| parse_capacity(&s));
    let status = fs::read_to_string(base.join("status"))
        .ok()
        .map(|s| s.trim().eq_ignore_ascii_case("charging"));
    (capacity, status)
}

fn read_temperatures() -> Vec<TemperatureReading> {
    use sysinfo::Components;
    let components = Components::new_with_refreshed_list();
    components
        .iter()
        .filter_map(|c| {
            c.temperature().map(|t| TemperatureReading {
                label: c.label().to_string(),
                celsius: t,
            })
        })
        .collect()
}

#[tauri::command]
pub fn get_sensor_snapshot() -> SensorSnapshot {
    let (battery_percent, battery_charging) = read_battery();
    SensorSnapshot {
        battery_percent,
        battery_charging,
        temperatures: read_temperatures(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_battery_capacity_from_sysfs_content() {
        assert_eq!(parse_capacity("87\n"), Some(87));
        assert_eq!(parse_capacity(""), None);
        assert_eq!(parse_capacity("not-a-number"), None);
    }
}
```

- [ ] **Step 4: Register the command**

Modify `src-tauri/src/lib.rs`:

```rust
mod sensors;
mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            system::get_system_snapshot,
            sensors::get_sensor_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test sensors:: 2>&1 | tail -30`
Expected: PASS (1 test)

- [ ] **Step 6: Wire into Dashboard**

Modify `src/pages/DashboardPage.vue` — add alongside the existing `SystemSnapshot` fetch:

```ts
interface SensorSnapshot {
  battery_percent: number | null;
  battery_charging: boolean | null;
  temperatures: { label: string; celsius: number }[];
}

const sensors = ref<SensorSnapshot | null>(null);

async function refreshSensors() {
  sensors.value = await invoke<SensorSnapshot>("get_sensor_snapshot");
}
```

In `onMounted`, call `refreshSensors()` alongside `refresh()` and include it in the same `setInterval`. Add to the template, inside `.dash-grid`:

```html
      <div class="dash-card" v-if="sensors?.battery_percent !== null && sensors?.battery_percent !== undefined">
        <div class="dash-label">Batterie</div>
        <div class="dash-value">{{ sensors!.battery_percent }}%{{ sensors!.battery_charging ? " ⚡" : "" }}</div>
      </div>
      <div class="dash-card" v-for="t in sensors?.temperatures ?? []" :key="t.label">
        <div class="dash-label">{{ t.label }}</div>
        <div class="dash-value">{{ t.celsius.toFixed(0) }}°C</div>
      </div>
```

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/sensors.rs src-tauri/src/lib.rs src/pages/DashboardPage.vue
git commit -m "feat: battery and temperature sensors on Dashboard"
```

---

## Task 10: Rust backend — hardware details (lspci) + Hardware page

**Files:**
- Create: `src-tauri/src/hardware.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/HardwarePage.vue`

- [ ] **Step 1: Write the failing Rust test**

```rust
// src-tauri/src/hardware.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lspci_line_into_device() {
        let line = "00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 620";
        let device = parse_lspci_line(line).expect("should parse");
        assert_eq!(device.slot, "00:02.0");
        assert_eq!(device.class, "VGA compatible controller");
        assert_eq!(device.description, "Intel Corporation UHD Graphics 620");
    }

    #[test]
    fn skips_malformed_lines() {
        assert!(parse_lspci_line("not a valid line").is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test hardware:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `hardware.rs`**

```rust
// src-tauri/src/hardware.rs
use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct PciDevice {
    pub slot: String,
    pub class: String,
    pub description: String,
}

pub fn parse_lspci_line(line: &str) -> Option<PciDevice> {
    let (slot, rest) = line.split_once(' ')?;
    let (class, description) = rest.split_once(": ")?;
    Some(PciDevice {
        slot: slot.to_string(),
        class: class.to_string(),
        description: description.to_string(),
    })
}

fn run_lspci() -> Vec<PciDevice> {
    let output = match Command::new("lspci").output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().filter_map(parse_lspci_line).collect()
}

#[tauri::command]
pub fn get_pci_devices() -> Vec<PciDevice> {
    run_lspci()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lspci_line_into_device() {
        let line = "00:02.0 VGA compatible controller: Intel Corporation UHD Graphics 620";
        let device = parse_lspci_line(line).expect("should parse");
        assert_eq!(device.slot, "00:02.0");
        assert_eq!(device.class, "VGA compatible controller");
        assert_eq!(device.description, "Intel Corporation UHD Graphics 620");
    }

    #[test]
    fn skips_malformed_lines() {
        assert!(parse_lspci_line("not a valid line").is_none());
    }
}
```

Note: `parse_lspci_line` splits on the first space to separate the PCI slot address, then splits the remainder on `": "` to separate device class from description — matching the real `lspci` default output format (no `-v`/`-mm` flags needed).

- [ ] **Step 4: Register the command**

Modify `src-tauri/src/lib.rs`:

```rust
mod hardware;
mod sensors;
mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            system::get_system_snapshot,
            sensors::get_sensor_snapshot,
            hardware::get_pci_devices
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test hardware:: 2>&1 | tail -30`
Expected: PASS (2 tests)

- [ ] **Step 6: Build the Hardware page**

```vue
<!-- src/pages/HardwarePage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface PciDevice { slot: string; class: string; description: string }

const devices = ref<PciDevice[]>([]);

onMounted(async () => {
  devices.value = await invoke<PciDevice[]>("get_pci_devices");
});
</script>

<template>
  <div class="hw-page">
    <h1>Composants matériels</h1>
    <table class="hw-table">
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
.hw-table { width: 100%; border-collapse: collapse; margin-top: 16px; font-size: 13px; }
.hw-table th { text-align: left; color: var(--nx-text-secondary); border-bottom: 1px solid var(--nx-border); padding: 8px; }
.hw-table td { padding: 8px; border-bottom: 1px solid var(--nx-border); }
</style>
```

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/hardware.rs src-tauri/src/lib.rs src/pages/HardwarePage.vue
git commit -m "feat: PCI hardware listing backend command and Hardware page"
```

---

## Task 11: Rust backend — kernel modules & GPU driver detection + Drivers page

**Files:**
- Create: `src-tauri/src/drivers.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/DriversPage.vue`

- [ ] **Step 1: Write the failing Rust test**

```rust
// src-tauri/src/drivers.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_nvidia_driver_from_module_list() {
        let modules = vec!["nvidia".to_string(), "snd_hda_intel".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nvidia (propriétaire)");
    }

    #[test]
    fn detects_nouveau_driver_from_module_list() {
        let modules = vec!["nouveau".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nouveau (open-source)");
    }

    #[test]
    fn detects_amdgpu_driver_from_module_list() {
        let modules = vec!["amdgpu".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "amdgpu (open-source)");
    }

    #[test]
    fn falls_back_to_unknown_when_no_gpu_module_present() {
        let modules = vec!["ext4".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "inconnu");
    }

    #[test]
    fn parses_lsmod_line_into_module_name() {
        let line = "nvidia               56655872  42";
        assert_eq!(parse_lsmod_line(line), Some("nvidia".to_string()));
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test drivers:: 2>&1 | tail -40`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `drivers.rs`**

```rust
// src-tauri/src/drivers.rs
use serde::Serialize;
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct DriverSnapshot {
    pub loaded_modules: Vec<String>,
    pub gpu_driver: String,
}

pub fn parse_lsmod_line(line: &str) -> Option<String> {
    let name = line.split_whitespace().next()?;
    if name.eq_ignore_ascii_case("module") {
        return None; // header row
    }
    Some(name.to_string())
}

pub fn detect_gpu_driver(modules: &[String]) -> String {
    if modules.iter().any(|m| m == "nvidia") {
        "nvidia (propriétaire)".to_string()
    } else if modules.iter().any(|m| m == "nouveau") {
        "nouveau (open-source)".to_string()
    } else if modules.iter().any(|m| m == "amdgpu") {
        "amdgpu (open-source)".to_string()
    } else if modules.iter().any(|m| m == "i915") {
        "i915 (Intel, open-source)".to_string()
    } else {
        "inconnu".to_string()
    }
}

fn run_lsmod() -> Vec<String> {
    let output = match Command::new("lsmod").output() {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().filter_map(parse_lsmod_line).collect()
}

#[tauri::command]
pub fn get_driver_snapshot() -> DriverSnapshot {
    let loaded_modules = run_lsmod();
    let gpu_driver = detect_gpu_driver(&loaded_modules);
    DriverSnapshot { loaded_modules, gpu_driver }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_nvidia_driver_from_module_list() {
        let modules = vec!["nvidia".to_string(), "snd_hda_intel".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nvidia (propriétaire)");
    }

    #[test]
    fn detects_nouveau_driver_from_module_list() {
        let modules = vec!["nouveau".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "nouveau (open-source)");
    }

    #[test]
    fn detects_amdgpu_driver_from_module_list() {
        let modules = vec!["amdgpu".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "amdgpu (open-source)");
    }

    #[test]
    fn falls_back_to_unknown_when_no_gpu_module_present() {
        let modules = vec!["ext4".to_string()];
        assert_eq!(detect_gpu_driver(&modules), "inconnu");
    }

    #[test]
    fn parses_lsmod_line_into_module_name() {
        let line = "nvidia               56655872  42";
        assert_eq!(parse_lsmod_line(line), Some("nvidia".to_string()));
    }
}
```

- [ ] **Step 4: Register the command**

Modify `src-tauri/src/lib.rs`:

```rust
mod drivers;
mod hardware;
mod sensors;
mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            system::get_system_snapshot,
            sensors::get_sensor_snapshot,
            hardware::get_pci_devices,
            drivers::get_driver_snapshot
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 5: Run test to verify it passes**

Run: `cd src-tauri && cargo test drivers:: 2>&1 | tail -40`
Expected: PASS (5 tests)

- [ ] **Step 6: Build the Drivers page**

```vue
<!-- src/pages/DriversPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface DriverSnapshot { loaded_modules: string[]; gpu_driver: string }

const snapshot = ref<DriverSnapshot | null>(null);

onMounted(async () => {
  snapshot.value = await invoke<DriverSnapshot>("get_driver_snapshot");
});
</script>

<template>
  <div class="drv-page" v-if="snapshot">
    <h1>Pilotes & modules noyau</h1>
    <div class="drv-gpu">Pilote GPU actif : <strong>{{ snapshot.gpu_driver }}</strong></div>
    <h2>Modules chargés ({{ snapshot.loaded_modules.length }})</h2>
    <ul class="drv-list">
      <li v-for="mod in snapshot.loaded_modules" :key="mod">{{ mod }}</li>
    </ul>
  </div>
</template>

<style scoped>
.drv-page { padding: 24px; color: var(--nx-text-primary); }
.drv-gpu { margin: 12px 0; padding: 10px 14px; border-radius: 8px; background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); }
.drv-list { columns: 3; column-gap: 24px; font-size: 13px; margin-top: 8px; }
.drv-list li { padding: 3px 0; }
</style>
```

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/drivers.rs src-tauri/src/lib.rs src/pages/DriversPage.vue
git commit -m "feat: kernel module listing and GPU driver detection with Drivers page"
```

---

## Task 12: Rust backend — journalctl logs + Logs page

**Files:**
- Create: `src-tauri/src/logs.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/LogsPage.vue`

- [ ] **Step 1: Write the failing Rust test**

```rust
// src-tauri/src/logs.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_journalctl_json_line_into_entry() {
        let line = r#"{"__REALTIME_TIMESTAMP":"1785440000000000","PRIORITY":"3","MESSAGE":"disk write error","SYSLOG_IDENTIFIER":"kernel"}"#;
        let entry = parse_journal_line(line).expect("should parse");
        assert_eq!(entry.priority, 3);
        assert_eq!(entry.message, "disk write error");
        assert_eq!(entry.unit, "kernel");
    }

    #[test]
    fn skips_unparseable_lines() {
        assert!(parse_journal_line("not json").is_none());
    }
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd src-tauri && cargo test logs:: 2>&1 | tail -30`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `logs.rs`**

```rust
// src-tauri/src/logs.rs
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Serialize, Clone)]
pub struct LogEntry {
    pub priority: u8,
    pub message: String,
    pub unit: String,
}

#[derive(Deserialize)]
struct RawJournalLine {
    #[serde(rename = "PRIORITY")]
    priority: String,
    #[serde(rename = "MESSAGE")]
    message: String,
    #[serde(rename = "SYSLOG_IDENTIFIER", default)]
    syslog_identifier: String,
}

pub fn parse_journal_line(line: &str) -> Option<LogEntry> {
    let raw: RawJournalLine = serde_json::from_str(line).ok()?;
    let priority: u8 = raw.priority.parse().ok()?;
    Some(LogEntry {
        priority,
        message: raw.message,
        unit: raw.syslog_identifier,
    })
}

#[tauri::command]
pub fn get_recent_logs(limit: u32) -> Vec<LogEntry> {
    let output = match Command::new("journalctl")
        .args(["-o", "json", "-n", &limit.to_string(), "--no-pager"])
        .output()
    {
        Ok(o) => o,
        Err(_) => return Vec::new(),
    };
    let stdout = String::from_utf8_lossy(&output.stdout);
    stdout.lines().filter_map(parse_journal_line).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_journalctl_json_line_into_entry() {
        let line = r#"{"__REALTIME_TIMESTAMP":"1785440000000000","PRIORITY":"3","MESSAGE":"disk write error","SYSLOG_IDENTIFIER":"kernel"}"#;
        let entry = parse_journal_line(line).expect("should parse");
        assert_eq!(entry.priority, 3);
        assert_eq!(entry.message, "disk write error");
        assert_eq!(entry.unit, "kernel");
    }

    #[test]
    fn skips_unparseable_lines() {
        assert!(parse_journal_line("not json").is_none());
    }
}
```

- [ ] **Step 4: Add `serde_json` dependency if not already present**

Check `src-tauri/Cargo.toml` — Tauri projects already depend on `serde_json` transitively via `tauri`, but it must be a direct dependency to use `serde_json::from_str` in application code. Add if missing:

```toml
serde_json = "1"
```

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs`:

```rust
mod drivers;
mod hardware;
mod logs;
mod sensors;
mod system;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            system::get_system_snapshot,
            sensors::get_sensor_snapshot,
            hardware::get_pci_devices,
            drivers::get_driver_snapshot,
            logs::get_recent_logs
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 6: Run test to verify it passes**

Run: `cd src-tauri && cargo test logs:: 2>&1 | tail -30`
Expected: PASS (2 tests)

- [ ] **Step 7: Build the Logs page**

```vue
<!-- src/pages/LogsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface LogEntry { priority: number; message: string; unit: string }

const logs = ref<LogEntry[]>([]);

function priorityClass(priority: number): string {
  if (priority <= 3) return "log-error";
  if (priority <= 4) return "log-warning";
  return "log-info";
}

onMounted(async () => {
  logs.value = await invoke<LogEntry[]>("get_recent_logs", { limit: 200 });
});
</script>

<template>
  <div class="logs-page">
    <h1>Journaux système</h1>
    <div class="logs-list">
      <div v-for="(log, i) in logs" :key="i" class="log-entry" :class="priorityClass(log.priority)">
        <span class="log-unit">{{ log.unit }}</span>
        <span class="log-message">{{ log.message }}</span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.logs-page { padding: 24px; color: var(--nx-text-primary); }
.logs-list { margin-top: 16px; font-family: monospace; font-size: 12px; display: grid; gap: 2px; max-height: 70vh; overflow: auto; }
.log-entry { display: flex; gap: 10px; padding: 4px 8px; border-radius: 4px; }
.log-entry.log-error { background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); }
.log-entry.log-warning { background: color-mix(in srgb, var(--nx-accent-warning) 15%, transparent); }
.log-unit { color: var(--nx-text-secondary); flex-shrink: 0; min-width: 120px; }
</style>
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/logs.rs src-tauri/src/lib.rs src-tauri/Cargo.toml src/pages/LogsPage.vue
git commit -m "feat: journalctl-backed system logs command and Logs page"
```

---

## Task 13: Wire pages into navigation and verify end-to-end

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Wire pages behind simple client-side routing**

Modify `src/App.vue` to mount `LayoutShell` with a minimal nav (real router deferred to Phase 2 — a `ref<string>` page switch is enough to prove the shell + pillar integration end-to-end):

```vue
<script setup lang="ts">
import { onMounted, ref } from "vue";
import { useThemeStore } from "@/stores/themeStore";
import LayoutShell from "@/layouts/LayoutShell.vue";
import DashboardPage from "@/pages/DashboardPage.vue";
import HardwarePage from "@/pages/HardwarePage.vue";
import DriversPage from "@/pages/DriversPage.vue";
import LogsPage from "@/pages/LogsPage.vue";
import ThemeEditorPage from "@/pages/ThemeEditorPage.vue";

const themeStore = useThemeStore();
onMounted(() => themeStore.setTheme(themeStore.active));

type PageId = "dashboard" | "hardware" | "drivers" | "logs" | "theme-editor";
const currentPage = ref<PageId>("dashboard");
const pages: Record<PageId, unknown> = {
  dashboard: DashboardPage,
  hardware: HardwarePage,
  drivers: DriversPage,
  logs: LogsPage,
  "theme-editor": ThemeEditorPage,
};
</script>

<template>
  <LayoutShell>
    <template #nav>
      <nav class="app-nav">
        <button @click="currentPage = 'dashboard'">Dashboard</button>
        <button @click="currentPage = 'hardware'">Matériel</button>
        <button @click="currentPage = 'drivers'">Pilotes</button>
        <button @click="currentPage = 'logs'">Journaux</button>
        <button @click="currentPage = 'theme-editor'">Apparence</button>
      </nav>
    </template>
    <component :is="pages[currentPage]" />
  </LayoutShell>
</template>

<style scoped>
.app-nav { display: flex; flex-direction: column; gap: 4px; padding: 12px 8px; }
.app-nav button { text-align: left; padding: 8px 10px; border: none; background: transparent; color: var(--nx-text-secondary); border-radius: 6px; cursor: pointer; }
.app-nav button:hover { background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
</style>
```

- [ ] **Step 2: Run the full test suite**

Run: `npm run test`
Expected: PASS — all frontend suites (theme, layout, editor) green.

Run: `cd src-tauri && cargo test`
Expected: PASS — all Rust suites (system, sensors, hardware, drivers, logs) green.

- [ ] **Step 3: Manual verification in WSL2**

Run: `npm run tauri dev`
Expected: window opens with sidebar-classic layout, Dashboard shows live CPU/RAM/battery/temperature, switching to "Apparence" lets you pick any of the 12 themes and any of the 8 layouts with instant visual change, Matériel/Pilotes/Journaux pages show real data from the WSL2 host.

- [ ] **Step 4: Final commit**

```bash
git add src/App.vue
git commit -m "feat: wire Phase 1 pages into navigable app shell"
```
