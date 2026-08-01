# Checkpoint — Redesign (R1–R5) : TERMINÉE ✅, session démarrée 2026-08-01

## État antérieur (référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, publiés jusqu'à v0.8.0**. Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. Ce travail n'a jamais été remis en cause par la refonte — spec §7 "Out of scope" : aucune modification des commandes backend déjà livrées, seulement réutilisation.

## Refonte complète — TOUTES LES PHASES TERMINÉES ET PUBLIÉES

Spec écrite et validée par l'utilisateur : `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` (commit `69b2bf2`). Structure de nav (7 catégories), système visuel 3 axes (12 palettes × 8 dispositions × 12 styles), et 4 nouvelles fonctionnalités (Applications, Mises à jour, Pilotes enrichis, Rapports) — toutes implémentées, vérifiées indépendamment à chaque tâche, mergées et publiées.

- **Phase R1 (Foundation)** — v0.9.0-r1-foundation. Registre de styles, styleStore, 7 composants Nx*, categories.ts, AppNav.vue. 79 tests frontend + 124 Rust.
- **Phase R2 (Restructure)** — v0.10.0-r2-restructure. Toutes les pages existantes componentisées sur Nx*, DisksPage/SecurityPage scindées, branchement App.vue→AppNav. Tests : 79→111 frontend, 124 Rust.
- **Phase R3 (Applications > Installation rapide)** — v0.11.0. Catalogue de 16 apps, détection auto du gestionnaire, install en un clic via `install_package` déjà VM-vérifiée. Tests : 111→119 frontend, 124 Rust.
- **Phase R4 (Maintenance > Mises à jour + Pilotes enrichis)** — v0.12.0. UpdatesPage dédiée, mapping pilote-par-périphérique via `lspci -k`, note honnête sur le modèle Linux. Tests : 119→124 frontend, 124→128 Rust.
- **Phase R5 (Rapports > Générateur de rapport)** — v0.13.0, **dernière phase**. `SystemReport` agrégeant 9 sources déjà en lecture seule (système/capteurs/PCI/pilotes/disques/réseau/pare-feu/MAJ), 4 formats de rendu (JSON/Markdown/TXT/HTML), export via téléchargement client-side (Blob, pas de nouvelle dépendance Tauri). `ComingSoonPage.vue` et son spec supprimés (plus aucune référence, c'était le dernier utilisateur). Tests : 124→128 frontend (net, +5 -1 car ComingSoonPage.spec.ts supprimé), 128→132 Rust.

**Résultat final vérifié** : `grep -rln 'Placeholder' src/pages/` → **aucun fichier placeholder ne subsiste**. Les 15 ids de `categories.ts` pointent tous vers une vraie page implémentée. 132 tests Rust, 128 tests frontend, `vue-tsc` clean sur `master`.

Merge master (fast-forward propre à chaque phase) + version bump 0.9.0→0.13.0 progressif + build réel (.deb/.rpm) à chaque étape + 5 tags/releases GitHub publiés (v0.9.0 à v0.13.0) — le dernier commit de version bump et Cargo.lock pour v0.13.0 vient d'être fait, **push+tag+release restent à faire juste après ce checkpoint**.

## Prochaine action

Pousser master + créer le tag `v0.13.0` + créer la release GitHub finale avec les assets `.deb`/`.rpm`. Nettoyer le worktree `r5-report-generator` (mergé, plus besoin).

**La refonte NiTruX (R1-R5) sera alors intégralement terminée.** Plus aucune tâche en attente sur ce chantier. Prochaines étapes possibles côté produit (non planifiées, à discuter avec l'utilisateur au réveil) : Flatpak/Snap install (privilégié, décision différée volontairement en R3, nécessite conception+revue humaine dédiée comme toute nouvelle surface pkexec) ; enrichissement pilotes avec version/date (actuellement seulement nom du module noyau) ; PDF comme 5ème format de rapport ; historique des rapports générés.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.13.0`
- Worktrees mergés à nettoyer si encore présents : `r5-report-generator` (`.worktrees/r5-report-generator`)
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS — juste ce qui est injecté dans le slot `#nav`/défaut change
- themeStore/layoutStore/styleStore : 3 axes visuels indépendants
- Playwright headless bloqué (libnspr4 manquant, sudo interactif indisponible) — vérification visuelle par lecture de code/tests, pas par screenshot, jusqu'à ce que l'utilisateur installe les libs manquantes lui-même
- Référence Windows : `C:\Users\Momo\Desktop\Nitrite 2.0\src\` — consulter pour inspiration de contenu/structure, jamais copier le code tel quel
- Build release : `npx tauri build` en WSL2 depuis la racine du repo (pas un worktree) — produit `.deb`+`.rpm` dans `src-tauri/target/release/bundle/`, régénère `Cargo.lock` (à committer après), AppImage échoue systématiquement (xdg-open manquant, non bloquant)
- **Piège privacy Rust découvert en R5** : un item `fn` privé défini au niveau racine du crate (`lib.rs`) est automatiquement visible depuis tous les modules descendants (`crate::ma_fonction()` fonctionne sans changer sa visibilité) — inutile de la passer en `pub(crate)`. À l'inverse, **passer une fonction `#[tauri::command]` définie à la racine du crate en `pub`/`pub(crate)` casse la compilation** (collision de macro `__cmd__<nom>` dans tauri-macros 2.6.3) — ne jamais changer la visibilité d'un `#[tauri::command]` racine sans le vérifier d'abord par un essai isolé.
- Piège race condition Vue (R3) : ne jamais gater un bouton sur un ref rempli par une promesse async démarrée dans `onMounted` si un clic peut arriver avant que le DOM patch soit flush — stocker la promesse et l'attendre dans le handler du clic lui-même.
- Pattern "unhandled rejection bénin dans App.spec.ts" (vu en R2/R4/R5) : toute page qui fait `ref.value = await invoke<T[]>(...)` puis accède à `.length` sans garde null lèvera une erreur non bloquante dans `App.spec.ts` (mock global `invoke` résout `null`) — n'affecte AUCUN résultat de test, ne pas "corriger" le composant pour ce cas synthétique qui ne se produit jamais en usage réel.
