# Checkpoint — Redesign (R1–R5), session démarrée 2026-08-01

## État antérieur (terminé, référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, mergés, publiés jusqu'à v0.8.0** (voir git log/releases GitHub). Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. Ce travail n'est PAS remis en cause par la refonte — voir spec §7 "Out of scope" : aucune modification des commandes backend déjà livrées, seulement réutilisation.

## Nouveau travail en cours

Spec écrite et committée : `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` (commit `69b2bf2`). Utilisateur a validé la structure de nav (9 catégories, devenues 7 après fusion Paramètres), le système visuel 3 axes (12 palettes × 8 dispositions × 12 styles), et les 4 nouvelles fonctionnalités via le compagnon visuel, puis a dit "oui fait tout je vais me coucher" — autorisation complète, pas d'attente de revue supplémentaire de la spec.

**Phase R1 (Foundation) : TERMINÉE, publiée en v0.9.0-r1-foundation.** 9 tâches (registre de styles, styleStore, style-tokens.css, 7 composants Nx*, categories.ts, AppNav.vue, extension ThemeEditorPage). 79 tests frontend + 124 Rust.

**Phase R2 (Restructure) : TERMINÉE, publiée en v0.10.0-r2-restructure.** 9 tâches (DiagnosticPage, PackagesPage, LogsPage, split DisksPage/FileToolsPage, split FirewallPage/TroubleshootPage, NetworkPage, preferencesStore+SettingsPreferencesPage, branchement App.vue→AppNav+15 ids categories.ts). Tests : 79→111 (frontend), 124 Rust inchangé.

**Phase R3 (Applications > Installation rapide) : TERMINÉE, publiée en v0.11.0.** Spec §4.1. Catalogue de 16 apps (`appCatalog.ts`), `QuickInstallPage.vue` (détection auto du gestionnaire via nouvelle commande `detect_native_manager`, install réel via `install_package` déjà VM-vérifiée, flatpak/snap affichés mais désactivés). Bug réel trouvé et corrigé par le sous-agent (race condition bouton/promesse async — pattern à retenir, voir ci-dessous). Tests : 111→119 frontend, 124 Rust inchangé.

**Phase R4 (Maintenance > Mises à jour + Pilotes enrichis) : TERMINÉE, mergée dans master, publiée en v0.12.0.** Spec §4.2/§4.3. 5 tâches exécutées et vérifiées indépendamment :
- Task 1 : `drivers.rs` — nouveau `DeviceDriver`/`parse_lspci_k_output` (parsing `lspci -k`, associe chaque périphérique PCI à son pilote noyau réellement chargé), `DriverSnapshot` étendu avec `devices: Vec<DeviceDriver>`, dégradation gracieuse (`unwrap_or_default`) si `pciutils` absent — 124→128 Rust.
- Task 2 : `DriversPage.vue` — passe de richesse complète sur composants Nx* (NxCard/NxStatTile/NxBadge), tableau par périphérique, note honnête sur le fait que Linux n'a pas de mécanisme séparé de "mise à jour de pilote" (contrairement à Windows) — les pilotes suivent le gestionnaire de paquets.
- Task 3 : `UpdatesPage.vue` — nouvelle page dédiée réutilisant `list_updates`/`upgrade_all_packages` déjà existants et VM-vérifiés (zéro nouveau backend), `PackagesPage.vue` volontairement non touché (duplication UI intentionnelle, même paire de commandes appelée indépendamment par les deux pages).
- Task 4 : branchement `App.vue`/`App.spec.ts` sur la vraie `UpdatesPage`, suppression de `UpdatesPlaceholder.vue` — fait directement par moi (tâche petite et précise), TDD respecté.
- Task 5 : vérification finale — 124/124 frontend (119 R3 + 1 DriversPage + 3 UpdatesPage + 1 App), `vue-tsc` clean, 128 Rust inchangé, `UpdatesPlaceholder` confirmé absent.

Merge master (fast-forward propre) + version bump 0.11.0→0.12.0 (package.json/Cargo.toml/tauri.conf.json) + build réel (.deb/.rpm, AppImage toujours bloqué xdg-open) + Cargo.lock committé — push+tag+release restent à faire juste après ce checkpoint.

## Prochaine action

Pousser master + créer le tag `v0.12.0` + créer la release GitHub avec les assets `.deb`/`.rpm` construits. Nettoyer le worktree `r4-updates-drivers` (mergé, plus besoin).

Puis écrire et exécuter le plan Phase R5 (Rapports > Générateur HTML/MD/TXT/JSON — spec §4.4, dernière phase de la refonte). Remplace `ReportGeneratorPlaceholder.vue`. Nouvelle commande Tauri `generate_system_report(format: "html"|"markdown"|"txt"|"json") -> String` agrégeant les snapshots déjà tous en lecture seule (`get_system_snapshot`, `get_sensor_snapshot`, `get_pci_devices`, `get_driver_snapshot`, `list_disks`, `list_disk_usage`, `get_network_snapshot`, `get_firewall_status`, `list_updates`) — zéro nouvelle surface privilégiée. Une structure `SystemReport` intermédiaire commune, 4 renderers (JSON quasi gratuit via `Serialize`, HTML/Markdown/TXT via templates). UI : bouton "Générer" + sélecteur de format + dialogue de sauvegarde fichier Tauri.

Après R5 : la refonte complète (R1-R5) sera terminée — toutes les catégories du spec seront réelles, plus aucun placeholder "Bientôt disponible" dans `categories.ts`.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.12.0` (R4 en cours de publication)
- Worktrees mergés à nettoyer si encore présents : `r4-updates-drivers` (`.worktrees/r4-updates-drivers`)
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS — juste ce qui est injecté dans le slot `#nav`/défaut change
- themeStore/layoutStore/styleStore : 3 axes visuels indépendants, pattern à réutiliser si un jour un 4ème axe est ajouté
- Playwright headless bloqué (libnspr4 manquant, sudo interactif indisponible) — vérification visuelle par lecture de code/tests, pas par screenshot, jusqu'à ce que l'utilisateur installe les libs manquantes lui-même
- Référence Windows : `C:\Users\Momo\Desktop\Nitrite 2.0\src\` (navigation.ts, AppSidebar.vue, MasterInstallPage.vue, StatsReportsPage.vue, DriversPage.vue, UpdatesPage.vue, StatsReportsPage.vue pour R5) — consulter pour inspiration de contenu/structure, jamais copier le code tel quel
- Build release : `npx tauri build` en WSL2 depuis la racine du repo (pas le worktree) — produit `.deb`+`.rpm` dans `src-tauri/target/release/bundle/`, régénère `Cargo.lock` (à committer après), AppImage échoue systématiquement (xdg-open manquant, non bloquant, ignoré comme pour toutes les releases précédentes)
- Piège race condition Vue à retenir pour les prochaines pages avec détection asynchrone au montage + bouton conditionnellement désactivé : ne jamais gater un bouton sur un ref rempli par une promesse async démarrée dans `onMounted` si un clic peut arriver avant que le DOM patch soit flush — préférer stocker la promesse et l'attendre dans le handler du clic lui-même (confirmé nécessaire dans R3 Task 3, absent dans R4 car aucune page R4 n'a ce pattern détection-async+bouton).
- Pattern "unhandled rejection bénin dans App.spec.ts" : toute page qui fait `ref.value = await invoke<T[]>(...)` puis accède à `.length` sans garde null lèvera une erreur non bloquante dans `App.spec.ts` (dont le mock global `invoke` résout `null` pour toutes les commandes) — n'affecte AUCUN résultat de test (déjà vu avec DiagnosticPage en R2, confirmé à nouveau avec UpdatesPage en R4), ne pas "corriger" le composant pour ce cas synthétique qui ne se produit jamais en usage réel.
