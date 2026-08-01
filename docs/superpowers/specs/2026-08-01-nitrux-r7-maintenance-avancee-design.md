# NiTruX Phase R7 — Maintenance avancée — Design

## 1. Contexte

Deuxième phase du second round de refonte (voir `docs/superpowers/specs/2026-08-01-nitrux-r6-visual-foundation-performance-design.md` §1 pour le découpage complet R6-R11, déjà validé avec l'utilisateur). R7 couvre le groupe "Maintenance avancée" de `navigation.ts` (NiTriTe Windows) : Désinstalleur, Nettoyeur avancé, Sauvegarde, Antivirus dédié, Dépendances.

**Investigation concrète menée avant de spécifier** (lecture du code existant, pas de supposition) :
- `nitrux-pkexec-helper` a déjà un subcommand `troubleshoot` avec les actions `clean-cache` (vide le cache du gestionnaire de paquets détecté) et `vacuum-logs` (purge journalctl >7j) — **déjà entièrement implémentées et déjà exposées** via `run_troubleshoot_action`, actuellement seulement accessibles depuis un onglet de `TroubleshootPage.vue`. Le "Nettoyeur avancé" n'a donc **aucun nouveau code privilégié à écrire** pour son cœur — seulement une meilleure présentation dédiée + une nouvelle fonctionnalité non-privilégiée (aperçu de taille avant nettoyage).
- `scan_for_malware`/`quarantine_file` existent déjà et sont pleinement fonctionnels, actuellement dans le même onglet groupé `TroubleshootPage.vue`.
- Le pattern `PackageManager` trait (`packages/mod.rs`) + 4 implémentations par gestionnaire (`apt.rs`/`dnf.rs`/`pacman.rs`/`zypper.rs`) est déjà établi pour `list_upgradable()` — s'étend naturellement à un nouveau `list_installed()` pour le Désinstalleur, en suivant exactement le même schéma de parsing testé unitairement par littéral (indépendant de la machine réelle).
- Le pattern de confirmation tapée pour une action destructive (`format_partition` dans `DisksPage.vue` : l'utilisateur doit retaper le nom exact avant que le bouton s'active) est déjà établi et sera réutilisé pour la désinstallation de paquet.

## 2. Restructuration de `TroubleshootPage.vue`

Actuellement 3 onglets : Scan malware, Snapshots, Dépannage (fix-broken/restart-network/clean-cache/vacuum-logs). Cette phase en extrait deux vers leurs propres pages dédiées (même pattern que le split `DisksPage`→`DisksPage`+`FileToolsPage` en R2, pas une duplication comme R4's Updates/Packages — ce contenu appartient conceptuellement à une nouvelle catégorie, pas à deux endroits distincts) :

- **Antivirus dédié** (nouvelle page `AntivirusPage.vue`) — scan malware + quarantaine, retiré de `TroubleshootPage.vue`. Réutilise `scan_for_malware`/`quarantine_file` à l'identique, zéro changement backend.
- **Nettoyeur avancé** (nouvelle page `CleanerPage.vue`) — `clean-cache`/`vacuum-logs`, retiré de `TroubleshootPage.vue`. Réutilise `run_troubleshoot_action` à l'identique pour ces deux actions, plus une nouvelle fonctionnalité non-privilégiée : aperçu de taille (cache utilisateur `~/.cache`, taille du cache du gestionnaire de paquets détecté si lisible sans privilège) avant de nettoyer.

`TroubleshootPage.vue` après cette phase : 2 onglets (Snapshots, Dépannage avec seulement `fix-broken`/`restart-network`). Toute la logique métier autour de ces deux actions restantes reste identique.

## 3. Nouvelles pages

### 3.1 Dépendances (`DependenciesPage.vue`) — lecture seule

Nouveau module backend `src-tauri/src/dependencies.rs` : scanne une liste fixe de binaires système courants (`/bin`, `/usr/bin` — utilise `ldd` sur chaque exécutable trouvé) pour détecter des bibliothèques partagées manquantes ("not found" dans la sortie `ldd`). Aucune opération privilégiée — `ldd` sur un binaire ne nécessite jamais root. Résultat honnête : liste des binaires avec au moins une dépendance manquante, avec le nom de la bibliothèque manquante. Si aucun problème n'est trouvé (cas normal sur un système sain), message clair "Aucune dépendance manquante détectée" plutôt qu'une liste vide silencieuse.

### 3.2 Sauvegarde (`BackupPage.vue`)

Nouveau module backend `src-tauri/src/backup.rs` : archive (`tar.gz`) un répertoire source choisi par l'utilisateur (par défaut `$HOME`, mais l'utilisateur peut restreindre à un sous-dossier) vers un fichier horodaté dans `$HOME` (ex: `~/nitrux-backup-2026-08-01-201530.tar.gz`). Aucune opération privilégiée — n'écrit et ne lit que dans l'espace utilisateur. Pas de dialogue de sélection de destination Tauri (même décision qu'en R5 pour le générateur de rapport — éviter une nouvelle dépendance de plugin) : le chemin de destination est fixe et prévisible, affiché à l'utilisateur après la création. Barre de progression indéterminée pendant la création (l'archivage peut prendre du temps selon la taille), pas de pourcentage précis (même honnêteté que la barre d'installation en R3 — `tar` n'expose pas de progression parseable simplement).

### 3.3 Désinstalleur (`UninstallerPage.vue`)

**Seule partie de R7 avec une nouvelle surface privilégiée** — suit exactement la discipline déjà établie tout au long du projet (chemin `exec.path` dédié, validation ré-effectuée côté script shell, jamais de confiance aveugle côté Rust seul, vérification live sur la VM Debian jetable avant merge) :

- Extension du trait `PackageManager` (`packages/mod.rs`) avec une nouvelle méthode `list_installed() -> Result<Vec<InstalledPackage>, String>`, implémentée pour les 4 gestionnaires (`dpkg -l` / `rpm -qa` / `pacman -Q` / `zypper se --installed-only`), même style de parsing littéral déjà utilisé pour `list_upgradable`. Nouvelle commande `list_installed_packages() -> Result<Vec<InstalledPackage>, String>` (lecture seule, agrège tous les gestionnaires détectés comme le fait déjà `list_updates`).
- Nouvelle action pkexec `uninstall-package` dans `nitrux-pkexec-helper`, installée sous son propre nom dédié `nitrux-pkexec-uninstall-package` (jamais partagé avec `nitrux-pkexec-install-package` — même piège de résolution d'action déjà rencontré et documenté dans ce projet). Réutilise `validate_package_name`/`validate_manager` du script existant à l'identique. Nouvelle commande Rust `uninstall_package(manager, package) -> Result<String, String>`, miroir exact de `install_package` (même validation, même timeout, même pattern d'invocation).
- UI : liste des paquets installés (recherche/filtre textuel côté client, pas de nouvelle capacité backend), désinstallation protégée par confirmation tapée (l'utilisateur retape le nom exact du paquet avant que le bouton s'active) — même pattern que `format_partition` dans `DisksPage.vue`.
- **Vérification obligatoire avant merge** : test live sur la VM Debian jetable avec un paquet réellement installé puis désinstallé sans danger (ex: installer `sl` ou `cowsay` via `install_package` déjà existant, puis le désinstaller via cette nouvelle commande, confirmer la disparition réelle).

## 4. Hors scope pour R7

- Toute action de désinstallation en masse / sélection multiple (une seule désinstallation à la fois dans cette v1).
- Sauvegarde incrémentale ou planifiée (une sauvegarde manuelle ponctuelle seulement).
- Restauration de sauvegarde depuis l'UI (créer un fichier `.tar.gz` compréhensible/restaurable manuellement suffit pour cette v1 — une UI de restauration dédiée serait une fonctionnalité séparée).
- R8-R11 (22 pages restantes du découpage validé).

## 5. Vérification

Même discipline que toutes les phases précédentes : tests unitaires Rust (parsing par littéraux, indépendant de la machine), tests Vitest par page, `vue-tsc --noEmit`, vérification manuelle sur la VM Debian pour les commandes qui touchent le système réel (`ldd`, `tar`, et surtout le cycle install→uninstall complet). Merge, version bump (0.14.0→0.15.0 attendu), build `.deb`/`.rpm`, release GitHub.
