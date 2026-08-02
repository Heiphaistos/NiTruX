# Phase R12 (Outils système — catalogue de commandes en un clic) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a new "Outils système" navigation category — a searchable, filterable grid of one-click command-launcher buttons (mirroring NiTriTe Windows's `ToolsPage.vue` UX), so a beginner Linux user can run useful diagnostic/maintenance commands without ever opening a terminal.

**Architecture:** Non-privileged commands (the majority, ~41 entries) reuse `run_script` (existing since R8, runs `sh -c <content>` as the invoking user — no new privilege boundary since every command string is fixed by this plan, never user-typed). Privileged commands (7 entries, all safe/idempotent/non-destructive) share ONE new consolidated pkexec action (`org.heiphaistos.nitrux.system-tools`, exec path `nitrux-pkexec-system-tools`) with a hardcoded subcommand switch — the same pattern already in production for `nitrux-pkexec-troubleshoot`'s 4 subcommands. Never a shared "run arbitrary command as root" path.

**Tech Stack:** Same as R1-R11 — Tauri v2 + Rust backend, Vue 3 + Pinia frontend, vitest + `cargo test`.

---

## Task 1: `system_tools.rs` (privileged, new consolidated pkexec action)

**Files:**
- Create: `src-tauri/src/system_tools.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/packaging/nitrux-pkexec-helper`
- Create: `src-tauri/packaging/org.heiphaistos.nitrux.system-tools.policy`
- Modify: `src-tauri/tauri.conf.json`

### Backend Rust

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/system_tools.rs
use crate::subprocess;
use std::time::Duration;

/// The complete, fixed set of privileged one-click actions this catalog
/// exposes. Deliberately a closed allowlist -- NEVER accept an arbitrary
/// action string and NEVER construct the underlying shell command from
/// anything but a hardcoded match arm, both here and in the wrapper script
/// this dispatches to. Each of these 7 actions was chosen because it is
/// safe, idempotent, and never touches user data (see the design spec's
/// table for the exact command each one runs).
const VALID_ACTIONS: &[&str] = &[
    "apt-autoremove",
    "journal-vacuum-size",
    "rebuild-ld-cache",
    "systemd-reload",
    "fstrim-all",
    "rebuild-locate-db",
    "regenerate-grub",
];

pub fn validate_action(action: &str) -> Result<(), String> {
    if VALID_ACTIONS.contains(&action) {
        Ok(())
    } else {
        Err(format!("action système inconnue : {action}"))
    }
}

/// Runs one of the 7 fixed privileged system-tool actions, escalating
/// through polkit. Mirrors `packages::install::install_package`'s
/// dedicated-exec-path reasoning: `nitrux-pkexec-system-tools` is its own
/// on-disk copy of the shared wrapper script, never reusing another
/// action's exec path, since pkexec resolves the action purely by exec
/// path with no visibility into argv.
#[tauri::command]
pub fn run_system_tool(action: String) -> Result<String, String> {
    validate_action(&action)?;
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-system-tools", "system-tool", &action],
        Duration::from_secs(60),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_every_known_action() {
        for action in VALID_ACTIONS {
            assert!(validate_action(action).is_ok(), "should accept {action}");
        }
    }

    #[test]
    fn rejects_unknown_action() {
        assert!(validate_action("rm-rf-root").is_err());
    }

    #[test]
    fn rejects_empty_action() {
        assert!(validate_action("").is_err());
    }

    #[test]
    fn run_system_tool_rejects_unknown_action_before_ever_shelling_out() {
        let result = run_system_tool("bogus-action; rm -rf /".to_string());
        assert!(result.is_err());
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (module not registered).

- [ ] **Step 3: Register `mod system_tools;` in `lib.rs`** (alphabetically — sorts right after `mod system;`, before `mod trash;`) and add `system_tools::run_system_tool,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `211 passed; 0 failed; 1 ignored`** (207 R11 baseline + 4 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/system_tools.rs src-tauri/src/lib.rs
git commit -m "feat: add run_system_tool — consolidated pkexec action for 7 safe root maintenance commands (spec section 2.2)"
```

### Packaging (privileged surface — full dedicated review required)

- [ ] **Step 6: Add the `system-tool` subcommand to `nitrux-pkexec-helper`**

Add a new `case` arm right after the `install-snap)` arm (or any convenient spot in the `case "$cmd" in` block):

```sh
  system-tool)
    action="${2:-}"
    case "$action" in
      apt-autoremove)
        if command -v apt-get >/dev/null 2>&1; then apt-get autoremove -y; fi
        if command -v dnf >/dev/null 2>&1; then dnf autoremove -y; fi
        if command -v zypper >/dev/null 2>&1; then zypper --non-interactive remove --clean-deps; fi
        ;;
      journal-vacuum-size)
        exec journalctl --vacuum-size=200M
        ;;
      rebuild-ld-cache)
        exec ldconfig
        ;;
      systemd-reload)
        exec systemctl daemon-reload
        ;;
      fstrim-all)
        exec fstrim -av
        ;;
      rebuild-locate-db)
        if command -v updatedb >/dev/null 2>&1; then exec updatedb; fi
        echo "updatedb non installé" >&2
        exit 1
        ;;
      regenerate-grub)
        if command -v update-grub >/dev/null 2>&1; then exec update-grub; fi
        if command -v grub2-mkconfig >/dev/null 2>&1; then exec grub2-mkconfig -o /boot/grub2/grub.cfg; fi
        echo "aucun générateur de configuration GRUB trouvé" >&2
        exit 1
        ;;
      *)
        die "unknown system-tool action: $action"
        ;;
    esac
    ;;
```

Update the script's header comment: bump "13 distinct names" to "14", add `nitrux-pkexec-system-tools` to the list, and add the usage line:
```
#   nitrux-pkexec-system-tools system-tool <action>
```

- [ ] **Step 7: Create `org.heiphaistos.nitrux.system-tools.policy`**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE policyconfig PUBLIC "-//freedesktop//DTD PolicyKit Policy Configuration 1.0//EN"
 "http://www.freedesktop.org/standards/PolicyKit/1/policyconfig.dtd">
<policyconfig>
  <vendor>NiTruX</vendor>
  <vendor_url>https://github.com/Heiphaistos/NiTruX</vendor_url>

  <action id="org.heiphaistos.nitrux.system-tools">
    <description>Exécuter un outil de maintenance système</description>
    <message>NiTruX veut exécuter un outil de maintenance système</message>
    <defaults>
      <allow_any>auth_admin</allow_any>
      <allow_inactive>auth_admin</allow_inactive>
      <allow_active>auth_admin</allow_active>
    </defaults>
    <annotate key="org.freedesktop.policykit.exec.path">/usr/bin/nitrux-pkexec-system-tools</annotate>
  </action>
</policyconfig>
```

- [ ] **Step 8: Add the file mapping to `tauri.conf.json`**

In both `bundle.linux.deb.files` and `bundle.linux.rpm.files`, add these two entries (new policy file + new exec-path name pointing at the shared wrapper script):

```json
          "/usr/share/polkit-1/actions/org.heiphaistos.nitrux.system-tools.policy": "packaging/org.heiphaistos.nitrux.system-tools.policy",
          "/usr/bin/nitrux-pkexec-system-tools": "packaging/nitrux-pkexec-helper",
```

- [ ] **Step 9: Commit packaging**

```bash
git add src-tauri/packaging/nitrux-pkexec-helper src-tauri/packaging/org.heiphaistos.nitrux.system-tools.policy src-tauri/tauri.conf.json
git commit -m "feat: register the new system-tools pkexec action and policy (spec section 2.2)"
```

- [ ] **Step 10: Build and verify EVERY one of the 7 privileged actions live on the VM before merging**

Build: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r12-outils-systeme && npx tauri build 2>&1 | tail -30"`.

Deploy the `.deb` to the VM (`172.18.32.124`, user `dev`, password `1998`), install it. For each of the 7 actions, run via `ssh_interactive.py` (mirrors the R10 Task 2 `install-snap` verification pattern exactly):
```
set +m; pkttyagent --process $$ & sleep 1; pkexec /usr/bin/nitrux-pkexec-system-tools system-tool <action>
```
Confirm the line `==== AUTHENTICATING FOR org.heiphaistos.nitrux.system-tools ====` appears for at least one action (proves polkit resolved this new, distinct action). For each of the 7, confirm the command's real effect or clean exit:
- `apt-autoremove` — confirm it runs without error (this VM may have nothing to autoremove, exit 0 either way).
- `journal-vacuum-size` — confirm output mentions vacuuming (e.g. "Vacuuming done").
- `rebuild-ld-cache` — confirm clean exit (ldconfig produces no output on success).
- `systemd-reload` — confirm clean exit.
- `fstrim-all` — confirm output lists the trimmed filesystem(s) or "not trimmed" for filesystems that don't support it (both are success states).
- `rebuild-locate-db` — confirm either it runs (if `updatedb`/`plocate` is installed) or cleanly reports "non installé" (check `which updatedb` first to know which outcome to expect).
- `regenerate-grub` — confirm it runs `update-grub` successfully (Debian VM, so this is the expected path, not the Fedora fallback).

Also test the **rejection path**: `pkexec /usr/bin/nitrux-pkexec-system-tools system-tool bogus-action` must fail cleanly (non-zero exit, no destructive side effect).

---

## Task 2: `systemToolsCatalog.ts` (non-privileged catalog data)

**Files:**
- Create: `src/data/systemToolsCatalog.ts`

- [ ] **Step 1: Write the catalog.** All commands below were confirmed runnable without root on the project's dev VM during this plan's research (see design spec §3), except entries marked "si installé", which degrade gracefully via `run_script`'s existing error surfacing rather than crashing.

```typescript
// src/data/systemToolsCatalog.ts
export interface SystemTool {
  id: string;
  name: string;
  description: string;
  category: "diagnostics" | "reseau" | "performance" | "nettoyage" | "stockage" | "privilegie";
  /** Exactly one of `command`/`privilegedAction` is set, never both. */
  command?: string;
  /** One of system_tools.rs's 7 fixed action names -- routed through
   *  run_system_tool (pkexec) instead of run_script. */
  privilegedAction?: string;
}

export const systemToolsCatalog: SystemTool[] = [
  // Diagnostics système
  { id: "uname", name: "Informations noyau", description: "Version du noyau et architecture.", category: "diagnostics", command: "uname -a" },
  { id: "uptime", name: "Disponibilité", description: "Depuis combien de temps le système tourne.", category: "diagnostics", command: "uptime -p" },
  { id: "free", name: "Mémoire", description: "Utilisation de la RAM et du swap.", category: "diagnostics", command: "LC_ALL=C free -h" },
  { id: "df", name: "Espace disque", description: "Espace utilisé/disponible par système de fichiers monté.", category: "diagnostics", command: "df -h" },
  { id: "lsblk", name: "Périphériques bloc", description: "Arborescence des disques et partitions.", category: "diagnostics", command: "lsblk" },
  { id: "lscpu", name: "Détails processeur", description: "Modèle, cœurs, threads, architecture.", category: "diagnostics", command: "LC_ALL=C lscpu" },
  { id: "whoami-id", name: "Identité courante", description: "Utilisateur et groupes actuels.", category: "diagnostics", command: "whoami && id" },
  { id: "hostnamectl", name: "Informations machine", description: "Nom d'hôte, type de machine, noyau, OS.", category: "diagnostics", command: "hostnamectl status" },
  { id: "timedatectl", name: "Date & heure", description: "Fuseau horaire et synchronisation NTP.", category: "diagnostics", command: "timedatectl status" },
  { id: "localectl", name: "Paramètres régionaux", description: "Langue système et disposition clavier.", category: "diagnostics", command: "localectl status" },
  { id: "w", name: "Sessions actives", description: "Utilisateurs connectés et leur activité.", category: "diagnostics", command: "w" },
  { id: "last", name: "Dernières connexions", description: "Historique des 10 dernières connexions.", category: "diagnostics", command: "last -n 10" },
  { id: "nproc", name: "Nombre de cœurs", description: "Nombre de cœurs/threads disponibles.", category: "diagnostics", command: "nproc" },
  { id: "failed-units", name: "Services en échec", description: "Services systemd actuellement en échec.", category: "diagnostics", command: "systemctl --failed --no-pager --plain" },
  { id: "journal-errors", name: "Erreurs récentes", description: "50 dernières erreurs du journal système (accès partiel possible selon vos groupes).", category: "diagnostics", command: "journalctl -p err -b --no-pager -n 50" },
  { id: "vmstat", name: "Statistiques mémoire/CPU", description: "Aperçu mémoire, swap et CPU sur 2 secondes.", category: "diagnostics", command: "LC_ALL=C vmstat 1 2" },
  { id: "sensors", name: "Capteurs matériels", description: "Températures et tensions (si lm-sensors installé).", category: "diagnostics", command: "sensors" },
  { id: "dmesg", name: "Journal noyau", description: "Messages du noyau (souvent restreint sans root).", category: "diagnostics", command: "dmesg" },

  // Réseau
  { id: "ip-a", name: "Interfaces réseau", description: "Liste des interfaces et adresses IP.", category: "reseau", command: "ip a" },
  { id: "ip-route", name: "Table de routage", description: "Routes réseau configurées.", category: "reseau", command: "ip route" },
  { id: "ip-link-stats", name: "Statistiques interfaces", description: "Compteurs de paquets/erreurs par interface.", category: "reseau", command: "ip -s link" },
  { id: "nmcli-device", name: "Périphériques réseau", description: "État des connexions via NetworkManager.", category: "reseau", command: "nmcli device status" },
  { id: "nmcli-connection", name: "Connexions configurées", description: "Liste des profils de connexion NetworkManager.", category: "reseau", command: "nmcli connection show" },
  { id: "ping", name: "Test de connectivité", description: "4 pings vers 8.8.8.8.", category: "reseau", command: "ping -c 4 8.8.8.8" },
  { id: "public-ip", name: "Adresse IP publique", description: "Votre IP publique actuelle via DNS OpenDNS.", category: "reseau", command: "dig +short myip.opendns.com @resolver1.opendns.com" },
  { id: "dig-google", name: "Résolution DNS (dig)", description: "Résout google.com via dig.", category: "reseau", command: "dig google.com" },
  { id: "host-google", name: "Résolution DNS (host)", description: "Résout google.com via host.", category: "reseau", command: "host google.com" },
  { id: "traceroute", name: "Traceroute", description: "Chemin réseau vers 8.8.8.8.", category: "reseau", command: "traceroute -m 15 8.8.8.8" },
  { id: "ss-ports", name: "Ports en écoute", description: "Sockets TCP/UDP en écoute.", category: "reseau", command: "ss -tulpn" },
  { id: "resolvectl", name: "État de la résolution DNS", description: "Résolveurs DNS actifs par interface (si systemd-resolved installé).", category: "reseau", command: "resolvectl status" },

  // Performance
  { id: "ps-cpu", name: "Top processus CPU", description: "15 processus consommant le plus de CPU.", category: "performance", command: "ps aux --sort=-%cpu | head -15" },
  { id: "ps-mem", name: "Top processus mémoire", description: "15 processus consommant le plus de mémoire.", category: "performance", command: "ps aux --sort=-%mem | head -15" },
  { id: "top-snapshot", name: "Aperçu instantané", description: "Instantané top (sans rafraîchissement continu).", category: "performance", command: "top -bn1 | head -20" },

  // Nettoyage (utilisateur, non-privilégié)
  { id: "clean-thumbnails", name: "Vider le cache des miniatures", description: "Supprime les vignettes d'images mises en cache.", category: "nettoyage", command: "rm -rf ~/.cache/thumbnails/*" },
  { id: "clean-old-cache", name: "Nettoyer le cache ancien", description: "Supprime les fichiers de cache non touchés depuis 30 jours.", category: "nettoyage", command: "find ~/.cache -type f -atime +30 -delete" },
  { id: "cache-size", name: "Taille du cache", description: "Espace occupé par le dossier de cache utilisateur.", category: "nettoyage", command: "du -sh ~/.cache" },
  { id: "npm-cache-clean", name: "Vider le cache npm", description: "Nettoie le cache npm (si npm installé).", category: "nettoyage", command: "npm cache clean --force" },
  { id: "pip-cache-purge", name: "Vider le cache pip", description: "Nettoie le cache pip (si pip installé).", category: "nettoyage", command: "pip cache purge" },

  // Stockage
  { id: "lsblk-fs", name: "Systèmes de fichiers", description: "Disques avec type de système de fichiers et point de montage.", category: "stockage", command: "lsblk -f" },
  { id: "biggest-home-dirs", name: "Plus gros dossiers du home", description: "10 plus gros éléments de votre dossier personnel.", category: "stockage", command: "du -sh ~/* 2>/dev/null | sort -rh | head -10" },

  // Privilégié (root, via l'action polkit consolidée)
  { id: "apt-autoremove", name: "Retirer les paquets orphelins", description: "Supprime les dépendances devenues inutiles.", category: "privilegie", privilegedAction: "apt-autoremove" },
  { id: "journal-vacuum-size", name: "Réduire les journaux (taille)", description: "Limite le journal système à 200 Mo.", category: "privilegie", privilegedAction: "journal-vacuum-size" },
  { id: "rebuild-ld-cache", name: "Reconstruire le cache des bibliothèques", description: "Reconstruit le cache de l'éditeur de liens dynamique (ldconfig).", category: "privilegie", privilegedAction: "rebuild-ld-cache" },
  { id: "systemd-reload", name: "Recharger systemd", description: "Recharge les fichiers d'unités systemd modifiés.", category: "privilegie", privilegedAction: "systemd-reload" },
  { id: "fstrim-all", name: "TRIM des disques SSD", description: "Exécute TRIM sur tous les systèmes de fichiers compatibles.", category: "privilegie", privilegedAction: "fstrim-all" },
  { id: "rebuild-locate-db", name: "Reconstruire la base locate", description: "Met à jour la base de données utilisée par la commande locate.", category: "privilegie", privilegedAction: "rebuild-locate-db" },
  { id: "regenerate-grub", name: "Régénérer la configuration GRUB", description: "Régénère la configuration de démarrage après un changement de noyau.", category: "privilegie", privilegedAction: "regenerate-grub" },
];
```

- [ ] **Step 2: Commit**

```bash
git add src/data/systemToolsCatalog.ts
git commit -m "feat: add systemToolsCatalog — 41 non-privileged + 7 privileged one-click commands (spec section 3)"
```

---

## Task 3: `SystemToolsPage.vue`

**Files:**
- Create: `src/pages/SystemToolsPage.vue`
- Test: `src/pages/SystemToolsPage.spec.ts`

- [ ] **Step 1: Write the failing frontend test**

Trace both assertions against the template in Step 3 by hand before writing it (discipline established since R8, reused through R9-R11).

```typescript
// src/pages/SystemToolsPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import SystemToolsPage from "./SystemToolsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "run_script" && args?.content === "uname -a") {
      return Promise.resolve("Linux DEV 6.12.0 x86_64");
    }
    if (cmd === "run_system_tool" && args?.action === "systemd-reload") {
      return Promise.resolve("");
    }
    return Promise.resolve(null);
  }),
}));

describe("SystemToolsPage", () => {
  it("runs a non-privileged tool via run_script and shows its output", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(SystemToolsPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Informations noyau"))!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Linux DEV 6.12.0 x86_64"));
    expect(invoke).toHaveBeenCalledWith("run_script", { content: "uname -a" });
  });

  it("runs a privileged tool via run_system_tool, not run_script", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(SystemToolsPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Recharger systemd"))!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("run_system_tool", { action: "systemd-reload" }));
  });

  it("shows a root badge on privileged tools", () => {
    const wrapper = mount(SystemToolsPage);
    const card = wrapper.findAll(".st-card").find((c) => c.text().includes("Recharger systemd"))!;
    expect(card.text()).toContain("root");
  });

  it("filters the catalog by category", async () => {
    const wrapper = mount(SystemToolsPage);
    const chips = wrapper.findAll(".st-chip");
    const networkChip = chips.find((c) => c.text() === "Réseau")!;
    await networkChip.trigger("click");
    expect(wrapper.text()).toContain("Test de connectivité");
    expect(wrapper.text()).not.toContain("Informations noyau");
  });

  it("filters the catalog by search text", async () => {
    const wrapper = mount(SystemToolsPage);
    const search = wrapper.find("input");
    await search.setValue("DNS");
    expect(wrapper.text()).toContain("Résolution DNS");
    expect(wrapper.text()).not.toContain("Informations noyau");
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

- [ ] **Step 3: Write `SystemToolsPage.vue`**

```vue
<!-- src/pages/SystemToolsPage.vue -->
<script setup lang="ts">
import { ref, computed } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { systemToolsCatalog, type SystemTool } from "@/data/systemToolsCatalog";

const CATEGORY_LABELS: Record<SystemTool["category"], string> = {
  diagnostics: "Diagnostics",
  reseau: "Réseau",
  performance: "Performance",
  nettoyage: "Nettoyage",
  stockage: "Stockage",
  privilegie: "Privilégié",
};

const activeCategory = ref<string>("all");
const search = ref("");
const running = ref<string | null>(null);
const outputs = ref<Record<string, string>>({});
const errors = ref<Record<string, string>>({});

const categories = computed(() => ["all", ...new Set(systemToolsCatalog.map((t) => t.category))]);

const filteredCatalog = computed<SystemTool[]>(() => {
  let result = systemToolsCatalog;
  if (activeCategory.value !== "all") {
    result = result.filter((t) => t.category === activeCategory.value);
  }
  if (search.value) {
    const q = search.value.toLowerCase();
    result = result.filter((t) => t.name.toLowerCase().includes(q) || t.description.toLowerCase().includes(q));
  }
  return result;
});

async function runTool(tool: SystemTool) {
  running.value = tool.id;
  delete errors.value[tool.id];
  try {
    const output = tool.privilegedAction
      ? await invoke<string>("run_system_tool", { action: tool.privilegedAction })
      : await invoke<string>("run_script", { content: tool.command });
    outputs.value = { ...outputs.value, [tool.id]: output || "(terminé, aucune sortie)" };
  } catch (e) {
    errors.value = { ...errors.value, [tool.id]: String(e) };
  } finally {
    running.value = null;
  }
}
</script>

<template>
  <div class="st-page">
    <NxSectionHeader title="Outils système" description="Commandes courantes en un clic, sans terminal." />

    <div class="st-chips">
      <button
        v-for="cat in categories"
        :key="cat"
        class="st-chip"
        :class="{ active: activeCategory === cat }"
        @click="activeCategory = cat"
      >
        {{ cat === "all" ? "Tout" : CATEGORY_LABELS[cat as SystemTool["category"]] }}
      </button>
    </div>

    <NxInput v-model="search" placeholder="Rechercher un outil..." />

    <div class="st-grid">
      <NxCard v-for="tool in filteredCatalog" :key="tool.id" class="st-card">
        <div class="st-card-header">
          <strong>{{ tool.name }}</strong>
          <NxBadge v-if="tool.privilegedAction" status="warning">root</NxBadge>
        </div>
        <p class="st-desc">{{ tool.description }}</p>
        <NxButton :disabled="running === tool.id" @click="runTool(tool)">
          {{ running === tool.id ? "Exécution..." : "Exécuter" }}
        </NxButton>
        <pre v-if="outputs[tool.id]" class="st-output">{{ outputs[tool.id] }}</pre>
        <NxCard v-if="errors[tool.id]" danger class="st-error">{{ errors[tool.id] }}</NxCard>
      </NxCard>
    </div>
  </div>
</template>

<style scoped>
.st-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.st-chips { display: flex; gap: 8px; flex-wrap: wrap; }
.st-chip { padding: 6px 14px; border-radius: 99px; border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-secondary); cursor: pointer; font: inherit; font-size: 12px; }
.st-chip.active { color: var(--nx-text-primary); font-weight: 600; border-color: var(--nx-accent-primary); }
.st-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(260px, 1fr)); gap: 14px; }
.st-card { display: flex; flex-direction: column; gap: 8px; }
.st-card-header { display: flex; align-items: center; justify-content: space-between; gap: 8px; }
.st-desc { font-size: 12px; color: var(--nx-text-secondary); margin: 0; }
.st-output { font-size: 11px; background: var(--nx-bg-base); padding: 8px; border-radius: 6px; max-height: 160px; overflow: auto; white-space: pre-wrap; word-break: break-word; }
.st-error { font-size: 12px; padding: 8px 10px; }
</style>
```

- [ ] **Step 4: Run it, confirm it PASSES (5 tests)**

- [ ] **Step 5: Commit**

```bash
git add src/pages/SystemToolsPage.vue src/pages/SystemToolsPage.spec.ts
git commit -m "feat: add SystemToolsPage — searchable one-click command catalog (spec section 2.3)"
```

---

## Task 4: Wire navigation

**Files:**
- Modify: `src/navigation/categories.ts`
- Modify: `src/navigation/categories.spec.ts`
- Modify: `src/components/nav/AppNav.vue`
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`

- [ ] **Step 1: Add a new "Outils système" category to `categories.ts`**, inserted right after the "diagnostic-avance" category (before "performance"):

```typescript
  {
    id: "outils-systeme",
    title: "Outils système",
    pages: [
      { id: "system-tools", label: "Commandes rapides", icon: "terminal" },
    ],
  },
```

- [ ] **Step 2: Add the `terminal` icon to `AppNav.vue`'s `iconMap`** — new import:
```typescript
  Terminal,
```
and:
```typescript
  terminal: Terminal,
```

- [ ] **Step 3: Update `categories.spec.ts`** — category count goes from 9 to 10:
```typescript
  it("has exactly 10 categories", () => {
    expect(navigationCategories).toHaveLength(10);
  });
```
Add:
```typescript
  it("includes the new Phase R12 Outils système page by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("system-tools");
  });
```

- [ ] **Step 4: Wire `App.vue`** — add the import:
```typescript
import SystemToolsPage from "@/pages/SystemToolsPage.vue";
```
and the map entry:
```typescript
  "system-tools": SystemToolsPage,
```

- [ ] **Step 5: Update the "renders AppNav with all N category titles" test in `App.spec.ts`** to 10 categories, adding `"Outils système"`. Add a new dedicated test:
```typescript
  it("shows the real SystemToolsPage for the system-tools id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Commandes rapides")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("sans terminal");
  });
```

- [ ] **Step 6: Run `npx vitest run src/App.spec.ts src/navigation/categories.spec.ts src/components/nav/AppNav.spec.ts`, confirm all pass.**

- [ ] **Step 7: Commit**

```bash
git add src/navigation/categories.ts src/navigation/categories.spec.ts src/components/nav/AppNav.vue src/App.vue src/App.spec.ts
git commit -m "feat: add Outils système navigation category (spec section 2.3)"
```

---

## Task 5: Full verification pass — frontend, backend, and live VM check

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite.**

Expected baseline entering this plan: 217 (end of R11). Net delta: Task 3 (+5) + Task 4 (+1 categories.spec.ts new test + 1 App.spec.ts new test, the two count-updates to existing tests are modifications not additions) = +7, expected total **224**. Report the real observed total and reconcile if it differs — verify, don't trust blindly.

- [ ] **Step 2: Type-check.** `npx vue-tsc --noEmit`, expect clean.

- [ ] **Step 3: Confirm the Rust suite.** Expect `211 passed; 0 failed; 1 ignored` (207 R11 baseline + 4 Task 1).

- [ ] **Step 4: Spot-check a representative sample of non-privileged catalog entries live on the VM** (the full privileged set was already verified in Task 1 Step 10 — this step covers the non-privileged majority, at least one per category): `uname -a`, `dig +short myip.opendns.com @resolver1.opendns.com`, `ps aux --sort=-%cpu | head -15`, `rm -rf ~/.cache/thumbnails/*` (confirm it's a no-op/safe on an empty dir), `lsblk -f`. Confirm each runs via SSH exactly as the catalog specifies and produces sensible output.

- [ ] **Step 5: Commit any final cleanup.** No further commit expected if Steps 1-4 all pass clean.

---

## Self-Review

**Spec coverage:** §2.1 (non-privileged architecture, reuse `run_script`) — Task 2/3. §2.2 (privileged consolidated pkexec action) — Task 1. §2.3 (frontend page + nav) — Task 3/4. §3 (catalog) — Task 2. §4 (out of scope: Activation category, categories already covered elsewhere) — confirmed no task in this plan touches those.

**Placeholder scan:** No "TBD"/"TODO". `journal-errors`'s partial-access-under-restricted-groups behavior and `dmesg`'s root restriction are both explicitly documented, expected outcomes, not oversights.

**Type consistency:** `SystemTool`'s `category` union (Task 2) matches `CATEGORY_LABELS`'s keys exactly (Task 3) — cross-checked by hand while writing both. `run_system_tool`'s `action: String` param (Task 1, Rust) matches the frontend's `{ action: tool.privilegedAction }` call shape (Task 3) exactly, and `run_script`'s existing `{ content }` shape (confirmed by reading `ScriptsPage.vue`'s real usage during this plan's research) is used unchanged, not guessed.

**Security invariant check:** Every privileged action in `system_tools.rs`'s `VALID_ACTIONS` allowlist has a corresponding, and ONLY that, hardcoded `case` arm in the wrapper script — no dynamic command construction from `$action` anywhere in the dispatch path. The wrapper's `case "$action" in ... *) die ...` default arm rejects anything not explicitly enumerated, mirroring the existing `troubleshoot` action's own structure exactly.
