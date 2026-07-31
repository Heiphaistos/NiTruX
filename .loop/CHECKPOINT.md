# Checkpoint — Redesign (R1–R5), session démarrée 2026-08-01

## État antérieur (terminé, référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, mergés, publiés jusqu'à v0.8.0** (voir git log/releases GitHub). Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. Ce travail n'est PAS remis en cause par la refonte — voir spec §7 "Out of scope" : aucune modification des commandes backend déjà livrées, seulement réutilisation.

## Nouveau travail en cours

Spec écrite et committée : `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` (commit `69b2bf2`). Utilisateur a validé la structure de nav (9 catégories, devenues 7 après fusion Paramètres), le système visuel 3 axes (12 palettes × 8 dispositions × 12 styles), et les 4 nouvelles fonctionnalités via le compagnon visuel, puis a dit "oui fait tout je vais me coucher" — autorisation complète, pas d'attente de revue supplémentaire de la spec.

**Phase R1 (Foundation) : TERMINÉE, mergée, publiée en v0.9.0-r1-foundation.** 9 tâches (registre de styles, styleStore, style-tokens.css, 7 composants Nx*, categories.ts, AppNav.vue, extension ThemeEditorPage) — toutes vérifiées indépendamment (jamais confiance aveugle dans un rapport de sous-agent). 79 tests frontend + 124 Rust (inchangé), diff vérifié strictement scopé via `git diff master --stat` avant merge (aucune page existante déplacée). Le nouveau sélecteur de style (12 styles) est déjà utilisable dès maintenant via Apparence > onglet Style — c'est le seul changement visible de R1, tout le reste (AppNav, categories.ts, composants Nx*) est fondation invisible tant que R2 ne les branche pas.

**Phase R2 (Restructure) : plan écrit et committé** (`docs/superpowers/plans/2026-08-01-nitrux-r2-restructure.md`, commit `4268f19`, 9 tâches, self-review passée, comptages de tests vérifiés cohérents 79→111). Worktree `r2-restructure` créé, baseline 79/79 reconfirmée dedans. **Task 1 (DiagnosticPage) en cours d'exécution** (sous-agent lancé, résultat attendu au prochain réveil).

Découpage des 9 tâches R2 : Task 1 DiagnosticPage (renomme HardwarePage) · Task 2 PackagesPage componentisée en place · Task 3 LogsPage componentisée en place · Task 4 split DisksPage → DisksPage + FileToolsPage · Task 5 split SecurityPage → FirewallPage + TroubleshootPage · Task 6 NetworkPage componentisée en place · Task 7 preferencesStore + SettingsPreferencesPage (nouveau, spec §4.5) · Task 8 branchement final App.vue sur AppNav + les 15 ids de categories.ts (avec 3 pages "Bientôt disponible" pour quick-install/updates/report-generator, remplacées entièrement par R3/R4/R5) · Task 9 vérification finale.

Chaque tâche 1-7 fait AUSSI un petit ajustement mécanique de App.vue (import+entrée de map, parfois un bouton temporaire) pour garder le build/vue-tsc vert tout du long — seule Task 8 fait le vrai remplacement complet de la nav plate par AppNav.

## Prochaine action

Continuer le dispatch tâche par tâche de R2 (Task 2 après vérification indépendante de Task 1), même discipline que R1 (subagent-driven + vérification indépendante systématique + vue-tsc/tests réels à chaque étape). Une fois les 9 tâches terminées : merge master, version bump (0.9.0→0.10.0), release. Puis enchaîner sur l'écriture du plan R3 (Applications > Installation rapide).

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.9.0` (R1 publiée)
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS — juste ce qui est injecté dans le slot `#nav`/défaut change
- themeStore/layoutStore : pattern existant à reproduire pour le nouveau 3ème axe "style"
- Playwright headless bloqué (libnspr4 manquant, sudo interactif indisponible) — vérification visuelle par lecture de code/tests, pas par screenshot, jusqu'à ce que l'utilisateur installe les libs manquantes lui-même
- Référence Windows : `C:\Users\Momo\Desktop\Nitrite 2.0\src\` (navigation.ts, AppSidebar.vue, MasterInstallPage.vue, StatsReportsPage.vue, DriversPage.vue, UpdatesPage.vue) — consulter pour inspiration de contenu/structure, jamais copier le code React/autre framework tel quel (NiTriTe est en Vue aussi en fait — vérifier le framework exact si besoin de réutiliser des patterns)
