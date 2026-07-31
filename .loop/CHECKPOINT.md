# Checkpoint

Dernière étape réussie : **Phase 1 + Phase 2 Part 1 + Phase 3 Part 1 COMPLÈTES**, mergées sur master, 3 releases publiées :
- v0.1.0-phase1 : moteur thème/layout + Système & diagnostic
- v0.2.0-phase2part1 : détection paquets multi-distro + listing lecture seule
- v0.3.0-phase3part1 : outils disque lecture seule (listing, doublons, gros fichiers, hash, SMART)

Prochaine action : écrire et exécuter le plan Phase 4 Part 1 (Réseau, sécurité & maintenance — lecture seule) : analyseur wifi, scanner de ports, éditeur hosts (lecture), DNS switcher (lecture config actuelle), infos Docker. Même pattern que Phases 1-3. Le "bouton de dépannage" (actions correctives) et toute écriture système restent hors scope autonome — nécessitent revue humaine (polkit/pkexec).

Contexte pour reprendre à froid :
- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git` (privé), branche `master`
- Version actuelle : 0.3.0
- Convention établie : `subprocess::run_with_timeout` pour tout shell-out, `Result<T, String>` pour toute commande faillible, trait-based abstraction pour tout ce qui a plusieurs implémentations (ex: PackageManager)
- Environnement : WSL2 Ubuntu pour npm/cargo/tauri, git côté Windows (voir LESSONS.md)
- Cycle par tâche : implémenteur (subagent) → vérification INDÉPENDANTE (re-lancer les tests soi-même, ne jamais faire confiance au rapport seul) → revue qualité si le risque le justifie → fix si besoin → journal → push
- 7 pages navigables : Dashboard, Matériel, Pilotes, Journaux, Apparence (thème/layout), Paquets, Disques

En attente de validation humaine (non-bloquant pour continuer le reste) :
1. npm audit fix --force (8 vulnérabilités high, devDependency uniquement) — bloqué par le classificateur auto-mode
2. AppImage — nécessite `sudo apt install xdg-utils` (mot de passe non disponible en autonomie)
3. Toute opération d'écriture système (install/upgrade paquets, formatage partition, actions de dépannage) — nécessite conception + revue avant implémentation, jamais en autonomie
