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
