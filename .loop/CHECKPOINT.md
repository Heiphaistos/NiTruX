# Checkpoint

Dernière étape réussie : **Phase 1 COMPLÈTE et release v0.1.0-phase1 publiée** (https://github.com/Heiphaistos/NiTruX/releases/tag/v0.1.0-phase1, .deb+.rpm).

Prochaine action : exécuter `docs/superpowers/plans/2026-07-31-nitrux-phase2-part1.md` (8 tâches, PackageManager trait + apt/dnf/pacman/zypper + Flatpak/Snap + PackagesPage.vue, lecture seule uniquement) via subagent-driven-development dans un nouveau worktree `.worktrees/phase-2-part1`.

Contexte pour reprendre à froid :
- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git` (privé), branche principale `master`
- `master` contient tout Phase 1 (13 tâches) + release v0.1.0-phase1 taguée
- Convention établie : toute commande shell-out via `subprocess::run_with_timeout` (`src-tauri/src/subprocess.rs`) + `Result<T, String>`, jamais de `Command::new()` brut
- Environnement : WSL2 Ubuntu pour npm/cargo/tauri, git côté Windows (voir LESSONS.md)
- Décision de scope importante : Phase 2 Part 1 = détection + listing LECTURE SEULE des paquets uniquement. Install/upgrade (écriture, privilégié, polkit/pkexec) volontairement exclu de l'exécution autonome — nécessite revue humaine avant d'écrire du code qui modifierait des paquets sur la vraie machine de Momo. À proposer comme "Phase 2 Part 2" avec validation explicite.

En attente de validation humaine :
1. Fix npm audit (8 vulnérabilités high, devDependency uniquement — vue-tsc/@vue/test-utils, `npm audit fix --force` bloqué par le classificateur auto-mode car breaking change). Non-bloquant pour les releases (jamais dans le bundle livré).
2. AppImage manquant — `sudo apt install xdg-utils` requis dans l'environnement WSL2 (mot de passe sudo non disponible en autonomie).
3. Phase 2 Part 2 (install/upgrade réel de paquets, polkit/pkexec) — nécessite conception + revue avant implémentation, pas à faire en autonomie sans supervision.
