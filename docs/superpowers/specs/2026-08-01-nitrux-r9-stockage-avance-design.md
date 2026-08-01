# NiTruX Phase R9 — Stockage avancé — Design

## 1. Contexte

Quatrième phase du second round de refonte (découpage R6-R11 validé, voir `docs/superpowers/specs/2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1). R9 couvre le groupe "Avancé (BETA)" de `navigation.ts` (NiTriTe Windows), restreint à ses éléments liés au stockage : Récupération de données, Visualiseur de disque, Boot Manager, Restauration. "Clonage Système" (`clone`) est explicitement **hors scope** — `clone_disk` est déjà exposé dans `DisksPage.vue` aux côtés de `format_partition`/`extend_partition`, dans la catégorie Stockage déjà dédiée ; l'extraire dans une page séparée n'apporterait aucune valeur, contrairement à l'extraction d'Antivirus en R7 qui sortait d'une page "Dépannage" mal nommée.

## 2. Investigation menée avant de spécifier (vérifiée en direct sur la VM)

- **`/boot/grub/grub.cfg` n'est PAS lisible par un utilisateur normal** (`-rw------- root root`, confirmé sur la VM). Lire ce fichier compilé nécessiterait `pkexec` juste pour un affichage — disproportionné pour une simple consultation. À la place, `/etc/default/grub` (le fichier de configuration source, non compilé) est lisible sans privilège et contient l'information utile (entrée par défaut, timeout, distributeur, ligne de commande noyau).
- **`efibootmgr` fonctionne sans privilège sur cette VM** (confirmé, liste réellement les entrées de démarrage UEFI). Ce n'est pas garanti sur tous les systèmes (dépend des règles udev de la distribution) — dégradation gracieuse obligatoire si la commande échoue.
- Aucun dossier Corbeille (`~/.local/share/Trash`) n'existe encore sur la VM de test (rien n'y a jamais été supprimé via un gestionnaire de fichiers) — le module doit gérer honnêtement ce cas (dossier absent = corbeille vide, pas une erreur).

## 3. Les 4 pages de cette phase

### 3.1 Visualiseur de disque (`disk-visualizer`) — zéro nouveau backend

Réutilise intégralement `list_disk_usage` (déjà existant, `UsageEntry` avec `mountpoint`/`total_bytes`/`used_bytes`/`used_percent`) et `find_large_files_cmd` (déjà existant depuis la Phase 3, `LargeFile` avec `path`/`size_bytes`). Présentation visuelle : barre de progression par point de montage (réutilise le pattern déjà établi en R8 pour la force du signal WiFi), et une liste des plus gros fichiers/dossiers du répertoire choisi, triée par taille décroissante avec une barre proportionnelle. Pas de nouvelle capacité, une meilleure vue d'ensemble visuelle de données déjà collectées.

### 3.2 Récupération de données (`data-recovery`) — nouveau, non-privilégié

**Décision de scope délibérée** : au lieu d'intégrer un outil de récupération de données brute sur disque (`testdisk`/`photorec` — complexe, à risque, nécessiterait de parser une sortie interactive/scriptée non triviale, et opère directement sur des périphériques bloc), cette page implémente un **navigateur de Corbeille** suivant la spécification XDG Trash (`~/.local/share/Trash/files/` + `~/.local/share/Trash/info/*.trashinfo`, le standard utilisé par tous les gestionnaires de fichiers Linux modernes — Nautilus, Dolphin, etc.). C'est un vrai besoin de "récupération de données" (retrouver un fichier supprimé récemment), scopé de façon sûre : toutes les opérations (lister, restaurer, supprimer définitivement) se font exclusivement dans le dossier personnel de l'utilisateur, sans aucun privilège. Le fichier `.trashinfo` associé à chaque élément contient son chemin d'origine (format INI simple, `Path=` + `DeletionDate=`), permettant une restauration réelle à l'emplacement d'origine.

Nouveau module backend `src-tauri/src/trash.rs` : liste le contenu de la corbeille (parsing des fichiers `.trashinfo`), restaure un élément (déplace de `Trash/files/` vers son `Path=` d'origine, supprime le `.trashinfo` correspondant), supprime définitivement un élément (retire de `Trash/files/` et son `.trashinfo`). Absence du dossier Corbeille = liste vide, pas une erreur.

### 3.3 Boot Manager (`boot-manager`) — nouveau, lecture seule, non-privilégié

Nouveau module backend `src-tauri/src/boot_manager.rs` : lit `/etc/default/grub` (parsing simple clé=valeur, extrait `GRUB_DEFAULT`/`GRUB_TIMEOUT`/`GRUB_DISTRIBUTOR`/`GRUB_CMDLINE_LINUX_DEFAULT`) et tente `efibootmgr` pour lister les entrées de démarrage UEFI si le système en a un (`/sys/firmware/efi` existe). Dégradation gracieuse honnête : si `/etc/default/grub` est absent (système non-GRUB), ou si `efibootmgr` échoue/n'est pas installé (système BIOS legacy ou permissions insuffisantes selon la distribution), affiche un message clair plutôt qu'une erreur brute. Aucune capacité d'édition dans cette v1 — même décision conservatrice qu'Optimisations (R6) et Bluetooth (R8) : modifier la configuration de démarrage est une opération à fort risque (un système mal configuré peut ne plus démarrer), hors scope sans conception et vérification dédiées.

### 3.4 Restauration (`restore-points`) — extraction, zéro nouveau backend

Extrait l'onglet "Snapshots" de `TroubleshootPage.vue` vers sa propre page dédiée — même pattern que l'extraction d'Antivirus en R7. Réutilise `create_snapshot`/`list_snapshots` (déjà existants, déjà privilégiés via `nitrux-pkexec-create-snapshot`, déjà vérifiés en VM lors des phases initiales) à l'identique. `TroubleshootPage.vue` après cette phase : un seul onglet restant (Dépannage : `fix-broken`/`restart-network`) — suffisamment simple pour retirer la structure à onglets entièrement et afficher directement son contenu.

## 4. Hors scope pour R9

- Récupération de données brute sur disque (testdisk/photorec) — décision explicite, voir §3.2.
- Édition de la configuration de démarrage (GRUB/EFI) — lecture seule uniquement dans cette version.
- "Clonage Système" en page dédiée — reste dans `DisksPage.vue`, décision explicite voir §1.
- R10 (Logiciels & déploiement), R11 (Diagnostic & config) — 9 pages restantes du découpage validé.

## 5. Vérification

Même discipline que R1-R8 : tests unitaires Rust (parsing `.trashinfo`/`/etc/default/grub`/`efibootmgr` par littéraux), tests Vitest par page, `vue-tsc --noEmit`, vérification manuelle sur la VM Debian pour les commandes touchant le système réel — en particulier un cycle complet suppression→corbeille→restauration testé pour de vrai (créer un fichier de test, le supprimer via la corbeille XDG, confirmer qu'il apparaît dans la liste, le restaurer, confirmer qu'il est revenu à son emplacement d'origine). Merge, version bump (0.16.0→0.17.0 attendu), build `.deb`/`.rpm`, release GitHub.
