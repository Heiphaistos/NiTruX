# Checkpoint — Redesign (R1–R5), session démarrée 2026-08-01

## État antérieur (terminé, référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, mergés, publiés jusqu'à v0.8.0** (voir git log/releases GitHub). Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. Ce travail n'est PAS remis en cause par la refonte — voir spec §7 "Out of scope" : aucune modification des commandes backend déjà livrées, seulement réutilisation.

## Nouveau travail en cours

Spec écrite et committée : `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` (commit `69b2bf2`). Utilisateur a validé la structure de nav (9 catégories, devenues 7 après fusion Paramètres), le système visuel 3 axes (12 palettes × 8 dispositions × 12 styles), et les 4 nouvelles fonctionnalités via le compagnon visuel, puis a dit "oui fait tout je vais me coucher" — autorisation complète, pas d'attente de revue supplémentaire de la spec.

**Phase R1 (Foundation) : TERMINÉE, mergée, publiée en v0.9.0-r1-foundation.** 9 tâches (registre de styles, styleStore, style-tokens.css, 7 composants Nx*, categories.ts, AppNav.vue, extension ThemeEditorPage) — toutes vérifiées indépendamment. 79 tests frontend + 124 Rust.

**Phase R2 (Restructure) : TERMINÉE, mergée dans master, publiée en v0.10.0-r2-restructure.** 9 tâches (DiagnosticPage, PackagesPage, LogsPage, split DisksPage/FileToolsPage, split FirewallPage/TroubleshootPage, NetworkPage, preferencesStore+SettingsPreferencesPage, branchement final App.vue→AppNav+15 ids categories.ts, vérification finale) — toutes vérifiées indépendamment. Tests : 79→111 (frontend), 124 Rust inchangé.

**Phase R3 (Applications > Installation rapide) : TERMINÉE, mergée dans master, publiée en v0.11.0.** Spec §4.1. 5 tâches exécutées et vérifiées indépendamment :
- Task 1 : commande Tauri `detect_native_manager` (wrapper read-only sur `packages::detect_package_managers()` déjà testé) — 124 Rust inchangé (aucun test dédié, même précédent que `list_updates`).
- Task 2 : `src/data/appCatalog.ts` (16 apps curées, catégories Navigateurs/Communication/Bureautique/Média/Jeux/Utilitaires/Développement) + 3 tests.
- Task 3 : `QuickInstallPage.vue` (grille de cartes NxCard, filtre par catégorie, installation apt réelle via `install_package` déjà VM-vérifiée, barre de progression indéterminée animée, flatpak/snap affichés mais bouton désactivé "Bientôt disponible") + 5 tests. **Bug réel trouvé et corrigé par le sous-agent implémenteur** : le code exemple du plan gatait le bouton d'installation sur `!nativeManager`, créant une race condition (le clic de test arrivait avant que Vue ne flush le DOM patch retirant `disabled`, donc le clic tombait sur un bouton encore désactivé et `install()` n'était jamais appelé). Corrigé en stockant la promesse de détection (`managerReady`) et en l'attendant dans `install()` au lieu de gater le bouton dessus — vérifié indépendamment, comportement spec préservé.
- Task 4 : branchement `App.vue`/`App.spec.ts` sur la vraie `QuickInstallPage` (au lieu du placeholder), suppression de `QuickInstallPlaceholder.vue` — fait directement par moi (tâche assez petite et précisément spécifiée pour ne pas justifier un sous-agent dédié), TDD respecté (test modifié confirmé en échec avant le swap, en succès après).
- Task 5 : vérification finale — 119/119 frontend (111 R2 + 3 catalogue + 5 QuickInstallPage), `vue-tsc` clean, 124 Rust inchangé, `QuickInstallPlaceholder` confirmé absent du code.

Une interruption de quota a eu lieu au tout début de R3 (sous-agent Task 1 coupé avant tout changement de fichier) — reprise propre après reconnexion utilisateur ("continu"), rien à récupérer (worktree resté vierge), Task 1 relancée depuis zéro.

Merge master (fast-forward propre) + version bump 0.10.0→0.11.0 (package.json/Cargo.toml/tauri.conf.json) + build réel (.deb/.rpm, AppImage toujours bloqué xdg-open) + tag+release GitHub `v0.11.0` — en cours de finalisation dans cette session (commit Cargo.lock fait, push+tag+release restent à faire juste après ce checkpoint).

## Prochaine action

Pousser master + créer le tag `v0.11.0` + créer la release GitHub avec les assets `.deb`/`.rpm` construits. Nettoyer le worktree `r3-quick-install` (mergé, plus besoin).

Puis écrire et exécuter le plan Phase R4 (Maintenance > Mises à jour dédiée + Pilotes enrichis — spec §4.2/§4.3). Remplace `UpdatesPlaceholder.vue` ; §4.2 est frontend-only (promotion de `list_updates`/`upgrade_all_packages` déjà existants et VM-vérifiés vers une page dédiée, zéro nouveau backend) ; §4.3 dépend de ce que `get_driver_snapshot` retourne déjà — vérifier avant de décider si un enrichissement backend est nécessaire ou si c'est aussi frontend-only.

Puis Phase R5 (Rapports > Générateur HTML/MD/TXT/JSON, remplace `ReportGeneratorPlaceholder.vue` — spec §4.4, nouvelle commande Tauri `generate_system_report` agrégeant des snapshots déjà tous en lecture seule, zéro nouvelle surface privilégiée).

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.11.0` (R3 en cours de publication)
- Worktrees mergés à nettoyer si encore présents : `r3-quick-install` (`.worktrees/r3-quick-install`)
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS — juste ce qui est injecté dans le slot `#nav`/défaut change
- themeStore/layoutStore/styleStore : 3 axes visuels indépendants, pattern à réutiliser si un jour un 4ème axe est ajouté
- Playwright headless bloqué (libnspr4 manquant, sudo interactif indisponible) — vérification visuelle par lecture de code/tests, pas par screenshot, jusqu'à ce que l'utilisateur installe les libs manquantes lui-même
- Référence Windows : `C:\Users\Momo\Desktop\Nitrite 2.0\src\` (navigation.ts, AppSidebar.vue, MasterInstallPage.vue, StatsReportsPage.vue, DriversPage.vue, UpdatesPage.vue) — consulter pour inspiration de contenu/structure, jamais copier le code tel quel
- Build release : `npx tauri build` en WSL2 depuis la racine du repo (pas le worktree) — produit `.deb`+`.rpm` dans `src-tauri/target/release/bundle/`, régénère `Cargo.lock` (à committer après), AppImage échoue systématiquement (xdg-open manquant, non bloquant, ignoré comme pour toutes les releases précédentes)
- Piège race condition Vue à retenir pour les prochaines pages avec détection asynchrone au montage + bouton conditionnellement désactivé : ne jamais gater un bouton sur un ref rempli par une promesse async démarrée dans `onMounted` si un clic peut arriver avant que le DOM patch soit flush — préférer stocker la promesse et l'attendre dans le handler du clic lui-même.
