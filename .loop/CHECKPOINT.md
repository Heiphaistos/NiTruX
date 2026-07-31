# Checkpoint — Redesign (R1–R5), session démarrée 2026-08-01

## État antérieur (terminé, référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, mergés, publiés jusqu'à v0.8.0** (voir git log/releases GitHub). Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. Ce travail n'est PAS remis en cause par la refonte — voir spec §7 "Out of scope" : aucune modification des commandes backend déjà livrées, seulement réutilisation.

## Nouveau travail en cours

Spec écrite et committée : `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` (commit `69b2bf2`). Utilisateur a validé la structure de nav (9 catégories, devenues 7 après fusion Paramètres), le système visuel 3 axes (12 palettes × 8 dispositions × 12 styles), et les 4 nouvelles fonctionnalités via le compagnon visuel, puis a dit "oui fait tout je vais me coucher" — autorisation complète, pas d'attente de revue supplémentaire de la spec.

**Phase R1 (Foundation) : TERMINÉE, mergée, publiée en v0.9.0-r1-foundation.** 9 tâches (registre de styles, styleStore, style-tokens.css, 7 composants Nx*, categories.ts, AppNav.vue, extension ThemeEditorPage) — toutes vérifiées indépendamment (jamais confiance aveugle dans un rapport de sous-agent). 79 tests frontend + 124 Rust (inchangé), diff vérifié strictement scopé via `git diff master --stat` avant merge (aucune page existante déplacée). Le nouveau sélecteur de style (12 styles) est déjà utilisable dès maintenant via Apparence > onglet Style — c'est le seul changement visible de R1, tout le reste (AppNav, categories.ts, composants Nx*) est fondation invisible tant que R2 ne les branche pas.

## Prochaine action

Écrire `docs/superpowers/plans/2026-08-01-nitrux-r2-restructure.md` (skill writing-plans) pour Phase R2 — restructuration des pages existantes (DisksPage split, SecurityPage split, NetworkPage, HardwarePage→Diagnostic, PackagesPage, LogsPage → tous branchés sur AppNav + composants Nx*, remplaçant la liste plate de App.vue). Voir spec §5 pour le détail exact du découpage. Puis worktree `r2-restructure`, puis dispatch tâche par tâche, même discipline que R1 (subagent-driven + vérification indépendante systématique).

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.9.0`
- App.vue actuel (INCHANGÉ par R1, à modifier en R2) : liste plate de 9 boutons, `Record<PageId,Component>` — R2 doit remplacer ça par `<AppNav v-model="currentPage" />` dans le slot `#nav`, et étendre le `Record` avec les nouveaux `pageId`s de `categories.ts`
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS — juste ce qui est injecté dans le slot `#nav`/défaut change
- themeStore/layoutStore : pattern existant à reproduire pour le nouveau 3ème axe "style"
- Playwright headless bloqué (libnspr4 manquant, sudo interactif indisponible) — vérification visuelle par lecture de code/tests, pas par screenshot, jusqu'à ce que l'utilisateur installe les libs manquantes lui-même
- Référence Windows : `C:\Users\Momo\Desktop\Nitrite 2.0\src\` (navigation.ts, AppSidebar.vue, MasterInstallPage.vue, StatsReportsPage.vue, DriversPage.vue, UpdatesPage.vue) — consulter pour inspiration de contenu/structure, jamais copier le code React/autre framework tel quel (NiTriTe est en Vue aussi en fait — vérifier le framework exact si besoin de réutiliser des patterns)
