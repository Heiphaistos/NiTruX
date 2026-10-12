#!/usr/bin/env bash
# Construit les dépôts de paquets NiTruX (apt, dnf/zypper, pacman), signés,
# pour le VPS de Heiphaistos : https://nitrite.heiphaistos.org/depot/
#
#   outils/depot.sh <dossier-des-paquets> [--televerser]
#
# <dossier-des-paquets> contient les .deb, .rpm et .pkg.tar.zst d'une version
# (par exemple les artefacts de la release GitHub). Chaque dépôt n'est
# construit que si ses paquets sont présents ; il exige alors son outil :
#   apt     apt-ftparchive (apt-utils)
#   rpm     createrepo_c
#   pacman  repo-add (pacman, donc un Arch ou le conteneur archlinux)
#
# Signature : clé GPG DÉDIÉE aux dépôts, pas celle de l'updater Tauri (format
# minisign, incompatible). NITRUX_GPG_KEY = son identifiant ; la clé privée
# doit être dans le trousseau (gpg --import). La clé publique est publiée à
# côté des dépôts (nitrux.asc et nitrux.gpg) pour les postes clients.
#
# --televerser envoie le résultat sur le VPS par rsync (clé SSH requise, comme
# outils/publier.py). Sans lui, tout est prêt dans release/_depot/.
set -euo pipefail

SOURCE="${1:?usage : outils/depot.sh <dossier-des-paquets> [--televerser]}"
RACINE="$(cd "$(dirname "$0")/.." && pwd)"
SORTIE="${NITRUX_DEPOT_OUT:-$RACINE/release/_depot}"
VPS="root@VPS_HOST"
VPS_DIR="/var/www/nitrux-depot"
KEY="${NITRUX_GPG_KEY:?NITRUX_GPG_KEY : identifiant de la clé GPG des dépôts}"

besoin() { command -v "$1" >/dev/null || { echo "outil manquant pour le dépôt $2 : $1" >&2; exit 1; }; }
besoin gpg "(signature)"
gpg --list-secret-keys "$KEY" >/dev/null 2>&1 || { echo "clé privée $KEY absente du trousseau" >&2; exit 1; }
signer() { gpg --batch --yes --local-user "$KEY" "$@"; }

rm -rf "$SORTIE"
mkdir -p "$SORTIE"
gpg --batch --yes --armor --export "$KEY" > "$SORTIE/nitrux.asc"
gpg --batch --yes --export "$KEY" > "$SORTIE/nitrux.gpg"

shopt -s nullglob globstar
debs=("$SOURCE"/**/*.deb)
rpms=("$SOURCE"/**/*.rpm)
pkgs=("$SOURCE"/**/*.pkg.tar.zst)
shopt -u nullglob globstar

if ((${#debs[@]})); then
  # Dépôt « plat » : deb [signed-by=...] https://.../depot/apt ./
  besoin apt-ftparchive apt
  mkdir -p "$SORTIE/apt"
  cp "${debs[@]}" "$SORTIE/apt/"
  (
    cd "$SORTIE/apt"
    apt-ftparchive packages . > Packages
    gzip -9kn Packages
    apt-ftparchive -o APT::FTPArchive::Release::Origin=NiTruX -o APT::FTPArchive::Release::Label=NiTruX release . > Release
    signer --clearsign -o InRelease Release
    signer --armor --detach-sign -o Release.gpg Release
  )
  echo "  apt     ${#debs[@]} paquet(s)"
fi

if ((${#rpms[@]})); then
  besoin createrepo_c rpm
  mkdir -p "$SORTIE/rpm"
  cp "${rpms[@]}" "$SORTIE/rpm/"
  createrepo_c --quiet "$SORTIE/rpm"
  # Les .rpm de Tauri ne sont pas signés : c'est la métadonnée signée
  # (repo_gpgcheck=1) qui garantit les empreintes de chaque paquet.
  signer --armor --detach-sign -o "$SORTIE/rpm/repodata/repomd.xml.asc" "$SORTIE/rpm/repodata/repomd.xml"
  cat > "$SORTIE/rpm/nitrux.repo" <<'EOF'
[nitrux]
name=NiTruX
baseurl=https://nitrite.heiphaistos.org/depot/rpm
enabled=1
repo_gpgcheck=1
gpgcheck=0
gpgkey=https://nitrite.heiphaistos.org/depot/nitrux.asc
EOF
  echo "  rpm     ${#rpms[@]} paquet(s)"
fi

if ((${#pkgs[@]})); then
  besoin repo-add pacman
  mkdir -p "$SORTIE/arch"
  for p in "${pkgs[@]}"; do
    cp "$p" "$SORTIE/arch/"
    signer --detach-sign -o "$SORTIE/arch/$(basename "$p").sig" "$SORTIE/arch/$(basename "$p")"
  done
  (
    cd "$SORTIE/arch"
    GPGKEY="$KEY" repo-add --quiet --sign --key "$KEY" nitrux.db.tar.gz ./*.pkg.tar.zst
    # pacman lit nitrux.db : repo-add crée des liens symboliques, que rsync
    # et nginx servent mal ; des copies réelles évitent la question.
    for f in nitrux.db nitrux.db.sig nitrux.files nitrux.files.sig; do
      [ -L "$f" ] && cp --remove-destination "$(readlink "$f")" "$f"
    done
  )
  echo "  pacman  ${#pkgs[@]} paquet(s)"
fi

echo "Dépôts prêts dans $SORTIE"
if [ "${2:-}" = "--televerser" ]; then
  rsync -a --delete -e "ssh -o BatchMode=yes" "$SORTIE/" "$VPS:$VPS_DIR/"
  echo "  -> https://nitrite.heiphaistos.org/depot/"
fi
