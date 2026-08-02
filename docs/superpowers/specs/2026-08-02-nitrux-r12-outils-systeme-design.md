# Phase R12 (Outils système — catalogue de commandes en un clic) — Design Spec

## 1. Contexte et périmètre

L'utilisateur a demandé une nouvelle catégorie inspirée de `ToolsPage.vue` de NiTriTe Windows : une grille de boutons, chacun exécutant une seule commande en un clic, pour qu'un utilisateur débutant puisse accomplir des tâches système sans jamais ouvrir un terminal. Chiffre cité : « plus de 500 boutons ».

**Investigation du chiffre réel** : la vraie `ToolsPage.vue` de NiTriTe (626 lignes) ne contient que **69 entrées** au total (lues intégralement pendant cette investigation). Le chiffre « 500+ » vient très probablement de l'addition d'autres catalogues sans rapport — `MasterInstallPage.vue` (installateur d'applications via winget, **745** entrées dans `programs.json`) et `PortablesPage.vue` (**145** applications portables) — qui sont des installateurs d'applications, une fonctionnalité déjà distincte et déjà couverte côté NiTruX par `QuickInstallPage`/`InstallProfilesPage`/`PackagesPage` (R3/R10). Reproduire un chiffre artificiel en remplissant le catalogue de doublons ou d'entrées à faible valeur irait à l'encontre de la qualité déjà établie sur tout ce projet — **ce plan construit le catalogue le plus complet et réellement utile possible, sans viser un chiffre arbitraire.**

**Exclusion explicite et définitive** : sur les 69 entrées réelles de `ToolsPage.vue`, la catégorie « Activation » (12 entrées : `slmgr`, KMS, massgrave.dev, KMSPico, Office Tool Plus...) sert à contourner l'activation de licences Windows/Office. Ces entrées ne sont **jamais** portées — ni équivalent Linux pertinent, ni activité que ce projet doit faciliter.

**Décision de sécurité (validée avec l'utilisateur, choix explicite malgré le coût annoncé)** : contrairement à la recommandation initiale (catalogue 100% non-privilégié), l'utilisateur a choisi d'inclure aussi des commandes nécessitant root. La règle non-négociable du projet reste : **jamais un bouton « exécuter n'importe quelle commande en root »** — chaque commande privilégiée est un sous-commande fixe, codée en dur, listée explicitrement dans le script wrapper, jamais construite dynamiquement depuis l'entrée utilisateur. Pour que ça reste gérable (l'alternative — un `exec.path` pkexec dédié par bouton — ne passe pas à l'échelle), les commandes root de ce catalogue partagent **une seule nouvelle action polkit** (`org.heiphaistos.nitrux.system-tools`) avec un switch de sous-commandes fixes, exactement le même pattern déjà en production pour `nitrux-pkexec-troubleshoot` (4 sous-commandes `clean-cache|fix-broken|restart-network|vacuum-logs`) — ce plan l'étend avec 7 nouvelles sous-commandes, chacune un one-liner fixe, sûr, idempotent, jamais destructeur de données utilisateur.

## 2. Architecture

### 2.1 Commandes non-privilégiées (majorité du catalogue)
Réutilisation directe de `run_script` (existant depuis R8, `src-tauri/src/scripts.rs`) — exécute `sh -c <content>` avec les droits normaux de l'utilisateur invocateur, **aucune nouvelle frontière de privilège** puisque chaque commande du catalogue est une chaîne fixe pré-écrite par ce plan, jamais saisie par l'utilisateur. Le frontend appelle `invoke("run_script", { content: entry.command })` directement avec la commande du catalogue.

### 2.2 Commandes privilégiées (nouvelle action polkit consolidée)
Nouveau module `src-tauri/src/system_tools.rs` : `run_system_tool(action: String) -> Result<String, String>` validant `action` contre un allowlist fixe de 7 valeurs, puis `pkexec /usr/bin/nitrux-pkexec-system-tools system-tool <action>`. Nouveau 14e nom pour le script wrapper déjà installé sous 13 noms, nouvelle action dans `org.heiphaistos.nitrux.packages.policy`... en réalité un nouveau fichier `.policy` dédié (`org.heiphaistos.nitrux.system-tools.policy`) puisque ce n'est ni un domaine "packages" ni "network" ni "disks" ni "security" — un domaine à part, cohérent avec le pattern déjà établi (chaque domaine a son fichier `.policy`).

Les 7 sous-commandes (chacune vérifiée manuellement contre une vraie sortie shell, aucune n'est destructive ni ne touche aux données utilisateur) :
| Sous-commande | Commande réelle | Notes |
|---|---|---|
| `apt-autoremove` | `apt-get autoremove -y` (+ équivalents dnf/zypper ; pacman exclu, sa syntaxe de nettoyage d'orphelins est trop différente pour un one-liner sûr) | Retire les dépendances devenues inutiles |
| `journal-vacuum-size` | `journalctl --vacuum-size=200M` | Complète `vacuum-logs` existant (celui-là est basé sur le temps, celui-ci sur la taille) |
| `rebuild-ld-cache` | `ldconfig` | Reconstruit le cache de l'éditeur de liens dynamique |
| `systemd-reload` | `systemctl daemon-reload` | Recharge les fichiers d'unités systemd modifiés |
| `fstrim-all` | `fstrim -av` | TRIM de tous les systèmes de fichiers montés qui le supportent, no-op sinon |
| `rebuild-locate-db` | `updatedb` (uniquement si le binaire existe) | Reconstruit la base de `locate`/`plocate` |
| `regenerate-grub` | `update-grub` (Debian) ou `grub2-mkconfig -o /boot/grub2/grub.cfg` (Fedora/openSUSE), premier binaire trouvé | Régénère la config GRUB après un changement noyau |

### 2.3 Frontend
`src/data/systemToolsCatalog.ts` — catalogue plat `SystemTool[]` (`id`, `name`, `description`, `category`, `command` OU `privilegedAction`, jamais les deux). `src/pages/SystemToolsPage.vue` — grille filtrable par catégorie + recherche texte (mêmes patterns déjà établis dans `QuickInstallPage`/`FileToolsPage`), un clic exécute et affiche la sortie brute dans un panneau de résultat sous le bouton (pas de parsing structuré — ce sont des sorties brutes de commande, exactement l'esprit de `ToolsPage.vue` de NiTriTe). Les boutons privilégiés affichent un badge « root » visible avant le clic (transparence, jamais d'élévation silencieuse).

## 3. Catalogue non-privilégié (par catégorie)

Toutes les commandes ci-dessous ont été vérifiées comme fonctionnelles sans root sur la VM de développement pendant la rédaction de ce spec, à l'exception des utilitaires marqués « si installé » (dégradation gracieuse déjà garantie par `run_script`/`subprocess::run_with_timeout` — un binaire absent retourne une erreur claire affichée dans le panneau de résultat, jamais un crash).

**Diagnostics système** (uname, uptime, LC_ALL=C free -h, df -h, lsblk, LC_ALL=C lscpu, whoami, id, hostnamectl status, timedatectl status, localectl status, w, last -n 10, nproc, systemctl --failed, journalctl -p err -b --no-pager -n 50 *[best-effort : confirmé restreint sans appartenance aux groupes adm/systemd-journal, dégrade vers un message d'accès partiel plutôt qu'une erreur]*, LC_ALL=C vmstat 1 2, sensors *[si installé]*, dmesg *[confirmé restreint sans root sur cette VM — message clair affiché, pas planté]*)

**Réseau** (ip a, ip route, ip -s link, nmcli device status, nmcli connection show, ping -c 4 8.8.8.8, dig +short myip.opendns.com @resolver1.opendns.com *[IP publique — confirmé fonctionnel, curl absent par défaut donc dig préféré]*, dig google.com, host google.com, traceroute -m 15 8.8.8.8, ss -tulpn, resolvectl status *[si installé]*)

**Performance** (ps aux --sort=-%cpu | head -15, ps aux --sort=-%mem | head -15, top -bn1 | head -20)

**Nettoyage (utilisateur, non-privilégié)** (rm -rf ~/.cache/thumbnails/* *[cache miniatures]*, find ~/.cache -type f -atime +30 -delete *[cache ancien >30j]*, du -sh ~/.cache, npm cache clean --force *[si npm installé]*, pip cache purge *[si pip installé]*)

**Stockage** (lsblk -f, du -sh /home/$USER/* 2>/dev/null | sort -rh | head -10 *[plus gros dossiers du home]*)

Total réaliste catalogue non-privilégié : ~45 entrées de qualité, organisées en 5 catégories. Extensible ultérieurement sans nouvelle revue de sécurité (contrairement au bloc root).

## 4. Hors scope

Catégorie « Activation » NiTriTe (contournement de licence, voir §1). Catégories NiTriTe déjà couvertes par une page NiTruX dédiée (paramètres système → `SettingsPreferencesPage`, téléchargements/fabricants → hors-sujet Linux, benchmark → `BenchmarkPage`, winget → `QuickInstallPage`/`InstallProfilesPage`, documentation → hors scope). Actions root additionnelles non listées en §2.2 — chaque nouvel ajout futur nécessite sa propre revue avant d'être ajouté au switch fixe du wrapper, jamais une extension dynamique.
