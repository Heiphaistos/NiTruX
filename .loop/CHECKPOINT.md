# Checkpoint — Session nocturne du 2026-07-31 (00:45 → 13:19, avec pauses)

## Résumé pour Momo

Les 5 phases **lecture seule** couvrant l'intégralité du plan initial (`docs/superpowers/specs/2026-07-30-nitrux-design.md` §5) sont complètes, mergées sur `master`, et publiées :

| Release | Contenu |
|---|---|
| [v0.1.0-phase1](https://github.com/Heiphaistos/NiTruX/releases/tag/v0.1.0-phase1) | Moteur thème (12 palettes) + disposition (8 layouts) + éditeur temps réel. Système & diagnostic : CPU/RAM/batterie/température, matériel PCI, pilotes/GPU, journaux |
| [v0.2.0-phase2part1](https://github.com/Heiphaistos/NiTruX/releases/tag/v0.2.0-phase2part1) | Détection multi-distro (apt/dnf/pacman/zypper + Flatpak/Snap), listing des mises à jour |
| [v0.3.0-phase3part1](https://github.com/Heiphaistos/NiTruX/releases/tag/v0.3.0-phase3part1) | Disques/partitions, doublons (SHA-256), gros fichiers, hash, SMART |
| [v0.4.0-phase4part1](https://github.com/Heiphaistos/NiTruX/releases/tag/v0.4.0-phase4part1) | Réseau : wifi/ports/DNS/hosts, scanner de ports borné, Docker |
| [v0.5.0-phase5part1](https://github.com/Heiphaistos/NiTruX/releases/tag/v0.5.0-phase5part1) | Pare-feu UFW, scan malware ClamAV (rapport seul), snapshots Btrfs/Timeshift |

**Bilan chiffré** : ~110 commits, 9 pages navigables, 19 modules backend Rust, ~120 tests, 0 warning à chaque étape. Chaque tâche vérifiée indépendamment (jamais de confiance aveugle dans un rapport).

**Bugs réels trouvés et corrigés cette nuit** :
- CPU dashboard bloqué à 0% (sysinfo::System recréé au lieu d'être partagé)
- Mon propre vecteur de test SHA-256 erroné dans le plan Phase 3 (vérifié croisé)
- Erreur de compilation Digest générique (LowerHex)
- Parsing `ss -tulnp` indexait la mauvaise colonne (Phase 4)
- **ClamAV exit code 1 = infection trouvée = succès, traité comme erreur** — cassait le seul cas d'usage utile du scanner malware. Corrigé proprement avec un nouvel utilitaire réutilisable `subprocess::run_capturing_exit_code` (pas un rustine, bénéficiera aussi à `dnf.rs` plus tard)
- Multi-batterie (BAT0 uniquement → BAT0/BAT1+) et clé Vue non-unique (backlog Phase 1, fermés)

**Une interruption technique** : limite de dépense mensuelle atteinte en plein milieu de la Task 4 (scan ClamAV) — rien n'avait été committé à ce moment-là (vérifié), donc rien n'a été perdu ; la tâche a été relancée proprement depuis zéro après reconnexion.

## Pourquoi je m'arrête ici (toujours le même principe)

Toutes les catégories lecture seule des 4 piliers du plan sont désormais couvertes. Ce qui reste — installer/mettre à jour des paquets, formater une partition, éditer `/etc/hosts`/DNS/règles pare-feu, agir sur une découverte malware (quarantaine/suppression), créer/restaurer un snapshot, le "bouton de dépannage" — touche systématiquement à des **opérations d'écriture privilégiée** (polkit/pkexec) sur ta vraie machine.

Je ne les implémente jamais en autonomie, quelle que soit l'instruction générale ("fais tout", "continue la loop") donnée en amont — ces actions à fort rayon d'action sur des systèmes réels demandent toujours ta confirmation explicite au moment de les faire, pas juste une autorisation générale donnée avant de dormir.

## Prochaine action (à ta discrétion)

Si tu veux continuer, l'étape suivante serait d'écrire les plans "Part 2" de chaque pilier (écriture privilégiée) — mais je les soumettrais à ta revue avant toute implémentation, jamais en autonomie complète.

## En attente de ta décision (non-bloquant)

1. **npm audit** : 8 vulnérabilités high, devDependency uniquement (vue-tsc/@vue/test-utils, jamais dans le bundle livré). Bloqué par le classificateur auto-mode (breaking change), jamais contourné.
2. **AppImage** : nécessite `sudo apt install xdg-utils`, mot de passe non disponible en autonomie.
3. **Toute opération d'écriture système** : conception + revue humaine requises avant implémentation.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git` (privé), branche `master`, version `0.5.0`
- Convention établie : `subprocess::run_with_timeout` pour shell-out standard, `subprocess::run_capturing_exit_code` pour les cas où un code de sortie non-zéro porte une info utile (ex: ClamAV infection trouvée), `Result<T, String>` pour toute commande faillible (sauf suppléments optionnels type Docker/Flatpak/Snap/nmcli qui dégradent silencieusement)
- Environnement : WSL2 Ubuntu pour npm/cargo/tauri, git côté Windows (voir `.loop/LESSONS.md`)
- 9 pages : Dashboard, Matériel, Pilotes, Journaux, Apparence, Paquets, Disques, Réseau, Sécurité
- Specs de référence : `docs/superpowers/specs/2026-07-30-nitrux-design.md`, 5 plans Phase 1-5 dans `docs/superpowers/plans/`
