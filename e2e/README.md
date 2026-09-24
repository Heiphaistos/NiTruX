# Tests de bout en bout (application réelle)

Pilotent la **vraie** application compilée (pas des mocks) via WebDriver
(`tauri-driver` + `WebKitWebDriver`), comme le ferait un utilisateur.

```bash
./e2e/run.sh                # toutes les pages + scénarios sans effet sur le système
./e2e/run.sh --privileged   # + activation pkexec, install/désinstall de cowsay, cycle pare-feu
REBUILD=1 ./e2e/run.sh      # recompiler l'application avant
```

À lancer **en utilisateur normal**, pas en root : root masque les problèmes de
droits (c'est comme ça que le bug de la page Pare-feu passait inaperçu).

## Prérequis (Debian/Ubuntu)

```bash
sudo apt install webkit2gtk-driver xvfb python3-selenium   # ou pip install --user selenium
cargo install tauri-driver --locked
```

Depuis une session graphique, `DISPLAY` est déjà défini et la fenêtre s'ouvre
à l'écran ; sans écran (SSH), `run.sh` démarre un `Xvfb` tout seul.

## Ce que ça vérifie

- `sweep.py` : ouvre chacune des pages du menu, relève les cartes d'erreur
  et les erreurs JavaScript (console, exceptions, promesses rejetées).
  Échoue s'il y a au moins une erreur JavaScript. Les cartes d'erreur sont
  listées pour relecture : un outil absent (bluez, timeshift…) est un état
  normal tant que le message dit quoi installer.
- `flows.py` : terminal (accents, `exit` puis nouvelle session), rapport PDF,
  benchmark, doublons et gros fichiers, pilotes ; avec `--privileged`,
  activation des fonctions privilégiées, installation puis désinstallation
  de `cowsay`, pare-feu (activation si besoin, ajout/retrait de 8080/tcp,
  retour à l'état initial).

Captures d'écran et `sweep.json` : `e2e/out/`.

Sans agent polkit (SSH sans bureau), pkexec ne peut pas demander le mot de
passe : lancez `--privileged` depuis la session graphique.
