# NiTruX Redesign — Navigation, Visual System & New Categories (Design Spec)

## 1. Problem statement

NiTruX (the Linux port of NiTriTe) currently ships 9 flat pages with no categorization: `App.vue` hardcodes a single-column list of 9 nav buttons and a `Record<PageId, Component>` map. Compared to NiTriTe on Windows (47 pages across 10 categorized nav sections, a mature "glass & glow" visual language with blur/glow/gradients, and dedicated flagship features like Master Install, Updates, Drivers, and a multi-format config report generator), NiTruX reads as a prototype: each page hand-rolls its own scoped CSS (slightly different card/input/button styles per page), several real features exist only as buried sub-tabs inside oversized pages (e.g. `DisksPage.vue` crams disk listing, duplicate-file scanning, large-file scanning, hash checking, and the format/extend/clone controls into one flat tab bar), and whole categories NiTriTe users expect (a curated one-click app installer, a dedicated Updates page, richer Drivers, and report export) don't exist at all.

The user has explicitly asked for a full redesign — matching or exceeding NiTriTe's polish and completeness — validated interactively via the visual brainstorming companion on 2026-08-01. This spec captures what was agreed.

## 2. Navigation architecture

### 2.1 Categories (confirmed)

Nine top-level categories, replacing the flat 9-button list:

| Category | Pages |
|---|---|
| **Système** | Tableau de bord, Diagnostic |
| **Applications** | Installation rapide (NEW), Gestionnaire de paquets (existing `PackagesPage`) |
| **Stockage** | Disques & partitions, Doublons / Gros fichiers / Hash (split out of the current `DisksPage` mega-tabs) |
| **Maintenance** | Mises à jour (NEW), Pilotes (existing `DriversPage`, enriched), Dépannage (existing troubleshoot/snapshot/malware content, moved out of `SecurityPage`) |
| **Réseau** | Vue d'ensemble (existing `NetworkPage` core), Pare-feu (moved out of `SecurityPage`) |
| **Rapports** | Générateur de rapport (NEW), Journaux (existing `LogsPage`) |
| **Paramètres** | Préférences (NEW — see §4.5), Thèmes & dispositions (existing `ThemeEditorPage`, extended — see §3) |

"Diagnostic" under Système absorbs the existing `HardwarePage` content (renamed for clarity — it's system diagnostic info, not a hardware-shopping page).

**Why this shape:** it mirrors NiTriTe's category *names and spirit* (per explicit user request — "une copie ou une version encore mieux") while dropping categories that don't apply to a Linux tool (WinPE, Intelligence/AI-agents, BSOD Analyzer) and merging "Apparence" + "Configuration" into a single "Paramètres" category (explicit user correction during brainstorming). Sécurité is folded into Maintenance/Réseau rather than kept as its own top-level category — the malware scan, troubleshoot actions, and snapshot creation are maintenance activities; the firewall status is a network concern. This avoids a near-empty "Sécurité" category once its 3 sub-features are redistributed to where a user would actually look for them.

### 2.2 Routing mechanism

**No vue-router.** The app stays a router-less desktop SPA (no URL bar to exploit deep-linking for), consistent with its current architecture. Replace the flat `PageId` union + `Record<PageId, Component>` in `App.vue` with:

- `src/navigation/categories.ts` — the category/page data structure (id, label, icon name, category id), analogous to NiTriTe's `navigation.ts` structure but Linux-scoped per §2.1.
- `src/components/nav/AppNav.vue` — renders the categorized nav (collapsible sections, active-item highlighting) from `categories.ts`, replacing the hardcoded button list currently inlined in `App.vue`. Slotted into `LayoutShell`'s existing `#nav` slot exactly as today's flat list is — **no changes needed to any of the 8 layout components**, since they only render whatever's given to that slot.
- `App.vue` keeps a single `ref<PageId>` (now a flat string id like `"quick-install"`, `"reports-generate"` — flat ids, nested only in the *data structure* used to render the nav, not in routing logic) and a flattened `Record<PageId, Component>` built from `categories.ts`, so adding a page later means one entry in `categories.ts`, not edits in two places.

**Rejected alternative:** introducing vue-router. Would add a dependency and URL-based state neither useful (no browser chrome) nor requested, for a ~25-30 page app that the existing ref-based pattern already handles fine once organized by category data instead of ad-hoc booleans.

## 3. Visual system — three independent, combinable axes

NiTruX already has two working axes (12 color palettes via `themeStore`, 8 structural layouts via `layoutStore`). This redesign adds a **third axis: visual style** (12 style "treatments" — border/shadow/blur/typography character, independent of color) confirmed via the visual companion:

| id | Name | Character |
|---|---|---|
| `glass-glow` | Verre & lueur | Blur, translucency, colored glow shadows, gradient text — closest to NiTriTe's actual CSS (confirmed by reading its `main.css`: `backdrop-filter: blur(12px)`, glow box-shadows, pulsing status dots) |
| `flat-modern` | Flat moderne | Solid cards, sharp borders, no blur/glow |
| `neon-terminal` | Neon Terminal | Monospace, green CRT glow |
| `neumorphism` | Néumorphisme | Soft embossed dual-shadow, monochrome base |
| `brutalism` | Brutalisme | Bold black borders, hard offset shadows, no radius |
| `paper` | Papier minimal | Very light, subtle shadow, editorial |
| `aero-glass` | Aero Glass | Glossy reflective highlight gradient (Windows Aero nod) |
| `cyber-grid` | Cyber Grid | Faint grid background, angular clipped corners, cyan glow |
| `line-art` | Ligne claire | 1px outlines only, no fills, technical/blueprint |
| `gradient-mesh` | Gradient Mesh | Vibrant multi-color gradient backgrounds behind glass cards |
| `amber-crt` | CRT Ambre | Monospace, amber/orange CRT glow (distinct from neon-terminal's green) |
| `mono-contrast` | Monochrome | Pure black/white, high-contrast, sharp |

All 12 confirmed in scope by the user ("je les veux tous").

### 3.1 Architecture: shared component primitives, not per-page CSS

**This is the actual fix for the "looks like a kid coded it" complaint**, more than the style catalog above. Today, `DisksPage.vue`, `SecurityPage.vue`, `NetworkPage.vue` etc. each define their own scoped `.xxx-panel`, `.xxx-input`, `.xxx-error`, `.xxx-success` classes with near-identical but subtly-inconsistent values (e.g. slightly different padding/radius across pages). A 12-style system layered on top of that inconsistency would just produce 12× the inconsistency.

Introduce `src/components/ui/`: a small set of shared presentational components used by **every** page across all 9 categories —

- `NxCard.vue` — the base panel/card container (replaces every page's `.xxx-panel`)
- `NxButton.vue` (variants: default, danger, ghost) — replaces every page's ad-hoc `<button>` styling
- `NxInput.vue` / `NxSelect.vue` — form controls
- `NxStatTile.vue` — labeled metric display (CPU %, disk usage, etc.)
- `NxBadge.vue` — status pill (success/warning/danger/info), replacing the various `.xxx-success`/`.xxx-error` divs
- `NxSectionHeader.vue` — category/section title with optional description

Each of these components reads its visual character from CSS custom properties set by the active style (`--nx-style-*`, e.g. `--nx-style-radius`, `--nx-style-blur`, `--nx-style-shadow`, `--nx-style-border-width`) plus the existing palette variables (`--nx-accent-primary`, `--nx-bg-elevated`, etc.) for color. The **style** axis is implemented as a `data-nx-style="<id>"` attribute on the root element with one stylesheet per style defining that style's custom-property values and any structural CSS (e.g. `cyber-grid`'s `clip-path` corners, `neon-terminal`/`amber-crt`'s `font-family: monospace`) — mirroring how `themeStore`/`layoutStore` already work (an id-keyed store driving a data attribute or component swap), so this is a natural extension of the existing pattern, not a new paradigm.

**Migration approach:** every *new* page (§4) is built on `NxCard`/`NxButton`/etc. from the start. Every *existing* page (§5) is migrated to use them as part of its restructuring work — this is not a separate "refactor pass," it's how the restructuring gets done, since splitting `DisksPage`'s 4 crammed tabs into proper category pages is the same work as rebuilding them on shared primitives.

### 3.2 Live preview

`ThemeEditorPage.vue` (moved to Paramètres > Thèmes & dispositions) becomes the control surface for all 3 axes: palette picker (existing 12), layout picker (existing 8), **style picker (new 12)** — each applies live via its store (no reload), exactly like the existing palette/layout live-switching already does. This satisfies the explicit request for "une fonction d'aperçu en temps réel du changement comme sur NiTriTe."

## 4. New features (previously absent from NiTruX entirely)

### 4.1 Applications > Installation rapide

A curated catalog (`src/data/appCatalog.ts`) of popular Linux applications (browsers, editors, communication, gaming, media — mirroring the spirit of NiTriTe's Master Install page), each entry: `{ id, name, description, icon, installMethod: "apt" | "flatpak" | "snap", packageId }`.

**v1 scope decision:** only catalog entries whose `installMethod` is `"apt"` (or the detected system's native manager — `dnf`/`pacman`/`zypper`, reusing the exact `install_package` Tauri command already built, tested, and **live-verified in the disposable VM** during Phase 2 Part 2) are installable in this pass. Flatpak/Snap entries are shown in the catalog (useful as a browsable reference) but their install button is disabled with a "bientôt disponible" note. **Rationale:** `install_package` already has a dedicated polkit action, a validated shell wrapper, and live VM proof it works — reusing it needs zero new privileged surface. Flatpak (`flatpak install --user`, no root needed) and Snap (`snap install`, needs root via snapd's own polkit integration) are a **separate, later privileged-surface decision** deliberately deferred rather than added under an unattended overnight pass — consistent with the project's established discipline of never adding new privileged operations without explicit review and live verification.

UI: grid of app cards (`NxCard`), each with an install button that shows a real progress state (indeterminate spinner while `install_package` runs — the underlying `apt-get`/`dnf` process doesn't expose parseable percentage progress over stdout in the current implementation, so "barre de chargement" is delivered as an animated indeterminate progress bar, not a false precise percentage) and a success/error state on completion, matching the "silencieux" request (no interactive terminal prompts — `install_package` already runs fully non-interactively).

### 4.2 Maintenance > Mises à jour

New dedicated page surfacing the existing `list_updates` Tauri command (currently only reachable, if at all, through `PackagesPage`'s tab clutter) with a clear list of upgradable packages and a prominent "Tout mettre à jour" button calling the existing, already-VM-verified `upgrade_all_packages` command. No new backend work — this is a frontend-only promotion of existing functionality to its own proper page.

### 4.3 Maintenance > Pilotes (enriched)

The existing `DriversPage.vue`/`drivers.rs` backend is extended (if needed, based on what `get_driver_snapshot` currently returns) to show, per device: driver name/version, whether a newer version might be available (best-effort — Linux driver "updates" are usually kernel/DKMS-module updates flowing through the normal package manager, not a separate mechanism like Windows Update — the page should say so honestly rather than fake a Windows-style "driver update" concept that doesn't exist on Linux). If the current backend already has everything needed, this page is a frontend-only richness pass (using `NxCard`/`NxStatTile` instead of a flat list).

### 4.4 Rapports > Générateur de rapport

New Tauri command `generate_system_report(format: "html" | "markdown" | "txt" | "json") -> String` (file path or content) aggregating existing read-only snapshot commands already built across every phase tonight (`get_system_snapshot`, `get_sensor_snapshot`, `get_pci_devices`, `get_driver_snapshot`, `list_disks`, `list_disk_usage`, `get_network_snapshot`, `get_firewall_status`, `list_updates`) into one structured report. Four renderers (one per format) share a common intermediate data structure (a `SystemReport` struct built once from the aggregated snapshots) — implementing a JSON `Serialize` derive gets that format almost for free; HTML/Markdown/TXT are template renderers over the same struct. **No new privileged operations** — every underlying data source is an existing read-only command.

UI: a "Générer" button with format picker, writing to a user-chosen path (Tauri's file-save dialog) and a "voir le rapport" / "ouvrir le dossier" affordance after generation.

### 4.5 Paramètres > Préférences

A small, concrete set of app-level (not system-level) preferences persisted via the existing localStorage-backed store pattern (matching `themeStore`/`layoutStore`'s persistence approach) — not a vague placeholder page:

- **Répertoires par défaut** for the duplicate/large-file/hash-check scanners (§5) — currently every scan requires re-typing a path; a remembered default (e.g. `$HOME`) removes that friction.
- **Intervalle de rafraîchissement du tableau de bord** — how often `DashboardPage`'s live system stats re-poll (a numeric setting, e.g. 1s/2s/5s).
- **Confirmation pour actions non-destructives** — a toggle for whether actions like package install/upgrade show a lightweight confirm step before running (the destructive `format_partition` typed-confirmation gate from Phase 3 Part 2 is NOT affected by this toggle — that one stays mandatory regardless, it's not a preference).

This list is deliberately minimal for v1 — it exists to be a real, useful settings page rather than an empty one duplicating Thèmes & dispositions, and can grow later as concrete needs surface.

## 5. Restructuring existing pages

`DisksPage.vue` (currently: disks/duplicates/largefiles/hashcheck/format/extend/clone all in one flat 4-tab page) splits into:
- **Stockage > Disques & partitions**: disk/partition listing, usage, format/extend/clone controls (the format-confirmation-gate UX from Phase 3 Part 2 is preserved as-is — it's correct, not part of what's broken)
- **Stockage > Doublons / Gros fichiers / Hash**: the 3 file-analysis tools, on `NxCard`-based layout instead of flat tabs

`SecurityPage.vue` (currently: firewall/malware/snapshots/troubleshoot in one flat page) splits into:
- **Réseau > Pare-feu**: firewall status (moves out of Security entirely)
- **Maintenance > Dépannage**: malware scan, quarantine, snapshot creation/listing, the 4 curated troubleshoot actions

`NetworkPage.vue` keeps its core content (overview, port scanner, Docker) under **Réseau > Vue d'ensemble**; hosts/DNS/firewall-rule editing stays with it (firewall *rule editing* is a network config action, distinct from firewall *status display* which moves per above — on reflection during implementation this split may turn out to belong together on one Réseau page rather than two; the implementer should use judgment here and note the actual final shape in the plan).

`HardwarePage.vue` → **Système > Diagnostic** (rename only, content unchanged pending a later richness pass if time allows).

`PackagesPage.vue` → **Applications > Gestionnaire de paquets** (its existing install/upgrade UI stays here as the "raw package manager" view, distinct from the new curated §4.1 catalog).

`LogsPage.vue` → **Rapports > Journaux** (move only).

`ThemeEditorPage.vue` → **Paramètres > Thèmes & dispositions**, extended per §3.2.

Every moved/split page is rebuilt on the `NxCard`/`NxButton`/etc. primitives from §3.1 as part of the move — not moved as-is and polished later.

## 6. Testing

- Every new/restructured page keeps or gains component-level tests (Vitest) for its non-trivial logic (existing pattern: `LayoutShell.spec.ts`, `registry.spec.ts`, store specs) — new stores (style axis) get the same test treatment as `themeStore.spec.ts`/`layoutStore.spec.ts`.
- `generate_system_report`'s 4 renderers get Rust unit tests against a fixed sample `SystemReport` (golden-output style assertions — does the HTML contain expected sections, does the JSON round-trip, etc.), following the project's established TDD discipline.
- `appCatalog.ts`'s data is plain, testable — validate every entry against the allowed `installMethod` enum and non-empty required fields.
- `vue-tsc --noEmit` and the full existing test suite must stay green throughout, checked after each implementation phase (established discipline).
- Actual visual verification: `npx playwright` on this machine is currently missing system shared libraries (`libnspr4`) that require an interactive `sudo apt-get install` this session cannot run non-interactively. Visual verification during implementation will rely on careful reading of rendered template structure, CSS custom property resolution, and store logic tests — **not** pixel screenshots, unless the user fixes the missing system libs (`sudo apt install libnspr4 libnss3 ...` in WSL2) themselves at some point, after which real screenshot-based verification becomes possible and should be adopted.

## 7. Out of scope for this pass

- Flatpak/Snap installation (catalog entries shown, disabled) — deferred, needs its own privileged-surface review.
- Driver *installation/update* actions (only richer *display* of current driver/device info — Linux doesn't have a Windows-Update-style separate driver-update mechanism to hook into).
- Vue-router / URL-based navigation.
- Any change to the already-shipped, VM-verified privileged pkexec operations from Phases 2–5 Part 2 — this redesign is a presentation-layer and information-architecture pass, not a backend rewrite. Existing backend commands are reused, not modified, except where §4.4's report generator needs to *call* them (read-only) and where §4.3 might need a small, additive extension to `drivers.rs` if its current data is insufficient for the richer display.

## 8. Implementation approach

Given the scope (comparable to or larger than the whole previous night's 4-pillar buildout), implementation proceeds in phases, each in its own git worktree, verified (tests + `vue-tsc`) before merging to `master`, following the exact discipline established tonight:

1. **Phase R1 — Foundation**: `categories.ts`, `AppNav.vue`, the 3rd visual-style axis (store + 12 stylesheets), the `NxCard`/`NxButton`/`NxInput`/`NxStatTile`/`NxBadge`/`NxSectionHeader` component library, `ThemeEditorPage` extended to 3 pickers. No page content moves yet — this phase makes the new nav/visual system real and testable in isolation.
2. **Phase R2 — Restructure existing pages**: split/move/rebuild `DisksPage`, `SecurityPage`, `NetworkPage`, `HardwarePage`→Diagnostic, `PackagesPage`, `LogsPage`, `ThemeEditorPage` onto the new nav + shared primitives per §5.
3. **Phase R3 — Applications > Installation rapide** (§4.1).
4. **Phase R4 — Maintenance > Mises à jour + Pilotes enrichment** (§4.2, §4.3).
5. **Phase R5 — Rapports > Générateur de rapport** (§4.4).

Each phase ships a version bump + release, matching tonight's established release cadence, since each phase is independently shippable and valuable.
