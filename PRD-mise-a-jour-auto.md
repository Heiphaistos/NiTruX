# PRD — Mise a jour automatique de NiTruX (version Linux)

Statut : **en attente du feu vert de Momo**. Redige le 2026-09-16.
Pendant Windows : `D:\Projet\Nitrite 2.0\PRD-mise-a-jour-auto.md`.

## 1. Ce que Momo veut

NiTruX doit avoir **exactement la meme capacite de mise a jour automatique** que Nitrite : l'utilisateur telecharge une fois, et quinze versions plus tard son exemplaire lui dit « une nouvelle version est sortie, voulez-vous mettre a jour ? ». Il repond oui, tout se fait seul, l'application redemarre a jour. Il ne retourne jamais sur GitHub chercher un fichier.

NiTruX est loin d'etre fini — ce PRD ne parle que du mecanisme de mise a jour, pas du reste du produit.

## 2. Bonne nouvelle : sous Linux, tout est deja natif

Contrairement a Windows, **il n'y a aucun code de mise a jour a ecrire**. Verifie dans `tauri-plugin-updater-2.11.0/src/updater.rs` :

| Forme | Ce que fait le plugin | Verifie |
|---|---|---|
| **AppImage** | renomme l'AppImage en cours vers un dossier temporaire, ecrit la nouvelle a sa place, remet les permissions | `install_appimage`, l. 1046 |
| **deb** | controle que les octets sont bien un paquet Debian (`infer::archive::is_deb`), puis `dpkg -i` | `install_deb`, l. 1120 |
| **rpm** | meme chose en `rpm` | `install_rpm` |

L'elevation pour deb/rpm est geree par le plugin, dans cet ordre : `pkexec` (fenetre graphique), puis zenity/kdialog, puis `sudo` en terminal (l. 1183-1212).

**Le remplacement sur place de l'AppImage est exactement ce que la version portable Windows a demande 150 lignes de Rust.** Ici c'est fourni.

Et une deuxieme bonne nouvelle : **un seul manifeste suffit pour les trois formes.** Le plugin cherche dans l'ordre `linux-x86_64-appimage`, puis `linux-x86_64` (`get_urls`, l. 607-630) en deduisant la forme de la facon dont l'application a ete installee. Un `latest.json` avec trois entrees sert donc les trois publics, sans que l'application ait a savoir ce qu'elle est :

```json
{
  "version": "0.25.149",
  "platforms": {
    "linux-x86_64-appimage": { "url": "…/Nitrux_0.25.149_amd64.AppImage.tar.gz", "signature": "…" },
    "linux-x86_64-deb":      { "url": "…/Nitrux_0.25.149_amd64.deb",             "signature": "…" },
    "linux-x86_64-rpm":      { "url": "…/Nitrux-0.25.149-1.x86_64.rpm",          "signature": "…" }
  }
}
```

## 3. Etat actuel de NiTruX, releve

- `src-tauri/tauri.conf.json` : `productName` = `Nitrux`, identifiant `org.heiphaistos.nitrux`, version `0.25.148`, `bundle.targets: "all"` — donc deb + rpm + AppImage sont deja produits.
- Release `v0.25.148` en ligne, trois assets : `Nitrux_0.25.148_amd64.AppImage` (86,6 Mo), `Nitrux_0.25.148_amd64.deb` (9,5 Mo), `Nitrux-0.25.148-1.x86_64.rpm` (9,5 Mo).
- **Aucun plugin updater** dans `Cargo.toml`, **aucune section `plugins`** dans la configuration.
- **Aucune CI** : pas de dossier `.github`. Les versions sont construites et publiees a la main.
- `bundle.resources` ne contient que les fichiers de privileges (`packaging/*.policy`, `nitrux-pkexec-helper`). **Pas d'equivalent de `logiciel\` / `Drivers\` / `Script Windows\`** : le probleme de contenu a preserver qui domine le PRD Windows n'existe pas ici. Si un jour un tel dossier apparait, appliquer la meme regle — il ne doit jamais entrer dans le manifeste d'un paquet.

## 4. Architecture retenue

Meme forme que Windows, decision du 2026-09-16 : **le canal vit sur le VPS, pas sur la page GitHub.**

- Point d'entree : `https://nitrite.heiphaistos.org/maj-linux/latest.json`.
  Reutiliser le domaine deja servi evite une entree DNS et un certificat de plus ; un manifeste est lu par une machine, le nom de domaine ne raconte rien a personne. Si Momo prefere `nitrux.heiphaistos.org`, il faut le DNS et un certbot en plus — **aucun vhost NiTruX n'existe aujourd'hui**, verifie sur le serveur.
  Meme bloc nginx que pour Nitrite, avec les en-tetes de securite **repetes dans le `location`** (nginx ne les herite pas — piege deja paye sur ce VPS).
- Contenu de `/var/www/nitrux-maj/` : `latest.json`, plus les trois charges et leurs `.sig`. Place disponible sur le VPS : 208 Go libres, verifie.
- La page GitHub garde ses trois assets actuels. **Aucun renommage n'est necessaire** : `.AppImage`, `.deb` et `.rpm` disent deja ce qu'ils sont a qui les telecharge — c'est precisement ce qui manquait cote Windows.

**Cle de signature : une cle DEDIEE a NiTruX**, `~/.tauri/nitrux-updater.key`, pas celle de Nitrite. Deux produits, deux chaines de publication : la fuite de l'une ne doit pas donner le droit de pousser du code sur l'autre. A generer par `npx tauri signer generate`, sauvegarder hors machine — **perdue, plus aucun poste deja installe ne peut etre mis a jour**.

## 5. Ce qu'il faut ajouter

1. `tauri-plugin-updater` dans `Cargo.toml` + `@tauri-apps/plugin-updater` cote npm, enregistre dans le `Builder`.
2. `"updater:default"` dans les capacites.
3. Dans `tauri.conf.json` : `bundle.createUpdaterArtifacts: true`, et la section `plugins.updater` (point d'entree + cle publique).
4. Le composable de verification, reprenant celui de Nitrite : verification silencieuse au demarrage, bouton manuel dans les reglages, echec reseau muet au demarrage et bavard au clic.
   **Sans la garde `is_portable_install()`** : sous Linux le plugin deduit lui-meme la forme installee, il n'y a rien a couper.
   **Sans `relaunch()`** : a verifier a l'implementation, mais sous Linux le plugin ne fait pas `process::exit(0)` comme sous Windows — le comportement de redemarrage doit etre observe, pas suppose.
5. Un script de publication qui depose les quatre fichiers sur le VPS apres la construction.

Estimation : configuration + cablage, une seule vraie inconnue (point 6.1).

## 6. A verifier a l'implementation — ne pas supposer

1. **`tauri build` produit-il un `.sig` pour le deb et le rpm ?** La documentation du plugin ne promet l'artefact de mise a jour que pour l'AppImage (`.AppImage.tar.gz`). Si les paquets ne sont pas signes automatiquement, les signer a la main avec `tauri signer sign` avant de les deposer. **Une charge sans signature valide est refusee par le plugin** : c'est le comportement voulu, pas un bug a contourner.
2. **L'AppImage doit pouvoir etre remplacee sur place.** Le plugin exige un dossier temporaire sur **le meme systeme de fichiers** que l'AppImage (il compare les numeros de peripherique, l. 1070) et essaie dans l'ordre : `/tmp`, le cache utilisateur, puis le dossier de l'AppImage. Une AppImage posee sur une cle USB, un montage en lecture seule ou un autre disque est un cas a tester pour de vrai.
3. **Chaque mise a jour AppImage retelecharge 86 Mo** (contre 9,5 Mo pour deb/rpm). Rien a optimiser aujourd'hui, mais c'est a savoir avant de proposer des mises a jour frequentes.
4. **Le redemarrage apres mise a jour** : a observer sur les trois formes, et noter le resultat quel qu'il soit.

## 7. Securite — la ligne a ne pas franchir

Les fichiers de privileges de NiTruX sont **corrects aujourd'hui** : les cinq `.policy` demandent `auth_admin` pour `allow_any`, `allow_inactive` **et** `allow_active`, et `nitrux-pkexec-helper` revalide ses arguments sans faire confiance au code Rust qui l'appelle.

**Il ne faut surtout pas ajouter une regle polkit permissive pour rendre la mise a jour deb/rpm silencieuse.** Une regle qui autoriserait l'installation d'un paquet sans mot de passe donnerait a n'importe quel utilisateur local le droit d'installer du code arbitraire en root. La demande de mot de passe a chaque mise a jour d'un paquet systeme est le comportement correct, pas une gene a supprimer. L'utilisateur qui ne veut pas d'invite prend l'AppImage : elle se met a jour sans aucun privilege, parce qu'elle n'en demande aucun.

## 8. Recette

Comme cote Windows, il faut **deux publications** : la premiere pose le mecanisme, la seconde le prouve. La 0.25.149 embarque l'updater, la 0.25.150 est celle qu'on regarde arriver.

Pour chacune des trois formes :
- [ ] « Verifier les mises a jour » repond « a jour » tant que la version suivante n'existe pas ;
- [ ] a la publication suivante : la fenetre annonce la bonne version ;
- [ ] AppImage : remplacement sur place, **aucune trace hors du fichier lui-meme** — ni `/usr`, ni paquet enregistre, ni fichier oublie dans `/tmp` ;
- [ ] deb : invite `pkexec`, paquet remplace, `dpkg -l nitrux` annonce la nouvelle version ;
- [ ] rpm : idem avec `rpm -q` ;
- [ ] une charge signee avec une **autre** cle est refusee — a tester pour de vrai en fabriquant la fausse signature, pas en le supposant ;
- [ ] le VPS eteint, l'application demarre normalement et ne dit rien a l'ecran (une ligne dans les journaux suffit).

## 9. Hors perimetre

- Depot apt/dnf en bonne et due forme : ce serait la facon canonique de distribuer sous Linux, mais elle demande une infrastructure de depot signee. Le present mecanisme met a jour les paquets deja installes ; il ne remplace pas un depot.
- CI : NiTruX n'en a aucune. Les versions continuent d'etre construites a la main tant que le produit n'est pas fini. La cle privee doit alors etre presente dans l'environnement de construction (`TAURI_SIGNING_PRIVATE_KEY`).
- arm64 : seul `x86_64` est publie aujourd'hui. Une entree `linux-aarch64-*` s'ajoutera au manifeste le jour ou une construction arm64 existera.
- Flatpak et Snap : formats a mise a jour geree par leur propre magasin, hors du mecanisme decrit ici.
