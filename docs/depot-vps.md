# Dépôts de paquets NiTruX sur le VPS

Servis depuis `https://nitrite.heiphaistos.org/depot/` (même domaine que le
canal de mise à jour `maj-linux/`), fichiers dans `/var/www/nitrux-depot/`.

## Mise en place (une fois)

1. Clé GPG **dédiée** aux dépôts (pas celle de l'updater Tauri) :

   ```bash
   gpg --quick-gen-key "NiTruX dépôts <depot@heiphaistos.org>" ed25519 sign never
   gpg --armor --export-secret-keys <ID> > nitrux-depot.key   # à mettre au coffre, hors machine
   ```

   Secret GitHub `NITRUX_GPG_PRIVATE_KEY` = contenu de `nitrux-depot.key`
   (sans phrase de passe : la CI ne peut pas en saisir une).
   **Perdue, plus aucun poste ne peut vérifier les mises à jour du dépôt.**

2. nginx, dans le `server` de `nitrite.heiphaistos.org`. Les en-têtes de
   sécurité sont répétés dans le `location` : nginx ne les hérite pas dès
   qu'un `add_header` y apparaît.

   ```nginx
   location /depot/ {
       alias /var/www/nitrux-depot/;
       autoindex off;
       add_header X-Content-Type-Options "nosniff" always;
       add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload" always;
       add_header Referrer-Policy "strict-origin-when-cross-origin" always;
       add_header Cache-Control "no-cache" always;   # index et signatures toujours frais
   }
   ```

## À chaque version

La CI (`release.yml`) construit les dépôts dans l'artefact `depot` du run.
Le télécharger, puis :

```bash
rsync -a --delete depot/ root@VPS_HOST:/var/www/nitrux-depot/
```

Ou en local, avec la clé dans le trousseau :
`NITRUX_GPG_KEY=<ID> outils/depot.sh <dossier-des-paquets> --televerser`.

## Côté utilisateurs

Debian / Ubuntu :

```bash
sudo curl -fsSLo /usr/share/keyrings/nitrux.gpg https://nitrite.heiphaistos.org/depot/nitrux.gpg
echo "deb [signed-by=/usr/share/keyrings/nitrux.gpg] https://nitrite.heiphaistos.org/depot/apt ./" | sudo tee /etc/apt/sources.list.d/nitrux.list
sudo apt update && sudo apt install nitrux
```

Fedora / openSUSE :

```bash
sudo curl -fsSLo /etc/yum.repos.d/nitrux.repo https://nitrite.heiphaistos.org/depot/rpm/nitrux.repo
sudo dnf install nitrux        # openSUSE : sudo zypper addrepo ... puis zypper install nitrux
```

Arch / Omarchy :

```bash
curl -fsSL https://nitrite.heiphaistos.org/depot/nitrux.asc | sudo pacman-key --add -
sudo pacman-key --lsign-key <ID>
printf '\n[nitrux]\nServer = https://nitrite.heiphaistos.org/depot/arch\n' | sudo tee -a /etc/pacman.conf
sudo pacman -Sy nitrux-bin
```

Ensuite NiTruX se met à jour avec le système (`apt upgrade`, `dnf upgrade`,
`pacman -Syu` ou `omarchy-update`).
