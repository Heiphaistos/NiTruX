# Phase R10 (Logiciels & déploiement) — Design Spec

## 1. Contexte et périmètre

Après R9 (Stockage avancé, v0.18.0), la catégorie de navigation "Applications" n'a que 2 pages (`quick-install`, `package-manager`) alors que la référence NiTriTe Windows en a 4 sous sa catégorie "Logiciels" (`tools`, `master-install`, `portables`, `os-downloads`). Plutôt que porter ces 4 pages telles quelles — deux d'entre elles (`tools` : lanceur d'outils système Windows type msconfig/regedit ; `os-downloads` : liens de téléchargement d'ISO d'autres OS) n'ont pas d'équivalent Linux qui apporte une vraie valeur ajoutée distincte de ce qui existe déjà (Scripts & Snippets couvre le lancement ad-hoc, et un outil Linux qui liste des téléchargements d'autres OS est hors-sujet) — cette phase se concentre sur un gap concret et déjà visible dans le code actuel :

**`src/data/appCatalog.ts` a un type `InstallMethod = "apt" | "flatpak" | "snap"` depuis le début, mais `QuickInstallPage.vue` n'implémente que `"apt"`.** Les 3 entrées non-apt du catalogue (Discord/Steam en flatpak, Spotify en snap) affichent un bouton désactivé avec le badge **"Bientôt disponible (flatpak)"** / **"Bientôt disponible (snap)"** — une promesse explicite, déjà écrite dans l'UI, jamais tenue. `packages/mod.rs`'s `PackageUpdate.source` a le même commentaire ("apt", "dnf", "pacman", "zypper", "flatpak", "snap") anticipant ce même besoin.

**Décision de périmètre (validée avec l'utilisateur) :** fermer ce gap complètement, y compris Snap — qui nécessite une nouvelle surface pkexec (snapd exige toujours root, contrairement à Flatpak qui s'installe en `--user` sans élévation). Traité avec la même rigueur que toute nouvelle action privilégiée dans ce projet : chemin `exec.path` dédié, validation ré-appliquée côté script, testé en direct sur la VM avant merge.

**Portée de la phase :**
1. Support Flatpak (`--user`, non-privilégié) dans `install_package`.
2. Support Snap (nouvelle action pkexec `install-snap`).
3. `QuickInstallPage.vue` : retrait du badge "Bientôt disponible", dispatch réel selon `installMethod`.
4. Nouvelle page **Installation par profils** — sélection multiple + installation groupée, catégorie "Applications" 2→3 pages (le catalogue existant sert de source de données, aucune duplication).
5. Bonus (hors-thème mais gap orphelin réel) : `get_smart_status` est déjà backend + testé + enregistré dans `lib.rs` depuis R9, mais **n'a aucun consommateur frontend**. Ajout d'une section santé S.M.A.R.T. dans `DisksPage.vue` (lecture seule, gère gracieusement l'absence de droits root — comportement déjà documenté dans `smart.rs`).

**Hors scope (décision explicite) :** `tools` (lanceur d'outils système Windows) et `os-downloads` (liens ISO) de la référence NiTriTe — pas d'équivalent Linux à valeur ajoutée distincte du existant. Snap `--classic` confinement (aucune entrée du catalogue actuel n'en a besoin ; à traiter si un jour une entrée l'exige).

## 2. Support Flatpak (non-privilégié)

### 2.1 Contrainte réelle vérifiée sur la VM
`flatpak` n'est **pas installé par défaut** sur la VM Debian de développement (`which flatpak` → rien), mais est disponible via `apt` (`apt-cache show flatpak` confirme le paquet `flatpak 1.16.6-1~deb13u1`). Une installation fraîche n'a **aucun remote configuré** — `flatpak install <app>` échouerait sans le remote Flathub, que les `packageId` du catalogue (`com.discordapp.Discord`, `com.valvesoftware.Steam`) supposent implicitement.

### 2.2 Comportement
Nouveau module `src-tauri/src/packages/flatpak.rs` :
- `binary_exists("flatpak")` (déjà dans `packages/mod.rs`) — si absent, retourne une erreur claire immédiatement, pas de tentative.
- Avant l'installation : `flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo` (idempotent, jamais d'erreur si déjà présent — `--if-not-exists` gère ça nativement).
- Installation : `flatpak install --user --noninteractive flathub <app_id>`.
- **Aucun pkexec** : `--user` installe dans `~/.local/share/flatpak`, avec les droits normaux de l'utilisateur invocateur — même philosophie que `run_script` en R8 (pas une nouvelle frontière de privilège).
- Validation de `app_id` : réutilise `packages::install::validate_package_name` telle quelle (déjà accepte `.`/`+`/`:`/`_`/`-`, couvre le format reverse-DNS des IDs Flatpak comme `com.discordapp.Discord` sans modification).

## 3. Support Snap (nouvelle action pkexec)

### 3.1 Backend Rust
`install_snap_package(package: String) -> Result<String, String>` dans `packages/install.rs`, miroir exact d'`install_package` : réutilise `validate_package_name`, invoque `pkexec /usr/bin/nitrux-pkexec-install-snap install-snap <package>` avec le même timeout de 300s (une installation snap peut être lourde, comme apt/dnf).

### 3.2 Script wrapper
Nouveau cas `install-snap` dans `nitrux-pkexec-helper` (le script est déjà installé sous 12 noms distincts ; celui-ci devient le 13e) :
```sh
install-snap)
  package="${2:-}"
  validate_package_name "$package"
  exec snap install "$package"
  ;;
```
Re-validation indépendante du nom de paquet côté script, comme pour `install-package`/`uninstall-package` — jamais faire confiance à la validation Rust seule.

### 3.3 Policy Polkit
Un 4e `<action>` ajouté à `org.heiphaistos.nitrux.packages.policy` (même fichier que install/uninstall/upgrade-all — même domaine "packages", pas un nouveau fichier) :
```xml
<action id="org.heiphaistos.nitrux.install-snap">
  <description>Installer un paquet Snap</description>
  <message>NiTruX veut installer un paquet Snap</message>
  <defaults>
    <allow_any>auth_admin</allow_any>
    <allow_inactive>auth_admin</allow_inactive>
    <allow_active>auth_admin</allow_active>
  </defaults>
  <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-install-snap</annotate>
</action>
```

### 3.4 `tauri.conf.json`
Ajout de `/usr/bin/nitrux-pkexec-install-snap: packaging/nitrux-pkexec-helper` à la liste `files` de `bundle.linux.deb` et `bundle.linux.rpm` (même pattern que les 12 entrées existantes).

### 3.5 Vérification live obligatoire avant merge
`snapd` doit être installé et actif sur la VM (`which snap`, `systemctl is-active snapd` — à vérifier, installer si absent). Cycle complet : `pkttyagent --process $$ & sleep 1; pkexec /usr/bin/nitrux-pkexec-install-snap install-snap <paquet-snap-jetable>` via `ssh_interactive.py`, confirmer la ligne `==== AUTHENTICATING FOR org.heiphaistos.nitrux.install-snap ====` (résout la bonne action, pas une autre), confirmer l'installation réelle (`snap list`), puis `snap remove <paquet>` pour nettoyer (commande hors scope de cette phase, faite manuellement en vérification).

## 4. `QuickInstallPage.vue`

Remplace :
```vue
<template v-if="entry.installMethod !== 'apt'">
  <NxBadge status="info">Bientôt disponible ({{ entry.installMethod }})</NxBadge>
  <NxButton disabled>Installer</NxButton>
</template>
```
par un dispatch réel dans `install()` :
```ts
async function install(entry: AppCatalogEntry) {
  installState.value[entry.id] = "installing";
  delete installErrors.value[entry.id];
  try {
    if (entry.installMethod === "apt") {
      const manager = nativeManager.value ?? (await managerReady);
      if (!manager) throw new Error("aucun gestionnaire de paquets natif détecté");
      await invoke<string>("install_package", { manager, package: entry.packageId });
    } else if (entry.installMethod === "flatpak") {
      await invoke<string>("install_flatpak_package", { appId: entry.packageId });
    } else {
      await invoke<string>("install_snap_package", { package: entry.packageId });
    }
    installState.value[entry.id] = "success";
  } catch (e) {
    installState.value[entry.id] = "error";
    installErrors.value[entry.id] = String(e);
  }
}
```
Le bouton "Installer" et la barre de progression existants s'appliquent désormais à toutes les méthodes, plus de branche `disabled`/badge "Bientôt disponible".

## 5. Nouvelle page : Installation par profils

`src/pages/InstallProfilesPage.vue` — inspirée de `MasterInstallPage.vue` (NiTriTe), simplifiée : pas de dry-run modal ni d'export de script (YAGNI pour ce MVP, ajoutable plus tard si demandé). Réutilise `appCatalog` comme source unique de vérité (aucune duplication de données) :
- Profils prédéfinis (`src/data/installProfiles.ts`, nouveau) : listes d'`id` du catalogue existant, pas de nouvelles entrées de paquets. Ex. "Essentiels" (firefox, libreoffice, vlc), "Développement" (les entrées `category === "Développement"` du catalogue déjà présentes), etc. — construits à partir de ce qui existe déjà dans `appCatalog.ts`, pas inventés.
- Sélection multiple (checkboxes) à l'intérieur d'un profil ou librement dans le catalogue complet.
- Installation séquentielle (une à la fois, pas en parallèle — évite de saturer le gestionnaire de paquets natif avec des verrous concurrents), réutilise le même dispatch par `installMethod` que Section 4.
- Résumé final : liste succès/échecs par app, pas de modal bloquant.

## 6. Bonus : santé S.M.A.R.T. dans `DisksPage.vue`

Pour chaque disque listé (`disk.name`, ex. `"sda"`), un bouton "Vérifier la santé" appelle `get_smart_status({ device: "/dev/" + disk.name })`. Résultat affiché via `NxBadge` (`success` si `health === "PASSED"`, `danger` sinon) ou message explicite si la commande échoue (cas attendu sans root — `smartctl` retourne une erreur de permission sur la plupart des systèmes, déjà documenté dans `smart.rs`, géré comme un `Result` normal côté frontend, pas un crash).

## 7. Hors scope

- `tools`/`os-downloads` (voir §1).
- Snap `--classic` confinement.
- Export de script d'installation, mode dry-run (MasterInstallPage's fonctionnalités avancées) — YAGNI pour ce MVP.
- Désinstallation Flatpak/Snap (seule l'installation ferme le gap "Bientôt disponible" ; `PackagesPage.vue`/désinstalleur existant reste apt/dnf/pacman/zypper uniquement pour l'instant — pas de régression, juste pas d'extension).
