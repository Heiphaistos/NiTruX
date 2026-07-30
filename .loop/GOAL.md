# But

Faire progresser NiTruX (port Linux de NiTriTe, Tauri v2+Rust+Vue3) le plus loin possible de façon autonome pendant que Momo dort :
1. Terminer Phase 1 (Fondations) déjà en plan : `docs/superpowers/plans/2026-07-30-nitrux-phase1.md`, Tasks 8→13 restantes.
2. Merger `phase-1-fondations` sur `master` une fois Phase 1 vérifiée complète.
3. Créer le repo GitHub privé (`Heiphaistos/NiTruX` ou nom équivalent, suivre convention portefeuille) et pousser `master`.
4. Enchaîner Phase 2 (Paquets & applications), Phase 3 (Disques & stockage), Phase 4 (Réseau/sécurité/maintenance) — voir `docs/superpowers/specs/2026-07-30-nitrux-design.md` §5 pour le détail fonctionnel de chaque pilier.
5. Corriger tout bug réel trouvé en cours de route (jamais fabriqué — seulement des bugs reproduits et vérifiés).
6. Dès qu'un état buildable/stable est atteint (au minimum après Phase 1 mergée), builder et publier une release GitHub avec les 3 formats de paquet Linux (.deb, .rpm, AppImage — via le bundler Tauri, voir spec §2 "Packaging cible"). Republier une release à chaque phase complétée.

## Critère de fin (vérifiable)

Pas de "fin" unique — c'est une boucle de progression continue. S'arrêter et rapporter quand :
- Phase 4 est complète et vérifiée (tous les piliers du plan implémentés, testés, mergés, poussés, release publiée), OU
- `max_iterations` atteint, OU
- `stop_si_pas_de_progres` déclenché (aucun progrès mesurable sur N itérations consécutives).

Chaque itération doit produire un résultat vérifiable (tests passants, build réussi, commit réel) — jamais une simple déclaration de progrès.

## Hors-scope

- Pas de suppression de code existant hors dead-code évident.
- Pas d'action destructive/irréversible sur données réelles (aucune donnée réelle en jeu ici, projet neuf).
- Pas de changement de stack (Tauri v2+Rust+Vue3 reste la stack, voir CLAUDE.md §2).
- Live USB/ISO builder, GRUB avancé, extensions DE spécifiques — explicitement "Hors scope v1" dans la spec design, ne pas les implémenter même si le temps le permet.
- Ne jamais committer secrets/logs/artefacts de build (`.gitignore` déjà en place, le respecter).

## Bornes

max_iterations: 60
stop_si_pas_de_progres: 5

## Méthode par tâche

- Tâches Phase 1 restantes (8-13) : continuer `subagent-driven-development` déjà en cours (implémenteur + revue spec + revue qualité, cycle complet par tâche) — c'est le pattern déjà validé sur les tâches 1-7.
- Phase 2-4 : plan absent (pas encore écrit en détail tâche par tâche comme Phase 1). Avant de coder, écrire un plan d'implémentation par pilier (skill `writing-plans`, même format que `2026-07-30-nitrux-phase1.md`) puis exécuter en `subagent-driven-development`. Ne pas coder à l'aveugle sans plan écrit — même en autonomie, TDD + plan restent la discipline.
- Toujours vérifier réellement (tests, cargo check, npm run test, build) avant de marquer une tâche terminée dans CHECKPOINT.md.
