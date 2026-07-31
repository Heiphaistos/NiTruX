# Goal: NiTruX Redesign (R1–R5)

(Previous goal — Phase 1-4/original 4 pillars — is complete: shipped through v0.8.0, all merged/released. This file now tracks the NEW redesign effort.)

Implement the full redesign per `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md`, authorized end-to-end by the user ("fait tout je vais me coucher" — full autonomy, no further check-ins required until done or genuinely blocked).

## Verifiable completion criteria (per phase, all must hold before merging that phase)
- `cargo test` (src-tauri) — full suite green, 0 new warnings
- `npm run test -- --run` — full suite green
- `npx vue-tsc --noEmit` — clean
- Independent re-verification of each subagent's work (read the actual diff, don't just trust the report) — same discipline as Phases 2–5 Part 2 tonight
- Phase committed, merged to master, version bumped, tests re-run on master post-merge

## Scope (5 phases, see spec §8)
- R1 — Foundation: categories.ts, AppNav.vue, 3rd visual-style axis (12 styles), NxCard/NxButton/NxInput/NxStatTile/NxBadge/NxSectionHeader component library, ThemeEditorPage extended to 3 pickers
- R2 — Restructure existing pages onto new nav + shared primitives (Disks split, Security split, Network, Hardware→Diagnostic, Packages, Logs, ThemeEditor moves)
- R3 — Applications > Installation rapide (curated app catalog, apt-only installable in v1)
- R4 — Maintenance > Mises à jour + Pilotes enrichment
- R5 — Rapports > Générateur de rapport (HTML/MD/TXT/JSON)

## Explicit out of scope (do NOT do these without stopping to ask)
- Flatpak/Snap install (new privileged surface, deferred per spec §7)
- Any change to already-shipped/VM-verified pkexec commands from Phases 2–5 Part 2 (reuse only, don't modify)
- Driver install/update actions (display-only enrichment)
- vue-router

## Bornes

max_iterations: none imposé (session continue jusqu'à R5 fini ou blocage réel)
stop_si_pas_de_progres: 3 tâches consécutives sans progrès mesurable

## Method
- Same subagent-driven-development pattern validated across Phases 1-5 Part 2 tonight: writing-plans per phase → dispatch implementer subagent per task → independent re-verification by coordinator (never trust the subagent report alone) → commit → next task.
- No privileged/destructive live-VM testing needed for R1–R5 — frontend/presentation-layer redesign, backend commands reused as-is.
- Real verification only: tests + vue-tsc + reading the actual diff before marking any task done in CHECKPOINT.md.
