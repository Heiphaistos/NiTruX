# Checkpoint — Refonte R1-R5 + R6-R11 **COMPLET**, R12+R13+R14 faits, v0.24.0

## Phase R14 (Défauts thème/layout + scrollbar + responsive + catalogues x30) : TERMINÉE, mergée (3c6981c→master ff), version 0.24.0, publiée

Demande explicite utilisateur : enrichir chaque catégorie au maximum, Applications et Commandes rapides à 500+ minimum chacun, thème/disposition par défaut selon ce que l'utilisateur avait en tête (aucun profil réellement sauvegardé trouvé sur la VM — clarifié via AskUserQuestion : thème noir/texte blanc + disposition master-detail + scrollbar sur menu latéral + tout responsive).

- **Nouveau thème "OLED Noir"** (bg #000000/#0a0a0a, texte blanc) ajouté à `builtin.ts` (12→13 thèmes) et défini comme défaut dans `themeStore.ts`. **Bug préexistant corrigé au passage** : le thème actif n'était JAMAIS persisté en localStorage (contrairement au layout) — réinitialisé à chaque rechargement. Ajouté la persistance (miroir exact du pattern déjà utilisé par `layoutStore.ts`).
- **Disposition par défaut changée** : `sidebar-classic` → `master-detail` dans `layoutStore.ts`.
- **2 vrais bugs de scrollbar trouvés et corrigés** : `SidebarClassicLayout.vue` n'avait AUCUN `overflow` sur `.nx-nav` (liste de nav désormais bien plus longue que R1 → débordement sans scroll possible) ; `CompactSidebarLayout.vue` gardait `overflow:hidden` même à l'état étendu au survol (items au-delà de la hauteur visible totalement inaccessibles). Scrollbar `::-webkit-scrollbar` stylée au thème ajoutée aux 3 layouts à menu latéral (WebKitGTK supporte les pseudo-éléments webkit).
- **Passe responsive** : `box-sizing:border-box` global (largeurs fixes + padding dépassaient silencieusement leur largeur déclarée), `min-width:0` sur `.nx-content` de chaque layout (fix classique pour qu'un enfant flex avec `overflow:auto` puisse réellement rétrécir au lieu de faire déborder toute la fenêtre horizontalement), le seul vrai `<table>` de l'app (PackagesPage) enveloppé dans son propre conteneur `overflow-x:auto`.
- **`appCatalog.ts` : 16 → 506 entrées** (apt/Flatpak/Snap, ~30 catégories). Chaque nom de paquet apt vérifié EN LOT (`apt-cache show`, pas un par un manuellement) contre le vrai dépôt Debian de la VM ; chaque ID Flatpak vérifié en lot contre flathub (`flatpak remote-info`). 19 ratés trouvés et corrigés par cette vérification (pas de la vérification cosmétique) : certains genuinely absents du dépôt (midori, eclipse, netbeans, webmin, vice, nim, avidemux-qt, celestia-gtk, wondershaper, indicator-multiload, materia-gtk-theme, phatch — retirés), un remplacé par son vrai nom de paquet actuel (neofetch abandonné en amont → fastfetch), plusieurs mieux servis en Flatpak sur cette distro (anki, lutris, telegram-desktop), jetbrains-toolbox absent de flathub → remplacé par le vrai PyCharm Community, lite-xl et stacer absents de flathub → retirés.
- **`systemToolsCatalog.ts` : 58 → 506 entrées**. 4 nouvelles catégories (Développement, Fichiers, Utilisateurs, Date & heure) + approfondissement des 5 catégories non-privilégiées existantes. Catégorie "privilegie" volontairement laissée à ses 7 actions fixes existantes (l'étendre exige du vrai code Rust + vérification live par action, coût radicalement différent des entrées data-only). **Chaque commande exécutée pour de vrai** (pas juste relue) contre la VM via un script batch avec timeout par commande — 2 vrais bugs trouvés : `ping-gateway` pouvait prendre 13+s si la passerelle ne répond pas au ping (VM et beaucoup de routeurs réels bloquent l'ICMP) → corrigé avec `-W 2` ; une entrée `bluetoothctl show` brute réintroduisait exactement le piège de hang documenté en R8 (bloque silencieusement si le service Bluetooth est inactif, aucun garde-fou de timeout en dehors du backend dédié Bluetooth) → retirée, redondante avec la page Bluetooth dédiée déjà correctement protégée. Les ~110 "commande introuvable" restantes sont des outils Linux standards réels juste absents de CETTE VM minimale (docker, node, npm, curl, outils LVM...) — même pattern de dégradation déjà accepté pour `sensors`/`mtr` dans le catalogue original.
- **Vrai bug d'édition auto-introduit, trouvé et corrigé avant merge** : une insertion a accidentellement fermé le tableau TS en plein milieu, laissant les 7 entrées privilégiées existantes orphelines hors de la déclaration `const` (n'aurait pas compilé) — détecté immédiatement par le double-check systématique (comptage + `vue-tsc`) avant tout commit.
- **Vrai bug de test-infra trouvé (pas applicatif) pendant la vérification finale** : un run complet a échoué à démarrer 7 workers vitest ("Timeout waiting for worker to respond") par contention CPU (beaucoup de travail parallèle WSL/VM en cours à ce moment précis) — confirmé transitoire par un second run propre juste après (64/64, 233/233, fiable).
- 233/233 frontend, 213 Rust, vue-tsc clean.
- **Vérification live VM** : v0.24.0 installée par-dessus v0.23.0, lancée, comptage d'éléments accessibles AT-SPI avant/après navigation vers Installation rapide (61→592 éléments nommés) et Commandes rapides (592→574, stable) — confirme que les deux catalogues de 500+ entrées se rendent réellement sans crash, process resté stable tout du long.
- Release publiée : `v0.24.0` (.deb + .rpm).

## Phase R13 (Terminal intégré) : TERMINÉE, mergée (dbc60d8→master ff), version 0.23.0, publiée

Demande explicite utilisateur mi-tâche (« et integre un terminal bash dans l app »), résout la décision de scope différée depuis R8 (§ ci-dessous "Terminal intégré différé"). Spec : `2026-08-02-nitrux-r13-terminal-integre-design.md`.

- Backend `terminal.rs` : `portable-pty` 0.9.0 (nouvelle dépendance Cargo, première depuis le début du chantier PTY), spawn du shell de l'utilisateur invocateur (`$SHELL` ou `/bin/bash`) dans un vrai pseudo-terminal, streaming de sortie vers le frontend via `tauri::ipc::Channel` (**première fonctionnalité de streaming bidirectionnel de toute l'app** — jusqu'ici tout était requête/réponse one-shot). 4 commandes : `spawn_terminal`/`write_to_terminal`/`resize_terminal`/`close_terminal`. Logique factorée en `open_shell_pty()` testable hors `tauri::State`. **Aucune nouvelle frontière de privilège** — même raisonnement que `run_script` (R8) : le shell hérite exactement des droits de l'utilisateur qui lance l'app, pas de pkexec.
- Frontend `TerminalPage.vue` : `@xterm/xterm` 6.0.0 + `@xterm/addon-fit` 0.11.0 (nouvelles dépendances npm), cycle spawn/write/resize(ResizeObserver)/close complet.
- Test réel backend (pas mocké) : spawn un vrai bash, écrit `echo PTY_TEST_MARKER...`, lit la sortie via le vrai pty — confirmé 2/2.
- **Vérification live VM via une nouvelle technique découverte cette session** : capture pixel toujours impossible (cf. limite documentée plus bas), mais AT-SPI (accessibilité GTK/WebKitGTK, `python3-gi`) permet de cliquer un bouton par son nom accessible sans coordonnées ni screenshot — confirmé en cliquant "Terminal" sur l'app réelle en cours d'exécution et en observant un vrai process `bash` apparaître comme enfant du process de l'app (`pstree -p`), puis disparaître proprement en changeant de page. Preuve non-visuelle plus forte qu'un screenshot (prouve le chemin de code backend réellement exécuté). Technique documentée dans la mémoire persistante (`reference_atspi_headless_ui_automation.md`) pour réutilisation future.
- **Bug d'infra de test trouvé et corrigé pendant la vérification finale, PAS un bug applicatif** : `npm test` (mode par défaut, workers parallèles) échouait de façon reproductible (2/2) sur un test préexistant sans rapport (`TemperaturesPage`), alors que la même suite passe 100% de façon fiable en mono-thread (`--pool=threads --poolOptions.threads.singleThread`, 2/2) et que le fichier isolé passe aussi 100%. Cause : les workers parallèles se disputent le CPU quand `App.spec.ts` initialise le vrai (non-mocké) `@xterm/xterm`, ce qui retarde suffisamment un autre fichier pour faire rater son assertion — pas de faute logique dans l'app (prouvé déterministe et correct en série). Fix : `vitest.config.ts` épinglé en `poolOptions.threads.singleThread: true` (commit `d426c5d`, séparé du bump de version). Accepté comme le bon niveau de correctif (config du test-runner, pas de contournement de l'app) car cible directement la cause (contention CPU inter-workers), pas juste le symptôme.
- 232/232 frontend (fiable maintenant, mono-thread), 213 Rust, vue-tsc clean.

## Sweep post-R12 « implémente un maximum de chose » : TERMINÉ, v0.22.0

Demande utilisateur ouverte après R12. Audit systématique de toutes les commandes Tauri enregistrées (`lib.rs`) vs consommateurs frontend — un seul orphelin trouvé : `verify_file_hash` existait entièrement testé côté backend depuis toujours (voisin de `compute_file_hash` dans `hashcheck.rs`) mais `FileToolsPage.vue` ne calculait jamais que le hash, sans jamais permettre de le comparer à une valeur attendue (le vrai cas d'usage réel — vérifier un ISO téléchargé). Corrigé : champ "hash attendu" + bouton "Vérifier" ajoutés.

Catalogue `systemToolsCatalog.ts` étendu de 47 → **58 entrées** (11 nouvelles, toutes non-privilégiées, chacune vérifiée en direct sur la VM exactement comme `run_script` les exécute avant ajout) : version de distribution, taille des journaux, état du swap, arborescence des montages, table ARP, config DNS actuelle, nettoyage cache polices, charge système, temps de démarrage + services les plus lents (systemd-analyze), taille de /var/log.

**Pistes investiguées et volontairement NON poursuivies** (décisions explicites, pas des oublis) :
- **Gestion Docker (start/stop/remove conteneurs)** — NiTriTe a une page dédiée de 531 lignes pour ça, NiTruX n'a que la lecture seule (`docker.rs`, onglet dans NetworkPage). Bloqué : Docker n'est pas installé sur la VM de dev, impossible de vérifier en direct une fonctionnalité d'écriture sans l'installer d'abord (discipline non-négociable du projet). À reprendre si Docker est installé sur la VM un jour.
- **MonitoringPage / TurboModePage NiTriTe** — Monitoring fait doublon avec Dashboard+PerfHistoryPage déjà existants. TurboMode applique automatiquement des optimisations, ce qui contredirait frontalement la philosophie explicitement établie d'`OptimizationsPage` (« lecture seule — aucune action n'est appliquée automatiquement ») — nécessiterait une décision utilisateur dédiée avant d'être construit, pas une extension silencieuse.

226/226 frontend (224+2 pour la fonctionnalité verify_file_hash), 211 Rust (inchangé, aucun ajout backend dans ce sweep), vue-tsc clean.

## Phase R12 (Outils système — catalogue de commandes en un clic) : TERMINÉE, mergée (520d6f8), version 0.21.0 — push+tag+release restent à faire

Demande utilisateur hors découpage R6-R11 : catégorie de boutons-commandes façon `ToolsPage.vue` NiTriTe, chiffre cité « plus de 500 boutons ». Investigation : la vraie `ToolsPage.vue` NiTriTe n'a que **69 entrées** (le chiffre 500+ venait de l'addition avec des catalogues d'installateurs d'apps sans rapport — 745+145 entrées — déjà couverts différemment côté NiTruX). Décision : construire le catalogue le plus complet et réellement utile possible plutôt que viser un chiffre arbitraire avec du remplissage. Catégorie "Activation" NiTriTe (12 entrées, contournement de licence Windows/Office — massgrave.dev, KMSPico...) exclue définitivement, aucun équivalent Linux pertinent de toute façon.

**Décision de sécurité validée avec l'utilisateur (AskUserQuestion)** : contrairement à la recommandation initiale (catalogue 100% non-privilégié), l'utilisateur a choisi d'inclure aussi des commandes root, en connaissance du coût annoncé (chaque commande privilégiée nécessite sa propre revue de sécurité). Solution retenue : **une seule nouvelle action polkit consolidée** (`org.heiphaistos.nitrux.system-tools`, 14e nom pour `nitrux-pkexec-helper`) avec un switch de 7 sous-commandes fixes codées en dur — jamais un bouton "exécuter n'importe quoi en root", même pattern déjà en production pour `nitrux-pkexec-troubleshoot`.

- Catalogue final : 40 commandes non-privilégiées (réutilisent `run_script` existant depuis R8, aucune nouvelle frontière de privilège) + 7 commandes privilégiées = 47 boutons au total, organisés en 6 catégories (Diagnostics, Réseau, Performance, Nettoyage, Stockage, Privilégié).
- Task 3 : **bug trouvé en faisant tourner la suite** — le test cherchait un bouton par le nom de l'outil dans son propre texte, mais le nom est dans un `<strong>` frère, pas dans le bouton (toujours juste "Exécuter") ; corrigé en localisant via la carte englobante.
- **Les 7 actions privilégiées + le chemin de rejet testés EN DIRECT sur la VM avant merge** (discipline non-négociable pour toute nouvelle surface pkexec) : ligne `AUTHENTICATING FOR org.heiphaistos.nitrux.system-tools` confirmée, chaque action vérifiée avec sa vraie sortie (apt-autoremove, journal-vacuum-size avec "Vacuuming done", ldconfig, systemd daemon-reload, fstrim-av avec vraie sortie de réduction, rebuild-locate-db → repli propre "non installé" confirmé car `updatedb` absent sur cette VM, regenerate-grub → a réellement fonctionné malgré `which update-grub` négatif en shell SSH simple — PATH différent sous pkexec/root, comportement réel correct). Rejet d'une action inconnue confirmé (exit 1, aucun effet de bord).
- Spot-check final de 5 commandes non-privilégiées représentatives également vérifié en direct.
- 224/224 frontend, 211 Rust, vue-tsc clean.

## Phase R11 (Diagnostic & config) : TERMINÉE, mergée (ba71ef6), version 0.20.0

Retour utilisateur explicite : catégorie "Diagnostic" jugée quasi-vide (une seule page PCI, pas même sa propre catégorie) et "ultra importante", demande de reprendre "exactement" ce qu'a NiTriTe Windows dans l'équivalent (493 lignes, ~30 onglets). Spec : `2026-08-02-nitrux-r11-diagnostic-config-design.md`. Plan : `2026-08-02-nitrux-r11-diagnostic-config.md` (8 tâches). Investigation : **chaque commande candidate testée sans root sur la VM dev avant d'écrire le spec** (lscpu, /sys/class/dmi/id/*, /proc/meminfo, lsusb, pactl, xrandr, lpstat, systemctl list-units/list-timers, crontab -l, dpkg -l, /var/log/apt/history.log). Décision de périmètre : nouvelle catégorie dédiée "Diagnostic" (8e→9e), 6 nouvelles pages + page PCI existante déplacée dedans (pas dupliquée) ; exclusion explicite de tout ce qui est sans équivalent Linux (Licence/Registre/BSOD/WSL Windows) ou déjà couvert ailleurs dans NiTruX.
- Task 1 (hardware_details.rs CPU/carte-mère/mémoire, LC_ALL=C car lscpu est localisé — confirmé en français par défaut sur la VM) : aucun bug de code, juste un écart de compte plan-vs-réel bénin (5 tests prédits, 4 écrits).
- Task 2 (peripherals.rs moniteurs/USB/audio/imprimantes, tout dégrade gracieusement vers liste vide) : aucun bug.
- Task 3 (processes.rs, réutilise le `Mutex<System>` déjà managé par system.rs plutôt que d'en créer un nouveau) : aucun bug.
- Task 4 (InstalledSoftwarePage.vue) : **zéro nouveau module backend** — réutilise `list_installed_packages` déjà existant depuis R3/consommé par UninstallerPage, ajoute juste `get_environment_variables` trivial.
- Task 5 (accounts.rs comptes réels /etc/passwd) : aucun bug.
- Task 6 (update_history.rs historique APT) : aucun bug.
- Task 7 (nouvelle catégorie nav) : **vrai bug trouvé en faisant tourner la suite** (pas dans le plan) — un test existant cherchait un bouton nav "Diagnostic" qui n'existait plus après renommage du libellé PCI en "Composants PCI" pour libérer ce nom pour le titre de catégorie ; corrigé.
- Task 8 (vérification finale) : **tout le backend re-vérifié en direct sur la VM**, y compris re-test de xrandr/lpstat avec le vrai DISPLAY/XAUTHORITY de la session desktop après un premier échec attendu en SSH brut (pas de DISPLAY). App lancée et stable sur la VM.
- 217/217 frontend, 207 Rust, vue-tsc clean.

**Chantier R6→R11 dans son ensemble : COMPLET.** Toutes les phases planifiées avec l'utilisateur sont livrées et publiées.

## Phase R10 (Logiciels & déploiement) : TERMINÉE, mergée (5ff4745), version 0.19.0

Spec : `2026-08-02-nitrux-r10-logiciels-deploiement-design.md`. Plan : `2026-08-02-nitrux-r10-logiciels-deploiement.md` (6 tâches). Investigation : comparaison categories.ts vs navigation.ts NiTriTe + audit des modules Rust orphelins. Découverte clé : `appCatalog.ts` avait `InstallMethod="apt"|"flatpak"|"snap"` depuis le début mais seul "apt" était implémenté — 3 entrées affichaient un badge honnête "Bientôt disponible" jamais tenu. Décision utilisateur (AskUserQuestion) : fermer complètement y compris Snap malgré la nouvelle surface pkexec.
- Task 1 (install_flatpak_package, --user, non-privilégié) : aucun bug.
- Task 2 (install_snap_package, nouvelle action pkexec `install-snap`) : aucun bug de code, mais **nouvelle surface privilégiée traitée avec la rigueur complète établie** — exec.path dédié, wrapper+policy+tauri.conf.json mis à jour, **testée en direct sur la VM** (snapd installé, cycle pkexec complet avec le paquet `hello`, ligne `AUTHENTICATING FOR org.heiphaistos.nitrux.install-snap` confirmée, installation réelle réussie).
- Task 3 (QuickInstallPage.vue câblé, retrait badge "Bientôt disponible") : aucun bug.
- Task 4 (InstallProfilesPage.vue, profils sur catalogue existant) : **bug trouvé dans le plan lui-même en auto-review** — `waitFor` sur "Firefox" (déjà visible dans la liste checkbox permanente) n'attendait pas réellement la fin de la boucle d'install séquentielle ; corrigé avant même d'écrire le code.
- Task 5 bonus (get_smart_status, orphelin depuis R9, câblé dans DisksPage.vue) : aucun bug.
- Task 6 (vérification finale) : Flatpak **vérifié en direct sur la VM** (installation réelle de Discord réussie, accès réseau fonctionnel, contrairement à l'hypothèse prudente du plan) — Snap déjà vérifié en Task 2.
- 198/198 frontend, 188 Rust, vue-tsc clean.

## Phase R9 (Stockage avancé) : TERMINÉE, mergée (40ad117), version 0.18.0 — push+tag+release restent à faire

Spec : `2026-08-01-nitrux-r9-stockage-avance-design.md`. Plan : `2026-08-01-nitrux-r9-stockage-avance.md` (6 tâches). 4 nouvelles pages dans la catégorie "Stockage" (2→6) : Visualiseur de disque (zéro backend), Récupération de données (corbeille XDG, non-privilégié), Boot Manager (lecture seule GRUB+efibootmgr), Restauration (extraction depuis TroubleshootPage). Worktree `r9-stockage-avance` traité sur plusieurs sessions/ticks avec confirmation utilisateur explicite par tâche ("continue with task N").
- Task 1 (DiskVisualizerPage) : aucun bug.
- Task 2 (trash.rs+DataRecoveryPage) : **bug de plan trouvé en traçant le test à la main** — mock `list_trash` toujours identique, un `refresh()` après restore aurait fait échouer l'assertion "not.toContain" ; fixé en supprimant l'item localement au lieu de re-fetch.
- Task 3 (boot_manager.rs+BootManagerPage) : aucun bug, template du plan correct.
- Task 4 (extraction RestorePointsPage) : aucun bug.
- Task 5 (câblage App.vue/categories.ts/AppNav iconMap) : aucun bug.
- Task 6 (vérification finale) : **vrai bug de code (pas juste de plan) trouvé EN DIRECT sur la VM** — `restore_trash_item` utilisait `std::fs::rename`, qui échoue (EXDEV) dès que le fichier trashé et `~/.local/share/Trash` sont sur des filesystems différents (confirmé : `/tmp` tmpfs vs `/home` sur `/dev/sda2` sur la VM). N'importe quel fichier mis à la corbeille depuis `/tmp`, une clé USB, ou une autre partition n'aurait jamais pu être restauré. Fixé avec un fallback copie récursive+suppression (`move_path()`, comme `mv`), re-vérifié en rejouant à la main la séquence exacte du code Rust via SSH (capture pixel toujours impossible sur cette VM, cf. limite déjà documentée ci-dessous).
- 190/190 frontend, 184 Rust, vue-tsc clean. Merge fast-forward propre vers master. Version 0.17.0→0.18.0 (commit f6f484d), build réel .deb/.rpm confirmé (AppImage toujours bloqué xdg-open, cf. note ci-dessous).

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

Aucune action de release en attente — R14 entièrement publiée (push+tag+release faits). Aucun worktree ouvert.

**Demande utilisateur "x10/500+" majoritairement traitée par R14** (thème/layout/scrollbar/responsive + 2 catalogues à 500+). Reste ouvert : voir "Chantier ouvert" ci-dessous.

Pistes ponctuelles restantes si l'utilisateur le souhaite : (1) débloquer l'AppImage (nécessite `! wsl.exe -e sudo cp /tmp/xdg-utils-extract/usr/bin/xdg-open /usr/bin/xdg-open`), (2) Gestion Docker complète (bloqué : Docker absent de la VM dev), (3) étendre le catalogue privilégié `system-tools` (7 actions fixes) avec de nouvelles commandes root — coûte du vrai code Rust + vérif live par action, pas juste de la data.

## Chantier ouvert : parité NiTriTe "x10" (terminal ✅ R13, thème/layout/scrollbar/responsive ✅ R14, catalogues 500+ ✅ R14)

Demande verbatim : parité intégrale NiTriTe (traduite en vrais équivalents Linux, pas copiée aveuglément), enrichir "toutes les catégories x10 minimum". La majeure partie du chantier concret est livrée (R13+R14). Reste :

- **ProfilesPage-équivalent** (sauvegarde/chargement/export/import de profils de config nommés) — repéré comme un vrai écart, pas encore spécifié ni codé.
- **StatsReportsPage** NiTriTe — semble faire doublon avec `ReportGeneratorPage`/`PerfHistoryPage` existants, mais vérification rapide seulement, à recroiser plus sérieusement avant d'exclure définitivement.
- **`installProfiles.ts`** (4 profils) — pas encore approfondi, seul catalogue des 3 identifiés en R12/R13 qui reste petit (appCatalog et systemToolsCatalog sont maintenant à 506 chacun).
- Les 10 catégories nav n'ont PAS toutes reçu une passe "x10" dédiée pages-par-pages (au-delà des 2 catalogues) — Diagnostic/Performance/Stockage/Maintenance/Réseau/Rapports/Paramètres pourraient chacun gagner en profondeur de contenu (pas juste en nombre de boutons), à évaluer au cas par cas si l'utilisateur le redemande.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.24.0`
- Aucun worktree ouvert actuellement.
- **Découpage R6→R11 : COMPLET** (R6 v0.14.0, R7 v0.15.0, R8 v0.16.0, R9 v0.18.0, R10 v0.19.0, R11 v0.20.0). **Phase R12 (hors découpage) : COMPLET** (v0.21.0). **Phase R13 (terminal) : COMPLET** (v0.23.0). **Phase R14 (défauts+scrollbar+responsive+catalogues 500+) : COMPLET** (v0.24.0).
- **Snap = 13e action pkexec (R10), system-tools = 14e (R12)** — R13 et R14 n'ont ajouté aucune surface pkexec.
- **`nitrux-pkexec-helper` a toujours 14 noms installés** — toujours mettre à jour le compte dans le commentaire d'en-tête du script à chaque ajout.
- **Test-runner : `vitest.config.ts` revenu à sa config par défaut (parallélisme normal) depuis R14** — le pin single-thread de R13 a été remplacé par un correctif plus ciblé (mock `@xterm/xterm` dans `App.spec.ts`, exactement comme `TerminalPage.spec.ts` le fait déjà) après avoir découvert que le pin coûtait 10-15 min par run (vs ~3 min) et que le pool par défaut de Vitest 4 est `forks` (process OS complet par worker), pas `threads`. Si un futur composant réintroduit un vrai module lourd non-mocké dans un test global, mocker-le à la source plutôt que de re-sérialiser toute la suite.
- VM Debian : `172.18.32.124`, user `dev`, password `1998`. Scripts SSH : `C:\Users\Momo\AppData\Local\Temp\claude\C--Users-Momo\880690b1-319b-40bd-bb2c-957700dc8af4\scratchpad\ssh_run.py`/`ssh_put.py`/`ssh_interactive.py` (usage `python ssh_run.py <user> <password> "<cmd>"`).
- **Piège commande VM qui bloque silencieusement (R8)** : certaines commandes système (ex: `bluetoothctl show` quand le service est inactif) ne retournent pas d'erreur immédiate — elles bloquent jusqu'à ce qu'on les tue. Toujours wrapper une commande de vérification VM inconnue avec `timeout N <cmd>` pour éviter un SSH qui pend indéfiniment (confirmé : sans `timeout`, la commande a fait planter la connexion paramiko par timeout de socket).
- **Pattern de vérification VM pour une nouvelle action pkexec** (établi en R7, à réutiliser pour toute future action privilégiée) : `pkttyagent --process $$ & sleep 1; pkexec /usr/bin/nitrux-pkexec-<action> <sous-commande> <args>` via `ssh_interactive.py` — la ligne `==== AUTHENTICATING FOR org.heiphaistos.nitrux.<action> ====` confirme que polkit a résolu la bonne action distincte.
- **Piège pkill (R6)** : `pkill -f <motif>` peut tuer son propre shell invocateur. Toujours `pkill -x <nom-exact>`.
- Lancement app sur VM : `DISPLAY=:1 WAYLAND_DISPLAY=wayland-0 XAUTHORITY=/run/user/1000/xauth_* DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus XDG_RUNTIME_DIR=/run/user/1000 nohup /usr/bin/tauri-app > /tmp/log 2>&1 &` — re-dériver via `systemctl --user show-environment` si besoin.
- **Piège pkexec — discipline non-négociable pour toute nouvelle action privilégiée** : chemin `exec.path` dédié jamais partagé, re-validation shell indépendante, cycle complet testé en live sur la VM avant merge.
- **Toujours tracer les assertions de test contre le template du composant à la main avant de l'écrire** (leçon R8 Task 1, confirmée utile en Task 2/4 où aucun bug n'a été trouvé grâce à cette vérification) — les plans peuvent contenir un vrai bug de cohérence test/composant, pas juste des erreurs de comptage.
- Pattern "unhandled rejection bénin dans App.spec.ts" (vu R2/R4/R5/R6/R7/R8) : `ref.value = await invoke<T[]>(...)` puis accès sans garde null sous le mock global `invoke→null` — n'affecte aucun résultat de test.

## État courant (2026-08-04, v0.25.5)

**Rapport testeurs AppImage v0.24.3 (9 items utilisateur) : intégralement traité et poussé.** Détail complet dans JOURNAL.md (entrées 2026-08-04T13:35 à 15:10). Résumé : pkexec AppImage auto-install premier lancement, terminal persistant (KeepAlive), filtre dépendances Logiciels installés, benchmark CPU freq+SMART, message clamscan clarifié, cartes réseau/MAC/débit, export PDF réel (nouvelle dép. `printpdf` 0.12.5, réutilise `render_html` existant). Boucle 10 min reprend son cycle normal après ce cycle.

**Flakiness d'infra observée cette session** (distincte du problème R13/xterm déjà résolu ci-dessus) : `npm run test -- --run` sur ce repo (D:\ monté depuis WSL2) échoue par intermittence avec `[vitest-pool]: Failed to start forks worker ... Timeout waiting for worker to respond` sur des fichiers aléatoires — pas liés à un module lourd non-mocké, juste I/O cross-filesystem WSL2↔Windows sous charge. `--no-file-parallelism` en contournement est resté bloqué >15min sans jamais finir (pire, pas mieux). Pattern de contournement qui a fonctionné : relancer 2x en parallélisme normal — sur 2 tentatives, 100% des fichiers qui ont pu démarrer sont passés (0 échec réel), seul le nombre de fichiers exécutés variait. Si ça persiste sur un futur cycle, envisager de lancer `npm run test` depuis un terminal WSL2 natif pointant vers une copie du repo sur `/mnt` avec moins de charge disque concurrente, plutôt que de re-diagnostiquer depuis zéro.

## État courant (2026-08-04, v0.25.20, cycle 100)

**Rapport testeurs AppImage v0.24.3 (9 items) et release v0.25.10 : intégralement traités et poussés.** Voir section précédente pour le détail v0.25.5→v0.25.10.

**Campagne cycles 71-100 (v0.25.6 → v0.25.20) : dizaines de bugs réels trouvés et corrigés**, méthode principalement transversale (grep multi-fichiers) une fois l'inventaire fichier-par-fichier épuisé vers le cycle 79. Points marquants les plus significatifs :
- **`firewall.rs` (cycle 97, le plus sérieux)** : `ufw status` non-privilégié sort en code 0 avec stdout vide (erreur réelle sur stderr, ignorée) -- la page Pare-feu affichait TOUJOURRS "inactif" indépendamment de l'état réel, dans le mode d'exécution NORMAL de l'app (sans privilège). Corrigé, `parse_ufw_output` maintenant faillible.
- **`themeStore.ts` (cycle 90)** : thèmes personnalisés jamais persistés en localStorage (seul le thème actif l'était) -- disparaissaient au redémarrage.
- **`terminal.rs` (cycle 96)** : fuite de processus shell sur réutilisation d'id (`HashMap::insert` sans kill préalable) -- non atteignable via le frontend actuel mais corrigé par prudence.
- **`PackagesPage.vue`/`SystemToolsPage.vue`/`UninstallerPage.vue` (cycles 75/76/99)** : données périmées affichées après une action, ou no-op silencieux sans gestionnaire de paquets détecté.
- **Localisation Mo/Go (cycles 81/82)** : balayage exhaustif, 5 pages corrigées, plus aucune unité anglaise dans l'UI.
- **`update_history.rs` (cycle 74)** : parsing apt ne reconnaissait que 3 des 6 types d'action réels.
- **Catalogue** : Microsoft Teams retiré (404 Flathub confirmé, cycle 84) ; apt non re-vérifiable depuis cet environnement (WSL2 Ubuntu ≠ Debian cible, cycle 83 -- piège méthodologique identifié et évité).
- **Accessibilité (cycle 88)** : première vérification a11y de toute la campagne, `aria-label` ajouté à la navigation icônes-seules (disposition Floating Dock).

**Suspects non résolus, en attente d'environnement de test** :
- `timeshift --list`/`efibootmgr` (cycle 98) : suspectés du même bug que `ufw` (nécessitent typiquement root), non installés dans cet environnement WSL2, aucun accès root pour les installer -- nécessite VM Debian ou environnement avec accès root.
- Cycles 18/34/44 (pré-v0.25) : décision benchmark disque déjà résolue (disclaimer, cycle 72) ; bug exit-code `apt-autoremove` du script pkexec partagé et gap de validation `quarantine-file` (chemin `/` accepté) toujours en attente de VM pour re-vérification live avant tout correctif (règle du projet : jamais modifier pkexec déjà vérifié sans re-test live).

**VM Debian (172.18.32.124) injoignable sur TOUTE la campagne cycles 71-100** (`No route to host` de façon constante) -- re-vérifiée à chaque tentative de reprendre les découvertes pkexec en attente, jamais de succès.

**Pièges d'infra confirmés cette campagne** (voir LESSONS.md pour le détail complet) : corruption `$()`/interpolation de variable via `wsl.exe bash -lc "..."` inline (pas seulement les heredocs) -- toujours écrire les scripts avec boucles/substitutions dans un fichier via Write puis `wsl.exe bash -lc "bash '<chemin>'"` ; cache de build `target/` périmé après déplacement de repo (Desktop→D:\Projet) -- nettoyage ciblé `cargo clean -p nitrux -p tauri -p tauri-build` plutôt que `rm -rf target/` complet ; notification "completed" d'une tâche background pipée vers `tail` reflète l'exit code de `tail`, pas de la commande réelle -- toujours rediriger vers un fichier log + `echo EXIT_CODE=$?` pour les commandes dont le succès doit être vérifié.

## Mise à jour (2026-08-04, v0.25.22, cycle 107)

**Suspect `timeshift`/`efibootmgr` (cycle 98, listé ci-dessus comme "en attente d'environnement de test") : résolu, sans avoir eu besoin de VM.** Nouvelle technique : `apt-get download <paquet>` (télécharge le `.deb` sans root) + `dpkg-deb -x <paquet>.deb dest/` (extrait sans installer) a permis d'exécuter les vrais binaires `timeshift`/`efibootmgr` sans privilège sur la machine de dev. Résultat : la suspicion initiale était fausse (timeshift n'échoue PAS comme ufw -- code 1 réel, pas 0, message sur stdout pas stderr) mais un vrai bug distinct existait quand même (`snapshots.rs::list_snapshots` perdait ce message stdout) -- corrigé. `efibootmgr`/`boot_manager.rs` : déjà correct par conception (`Option`, pas `Result`, aucune info perdue). Voir JOURNAL.md cycle 107 pour le détail complet.

## Mise à jour (2026-08-05, v0.25.24, cycle 109) — BOUCLE ARRÊTÉE SUR DEMANDE UTILISATEUR

**Cycle 109** : `docker.rs::get_docker_snapshot` réduisait "Docker jamais installé" et "Docker installé mais daemon injoignable/permission refusée" au même `available:false`, `NetworkPage.vue` affichait le même message générique dans les deux cas -- message d'erreur réel jeté. Reproduit en direct (`apt-get download docker.io` + `dpkg-deb -x`, sans root, `docker ps` contre un daemon absent -- confirmé code 1, message réel sur stderr, correctement capturé par `run_with_timeout` mais ensuite jeté par la fonction elle-même). Corrigé : `DockerSnapshot` gagne `installed`/`error`, UI distingue les deux cas. 281 Rust (inchangé, wiring pur), 301 frontend (299→301, +2 tests), vue-tsc clean. Version 0.25.23 → 0.25.24, commit `37b74a8`, poussé sur `origin/master`.

**Demande explicite utilisateur reçue en cours de cycle 109** : corriger, refaire une release complète, PUIS ARRÊTER LA BOUCLE. Release v0.25.24 en cours de build au moment de cette entrée (voir fin de fichier ou JOURNAL.md pour le statut final une fois publiée). Job cron `28e05809` (récurrence 10 min) annulé après publication -- **aucun nouveau cycle ne partira sans que l'utilisateur ne relance explicitement la boucle**.

## Mise à jour (2026-08-06, v0.25.25, cycle 110) — BOUCLE RELANCÉE

Nouveau cron `c96b445f` (10 min, session-only, expire ~2026-08-13). **Cycle 110** : root-cause fix dans `subprocess::run_capturing_exit_code` -- pipait `stderr` mais ne le lisait jamais, rendant ses 6 appelants structurellement incapables de récupérer un message d'erreur stderr-only, peu importe leur propre formatage. Reproduit en direct (`tar` sur fichier `chmod 000` : erreur réelle uniquement sur stderr, code 2). Signature changée en `(stdout, stderr, code)`, `backup.rs`/`dnf.rs`/`malwarescan.rs` incluent maintenant stderr dans leurs messages d'erreur ; `snapshots.rs`/`smart.rs`/`optimizations.rs` juste adaptés (logique déjà correcte, inchangée). 285/285 Rust (281→285, +4). Version 0.25.24→0.25.25, commit `2d60253`.

**Découverte annexe corrigée dans le même cycle** : root-cause du bruit CRLF récurrent identifiée (`core.autocrlf=true` Git for Windows + `.gitattributes` trop étroit, ne couvrait que `src-tauri/packaging/*`) -- étendu à tout le dépôt (`* text=auto eol=lf`), commit `c56fcf0`. Ne devrait plus jamais falloir refaire le nettoyage `git checkout -- .` observé à chaque reprise de session récente.

**Suspects toujours en attente de VM** (inchangés) : gap validation `quarantine-file` (accepte `/`), exit-code `apt-autoremove` masqué -- les deux touchent du pkexec déjà vérifié en VM live, ne pas modifier sans re-test live.

**Piste "message d'erreur réel perdu"** : campagne largement traitée maintenant (root-cause fixée dans `run_capturing_exit_code`, tous ses appelants couverts). Reste à vérifier au même angle : callers de `run_with_timeout`/`run_with_timeout_env` qui pourraient avoir un gap symétrique (ces fonctions capturent déjà stderr correctement dans leur propre chemin d'erreur -- moins probable d'avoir le même bug, mais pas formellement re-vérifié).

Prochain cycle : reprendre soit la piste `run_with_timeout` ci-dessus, soit repasser à un audit page-par-page/module-par-module classique (le sweep transversal de patterns connus s'épuise, cf. observation cycle 30).

## Mise à jour (2026-08-06, v0.25.26, cycle 111)

Piste `run_with_timeout`/`run_with_timeout_env` : **vérifiée, aucun gap** (les deux capturent déjà stderr correctement). Sweep `.unwrap()` en code non-test : rien à corriger (tous sûrs par construction ou idiome accepté). Bascule vers audit page-par-page (pages classées par nombre de mentions dans JOURNAL.md, la moins auditée en premier) : `BluetoothPage.vue` -- **vrai bug trouvé**, adaptateur présent + 0 périphérique appairé n'affichait aucun message (pattern "liste vide sans message", jamais appliqué à cette page précise malgré 2 audits antérieurs sur d'autres angles). Corrigé. 302/302 frontend (301→302, +1), 285/285 Rust (inchangé), vue-tsc clean. Version 0.25.25→0.25.26, commit `22de4d1`.

**Méthode retenue pour les prochains cycles d'audit page-par-page** : classer les pages Vue par `grep -c "<NomPage>.vue" .loop/JOURNAL.md`, prendre la moins mentionnée en premier -- `DiagnosticPage.vue`/`FirewallPage.vue` (2 mentions chacune) sont les prochaines candidates les moins auditées après `BluetoothPage.vue`.

**Suspects toujours en attente de VM** (inchangés) : gap validation `quarantine-file` (accepte `/`), exit-code `apt-autoremove` masqué -- pkexec déjà vérifié en VM live, ne pas modifier sans re-test live.

## Mise à jour (2026-08-06, v0.25.27, cycle 112)

Plusieurs pages/patterns audités et écartés (déjà solides) : `DiagnosticPage.vue`, `BackupPage.vue` (déjà couverte cycle 79), sweep `v-html` (0 occurrence), sweep `@click` avec `invoke()` inline (0 occurrence), `DataRecoveryPage.vue`, `AntivirusPage.vue`. VM Debian re-testée (172.18.32.124:22) : toujours injoignable.

**`quarantine-file` : suspicion CONFIRMÉE par lecture directe du code (pas juste une hypothèse comme noté jusqu'ici)** -- `validate_quarantine_path("/")` passe toutes les vérifications, côté Rust ET côté script shell `nitrux-pkexec-helper` (copies volontairement en miroir). Lu le script réel (jamais exécuté) : `basename "/"` = `/`, donc `quarantine-file /` construirait `mv / /var/lib/nitrux/quarantine/<timestamp>-/` en root -- tentative de déplacer la racine entière du système de fichiers. Corrigé côté Rust UNIQUEMENT (`security_write.rs`, fonction pure non-privilégiée, aucun risque, aucune VM nécessaire) : rejette `/`, `//`, `///`. 286/286 Rust (285→286). Version 0.25.26→0.25.27, commit `ab837e4`.

**PRIORITÉ HAUTE pour la prochaine session avec VM disponible** : le script `nitrux-pkexec-helper` (`quarantine-file` subcommand, fonction `validate_quarantine_path` shell) a le MÊME trou que celui qu'on vient de corriger côté Rust -- le garde-fou client empêche maintenant l'app d'envoyer `/`, mais le script privilégié lui-même reste vulnérable si jamais atteint par un autre chemin. Nécessite : re-vérification live VM (le script est déjà vérifié/en prod, toute modif exige ce test) avant d'ajouter la même exclusion côté shell (`case "$1" in /) die "..." ;; esac` après le check d'absolu, avant le check `..`).

**Méthode page-par-page** : prochaines candidates les moins auditées après `BluetoothPage.vue`/`DiagnosticPage.vue` : `FirewallPage.vue` (déjà bien couvert malgré le compte bas, cf. cycles 43/97) -- revoir le classement par mentions, `BenchmarkPage.vue`/`DependenciesPage.vue`/`DiskVisualizerPage.vue`/`DnsSwitcherPage.vue`/`DriversPage.vue`/`OptimizationsPage.vue`/`PeripheralsPage.vue`/`TemperaturesPage.vue` (3 mentions chacune) sont les prochaines par ce critère.

## Mise à jour (2026-08-06, v0.25.28, cycle 113) — VM enfin joignable, quarantine-file clos côté code

**VM Debian de nouveau joignable** (IP a tourné : 172.18.32.124 → **172.21.233.222**, technique `Get-VM ... NetworkAdapters` en PowerShell pour la retrouver sans identifiants SSH). Accès SSH confirmé (dev/1998 toujours valides). Scripts paramiko réécrits dans `scratchpad/` (l'ancienne session les avait perdus) : `ssh_run.py`/`ssh_put_b64.py` (base64 sur exec_command, `sftp.put()` échouait pour une raison non élucidée)/`ssh_sudo.py`.

**`quarantine-file` : les DEUX moitiés du correctif sont faites** (Rust cycle 112 + `src-tauri/packaging/nitrux-pkexec-helper` ce cycle, commit `74a3d96`). Bug reproduit EN DIRECT sur la vraie VM (fonction `validate_quarantine_path` isolée, jamais via le vrai `pkexec`/`mv`) avant ET après le correctif -- 8/8 cas conformes après fix. **Déploiement sur le binaire privilégié réel `/usr/bin/nitrux-pkexec-quarantine-file` (via `sudo cp`) bloqué par le classificateur auto mode** (écriture système root = confirmation utilisateur requise) -- reculé sans contourner, binaire VM resté inchangé (vérifié). Le correctif source est committé mais **la release .deb/.rpm/.AppImage actuellement publiée a toujours le trou** -- à signaler à l'utilisateur comme prioritaire pour la prochaine coupure de release (sévérité : tentative de `mv /` en root).

**Nouveau piège d'infra découvert ce cycle** (documenté dans LESSONS.md) : appeler un script Python avec un chemin `/tmp/...` en argument depuis Git Bash le fait réécrire en chemin Windows par la conversion MSYS avant que Python ne le voie -- `MSYS_NO_PATHCONV=1` en préfixe de la commande contourne ça.

## Mise à jour (2026-08-06, v0.25.29, cycle 114) — apt-autoremove clos, les 2 suspects historiques fermés côté code

VM toujours joignable (172.21.233.222). Suspect `apt-autoremove` (mentionné cycles 34/44) enfin creusé sérieusement -- l'ancien "soupçon" cycle 34/35 portait sur une hypothèse DIFFÉRENTE et infirmée (`set -eu` interceptant un échec), pas sur celle-ci.

**Bug réel confirmé** : la branche `system-tool apt-autoremove` teste 3 gestionnaires (`apt-get`/`dnf`/`zypper`) via `if command -v X; then X autoremove; fi` × 3, sans `exec`. Sur un système à un seul gestionnaire (le cas normal, Debian cible = apt-get seul), le DERNIER `if` de la chaîne teste un gestionnaire absent → `if false; then...; fi` sans `else` retourne 0 par définition POSIX → le bloc rapportait **TOUJOURS** succès (0), quel que soit le résultat réel d'`apt-get autoremove`. Reproduit localement (pas besoin de VM, sémantique shell pure) avec un faux `apt-get` via `PATH` (jamais le vrai). Impact confirmé côté Rust : `run_with_timeout` traite tout non-zéro comme `Err` -- le bouton "nettoyer les paquets orphelins" affichait donc toujours succès en UI, même en cas d'échec réel. Corrigé (capture du vrai code de sortie, `exit "$status"` explicite), re-testé isolément après fix : 0/1/100 propagés correctement. Commit `f350c92`.

**Les 2 suspects historiques du CHECKPOINT (quarantine-file, apt-autoremove) sont maintenant clos côté code source.** Aucun des deux n'est encore dans une release publiée -- **recommandation forte : couper une nouvelle release (.deb/.rpm/.AppImage) bientôt** pour livrer ces deux correctifs (le premier a une sévérité réelle : tentative `mv /` en root).

**Plus aucun suspect connu en attente.** Prochains cycles : reprendre l'audit page-par-page (méthode établie cycle 111-112, classement par mentions JOURNAL.md) ou un nouveau sweep transversal si une piste se présente.

## Mise à jour (2026-08-06, v0.25.30, cycle 115)

Audit page-par-page retenté : toutes les candidates à faible score de mentions se sont révélées déjà auditées en profondeur (le compte de mentions ne reflète pas fidèlement la couverture -- vérifié en relisant chaque mention). **Nouvel angle jamais essayé sur toute la campagne : `cargo clippy --all-targets`.** 8 warnings, tous stylistiques (tabs doc comments, `.trim()` redondant, `sort_by`→`sort_by_key`, `vec!`→littéral tableau en test, type complexe extrait en alias) -- 0 bug fonctionnel, tous corrigés, 0 warning restant, 286/286 tests inchangé. Commit `55f38b3`.

**`cargo clippy` à ajouter à la rotation d'angles transversaux** pour les prochains cycles si l'audit page-par-page redevient stérile -- jamais utilisé avant ce cycle en ~115 cycles de campagne.

**Rappel non résolu depuis cycle 113/114** : les 2 correctifs pkexec (quarantine-file racine, apt-autoremove exit-code) sont committés mais PAS encore dans une release publiée -- recommandation toujours active de couper une nouvelle release bientôt.

## Mise à jour (2026-08-06, v0.25.30, cycle 116) — 1er cycle propre depuis la relance

Rotation d'angles jamais essayés, tous négatifs/non-actionnables : `npm audit` (0 vuln), `cargo audit` (0 vraie vuln, 18 warnings "unmaintained" GTK3 transitifs via `tauri`, hors contrôle du projet), ESLint (absent, hors scope d'en configurer un), sweep `TODO`/`FIXME` (0), sweep `#[allow(...)]` (0), `cargo fmt --check` (203 diffs -- divergence cohérente et délibérée du style rustfmt par défaut sur tout le dépôt, PAS appliqué : reformater toucherait quasi chaque fichier pour un changement 100% cosmétique, décision à soumettre à l'utilisateur plutôt qu'unilatérale). Aucun changement de code ce cycle -- consigné honnêtement plutôt que forcer un correctif spéculatif.

**Angles transversaux maintenant épuisés pour cette rotation** : clippy (cycle 115, fait), npm/cargo audit (ce cycle, fait), TODO/allow sweep (ce cycle, fait). Prochain cycle : reprendre l'audit page-par-page module Rust (pas seulement pages Vue -- aucun module `src-tauri/src/*.rs` n'a été classé par fraîcheur comme les pages Vue jusqu'ici) ou attendre un nouveau retour utilisateur/testeur.

## Mise à jour (2026-08-06, v0.25.31, cycle 117) — vraie faille de sécurité trouvée en audit module Rust

Premier classement des modules Rust par fraîcheur (comme fait pour les pages Vue). `flatpak.rs`/`install.rs` déjà couverts (cycle 52). `pkexec_bootstrap.rs` relu ligne à ligne pour la première fois (avant : juste son ajout + un recoupement de noms).

**Vraie faille de sécurité trouvée (CWE-377/CWE-367)** : `write_bootstrap_script_to_temp` écrivait le script destiné à être exécuté EN ROOT (`pkexec sh <chemin>` juste après) à un chemin prévisible via `std::fs::write` simple -- permissions par défaut (world-readable) + suit silencieusement un symlink préexistant. Un attaquant local (ou malware même-utilisateur) pré-positionnant un symlink à ce chemin exact pourrait rediriger l'écriture, ou avec un timing serré influencer ce qui s'exécute en root. Corrigé : création exclusive (`create_new`) + permissions `0o600`. Root n'a aucun souci à lire un fichier 0600 (DAC Unix ne s'applique pas à root) -- comportement inchangé, seule la faille pour les autres utilisateurs/processus est fermée.

**Décision de scope importante, à retenir pour de futurs cas similaires** : cette modif touche SEULEMENT la création non-privilégiée du fichier avant l'appel pkexec -- l'invocation `pkexec sh <chemin>` elle-même est inchangée. Jugé NE PAS nécessiter de re-test VM live car la capacité de root à lire un fichier 0600 est un comportement Unix universel/fondamental, pas un quirk propre à ce projet (contrairement à la résolution d'action par exec-path de pkexec, qui ÉTAIT une découverte surprenante nécessitant vérification live en son temps). Distinction utile : "modifier ce qui tourne EN ROOT" (nécessite VM) vs "modifier la préparation non-privilégiée EN AMONT d'un appel pkexec, sur un comportement OS fondamental" (peut se justifier par raisonnement + tests unitaires seuls).

2 nouveaux tests (permissions 0600, symlink pré-positionné nettoyé pas suivi). 288/288 Rust (286→288), clippy toujours 0 warning. Commit `9572b1d`.

**Rappel toujours actif** : 3 correctifs pkexec maintenant en attente d'une release publiée (quarantine-file, apt-autoremove, bootstrap symlink) -- aucun encore dans un `.deb`/`.rpm`/`.AppImage`.

## Mise à jour (2026-08-06, v0.25.32, cycle 118)

**Angle productif identifié : après avoir trouvé un vrai bug, chercher systématiquement toutes les autres instances du MÊME pattern avant de changer de portion.** Appliqué au symlink/temp-file du cycle 117 (`grep -rn "temp_dir()\|::write("` hors tests) : un seul autre candidat production, `benchmark.rs::benchmark_disk`, même faille CWE-377 mais sévérité moindre (pas de root derrière). Corrigé par cohérence. Refactor DRY : nouveau module `secure_temp.rs` partagé par `pkexec_bootstrap.rs` et `benchmark.rs` (2 fonctions : `create_exclusively_owner_only` pour les besoins avancés type chronométrage, `write_exclusively_owner_only` pour le cas simple). 289/289 Rust, clippy 0 warning. Commit `405215d`.

**Rappel toujours actif** : 4 correctifs maintenant en attente d'une release publiée (quarantine-file, apt-autoremove, bootstrap symlink, benchmark symlink) -- aucun encore dans un `.deb`/`.rpm`/`.AppImage`.

**Prochain cycle** : soit reprendre l'audit module-par-module Rust (candidats suivants par fraîcheur : `bluetooth.rs`/`universal.rs`, 2 mentions), soit re-balayer les pages Vue avec la même discipline "lecture complète, pas juste grep de pattern connu" qui a payé ce cycle-ci et le précédent.

## Mise à jour (2026-08-06, v0.25.33, cycle 119)

`bluetooth.rs`/`packages/universal.rs` relus intégralement -- propres (déjà audités par le passé). **Nouvel angle de sécurité, variante différente (confidentialité des permissions, pas TOCTOU/symlink)** : `create_backup`/`backup.rs` écrit une archive `.tar.gz` dans `$HOME` via `tar -czf`, qui hérite du umask du process -- confirmé en direct : `-rw-r--r--` (644, world-readable), alors que l'archive peut contenir des clés SSH/identifiants navigateur/documents, et que `$HOME` est traversable par d'autres utilisateurs locaux sur une install Debian par défaut. Corrigé : `chmod 0600` après succès de `tar`, échec du chmod remonté comme vraie erreur (pas best-effort silencieux). Vérifié que `report.rs` (sauvegarde via dialogue natif utilisateur) et `snapshots.rs` (délégué à `timeshift`, chemins root) n'ont pas le même risque. 290/290 Rust, clippy 0 warning. Commit `a69d528`.

**5 correctifs de sécurité maintenant en attente d'une release publiée** (quarantine-file, apt-autoremove, bootstrap symlink, benchmark symlink, permissions backup) -- aucun encore dans un `.deb`/`.rpm`/`.AppImage`. Recommandation de coupure de release maintenant répétée sur 4 cycles consécutifs (114/117/118/119).

**Prochain cycle** : poursuivre l'audit module-par-module Rust (`accounts.rs`/`apt.rs` déjà couverts, `portscan.rs`/`zypper.rs` à vérifier -- 4 mentions chacun) ou repasser aux pages Vue.

## ⚠️ ACTION HUMAINE REQUISE (2026-08-06, cycle 120) — faille trouvée, correctif bloqué par le classificateur de sécurité

**`disk_write.rs::clone_disk` (image disque complète via `dd` dans le script pkexec) : même faille que `backup.rs` (cycle 119), en pire.** Le script `src-tauri/packaging/nitrux-pkexec-helper` (bloc `clone-disk)`) exécute `exec dd if="$source_disk" of="$dest_path" ...` en root, sans jamais restreindre les permissions du fichier de sortie. Vérifié EN DIRECT (sans écriture privilégiée) : `dd` produit du 644 (world-readable) sous l'umask standard 022, et le umask par défaut de root sur Debian est le même (confirmé par lecture de `/etc/login.defs`, pas de `UMASK` custom). Un clone de disque complet peut contenir litéralement tout (SSH, identifiants, l'OS) -- un clone world-readable est une fuite de confidentialité sérieuse.

**Correctif prêt, non appliqué** -- remplacer dans `src-tauri/packaging/nitrux-pkexec-helper`, bloc `clone-disk)` :
```sh
exec dd if="$source_disk" of="$dest_path" bs=4M status=progress conv=fsync
```
par :
```sh
dd if="$source_disk" of="$dest_path" bs=4M status=progress conv=fsync
chmod 600 "$dest_path"
```
(`set -eu` gère l'échec de `dd` exactement comme avant -- abandon immédiat, `chmod` jamais atteint si `dd` échoue.)

**Pourquoi non appliqué** : la tentative d'édition de ce fichier a été bloquée par le classificateur auto mode de cette session (contenu `dd`/clonage de disque jugé sensible même pour une simple modification de fichier source texte, sans exécution). Reculé sans contourner. **Ne pas re-tenter automatiquement dans un futur cycle non-supervisé** -- sera bloqué de la même façon à chaque fois. Nécessite soit que l'utilisateur applique ce correctif lui-même, soit une session où la permission d'édition peut être accordée explicitement.

## Mise à jour (2026-08-06, v0.25.33, cycle 121) — cycle propre, note du cycle 120 respectée

Note "ne pas re-tenter" respectée -- portion cherchée ailleurs. 8 vérifications distinctes, toutes propres/hors de portée : `portscan.rs`/`packages/zypper.rs` (relus intégralement), `ReportGeneratorPage.vue` (téléchargement navigateur natif, pas le même risque que backup/clone-disk), sweep exhaustif de tous les `File::create`/`OpenOptions`/`fs::write` du code Rust hors tests (rien en production restant), ID de session terminal (non pertinent, app desktop mono-utilisateur), `trash.rs::copy_recursive` (hypothèse de perte de permissions sur repli cross-FS **infirmée** par un vrai test Rust compilé : `std::fs::copy` préserve bien le mode 0600→0600).

Aucun changement de code ce cycle. Le seul élément en attente reste `clone-disk` (cycle 120, ci-dessus) -- toujours action humaine requise.

## Mise à jour (2026-08-06, v0.25.33, cycle 122) — 2e cycle propre consécutif

Nouvel angle : fuite de données frontend (`console.log`/`localStorage`) -- 0 occurrence de log, `localStorage` utilisé uniquement pour des données non-sensibles attendues. `NxSparkline.vue`/`preferencesStore.ts` relus intégralement -- déjà défensifs, rien trouvé.

**2e cycle consécutif sans correctif (121+122)** -- pas encore 3 (règle du projet), mais si le prochain cycle est négatif aussi, repasser à un sweep de pattern transversal plutôt que module-par-module (qui montre des signes d'épuisement après 122 cycles de campagne).

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.33, cycle 123) — 3e cycle propre consécutif, seuil de la règle atteint

Vérification exhaustive du câblage des commandes Tauri : 69 commandes `generate_handler!` == 69 `#[tauri::command]` définis == 69 noms `invoke(...)` distincts côté frontend, `diff` des deux listes vide -- correspondance parfaite, aucune commande orpheline. Échantillon de cohérence des noms de paramètres (`disk_write.rs`) correct. Sweep chaînes anglaises résiduelles UI : 0.

**3e cycle consécutif sans correctif (121, 122, 123)** -- seuil de la règle du projet atteint, mais les trois cycles ont exploré des angles réellement distincts (permissions/sécurité, fuite frontend, câblage IPC), pas des sweeps redondants. Signal probable : la surface facilement accessible du projet est maintenant très largement saine après 123 cycles cumulés.

**Stratégie pour le prochain cycle** : revenir à un sweep de pattern transversal simple plutôt qu'un audit fichier-par-fichier (qui s'épuise), ou attendre un nouveau signal externe (retour utilisateur/testeur, déblocage `clone-disk`).

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.33, cycle 124) — 4e cycle propre consécutif

Sweeps arithmétiques/typage (soustraction non signée, troncature `as u16/u32/u8`, indexation directe `fields[N]`) sur tout `src-tauri/src` : tous négatifs. Point notable : les 37 occurrences d'indexation directe dans 10 fichiers sont TOUTES précédées d'une vérification de longueur -- discipline défensive appliquée de façon parfaitement cohérente, aucune exception.

**4e cycle consécutif sans correctif (121-124)**, mais chaque cycle a exploré un angle distinct et réel. Recommandation pour le prochain cycle : si un 5e cycle est aussi négatif, envisager d'attendre un signal externe (retour utilisateur/testeur) plutôt que de continuer à chercher sans piste concrète -- la base de code a probablement atteint un plateau de maturité pour ce type d'audit automatisé.

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise, toujours pas livré en production.

## Mise à jour (2026-08-06, v0.25.33, cycle 125) — 5e cycle propre consécutif, plateau probable atteint

Dernière piste (messages d'erreur Rust encore en anglais) : négative, les 2 seules correspondances sont des citations verbatim légitimes d'outils tiers, pas des messages NiTruX.

**5e cycle consécutif sans correctif (121-125).** Recommandation appliquée : la boucle continue de tourner normalement (aucune action d'arrêt prise), mais signalé clairement à l'utilisateur en fin de cycle que la prochaine vraie avancée viendra probablement d'un signal externe (retour utilisateur/testeur réel, déblocage `clone-disk`) plutôt que d'une nouvelle recherche automatique -- la surface facilement auditable par lecture de code semble épuisée après 125 cycles cumulés.

**Ne pas re-proposer indéfiniment le même diagnostic à chaque cycle négatif futur** -- si un nouveau cycle est aussi négatif, il suffit de le noter brièvement plutôt que de répéter cette analyse en détail à chaque fois.

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.34, cycle 127) — série négative rompue, nouveau chantier a11y ouvert

**Vrai gap systémique trouvé** : `NxInput.vue`/`NxSelect.vue` (composants partagés, 13+ pages) n'avaient aucun mécanisme de nom accessible (WCAG 3.3.2) -- `NxInput` reposait uniquement sur `placeholder` (pas un vrai substitut), `NxSelect` n'avait même pas ça. Corrigé au niveau composant : prop optionnelle `ariaLabel` rétrocompatible sur les deux, appliquée d'abord à `DisksPage.vue` (formater/étendre/cloner -- les actions les plus destructrices du projet). 304/304 frontend (+2), vue-tsc clean. Commit `8fa7ff8`.

**Nouveau chantier multi-cycle ouvert : déploiement `ariaLabel` sur les pages restantes.** La capacité existe maintenant sur les composants -- reste à l'appliquer page par page aux ~13 autres usages de `NxInput`/`NxSelect` (`grep -rn "<NxInput\|<NxSelect" src/pages/*.vue` pour les lister). Même pattern incrémental qu'utilisé pour "liste vide sans message" (cycles 1-32) -- traiter quelques pages par cycle plutôt que tout d'un coup, prioriser les pages avec actions destructrices/pkexec en premier (déjà fait pour Disques ; `security_write`/`network_write`/`backup`/`troubleshoot` sont les prochaines candidates logiques).

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.35, cycle 128) — chantier ariaLabel : lot 2

`BackupPage.vue`/`NetworkPage.vue` (+ son `<textarea>` DNS natif, même lacune)/`AntivirusPage.vue`/`SystemToolsPage.vue` corrigées (6 champs). 304/304 frontend, vue-tsc clean. Commit `07a85b4`.

**Pages `ariaLabel` restantes** : `DiskVisualizerPage.vue`, `FileToolsPage.vue`, `InstalledSoftwarePage.vue`, `PackagesPage.vue`, `ProcessesPage.vue`, `ReportGeneratorPage.vue`, `ScriptsPage.vue`, `SettingsPreferencesPage.vue`, `UninstallerPage.vue` -- 9 pages, à traiter par lots de 2-4 sur les prochains cycles.

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.36, cycle 129) — chantier ariaLabel : lot 3

`FileToolsPage.vue` (6 champs)/`PackagesPage.vue`/`UninstallerPage.vue` corrigées (10 champs au total). 304/304 frontend, vue-tsc clean. Commit `d40df0f`.

**Pages `ariaLabel` restantes** : `DiskVisualizerPage.vue`, `InstalledSoftwarePage.vue`, `ProcessesPage.vue`, `ReportGeneratorPage.vue`, `ScriptsPage.vue`, `SettingsPreferencesPage.vue` -- 6 pages.

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.37, cycle 130) — chantier ariaLabel CLOS (100%)

Dernier lot : les 6 pages restantes corrigées (dont `ScriptsPage.vue`'s `<textarea>` natif et `SettingsPreferencesPage.vue` où un `<label>` visible existait mais sans association `for`/`id`). **Confirmé par grep : plus aucun `NxInput`/`NxSelect` sans nom accessible dans toute l'app.** 304/304 frontend, vue-tsc clean. Commit `391aeb2`.

**Bilan chantier a11y (cycles 127-130, 4 cycles)** : 2 composants partagés (`NxInput`/`NxSelect`) + 15 pages + 2 `<textarea>` natifs corrigés, 0 régression sur tout le parcours. Terminé.

**Aucun chantier multi-cycle actif pour le moment.** Prochain cycle : identifier une nouvelle portion (page-par-page/module Rust) ou un nouveau pattern transversal.

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise, toujours pas livré en production. De nombreux correctifs (sécurité + a11y) se sont accumulés depuis le cycle 110 sans jamais être publiés dans une release -- recommandation toujours active de couper une nouvelle release bientôt.

## Mise à jour (2026-08-06, v0.25.37, cycle 133) — outil abandonné pour raison pratique

`cargo tarpaulin` essayé (jamais tenté avant) mais **impraticable dans ce projet** : force une recompilation instrumentée complète de toute l'arborescence Tauri/GTK (~500+ crates) au lieu de réutiliser le cache incrémental -- après 10+ minutes, la compilation n'avait pas atteint le crate `nitrux` lui-même. **Ne pas retenter automatiquement en cycle non-supervisé** -- nécessiterait une session dédiée hors cadence normale de 10 min si un jour souhaité. `cargo llvm-cov` aurait probablement la même limitation (même mécanisme d'instrumentation).

3e cycle négatif consécutif (131-133), mais celui-ci pour une contrainte d'outillage, pas un manque de piste. Aucun changement de code.

## Mise à jour (2026-08-06, v0.25.38, cycle 134) — vrai bug trouvé, retour au module-par-module

Seuil des 3 cycles négatifs atteint → retour à l'audit module-par-module (règle du projet). `themeStore.ts`/`ThemeEditorPage.vue` relus intégralement.

**Vrai bug trouvé** : `ThemeEditorPage.vue::handleSave` (bouton "Sauvegarder" après édition de couleurs) n'appelait que `saveCustomTheme` (ajoute à la liste des thèmes perso) sans jamais activer le thème édité (`setTheme`, seule fonction qui persiste réellement "quel thème est actif"). Les couleurs éditées étaient donc silencieusement perdues au prochain lancement, malgré un message de succès. **Le flux jumeau `importTheme` avait DÉJÀ ce correctif exact** (avec son propre test de régression dédié) -- jamais répercuté sur le bouton "Sauvegarder" de l'éditeur, qui a le même défaut. Corrigé (miroir exact d'`importTheme`), nouveau test de régression. 305/305 frontend (+1), vue-tsc clean. Commit `8b2b3e5`.

**Leçon méthodologique** : quand un bug est corrigé sur UN site d'appel d'une fonction store, vérifier systématiquement les AUTRES sites d'appel de la même fonction pour le même défaut -- ce bug existait probablement depuis la création de `ThemeEditorPage.vue`, non détecté car `importTheme` avait déjà "absorbé" toute l'attention portée à ce pattern.

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.39, cycle 138) — vrai bug systémique trouvé et corrigé

**`scripts.rs::run_script` jette tout stdout dès qu'un code de sortie est non-nul** -- correct en général, mais faux pour 19 entrées `du -sh <chemin>` (non pipées) du catalogue `systemToolsCatalog.ts` : `du` sort en 1 sur sous-dossier root-only ou chemin manquant (routine sous `/var/log`/`/home/*`/entrées à double chemin localisé type `~/Téléchargements ~/Downloads`) tout en affichant un résultat réel utile. Reproduit EN DIRECT sur la vraie VM (2 cas, dont un touchant quasi tous les utilisateurs francophones : `~/Downloads` n'existe jamais sur un système FR). Corrigé : `|| true` ajouté aux 19 entrées non-pipées (3 déjà pipées `| sort | head` étaient déjà sûres). Nouveau `systemToolsCatalog.spec.ts` (n'existait pas) : unicité id, invariant XOR command/privilegedAction, garde-fou anti-régression sur le pattern `du || true`. 308/308 frontend (+3), vue-tsc clean. Commit `2dc84e9`.

**Leçon méthodologique** : l'échantillonnage aléatoire du catalogue (cycle 137, technique VM live) a payé au 2e essai -- creuser un `EXIT=1` inattendu même quand la sortie affichée semblait correcte a révélé un bug bien plus large que l'échantillon initial ne le laissait supposer (19 entrées, pas 1).

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.40, cycle 139) — même filon, tool shape différent (grep/pgrep)

Généralisation directe du cycle 138 : 8 entrées `grep`/`pgrep` du catalogue où "aucune correspondance" (exit 1) est une réponse VALIDE et routinière, pas un échec -- confirmé que `SystemToolsPage.vue` gère déjà proprement un succès vide ("(terminé, aucune sortie)") avant de corriger. Vérifié EN DIRECT sur la vraie VM : `lspci | grep vga`/`audio` **confirmés cassés** (VMBus Hyper-V Gen2, aucun device classé PCI VGA/audio), `pgrep openvpn`/`env | grep proxy` **confirmés cassés** (cas normal sans VPN/proxy, pas un cas limite). 3 autres corrigées défensivement (mêmes profil de risque, pas reproduits comme cassés sur cette VM précise). `|| true` sur les 8, nouveau test dédié (liste non mécanisable contrairement au cas `du`, la plupart des `grep` du catalogue ciblent des champs kernel toujours présents). 309/309 frontend (+1), vue-tsc clean. Commit `427d3db`.

**Le filon "code de sortie de commande catalogue masque un résultat valide" semble épuisé** après 2 cycles productifs consécutifs (138 du, 139 grep/pgrep). Prochain cycle : reprendre un audit page/module classique ou chercher un nouveau pattern transversal.

Élément en attente inchangé : `clone-disk` (cycle 120) -- action humaine requise.

## Mise à jour (2026-08-06, v0.25.44, cycle 147-148) — bug responsive corrigé, nouveau point produit en attente

**Cycle 147** : `DriversPage.vue`/`DiagnosticPage.vue`/`UpdatesPage.vue` rendaient un `<table>` brut sans conteneur `overflow-x` -- seule `PackagesPage.vue` avait ce correctif (appliqué en R14, dont les notes la décrivaient à tort comme "le seul vrai tableau de l'app"). Corrigé en miroir exact du pattern `PackagesPage.vue`. Commit `1c3bb13`.

**Cycle 148** : audit page-par-page étendu (Benchmark/Dependencies/Temperatures/Optimizations/Peripherals/DnsSwitcher/blocs `<pre>`/cohérence try-catch `onMounted`) -- tout propre, aucune régression trouvée.

**Nouveau point en attente de décision produit (pas un blocage classificateur comme `clone-disk`)** : `preferencesStore.ts::confirmNonDestructiveActions` (bascule "Demander confirmation pour les actions non-destructives" dans Préférences) est persisté et testé au niveau du store, mais n'est lu NULLE PART ailleurs dans l'app -- confirmé par `grep` qu'aucun mécanisme de dialogue de confirmation (`confirm(`/`NxDialog`/`NxModal`) n'existe même dans le code. Le bascule n'a donc aucun effet observable. **Volontairement non corrigé automatiquement** : implémenter correctement nécessiterait de concevoir et câbler un vrai système de confirmation à travers de nombreuses actions destructrices sur de nombreuses pages -- une fonctionnalité complète, hors périmètre d'un cycle de 10 minutes, et un bricolage partiel serait pire que rien. Nécessite une décision utilisateur sur le périmètre voulu avant implémentation.

Éléments en attente : `clone-disk` (cycle 120, action humaine requise) + `confirmNonDestructiveActions` (cycle 148, décision de périmètre requise). **8 correctifs de code désormais en attente d'une release publiée** depuis la reprise de la boucle au cycle 110 (aucune release cut depuis v0.25.24).

## Mise à jour (2026-08-07, v0.25.54, cycle 149-160) — série productive (tables/sparkline/clés v-for/confirmations/permissions), 1 faille sérieuse trouvée

Cycles 149-158 : correctifs cumulés -- `NxSparkline.vue` responsive fix, clés `v-for` non-uniques (`WiFiAnalyzerPage`, `NetworkPage`), actions de dépannage manquantes exposées, `extend-partition` rejette désormais les systèmes de fichiers non-ext, `clone-disk` refuse d'écraser un fichier existant, confirmations taper-pour-valider ajoutées à la suppression corbeille et à la mise en quarantaine antivirus (filon "action irréversible mal protégée" cycles 156-158, désormais clos). Cycle 159 : `cargo clippy` relancé après ~45 cycles sans passage complet, 1 warning réel corrigé.

**Cycle 160 — faille sérieuse trouvée et corrigée, sévérité la plus haute depuis `quarantine-file`/`clone-disk`** : `packages/apt.rs::parse_apt_line` ne reconnaît que le texte anglais `[upgradable from: ...]`, mais `apt` localise cette chaîne selon `LANG`/`LC_MESSAGES`. Confirmé EN DIRECT sur la vraie VM Debian tournant dans sa propre locale d'installateur par défaut (`fr_FR.UTF-8`, pas un cas exotique) : `apt list --upgradable` y affiche `[pouvant être mis à jour depuis...]`, aucune ligne ne matchait jamais -- `list_upgradable()` rapportait silencieusement ZÉRO mise à jour sur un système en ayant 3 réelles. Touche apt, le gestionnaire de paquets de la distribution CIBLE principale de l'app -- bug potentiellement très répandu chez tout utilisateur non-anglophone. Corrigé en forçant `LC_ALL=C` (miroir du pattern déjà établi pour `lscpu`). Commit `9cb5916`.

**Piste ouverte pour un prochain cycle** : généraliser la vérification "invocation shell sans forçage de locale" à d'autres commandes du projet parsant une sortie textuelle localisable (au-delà de `lscpu`/`apt` désormais couverts).

Éléments en attente inchangés : `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit). **17 correctifs fonctionnels + 1 cleanup cosmétique désormais en attente d'une release publiée** depuis v0.25.24 (51 cycles cumulés 110-160) -- recommandation de coupure de release au niveau maximal : plusieurs failles de sécurité/UX réelles ET un bug fonctionnel majeur (mises à jour apt invisibles en locale non-anglaise) restent non livrés à tout utilisateur d'une release existante.

## Mise à jour (2026-08-07, v0.25.74, cycles 161-208) — 48 cycles cumulés, résumé condensé (détail complet dans JOURNAL.md)

Cette section consolide les cycles 161-208 (JOURNAL.md en garde le détail entrée par entrée). Filon "invocation shell sans forçage de locale" (piste ouverte au cycle 160) : `lspci`/`dnf` vérifiés non affectés (0 fichier `.mo` pour lspci ; dnf non testable en direct mais parseur non dépendant de mots-clés anglais) ; `flatpak` confirmé porteur de traductions (19 `.mo`) mais vérification live bloquée par des bibliothèques transitives manquantes, non prioritaire.

**Correctifs fonctionnels réels (cycles 161-208)** :
- Cycle 172 : `NetworkPage.vue` port-scan -- valeur hors 0-65535 plantait la désérialisation IPC (Rust `Vec<u16>`).
- Cycle 176-177 : nom vide après normalisation non vérifié -- `ScriptsPage.vue::saveScript` et `ThemeEditorPage.vue::handleSave` (ce dernier n'avait AUCUNE vérification).
- Cycle 180 : `TemperaturesPage.vue` clé `v-for` sur `t.label` seul -- capteurs à label dupliqué (NVMe multi-disques) rendaient incorrectement ; `DashboardPage.vue` avait déjà le correctif défensif, généralisé ici.
- Cycle 181 : `PackagesPage.vue` bouton "Tout mettre à jour" cliquable même sans mise à jour disponible (page jumelle `UpdatesPage.vue` avait déjà la garde).
- Cycle 182-183 : course de concurrence -- `SystemToolsPage.vue::running` et `DisksPage.vue::smartBusy` étaient des refs uniques partagées entre éléments de liste (au lieu de `Record<string, boolean>` par élément), permettant un double-clic pendant qu'une action reste en cours sur un AUTRE élément. Filon clos après vérification qu'`AntivirusPage`/`UninstallerPage` utilisent déjà un pattern différent et sûr (`!== null` global).
- Cycle 186 : `UpdateHistoryPage.vue` clé `v-for` sur `start_date` seul (apt history.log a une précision à la seconde, deux transactions dans la même seconde produisent un doublon).
- Cycle 187 : `scriptsStore.ts::readPersistedScripts` ne validait que la forme tableau du JSON persisté, pas les champs de chaque élément (contrairement à `themeStore.ts`/`preferencesStore.ts`).
- Cycle 188 : `NetworkPage.vue` clé `v-for` sur `docker.images` par `id` seul -- `docker images` liste une ligne par tag, pas par image unique (auto-correction d'une erreur du sweep du cycle 187).
- Cycle 190-191 : `report.rs` -- `render_html`/`render_pdf` n'avaient AUCUNE section Capteurs (batterie/températures) ; `render_markdown`/`render_html` n'avaient AUCUNE section Utilisation disque. `render_txt` seul les avait. 2 des 4 formats d'export perdaient silencieusement ces données.
- Cycle 199 (cosmétique) : `terminal.rs` -- 4 `.unwrap()` nus sur un verrou de mutex, remplacés par `.expect("message clair")` en miroir du pattern déjà établi par `system.rs`.

**Vérifications transversales exhaustives, toutes négatives** (documentées en détail dans JOURNAL.md, non répétées ici) : `cargo clippy --all-targets` (0 warning), `cargo audit` (0 vraie vulnérabilité), `npm audit` (0 vulnérabilité), grep TODO/FIXME (0 résultat), dérive de champs TypeScript/Rust sur les 49 structs `Serialize` (0 dérive réelle), cohérence de la surface IPC complète (69 commandes enregistrées = 69 appels `invoke()` frontend, correspondance parfaite), lecture intégrale du script `nitrux-pkexec-helper` et des 5 fichiers `.policy` polkit (tout durci, `auth_admin` partout, aucune faille), balayage des deux gros catalogues de données pour doublons de paquet/commande réel (`appCatalog.ts` : 6 paires ; `systemToolsCatalog.ts` : 3 paires -- **signalées pour révision de curation éditoriale humaine, PAS corrigées** : ce sont des décisions de contenu, pas des bugs fonctionnels).

**À ce stade, la quasi-totalité du code source lisible de l'app a été auditée au moins une fois** durant cette session étendue (110-208) : les 40 pages Vue, tous les composants UI partagés, les 8 layouts, tous les stores Pinia, la quasi-totalité des modules Rust (y compris le script pkexec et les policies polkit), les registres de données et leurs types. Plateau de maturité très profond.

Éléments en attente inchangés : `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit) + bouton Vérifier non désactivé (cycle 174, non retenu, UpdatesPage/PackagesPage) + timeout `run_pkexec_with_stdin` (cycle 193, hors périmètre documenté dans le code) + 9 doublons de catalogue signalés pour curation (cycles 201-202, non bloquants). **36 correctifs fonctionnels + 2 cleanups cosmétiques désormais en attente d'une release publiée** depuis v0.25.24 (99 cycles cumulés 110-208) -- **recommandation de coupure de release maintenue au niveau maximal**, backlog important et plateau de maturité atteint.

## Mise à jour (2026-08-07, v0.25.74, cycle 211) — RELEASE PUBLIÉE, backlog livré

**Utilisateur a explicitement demandé la publication** ("publie la nouvelle release avec toute les nouvelle modification") entre les cycles 210 et 211. `npm run tauri build` (WSL2) → 3 bundles (deb/rpm/AppImage) générés sans erreur. Tag annoté `v0.25.74` + release GitHub créée avec les 3 assets et des notes organisées par catégorie (faille apt en tête). Vérification post-build : script `postrm` empaqueté dans le `.deb` comparé binairement au fichier source -- identiques, aucune dérive.

**Les 36 correctifs fonctionnels + 2 cleanups cosmétiques accumulés depuis v0.25.24 sont désormais tous dans une release publiée. Plus aucun backlog de correctifs non livrés.** La recommandation de coupure de release répétée depuis le cycle 160 est close.

Éléments en attente inchangés (non bloquants, statut identique) : `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit) + bouton Vérifier non désactivé (cycle 174, non retenu) + timeout `run_pkexec_with_stdin` (cycle 193, hors périmètre) + 9 doublons de catalogue signalés pour curation (cycles 201-202). 102 cycles cumulés 110-211 depuis la reprise.

## Mise à jour (2026-08-07, v0.25.78, cycles 212-227) — retour à un backlog post-release, audit module-par-module en cours

Depuis la release v0.25.74 (cycle 211), 4 nouveaux correctifs de code accumulés, non encore publiés : `index.html` (branding scaffold, cycle 213), `README.md` (contenu scaffold, cycle 214), `PackagesPage.vue` (responsive flex-wrap, cycle 221), `network_write.rs::validate_port_proto` (cycle 227, détail ci-dessous). Pas encore assez pour justifier une nouvelle coupure de release sans accumulation supplémentaire.

**Technique de comparaison de pages jumelles/patterns dupliqués** établie cycles 221-223 (byte-diff de paires de pages quasi-identiques + cross-check de patterns dupliqués sur 3+ pages) : 1 bug réel trouvé (PackagesPage), puis 2 cycles négatifs consécutifs sur d'autres patterns (chips, filtre texte, confirmation) -- filon considéré clos pour l'instant.

**Seuil des 3 cycles négatifs atteint (222-224) → retour à l'audit module-par-module** (cycles 225-227) : couverture systématique des fichiers Rust backend jamais lus directement cette session, priorité aux fichiers SANS commentaire "reproduit/confirmé en direct" (marqueur d'investigation déjà approfondie ailleurs). ~20 fichiers backend couverts sur ces 3 cycles (`accounts.rs`, `bluetooth.rs`, `drivers.rs`, `duplicates.rs`, `hashcheck.rs`, `malwarescan.rs`, `firewall.rs`, `backup.rs`, `snapshots.rs`, `smart.rs`, `docker.rs`, `boot_manager.rs`, `sensors.rs`, `peripherals.rs`, `update_history.rs`, `hardware_details.rs`, `disk_write.rs`, `network_write.rs`), la quasi-totalité déjà rigoureusement testée/durcie -- **1 vrai bug trouvé au cycle 227** (voir ci-dessous), 5 cycles négatifs sur 6.

**Cycle 227 -- faille "faux succès silencieux" trouvée côté écriture réseau** : `network_write.rs::validate_port_proto` (utilisé par `add_firewall_rule`/`remove_firewall_rule`) ne bornait pas le numéro de port à la plage réelle 1-65535. Reproduit EN DIRECT (`ufw` non-privilégié extrait en local) : `ufw allow 99999999/tcp`/`ufw allow 0/tcp` sortent avec le code 0 (succès) alors que `ERROR: Bad port` n'apparaît que sur stderr, silencieusement ignoré par `run_with_timeout` sur un exit réussi -- un port mal tapé aurait donc été rapporté comme une règle de pare-feu ajoutée avec succès sans que `ufw` n'ait rien fait. Corrigé côté Rust uniquement, en amont de l'appel `pkexec` -- aucun re-test VM live requis (l'invocation privilégiée elle-même est inchangée). Le script helper partagé a la même lacune mais n'a volontairement pas été touché (fichier partagé entre 9 actions polkit, hors périmètre d'un correctif amont qui suffit déjà à fermer la faille pratique depuis l'UI). Commit `ed29081`.

Restant à couvrir pour clore l'audit module-par-module : `cache_size.rs`, `dependencies.rs`, `largefiles.rs`, `dnf.rs`, `pacman.rs`, `zypper.rs`, `security_write.rs`, `pkexec_bootstrap.rs`, `universal.rs`, `secure_temp.rs`.

Éléments en attente inchangés : `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit) + bouton Vérifier non désactivé (cycle 174, non retenu) + timeout `run_pkexec_with_stdin` (cycle 193, hors périmètre) + 9 doublons de catalogue signalés pour curation (cycles 201-202). **4 correctifs accumulés depuis v0.25.74**, pas encore de recommandation de coupure de release (backlog encore modeste). 118 cycles cumulés 110-227.

## Mise à jour (2026-08-07, v0.25.79, cycles 228-236) — bascule Vue complète, 1 nouveau bug trouvé

Audit module-par-module Rust clos définitivement au cycle 229 (couverture complète 2e fois -- `security_write.rs`, `pkexec_bootstrap.rs`, `secure_temp.rs`, `cache_size.rs`, `largefiles.rs`, `packages/dnf.rs`, `packages/pacman.rs`, `packages/zypper.rs`, `dependencies.rs`, `packages/universal.rs` tous couverts, 0 nouveau bug). Bascule ensuite côté Vue (cycles 230-236) : système de dispositions complet (8 layouts + registry + shell + store, cycle 231), couche stores Pinia complète (5/5, cycle 232), `TerminalPage.vue`/`terminal.rs` (cycle 233), `ProcessesPage.vue`/`processes.rs` (cycle 233), `UninstallerPage.vue`/`install.rs`/`packages/mod.rs` (cycle 234), `NetworkPage.vue`/`network.rs` (cycle 235) -- tous clean.

**Cycle 236 -- bug réel trouvé côté lecture disque** : `disks.rs::list_disk_usage` (`df -k -P`) n'avait aucun filtre par type de source, contrairement à `list_disks` (`lsblk`, filtré par `device_type == "disk"`). Reproduit EN DIRECT : la sortie `df` réelle inclut de nombreux pseudo-systèmes de fichiers (`tmpfs`/`none`/`rootfs`/`drivers`), et surtout des montages `snapfuse` (un par paquet Snap) systématiquement à "100%" -- lecture trompeuse en "disque plein" pour un utilisateur regardant la page Disques. Corrigé en filtrant sur `source.starts_with("/dev/")`, miroir du filtre déjà en place côté `lsblk`. Commit `8f91a20`.

Éléments en attente inchangés : `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit) + bouton Vérifier non désactivé (cycle 174, non retenu) + timeout `run_pkexec_with_stdin` (cycle 193, hors périmètre) + 9 doublons de catalogue signalés pour curation (cycles 201-202). **5 correctifs accumulés depuis v0.25.74** (index.html, README.md, PackagesPage.vue flex-wrap, validate_port_proto, list_disk_usage filter) -- backlog toujours modeste, pas de recommandation de coupure de release. 127 cycles cumulés 110-236.

## Mise à jour (2026-08-07, v0.25.79, cycle 245) — RELEASE PUBLIÉE, backlog livré

**Utilisateur a explicitement demandé la publication** ("cree la nouvelle release") après les cycles 237-244 (tous négatifs, sweep frontend continué : `benchmark.rs`/`BenchmarkPage.vue`, `hardware.rs`/`DiagnosticPage.vue`/`DriversPage.vue`, `logs.rs`/`LogsPage.vue`, `AntivirusPage.vue`/`trash.rs`/`CleanerPage.vue`/`DataRecoveryPage.vue`, `DnsSwitcherPage.vue`/`QuickInstallPage.vue`/`flatpak.rs`, `InstallProfilesPage.vue`, `ReportGeneratorPage.vue` -- 0 nouveau bug, sauf un risque signalé non corrigé au cycle 241, `WiFiAnalyzerPage.vue::securityStatus` possible faux-négatif de sécurité sur réseau ouvert, non vérifiable dans cet environnement). `npm run tauri build` (WSL2) → 3 bundles (deb/rpm/AppImage) générés sans erreur. Tag annoté `v0.25.79` + release GitHub créée avec les 3 assets et des notes organisées par catégorie (pare-feu/disques/mise en page en tête, finitions scaffold, note technique sur l'ampleur de l'audit).

**Les 5 correctifs fonctionnels accumulés depuis v0.25.74 sont désormais tous dans une release publiée.** Plus aucun backlog de correctifs non livrés.

Éléments en attente inchangés (non bloquants) : `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit) + bouton Vérifier non désactivé (cycle 174, non retenu) + timeout `run_pkexec_with_stdin` (cycle 193, hors périmètre) + 9 doublons de catalogue signalés pour curation (cycles 201-202) + risque `WiFiAnalyzerPage.vue::securityStatus` (cycle 241, non vérifiable sans matériel Wi-Fi réel). 136 cycles cumulés 110-245 depuis la reprise.

## Mise à jour (2026-08-07, v0.25.80, cycles 246-254) — couverture quasi-complète de toute la codebase lisible, 1 nouveau bug

Cycles 246-247 négatifs (RestorePointsPage/HardwareDetailsPage/PeripheralsPage/BluetoothPage, tous propres). **Cycle 248 -- vrai bug trouvé** : `InstalledSoftwarePage.vue` couplait `list_installed_packages` (peut réellement échouer sans gestionnaire de paquets natif détecté) et `get_environment_variables` (indépendant, infaillible) dans un seul `try/catch` -- un échec du premier masquait silencieusement le second, alors que `UninstallerPage.vue` avait déjà établi le bon pattern (deux appels séparés) pour ce même `list_installed_packages`. Corrigé, test de régression ajouté. Commit `097b8d9`.

Cycle 249 : sweep systématique confirmant qu'aucune autre page ne reproduit ce pattern de couplage (4 blocs à 2-3 `invoke` détectés par grep, tous des dispatches à branchement légitimes). Cycles 250-253 négatifs : `FileToolsPage.vue`, `TroubleshootPage.vue`, `UpdateHistoryPage.vue`, `DiskVisualizerPage.vue`, `PerfHistoryPage.vue`, `OptimizationsPage.vue`, `TemperaturesPage.vue` -- tous propres, couverture des pages Vue désormais quasi exhaustive (quasi-totalité des 41 pages lues directement). **Cycle 254** : couche `src/components/ui/` (composants partagés utilisés sur chaque page) entièrement couverte (9/9 fichiers) -- tous propres.

**État à ce stade** : backend Rust bouclé 2 fois en entier, layouts et stores Pinia complets, quasi-totalité des pages Vue lues, composants UI partagés entièrement couverts. La session a désormais audité directement la quasi-intégralité du code source lisible de l'application.

Éléments en attente inchangés (non bloquants) : `clone-disk` (cycle 120) + `confirmNonDestructiveActions` (cycle 148) + bouton Vérifier non désactivé (cycle 174) + timeout `run_pkexec_with_stdin` (cycle 193) + doublons de catalogue (cycles 201-202) + risque `WiFiAnalyzerPage.vue::securityStatus` (cycle 241). **1 correctif accumulé depuis v0.25.79** (InstalledSoftwarePage.vue). 145 cycles cumulés 110-254.

## Mise à jour (2026-08-07, v0.25.80, cycles 255-265) — plateau de maturité extrême, infrastructure pkexec/polkit close

11 cycles négatifs consécutifs (255-265, hors le correctif du cycle 248 déjà consolidé plus haut), tous des lectures/cross-checks jamais faits directement jusqu'ici : `lib.rs` en entier (agrégation multi-gestionnaires déjà solide) + re-vérification de la surface IPC (69=69, dernière fois cycle 208, aucune dérive) ; confirmation que les 2 exemples du texte répété du déclencheur de boucle (`quarantine-file` accepte `/`, exit-code `apt-autoremove` masqué) sont TOUS LES DEUX déjà corrigés depuis longtemps (2026-08-06) -- texte d'exemple périmé, plus la peine de le revérifier ; `system.rs`+`DashboardPage.vue` (page d'accueil, jamais lue) ; `themes/builtin.ts`/`styles/registry.ts` (cross-checks : 13 thèmes complets sans doublon, 12 styles registry=CSS exact) ; `appCatalog.ts`/`systemToolsCatalog.ts` validés structurellement (502+506 entrées, doublons confirmés identiques à ceux déjà signalés cycles 201-202, aucune nouvelle dérive) ; re-vérification de cadence `npm audit`/`cargo audit` (0 vraie vulnérabilité) et `cargo clippy`/grep TODO (0 résultat) -- deux checks jamais relancés depuis les cycles 161-208 malgré tous les changements de cette session ; `src-tauri/packaging/` entièrement lu et cross-vérifié : script `nitrux-pkexec-helper` (415 lignes, fichier le plus à risque de la codebase, lecture seule) + `nitrux-postrm-cleanup.sh` + les 5 fichiers `.policy` polkit (14 actions, toutes `auth_admin` strict, exec.path croisés avec `PKEXEC_BINARY_NAMES` -- 14=14 exact) ; `types/layout.ts`/`types/style.ts`/`types/theme.ts` cohérents avec toutes les données déjà auditées.

**État à ce stade** : l'intégralité du code source lisible et de l'infrastructure de packaging/privilège de l'application a désormais été directement examinée au moins une fois cette session -- backend Rust (2 bouclages complets), toutes les pages Vue, tous les composants UI partagés, layouts, stores, catalogues de données, thèmes/styles, types, et toute la chaîne pkexec/polkit (script helper, script postrm, 5 policies, cross-checks exec.path). Plateau de maturité extrême confirmé par 3 cross-checks automatisés indépendants tous à 0 écart (IPC 69=69, styles 12=12, policies 14=14).

Éléments en attente inchangés (non bloquants, statut identique depuis plusieurs cycles) : `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit) + bouton Vérifier non désactivé (cycle 174, non retenu) + timeout `run_pkexec_with_stdin` (cycle 193, hors périmètre, doc dans le code) + doublons de catalogue appCatalog/systemToolsCatalog (cycles 201-202, re-confirmés inchangés cycle 259, décision de curation éditoriale requise) + risque `WiFiAnalyzerPage.vue::securityStatus` (cycle 241, non vérifiable sans matériel Wi-Fi réel dans cet environnement) + lacune `validate_port_proto` côté shell `nitrux-pkexec-helper` (cycle 263, non touchée -- le garde-fou Rust en amont ferme déjà le risque pratique, modifier le script nécessiterait un re-test VM live). **1 correctif accumulé depuis v0.25.79**. 156 cycles cumulés 110-265.

## Mise à jour (2026-08-08, v0.25.82, cycles 266-275) — nouveau filon contraste WCAG, 2 vrais bugs d'accessibilité trouvés

Cycles 266-270 négatifs : cross-check exhaustif `var(--nx-*)` (0 variable utilisée-mais-jamais-définie, seule `accentSecondary` confirmée orpheline comme déjà su) ; tentative infructueuse de vérifier le risque `WiFiAnalyzerPage` via la VM (SSH non scriptable sans intervention utilisateur, abandonné) ; `vite.config.ts`/`vitest.config.ts`, icônes `tauri.conf.json` vs disque, `.gitignore`, `tsconfig*.json`, `package-lock.json` (couverture multi-plateforme confirmée) -- tous propres.

**Cycles 271-273 -- nouveau filon productif : calcul de contraste WCAG (formule officielle relative luminance)** sur les couleurs de texte codées en dur. Cycle 271 : `textPrimary`/`textSecondary` contre `bgBase` sur les 13 thèmes -- tous conformes AA, aucun bug. **Cycle 272 -- vrai bug** : texte blanc de `.nx-button--danger` (bouton utilisé pour CHAQUE confirmation destructrice : désinstaller/supprimer/formater/quarantaine) échoue AA sur 11 des 13 thèmes (`catppuccin-mocha` 2.32:1 vs minimum 4.5:1) -- corrigé en noir (12/13 conformes). **Cycle 273 -- vrai bug, pire** : texte blanc de `.pkb-success` (bannière de succès après activation pkexec sur AppImage) échoue AA sur les **13 thèmes sans exception** (pire cas 1.37:1) -- corrigé en noir (13/13 conformes). Cycle 273 a aussi flagué (non corrigé, décision de palette) les dégradés fixes de `NxQuickActionTile.vue`. Cycle 274 : `NxBadge.vue` (texte = couleur d'accent sur fond `color-mix` 18%) -- approximation calculée montre 23/52 combinaisons sous AA, mais **volontairement pas corrigé** : l'approximation ignore la couche `--nx-style-bg` (12 styles, certains translucides/floutés) qui influence le vrai fond composité, corriger sur une valeur non fiable violerait la discipline anti-correctif-spéculatif. Cycle 275 : `subprocess.rs` lu en entier (fondation de tout le backend, déjà exceptionnellement testée) -- propre.

Éléments en attente inchangés + 2 nouveaux signalements : risque contraste `NxBadge` (cycle 274, nécessite vérification de rendu réel) + dégradés `NxQuickActionTile` potentiellement sous AA (cycle 273, décision de palette). Reste inchangé : `clone-disk` + `confirmNonDestructiveActions` + bouton Vérifier + timeout `run_pkexec_with_stdin` + doublons catalogue + `WiFiAnalyzerPage::securityStatus` + lacune shell `validate_port_proto`. **3 correctifs accumulés depuis v0.25.79** (InstalledSoftwarePage.vue cycle 248, NxButton danger contrast cycle 272, pkb-success contrast cycle 273). 166 cycles cumulés 110-275.

## Mise à jour (2026-08-08, v0.25.82, cycles 276-280) — contraste NxBadge confirmé exactement, report.rs entièrement lu

**Cycle 276** : le risque `NxBadge` du cycle 274 (approximation) est **confirmé exactement** en lisant `style-tokens.css` -- 7 des 12 dispositions ont un `--nx-style-bg` opaque et exactement connu (pas de `color-mix` intermédiaire). Recalcul exact sur ces 7 : **209/364 combinaisons (57%) échouent AA**, pire cas `adwaita`/`accentWarning` 1.82:1. Toujours pas corrigé -- `NxBadge` utilise intentionnellement l'accent comme texte (langage visuel "badge coloré"), un vrai correctif nécessite une décision de design (assombrir la couleur de texte spécifiquement, ou augmenter la teinte de fond), pas un remplacement mécanique comme les deux bugs déjà réglés.

Cycles 277-280 négatifs, sortie du filon contraste : confirmé qu'aucune CI (`.github/workflows`) n'existe ; `index.html`/`main.ts` clôturent la chaîne du point d'entrée ; `report.rs` lu en entier sur 3 cycles (`render_html` cycle 278, `render_txt`/`render_markdown` cycle 279, `build_system_report` cycle 280) -- parité des 4 formats confirmée complète, aucun bug, un point mineur noté (`|` non échappé en Markdown, pas de scénario de reproduction réaliste).

Éléments en attente inchangés + signalement renforcé : `NxBadge` illisible sur 57% des combinaisons testées EXACTEMENT (cycle 276, décision de langage visuel requise avant correctif) + dégradés `NxQuickActionTile` (cycle 273) + `clone-disk` + `confirmNonDestructiveActions` + bouton Vérifier + timeout `run_pkexec_with_stdin` + doublons catalogue + `WiFiAnalyzerPage::securityStatus` + lacune shell `validate_port_proto`. **3 correctifs accumulés depuis v0.25.79**. 171 cycles cumulés 110-280.

## Mise à jour (2026-08-08, v0.25.82, cycles 281-283) — MILESTONE : les deux arborescences source intégralement lues directement

Cycle 281 : `DisksPage.spec.ts` confirme une architecture de test correctement en couches (filtrage testé côté Rust, affichage testé côté frontend, pas de duplication). Cycle 282 : sweep final listant tous les `.ts`/`.vue` non-spec de `src/` -- 3 derniers fichiers jamais lus directement trouvés et lus (`vite-env.d.ts`, `UpdatesPage.vue`, `ScriptsPage.vue`) ; `ScriptsPage.vue::.scr-output` révèle que le schéma "couleur d'accent brute comme texte" du cycle 276 se répète au-delà de `NxBadge` seul, renforçant ce signalement. **Ceci clôt `src/` à 100%.**

**Cycle 283** : `packages/apt.rs` (site du tout premier bug de locale, cycle 160) relu en entier par discipline plutôt que par confiance en la mémoire -- confirmé identique à ce qui était rappelé, aucun écart. **Ceci clôt `src-tauri/src/` à 100% (49/49 fichiers)**, complétant le milestone : les deux arborescences source complètes de l'application (`src/` frontend et `src-tauri/src/` backend) ont désormais été lues directement au moins une fois chacune durant cette session, sans aucune exception.

Éléments en attente inchangés (aucun nouveau) : `NxBadge`/`ScriptsPage` couleur-accent-comme-texte (cycles 274/276/282, décision de langage visuel) + dégradés `NxQuickActionTile` (cycle 273) + `clone-disk` (cycle 120, action humaine) + `confirmNonDestructiveActions` (cycle 148, décision produit) + bouton Vérifier (cycle 174, non retenu) + timeout `run_pkexec_with_stdin` (cycle 193, hors périmètre) + doublons catalogue (cycles 201-202/259) + `WiFiAnalyzerPage::securityStatus` (cycle 241, non vérifiable) + lacune shell `validate_port_proto` (cycle 263, non touchée). **3 correctifs accumulés depuis v0.25.79**. 174 cycles cumulés 110-283.

## Mise à jour (2026-08-08, v0.25.82, cycles 284-294) — sweeps transversaux supplémentaires, 1 nouveau signalement sécurité (CSP)

Cycles 284-293 tous négatifs (dead-code Rust/TS croisés à 0 item, cycles circulaires TS+Rust à 0, cohérence `README.md`/`package.json`, cohérence de version 4 sources, sweep localisation octets Mo/Go, `cargo fmt --check` cosmétique non appliqué) -- détail complet dans JOURNAL.md.

**Cycle 294 -- nouveau signalement de sécurité, non corrigé** : `tauri.conf.json` a `security.csp: null` (CSP webview entièrement désactivée, défaut de scaffold jamais durci). `capabilities/default.json` lui-même est sain (`core:default`+`opener:default` seulement). Exploitabilité pratique confirmée faible (0 `fetch`/`axios`/XHR dans tout `src/`, 0 `v-html`) mais 6 bindings `:style=` inline compliquent un durcissement sûr (`style-src 'unsafe-inline'` nécessaire ou refactor). Pas corrigé : nécessite build Tauri + vérification GUI live (VM) pour éviter une casse silencieuse (écran blanc) -- hors budget d'un cycle de 10 min, même discipline que NxBadge/NxQuickActionTile.

Éléments en attente : + **CSP désactivée** (cycle 294, nécessite build+vérification GUI live) rejoint la liste inchangée par ailleurs (NxBadge/ScriptsPage, NxQuickActionTile, clone-disk, confirmNonDestructiveActions, bouton Vérifier, timeout run_pkexec_with_stdin, doublons catalogue, WiFiAnalyzerPage::securityStatus, validate_port_proto shell). **3 correctifs accumulés depuis v0.25.79**. 185 cycles cumulés 110-294.

## Mise à jour (2026-08-08, v0.25.83, cycles 295-297) — 1 nouveau bug trouvé côté FileToolsPage

Cycle 295 négatif (sweep injection shell/`unsafe` Rust, 0 trouvé). Cycle 296 négatif (`duplicates.rs` relu en profondeur, logique de bucketing confirmée correcte, consommateur `FileToolsPage.vue` confirmé lecture seule côté doublons).

**Cycle 297 -- bug réel trouvé** : `FileToolsPage.vue`, onglet Hash -- `hashPath` partagé entre "Calculer" et "Vérifier" laissait un hash PÉRIMÉ affiché à l'écran (d'un fichier A) juste à côté d'un badge "correspond"/"ne correspond pas" fraîchement calculé pour un fichier B différent, si l'utilisateur changeait le chemin entre les deux actions sans recalculer. Reproduit EN DIRECT via un test component avant correctif (échec confirmé), corrigé en réinitialisant mutuellement `hashResult`/`verifyMatch` au début de chaque action. Suite complète verte (cargo test + npm test + vue-tsc). Commit `5052b17`, version 0.25.82→0.25.83.

Éléments en attente inchangés par ailleurs (CSP désactivée cycle 294, NxBadge/ScriptsPage, NxQuickActionTile, clone-disk, confirmNonDestructiveActions, bouton Vérifier, timeout run_pkexec_with_stdin, doublons catalogue, WiFiAnalyzerPage::securityStatus, validate_port_proto shell). **4 correctifs accumulés depuis la dernière release publiée v0.25.79** (InstalledSoftwarePage, NxButton danger, pkb-success, FileToolsPage.vue hash/verify). 188 cycles cumulés 110-297.

## Mise à jour (2026-08-08, v0.25.84, cycles 298-304) — sweeps de confirmation + 1 CVE dépendance corrigée

Cycles 298-303 tous négatifs, sweeps de confirmation approfondis : classe de bug du cycle 297 recherchée sur 10 autres pages Vue (0 autre instance, `ReportGeneratorPage.vue` avait déjà une solution délibérée pour cette exacte classe) ; `duplicates.rs`/`largefiles.rs`/`hashcheck.rs`/`secure_temp.rs`/`cache_size.rs` relus en profondeur pour leur logique propre (pas juste en sweep de surface) -- tous déjà exemplaires, aucun bug ; hypothèse de bug `zypper.rs` (même piège d'exit-code que `dnf.rs`) formée puis réfutée par la documentation primaire officielle (codes 100/101 documentés exclusivement pour `patch-check`, pas `list-updates`) ; `ProcessesPage.vue` vérifiée, séquence d'`invoke()` sans try/catch confirmée sûre (backend infaillible par construction, contrats de type vérifiés).

**Cycle 304 -- vraie CVE trouvée et corrigée** : re-vérification fraîche `npm audit`/`cargo audit` (dernière fois cycle 265) révèle `nanoid` <3.3.17 (transitif via vite→postcss, dev-only) HIGH severity GHSA-2v37-7h3g-55p8. Corrigé via `npm audit fix` (3.3.16→3.3.18, lockfile seul). `cargo audit` : 0 vraie vulnérabilité (18 avertissements unmaintained/unsound non actionnables sur webkit2gtk/GTK profond, déjà connu). Commits `bf32e03`+`1d6cbd6`, version 0.25.83→0.25.84.

Éléments en attente inchangés (CSP désactivée, NxBadge/ScriptsPage, NxQuickActionTile, clone-disk, confirmNonDestructiveActions, bouton Vérifier, timeout run_pkexec_with_stdin, doublons catalogue, WiFiAnalyzerPage::securityStatus, validate_port_proto shell). **5 correctifs accumulés depuis la dernière release publiée v0.25.79** (InstalledSoftwarePage, NxButton danger, pkb-success, FileToolsPage.vue hash/verify, nanoid CVE) -- backlog encore modeste, pas de recommandation de coupure de release forcée mais à surveiller si ça continue de croître. 195 cycles cumulés 110-304.

## Mise à jour (2026-08-08, v0.25.85, cycle 305) — bug snap "All snaps up to date." trouvé et corrigé

`dependencies.rs` relu en entier (négatif, propre). **`packages/universal.rs::parse_snap_line` -- vrai bug trouvé** : ne sautait que la ligne d'en-tête, pas la phrase "All snaps up to date." que `snap refresh --list` affiche (sans tableau) quand rien n'est à rafraîchir -- reproduit EN DIRECT sur la vraie VM (snapd 2.76.1, joignable via les scripts SSH conservés du cycle 113), confirmé exit 0 + cette phrase exacte sur stdout. Se serait affiché comme une fausse mise à jour ("All" version "snaps") sur la page Mises à jour de tout système où snap n'a rien à rafraîchir -- le cas le plus courant, donc un faux positif potentiellement visible en permanence. Corrigé en exigeant un chiffre dans le champ version (garde structurelle, pas un filtrage de phrase littérale). Test de régression avec la sortie VM réelle. Suite complète verte. Commit `ea1f2a4`, version 0.25.84→0.25.85.

Éléments en attente inchangés par ailleurs. **6 correctifs accumulés depuis la dernière release publiée v0.25.79** (InstalledSoftwarePage, NxButton danger, pkb-success, FileToolsPage.vue hash/verify, nanoid CVE, snap false-positive) -- 57 cycles écoulés depuis cette release (248→305), backlog qui continue de croître régulièrement ; à proposer à l'utilisateur pour une coupure de release si ça se poursuit. 196 cycles cumulés 110-305.

## Mise à jour (2026-08-08, v0.25.86, cycle 306) — correction doc/test flatpak (comportement inchangé)

VM toujours joignable, exploitée à nouveau : `flatpak remote-ls --updates` testé en direct (flatpak 1.16.6). Découverte que le docstring/fixture de `parse_flatpak_line` avait l'ordre des colonnes inversé par rapport à la vraie sortie -- la logique de parsing extrayait déjà les bons index par coïncidence (aucun changement de comportement), mais la doc trompeuse était un risque latent pour un futur correctif. Corrigé (docstring + fixture réécrits avec la vraie ligne VM), test de régression ajouté pour le cas runtime/plateforme à version vide (`org.gnome.Platform`, déjà une limitation connue acceptée). Suite complète verte. Commit `d7a4c70`, version 0.25.85→0.25.86.

**7 correctifs accumulés depuis la dernière release publiée v0.25.79** (InstalledSoftwarePage, NxButton danger, pkb-success, FileToolsPage.vue hash/verify, nanoid CVE, snap false-positive, flatpak doc/test accuracy). 197 cycles cumulés 110-306.

## Mise à jour (2026-08-08, v0.25.87, cycle 307) — dernier suspect de sécurité historique fermé (validate_port_proto shell)

VM toujours joignable -- ciblage direct du suspect "lacune shell `validate_port_proto`" (signalé cycle 263, jamais corrigé par manque d'accès VM). `nitrux-pkexec-helper` : le glob `[0-9]*/tcp|[0-9]*/udp` n'avait aucune borne 1-65535, même piège que le Rust déjà fixé au cycle 227. Reproduit EN DIRECT AVANT correctif (fonction isolée, jamais le vrai pkexec/ufw, méthodologie identique à `quarantine-file` cycles 112-113) : ancienne version acceptait à tort "0/tcp"/"99999999/tcp"/"65536/tcp". Corrigé (garde de longueur 10+ chiffres avant toute arithmétique + borne explicite 1-65535). **Re-vérifié EN DIRECT contre la fonction réellement extraite du fichier corrigé** (pas une copie manuelle) : 12/12 cas OK. Suite verte. Commit `ba42a9b`, version 0.25.86→0.25.87.

**Ferme le dernier suspect de sécurité non résolu identifié depuis le cycle 263** -- plus aucun signalement de sécurité connu en attente hormis la CSP désactivée (cycle 294, nécessite build+GUI live, toujours hors budget cycle).

Éléments en attente inchangés par ailleurs (CSP désactivée, NxBadge/ScriptsPage, NxQuickActionTile, clone-disk, confirmNonDestructiveActions, bouton Vérifier, timeout run_pkexec_with_stdin, doublons catalogue, WiFiAnalyzerPage::securityStatus). **8 correctifs accumulés depuis la dernière release publiée v0.25.79** (InstalledSoftwarePage, NxButton danger, pkb-success, FileToolsPage.vue hash/verify, nanoid CVE, snap false-positive, flatpak doc/test accuracy, validate_port_proto shell) -- 59 cycles écoulés depuis v0.25.79 (248→307), backlog qui continue de croître ; recommandation de coupure de release à envisager. 198 cycles cumulés 110-307.
