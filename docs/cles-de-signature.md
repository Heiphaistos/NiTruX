# Clés de signature de NiTruX — à sauvegarder hors machine

Deux clés privées font tourner les mises à jour. **Perdues, plus aucun poste
déjà installé ne peut être mis à jour** : il faudrait réinstaller NiTruX à la
main partout. **Volées, n'importe qui pourrait pousser du code** sur ces postes.

| Clé | Rôle | Où elle vit aujourd'hui |
|---|---|---|
| `nitrux-updater.key` (minisign, Tauri) | signe les mises à jour automatiques (AppImage, deb, rpm) | machine de publication (`~/.tauri/` ou `D:\mdp\tauri-updater-keys\`) |
| `nitrux-depot.key` (GPG) | signe les dépôts apt/rpm/pacman du VPS | à créer, voir `docs/depot-vps.md` |

## Sauvegarde (une fois, puis à chaque nouvelle clé)

1. Deux copies, sur deux supports différents, hors de la machine de
   publication : un gestionnaire de mots de passe (pièce jointe chiffrée) **et**
   une clé USB chiffrée rangée ailleurs.
2. Jamais dans le dépôt Git, jamais dans un message, jamais dans le cloud en
   clair.
3. Noter dans le gestionnaire de mots de passe la phrase de passe de la clé
   (vide aujourd'hui pour `nitrux-updater.key`).
4. Vérifier la copie : `npx tauri signer sign -f <copie> -p "" README.md`
   doit produire un `README.md.sig` (puis le supprimer).

## Dans GitHub (pour la CI de release)

Settings → Secrets and variables → Actions :
`TAURI_SIGNING_PRIVATE_KEY` (contenu de `nitrux-updater.key`),
`TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (vide si aucune),
`NITRUX_GPG_PRIVATE_KEY` (contenu de `nitrux-depot.key`).
Un secret GitHub n'est pas une sauvegarde : il ne se relit pas.

## Si une clé fuit

La considérer comme compromise : en générer une nouvelle, publier une version
signée avec l'ANCIENNE qui embarque la NOUVELLE clé publique
(`plugins.updater.pubkey` dans `tauri.conf.json`), puis ne plus utiliser que la
nouvelle. Pour la clé GPG : nouvelle clé, nouveau `nitrux.asc` publié, et
prévenir les utilisateurs du dépôt de réimporter la clé.
