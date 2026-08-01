# NiTruX Phase R8 — Réseau & Terminal — Design

## 1. Contexte

Troisième phase du second round de refonte (découpage R6-R11 validé avec l'utilisateur, voir `docs/superpowers/specs/2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1). R8 couvre le groupe "Réseau & Terminal" de `navigation.ts` (NiTriTe Windows) : DNS Switcher, WiFi Analyzer, Bluetooth, Terminal, Scripts & Snippets.

## 2. Décision de scope : le Terminal intégré est différé, pas abandonné

**Investigation menée avant de spécifier** : toutes les pages construites jusqu'ici (R1-R7) suivent le même schéma technique — une commande Tauri requête/réponse (`invoke` → résultat unique), zéro dépendance nouvelle au-delà de `lucide-vue-next` (R6). Un vrai terminal intégré est structurellement différent : il nécessite un flux bidirectionnel continu (PTY — pseudo-terminal — avec streaming stdin/stdout, pas une requête ponctuelle), ce qui implique une **nouvelle dépendance Cargo** (ex: `portable-pty`) pour gérer le PTY côté Rust et très probablement une **nouvelle dépendance npm** (ex: `xterm.js`) pour le rendu des séquences ANSI côté frontend — les deux premières dépendances non-lucide de toute cette refonte.

Conformément à la discipline déjà appliquée dans ce projet (toute nouvelle dépendance ou nouvelle surface architecturale significative mérite une décision explicite, pas un ajout silencieux en passe autonome — voir §2 de la spec R6 pour l'exemple du choix conservateur sur le générateur de rapport, ou §3.4 pour Optimisations en lecture seule), le Terminal intégré est **différé** de cette phase. Les 4 autres pages de R8, elles, s'inscrivent exactement dans le schéma déjà établi (aucune nouvelle dépendance, réutilisation maximale du code existant) et sont construites normalement dans ce plan.

## 3. Les 4 pages construites dans cette phase

### 3.1 WiFi Analyzer (`wifi-analyzer`) — zéro nouveau backend

Réutilise intégralement `get_network_snapshot` (déjà existant, `wifi_networks: Vec<WifiNetwork>` avec `ssid`/`security`/`signal_percent`/`connected`). Présentation dédiée et plus riche que la simple liste de `NetworkPage` : classement par force de signal, barre de progression visuelle par réseau (réutilise le pattern déjà établi), badge de sécurité coloré (ouvert = danger, WEP = warning, WPA2/WPA3 = success).

### 3.2 DNS Switcher (`dns-switcher`) — réutilise `set_dns_servers`, ajoute de la valeur réelle

`NetworkPage.vue` a déjà un éditeur de texte brut pour les serveurs DNS (`set_dns_servers`, déjà privilégié — `nitrux-pkexec-set-dns`, déjà existant et déjà vérifié en VM lors des phases initiales). Cette page dédiée ajoute ce que le nom "Switcher" implique réellement : des préréglages en un clic (Cloudflare `1.1.1.1`/`1.0.0.1`, Google `8.8.8.8`/`8.8.4.4`, Quad9 `9.9.9.9`/`149.112.112.112`) en plus de l'édition manuelle déjà existante — pas juste un déplacement de l'UI existante, une vraie amélioration UX. Aucun nouveau code privilégié : les préréglages appellent `set_dns_servers` exactement comme le fait déjà l'édition manuelle.

### 3.3 Bluetooth (`bluetooth`) — nouveau module, lecture seule

Nouveau module backend `src-tauri/src/bluetooth.rs` : parse `bluetoothctl show` (statut de l'adaptateur : nom, allumé/éteint) et `bluetoothctl devices` (périphériques appairés). Lecture seule dans cette v1 — même décision que pour "Optimisations" en R6 : activer/désactiver l'adaptateur ou appairer un nouveau périphérique sont de vraies actions avec un effet réel sur le matériel, differées pour une passe dédiée plutôt qu'ajoutées sans réflexion. `bluetoothctl` en lecture (`show`/`devices`) ne nécessite pas de privilège root sur une configuration desktop standard (communication D-Bus utilisateur avec `bluez`), donc pas de nouvelle action `pkexec` ici de toute façon.

### 3.4 Scripts & Snippets (`scripts`)

Nouvelle commande Tauri `run_script(content: String) -> Result<String, String>` — exécute le texte fourni via `sh -c` avec les privilèges de l'utilisateur courant (aucune élévation, aucun `pkexec` — conceptuellement identique à l'utilisateur qui ouvrirait un terminal et taperait la commande lui-même, ce n'est pas une nouvelle surface d'escalade de privilège). Sortie capturée et bornée dans le temps (même pattern `subprocess::run_with_timeout` que partout ailleurs dans ce projet). Le stockage des scripts nommés (nom + contenu) est **entièrement côté frontend**, dans un nouveau store Pinia `scriptsStore` persistant en `localStorage`, suivant exactement le pattern déjà établi par `preferencesStore` (R2) — aucune nouvelle persistance backend, YAGNI respecté.

## 4. Hors scope pour R8

- **Terminal intégré** — différé, nécessite sa propre décision de conception (choix de dépendance PTY + rendu ANSI, revue de sécurité sur l'exposition d'un shell interactif depuis l'interface).
- Activer/désactiver l'adaptateur Bluetooth, appairage de nouveaux périphériques — différé comme Optimisations, nécessiterait sa propre revue si voulu.
- R9-R11 (17 pages restantes du découpage validé).

## 5. Vérification

Même discipline que R1-R7 : tests unitaires Rust (parsing par littéraux pour `bluetoothctl`), tests Vitest par page, `vue-tsc --noEmit`, vérification manuelle sur la VM Debian (qui dispose d'un vrai bluetoothctl/nmcli, contrairement à l'environnement de développement WSL2 qui n'a ni l'un ni l'autre — confirmé pendant la recherche de cette spec). Merge, version bump (0.15.0→0.16.0 attendu), build `.deb`/`.rpm`, release GitHub.
