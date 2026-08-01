# Checkpoint — Redesign (R1–R5), session démarrée 2026-08-01

## État antérieur (terminé, référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, mergés, publiés jusqu'à v0.8.0** (voir git log/releases GitHub). Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. Ce travail n'est PAS remis en cause par la refonte — voir spec §7 "Out of scope" : aucune modification des commandes backend déjà livrées, seulement réutilisation.

## Nouveau travail en cours

Spec écrite et committée : `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` (commit `69b2bf2`). Utilisateur a validé la structure de nav (9 catégories, devenues 7 après fusion Paramètres), le système visuel 3 axes (12 palettes × 8 dispositions × 12 styles), et les 4 nouvelles fonctionnalités via le compagnon visuel, puis a dit "oui fait tout je vais me coucher" — autorisation complète, pas d'attente de revue supplémentaire de la spec.

**Phase R1 (Foundation) : TERMINÉE, mergée, publiée en v0.9.0-r1-foundation.** 9 tâches (registre de styles, styleStore, style-tokens.css, 7 composants Nx*, categories.ts, AppNav.vue, extension ThemeEditorPage) — toutes vérifiées indépendamment. 79 tests frontend + 124 Rust.

**Phase R2 (Restructure) : TERMINÉE, mergée dans master, publiée en v0.10.0-r2-restructure.** 9 tâches exécutées (DiagnosticPage, PackagesPage, LogsPage, split DisksPage/FileToolsPage, split FirewallPage/TroubleshootPage, NetworkPage, preferencesStore+SettingsPreferencesPage, branchement final App.vue→AppNav+15 ids categories.ts, vérification finale) — chacune vérifiée indépendamment par moi (jamais confiance aveugle dans un rapport de sous-agent), y compris Task 6 (NetworkPage) où j'ai terminé moi-même la vérification après une coupure de quota du sous-agent implémenteur en cours de tâche, le travail partiel étant vérifié correct et complet avant d'être committé. Progression des tests : 79→81→84→85→90→95→97→106→111 (frontend), 124 Rust inchangé (cette phase ne touche aucun code Rust).

`HardwarePage.vue` et `SecurityPage.vue` supprimés (`git rm`), remplacés par `DiagnosticPage.vue` et `FirewallPage.vue`+`TroubleshootPage.vue`. App.vue branché sur `AppNav.vue` (R1) + `categories.ts` (7 catégories, 15 pages) ; 3 pages "Bientôt disponible" (`QuickInstallPlaceholder`/`UpdatesPlaceholder`/`ReportGeneratorPlaceholder`) couvrent les ids que R3/R4/R5 implémenteront réellement, chacune destinée à être remplacée intégralement (pas étendue) par la phase correspondante.

Merge master + version bump 0.9.0→0.10.0 (package.json/Cargo.toml/tauri.conf.json) + build réel (.deb/.rpm, AppImage toujours bloqué xdg-open comme d'habitude) + release GitHub `v0.10.0-r2-restructure` publiés dans cette session.

## Prochaine action

Écrire et exécuter le plan Phase R3 (Applications > Installation rapide — spec §4.1 : catalogue d'apps, installation en un clic silencieuse avec barre de progression, réutilise la commande `install_package` déjà livrée en Phase 2 Part 2). Remplace entièrement `QuickInstallPlaceholder.vue`. Même discipline : writing-plans → subagent-driven-development → vérification indépendante systématique → merge/version bump (0.10.0→0.11.0 attendu)/release.

Puis Phase R4 (Maintenance > Mises à jour dédiée + Pilotes enrichis, remplace `UpdatesPlaceholder.vue`), puis Phase R5 (Rapports > Générateur HTML/MD/TXT/JSON, remplace `ReportGeneratorPlaceholder.vue`).

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.10.0` (R2 publiée)
- Worktree `r2-restructure` (`.worktrees/r2-restructure`) peut être nettoyé (`git worktree remove`) — R2 est mergée et publiée, plus besoin.
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS — juste ce qui est injecté dans le slot `#nav`/défaut change
- themeStore/layoutStore/styleStore : 3 axes visuels indépendants, pattern à réutiliser si un jour un 4ème axe est ajouté
- Playwright headless bloqué (libnspr4 manquant, sudo interactif indisponible) — vérification visuelle par lecture de code/tests, pas par screenshot, jusqu'à ce que l'utilisateur installe les libs manquantes lui-même
- Référence Windows : `C:\Users\Momo\Desktop\Nitrite 2.0\src\` (navigation.ts, AppSidebar.vue, MasterInstallPage.vue, StatsReportsPage.vue, DriversPage.vue, UpdatesPage.vue) — consulter pour inspiration de contenu/structure, jamais copier le code tel quel
- Build release : `npx tauri build` en WSL2 depuis la racine du repo (pas le worktree) — produit `.deb`+`.rpm` dans `src-tauri/target/release/bundle/`, régénère `Cargo.lock` (à committer après), AppImage échoue systématiquement (xdg-open manquant, non bloquant, ignoré comme pour toutes les releases précédentes)
