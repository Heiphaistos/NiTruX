# -*- coding: utf-8 -*-
"""Change la version de NiTruX partout a la fois.

    python3 outils/version.py 0.25.154

package.json, tauri.conf.json, Cargo.toml, Cargo.lock et le PKGBUILD Arch
doivent porter le meme numero : un oubli publie un paquet qui se croit dans
une autre version et que le canal de mise a jour repropose sans fin.
"""
import io
import os
import re
import sys

RACINE = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
FICHIERS = [
    ("package.json", r'("version":\s*")[^"]+(")'),
    ("src-tauri/tauri.conf.json", r'("version":\s*")[^"]+(")'),
    ("src-tauri/Cargo.toml", r'(?m)(^version\s*=\s*")[^"]+(")'),
    ("src-tauri/Cargo.lock", r'(name = "nitrux"\nversion = ")[^"]+(")'),
    ("packaging/arch/PKGBUILD", r"(?m)(^pkgver=)[^\n]+()"),
]


def main():
    if len(sys.argv) != 2 or not re.fullmatch(r"\d+\.\d+\.\d+", sys.argv[1]):
        raise SystemExit("usage : python3 outils/version.py X.Y.Z")
    v = sys.argv[1]
    for chemin, motif in FICHIERS:
        p = os.path.join(RACINE, chemin)
        with io.open(p, encoding="utf-8") as f:
            texte = f.read()
        nouveau, n = re.subn(motif, lambda m: m.group(1) + v + m.group(2), texte, count=1)
        if n != 1:
            raise SystemExit("version introuvable dans %s" % chemin)
        with io.open(p, "w", encoding="utf-8", newline="\n") as f:
            f.write(nouveau)
        print("  %-28s %s" % (chemin, v))
    # pkgrel repart a 1 a chaque nouvelle version
    p = os.path.join(RACINE, "packaging/arch/PKGBUILD")
    with io.open(p, encoding="utf-8") as f:
        texte = re.sub(r"(?m)^pkgrel=.*$", "pkgrel=1", f.read())
    with io.open(p, "w", encoding="utf-8", newline="\n") as f:
        f.write(texte)


if __name__ == "__main__":
    main()
