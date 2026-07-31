# Checkpoint — Redesign (R1–R5), session démarrée 2026-08-01

## État antérieur (terminé, référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, mergés, publiés jusqu'à v0.8.0** (voir git log/releases GitHub). Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. Ce travail n'est PAS remis en cause par la refonte — voir spec §7 "Out of scope" : aucune modification des commandes backend déjà livrées, seulement réutilisation.

## Nouveau travail en cours

Spec écrite et committée : `docs/superpowers/specs/2026-08-01-nitrux-redesign-design.md` (commit `69b2bf2`). Utilisateur a validé la structure de nav (9 catégories), le système visuel 3 axes (12 palettes × 8 dispositions × 12 styles), et les 4 nouvelles fonctionnalités via le compagnon visuel, puis a dit "oui fait tout je vais me coucher" — autorisation complète, pas d'attente de revue supplémentaire de la spec.

**Dernière action avant écriture de ce checkpoint** : spec committée, fichiers `.loop/` réinitialisés pour ce nouveau travail. Étape suivante : invoquer `writing-plans` pour Phase R1 (Foundation), puis exécuter en `subagent-driven-development`.

## Prochaine action

Écrire `docs/superpowers/plans/2026-08-01-nitrux-r1-foundation.md` (skill writing-plans), puis worktree `phase-r1-foundation`, puis dispatch tâche par tâche.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.8.0`
- App.vue actuel : liste plate de 9 boutons, `Record<PageId,Component>` — c'est exactement ce qui doit disparaître en R1/R2
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS — juste ce qui est injecté dans le slot `#nav`/défaut change
- themeStore/layoutStore : pattern existant à reproduire pour le nouveau 3ème axe "style"
- Playwright headless bloqué (libnspr4 manquant, sudo interactif indisponible) — vérification visuelle par lecture de code/tests, pas par screenshot, jusqu'à ce que l'utilisateur installe les libs manquantes lui-même
- Référence Windows : `C:\Users\Momo\Desktop\Nitrite 2.0\src\` (navigation.ts, AppSidebar.vue, MasterInstallPage.vue, StatsReportsPage.vue, DriversPage.vue, UpdatesPage.vue) — consulter pour inspiration de contenu/structure, jamais copier le code React/autre framework tel quel (NiTriTe est en Vue aussi en fait — vérifier le framework exact si besoin de réutiliser des patterns)
