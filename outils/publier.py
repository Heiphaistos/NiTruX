# -*- coding: utf-8 -*-
"""Publie une version de NiTruX.

A lancer APRES `npx tauri build` cote Linux, avec TAURI_SIGNING_PRIVATE_KEY dans
l'environnement (sinon les artefacts de mise a jour ne sont pas signes).

Sans cette variable, `tauri build` produit bien les trois bundles puis s'arrete
sur « A public key has been found, but no private key » APRES les avoir ecrits :
la construction a l'air reussie, mais aucun `.sig` n'existe et toute mise a jour
serait refusee chez les utilisateurs deja installes. Construire ainsi :

    TAURI_SIGNING_PRIVATE_KEY="$(cat <cle>)" TAURI_SIGNING_PRIVATE_KEY_PASSWORD="" \\
        npm run tauri build

Le televersement se fait par `scp` vers le VPS : le lancer depuis l'environnement
qui detient la cle SSH (ici Windows, pas WSL2).

Ce que produit ce script :

  Pour le client, sur la page GitHub -- les trois formes, chacune dit ce
  qu'elle est :
    Nitrux_<v>_amd64.AppImage   portable, aucune installation, aucun privilege
    Nitrux_<v>_amd64.deb        Debian, Ubuntu, Mint...
    Nitrux-<v>-1.x86_64.rpm     Fedora, openSUSE...

  Pour l'application elle-meme, sur nitrite.heiphaistos.org/maj-linux/ --
  jamais telecharge a la main :
    latest.json                 UN manifeste, trois entrees
    les trois charges + .sig

Un seul manifeste suffit pour les trois formes : le plugin cherche
`linux-x86_64-appimage`, `-deb`, `-rpm`, puis retombe sur `linux-x86_64`
(tauri-plugin-updater, `get_urls`). Il deduit la forme de la facon dont la copie
qui l'interroge a ete installee — l'application n'a rien a savoir d'elle-meme.

Usage :
    python3 outils/publier.py                 # prepare et televerse
    python3 outils/publier.py --notes "..."   # une phrase pour la fenetre
    python3 outils/publier.py --sans-televersement   # CI : prepare seulement
"""

import argparse
import io
import json
import os
import shutil
import subprocess
import sys
from datetime import datetime, timezone

RACINE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
BUNDLE = os.path.join(RACINE, "src-tauri", "target", "release", "bundle")
SORTIE = os.path.join(RACINE, "release")
# La cle privee, dans l'ordre ou on la cherche. Le PRD prevoyait
# `~/.tauri/`, mais la paire vit en fait dans le coffre `D:\mdp` avec les
# autres cles de publication -- chercher aux deux endroits evite de
# redecouvrir ca a chaque version.
CLES_POSSIBLES = [
    os.path.expanduser("~/.tauri/nitrux-updater.key"),
    "D:/mdp/tauri-updater-keys/nitrux-updater.key",
    "/mnt/d/mdp/tauri-updater-keys/nitrux-updater.key",
]
VPS = "root@212.227.140.45"
VPS_DIR = "/var/www/nitrux-maj"
BASE_URL = "https://nitrite.heiphaistos.org/maj-linux"


def version():
    with io.open(os.path.join(RACINE, "package.json"), encoding="utf-8") as f:
        return json.load(f)["version"]


def executer(cmd, **kw):
    print("  $", os.path.basename(str(cmd[0])), " ".join(str(c) for c in cmd[1:4]))
    r = subprocess.run(cmd, cwd=RACINE, **kw)
    if r.returncode != 0:
        raise SystemExit("echec : %s (code %d)" % (cmd[0], r.returncode))
    return r


def charges(v, bundle=BUNDLE):
    """Les trois formes : cle du manifeste -> chemin. arm64 en plus quand une
    construction arm64 existe (sa cle n'apparait pas sinon)."""
    resultat = {
        "linux-x86_64-appimage": os.path.join(
            bundle, "appimage", "Nitrux_%s_amd64.AppImage" % v
        ),
        "linux-x86_64-deb": os.path.join(bundle, "deb", "Nitrux_%s_amd64.deb" % v),
        "linux-x86_64-rpm": os.path.join(bundle, "rpm", "Nitrux-%s-1.x86_64.rpm" % v),
    }
    arm = {
        "linux-aarch64-appimage": os.path.join(bundle, "appimage", "Nitrux_%s_aarch64.AppImage" % v),
        "linux-aarch64-deb": os.path.join(bundle, "deb", "Nitrux_%s_arm64.deb" % v),
        "linux-aarch64-rpm": os.path.join(bundle, "rpm", "Nitrux-%s-1.aarch64.rpm" % v),
    }
    resultat.update({k: c for k, c in arm.items() if os.path.exists(c)})
    return resultat


def notes_par_defaut():
    """Les titres des commits depuis le tag precedent : la fenetre de mise a
    jour affichait une note vide a chaque version."""
    try:
        precedent = subprocess.run(
            ["git", "describe", "--tags", "--abbrev=0", "HEAD^"],
            cwd=RACINE, capture_output=True, text=True, check=True,
        ).stdout.strip()
        titres = subprocess.run(
            ["git", "log", "--no-merges", "--format=- %s", "%s..HEAD" % precedent],
            cwd=RACINE, capture_output=True, text=True, check=True,
        ).stdout.strip().splitlines()
    except (subprocess.CalledProcessError, FileNotFoundError):
        return ""
    return "\n".join(titres[:10])


def signature(chemin):
    """Signature du fichier, produite par le bundler ou fabriquee ici.

    Le bundler ne signe d'office que l'AppImage (via son `.tar.gz`). Les paquets
    deb et rpm sont signes a la main, avec la MEME cle : une charge sans
    signature valide est refusee par le plugin, et c'est le comportement voulu.
    """
    for candidat in (chemin + ".sig", chemin + ".tar.gz.sig"):
        if os.path.exists(candidat):
            with io.open(candidat, encoding="utf-8") as f:
                return f.read().strip(), candidat
    cle = next((c for c in CLES_POSSIBLES if os.path.exists(c)), None)
    if cle is None and os.environ.get("TAURI_SIGNING_PRIVATE_KEY"):
        # CI : la cle vient d'un secret, `tauri signer sign` la lit seul.
        executer(["npx", "tauri", "signer", "sign", chemin], stdout=subprocess.DEVNULL)
        with io.open(chemin + ".sig", encoding="utf-8") as f:
            return f.read().strip(), chemin + ".sig"
    if cle is None:
        raise SystemExit(
            "cle privee introuvable, cherchee dans :\n  %s\n"
            "Le plus simple reste de construire avec TAURI_SIGNING_PRIVATE_KEY "
            "dans l'environnement : le bundler produit alors les .sig lui-meme."
            % "\n  ".join(CLES_POSSIBLES)
        )
    executer(["npx", "tauri", "signer", "sign", "-f", cle, "-p", "", chemin],
             stdout=subprocess.DEVNULL)
    with io.open(chemin + ".sig", encoding="utf-8") as f:
        return f.read().strip(), chemin + ".sig"


def main():
    p = argparse.ArgumentParser()
    p.add_argument(
        "--notes",
        default=None,
        help="le texte affiche dans la fenetre de proposition ; par defaut, "
        "les titres des commits depuis le tag precedent",
    )
    p.add_argument(
        "--sans-televersement",
        action="store_true",
        help="prepare release/_canal sans l'envoyer sur le VPS (CI)",
    )
    p.add_argument("--bundle", default=BUNDLE, help="dossier des bundles construits")
    a = p.parse_args()
    if a.notes is None:
        a.notes = notes_par_defaut()
    v = version()
    print("NiTruX %s" % v)

    prep = os.path.join(SORTIE, "_canal")
    if os.path.exists(prep):
        shutil.rmtree(prep)
    os.makedirs(prep)

    plateformes = {}
    a_televerser = []
    for cle, chemin in charges(v, a.bundle).items():
        if not os.path.exists(chemin):
            raise SystemExit(
                "%s manquant : lancer d'abord `npx tauri build` cote Linux (%s)"
                % (cle, chemin)
            )
        sig, fichier_sig = signature(chemin)
        nom = os.path.basename(chemin)

        # L'AppImage se met a jour depuis l'archive .tar.gz quand le bundler en
        # produit une : c'est ce que le plugin sait lire.
        archive = chemin + ".tar.gz"
        source = archive if (cle.endswith("appimage") and os.path.exists(archive)) else chemin
        nom = os.path.basename(source)

        shutil.copyfile(source, os.path.join(prep, nom))
        shutil.copyfile(fichier_sig, os.path.join(prep, nom + ".sig"))
        plateformes[cle] = {"signature": sig, "url": "%s/%s" % (BASE_URL, nom)}
        a_televerser.append(nom)
        print("  %-26s %s" % (cle, nom))

    manifeste = {
        "version": v,
        "notes": a.notes,
        "pub_date": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": plateformes,
    }
    with io.open(os.path.join(prep, "latest.json"), "w", encoding="utf-8", newline="\n") as f:
        f.write(json.dumps(manifeste, indent=2))

    if a.sans_televersement:
        print("[canal] pret dans %s (non televerse)" % prep)
        return 0
    print("[canal] televersement vers %s:%s" % (VPS, VPS_DIR))
    fichiers = [os.path.join(prep, n) for n in sorted(os.listdir(prep))]
    executer(["scp", "-o", "BatchMode=yes"] + fichiers + ["%s:%s/" % (VPS, VPS_DIR)])
    print("  -> %s/latest.json" % BASE_URL)
    return 0


if __name__ == "__main__":
    sys.exit(main())
