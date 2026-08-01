# Checkpoint — Refonte R1-R5 terminée ✅, second round (R6-R11) en cours, R6+R7+R8 faits, correctifs post-R8 publiés (v0.17.0)

## Correctifs post-R8 : retour utilisateur, 3 bugs réels + demande AppImage — TERMINÉ, publié v0.17.0

Après v0.16.0, l'utilisateur a signalé 3 bugs réels : (1) paquet introuvable via apt/nom binaire divergent, (2) préférences conservées après désinstallation, (3) bugs d'affichage dans les 8 dispositions — plus une demande de `.appimage` dans la release. Root causes et fixes détaillés dans `JOURNAL.md` (2026-08-01T23:20 → 2026-08-02T00:05). Résumé :
- Renommage `Cargo.toml`/`main.rs` (`tauri-app`→`nitrux`/`nitrux_lib`) + `productName` (`"NiTruX"`→`"Nitrux"`) → `apt list --installed` affiche `nitrux`, `.desktop` Exec/Icon/StartupWMClass cohérents, titre fenêtre reste "NiTruX".
- `packaging/nitrux-postrm-cleanup.sh` (nouveau, `postRemoveScript` deb+rpm) supprime `~/.local/share/org.heiphaistos.nitrux` seulement sur purge réel — vérifié en direct sur VM (remove préserve, purge supprime).
- Prop `variant` (`list`/`horizontal`/`icons`) sur `AppNav.vue`, choisie par `App.vue` selon `layoutStore.current` — remplace l'injection aveugle de la liste verticale catégorisée dans les 8 dispositions. Retrait du `display:grid` forcé sur le contenu de page dans `widgets-grid`/`bento` (cassait toute page non-dashboard).
- **AppImage toujours PAS résolu** : bundler Tauri exige `/usr/bin/xdg-open`, absent de l'environnement WSL2, pas de sudo sans mot de passe. Fichier extrait prêt à `/tmp/xdg-utils-extract/usr/bin/xdg-open` dans WSL2 — il suffit que l'utilisateur lance `! wsl.exe -e sudo cp /tmp/xdg-utils-extract/usr/bin/xdg-open /usr/bin/xdg-open` une fois pour débloquer les prochains builds AppImage.
- **Découverte infra** : capture d'écran pixel impossible sur la VM Debian 13 (Hyper-V) headless — `import`/`xwd` (X11 root sous Xwayland rootless) → `BadMatch X_GetImage` ; `spectacle` → KWin `CaptureScreen` D-Bus timeout (pas de backend screencast, `xdg-desktop-portal-kde` absent). Vérification UI faite via analyse CSS déterministe + tests jsdom + build réel installé (process confirmé), PAS de screenshot — si une vraie capture est nécessaire un jour, il faudra soit installer/configurer un backend screencast fonctionnel sur cette VM, soit accepter cette limite.
- Version 0.16.0→0.17.0, release `v0.17.0` publiée (deb+rpm ; AppImage à ajouter dès que débloqué).

## État antérieur (référence uniquement)

Les 4 piliers originaux (Phases 1 à 5 Part 2) et la refonte R1-R5 sont **terminés, publiés jusqu'à v0.13.0**. Tout est mergé sur `master` — voir historique git pour le détail phase par phase.

## Second round de refonte : retour utilisateur post-R5

Après v0.13.0, l'utilisateur a jugé le résultat toujours "moche et fade" et signalé qu'il manquait ~75% des fonctionnalités de NiTriTe Windows. **Découpage validé avec l'utilisateur en 6 phases R6→R11** (26 pages) — voir spec `2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1. Décisions de scope qui s'appliquent à TOUT le chantier R6-R11 : catégorie "Intelligence" hors scope, fonctionnalités Windows-only adaptées en équivalents Linux (pas ignorées), nouvelles surfaces privilégiées toujours vérifiées en live VM avant merge, nouvelles dépendances majeures (ex: PTY/xterm.js pour un terminal) nécessitent leur propre décision explicite plutôt qu'un ajout silencieux.

## Phase R6 (Fondation visuelle + Performance) : TERMINÉE, publiée en v0.14.0

Icônes lucide-vue-next réellement rendues dans `AppNav.vue`, dashboard componentisé + 5 tuiles colorées, catégorie "Performance" (4 pages). Tests : 128→147 frontend, 132→142 Rust.

## Phase R7 (Maintenance avancée) : TERMINÉE, publiée en v0.15.0

5 nouvelles pages Maintenance (Antivirus, Nettoyeur, Dépendances, Sauvegarde, Désinstalleur). Seule surface privilégiée nouvelle : action pkexec `uninstall-package`, miroir exact d'`install_package`, **testée en live sur la VM avec un vrai cycle install→uninstall** (paquet `sl`). Tests : 147→160 frontend, 142→165 Rust.

## Phase R8 (Réseau & Terminal) : TERMINÉE, mergée, publiée en v0.16.0

Spec : `docs/superpowers/specs/2026-08-01-nitrux-r8-reseau-terminal-design.md`. Plan : `docs/superpowers/plans/2026-08-01-nitrux-r8-reseau-terminal.md` (6 tâches). **Décision de scope clé** : Terminal intégré explicitement différé (nécessiterait une nouvelle dépendance PTY côté Rust + xterm.js côté frontend, les deux premières dépendances non-lucide de toute la refonte — pas ajoutées sans décision dédiée). 4 pages construites à la place :
- Task 1 : `WiFiAnalyzerPage.vue` (zéro backend, réutilise `get_network_snapshot`). **Sous-agent a trouvé et corrigé un vrai bug dans mon propre plan** : le test attendait `.wifi-ssid` = "HomeWifi" exact, mais le template du plan mettait le suffixe "(connecté)" dans le même span — corrigé en séparant dans un span sibling.
- Task 2 : `bluetooth.rs` (lecture seule, parse `bluetoothctl show`/`devices`) + `BluetoothPage.vue`. Aucun bug cette fois (sous-agent a tracé les tests à la main avant d'écrire, comme demandé après la découverte de Task 1).
- Task 3 : `run_script` (non-privilégié, `sh -c` avec les droits de l'utilisateur invocateur, PAS une nouvelle frontière de privilège) + `scriptsStore.ts` (Pinia+localStorage, miroir de `preferencesStore.ts`) + `ScriptsPage.vue`. **Sous-agent coupé par la limite de dépenses mi-tâche** (juste après avoir écrit `scripts.rs`, avant de l'enregistrer dans `lib.rs`) — reconnexion utilisateur confirmée, rien perdu, j'ai terminé Task 3 moi-même (enregistrement du module, store, page, 3 commits).
- Task 4 : `DnsSwitcherPage.vue` (réutilise `set_dns_servers` déjà privilégié et déjà vérifié, ajoute des préréglages Cloudflare/Google/Quad9 en un clic). Aucun bug trouvé.
- Task 5 : branchement `App.vue` + extension catégorie "reseau" (2→6 pages) + icônes `AppNav.vue` — fait directement par moi.
- Task 6 : vérification finale — 173/173 frontend, 172 Rust, `vue-tsc` clean. **Découverte réelle en direct sur la VM** : `bluetoothctl show`/`devices` ne renvoient pas une erreur immédiate quand le service Bluetooth est inactif — ils bloquent silencieusement jusqu'au timeout. Confirmé que le timeout de 5s déjà en place dans `subprocess::run_with_timeout` gère ça correctement (dégradation gracieuse vers `adapter_present: false`), donc aucun changement de code nécessaire, juste une confirmation que le comportement observé correspond à ce que le code attend déjà.

Tests : 160→173 frontend, 165→172 Rust.

Merge master (fast-forward propre) + version bump 0.15.0→0.16.0 + build réel (.deb/.rpm) + Cargo.lock committé — **push+tag+release restent à faire juste après ce checkpoint**.

## Prochaine action

Pousser master + créer le tag `v0.16.0` + créer la release GitHub avec les assets `.deb`/`.rpm`. Nettoyer le worktree `r8-reseau-terminal`.

Puis écrire spec+plan pour **Phase R9 (Stockage avancé)** : Récupération de données, Visualiseur de disque, Boot Manager, Clonage dédié, Restauration dédiée (promotion depuis DisksPage). Continuer le découpage R9→R11 déjà validé.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.16.0`
- Worktrees mergés à nettoyer si encore présents : `r8-reseau-terminal`
- **Découpage validé R6→R11** — spec `2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1. R6, R7, R8 faits. Reste R9 (Stockage avancé), R10 (Logiciels & déploiement), R11 (Diagnostic & config).
- **Terminal intégré différé de R8, toujours en attente d'une décision dédiée** — nécessite une nouvelle dépendance Cargo (PTY, ex: `portable-pty`) + probablement une dépendance npm (rendu ANSI, ex: `xterm.js`), à traiter séparément avec sa propre revue de conception si l'utilisateur le souhaite un jour.
- VM Debian : `172.18.32.124`, user `dev`, password `1998`. Scripts SSH : `C:\Users\Momo\AppData\Local\Temp\claude\C--Users-Momo\880690b1-319b-40bd-bb2c-957700dc8af4\scratchpad\ssh_run.py`/`ssh_put.py`/`ssh_interactive.py` (usage `python ssh_run.py <user> <password> "<cmd>"`).
- **Piège commande VM qui bloque silencieusement (R8)** : certaines commandes système (ex: `bluetoothctl show` quand le service est inactif) ne retournent pas d'erreur immédiate — elles bloquent jusqu'à ce qu'on les tue. Toujours wrapper une commande de vérification VM inconnue avec `timeout N <cmd>` pour éviter un SSH qui pend indéfiniment (confirmé : sans `timeout`, la commande a fait planter la connexion paramiko par timeout de socket).
- **Pattern de vérification VM pour une nouvelle action pkexec** (établi en R7, à réutiliser pour toute future action privilégiée) : `pkttyagent --process $$ & sleep 1; pkexec /usr/bin/nitrux-pkexec-<action> <sous-commande> <args>` via `ssh_interactive.py` — la ligne `==== AUTHENTICATING FOR org.heiphaistos.nitrux.<action> ====` confirme que polkit a résolu la bonne action distincte.
- **Piège pkill (R6)** : `pkill -f <motif>` peut tuer son propre shell invocateur. Toujours `pkill -x <nom-exact>`.
- Lancement app sur VM : `DISPLAY=:1 WAYLAND_DISPLAY=wayland-0 XAUTHORITY=/run/user/1000/xauth_* DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus XDG_RUNTIME_DIR=/run/user/1000 nohup /usr/bin/tauri-app > /tmp/log 2>&1 &` — re-dériver via `systemctl --user show-environment` si besoin.
- **Piège pkexec — discipline non-négociable pour toute nouvelle action privilégiée** : chemin `exec.path` dédié jamais partagé, re-validation shell indépendante, cycle complet testé en live sur la VM avant merge.
- **Toujours tracer les assertions de test contre le template du composant à la main avant de l'écrire** (leçon R8 Task 1, confirmée utile en Task 2/4 où aucun bug n'a été trouvé grâce à cette vérification) — les plans peuvent contenir un vrai bug de cohérence test/composant, pas juste des erreurs de comptage.
- Pattern "unhandled rejection bénin dans App.spec.ts" (vu R2/R4/R5/R6/R7/R8) : `ref.value = await invoke<T[]>(...)` puis accès sans garde null sous le mock global `invoke→null` — n'affecte aucun résultat de test.
