# Checkpoint — Refonte R1-R5 terminée ✅, second round (R6-R11) en cours, R6+R7 faits

## État antérieur (référence uniquement)

Les 4 piliers originaux (lecture + écriture privilégiée, Phases 1 à 5 Part 2) sont **complets, publiés jusqu'à v0.8.0**. La refonte R1-R5 (nav catégorisée, 3 axes visuels, 4 nouvelles catégories) est **terminée, publiée jusqu'à v0.13.0**. Tout est mergé sur `master` — voir historique git pour le détail phase par phase.

## Second round de refonte : retour utilisateur post-R5

Après v0.13.0, l'utilisateur a jugé le résultat toujours "moche et fade" et signalé qu'il manquait ~75% des fonctionnalités de NiTriTe Windows. Investigation réelle : `navigation.ts` NiTriTe liste 44 pages/10 catégories vs 15 NiTruX. Cause du "fade" : icônes jamais rendues dans `AppNav.vue`, zéro lib d'icônes, `DashboardPage.vue` jamais componentisé.

**Découpage validé avec l'utilisateur en 6 phases R6→R11** (26 pages) — voir spec `2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1 pour le détail complet. Décisions de scope qui s'appliquent à TOUT le chantier R6-R11 : catégorie "Intelligence" hors scope, fonctionnalités Windows-only (WinPE/BSOD/WSL) adaptées en équivalents Linux (pas ignorées), nouvelles surfaces privilégiées toujours vérifiées en live VM avant merge.

## Phase R6 (Fondation visuelle + catégorie Performance) : TERMINÉE, publiée en v0.14.0

10 tâches. Icônes lucide-vue-next réellement rendues dans `AppNav.vue` (corrige toute la nav existante d'un coup), `NxQuickActionTile.vue` + 5 tuiles colorées sur le dashboard (enfin componentisé), nouvelle catégorie "Performance" (Optimisations lecture seule, Températures, Benchmark, Historique perf). 2 vrais bugs trouvés et corrigés pendant ma vérification finale (import inutilisé cassant vue-tsc, `categories.spec.ts` préexistant raté par le plan). Tests : 128→147 frontend, 132→142 Rust.

## Phase R7 (Maintenance avancée) : TERMINÉE, mergée, publiée en v0.15.0

Spec : `docs/superpowers/specs/2026-08-01-nitrux-r7-maintenance-avancee-design.md`. Plan : `docs/superpowers/plans/2026-08-01-nitrux-r7-maintenance-avancee.md` (9 tâches). 9 tâches exécutées et vérifiées indépendamment :
- Task 1 : `AntivirusPage.vue` extraite de `TroubleshootPage.vue` (malware scan+quarantaine, zéro backend).
- Task 2 : `cache_size.rs` (lecture seule, taille des caches) + `CleanerPage.vue` (réutilise `clean-cache`/`vacuum-logs` déjà existants). **Sous-agent coupé mi-tâche** (background cargo test), j'ai terminé la vérification+commit moi-même — le travail était en fait complet et correct.
- Task 3 : `dependencies.rs` (lecture seule, scan `ldd` sur binaires courants) + `DependenciesPage.vue`.
- Task 4 : `backup.rs` (non-privilégié, `tar.gz` vers `$HOME`) + `BackupPage.vue`.
- Task 5 : `list_installed()` étendu sur les 4 `PackageManager` (apt/dnf/pacman/zypper), mirroring `list_upgradable()`. Sous-agent a trouvé et corrigé 2 vrais problèmes : mon erreur d'arithmétique dans le plan (9 tests pas 8) et des collisions de noms de tests avec les tests `list_upgradable` déjà existants dans les mêmes fichiers.
- Task 6 : **nouvelle action pkexec `uninstall-package`**, seule surface privilégiée nouvelle de R7 — miroir exact d'`install_package`, chemin `exec.path` dédié (`nitrux-pkexec-uninstall-package`, jamais partagé), re-validation shell indépendante. Vérifié en détail par moi (diff ligne par ligne des 5 fichiers touchés).
- Task 7 : `UninstallerPage.vue` — confirmation tapée obligatoire avant désinstallation (même pattern que `format_partition`), vérifié match exact (pas partiel).
- Task 8 : branchement `App.vue` + extension catégorie "maintenance" (3→8 pages) + icônes `AppNav.vue` — fait directement par moi.
- Task 9 : vérification finale — 1 vrai bug trouvé et corrigé par moi (paramètre `args` inutilisé cassant `vue-tsc` dans `BackupPage.spec.ts`). **Cycle critique installation→désinstallation testé en LIVE sur la VM Debian** : paquet `sl` installé via `install-package` existant, confirmé présent (`dpkg -l` → `ii`), désinstallé via la NOUVELLE action `uninstall-package`, polkit a authentifié la bonne action distincte (`org.heiphaistos.nitrux.uninstall-package`, pas de confusion avec `install-package` — preuve que le chemin dédié fonctionne réellement), confirmé réellement absent après (`dpkg -l` → rien). Spot-check `ldd`/`tar` également réussi en direct sur la VM.

Tests : 147→160 frontend, 142→165 Rust.

Merge master (fast-forward propre) + version bump 0.14.0→0.15.0 + build réel (.deb/.rpm) + Cargo.lock committé — **push+tag+release restent à faire juste après ce checkpoint**.

## Prochaine action

Pousser master + créer le tag `v0.15.0` + créer la release GitHub avec les assets `.deb`/`.rpm`. Nettoyer le worktree `r7-maintenance-avancee`.

Puis écrire spec+plan pour **Phase R8 (Réseau & Terminal)** : WiFi Analyzer dédié, Bluetooth, Terminal intégré, Scripts & Snippets, DNS Switcher dédié (promotion depuis NetworkPage). Continuer le découpage R8→R11 déjà validé.

## Contexte pour reprendre à froid

- Repo local : `C:\Users\Momo\Desktop\NiTruX`, remote `https://github.com/Heiphaistos/NiTruX.git`, branche `master`, version `0.15.0`
- Worktrees mergés à nettoyer si encore présents : `r7-maintenance-avancee`
- **Découpage validé R6→R11** — spec `2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1 pour le tableau complet. R6 et R7 faits, reste R8 (Réseau & Terminal), R9 (Stockage avancé), R10 (Logiciels & déploiement), R11 (Diagnostic & config).
- VM Debian : `172.18.32.124`, user `dev`, password `1998`. Scripts SSH : `C:\Users\Momo\AppData\Local\Temp\claude\C--Users-Momo\880690b1-319b-40bd-bb2c-957700dc8af4\scratchpad\ssh_run.py`/`ssh_put.py`/`ssh_interactive.py` (usage `python ssh_run.py <user> <password> "<cmd>"`).
- **Pattern de vérification VM pour une nouvelle action pkexec** (établi en R7 Task 9, à réutiliser pour toute future action privilégiée) : `pkttyagent --process $$ & sleep 1; pkexec /usr/bin/nitrux-pkexec-<action> <sous-commande> <args>` via `ssh_interactive.py` (fournit le mot de passe sudo automatiquement) — la ligne `==== AUTHENTICATING FOR org.heiphaistos.nitrux.<action> ====` dans la sortie confirme que polkit a résolu la BONNE action distincte, pas une autre à cause d'un `exec.path` partagé.
- **Piège pkill (R6)** : `pkill -f <motif>` peut tuer son propre shell invocateur si la commande SSH envoyée contient littéralement le motif. Toujours `pkill -x <nom-exact>`.
- Lancement app sur VM : `DISPLAY=:1 WAYLAND_DISPLAY=wayland-0 XAUTHORITY=/run/user/1000/xauth_* DBUS_SESSION_BUS_ADDRESS=unix:path=/run/user/1000/bus XDG_RUNTIME_DIR=/run/user/1000 nohup /usr/bin/tauri-app > /tmp/log 2>&1 &` — re-dériver via `systemctl --user show-environment` si besoin.
- Capture d'écran VM (`spectacle -b -n -o /tmp/out.png`) parfois indisponible (D-Bus KWin ne répond plus) — se rabattre sur vérification fonctionnelle directe via SSH (comme fait pour `optimizations.rs`/`uninstall-package`/`dependencies.rs`/`backup.rs`).
- **Piège pkexec — discipline non-négociable pour toute nouvelle action privilégiée** : chaque action DOIT avoir son propre `exec.path` dédié (jamais partagé), le script shell doit re-valider indépendamment (jamais confiance aveugle dans la validation Rust), et le cycle complet doit être testé en live sur la VM avant merge — pas juste les tests unitaires de validation d'entrée.
- **Toujours vérifier l'existence de fichiers de test annexes avant d'écrire un plan** (leçon R6 Task 10, confirmée à nouveau en R7 Task 5 avec les collisions de noms de tests) : grep plus large avant de figer un plan touchant une structure de données ou un fichier partagé entre plusieurs tests.
- Pattern "unhandled rejection bénin dans App.spec.ts" (vu R2/R4/R5/R6/R7) : `ref.value = await invoke<T[]>(...)` puis accès sans garde null sous le mock global `invoke→null` — n'affecte aucun résultat de test, ne jamais "corriger" pour ce cas synthétique.
