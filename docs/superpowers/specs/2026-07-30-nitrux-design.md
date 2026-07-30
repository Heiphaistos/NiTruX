# NiTruX — Design

Date : 2026-07-30
Statut : Approuvé

## 1. Vision

NiTruX est la réinvention pour Linux de NiTriTe (`C:\Users\Momo\Desktop\Nitrite 2.0` — Tauri v2 + Rust + Vue 3, couteau suisse de maintenance/diagnostic Windows, 52 pages). NiTruX vise la même ambition — un couteau suisse complet de maintenance, diagnostic et administration système — mais entièrement pensé et réécrit pour Linux, sans aucun code Windows-only hérité (WMI, BCD, DISM/SFC, registre, drivers `.inf` disparaissent, remplacés par leurs équivalents Linux natifs).

Nom du repo : `NiTruX`. Emplacement local : `C:\Users\Momo\Desktop\NiTruX`.

## 2. Stack technique

- **Tauri v2 + Rust + Vue 3 + TypeScript + Vite** — identique à NiTriTe. Tauri est nativement multi-plateforme ; ce n'est pas un changement de stack, seulement le retrait de tout le code `#[cfg(windows)]` / appels PowerShell/WMI côté backend.
- **Environnement de dev** : WSL2 (Ubuntu) sur la machine Windows de développement, avec build/run Linux natif via WSLg pour le GUI (même approche que le toolchain Android déjà en place — voir mémoire `reference_android_toolchain_local`).
- **Packaging cible** : `.deb`, `.rpm`, AppImage via le bundler natif Tauri. Pas de binaire universel — chaque paquet reste dans l'écosystème de sa distro.

## 3. Identité visuelle

### 3.1 Direction
Base "moderne distro-neutre" poussée : cartes arrondies, palette calibrée, typographie sans-serif propre. Volontairement agnostique vis-à-vis du DE (GNOME/KDE/XFCE) — l'app impose sa propre identité visuelle cohérente partout, sans dépendre des thèmes système.

### 3.2 Thèmes livrés par défaut (12)
Catppuccin (Mocha/Latte), Nord, Adwaita (dark/light), Gruvbox, Dracula, Everforest, Tokyo Night, Solarized (dark/light), Rosé Pine, One Dark, Kanagawa, Ayu.

Tous éditables et dérivables depuis l'éditeur de thème.

### 3.3 Dispositions (layouts) proposées (8)
Sidebar classique, Dashboard modulaire (widgets réarrangeables), Command palette-first, Sidebar rétractable (icônes, extension au survol), Barre supérieure + onglets, Master-detail (liste + panneau détail), Bento grid, Dock flottant.

### 3.4 Éditeur temps réel
Reprend le pattern de `ThemeEditorPage.vue` de NiTriTe : aperçu global instantané appliqué à toute l'app pendant l'édition, export/import JSON, thèmes nommés sauvegardés localement. Étendu pour piloter également la disposition active (pas seulement les couleurs) — un même éditeur pilote thème + layout, avec bascule live entre les deux onglets.

### 3.5 Icône / branding
Réutilisation de l'iconset `src-tauri/icons/` de NiTriTe (`128x128.png`, `128x128@2x.png`, `32x32.png`, `icon.ico` comme source de conversion) comme identité de marque commune, cohérente avec le reste du portefeuille de produits. Une future vitrine web NiTruX dériverait son favicon de la même source.

## 4. Architecture multi-distro

Abstraction Rust via un trait `PackageManager` avec implémentations `Apt`, `Dnf`, `Pacman`, `Zypper`. Détection au démarrage par présence des binaires (`/usr/bin/apt`, `/usr/bin/dnf`, `/usr/bin/pacman`, `/usr/bin/zypper`). Flatpak et Snap sont toujours vérifiés en supplément comme dénominateur commun universel, indépendamment du gestionnaire natif détecté. Le frontend Vue ne connaît jamais le gestionnaire sous-jacent — il appelle des commandes Tauri génériques (`list_updates`, `install_package`, `upgrade_all`...) que le backend route vers la bonne implémentation.

## 5. Piliers fonctionnels v1

### 5.1 Système & diagnostic
- Infos matérielles temps réel : CPU/RAM/process (`sysinfo`), température (`sensors -j`), batterie (`/sys/class/power_supply`)
- Informations complètes par composant : `lspci`, `dmidecode` (nécessite root, optionnel)
- Page "Pilotes & modules noyau" : `lsmod`/`modinfo`, détection du driver GPU actif (nvidia/nouveau/amdgpu/i915), suggestions d'installation du driver propriétaire adaptées à la distro détectée
- Logs système via `journalctl`
- Dashboard vue d'ensemble + benchmark

### 5.2 Paquets & applications
- Gestion unifiée install/remove/search (§4)
- **Mise à jour en un clic** : agrège apt+dnf+pacman+flatpak+snap dans une seule action, sortie streamée en temps réel dans l'UI
- **Installation guidée (équivalent MasterInstall)** : sélection groupée d'outils/apps courants à installer en une passe

### 5.3 Disques & stockage
- Partition manager (`lsblk`/`parted`)
- Visualiseur d'espace disque, détection de doublons, recherche de gros fichiers
- Hash checker, vérification SMART (`smartctl`)
- Backup / clone

### 5.4 Réseau, sécurité & maintenance
- Analyseur wifi (`nmcli`/`iw`), scanner de ports, éditeur `/etc/hosts`, DNS switcher
- Gestionnaire Docker
- Scan malware/rootkit (`clamav`/`rkhunter`), nettoyeur de fichiers temporaires, snapshots Btrfs/Timeshift, pare-feu UFW
- **Bouton de dépannage** : ensemble curé d'actions non-interactives (réparer paquets cassés, vider caches, redémarrer NetworkManager/PipeWire, nettoyer paquets orphelins). "Silencieux" du point de vue UX (pas de confirmation étape par étape) mais **jamais silencieux côté sécurité** : une seule authentification polkit visible en amont, log complet de chaque commande exécutée.

## 6. Sécurité — élévation de privilèges

Aucune exécution de l'application entière en root. Chaque action privilégiée (install paquet, modification GRUB, snapshot, écriture `/etc/hosts`...) passe par **polkit** (`pkexec`) avec une règle `.policy` dédiée par catégorie d'action. Cohérent avec la discipline sécurité du projet (jamais d'élévation cachée, jamais de mot de passe géré par l'app elle-même).

## 7. Roadmap de phases

Portée finale complète voulue, mais implémentation séquencée pour rester livrable :

1. **Phase 1** — Fondations : shell applicatif, moteur de thème/layout (12 thèmes × 8 dispositions + éditeur temps réel), pilier Système & diagnostic
2. **Phase 2** — Paquets & applications (mises à jour un clic, installation guidée)
3. **Phase 3** — Disques & stockage
4. **Phase 4** — Réseau, sécurité & maintenance + bouton de dépannage

## 8. Hors scope v1

- Support d'un DE spécifique (extensions GNOME Shell, widgets Plasma) — l'app reste autonome
- Live USB / créateur d'ISO (équivalent WinPE de NiTriTe) — reporté après Phase 4
- Boot manager bas niveau (GRUB avancé) — reporté après Phase 4, en dehors du diagnostic kernel/modules de la Phase 1
