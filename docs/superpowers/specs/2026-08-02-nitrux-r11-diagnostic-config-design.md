# Phase R11 (Diagnostic & config) — Design Spec

## 1. Contexte et périmètre

L'utilisateur a signalé que la page "Diagnostic" actuelle (`src/pages/DiagnosticPage.vue`, 51 lignes — une simple liste de périphériques PCI via `get_pci_devices`) est quasi-vide et n'est même pas sa propre catégorie de navigation (c'est une page parmi deux dans "Système"), alors que dans la référence NiTriTe Windows, `DiagnosticPage.vue` fait **493 lignes** et orchestre **~30 onglets** de diagnostic matériel/logiciel profond, avec sa propre barre latérale filtrable et export multi-format.

Reproduire les 30 onglets tels quels serait mal aligné avec NiTruX : beaucoup sont soit **sans équivalent Linux réel** (Licence/Activation Windows, Registre, analyse BSOD, WSL — inverse du concept), soit **déjà couverts par une page NiTruX dédiée existante** (pare-feu → `FirewallPage`, pilotes → `DriversPage`, benchmark → `BenchmarkPage`, nettoyeur → `CleanerPage`, hosts → onglet dans `NetworkPage`, boot → `BootManagerPage`, bluetooth → `BluetoothPage`, historique de perf → `PerfHistoryPage`, outils réseau/scan de ports → onglet dans `NetworkPage`, dossiers volumineux → `FileToolsPage`, réparation → `TroubleshootPage`, export système complet → `ReportGeneratorPage`). Dupliquer ces derniers créerait une redondance de nav, pas de la valeur.

**Décision de périmètre :** une nouvelle catégorie de navigation dédiée **"Diagnostic"** (8e→9e catégorie), regroupant la page PCI existante (déplacée depuis "Système", pas dupliquée) + 6 nouvelles pages couvrant les domaines de `DiagnosticPage.vue` de NiTriTe **qui n'ont ni équivalent NiTruX existant ni exclusion justifiée**. Chaque nouvelle page est vérifiée contre des commandes réelles testées sans privilège sur la VM de dev pendant la rédaction de ce spec (voir §3-8, chaque section cite la sortie réelle obtenue).

**Hors scope (décision explicite, aucun équivalent Linux pertinent ou déjà couvert) :** Licence/Activation Windows, Registre Windows, analyse BSOD (nécessiterait l'analyse de core dumps noyau — complexe, potentiellement une phase future dédiée), WSL, Certificats système (niche, différé), dossiers volumineux (déjà `FileToolsPage`), et tous les domaines listés ci-dessus déjà couverts par une page NiTruX existante.

## 2. Nouvelle catégorie de navigation

`categories.ts` gagne une 9e catégorie **"Diagnostic"**, insérée juste après "Système" (avant "Performance") :
```typescript
{
  id: "diagnostic-avance",
  title: "Diagnostic",
  pages: [
    { id: "diagnostic", label: "Composants PCI", icon: "stethoscope" },          // déplacée depuis "systeme"
    { id: "hardware-details", label: "Matériel détaillé", icon: "cpu" },
    { id: "peripherals", label: "Périphériques", icon: "monitor" },
    { id: "processes", label: "Processus & services", icon: "activity" },
    { id: "installed-software", label: "Logiciels installés", icon: "list" },
    { id: "user-accounts", label: "Comptes utilisateurs", icon: "users" },
    { id: "update-history", label: "Historique des mises à jour", icon: "history" },
  ],
},
```
La catégorie "Système" perd son entrée `diagnostic` (ne garde que `dashboard`). 4 nouvelles icônes lucide requises : `Cpu` (déjà importée), `Monitor`, `Activity`, `List`, `Users`, `History` (5 réellement nouvelles, `Cpu` déjà présente).

## 3. `HardwareDetailsPage.vue` — Matériel détaillé

Réel sur la VM de dev (`lscpu`, sans root) :
```
Nom de modèle : 11th Gen Intel(R) Core(TM) i5-11400H @ 2.70GHz
Processeur(s) : 6
Cœur(s) par socket : 3
Thread(s) par cœur : 2
```
`dmidecode` n'est **pas installé** sur cette VM et exige root pour lire la table SMBIOS de toute façon — **ne pas en dépendre**. En revanche `/sys/class/dmi/id/{product_name,bios_version,sys_vendor,board_name}` sont lisibles **sans root**, confirmé (`product_name` → "Virtual Machine", `bios_version` → "Hyper-V UEFI Release v4.1"). RAM détaillée par barrette nécessiterait `dmidecode -t 17` (root) — **hors scope** ; à la place, `/proc/meminfo` étendu (déjà utilisé partiellement par `system.rs`) donne total/libre/disponible/cache/swap, suffisant pour un diagnostic utile sans élévation.

Nouveau module `hardware_details.rs` : `get_hardware_details()` agrège CPU (`lscpu`, force `LC_ALL=C` pour des clés stables en anglais — sans ça, sortie localisée en français comme vu ci-dessus, ce qui casserait un parseur codé en dur), carte mère/BIOS (lecture directe des fichiers `/sys/class/dmi/id/*`, chacun individuellement optionnel — un fichier absent ne doit jamais faire échouer les autres), mémoire (`/proc/meminfo`), GPU (réutilise `hardware::get_pci_devices` existant, filtré côté frontend sur `class` contenant "VGA" ou "3D").

## 4. `PeripheralsPage.vue` — Périphériques

Tous confirmés présents et fonctionnels sans root sur la VM :
- **Moniteurs** : `xrandr` (présent, `/usr/bin/xrandr`).
- **USB** : `lsusb` (présent, `/usr/bin/lsusb`).
- **Audio** : `pactl list short sinks` (présent, a retourné un sink réel : `auto_null PipeWire float32le 2ch 48000Hz SUSPENDED`).
- **Imprimantes** : `lpstat -p` (binaire présent ; **CUPS est très probablement absent/inactif** sur la plupart des installations serveur/minimales — chaque section doit dégrader gracieusement vers "aucune imprimante détectée" plutôt que planter, même schéma que Bluetooth en R8 pour un service potentiellement inactif).

Nouveau module `peripherals.rs`, 4 commandes indépendantes (une section peut échouer sans bloquer les autres, même philosophie que `docker`/`network` dans `NetworkPage.vue`).

## 5. `ProcessesPage.vue` — Processus & services

- **Processus** : `sysinfo` est déjà une dépendance (`Cargo.toml`, utilisée par `system.rs`) — `System::processes()` donne PID/nom/CPU%/mémoire par processus directement, **aucune nouvelle dépendance, aucun subprocess**.
- **Services systemd** : `systemctl list-units --type=service --no-pager` (confirmé disponible).
- **Démarrage** : équivalent Linux des "programmes de démarrage" Windows = services utilisateur activés (`systemctl --user list-unit-files --state=enabled`) + entrées `~/.config/autostart/*.desktop`.
- **Tâches planifiées** : `crontab -l` (confirmé fonctionnel, retourne "no crontab for dev" proprement quand vide — pas une erreur) + `systemctl list-timers --no-pager` (confirmé, a listé de vraies tâches : `fwupd-refresh.timer`, `apt-daily.timer`, etc.).

Nouveau module `processes.rs`.

## 6. `InstalledSoftwarePage.vue` — Logiciels installés

Distinct de `PackagesPage.vue` (`package-manager`, orienté mises à jour/actions) : ici, une liste de consultation en lecture seule de **tout** ce qui est installé. `dpkg -l` confirmé, **2246 paquets** sur la VM de dev — la page doit avoir une recherche/filtre texte (comme `QuickInstallPage`/`FileToolsPage` l'ont déjà établi comme pattern), pas une table brute de 2246 lignes sans filtre. Réutilise `packages::detect_package_managers()` existant pour choisir la commande de listage adaptée (`dpkg -l` / `rpm -qa` / `pacman -Q` / `zypper se --installed-only`) plutôt que coder en dur `dpkg`.

Variables d'environnement : `std::env::vars()` — trivial, aucun subprocess. Affichées en lecture seule à titre diagnostic (le process de l'app tourne avec les variables de l'utilisateur invocateur, donc rien d'exposé qui ne soit pas déjà visible par l'utilisateur lui-même dans son propre shell).

## 7. `UserAccountsPage.vue` — Comptes utilisateurs

`/etc/passwd`, filtré sur `uid >= 1000 && uid < 60000` (exclut comptes système) — confirmé sur la VM : un seul compte réel, `dev` (uid 1000, home `/home/dev`, shell `/bin/bash`). Lecture seule, aucune action de modification de compte dans ce scope (créer/supprimer un utilisateur est une opération privilégiée qui mériterait sa propre décision de conception dédiée si un jour demandée — non demandé ici).

## 8. `UpdateHistoryPage.vue` — Historique des mises à jour

Distinct de `UpdatesPage.vue` (mises à jour **disponibles/en attente**) : ici, l'historique de ce qui a **déjà été installé/mis à jour** dans le passé. `/var/log/apt/history.log` confirmé présent et lisible (109 Ko sur la VM de dev) — format APT standard (`Start-Date:`/`Commandline:`/`Install:`/`Upgrade:`/`End-Date:` par bloc). Pour les autres gestionnaires : `dnf history list` (dnf), `pacman -Qi` ne donne pas d'historique natif (pacman n'a pas de log d'historique structuré équivalent — `/var/log/pacman.log` existe mais format différent), `zypper` a son propre historique. **v1 se limite au parsing du format APT** (la VM de dev et la cible principale sont Debian-based) ; les autres gestionnaires affichent un message clair "historique non disponible pour ce gestionnaire" plutôt qu'une tentative de parsing hasardeuse — honnête plutôt que silencieusement faux.

## 9. Backend : aucune nouvelle surface privilégiée

Les 6 nouvelles pages sont **100% lecture seule et non-privilégiées** — aucune ne passe par `pkexec`. Toutes les commandes utilisées (`lscpu`, `xrandr`, `lsusb`, `pactl`, `lpstat`, `systemctl list-units/list-timers`, `crontab -l`, `dpkg -l`, `/proc/meminfo`, `/sys/class/dmi/id/*`, `/etc/passwd`, `/var/log/apt/history.log`) ont été confirmées exécutables/lisibles par l'utilisateur `dev` sans `sudo` sur la VM de développement pendant la rédaction de ce spec.

## 10. Hors scope

Licence/Activation Windows, Registre Windows, analyse BSOD, WSL, Certificats système, dossiers volumineux (déjà `FileToolsPage`) — voir §1. Historique de mise à jour limité au format APT en v1 (§8). Aucune action de modification (création de service, édition de tâche cron, gestion de compte) — toutes les 6 pages sont strictement des vues de consultation.
