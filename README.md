# NiTruX

Couteau suisse de maintenance et diagnostic pour Linux — informations matérielles en temps réel, gestion des paquets multi-distro, pilotes et modules noyau, journaux système, et un moteur de thèmes/dispositions personnalisable.

Distro-agnostique : détection automatique d'apt/dnf/pacman/zypper, avec Flatpak et Snap en couche supplémentaire systématique.

## Fonctionnalités

- **Diagnostic** : composants PCI, matériel détaillé, périphériques, processus & services, logiciels installés, comptes utilisateurs, historique des mises à jour
- **Performance** : optimisations système, températures, benchmark CPU/disque/mémoire, historique de performance
- **Applications** : installation rapide (catalogue de 500+ applications), gestionnaire de paquets, installation par profils
- **Stockage** : disques & partitions, recherche de doublons/gros fichiers/vérification de hash, visualiseur de disque, récupération de données (corbeille), boot manager, points de restauration
- **Maintenance** : mises à jour, pilotes, dépannage, désinstalleur, nettoyeur, sauvegarde, antivirus, dépendances manquantes
- **Réseau** : vue d'ensemble, changement de DNS, analyseur Wi-Fi, Bluetooth, scripts & snippets, pare-feu (UFW)
- **Rapports** : générateur de rapport (JSON/Markdown/TXT/HTML/PDF), journaux système
- **Personnalisation** : 13 thèmes, 8 dispositions, éditeur en temps réel

Les actions nécessitant les droits administrateur (installation de paquets, formatage de partition, pare-feu...) passent par polkit (`pkexec`), jamais l'application entière en root.

## Installation

Télécharger le dernier `.deb`, `.rpm` ou `.AppImage` depuis la [page des releases](https://github.com/Heiphaistos/NiTruX/releases).

## Développement

Stack : [Tauri v2](https://tauri.app/) + Rust + Vue 3 + TypeScript.

```bash
npm install
npm run tauri dev    # lancer en mode développement
npm run tauri build  # construire les paquets .deb/.rpm/.AppImage
```

### Tests

```bash
npm run test               # suite frontend (Vitest)
cargo test --manifest-path src-tauri/Cargo.toml  # suite backend (Rust)
npx vue-tsc --noEmit       # vérification de types
```

### Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Vue - Official](https://marketplace.visualstudio.com/items?itemName=Vue.volar) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
