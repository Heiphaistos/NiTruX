# NiTruX Phase R6 — Fondation visuelle + catégorie Performance — Design

## 1. Contexte et motivation

Après la refonte R1-R5 (nav catégorisée, 3 axes visuels, 4 nouvelles catégories fonctionnelles), l'utilisateur juge le résultat toujours "moche et fade" et signale qu'il manque encore une grande partie des fonctionnalités de NiTriTe Windows.

**Investigation concrète (pas de supposition) :**
- `C:\Users\Momo\Desktop\Nitrite 2.0\src\data\navigation.ts` liste **44 pages réparties en 10 catégories**. NiTruX n'en a que 15 (~34% de couverture).
- `NiTruX\src\navigation\categories.ts` définit déjà un champ `icon` (nom d'icône lucide, ex. `"stethoscope"`) pour chacune de ses 15 pages — mais `AppNav.vue` ne le lit jamais, n'affichant que du texte brut. **NiTruX n'a aucune librairie d'icônes installée.** NiTriTe utilise `lucide-vue-next` partout : nav, boutons, tuiles d'action colorées par dégradé sur le tableau de bord.
- `DashboardPage.vue` (page d'atterrissage par défaut) n'a **jamais été migrée** vers la bibliothèque de composants partagés `Nx*` ni vers les tokens de style `--nx-style-*` introduits en R1 — elle utilise encore d'anciennes variables CSS codées en dur (`--nx-border`, `--nx-bg-elevated`). Résultat concret : changer de style visuel (parmi les 12 disponibles) n'affecte visuellement presque pas la toute première page vue par l'utilisateur.

Ces trois éléments (pas d'icônes nulle part, pas de tuiles colorées, dashboard non stylé) expliquent une part importante du ressenti "fade" — ce ne sont pas des questions de goût mais des lacunes d'implémentation concrètes et réparables.

**Décision de périmètre (validée avec l'utilisateur) :** les 29 pages manquantes/à dédier sont réparties en 6 phases thématiques (R6 à R11). La catégorie "Intelligence" de NiTriTe (agent IA, base de connaissances, documentation intégrée) est explicitement **hors scope**. Les fonctionnalités Windows-only sans équivalent direct (WinPE, analyse BSOD, WSL) seront adaptées en équivalents Linux pertinents dans une phase ultérieure (R10/R11), pas ignorées.

Cette phase, **R6**, est la première : elle corrige la fondation visuelle (icônes + tuiles d'action colorées, dashboard enfin componentisé) **en même temps** qu'elle livre la première nouvelle catégorie de nav, "Performance", avec 4 pages réelles — pas une phase "juste des icônes" sans valeur fonctionnelle nouvelle.

## 2. Fondation visuelle

### 2.1 Dépendance
Ajout de `lucide-vue-next` (même version que NiTriTe : `^0.474.0`) — aucune autre nouvelle dépendance npm.

### 2.2 Icônes dans la navigation
`AppNav.vue` (R1, jusqu'ici du texte brut) reçoit une petite table `iconMap: Record<string, Component>` mappant les noms d'icônes déjà présents dans `categories.ts` (`layout-dashboard`, `stethoscope`, `download`, `package`, `hard-drive`, `files`, `refresh-cw`, `cpu`, `wrench`, `wifi`, `shield`, `file-text`, `scroll-text`, `settings`, `palette`) vers les composants `lucide-vue-next` correspondants, plus les 4 nouveaux noms introduits par cette phase (`zap`, `thermometer`, `gauge`, `bar-chart-3` — mêmes noms que `navigation.ts` de NiTriTe pour "Optimisations"/"Températures"/"Benchmark"/"Historique Perf.", cohérence délibérée). Un item sans correspondance retombe sur une icône neutre (`Circle`), jamais un crash. Fallback identique au pattern déjà établi ailleurs dans ce projet (`pages[currentPage] ?? pages.dashboard`).

Comme les 15 pages existantes ont déjà leurs noms d'icônes dans `categories.ts`, ce seul changement d'`AppNav.vue` corrige visuellement **toute** la navigation existante sans toucher à chacune des 15 pages individuellement.

### 2.3 Tuiles d'action rapide sur le tableau de bord
Nouveau composant partagé `src/components/ui/NxQuickActionTile.vue` : icône + libellé + dégradé de fond spécifique à l'action (couleur en `linear-gradient`, pas liée aux 12 styles/12 palettes existants — un choix délibéré et scopé : cette teinte n'est utilisée QUE sur ces tuiles du dashboard, le reste de l'app garde le système de style existant intact, conformément à la décision utilisateur). 5 tuiles pointant vers des pages déjà réellement construites (aucune tuile ne pointe vers une page pas encore livrée) :

| Libellé | Icône | Dégradé | Cible |
|---|---|---|---|
| Diagnostic | `stethoscope` | orange (`#f97316→#fb923c`) | `diagnostic` |
| Installation rapide | `download` | bleu (`#3b82f6→#2563eb`) | `quick-install` |
| Mises à jour | `refresh-cw` | vert (`#22c55e→#16a34a`) | `updates` |
| Dépannage | `wrench` | rouge (`#ef4444→#dc2626`) | `troubleshoot` |
| Générateur de rapport | `file-text` | violet (`#8b5cf6→#7c3aed`) | `report-generator` |

Cliquer une tuile émet un événement que `App.vue` écoute pour changer `currentPage` — mécanisme simple, pas de nouvelle gestion d'état globale.

### 2.4 Componentisation de `DashboardPage.vue`
Remplacement des `.dash-card` codés en dur par `NxCard`/`NxStatTile`/`NxSectionHeader` (bibliothèque R1), et des variables `--nx-border`/`--nx-bg-elevated` par les tokens `--nx-style-*`. Toute la logique métier (polling 2s, calcul GB, gestion des erreurs séparées snapshot/capteurs) reste strictement identique — seule la présentation change, même discipline que la componentisation R2.

## 3. Nouvelle catégorie de navigation : Performance

`categories.ts` gagne une 8e catégorie (7→8), insérée après "Système" (ordre cohérent avec `navigation.ts` de NiTriTe, où "Performance" suit directement "Système"/"Logiciels") :

```typescript
{
  id: "performance",
  title: "Performance",
  pages: [
    { id: "optimizations", label: "Optimisations", icon: "zap" },
    { id: "temperatures", label: "Températures", icon: "thermometer" },
    { id: "benchmark", label: "Benchmark", icon: "gauge" },
    { id: "perf-history", label: "Historique perf.", icon: "bar-chart-3" },
  ],
},
```

Total pages : 15 → 19.

### 3.1 Températures (`temperatures`)
Réutilise intégralement la commande déjà existante et testée `get_sensor_snapshot` (aucun changement backend) — `sensors::SensorSnapshot.temperatures: Vec<TemperatureReading>` est déjà tout ce dont cette page a besoin. Présentation dédiée et plus riche que la simple ligne du tableau de bord : une `NxCard`/`NxStatTile` par capteur, avec un badge de couleur selon un seuil honnête (vert < 60 °C, orange 60-80 °C, rouge > 80 °C — seuils génériques, pas de config par matériel spécifique dans cette v1).

### 3.2 Benchmark (`benchmark`)
Nouveau module backend `src-tauri/src/benchmark.rs`, commande unique `run_benchmark() -> Result<BenchmarkResult, String>` :
- **CPU** : calcule des hachages SHA-256 en boucle pendant une fenêtre de temps fixe (réutilise la dépendance `sha2` déjà présente dans `Cargo.toml` — aucune nouvelle dépendance), rapporte un score en hachages/seconde.
- **Disque** : écrit puis relit un fichier temporaire de taille fixe dans `std::env::temp_dir()` (jamais dans un répertoire système), mesure les débits d'écriture/lecture en Mo/s, supprime le fichier immédiatement après (`Drop`/nettoyage explicite, y compris en cas d'erreur en cours de route).
- **Mémoire** : alloue un buffer et effectue une boucle de copie mémoire bornée en temps, mesure la bande passante en Go/s.

Aucune opération privilégiée — tout se passe dans l'espace utilisateur normal, aucun nouveau `pkexec`. Durée totale bornée (quelques secondes), un seul bouton "Lancer le benchmark" avec état de progression.

### 3.3 Historique perf. (`perf-history`)
Entièrement frontend — pas de nouvelle commande backend. La page interroge `get_system_snapshot`/`get_sensor_snapshot` (déjà utilisés par le dashboard) à l'intervalle déjà configurable via `preferencesStore.dashboardRefreshIntervalMs` (R2), accumule un buffer glissant côté client (ex. 60 derniers échantillons, perdu à la fermeture de la page — pas de persistance disque dans cette v1, cohérent avec YAGNI), et l'affiche via un nouveau composant partagé léger `src/components/ui/NxSparkline.vue` (un simple graphique en ligne SVG fait main, pas de nouvelle dépendance de graphique — mêmes contraintes de dépendances que le reste du projet).

### 3.4 Optimisations (`optimizations`) — lecture seule
Nouveau module backend `src-tauri/src/optimizations.rs`, entièrement en lecture seule (décision explicite de l'utilisateur : pas de nouvelle surface privilégiée dans cette phase) :
- `list_startup_services() -> Result<Vec<ServiceInfo>, String>` — parse `systemctl list-unit-files --type=service --state=enabled`
- lecture de `/proc/sys/vm/swappiness`
- détection zram (présence de périphériques `/dev/zram*` actifs en swap)
- statut du timer `fstrim.timer` via `systemctl is-enabled fstrim.timer`

Regroupés dans une commande unique `get_optimization_snapshot() -> Result<OptimizationSnapshot, String>`. La page affiche ces données avec des recommandations **textuelles uniquement**, honnêtes et non alarmistes (ex. "swappiness à 60 — valeur par défaut, envisager de la réduire sur un poste de travail avec beaucoup de RAM" plutôt qu'un bouton qui agirait). Aucune action n'écrit quoi que ce soit sur le système dans cette version.

## 4. Vérification

Comme pour R1-R5 : tests unitaires Rust (parsing `systemctl`/`/proc` avec des fixtures, calculs de benchmark testés sur leur logique pas leur résultat réel puisque dépendant du matériel), tests Vitest par page/composant, `vue-tsc --noEmit`, vérification manuelle sur la VM Debian (déjà accessible, déjà utilisée cette nuit) pour confirmer que `systemctl`/`/proc/sys/vm/swappiness`/le benchmark tournent réellement sur un vrai système Linux — pas seulement en mock. Merge, version bump (0.13.0→0.14.0 attendu), build `.deb`/`.rpm`, release GitHub, comme chaque phase précédente.

## 5. Hors scope pour R6

- Toute capacité d'action (activer/désactiver un service, ajuster swappiness) — nécessitera sa propre phase avec conception + test VM live dédiés, comme toute nouvelle surface `pkexec`.
- Couleurs d'accent par catégorie de nav (uniquement les 5 tuiles du dashboard pour cette phase, décision utilisateur).
- Persistance de l'historique de performance au-delà de la session (pas de base de données introduite).
- Les 22 autres pages manquantes (R7-R11) et la catégorie "Intelligence" (hors scope, décision utilisateur).
