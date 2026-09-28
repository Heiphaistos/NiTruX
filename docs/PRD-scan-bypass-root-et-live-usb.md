# PRD — Scan complet, contournement du root, et NiTruX en Live USB

Statut : proposition. Dernière mise à jour : 2026-09-28.
Auteur : demande utilisateur (« scan qui sort tout de A à Z », « bypasser le
root », « inclure un live de MX Linux pour lancer NiTruX depuis une clé USB »).

Ce document pose la cible et le plan. Une partie est **déjà livrée** dans cette
branche (voir §1) ; le reste (§2 à §4) est cadré ici mais volontairement laissé
« sur le côté » comme demandé, faute de pouvoir le vérifier sur une VM/live réel
depuis l'environnement de développement actuel.

---

## 1. Déjà livré dans cette branche

| Fonctionnalité | Page | État |
|---|---|---|
| Scan PC de A à Z (matériel, capteurs, réseau, certificats, pannes, antivirus, sécurité) + rapport exportable | **Scan PC** | Livré, vérifié en direct |
| Inventaire matériel complet sans root (CPU, firmware, TPM, Secure Boot, RAM, GPU, stockage, alim, réseau, USB, audio) | **Matériel complet** | Livré |
| Détails réservés admin (barrettes RAM via dmidecode, S.M.A.R.T., clé Windows OEM du firmware) | **Matériel complet** | Livré |
| Tous les capteurs (températures, ventilateurs, pompes/watercooling, tensions, puissances) via hwmon | **Capteurs** | Livré |
| Audit de sécurité (SSH, comptes sans mot de passe, UID 0 cachés, sudo NOPASSWD, SUID inhabituels, secrets en clair) | **Scan PC** (option admin) | Livré |
| Magasin d'applications : recherche en direct dans tous les dépôts + install en un clic + activation Flatpak/Snap | **Magasin d'applications** | Livré, vérifié |
| Six familles de gestionnaires de paquets : apt, dnf, pacman, zypper, apk (Alpine), xbps (Void) | transverse | Livré |

Ces éléments couvrent l'essentiel de la demande « scan PC » et « toutes les
sources d'installation ». La suite concerne les deux points explicitement
reportés : le **contournement du root** et le **Live USB**.

---

## 2. Contournement du root (« bypasser le route »)

### 2.1 Le problème réel

Sur les machines du parc, trois situations empêchent une action administrateur
normale :

1. **Pas de mot de passe admin connu** — la machine est là, elle démarre, mais
   on n'a pas les identifiants.
2. **polkit/sudo mal configuré ou verrouillé** — pkexec échoue même avec le bon
   mot de passe (politique restrictive, pas d'agent d'authentification).
3. **La machine ne démarre pas normalement** — il faut passer par une clé USB
   (cf. §3).

Important : « contourner le root » ne veut pas dire « casser la sécurité d'une
machine d'autrui ». La cible est le **technicien légitime devant la machine
physique**, avec accord du propriétaire — exactement le cas d'usage de NiTriTe
côté Windows (WinPE, montage du disque, lecture hors-ligne).

### 2.2 Ce qui ne demande jamais le root (déjà exploité)

Beaucoup d'informations « sensibles » sont lisibles sans aucun privilège, et le
scan actuel s'appuie déjà dessus :

- Tout `/sys` et `/proc` : matériel, firmware, TPM, Secure Boot, capteurs,
  réseau, USB… (page **Matériel complet**).
- Les ports en écoute, la config DNS, les routes (page **Réseau**).
- La clé Windows OEM : elle est dans `/sys/firmware/acpi/tables/MSDM`, lisible
  par root uniquement **en ligne**, mais accessible **sans mot de passe en
  Live USB** (cf. §3.3).

### 2.3 Paliers de privilège, du moins au plus intrusif

Le futur module « accès avancé » proposerait, dans l'ordre, le premier palier
qui marche :

| Palier | Mécanisme | Ce que ça débloque | Pré-requis |
|---|---|---|---|
| 0 | Lecture `/sys` `/proc` | Inventaire, capteurs, réseau | aucun |
| 1 | pkexec / sudo classique | Toutes les actions admin actuelles | mot de passe admin |
| 2 | Groupes de l'utilisateur (`disk`, `adm`, `docker`…) | Lecture brute des disques, journaux, Docker | appartenance au groupe |
| 3 | Capabilities ciblées (`cap_sys_rawio`, `cap_dac_read_search`) | SMART, lecture de fichiers protégés | binaire doté de la cap |
| 4 | **Live USB en root** (§3) | **Tout**, y compris disque non monté et registres SAM/hives Windows | démarrer sur la clé |

Le palier 4 est le vrai « bypass » : sur un système démarré depuis une clé,
l'utilisateur **est** root, sans mot de passe, et les disques internes sont
montables en lecture seule pour analyse. C'est ainsi que NiTriTe/WinPE lit une
machine Windows verrouillée ; l'équivalent Linux est un live root.

### 2.4 Scan hors-ligne d'un disque interne

En Live USB (ou avec un second disque branché), NiTruX devrait proposer :

- Monter chaque partition détectée **en lecture seule** (`ro,noload` pour
  ext4, `ro` pour NTFS via `ntfs-3g`).
- Rejouer l'audit de sécurité (§ audit) et l'inventaire **sur le système
  monté**, pas sur le live : chemins préfixés par le point de montage.
- Pour une machine **Windows** analysée depuis Linux : lire les hives du
  registre (`hivex`), lister les comptes (`SAM`), les logiciels installés et
  les licences (déjà le cœur de NiTriTe). C'est faisable proprement en lecture
  seule et sans casser quoi que ce soit.

Décision de conception : **jamais** d'écriture sans confirmation explicite et
sans double-vérification du chemin — même discipline que les actions pkexec
actuelles.

### 2.5 Ce qui bloque la vérification aujourd'hui

Tout le palier 4 exige un **vrai environnement live** (root sans mot de passe,
disques internes présents) pour être testé selon la règle du projet (« aucune
action privilégiée ne part sans vérification réelle »). L'environnement de dev
actuel n'a ni disque interne à monter, ni image live. À reprendre quand une VM
avec un second disque, ou une clé live, est disponible.

---

## 3. NiTruX en Live USB (base MX Linux)

### 3.1 Objectif

Une image `.iso`/`.img` amorçable qui démarre directement sur NiTruX, pour :

- diagnostiquer une machine qui ne démarre plus ;
- travailler en root sans toucher au système installé ;
- transporter l'outil sur une clé, comme un « Hiren's BootCD » sous Linux.

### 3.2 Pourquoi MX Linux comme base

- Live-USB mûr et éprouvé (`live-usb-maker`, persistance optionnelle).
- Base Debian → même famille de paquets que la cible principale de NiTruX, et
  le `.deb` de NiTruX s'installe tel quel dans l'image.
- Léger, démarre vite, bon support matériel (noyau récent + firmwares).
- Outillage de remastérisation officiel (`MX Snapshot`) qui produit une ISO
  amorçable à partir d'un système configuré.

### 3.3 Chaîne de fabrication proposée

1. Partir d'une ISO MX Linux minimale (Fluxbox, sans applis lourdes).
2. Préinstaller : le `.deb` NiTruX + ses dépendances runtime (les ~20 outils
   déjà listés dans `subprocess.rs` : `smartctl`, `dmidecode`, `ntfs-3g`,
   `hivex`, `pciutils`, `usbutils`, `lm-sensors`…).
3. Démarrer NiTruX en plein écran au login automatique (root), en **mode
   kiosque**.
4. Régénérer l'ISO via `mx-snapshot` en CI.
5. Publier l'ISO à côté des `.deb`/`.rpm`/`.AppImage` sur la page des releases.

### 3.4 Adaptations applicatives nécessaires

- Un **« mode live »** détecté au lancement (présence de `/run/live` ou
  `boot=live` dans `/proc/cmdline`) qui, sachant qu'on est déjà root, saute
  toute la couche pkexec et exécute directement.
- L'écran « Scan PC » proposerait alors le **choix de la cible** : le live
  lui-même, ou un disque interne monté (§2.4).
- Sur cette base, la génération de rapport existante exporte directement sur
  une partition de données ou une seconde clé.

### 3.5 Ce qui bloque aujourd'hui

Construire et **démarrer réellement** une ISO MX pour vérifier le mode live
dépasse l'environnement de dev actuel (pas de build d'ISO, pas de boot bare
metal). C'est un chantier CI + test sur machine physique, à planifier à part.

---

## 4. Parité NiTriTe restante (rappel, hors périmètre immédiat)

Repris de `docs/AUDIT_2026-09-21.md`, toujours d'actualité :

- **Nécessite une VM avec disques** : mutations GRUB, réparation du chargeur
  d'amorçage, partitionnement complet, restauration fichier-à-fichier d'un
  snapshot Timeshift.
- **Décisions produit** : mode Turbo un clic et profils d'alimentation (ils
  feraient écrire une page aujourd'hui en lecture seule) ; benchmarks
  compression/chiffrement (dépendance en plus pour un chiffre synthétique).
- **Priorité basse** : assistant IA local (llama.cpp/Ollama), récupération de
  fichiers effacés (`testdisk`/`photorec`), image disque (`dd`/`partclone`).

---

## 5. Prochaines étapes concrètes

1. **Quand une VM avec un second disque est disponible** : implémenter le scan
   hors-ligne d'un disque monté en lecture seule (§2.4) et le vérifier.
2. **Mode live** applicatif (§3.4) : détection `boot=live`, saut de pkexec,
   choix de cible — codable et testable en partie sans ISO (simuler le
   marqueur), mais à valider sur un vrai live.
3. **Pipeline ISO MX** (§3.3) en CI, publication à côté des autres formats.
4. Reprendre la parité §4 au fil des VMs disponibles.

Aucune de ces étapes n'introduit d'action privilégiée non vérifiée : elles
restent bloquées derrière la règle du projet tant qu'un environnement réel
(live ou VM à disques) ne permet pas de les tester.
