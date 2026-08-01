# Checkpoint — Refonte R1-R5 terminée ✅, second round (R6+) en cours

## État antérieur (référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, publiés jusqu'à v0.8.0**. Toutes les opérations pkexec ont été vérifiées en live sur VM Debian jetable. La refonte R1-R5 (nav catégorisée 7 catégories, système visuel 3 axes, 4 nouvelles catégories fonctionnelles : Applications/Mises à jour/Pilotes/Rapports) a été **entièrement terminée et publiée jusqu'à v0.13.0** — voir historique git pour le détail phase par phase, tout est mergé sur `master`.

## Second round de refonte : retour utilisateur post-R5

Après v0.13.0, l'utilisateur a jugé le résultat toujours "moche et fade" et signalé qu'il manquait ~75% des fonctionnalités de NiTriTe Windows. **Investigation concrète menée (pas de supposition)** :
- `C:\Users\Momo\Desktop\Nitrite 2.0\src\data\navigation.ts` liste réellement **44 pages / 10 catégories**. NiTruX n'en avait que 15 (~34% de couverture).
- Cause concrète du "fade" trouvée : `categories.ts` avait déjà un champ `icon` par page mais `AppNav.vue` ne l'utilisait jamais (aucune icône rendue) ; **zéro librairie d'icônes installée** ; `DashboardPage.vue` jamais migré vers Nx*/tokens de style depuis R1.

Décisions utilisateur (via AskUserQuestion) :
- Pas de passe de fondation visuelle isolée — icônes/couleurs livrées avec la première vague de nouvelles pages, pas de retrofit séparé.
- Découpage en 6 phases thématiques **R6→R11** (26 pages au total) : R6 Performance (+ fondation visuelle), R7 Maintenance avancée, R8 Réseau & Terminal, R9 Stockage avancé, R10 Logiciels & déploiement, R11 Diagnostic & config.
- Catégorie "Intelligence" de NiTriTe (agent IA, base de connaissances) : **hors scope**.
- Fonctionnalités Windows-only (WinPE, BSOD, WSL) : à adapter en équivalents Linux pertinents (R10/R11), pas ignorées.
- Page "Optimisations" : **lecture seule** pour l'instant (pas de nouvelle surface pkexec sans revue dédiée), mais vérification live VM autorisée pour les diagnostics.
- Couleurs d'accent : uniquement sur les tuiles d'action rapide du dashboard pour l'instant, pas par catégorie de nav partout.

Spec : `docs/superpowers/specs/2026-08-01-nitrux-r6-visual-foundation-performance-design.md` (commit `07287a7`). Plan : `docs/superpowers/plans/2026-08-01-nitrux-r6-visual-foundation-performance.md` (commit `85729f9`, 10 tâches).

## Phase R6 (Fondation visuelle + catégorie Performance) : TERMINÉE, mergée, publiée en v0.14.0

10 tâches exécutées et vérifiées indépendamment :
- Task 1 : `lucide-vue-next` + icônes réellement rendues dans `AppNav.vue` (corrige d'un coup toute la nav existante, 15 pages, car `categories.ts` avait déjà les noms d'icônes).
- Task 2 : `NxQuickActionTile.vue` (icône + dégradé + label).
- Task 3 : `DashboardPage.vue` enfin componentisé sur Nx*/tokens de style + 5 tuiles d'action colorées.
- Task 4 : catégorie "Performance" ajoutée à `categories.ts` (7→8 catégories, 15→19 pages).
- Task 5 : `TemperaturesPage.vue` (réutilise `get_sensor_snapshot` existant, zéro backend).
- Task 6 : `benchmark.rs` (CPU via sha2 déjà dépendance, disque via fichier temp nettoyé, mémoire via boucle copie, zéro pkexec) + `BenchmarkPage.vue`.
- Task 7 : `NxSparkline.vue` (SVG fait main, zéro dépendance graphique) + `PerfHistoryPage.vue` (buffer glissant client, zéro backend).
- Task 8 : `optimizations.rs` (lecture seule : systemctl/swappiness/zram/fstrim) + `OptimizationsPage.vue`, **vérifié en live sur la VM** (sortie réelle confirmée conforme aux parsers).
- Task 9 : branchement final `App.vue` (4 nouvelles pages + événement `navigate` du dashboard).
- Task 10 : vérification finale — **2 vrais bugs trouvés et corrigés pendant la vérification, pas seulement rapportés par les sous-agents** : import `afterEach` inutilisé dans `PerfHistoryPage.spec.ts` (échec `vue-tsc`), et un fichier `categories.spec.ts` préexistant (raté par la recherche du plan) qui testait encore "7 catégories" — corrigé à 8 + test étendu pour les 4 nouveaux ids.

Tests : 128→147 frontend (net), 132→142 Rust (le plan avait sous-compté le bloc de tests d'`optimizations.rs`, 7 tests réels pas 6 — écart résolu en comptant le code réel, pas en suivant aveuglément l'estimation du plan).

**Vérification live VM** : build réel installé sur la VM Debian (172.18.32.124), lancé avec succès, process stable (confirmé après plusieurs secondes, log vide). Capture d'écran impossible cette fois — l'interface D-Bus de capture d'écran de KWin ne répondait plus sur cette session VM (`Did not receive a reply`), un problème d'environnement de la VM, pas du code NiTruX (le même outil avait fonctionné plus tôt dans la session pour v0.13.0). Compensé par : les tests couvrant explicitement les nouvelles pages via `App.spec.ts`, et les données `optimizations.rs` déjà vérifiées fonctionnellement contre la sortie VM réelle en Task 8.

Merge master (fast-forward propre) + version bump 0.13.0→0.14.0 + build réel (.deb/.rpm) + Cargo.lock committé — **push+tag+release restent à faire juste après ce checkpoint**.

## Prochaine action

Pousser master + créer le tag `v0.14.0` + créer la release GitHub avec les assets `.deb`/`.rpm`. Nettoyer le worktree `r6-visual-foundation-performance`.

Puis écrire spec+plan pour **Phase R7 (Maintenance avancée)** : Désinstalleur, Nettoyeur avancé, Sauvegarde, Dépendances, Antivirus dédié (promotion depuis l'onglet de TroubleshootPage). Continuer le découpage R7→R11 déjà validé avec l'utilisateur.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.14.0`
- Worktrees mergés à nettoyer si encore présents : `r6-visual-foundation-performance`
- **Découpage validé R6→R11** — voir spec `2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1 pour le tableau complet des 6 phases et leurs pages ; décisions de scope (Intelligence hors scope, Windows-only adaptés pas ignorés, Optimisations lecture seule) s'appliquent à tout le reste du chantier R6-R11, pas seulement R6.
- VM Debian : `172.18.32.124`, user `dev`, password `1998`. Script SSH réutilisable : `C:\Users\Momo\AppData\Local\Temp\claude\C--Users-Momo\880690b1-319b-40bd-bb2c-957700dc8af4\scratchpad\ssh_run.py` (usage `python ssh_run.py <user> <password> "<cmd>"`), `ssh_put.py`, `ssh_interactive.py` (pour sudo).
- **Piège pkill découvert en R6** : `pkill -f <motif>` matche aussi la ligne de commande de son propre process invocateur si cette ligne contient le motif (ex: `pkill -f tauri-app` envoyé via SSH exec_command tue le shell SSH lui-même puisque la commande envoyée contient littéralement "tauri-app") → connexion SSH qui meurt (exit -1, aucune sortie). Toujours utiliser `pkill -x <nom-exact-du-process>` pour tuer un process par nom depuis une commande SSH, jamais `-f`.
- Lancement app sur VM : `DISPLAY=:1 WAYLAND_DISPLAY=wayland-0 XAUTHORITY=/run/user/1000/xauth_* DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus XDG_RUNTIME_DIR=/run/user/1000 nohup /usr/bin/tauri-app > /tmp/log 2>&1 &` — valeurs exactes (surtout `xauth_*`) à re-dériver via `systemctl --user show-environment` sur la VM si elles ont changé (cookie Xwayland régénéré par session).
- Capture d'écran VM : `spectacle -b -n -o /tmp/out.png` (nécessite `WAYLAND_DISPLAY`+`XDG_RUNTIME_DIR`+`DBUS_SESSION_BUS_ADDRESS`) — peut échouer si l'interface D-Bus KWin ne répond plus (`Did not receive a reply`), pas forcément lié au code testé ; dans ce cas se rabattre sur lecture de code/tests + vérification fonctionnelle des commandes backend via SSH direct (comme fait pour `optimizations.rs` en Task 8).
- LayoutShell.vue : slot-based, les 8 layouts ne changent PAS.
- themeStore/layoutStore/styleStore : 3 axes visuels indépendants.
- **Piège privacy Rust (R5)** : un `fn` privé au niveau racine du crate (`lib.rs`) est visible depuis tous les modules descendants sans `pub(crate)`. Passer une fonction `#[tauri::command]` racine en `pub`/`pub(crate)` casse la compilation (collision macro `__cmd__<nom>` dans tauri-macros 2.6.3) — ne jamais changer sa visibilité sans test isolé d'abord.
- Piège race condition Vue (R3) : ne jamais gater un bouton sur un ref rempli par une promesse async démarrée dans `onMounted` si un clic peut arriver avant que le DOM patch soit flush.
- Pattern "unhandled rejection bénin dans App.spec.ts" (vu R2/R4/R5/R6) : `ref.value = await invoke<T[]>(...)` puis `.length` sans garde null lève une erreur non bloquante sous le mock global `invoke→null` — n'affecte aucun résultat de test, ne pas "corriger" le composant pour ce cas synthétique.
- **Toujours vérifier l'existence de fichiers de test/données annexes avant d'écrire un plan** (leçon R6 Task 10) : `categories.spec.ts` existait déjà et testait "7 catégories" séparément d'`App.spec.ts`, raté par la recherche initiale du plan R6 — grep plus large (`grep -rl "toHaveLength\|categories.length"`) avant de figer un plan touchant une structure de données partagée.
