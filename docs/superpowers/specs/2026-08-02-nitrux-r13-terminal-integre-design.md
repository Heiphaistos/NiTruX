# Phase R13 (Terminal intégré) — Design Spec

## 1. Contexte

Demandé explicitement par l'utilisateur. Ce besoin avait été identifié dès R8 puis délibérément différé — il nécessite deux dépendances nouvelles jamais utilisées ailleurs dans le projet (`portable-pty` côté Rust, `@xterm/xterm` côté frontend) et une architecture radicalement différente de tout ce qui existe : chaque commande de l'app jusqu'ici est une requête/réponse ponctuelle (`invoke` → `Result`), alors qu'un terminal exige un flux bidirectionnel continu (frappes clavier → shell, sortie shell → affichage, en continu, pas un aller-retour unique).

## 2. Architecture

### 2.1 Backend : `terminal.rs`
- `portable-pty = "0.9.0"` — crate établi, cross-platform, déjà utilisé par des projets Tauri similaires.
- Un shell réel est lancé dans un pseudo-terminal : `$SHELL` de l'utilisateur si défini, sinon `/bin/bash` (confirmé présent sur la VM de dev). **Aucune élévation de privilège** — le shell hérite exactement des droits de l'utilisateur qui a lancé NiTruX, identique à ouvrir un terminal normalement. Ce n'est pas une nouvelle frontière de sécurité, seulement une nouvelle façon d'exposer ce que l'utilisateur pourrait déjà faire lui-même.
- État partagé : `Mutex<HashMap<String, TerminalSession>>` managé par Tauri, une session par `id` d'onglet terminal (le frontend génère un UUID par onglet ouvert).
- 4 commandes :
  - `spawn_terminal(id, on_data: Channel<String>)` — ouvre le pty, lance le shell, démarre un thread qui lit en continu la sortie et l'envoie via le `Channel` Tauri (mécanisme de streaming natif de Tauri v2, plus efficace qu'émettre des événements un par un).
  - `write_to_terminal(id, data)` — écrit les frappes clavier de l'utilisateur dans le pty.
  - `resize_terminal(id, rows, cols)` — redimensionne le pty quand la fenêtre change de taille.
  - `close_terminal(id)` — ferme la session, tue le processus shell.
- La logique cœur (ouverture pty + lancement shell + obtention reader/writer) est factorisée dans une fonction pure `open_shell_pty()` testable indépendamment du `Channel` Tauri (qui exige un contexte d'app réel, impossible à instancier dans un test unitaire) — le test réel écrit une commande dans le pty et lit la sortie via un thread + `mpsc` + timeout, exactement le pattern déjà utilisé et éprouvé dans `subprocess.rs::run_with_timeout`.

### 2.2 Frontend : `TerminalPage.vue`
- `@xterm/xterm` (rendu terminal, canvas) + `@xterm/addon-fit` (redimensionnement automatique à la taille du conteneur).
- À l'ouverture de la page : génère un `id` (UUID), crée un `Terminal` xterm.js, l'attache au DOM, ouvre un `Channel` Tauri pour recevoir les données (`channel.onmessage = (data) => term.write(data)`), appelle `spawn_terminal`.
- `term.onData((data) => invoke("write_to_terminal", { id, data }))` — chaque frappe clavier part immédiatement vers le pty.
- `ResizeObserver` sur le conteneur → `fitAddon.fit()` + `invoke("resize_terminal", { id, rows, cols })`.
- À la fermeture de la page (`onUnmounted`) : `invoke("close_terminal", { id })`, tue le shell proprement, pas de processus orphelin.

## 3. Vérification (adaptée à une fonctionnalité inhérentement interactive)

- **Backend** : test réel avec un vrai shell (pas de mock) — écrit une commande connue dans le pty, lit la sortie via thread+timeout, confirme le contenu attendu. Ceci s'exécute dans l'environnement de test WSL2/Linux existant, donc lance un vrai bash.
- **Frontend** : `@xterm/xterm` est mocké dans les tests (comme `@tauri-apps/api/core` l'est déjà partout ailleurs) — un rendu canvas réel n'est ni testable ni pertinent à vérifier en jsdom ; on vérifie le câblage (spawn appelé au montage, écriture utilisateur relayée, données reçues écrites dans le terminal, fermeture propre au démontage), pas le rendu pixel.
- **VM (remplace la capture d'écran, toujours impossible sur cette VM)** : build réel installé sur la VM, lancement de l'app, navigation vers la page Terminal via `xdotool` (déjà installé en R9), puis confirmation **côté système** qu'un vrai processus shell enfant existe sous le PID de l'app (`ps --ppid <pid_app>`) — preuve directe et non-visuelle que le pty a bien été ouvert et un shell réellement lancé.

## 4. Hors scope
Multi-onglets avec fermeture/réouverture dynamique (v1 : un seul terminal actif à la fois, page dédiée). Thèmes de couleur xterm.js personnalisés (v1 : palette par défaut). Historique de commandes persistant entre sessions (le shell lancé a son propre historique bash natif via `~/.bash_history`, suffisant).
