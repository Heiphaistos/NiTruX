# Phase R10 (Logiciels & déploiement) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the "Bientôt disponible (flatpak/snap)" placeholder in `QuickInstallPage.vue` by implementing real Flatpak (non-privileged) and Snap (new pkexec action) install support, add a bulk-install "Installation par profils" page, and wire the orphaned `get_smart_status` command into `DisksPage.vue`.

**Architecture:** Flatpak installs run unprivileged in `--user` scope (no pkexec, same philosophy as R8's `run_script`). Snap always needs root, so it gets a 13th name for the existing `nitrux-pkexec-helper` script plus a 4th `<action>` in the existing `org.heiphaistos.nitrux.packages.policy` file — never a shared exec path with another action, per this project's non-negotiable pkexec discipline. `QuickInstallPage.vue`'s `install()` becomes method-aware. `InstallProfilesPage.vue` is pure frontend curation over the existing `appCatalog` — no new backend, no duplicated package data.

**Tech Stack:** Same as R1-R9 — Tauri v2 + Rust backend, Vue 3 + Pinia frontend, vitest + `cargo test`.

---

## Task 1: Flatpak install support (non-privileged)

**Files:**
- Create: `src-tauri/src/packages/flatpak.rs`
- Modify: `src-tauri/src/packages/mod.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/packages/flatpak.rs
use crate::packages::binary_exists;
use crate::packages::install::validate_package_name;
use crate::subprocess;
use std::time::Duration;

const FLATHUB_REPO_URL: &str = "https://flathub.org/repo/flathub.flatpakrepo";

/// Installs `app_id` (a reverse-DNS Flatpak application id, e.g.
/// "com.discordapp.Discord") in `--user` scope. Deliberately NOT routed
/// through pkexec: `flatpak install --user` writes to
/// `~/.local/share/flatpak`, under the invoking user's own account, the
/// same "not a new privilege boundary" reasoning R8 applied to
/// `run_script`. Reuses `validate_package_name` as-is -- it already
/// accepts '.', '+', ':', '_', '-', which covers every real Flatpak app id
/// without modification.
///
/// A fresh Flatpak install has zero remotes configured (confirmed on the
/// project's dev VM), so `flatpak install` would fail before ever reaching
/// the app id -- `remote-add --if-not-exists` is run first, unconditionally
/// and idempotently, so this works on both a fresh install and one that
/// already has flathub configured.
#[tauri::command]
pub fn install_flatpak_package(app_id: String) -> Result<String, String> {
    validate_package_name(&app_id)?;
    if !binary_exists("flatpak") {
        return Err("flatpak n'est pas installé sur ce système".to_string());
    }
    // Best-effort: if this fails for a reason other than "already exists"
    // (which --if-not-exists itself already suppresses), the install
    // attempt right after will surface a clear error anyway.
    let _ = subprocess::run_with_timeout(
        "flatpak",
        &["remote-add", "--if-not-exists", "--user", "flathub", FLATHUB_REPO_URL],
        Duration::from_secs(30),
    );
    subprocess::run_with_timeout(
        "flatpak",
        &["install", "--user", "--noninteractive", "flathub", &app_id],
        Duration::from_secs(300),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_empty_app_id() {
        assert!(install_flatpak_package("".to_string()).is_err());
    }

    #[test]
    fn rejects_app_id_with_shell_metacharacters() {
        assert!(install_flatpak_package("com.example.App; rm -rf /".to_string()).is_err());
    }

    #[test]
    fn accepts_a_well_formed_reverse_dns_app_id_through_validation() {
        // Validation is the only thing this test can exercise portably --
        // it does not actually run flatpak (would require it installed and
        // network access, neither guaranteed in a test sandbox). The real
        // install path is verified live on the VM in Task 6.
        assert!(validate_package_name("com.discordapp.Discord").is_ok());
        assert!(validate_package_name("com.valvesoftware.Steam").is_ok());
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (module not registered, `binary_exists` not `pub`).

- [ ] **Step 3: Make `binary_exists` public if not already, register `pub mod flatpak;` in `packages/mod.rs`**

Check `src-tauri/src/packages/mod.rs` — `binary_exists` is already `pub fn`. Add the module declaration alongside the existing ones (alphabetical order):

```rust
pub mod apt;
pub mod dnf;
pub mod flatpak;
pub mod install;
pub mod pacman;
pub mod universal;
pub mod zypper;
```

- [ ] **Step 4: Register the command in `lib.rs`**

Add `packages::flatpak::install_flatpak_package,` to the `invoke_handler` list, right after `packages::list_installed_packages,`.

- [ ] **Step 5: Run the full Rust suite, expect `187 passed; 0 failed; 1 ignored`** (184 R9 baseline + 3 new).

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/packages/flatpak.rs src-tauri/src/packages/mod.rs src-tauri/src/lib.rs
git commit -m "feat: add install_flatpak_package — non-privileged --user Flatpak install (spec section 2)"
```

---

## Task 2: Snap install support (new pkexec action)

**Files:**
- Modify: `src-tauri/src/packages/install.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/packaging/nitrux-pkexec-helper`
- Modify: `src-tauri/packaging/org.heiphaistos.nitrux.packages.policy`
- Modify: `src-tauri/tauri.conf.json`

- [ ] **Step 1: Write the failing Rust tests**

Add to `src-tauri/src/packages/install.rs`, right after `install_package`:

```rust
/// Installs a Snap package, escalating through polkit. Snapd always
/// requires root for `snap install` (unlike Flatpak, there is no
/// unprivileged `--user` equivalent) -- see `install_flatpak_package`'s
/// doc comment for the contrast. Mirrors `install_package` exactly (same
/// package-name validation, same generous timeout), but uses its own
/// dedicated `nitrux-pkexec-install-snap` exec path rather than sharing
/// `install-package`'s -- pkexec resolves the action purely by exec path,
/// with no visibility into argv, so a shared path across actions with
/// different argument shapes (this one has no `manager` argument) would be
/// ambiguous to it.
#[tauri::command]
pub fn install_snap_package(package: String) -> Result<String, String> {
    validate_package_name(&package)?;
    subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-install-snap", "install-snap", &package],
        Duration::from_secs(300),
    )
}
```

And a new test in the `#[cfg(test)] mod tests` block:

```rust
#[test]
fn install_snap_package_rejects_malicious_package_name_before_ever_shelling_out() {
    let result = install_snap_package("curl; rm -rf /".to_string());
    assert!(result.is_err());
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (function not registered yet — this test alone will actually compile and pass once the function exists, since it never reaches pkexec; run it after Step 1 lands to confirm no accidental typo).

- [ ] **Step 3: Register the command in `lib.rs`**

Add `packages::install::install_snap_package,` right after `packages::install::upgrade_all_packages,` (find the existing entry — search for `upgrade_all_packages` in the `invoke_handler!` list).

- [ ] **Step 4: Run the full Rust suite, expect `188 passed; 0 failed; 1 ignored`** (187 from Task 1 + 1 new).

- [ ] **Step 5: Add the `install-snap` subcommand to `nitrux-pkexec-helper`**

In `src-tauri/packaging/nitrux-pkexec-helper`, add a new `case` arm right after `install-package)` and before `uninstall-package)`:

```sh
  install-snap)
    package="${2:-}"
    validate_package_name "$package"
    exec snap install "$package"
    ;;
```

Also update the script's own header comment listing the 12 installed names — it becomes 13:

```
# This same script is installed under 13 distinct names (one per polkit
# action): nitrux-pkexec-install-package, nitrux-pkexec-uninstall-package,
# nitrux-pkexec-upgrade-all, nitrux-pkexec-install-snap,
# nitrux-pkexec-write-hosts, ...
```
(keep the rest of that list identical, just insert `nitrux-pkexec-install-snap` and update "12" to "13").

Also add the usage line alongside the others near the top:
```
#   nitrux-pkexec-install-snap install-snap <package>
```

- [ ] **Step 6: Add the 4th `<action>` to `org.heiphaistos.nitrux.packages.policy`**

Append inside `<policyconfig>`, after the existing `upgrade-all` action and before the closing `</policyconfig>`:

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

- [ ] **Step 7: Add the file mapping to `tauri.conf.json`**

In both `bundle.linux.deb.files` and `bundle.linux.rpm.files`, add a new entry right after the existing `/usr/bin/nitrux-pkexec-install-package` line:

```json
          "/usr/bin/nitrux-pkexec-install-snap": "packaging/nitrux-pkexec-helper",
```

- [ ] **Step 8: Commit**

```bash
git add src-tauri/src/packages/install.rs src-tauri/src/lib.rs src-tauri/packaging/nitrux-pkexec-helper src-tauri/packaging/org.heiphaistos.nitrux.packages.policy src-tauri/tauri.conf.json
git commit -m "feat: add install_snap_package — new pkexec action for Snap installs (spec section 3)"
```

- [ ] **Step 9: Build and verify the new pkexec action live on the VM before merging**

Build: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r10-logiciels-deploiement && npx tauri build 2>&1 | tail -30"`.

On the VM (`172.18.32.124`, user `dev`, password `1998`): install `snapd` first if absent (confirmed absent during this plan's research — `dpkg -l snapd` showed `un`/not installed, though the package exists in apt's cache):
```
echo 1998 | sudo -S apt-get install -y snapd
echo 1998 | sudo -S systemctl enable --now snapd
echo 1998 | sudo -S systemctl enable --now snapd.socket
```
Wait for `snap.socket`/`snapd.service` to be active (`systemctl is-active snapd`) before testing — snapd needs a moment after first install.

Deploy the freshly built `.deb`, install it, then run the pkexec cycle via `ssh_interactive.py` (mirrors the R7 uninstall-package verification pattern exactly):
```
pkttyagent --process $$ & sleep 1; pkexec /usr/bin/nitrux-pkexec-install-snap install-snap hello
```
(`hello` is snapd's own canonical tiny tutorial package — safe, ~run in seconds, matches the spirit of R7's throwaway `sl` apt package.)

Confirm:
1. The line `==== AUTHENTICATING FOR org.heiphaistos.nitrux.install-snap ====` appears (proves polkit resolved the new, distinct action — not `install-package` or any other).
2. `snap list` afterward shows `hello` installed.
3. Clean up: `echo 1998 | sudo -S snap remove hello` (not part of this feature's scope, just VM hygiene).

If `pkttyagent`/`ssh_interactive.py` friction reproduces past issues, fall back to the documented pattern from the checkpoint file (`set +m` for job control in a non-interactive SSH shell).

---

## Task 3: Wire `QuickInstallPage.vue` to the real install paths

**Files:**
- Modify: `src/pages/QuickInstallPage.vue`
- Modify: `src/pages/QuickInstallPage.spec.ts`

- [ ] **Step 1: Read the live files first** (reproduced from this plan's research above — `QuickInstallPage.vue`'s `install()` currently has `if (entry.installMethod !== "apt") return;`, and the template has a `v-if="entry.installMethod !== 'apt'"` branch rendering `NxBadge status="info">Bientôt disponible ({{ entry.installMethod }})</NxBadge>` + a permanently-disabled button).

- [ ] **Step 2: Replace the existing "disables the install button..." test** in `QuickInstallPage.spec.ts` with real flatpak/snap dispatch tests. Update the top-level mock and replace that one test:

```typescript
// src/pages/QuickInstallPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import QuickInstallPage from "./QuickInstallPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "detect_native_manager") return Promise.resolve("apt");
    if (cmd === "install_package") {
      if (args?.package === "fail-me") return Promise.reject("apt: paquet introuvable");
      return Promise.resolve("Installation réussie");
    }
    if (cmd === "install_flatpak_package") return Promise.resolve("Installation Flatpak réussie");
    if (cmd === "install_snap_package") return Promise.resolve("Installation Snap réussie");
    return Promise.resolve(null);
  }),
}));

describe("QuickInstallPage", () => {
  beforeEach(() => vi.clearAllMocks());

  it("detects the native manager on mount and renders the catalog", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    expect(invoke).toHaveBeenCalledWith("detect_native_manager");
  });

  it("installs an apt-method app via install_package using the detected manager", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    const buttons = wrapper.findAll("button");
    const firefoxButton = buttons.find((b) => b.text() === "Installer" && b.element.closest(".qi-card")?.textContent?.includes("Firefox"))!;
    await firefoxButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "firefox" });
  });

  it("installs a flatpak-method app via install_flatpak_package", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Discord"));
    const discordCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("Discord"))!;
    const button = discordCard.find("button")!;
    expect(button.attributes("disabled")).toBeUndefined();
    await button.trigger("click");
    await vi.waitFor(() => expect(discordCard.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_flatpak_package", { appId: "com.discordapp.Discord" });
  });

  it("installs a snap-method app via install_snap_package", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Spotify"));
    const spotifyCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("Spotify"))!;
    const button = spotifyCard.find("button")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(spotifyCard.text()).toContain("Installé"));
    expect(invoke).toHaveBeenCalledWith("install_snap_package", { package: "spotify" });
  });

  it("shows an error message when install_package rejects", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string, args?: Record<string, unknown>) => {
      if (cmd === "detect_native_manager") return Promise.resolve("apt");
      if (cmd === "install_package" && args?.package === "gimp") return Promise.reject("apt: échec de l'installation");
      return Promise.resolve("ok");
    });
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("GIMP"));
    const gimpCard = wrapper.findAll(".qi-card").find((c) => c.text().includes("GIMP"))!;
    await gimpCard.find("button").trigger("click");
    await vi.waitFor(() => expect(gimpCard.text()).toContain("apt: échec de l'installation"));
  });

  it("filters the catalog by category", async () => {
    const wrapper = mount(QuickInstallPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Firefox"));
    const chips = wrapper.findAll(".qi-chip");
    const jeuxChip = chips.find((c) => c.text() === "Jeux")!;
    await jeuxChip.trigger("click");
    expect(wrapper.text()).toContain("Steam");
    expect(wrapper.text()).not.toContain("Firefox");
  });
});
```

- [ ] **Step 3: Run it, confirm the new flatpak/snap tests FAIL** (component still has the old placeholder branch).

- [ ] **Step 4: Rewrite `install()` and the template's method branch in `QuickInstallPage.vue`**

Replace the `install()` function:

```typescript
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

Replace the template's method-branch block (the `<template v-if="entry.installMethod !== 'apt'">...</template>` and the following `<template v-else-if="stateOf(entry) === 'success'">`/`<template v-else>` siblings) with:

```vue
        <template v-if="stateOf(entry) === 'success'">
          <NxBadge status="success">Installé</NxBadge>
        </template>
        <template v-else>
          <div v-if="stateOf(entry) === 'installing'" class="qi-progress"><div class="qi-progress-bar"></div></div>
          <NxCard v-if="stateOf(entry) === 'error'" danger class="qi-error">{{ installErrors[entry.id] }}</NxCard>
          <NxButton :disabled="stateOf(entry) === 'installing'" @click="install(entry)">
            {{ stateOf(entry) === "installing" ? "Installation..." : "Installer" }}
          </NxButton>
        </template>
```

`NxBadge` import stays (still used for the "Installé" success state) — do not remove it.

- [ ] **Step 5: Run it, confirm all 6 tests PASS**

- [ ] **Step 6: Commit**

```bash
git add src/pages/QuickInstallPage.vue src/pages/QuickInstallPage.spec.ts
git commit -m "feat: wire QuickInstallPage to real Flatpak/Snap installs, remove Bientôt disponible placeholder (spec section 4)"
```

---

## Task 4: `InstallProfilesPage.vue` (bulk profile install)

**Files:**
- Create: `src/data/installProfiles.ts`
- Create: `src/pages/InstallProfilesPage.vue`
- Test: `src/pages/InstallProfilesPage.spec.ts`
- Modify: `src/navigation/categories.ts`
- Modify: `src/navigation/categories.spec.ts`
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`

`appCatalog.ts`'s real entries (confirmed by reading the live file during this plan's research): `firefox`, `chromium` (Navigateurs); `thunderbird`, `discord` (Communication); `libreoffice` (Bureautique); `gimp`, `inkscape`, `blender`, `vlc`, `audacity`, `obs-studio`, `spotify` (Média); `steam` (Jeux); `keepassxc`, `htop` (Utilitaires); `git` (Développement).

- [ ] **Step 1: Write `src/data/installProfiles.ts`**

```typescript
// src/data/installProfiles.ts
export interface InstallProfile {
  id: string;
  label: string;
  description: string;
  /** Ids referencing entries in appCatalog.ts -- never duplicated app data. */
  appIds: string[];
}

export const installProfiles: InstallProfile[] = [
  {
    id: "essentiels",
    label: "Essentiels",
    description: "Navigateur, bureautique, lecteur multimédia et mot de passe.",
    appIds: ["firefox", "libreoffice", "vlc", "keepassxc"],
  },
  {
    id: "developpement",
    label: "Développement",
    description: "Outils de base pour coder et gérer des versions.",
    appIds: ["git", "htop"],
  },
  {
    id: "creation",
    label: "Création & Média",
    description: "Édition d'image, audio, vidéo et création 3D.",
    appIds: ["gimp", "inkscape", "audacity", "obs-studio", "blender"],
  },
  {
    id: "communication",
    label: "Communication",
    description: "Messagerie et discussion.",
    appIds: ["thunderbird", "discord"],
  },
];
```

- [ ] **Step 2: Write the failing frontend test**

```typescript
// src/pages/InstallProfilesPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import InstallProfilesPage from "./InstallProfilesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string, args?: Record<string, unknown>) => {
    if (cmd === "detect_native_manager") return Promise.resolve("apt");
    if (cmd === "install_package") {
      if (args?.package === "libreoffice") return Promise.reject("apt: échec");
      return Promise.resolve("ok");
    }
    return Promise.resolve("ok");
  }),
}));

describe("InstallProfilesPage", () => {
  it("lists every profile with its app count", () => {
    const wrapper = mount(InstallProfilesPage);
    expect(wrapper.text()).toContain("Essentiels");
    expect(wrapper.text()).toContain("Développement");
  });

  it("selecting a profile checks all its apps", async () => {
    const wrapper = mount(InstallProfilesPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Essentiels"))!;
    await button.trigger("click");
    const checkboxes = wrapper.findAll("input[type=checkbox]:checked");
    // essentiels has 4 apps
    expect(checkboxes.length).toBe(4);
  });

  it("installs every checked app sequentially and reports a per-app summary, including failures", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(InstallProfilesPage);
    const profileButton = wrapper.findAll("button").find((b) => b.text().includes("Essentiels"))!;
    await profileButton.trigger("click");
    const installButton = wrapper.findAll("button").find((b) => b.text() === "Installer la sélection")!;
    await installButton.trigger("click");
    // Waiting on "Firefox" here would be a trap: it's already in the
    // always-rendered checkbox list from the very first render, so that
    // wait would resolve immediately and NOT actually wait for the
    // sequential install loop to finish -- confirmed by tracing this by
    // hand before writing it (installSelection awaits each installOne in
    // turn; trigger("click") only awaits Vue's nextTick, not the full
    // async handler). "échec" only appears once the libreoffice result
    // (the 2nd of 4, which fails per the mock above) has actually landed
    // in `results`, which is a real completion signal.
    await vi.waitFor(() => expect(wrapper.text()).toContain("échec"));
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "firefox" });
    expect(invoke).toHaveBeenCalledWith("install_package", { manager: "apt", package: "libreoffice" });
  });
});
```

- [ ] **Step 3: Run it, confirm it FAILS**

- [ ] **Step 4: Write `InstallProfilesPage.vue`**

Trace both later test assertions against this template by hand before writing it (per the discipline established since R8 Task 1, and reused successfully throughout R9).

```vue
<!-- src/pages/InstallProfilesPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { appCatalog, type AppCatalogEntry } from "@/data/appCatalog";
import { installProfiles } from "@/data/installProfiles";

const nativeManager = ref<string | null>(null);
let managerReady: Promise<string | null> | null = null;
onMounted(() => {
  managerReady = invoke<string | null>("detect_native_manager").then((result) => {
    nativeManager.value = result;
    return result;
  });
});

const catalogById = new Map<string, AppCatalogEntry>(appCatalog.map((e) => [e.id, e]));

const selected = ref<Set<string>>(new Set());

function toggle(id: string) {
  const next = new Set(selected.value);
  if (next.has(id)) next.delete(id);
  else next.add(id);
  selected.value = next;
}

function selectProfile(appIds: string[]) {
  selected.value = new Set(appIds);
}

const selectedEntries = computed<AppCatalogEntry[]>(() =>
  [...selected.value].map((id) => catalogById.get(id)).filter((e): e is AppCatalogEntry => !!e),
);

interface InstallResult { id: string; name: string; success: boolean; message: string }

const installing = ref(false);
const results = ref<InstallResult[] | null>(null);

async function installOne(entry: AppCatalogEntry): Promise<InstallResult> {
  try {
    if (entry.installMethod === "apt") {
      const manager = nativeManager.value ?? (await managerReady);
      if (!manager) throw new Error("aucun gestionnaire de paquets natif détecté");
      const message = await invoke<string>("install_package", { manager, package: entry.packageId });
      return { id: entry.id, name: entry.name, success: true, message };
    }
    if (entry.installMethod === "flatpak") {
      const message = await invoke<string>("install_flatpak_package", { appId: entry.packageId });
      return { id: entry.id, name: entry.name, success: true, message };
    }
    const message = await invoke<string>("install_snap_package", { package: entry.packageId });
    return { id: entry.id, name: entry.name, success: true, message };
  } catch (e) {
    return { id: entry.id, name: entry.name, success: false, message: String(e) };
  }
}

async function installSelection() {
  installing.value = true;
  results.value = [];
  // Sequential, not parallel -- avoids concurrent lock contention on the
  // native package manager (apt/dnf/etc hold an exclusive lock; parallel
  // installs would just serialize at the OS level anyway, but with
  // confusing partial-failure errors instead of a clean queue).
  for (const entry of selectedEntries.value) {
    const result = await installOne(entry);
    results.value = [...results.value, result];
  }
  installing.value = false;
}
</script>

<template>
  <div class="ip-page">
    <NxSectionHeader
      title="Installation par profils"
      description="Sélectionnez un profil ou des applications individuelles, puis installez tout en une fois."
    />

    <div class="ip-profiles">
      <button
        v-for="profile in installProfiles"
        :key="profile.id"
        class="ip-profile-button"
        @click="selectProfile(profile.appIds)"
      >
        <strong>{{ profile.label }}</strong>
        <p>{{ profile.description }}</p>
        <span class="ip-profile-count">{{ profile.appIds.length }} applications</span>
      </button>
    </div>

    <NxCard>
      <NxSectionHeader title="Sélection" />
      <label v-for="entry in appCatalog" :key="entry.id" class="ip-check-row">
        <input type="checkbox" :checked="selected.has(entry.id)" @change="toggle(entry.id)" />
        <span>{{ entry.icon }} {{ entry.name }}</span>
      </label>
      <NxButton :disabled="selected.size === 0 || installing" @click="installSelection">
        {{ installing ? "Installation..." : "Installer la sélection" }}
      </NxButton>
    </NxCard>

    <NxCard v-if="results && results.length > 0">
      <NxSectionHeader title="Résultat" />
      <div v-for="r in results" :key="r.id" class="ip-result-row">
        <span>{{ r.name }}</span>
        <NxBadge :status="r.success ? 'success' : 'danger'">{{ r.success ? 'installé' : 'échec' }}</NxBadge>
        <span v-if="!r.success" class="ip-result-message">{{ r.message }}</span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.ip-page { padding: 24px; display: flex; flex-direction: column; gap: 16px; }
.ip-profiles { display: grid; grid-template-columns: repeat(auto-fill, minmax(220px, 1fr)); gap: 12px; }
.ip-profile-button { text-align: left; padding: 14px; border-radius: 10px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); cursor: pointer; font: inherit; }
.ip-profile-button p { margin: 6px 0; font-size: 12px; color: var(--nx-text-secondary); }
.ip-profile-count { font-size: 11px; color: var(--nx-text-secondary); }
.ip-check-row { display: flex; align-items: center; gap: 8px; padding: 4px 0; font-size: 13px; }
.ip-result-row { display: flex; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; }
.ip-result-message { color: var(--nx-text-secondary); font-size: 12px; }
</style>
```

- [ ] **Step 5: Run it, confirm it PASSES (3 tests)**

- [ ] **Step 6: Add the nav entry.** In `src/navigation/categories.ts`, the `"applications"` category currently has `quick-install` and `package-manager`. Add a 3rd entry right after `package-manager`:

```typescript
      { id: "install-profiles", label: "Installation par profils", icon: "layers" },
```

- [ ] **Step 7: Add the `layers` icon to `AppNav.vue`'s `iconMap`** — new import from `lucide-vue-next`:

```typescript
  Layers,
```
(add to the existing import list) and:
```typescript
  layers: Layers,
```
(add to `iconMap`).

- [ ] **Step 8: Update `categories.spec.ts`**

```typescript
  it("includes the new Phase R10 Logiciels & déploiement page by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("install-profiles");
  });
```

- [ ] **Step 9: Wire `App.vue`** — add the import and page-map entry:

```typescript
import InstallProfilesPage from "@/pages/InstallProfilesPage.vue";
```
```typescript
  "install-profiles": InstallProfilesPage,
```

- [ ] **Step 10: Add to `App.spec.ts`**

```typescript
  it("shows the real InstallProfilesPage for the install-profiles id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Installation par profils")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Sélectionnez un profil");
  });
```

- [ ] **Step 11: Run `npx vitest run src/pages/InstallProfilesPage.spec.ts src/navigation/categories.spec.ts src/App.spec.ts`, confirm all pass**

- [ ] **Step 12: Commit**

```bash
git add src/data/installProfiles.ts src/pages/InstallProfilesPage.vue src/pages/InstallProfilesPage.spec.ts src/navigation/categories.ts src/navigation/categories.spec.ts src/components/nav/AppNav.vue src/App.vue src/App.spec.ts
git commit -m "feat: add InstallProfilesPage — curated bulk-install profiles over the existing app catalog (spec section 5)"
```

---

## Task 5: Bonus — wire `get_smart_status` into `DisksPage.vue`

**Files:**
- Modify: `src/pages/DisksPage.vue`
- Modify: `src/pages/DisksPage.spec.ts`

`get_smart_status(device: String) -> Result<SmartStatus, String>` (`SmartStatus { device: String, health: Option<String> }`) has existed since R9, is registered in `lib.rs`, and is fully unit-tested — but has zero frontend consumers. `smart.rs`'s own doc comment already documents that `smartctl` commonly needs root and a permission error is expected, not a bug, on most systems — the UI must show that gracefully, not treat it as a crash.

- [ ] **Step 1: Write the failing test**

Add to `src/pages/DisksPage.spec.ts`:

```typescript
  it("checks SMART health for a disk and shows the result", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_disks") return Promise.resolve([{ name: "sda", size: "500G", partitions: [] }]);
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      if (cmd === "get_smart_status") return Promise.resolve({ device: "/dev/sda", health: "PASSED" });
      return Promise.resolve(null);
    });
    const wrapper = mount(DisksPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sda"));
    const button = wrapper.findAll("button").find((b) => b.text() === "Vérifier la santé")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("PASSED"));
    expect(invoke).toHaveBeenCalledWith("get_smart_status", { device: "/dev/sda" });
  });

  it("shows a clear message when SMART is unavailable (e.g. no root)", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "list_disks") return Promise.resolve([{ name: "sda", size: "500G", partitions: [] }]);
      if (cmd === "list_disk_usage") return Promise.resolve([]);
      if (cmd === "get_smart_status") return Promise.reject("smartctl: Permission denied");
      return Promise.resolve(null);
    });
    const wrapper = mount(DisksPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("sda"));
    const button = wrapper.findAll("button").find((b) => b.text() === "Vérifier la santé")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("Permission denied"));
  });
```

- [ ] **Step 2: Run it, confirm it FAILS**

- [ ] **Step 3: Add SMART state and handler to `DisksPage.vue`**

Add near the other refs (after `disksError`):

```typescript
interface SmartStatus { device: string; health: string | null }

const smartStatus = ref<Record<string, SmartStatus>>({});
const smartError = ref<Record<string, string>>({});
const smartBusy = ref<string | null>(null);

async function checkSmart(diskName: string) {
  const device = `/dev/${diskName}`;
  smartBusy.value = diskName;
  delete smartError.value[diskName];
  try {
    smartStatus.value = { ...smartStatus.value, [diskName]: await invoke<SmartStatus>("get_smart_status", { device }) };
  } catch (e) {
    smartError.value = { ...smartError.value, [diskName]: String(e) };
  } finally {
    smartBusy.value = null;
  }
}
```

Add inside the existing disk card, right after the `<ul>` of partitions (still inside the same `NxCard v-for="disk in disks"`):

```vue
      <div class="disks-smart-row">
        <NxButton :disabled="smartBusy === disk.name" @click="checkSmart(disk.name)">
          {{ smartBusy === disk.name ? "Vérification..." : "Vérifier la santé" }}
        </NxButton>
        <NxBadge v-if="smartStatus[disk.name]" :status="smartStatus[disk.name].health === 'PASSED' ? 'success' : 'danger'">
          {{ smartStatus[disk.name].health ?? "inconnu" }}
        </NxBadge>
        <span v-if="smartError[disk.name]" class="disks-smart-error">{{ smartError[disk.name] }}</span>
      </div>
```

Add to the `<style scoped>` block:

```css
.disks-smart-row { display: flex; align-items: center; gap: 10px; margin-top: 8px; font-size: 12px; }
.disks-smart-error { color: var(--nx-text-secondary); }
```

- [ ] **Step 4: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 5: Commit**

```bash
git add src/pages/DisksPage.vue src/pages/DisksPage.spec.ts
git commit -m "feat: wire the orphaned get_smart_status command into DisksPage (spec section 6)"
```

---

## Task 6: Full verification pass — frontend, backend, and live VM check

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite.**

Expected baseline entering this plan: 190 (end of R9). Net delta: Task 3 (net +1: `QuickInstallPage.spec.ts` goes from 5 tests to 6 — the placeholder test is removed, two new ones for flatpak/snap are added) + Task 4 (+3 `InstallProfilesPage.spec.ts` + 1 `categories.spec.ts` + 1 `App.spec.ts` = +5) + Task 5 (+2) = +8, expected total **198**. Report the real observed total and reconcile if it differs — do not trust this arithmetic blindly, verify it (this exact kind of test-count arithmetic has been wrong in a prior phase's plan text before, per R7's own note).

- [ ] **Step 2: Type-check.** `npx vue-tsc --noEmit`, expect clean.

- [ ] **Step 3: Confirm the Rust suite.** Expect `188 passed; 0 failed; 1 ignored` (184 R9 baseline + 3 Task 1 + 1 Task 2).

- [ ] **Step 4: Build and install on the VM, verify Flatpak install for real (Snap was already verified live in Task 2's Step 9).**

Build: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r10-logiciels-deploiement && npx tauri build 2>&1 | tail -30"`.

On the VM: install flatpak first if still absent (confirmed absent during this plan's research):
```
echo 1998 | sudo -S apt-get install -y flatpak
```
Deploy + install the freshly built `.deb`. Since `install_flatpak_package` is non-privileged (no pkexec involved, runs as the invoking user), it cannot be exercised via the `pkttyagent`/`pkexec` SSH pattern used for privileged actions — either launch the actual app on the VM's desktop session and click through Quick Install → Discord, or reason through it directly over SSH as the `dev` user (no `sudo`, matching exactly what the app itself would do):
```
flatpak remote-add --if-not-exists --user flathub https://flathub.org/repo/flathub.flatpakrepo
flatpak install --user --noninteractive flathub com.discordapp.Discord
```
Confirm it actually installs (`flatpak list --user` shows the app) or fails with a clear, real error (e.g. a network restriction in this VM's environment — either outcome is informative; the goal is confirming the command sequence itself is correct against a real system, not guaranteeing network access exists in this sandbox). Clean up afterward: `flatpak uninstall --user -y com.discordapp.Discord` if the install succeeded.

- [ ] **Step 5: Commit any final cleanup.** No further commit expected if Steps 1-4 all pass clean.

---

## Self-Review

**Spec coverage:** §2 (Flatpak) — Task 1. §3 (Snap) — Task 2. §4 (QuickInstallPage wiring) — Task 3. §5 (Install Profiles) — Task 4. §6 (SMART bonus) — Task 5. §7 (out of scope: tools/os-downloads, Snap `--classic`, dry-run/export, Flatpak/Snap uninstall) — confirmed no task in this plan touches any of those.

**Placeholder scan:** No "TBD"/"TODO". `installProfiles.ts`'s 4 profiles reference only ids confirmed present in the live `appCatalog.ts` during this plan's research (`firefox`, `libreoffice`, `vlc`, `keepassxc`, `git`, `htop`, `gimp`, `inkscape`, `audacity`, `obs-studio`, `blender`, `thunderbird`, `discord` — 13 of 16 catalog entries used across the 4 profiles, `chromium`/`spotify`/`steam` intentionally left out of every profile as individually-selectable-only, not a gap).

**Type consistency:** `SmartStatus` (Task 5) matches `smart.rs`'s existing Rust struct field-for-field (`device: String`, `health: Option<String>` → `health: string | null`). `InstallResult` (Task 4) is purely local to the new page, no cross-file consistency requirement. Every template in this plan has been hand-traced against its own test assertions before being finalized, per the discipline established in R8 and reused successfully through R9 (where it caught a real bug in Task 2's plan text). This self-review's trace of Task 4's third test caught the same class of bug directly in this plan: `waitFor(() => text.toContain("Firefox"))` would have resolved immediately (that name is already in the always-rendered checkbox list) instead of actually waiting for the sequential install loop, letting the assertions after it run against incomplete state — fixed inline to wait on "échec" instead, which only appears once the loop has genuinely progressed past the failing entry.
