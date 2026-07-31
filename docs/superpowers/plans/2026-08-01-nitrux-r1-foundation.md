# NiTruX Redesign — Phase R1 (Foundation) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the foundation for the NiTruX redesign — a categorized navigation data model + component, a third independent visual-style axis (12 style treatments, alongside the existing 12 color palettes and 8 layouts), and a shared UI component library (`NxCard`/`NxButton`/`NxInput`/`NxSelect`/`NxStatTile`/`NxBadge`/`NxSectionHeader`) — so subsequent phases can restructure pages onto a consistent, testable system instead of each page hand-rolling its own CSS.

**Architecture:** The style axis mirrors the existing `layoutStore`/`layoutRegistry` pattern exactly (a `StyleId` union, a `styleRegistry` array, a Pinia store persisting the active id to `localStorage` and writing a `data-nx-style` attribute to `<html>`). Each style defines a fixed set of CSS custom properties (`--nx-style-radius`, `--nx-style-border-width`, `--nx-style-border-color`, `--nx-style-shadow`, `--nx-style-blur`, `--nx-style-bg`, `--nx-style-font-family`) scoped under `[data-nx-style="<id>"]` selectors in one global stylesheet; every shared UI component reads only those 7 custom properties (plus the existing `--nx-*` color/theme variables), so no component needs to know which of the 12 styles is active — swapping styles is purely a CSS custom-property swap, live, no reload, exactly like the existing palette/layout swapping.

**Tech Stack:** Vue 3 + `<script setup>` + TypeScript, Pinia, Vitest + `@vue/test-utils`, existing `@` → `/src` alias.

**No page content moves in this plan** — `App.vue`'s existing flat nav/page-switching stays as-is functionally; this plan only adds the new pieces (categories data, `AppNav.vue`, style system, `Nx*` components) as new, independently-tested files, plus extends `ThemeEditorPage.vue` to expose the new style picker. Wiring `AppNav.vue` into `App.vue` to *replace* the flat button list, and moving page content into the new categories, is Phase R2's job.

---

## File Structure

```
src/
├── types/
│   └── style.ts                    # NEW: StyleId union + StyleDefinition interface
├── styles/
│   ├── registry.ts                 # NEW: styleRegistry (12 entries), mirrors layouts/registry.ts
│   └── style-tokens.css            # NEW: 12 [data-nx-style="..."] blocks of CSS custom properties
├── stores/
│   └── styleStore.ts                # NEW: mirrors layoutStore.ts (persisted active style id)
├── components/
│   ├── nav/
│   │   └── AppNav.vue               # NEW: categorized nav, renders from categories.ts
│   └── ui/
│       ├── NxCard.vue                # NEW
│       ├── NxButton.vue              # NEW
│       ├── NxInput.vue               # NEW
│       ├── NxSelect.vue              # NEW
│       ├── NxStatTile.vue            # NEW
│       ├── NxBadge.vue               # NEW
│       └── NxSectionHeader.vue       # NEW
├── navigation/
│   └── categories.ts                # NEW: 9-category nav data model (id, label, icon, pages)
├── pages/
│   └── ThemeEditorPage.vue           # MODIFIED: add 3rd tab "Style" with live picker
└── main.ts                           # MODIFIED: import style-tokens.css globally
```

---

## Task 1: Style types + registry

**Files:**
- Create: `src/types/style.ts`
- Create: `src/styles/registry.ts`
- Test: `src/styles/registry.spec.ts`

- [ ] **Step 1: Write the failing test**

```typescript
// src/styles/registry.spec.ts
import { describe, it, expect } from "vitest";
import { styleRegistry } from "./registry";

describe("styleRegistry", () => {
  it("lists exactly 12 styles", () => {
    expect(styleRegistry).toHaveLength(12);
  });

  it("has unique ids", () => {
    const ids = styleRegistry.map((s) => s.id);
    expect(new Set(ids).size).toBe(ids.length);
  });

  it("every style has a non-empty name and description", () => {
    for (const style of styleRegistry) {
      expect(style.name.length).toBeGreaterThan(0);
      expect(style.description.length).toBeGreaterThan(0);
    }
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/styles/registry.spec.ts"`
Expected: FAIL — `./registry` module not found.

- [ ] **Step 3: Write the type definitions**

```typescript
// src/types/style.ts
export type StyleId =
  | "glass-glow" | "flat-modern" | "neon-terminal" | "neumorphism"
  | "brutalism" | "paper" | "aero-glass" | "cyber-grid"
  | "line-art" | "gradient-mesh" | "amber-crt" | "mono-contrast";

export interface StyleDefinition {
  id: StyleId;
  name: string;
  description: string;
}
```

- [ ] **Step 4: Write the registry**

```typescript
// src/styles/registry.ts
import type { StyleDefinition } from "@/types/style";

export const styleRegistry: StyleDefinition[] = [
  { id: "glass-glow", name: "Verre & lueur", description: "Cartes translucides floutées, ombres lumineuses colorées, dégradés." },
  { id: "flat-modern", name: "Flat moderne", description: "Cartes pleines, bords nets, sans flou ni lueur." },
  { id: "neon-terminal", name: "Neon Terminal", description: "Typographie monospace, lueur verte façon écran CRT." },
  { id: "neumorphism", name: "Néumorphisme", description: "Ombres doubles douces en relief, base monochrome." },
  { id: "brutalism", name: "Brutalisme", description: "Bordures noires épaisses, ombres décalées franches, sans arrondi." },
  { id: "paper", name: "Papier minimal", description: "Cartes très claires, ombre subtile, esprit éditorial." },
  { id: "aero-glass", name: "Aero Glass", description: "Reflet glossy en dégradé, clin d'œil à l'Aero de Windows." },
  { id: "cyber-grid", name: "Cyber Grid", description: "Fond quadrillé discret, lueur cyan, esprit interface futuriste." },
  { id: "line-art", name: "Ligne claire", description: "Contours fins uniquement, sans remplissage, esprit technique." },
  { id: "gradient-mesh", name: "Gradient Mesh", description: "Dégradés vifs multicolores derrière des cartes en verre." },
  { id: "amber-crt", name: "CRT Ambre", description: "Typographie monospace, lueur ambre façon terminal DOS." },
  { id: "mono-contrast", name: "Monochrome", description: "Noir et blanc pur, contraste maximal, bordures nettes." },
];
```

- [ ] **Step 5: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/styles/registry.spec.ts"`
Expected: PASS (3 tests)

- [ ] **Step 6: Commit**

```bash
git add src/types/style.ts src/styles/registry.ts src/styles/registry.spec.ts
git commit -m "feat: add StyleId type and 12-entry style registry"
```

---

## Task 2: Style store

**Files:**
- Create: `src/stores/styleStore.ts`
- Test: `src/stores/styleStore.spec.ts`

- [ ] **Step 1: Write the failing test**

```typescript
// src/stores/styleStore.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useStyleStore } from "./styleStore";
import { styleRegistry } from "@/styles/registry";

describe("styleStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
    document.documentElement.removeAttribute("data-nx-style");
  });

  it("defaults to glass-glow when nothing persisted", () => {
    const store = useStyleStore();
    expect(store.current).toBe("glass-glow");
  });

  it("setStyle updates state, persists to localStorage, and sets the data attribute", () => {
    const store = useStyleStore();
    store.setStyle("brutalism");
    expect(store.current).toBe("brutalism");
    expect(localStorage.getItem("nitrux-style")).toBe("brutalism");
    expect(document.documentElement.dataset.nxStyle).toBe("brutalism");
  });

  it("falls back to the default when localStorage holds an unknown style id", () => {
    localStorage.setItem("nitrux-style", "not-a-real-style");
    const store = useStyleStore();
    expect(store.current).toBe("glass-glow");
  });

  it.each(styleRegistry.map((s) => s.id))("accepts %s as a valid style id", (id) => {
    const store = useStyleStore();
    store.setStyle(id);
    expect(store.current).toBe(id);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/stores/styleStore.spec.ts"`
Expected: FAIL — `./styleStore` module not found.

- [ ] **Step 3: Write the store**

```typescript
// src/stores/styleStore.ts
import { defineStore } from "pinia";
import type { StyleId } from "@/types/style";
import { styleRegistry } from "@/styles/registry";

const STORAGE_KEY = "nitrux-style";
const DEFAULT_STYLE: StyleId = "glass-glow";

function isValidStyleId(value: string | null): value is StyleId {
  return value !== null && styleRegistry.some((s) => s.id === value);
}

function readPersistedStyle(): StyleId {
  const stored = localStorage.getItem(STORAGE_KEY);
  return isValidStyleId(stored) ? stored : DEFAULT_STYLE;
}

function applyToDom(id: StyleId) {
  document.documentElement.dataset.nxStyle = id;
}

export const useStyleStore = defineStore("style", {
  state: () => ({
    current: readPersistedStyle(),
  }),
  actions: {
    setStyle(id: StyleId) {
      this.current = id;
      localStorage.setItem(STORAGE_KEY, id);
      applyToDom(id);
    },
  },
});
```

Note: unlike `themeStore` (which never persists — a pre-existing inconsistency in the codebase, out of scope to fix here) but exactly like `layoutStore`, `styleStore` persists via `localStorage`. This plan does not apply the DOM attribute on store creation (no `applyToDom(readPersistedStyle())` call at module load) — that responsibility belongs to `App.vue`'s mount hook in Phase R2, mirroring how `themeStore.setTheme(themeStore.active)` is currently called from `App.vue`'s `onMounted`, not from the store itself. For this plan, tests set the attribute explicitly via `setStyle()`, which is sufficient to prove the store's own behavior in isolation.

- [ ] **Step 4: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/stores/styleStore.spec.ts"`
Expected: PASS (15 tests — 3 named + 12 from `it.each`)

- [ ] **Step 5: Commit**

```bash
git add src/stores/styleStore.ts src/stores/styleStore.spec.ts
git commit -m "feat: add styleStore (persisted active visual style)"
```

---

## Task 3: Style tokens CSS

**Files:**
- Create: `src/styles/style-tokens.css`
- Modify: `src/main.ts`

This task has no automated test (pure CSS) — verification is a manual read-through plus confirming the file parses as valid CSS via a build.

- [ ] **Step 1: Read the current `src/main.ts`**

```bash
cat src/main.ts
```

Confirm its current import list (should import `./styles.css` or similar global stylesheet, plus mount the Vue app) before editing — add the new import alongside the existing one, don't replace it.

- [ ] **Step 2: Write `style-tokens.css`**

Seven custom properties per style: `--nx-style-radius`, `--nx-style-border-width`, `--nx-style-border-color`, `--nx-style-shadow`, `--nx-style-blur`, `--nx-style-bg`, `--nx-style-font-family`. All 12 blocks reference the existing `--nx-*` palette variables (via `color-mix()`) so every style combines correctly with all 12 color palettes, rather than hardcoding colors that would clash with a user's chosen palette.

```css
/* src/styles/style-tokens.css
 *
 * Defines the 7 CSS custom properties every Nx* shared component reads
 * for its visual "style" character (independent of the active color
 * palette, set by themeStore, and the active structural layout, set by
 * layoutStore). Scoped by the `data-nx-style` attribute that styleStore
 * writes to <html>. Falls back to glass-glow's values via the unscoped
 * :root block below, so components never see an unset custom property
 * even before styleStore's mount-time hook runs.
 */

:root {
  --nx-style-radius: 14px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: color-mix(in srgb, var(--nx-accent-primary) 30%, transparent);
  --nx-style-shadow: 0 0 24px color-mix(in srgb, var(--nx-accent-primary) 20%, transparent);
  --nx-style-blur: 12px;
  --nx-style-bg: color-mix(in srgb, var(--nx-bg-elevated) 60%, transparent);
  --nx-style-font-family: inherit;
}

[data-nx-style="glass-glow"] {
  --nx-style-radius: 14px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: color-mix(in srgb, var(--nx-accent-primary) 30%, transparent);
  --nx-style-shadow: 0 0 24px color-mix(in srgb, var(--nx-accent-primary) 20%, transparent);
  --nx-style-blur: 12px;
  --nx-style-bg: color-mix(in srgb, var(--nx-bg-elevated) 60%, transparent);
  --nx-style-font-family: inherit;
}

[data-nx-style="flat-modern"] {
  --nx-style-radius: 8px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: var(--nx-border);
  --nx-style-shadow: none;
  --nx-style-blur: 0px;
  --nx-style-bg: var(--nx-bg-elevated);
  --nx-style-font-family: inherit;
}

[data-nx-style="neon-terminal"] {
  --nx-style-radius: 4px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: color-mix(in srgb, var(--nx-accent-primary) 40%, transparent);
  --nx-style-shadow: inset 0 0 16px color-mix(in srgb, var(--nx-accent-primary) 10%, transparent);
  --nx-style-blur: 0px;
  --nx-style-bg: var(--nx-bg-elevated);
  --nx-style-font-family: "Courier New", monospace;
}

[data-nx-style="neumorphism"] {
  --nx-style-radius: 14px;
  --nx-style-border-width: 0px;
  --nx-style-border-color: transparent;
  --nx-style-shadow: 6px 6px 12px color-mix(in srgb, black 15%, transparent), -6px -6px 12px color-mix(in srgb, white 6%, transparent);
  --nx-style-blur: 0px;
  --nx-style-bg: var(--nx-bg-elevated);
  --nx-style-font-family: inherit;
}

[data-nx-style="brutalism"] {
  --nx-style-radius: 0px;
  --nx-style-border-width: 3px;
  --nx-style-border-color: var(--nx-text-primary);
  --nx-style-shadow: 5px 5px 0 var(--nx-text-primary);
  --nx-style-blur: 0px;
  --nx-style-bg: var(--nx-bg-elevated);
  --nx-style-font-family: inherit;
}

[data-nx-style="paper"] {
  --nx-style-radius: 6px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: var(--nx-border);
  --nx-style-shadow: 0 1px 3px rgba(0, 0, 0, 0.15);
  --nx-style-blur: 0px;
  --nx-style-bg: var(--nx-bg-elevated);
  --nx-style-font-family: inherit;
}

[data-nx-style="aero-glass"] {
  --nx-style-radius: 10px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: color-mix(in srgb, white 40%, transparent);
  --nx-style-shadow: 0 2px 6px rgba(0, 0, 0, 0.2), inset 0 1px 0 color-mix(in srgb, white 50%, transparent);
  --nx-style-blur: 6px;
  --nx-style-bg: linear-gradient(180deg, color-mix(in srgb, white 30%, transparent), color-mix(in srgb, white 5%, transparent));
  --nx-style-font-family: inherit;
}

[data-nx-style="cyber-grid"] {
  --nx-style-radius: 2px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: color-mix(in srgb, var(--nx-accent-primary) 35%, transparent);
  --nx-style-shadow: 0 0 12px color-mix(in srgb, var(--nx-accent-primary) 15%, transparent);
  --nx-style-blur: 0px;
  --nx-style-bg: color-mix(in srgb, var(--nx-accent-primary) 5%, var(--nx-bg-elevated));
  --nx-style-font-family: inherit;
}

[data-nx-style="line-art"] {
  --nx-style-radius: 2px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: var(--nx-border);
  --nx-style-shadow: none;
  --nx-style-blur: 0px;
  --nx-style-bg: transparent;
  --nx-style-font-family: "Courier New", monospace;
}

[data-nx-style="gradient-mesh"] {
  --nx-style-radius: 14px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: color-mix(in srgb, white 30%, transparent);
  --nx-style-shadow: none;
  --nx-style-blur: 8px;
  --nx-style-bg: linear-gradient(135deg, color-mix(in srgb, var(--nx-accent-primary) 25%, transparent), color-mix(in srgb, var(--nx-accent-secondary) 25%, transparent));
  --nx-style-font-family: inherit;
}

[data-nx-style="amber-crt"] {
  --nx-style-radius: 2px;
  --nx-style-border-width: 1px;
  --nx-style-border-color: color-mix(in srgb, var(--nx-accent-primary) 40%, transparent);
  --nx-style-shadow: inset 0 0 16px color-mix(in srgb, var(--nx-accent-primary) 10%, transparent);
  --nx-style-blur: 0px;
  --nx-style-bg: var(--nx-bg-elevated);
  --nx-style-font-family: "Courier New", monospace;
}

[data-nx-style="mono-contrast"] {
  --nx-style-radius: 0px;
  --nx-style-border-width: 2px;
  --nx-style-border-color: var(--nx-text-primary);
  --nx-style-shadow: none;
  --nx-style-blur: 0px;
  --nx-style-bg: var(--nx-bg-base);
  --nx-style-font-family: inherit;
}
```

- [ ] **Step 3: Import it globally in `main.ts`**

Add `import "./styles/style-tokens.css";` alongside `main.ts`'s existing global CSS import (do not remove or reorder the existing import — add this as an additional line).

- [ ] **Step 4: Verify it builds**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit && npx vite build 2>&1 | tail -20"`
Expected: build succeeds, no CSS parse errors reported by Vite/esbuild's CSS minifier (a malformed `color-mix()`/`linear-gradient()` call would surface here as a build warning or error).

- [ ] **Step 5: Commit**

```bash
git add src/styles/style-tokens.css src/main.ts
git commit -m "feat: add 12-style CSS custom-property token set"
```

---

## Task 4: NxCard + NxSectionHeader

**Files:**
- Create: `src/components/ui/NxCard.vue`
- Create: `src/components/ui/NxSectionHeader.vue`
- Test: `src/components/ui/NxCard.spec.ts`
- Test: `src/components/ui/NxSectionHeader.spec.ts`

- [ ] **Step 1: Write the failing tests**

```typescript
// src/components/ui/NxCard.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxCard from "./NxCard.vue";

describe("NxCard", () => {
  it("renders default slot content", () => {
    const wrapper = mount(NxCard, { slots: { default: "<p class=\"probe\">hello</p>" } });
    expect(wrapper.find(".probe").exists()).toBe(true);
    expect(wrapper.find(".probe").text()).toBe("hello");
  });

  it("applies the nx-card base class", () => {
    const wrapper = mount(NxCard);
    expect(wrapper.classes()).toContain("nx-card");
  });

  it("adds a danger modifier class when the danger prop is set", () => {
    const wrapper = mount(NxCard, { props: { danger: true } });
    expect(wrapper.classes()).toContain("nx-card--danger");
  });
});
```

```typescript
// src/components/ui/NxSectionHeader.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxSectionHeader from "./NxSectionHeader.vue";

describe("NxSectionHeader", () => {
  it("renders the title", () => {
    const wrapper = mount(NxSectionHeader, { props: { title: "Disques & partitions" } });
    expect(wrapper.text()).toContain("Disques & partitions");
  });

  it("renders an optional description when provided", () => {
    const wrapper = mount(NxSectionHeader, { props: { title: "T", description: "Une description." } });
    expect(wrapper.text()).toContain("Une description.");
  });

  it("omits the description element entirely when not provided", () => {
    const wrapper = mount(NxSectionHeader, { props: { title: "T" } });
    expect(wrapper.find(".nx-section-header__description").exists()).toBe(false);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/ui/NxCard.spec.ts src/components/ui/NxSectionHeader.spec.ts"`
Expected: FAIL — components don't exist.

- [ ] **Step 3: Write `NxCard.vue`**

```vue
<!-- src/components/ui/NxCard.vue -->
<script setup lang="ts">
withDefaults(defineProps<{ danger?: boolean }>(), { danger: false });
</script>

<template>
  <div class="nx-card" :class="{ 'nx-card--danger': danger }">
    <slot />
  </div>
</template>

<style scoped>
.nx-card {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  box-shadow: var(--nx-style-shadow);
  backdrop-filter: blur(var(--nx-style-blur));
  -webkit-backdrop-filter: blur(var(--nx-style-blur));
  background: var(--nx-style-bg);
  font-family: var(--nx-style-font-family);
  color: var(--nx-text-primary);
  padding: 16px;
}
.nx-card--danger {
  border-color: color-mix(in srgb, var(--nx-accent-danger) 50%, transparent);
}
</style>
```

- [ ] **Step 4: Write `NxSectionHeader.vue`**

```vue
<!-- src/components/ui/NxSectionHeader.vue -->
<script setup lang="ts">
defineProps<{ title: string; description?: string }>();
</script>

<template>
  <div class="nx-section-header">
    <h2 class="nx-section-header__title">{{ title }}</h2>
    <p v-if="description" class="nx-section-header__description">{{ description }}</p>
  </div>
</template>

<style scoped>
.nx-section-header { margin-bottom: 16px; }
.nx-section-header__title { margin: 0; font-size: 18px; font-weight: 700; color: var(--nx-text-primary); font-family: var(--nx-style-font-family); }
.nx-section-header__description { margin: 4px 0 0; font-size: 13px; color: var(--nx-text-secondary); }
</style>
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/ui/NxCard.spec.ts src/components/ui/NxSectionHeader.spec.ts"`
Expected: PASS (6 tests)

- [ ] **Step 6: Commit**

```bash
git add src/components/ui/NxCard.vue src/components/ui/NxSectionHeader.vue src/components/ui/NxCard.spec.ts src/components/ui/NxSectionHeader.spec.ts
git commit -m "feat: add NxCard and NxSectionHeader shared UI components"
```

---

## Task 5: NxButton + NxBadge

**Files:**
- Create: `src/components/ui/NxButton.vue`
- Create: `src/components/ui/NxBadge.vue`
- Test: `src/components/ui/NxButton.spec.ts`
- Test: `src/components/ui/NxBadge.spec.ts`

- [ ] **Step 1: Write the failing tests**

```typescript
// src/components/ui/NxButton.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxButton from "./NxButton.vue";

describe("NxButton", () => {
  it("renders default slot content", () => {
    const wrapper = mount(NxButton, { slots: { default: "Enregistrer" } });
    expect(wrapper.text()).toBe("Enregistrer");
  });

  it("defaults to the 'default' variant class", () => {
    const wrapper = mount(NxButton);
    expect(wrapper.classes()).toContain("nx-button--default");
  });

  it.each(["default", "danger", "ghost"] as const)("applies the %s variant class", (variant) => {
    const wrapper = mount(NxButton, { props: { variant } });
    expect(wrapper.classes()).toContain(`nx-button--${variant}`);
  });

  it("emits a click event when clicked and not disabled", async () => {
    const wrapper = mount(NxButton);
    await wrapper.trigger("click");
    expect(wrapper.emitted("click")).toHaveLength(1);
  });

  it("reflects the disabled prop on the underlying <button> element", () => {
    const wrapper = mount(NxButton, { props: { disabled: true } });
    expect(wrapper.attributes("disabled")).toBeDefined();
  });
});
```

```typescript
// src/components/ui/NxBadge.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxBadge from "./NxBadge.vue";

describe("NxBadge", () => {
  it("renders default slot content", () => {
    const wrapper = mount(NxBadge, { slots: { default: "Actif" } });
    expect(wrapper.text()).toBe("Actif");
  });

  it.each(["success", "warning", "danger", "info"] as const)("applies the %s status class", (status) => {
    const wrapper = mount(NxBadge, { props: { status }, slots: { default: "x" } });
    expect(wrapper.classes()).toContain(`nx-badge--${status}`);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/ui/NxButton.spec.ts src/components/ui/NxBadge.spec.ts"`
Expected: FAIL — components don't exist.

- [ ] **Step 3: Write `NxButton.vue`**

```vue
<!-- src/components/ui/NxButton.vue -->
<script setup lang="ts">
withDefaults(
  defineProps<{ variant?: "default" | "danger" | "ghost"; disabled?: boolean }>(),
  { variant: "default", disabled: false },
);
defineEmits<{ click: [MouseEvent] }>();
</script>

<template>
  <button
    class="nx-button"
    :class="`nx-button--${variant}`"
    :disabled="disabled"
    @click="$emit('click', $event)"
  >
    <slot />
  </button>
</template>

<style scoped>
.nx-button {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  font-family: var(--nx-style-font-family);
  padding: 8px 14px;
  cursor: pointer;
  font-size: 13px;
  font-weight: 600;
}
.nx-button:disabled { opacity: 0.5; cursor: not-allowed; }
.nx-button--default { background: var(--nx-style-bg); color: var(--nx-text-primary); }
.nx-button--danger { background: var(--nx-accent-danger); color: white; border-color: var(--nx-accent-danger); }
.nx-button--ghost { background: transparent; color: var(--nx-text-secondary); border-color: transparent; }
</style>
```

- [ ] **Step 4: Write `NxBadge.vue`**

```vue
<!-- src/components/ui/NxBadge.vue -->
<script setup lang="ts">
withDefaults(defineProps<{ status?: "success" | "warning" | "danger" | "info" }>(), { status: "info" });
</script>

<template>
  <span class="nx-badge" :class="`nx-badge--${status}`">
    <slot />
  </span>
</template>

<style scoped>
.nx-badge {
  display: inline-flex;
  align-items: center;
  padding: 3px 10px;
  border-radius: 99px;
  font-size: 11px;
  font-weight: 600;
  font-family: var(--nx-style-font-family);
}
.nx-badge--success { background: color-mix(in srgb, var(--nx-accent-success) 18%, transparent); color: var(--nx-accent-success); }
.nx-badge--warning { background: color-mix(in srgb, var(--nx-accent-warning) 18%, transparent); color: var(--nx-accent-warning); }
.nx-badge--danger { background: color-mix(in srgb, var(--nx-accent-danger) 18%, transparent); color: var(--nx-accent-danger); }
.nx-badge--info { background: color-mix(in srgb, var(--nx-accent-primary) 18%, transparent); color: var(--nx-accent-primary); }
</style>
```

- [ ] **Step 5: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/ui/NxButton.spec.ts src/components/ui/NxBadge.spec.ts"`
Expected: PASS (12 tests — NxButton: 1 + 1 + 3 (it.each) + 1 + 1 = 7; NxBadge: 1 + 4 (it.each) = 5)

- [ ] **Step 6: Commit**

```bash
git add src/components/ui/NxButton.vue src/components/ui/NxBadge.vue src/components/ui/NxButton.spec.ts src/components/ui/NxBadge.spec.ts
git commit -m "feat: add NxButton and NxBadge shared UI components"
```

---

## Task 6: NxInput + NxSelect + NxStatTile

**Files:**
- Create: `src/components/ui/NxInput.vue`
- Create: `src/components/ui/NxSelect.vue`
- Create: `src/components/ui/NxStatTile.vue`
- Test: `src/components/ui/NxInput.spec.ts`
- Test: `src/components/ui/NxSelect.spec.ts`
- Test: `src/components/ui/NxStatTile.spec.ts`

- [ ] **Step 1: Write the failing tests**

```typescript
// src/components/ui/NxInput.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxInput from "./NxInput.vue";

describe("NxInput", () => {
  it("binds via v-model", async () => {
    const wrapper = mount(NxInput, { props: { modelValue: "" } });
    await wrapper.find("input").setValue("/dev/sda1");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual(["/dev/sda1"]);
  });

  it("forwards the placeholder prop to the underlying input", () => {
    const wrapper = mount(NxInput, { props: { modelValue: "", placeholder: "Chemin..." } });
    expect(wrapper.find("input").attributes("placeholder")).toBe("Chemin...");
  });
});
```

```typescript
// src/components/ui/NxSelect.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxSelect from "./NxSelect.vue";

describe("NxSelect", () => {
  const options = [
    { value: "ext4", label: "ext4" },
    { value: "btrfs", label: "btrfs" },
  ];

  it("renders one <option> per entry", () => {
    const wrapper = mount(NxSelect, { props: { modelValue: "ext4", options } });
    expect(wrapper.findAll("option")).toHaveLength(2);
  });

  it("binds via v-model", async () => {
    const wrapper = mount(NxSelect, { props: { modelValue: "ext4", options } });
    await wrapper.find("select").setValue("btrfs");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual(["btrfs"]);
  });
});
```

```typescript
// src/components/ui/NxStatTile.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import NxStatTile from "./NxStatTile.vue";

describe("NxStatTile", () => {
  it("renders the label and value", () => {
    const wrapper = mount(NxStatTile, { props: { label: "CPU", value: "34%" } });
    expect(wrapper.text()).toContain("CPU");
    expect(wrapper.text()).toContain("34%");
  });

  it("renders an optional status dot with the given status class when provided", () => {
    const wrapper = mount(NxStatTile, { props: { label: "CPU", value: "34%", status: "success" } });
    expect(wrapper.find(".nx-stat-tile__dot--success").exists()).toBe(true);
  });

  it("omits the status dot entirely when no status is provided", () => {
    const wrapper = mount(NxStatTile, { props: { label: "CPU", value: "34%" } });
    expect(wrapper.find(".nx-stat-tile__dot").exists()).toBe(false);
  });
});
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/ui/NxInput.spec.ts src/components/ui/NxSelect.spec.ts src/components/ui/NxStatTile.spec.ts"`
Expected: FAIL — components don't exist.

- [ ] **Step 3: Write `NxInput.vue`**

```vue
<!-- src/components/ui/NxInput.vue -->
<script setup lang="ts">
defineProps<{ modelValue: string; placeholder?: string }>();
defineEmits<{ "update:modelValue": [string] }>();
</script>

<template>
  <input
    class="nx-input"
    :value="modelValue"
    :placeholder="placeholder"
    @input="$emit('update:modelValue', ($event.target as HTMLInputElement).value)"
  />
</template>

<style scoped>
.nx-input {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  background: var(--nx-style-bg);
  color: var(--nx-text-primary);
  font-family: var(--nx-style-font-family);
  padding: 8px 10px;
  font-size: 13px;
  width: 100%;
  box-sizing: border-box;
}
</style>
```

- [ ] **Step 4: Write `NxSelect.vue`**

```vue
<!-- src/components/ui/NxSelect.vue -->
<script setup lang="ts">
defineProps<{ modelValue: string; options: { value: string; label: string }[] }>();
defineEmits<{ "update:modelValue": [string] }>();
</script>

<template>
  <select
    class="nx-select"
    :value="modelValue"
    @change="$emit('update:modelValue', ($event.target as HTMLSelectElement).value)"
  >
    <option v-for="opt in options" :key="opt.value" :value="opt.value">{{ opt.label }}</option>
  </select>
</template>

<style scoped>
.nx-select {
  border-radius: var(--nx-style-radius);
  border: var(--nx-style-border-width) solid var(--nx-style-border-color);
  background: var(--nx-style-bg);
  color: var(--nx-text-primary);
  font-family: var(--nx-style-font-family);
  padding: 8px 10px;
  font-size: 13px;
}
</style>
```

- [ ] **Step 5: Write `NxStatTile.vue`**

```vue
<!-- src/components/ui/NxStatTile.vue -->
<script setup lang="ts">
defineProps<{ label: string; value: string; status?: "success" | "warning" | "danger" }>();
</script>

<template>
  <div class="nx-stat-tile">
    <div class="nx-stat-tile__label-row">
      <span v-if="status" class="nx-stat-tile__dot" :class="`nx-stat-tile__dot--${status}`" />
      <span class="nx-stat-tile__label">{{ label }}</span>
    </div>
    <div class="nx-stat-tile__value">{{ value }}</div>
  </div>
</template>

<style scoped>
.nx-stat-tile { font-family: var(--nx-style-font-family); }
.nx-stat-tile__label-row { display: flex; align-items: center; gap: 6px; margin-bottom: 4px; }
.nx-stat-tile__label { font-size: 12px; color: var(--nx-text-secondary); }
.nx-stat-tile__value { font-size: 22px; font-weight: 700; color: var(--nx-text-primary); }
.nx-stat-tile__dot { width: 7px; height: 7px; border-radius: 50%; }
.nx-stat-tile__dot--success { background: var(--nx-accent-success); box-shadow: 0 0 6px var(--nx-accent-success); }
.nx-stat-tile__dot--warning { background: var(--nx-accent-warning); box-shadow: 0 0 6px var(--nx-accent-warning); }
.nx-stat-tile__dot--danger { background: var(--nx-accent-danger); box-shadow: 0 0 6px var(--nx-accent-danger); }
</style>
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/ui/NxInput.spec.ts src/components/ui/NxSelect.spec.ts src/components/ui/NxStatTile.spec.ts"`
Expected: PASS (7 tests)

- [ ] **Step 7: Commit**

```bash
git add src/components/ui/NxInput.vue src/components/ui/NxSelect.vue src/components/ui/NxStatTile.vue src/components/ui/NxInput.spec.ts src/components/ui/NxSelect.spec.ts src/components/ui/NxStatTile.spec.ts
git commit -m "feat: add NxInput, NxSelect, and NxStatTile shared UI components"
```

---

## Task 7: Navigation categories data model

**Files:**
- Create: `src/navigation/categories.ts`
- Test: `src/navigation/categories.spec.ts`

**Files reflect the target Phase R2 page ids** (some pages don't exist yet under these ids — that's fine, this task only builds the *data model*; `AppNav.vue` in Task 8 renders whatever this list contains, and Phase R2 is what makes each `pageId` resolve to a real page component in `App.vue`).

- [ ] **Step 1: Write the failing test**

```typescript
// src/navigation/categories.spec.ts
import { describe, it, expect } from "vitest";
import { navigationCategories } from "./categories";

describe("navigationCategories", () => {
  it("has exactly 7 categories", () => {
    expect(navigationCategories).toHaveLength(7);
  });

  it("every page id is unique across all categories", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(new Set(allPageIds).size).toBe(allPageIds.length);
  });

  it("every category has at least one page", () => {
    for (const category of navigationCategories) {
      expect(category.pages.length).toBeGreaterThan(0);
    }
  });

  it("includes the 4 new Phase R1+ feature pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("quick-install");
    expect(allPageIds).toContain("updates");
    expect(allPageIds).toContain("report-generator");
    expect(allPageIds).toContain("settings-preferences");
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/navigation/categories.spec.ts"`
Expected: FAIL — module not found.

- [ ] **Step 3: Write `categories.ts`**

```typescript
// src/navigation/categories.ts
export interface NavPage {
  id: string;
  label: string;
  icon: string;
}

export interface NavCategory {
  id: string;
  title: string;
  pages: NavPage[];
}

export const navigationCategories: NavCategory[] = [
  {
    id: "systeme",
    title: "Système",
    pages: [
      { id: "dashboard", label: "Tableau de bord", icon: "layout-dashboard" },
      { id: "diagnostic", label: "Diagnostic", icon: "stethoscope" },
    ],
  },
  {
    id: "applications",
    title: "Applications",
    pages: [
      { id: "quick-install", label: "Installation rapide", icon: "download" },
      { id: "package-manager", label: "Gestionnaire de paquets", icon: "package" },
    ],
  },
  {
    id: "stockage",
    title: "Stockage",
    pages: [
      { id: "disks", label: "Disques & partitions", icon: "hard-drive" },
      { id: "file-tools", label: "Doublons / Gros fichiers / Hash", icon: "files" },
    ],
  },
  {
    id: "maintenance",
    title: "Maintenance",
    pages: [
      { id: "updates", label: "Mises à jour", icon: "refresh-cw" },
      { id: "drivers", label: "Pilotes", icon: "cpu" },
      { id: "troubleshoot", label: "Dépannage", icon: "wrench" },
    ],
  },
  {
    id: "reseau",
    title: "Réseau",
    pages: [
      { id: "network-overview", label: "Vue d'ensemble", icon: "wifi" },
      { id: "firewall", label: "Pare-feu", icon: "shield" },
    ],
  },
  {
    id: "rapports",
    title: "Rapports",
    pages: [
      { id: "report-generator", label: "Générateur de rapport", icon: "file-text" },
      { id: "logs", label: "Journaux", icon: "scroll-text" },
    ],
  },
  {
    id: "parametres",
    title: "Paramètres",
    pages: [
      { id: "settings-preferences", label: "Préférences", icon: "settings" },
      { id: "settings-appearance", label: "Thèmes & dispositions", icon: "palette" },
    ],
  },
];
```

- [ ] **Step 4: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/navigation/categories.spec.ts"`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add src/navigation/categories.ts src/navigation/categories.spec.ts
git commit -m "feat: add categorized navigation data model (7 categories, spec section 2.1)"
```

---

## Task 8: AppNav.vue

**Files:**
- Create: `src/components/nav/AppNav.vue`
- Test: `src/components/nav/AppNav.spec.ts`

**Not wired into `App.vue` in this plan** — Phase R2 replaces `App.vue`'s current flat button list with `<AppNav v-model="currentPage" />`. This task builds and tests the component standalone.

- [ ] **Step 1: Write the failing test**

```typescript
// src/components/nav/AppNav.spec.ts
import { describe, it, expect } from "vitest";
import { mount } from "@vue/test-utils";
import AppNav from "./AppNav.vue";
import { navigationCategories } from "@/navigation/categories";

describe("AppNav", () => {
  it("renders every category title", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    for (const category of navigationCategories) {
      expect(wrapper.text()).toContain(category.title);
    }
  });

  it("renders every page label", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    const allPages = navigationCategories.flatMap((c) => c.pages);
    for (const page of allPages) {
      expect(wrapper.text()).toContain(page.label);
    }
  });

  it("marks the page matching modelValue as active", () => {
    const wrapper = mount(AppNav, { props: { modelValue: "disks" } });
    const activeButtons = wrapper.findAll("button.active");
    expect(activeButtons).toHaveLength(1);
    expect(activeButtons[0].text()).toBe("Disques & partitions");
  });

  it("emits update:modelValue with the clicked page's id", async () => {
    const wrapper = mount(AppNav, { props: { modelValue: "dashboard" } });
    const buttons = wrapper.findAll("button");
    const target = buttons.find((b) => b.text() === "Pilotes")!;
    await target.trigger("click");
    expect(wrapper.emitted("update:modelValue")?.[0]).toEqual(["drivers"]);
  });
});
```

- [ ] **Step 2: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/nav/AppNav.spec.ts"`
Expected: FAIL — component doesn't exist.

- [ ] **Step 3: Write `AppNav.vue`**

```vue
<!-- src/components/nav/AppNav.vue -->
<script setup lang="ts">
import { navigationCategories } from "@/navigation/categories";

defineProps<{ modelValue: string }>();
defineEmits<{ "update:modelValue": [string] }>();
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
        {{ page.label }}
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
  display: block;
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
.nx-app-nav__item:hover { background: var(--nx-style-bg); color: var(--nx-text-primary); }
.nx-app-nav__item.active { background: var(--nx-style-bg); color: var(--nx-text-primary); font-weight: 600; }
</style>
```

- [ ] **Step 4: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/components/nav/AppNav.spec.ts"`
Expected: PASS (4 tests)

- [ ] **Step 5: Commit**

```bash
git add src/components/nav/AppNav.vue src/components/nav/AppNav.spec.ts
git commit -m "feat: add AppNav categorized navigation component"
```

---

## Task 9: Extend ThemeEditorPage with the style picker

**Files:**
- Modify: `src/pages/ThemeEditorPage.vue`
- Test: create `src/pages/ThemeEditorPage.spec.ts` (none exists yet for this page)

- [ ] **Step 1: Read the CURRENT `src/pages/ThemeEditorPage.vue` in full** (reproduced in this plan's context-gathering — 2 tabs: "Thème", "Disposition"). You are adding a 3rd tab "Style" following the exact same pattern as the "Disposition" tab (a grid of selectable option cards backed by a store), not inventing a new UI pattern.

- [ ] **Step 2: Write the failing test**

```typescript
// src/pages/ThemeEditorPage.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import ThemeEditorPage from "./ThemeEditorPage.vue";
import { useStyleStore } from "@/stores/styleStore";
import { styleRegistry } from "@/styles/registry";

describe("ThemeEditorPage — style tab", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("shows a 'Style' tab button alongside the existing Thème/Disposition tabs", () => {
    const wrapper = mount(ThemeEditorPage);
    expect(wrapper.text()).toContain("Style");
  });

  it("lists all 12 styles when the Style tab is active", async () => {
    const wrapper = mount(ThemeEditorPage);
    const tabs = wrapper.findAll("button");
    const styleTab = tabs.find((b) => b.text() === "Style")!;
    await styleTab.trigger("click");
    for (const style of styleRegistry) {
      expect(wrapper.text()).toContain(style.name);
    }
  });

  it("clicking a style option calls styleStore.setStyle with that style's id", async () => {
    const wrapper = mount(ThemeEditorPage);
    const store = useStyleStore();
    const tabs = wrapper.findAll("button");
    const styleTab = tabs.find((b) => b.text() === "Style")!;
    await styleTab.trigger("click");
    const brutalismOption = wrapper.findAll(".te-style-option").find((el) => el.text().includes("Brutalisme"))!;
    await brutalismOption.trigger("click");
    expect(store.current).toBe("brutalism");
  });
});
```

- [ ] **Step 3: Run test to verify it fails**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/ThemeEditorPage.spec.ts"`
Expected: FAIL — no "Style" tab exists yet.

- [ ] **Step 4: Modify `ThemeEditorPage.vue`**

In the `<script setup>` block, add imports and extend `activeTab`'s type:

```typescript
import { useStyleStore } from "@/stores/styleStore";
import { styleRegistry } from "@/styles/registry";
```

Change:
```typescript
const activeTab = ref<"theme" | "layout">("theme");
```
to:
```typescript
const activeTab = ref<"theme" | "layout" | "style">("theme");
```

Add after the existing `const layoutStore = useLayoutStore();` line:
```typescript
const styleStore = useStyleStore();
```

In the `<template>`, add a third tab button after the existing "Disposition" button (inside `.te-tabs`):
```html
<button :class="{ active: activeTab === 'style' }" @click="activeTab = 'style'">Style</button>
```

Add a third `<section>` after the existing layout `<section>` (which currently uses `v-else` — change that one to `v-else-if="activeTab === 'layout'"` so the new style section can be the final `v-else`):

```html
<section v-else class="te-panel">
  <div class="te-layouts">
    <button
      v-for="style in styleRegistry"
      :key="style.id"
      class="te-layout-option te-style-option"
      :class="{ active: styleStore.current === style.id }"
      @click="styleStore.setStyle(style.id)"
    >
      <strong>{{ style.name }}</strong>
      <p>{{ style.description }}</p>
    </button>
  </div>
</section>
```

(Reuses the existing `.te-layouts`/`.te-layout-option` CSS classes already defined in this file's `<style scoped>` block — no new CSS needed. `.te-style-option` is an additional class purely so the test above can target style options specifically without also matching layout options, since both reuse `.te-layout-option`.)

- [ ] **Step 5: Run test to verify it passes**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vitest run src/pages/ThemeEditorPage.spec.ts"`
Expected: PASS (3 tests)

- [ ] **Step 6: Commit**

```bash
git add src/pages/ThemeEditorPage.vue src/pages/ThemeEditorPage.spec.ts
git commit -m "feat: add 3rd style picker tab to ThemeEditorPage (live preview, spec section 3.2)"
```

---

## Task 10: Full verification pass

**Files:** None (verification-only).

- [ ] **Step 1: Run the full test suite**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npm run test -- --run 2>&1 | tail -30"`
Expected: all test files pass, including every new spec added in Tasks 1–9. Compare the total test count against the pre-R1 baseline (25 tests) — expect 25 + 3 (registry) + 15 (styleStore) + 6 (NxCard/NxSectionHeader) + 12 (NxButton/NxBadge) + 7 (NxInput/NxSelect/NxStatTile) + 4 (categories) + 4 (AppNav) + 3 (ThemeEditorPage) = 79 tests.

- [ ] **Step 2: Type-check**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX && npx vue-tsc --noEmit"`
Expected: clean, no errors.

- [ ] **Step 3: Confirm the Rust suite is unaffected**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/src-tauri && cargo test 2>&1 | tail -10"`
Expected: unchanged from before this plan (this plan touches zero Rust code) — 124 passed, 1 ignored, 0 failed.

- [ ] **Step 4: Manual review — confirm no page content moved**

Run: `git diff master --stat` (from the worktree, comparing against the branch point) and confirm `src/App.vue`, `src/pages/DisksPage.vue`, `src/pages/SecurityPage.vue`, `src/pages/NetworkPage.vue`, `src/pages/HardwarePage.vue`, `src/pages/PackagesPage.vue`, `src/pages/LogsPage.vue` do **not** appear in the diff (only `src/pages/ThemeEditorPage.vue` should, per Task 9) — confirming this plan stayed foundation-only as scoped, with all page restructuring correctly deferred to Phase R2.

- [ ] **Step 5: Commit any final cleanup, then this plan is complete**

No further commit expected if Steps 1–4 all pass clean — this step exists only to catch and fix anything Steps 1–4 surfaced.
