# Suivi des 50 points (audit du 2026-10-12)

État au 2026-10-12, branche `fix/arch-wayland-locale-and-catalog` (PR #4).
Tests : `npm test` 571/571, `cargo test` 507/507, `vue-tsc` et `clippy -D warnings` sans erreur.

Légende : FAIT = codé et testé (tests auto) · VM = codé mais pas encore vérifié sur une VM · À FAIRE = non commencé.

## Cassé (1-18)
| # | Point | État |
|---|---|---|
| 1 | Orphelins (autoremove) sans pacman/xbps | FAIT, vérifié en VM Arch et Debian |
| 2 | Régénérer GRUB sur Arch / Limine / systemd-boot | FAIT, vérifié en VM Arch (grub-mkconfig) et Debian (update-grub) |
| 3 | DNS : ne plus écraser le lien resolv.conf (resolved / NetworkManager) | FAIT, vérifié en VM Debian (resolved) et dans l'app sur Omarchy ; branche NetworkManager seule non testée |
| 4 | Pare-feu firewalld | FAIT (polkit de firewalld/systemd, aucune action pkexec ajoutée) |
| 5 | Snapper (Omarchy, openSUSE, Fedora btrfs) | FAIT, vérifié en VM Debian (btrfs) |
| 6 | pacman -S sur base périmée | FAIT (`--needed` + message « mise à jour complète d'abord ») |
| 7-11 | Commandes non portables du catalogue (xrandr, iwconfig, route, netstat, lastlog, kvm-ok, lsb_release, apt/dpkg) | FAIT, exécutées pour de vrai sur Omarchy |
| 12 | checkupdates absent : repli silencieux | FAIT (pacman-contrib proposé dans Dépendances) |
| 13 | Erreurs d'outils en anglais | FAIT (`explain_error`) |
| 14 | pkexec non tuable après authentification | FAIT (message honnête) |
| 15 | Paquet `policykit-1` → `pkexec` | FAIT |
| 16 | Actions admin sur Arch | FAIT, paquet installé en VM Arch : 16 actions polkit, vrais appels pkexec OK |
| 17 | Parcours e2e sur Omarchy | FAIT : 51 pages, 0 erreur JS, scénarios terminal/PDF/benchmark/fichiers/pilotes OK (sous Xvfb). Trouvé et corrigé : imprimantes sans destination, historique pacman absent |
| 18 | Repli catalogue sur pacman | À FAIRE (e2e) |

## Fonctions validées (19-29)
| # | Point | État |
|---|---|---|
| 19 | Profils d'alimentation (powerprofilesctl) | FAIT |
| 20 | Assistant IA local (Ollama) | FAIT (testé avec un faux serveur Ollama) |
| 21 | Récupération PhotoRec / TestDisk (terminal intégré) | FAIT |
| 22 | Image disque : restauration (`restore-image`) | FAIT, vérifié en VM Debian : écriture identique octet par octet ; disque système, disque monté et image trop grande refusés |
| 23 | Export / import de la liste d'apps | FAIT |
| 24 | GRUB : délai, entrée par défaut, « retenir le dernier choix », démarrage unique (grub-reboot), liste des entrées | FAIT, vérifié en VM Debian (grub.d) et Arch (/etc/default/grub) |
| 25 | Réparation de l'amorçage : ordre EFI (remettre Linux en tête) | FAIT, vérifié en VM Debian UEFI (ordre appliqué puis restauré). Pas de grub-install à l'aveugle (risque de rendre la machine indémarrable) |
| 26 | Partitionnement (créer, supprimer, réduire) | À FAIRE · VM |
| 27 | Clé USB bootable depuis un ISO (`write-iso`, USB uniquement) | FAIT, vérifié en VM Debian : écriture identique octet par octet sur une clé USB ; non-USB, disque système, partition, chemin piégé, fichier absent refusés |
| 28 | Restauration Timeshift fichier par fichier | À FAIRE · VM |
| 29 | Benchmark compression / chiffrement | FAIT (SHA-256 + gzip, aucune dépendance) |

## Portabilité (30-35)
| # | Point | État |
|---|---|---|
| 30 | firewalld | FAIT (= 4) |
| 31 | Snapper | FAIT (= 5) |
| 32 | Mises à jour AUR (yay/paru) | FAIT |
| 33 | Firmware (fwupd) | FAIT (testé sur fixture, fwupd absent de la machine de dev) |
| 34 | Alerte « redémarrage nécessaire » | FAIT |
| 35 | arm64 | FAIT dans le workflow release (jamais exécuté) |

## Fiabilité / publication (36-44)
| # | Point | État |
|---|---|---|
| 36 | CI tests à chaque push | FAIT (`.github/workflows/ci.yml`, jamais exécuté) |
| 37 | Release signée par la CI | FAIT (`release.yml`, secrets `TAURI_SIGNING_PRIVATE_KEY` à créer) |
| 38 | e2e en CI | À FAIRE |
| 39 | Tests Debian / Fedora / Arch en conteneurs | FAIT dans ci.yml (jamais exécuté) |
| 40 | Clippy à 0 warning | FAIT |
| 41 | Bump de version automatique | FAIT (`outils/version.py X.Y.Z`) |
| 42 | Notes de version dans latest.json | FAIT (titres des commits par défaut) |
| 43 | Dépôt apt/dnf signé | À FAIRE |
| 44 | Sauvegarde de la clé de signature hors machine | À FAIRE (action humaine : copier `nitrux-updater.key` dans un coffre) |

## Interface (45-50)
| # | Point | État |
|---|---|---|
| 45 | Catalogue de 650 apps rendu d'un bloc | FAIT (`content-visibility: auto`) |
| 46 | Erreurs traduites | FAIT (= 13) |
| 47 | Proposer pacman-contrib | FAIT (= 12) |
| 48 | Moniteurs : modèle (EDID), fréquence, échelle, fréquence max inutilisée | FAIT |
| 49 | Historique des actions root | FAIT (Rapports > Actions administrateur) |
| 50 | Comparer deux rapports | FAIT |

## Version Arch / Omarchy
- `packaging/arch/PKGBUILD` (`nitrux-bin`) : reprend le .deb, marque la copie « gérée par pacman » (`/usr/lib/nitrux/managed-by`). Construit et vérifié avec `makepkg` sur Omarchy.
- Updater Tauri désactivé pour cette copie (il ferait `dpkg -i`) : mise à jour via pacman / `omarchy-update`.
- Omarchy : « Tout mettre à jour » lance `omarchy-update` dans le terminal intégré (snapshot Snapper + migrations).
- Construit automatiquement par `release.yml` à chaque tag `vX.Y.Z`.

## Corrigé en route
- GRUB : sur Debian/Ubuntu, `/etc/default/grub.d/*.cfg` écrasait les réglages (image cloud : délai forcé à 0) → écriture dans `grub.d/99-nitrux.cfg`, lecture dans le même ordre que GRUB.
- Ajout/retrait de règle ufw : délai de 15 s qui incluait la saisie du mot de passe → 60 s.

## Volontairement en attente de la VM
Les points 22, 24, 25, 26, 27 et 28 écrivent sur des disques ou sur l'amorçage. Ils ne seront codés
qu'avec la VM QEMU disponible : un code non vérifié ici pourrait partir en release dès que la PR est mergée.
Le point 43 (dépôt apt/dnf signé) demande une décision : où l'héberger (VPS ou GitHub Pages) et une clé GPG dédiée.

## Reprise (VM)
Images et clés dans le scratchpad de session (`vm/`) : Debian 13 (`deb.qcow2`, noyau standard, clé USB `usb.qcow2`,
disque jetable `scratch.qcow2`), Arch (`archvm.qcow2`). Scripts `verif-debian.sh`, `verif-arch.sh`, `verif-image.sh`.
Prochaines étapes : 26 (partitionnement : créer, supprimer, réduire) et 28 (restauration Timeshift fichier par fichier).

## Bloquants côté humain
1. `gh auth refresh -h github.com -s workflow` : sans ce droit, GitHub refuse de pousser `.github/workflows/`.
2. `sudo pacman -S --needed qemu-base edk2-ovmf webkitgtk-6.0 xorg-server-xvfb` : VM de test et e2e.
3. Secrets GitHub `TAURI_SIGNING_PRIVATE_KEY` (+ `_PASSWORD`) pour que la CI signe les releases.
