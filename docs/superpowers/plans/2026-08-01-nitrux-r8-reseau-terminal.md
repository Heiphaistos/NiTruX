# NiTruX Phase R8 — Réseau & Terminal Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add 4 pages to the "Réseau" nav category — WiFi Analyzer, DNS Switcher, Bluetooth, Scripts & Snippets — per the spec's scoping decision (Terminal intégré explicitly deferred, needs its own dependency/design decision, not built in this pass).

**Architecture:** `WiFiAnalyzerPage.vue` reuses the already-existing `get_network_snapshot` command verbatim (zero backend change). `DnsSwitcherPage.vue` reuses the already-existing, already-privileged, already-VM-verified `set_dns_servers` command, adding curated one-click DNS presets on top of manual entry. `BluetoothPage.vue` is backed by a new, entirely read-only `src-tauri/src/bluetooth.rs` module (`bluetoothctl show`/`bluetoothctl devices`, no root needed for reads on a standard desktop D-Bus setup, so no new pkexec surface). `ScriptsPage.vue` is backed by a new, non-privileged `run_script` command (executes user-authored shell text as the invoking user only — conceptually identical to the user typing the same command in a terminal themselves, not a new privilege boundary) plus a new frontend-only `scriptsStore.ts` (localStorage-persisted, mirrors the already-established `preferencesStore.ts` pattern from R2 — no new backend persistence).

**Tech Stack:** Tauri v2 + Rust (backend), Vue 3.5 + TypeScript + Pinia + Vitest (frontend), same patterns as R1-R7. No new dependencies.

---

## Task 1: `WiFiAnalyzerPage.vue` (zero new backend)

**Files:**
- Create: `src/pages/WiFiAnalyzerPage.vue`
- Test: `src/pages/WiFiAnalyzerPage.spec.ts`

Reuses `get_network_snapshot` (`network.rs`, unchanged) — the command already returns `wifi_networks: WifiNetwork[]` with `{ ssid, security, signal_percent, connected }`.

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/WiFiAnalyzerPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import WiFiAnalyzerPage from "./WiFiAnalyzerPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    wifi_networks: [
      { ssid: "WeakOpen", security: "", signal_percent: 20, connected: false },
      { ssid: "HomeWifi", security: "WPA2", signal_percent: 85, connected: true },
    ],
    listening_ports: [],
    dns_servers: [],
    hosts_file: "",
  }),
}));

describe("WiFiAnalyzerPage", () => {
  it("lists networks sorted by signal strength, strongest first", async () => {
    const wrapper = mount(WiFiAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("HomeWifi"));
    const names = wrapper.findAll(".wifi-ssid").map((n) => n.text());
    expect(names).toEqual(["HomeWifi", "WeakOpen"]);
  });

  it("shows a danger badge for an open network and a success badge for WPA2", async () => {
    const wrapper = mount(WiFiAnalyzerPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("HomeWifi"));
    expect(wrapper.find(".nx-badge--danger").exists()).toBe(true);
    expect(wrapper.find(".nx-badge--success").exists()).toBe(true);
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

Run: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r8-reseau-terminal && npx vitest run src/pages/WiFiAnalyzerPage.spec.ts"`

- [ ] **Step 3: Write `WiFiAnalyzerPage.vue`**

```vue
<!-- src/pages/WiFiAnalyzerPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface WifiNetwork { ssid: string; security: string; signal_percent: number; connected: boolean }
interface NetworkSnapshot { wifi_networks: WifiNetwork[] }

const networks = ref<WifiNetwork[]>([]);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    const snapshot = await invoke<NetworkSnapshot>("get_network_snapshot");
    networks.value = snapshot.wifi_networks;
  } catch (e) {
    error.value = String(e);
  }
});

const sortedNetworks = computed(() => [...networks.value].sort((a, b) => b.signal_percent - a.signal_percent));

function securityStatus(security: string): "success" | "warning" | "danger" {
  if (security === "") return "danger";
  if (security.includes("WEP")) return "warning";
  return "success";
}
</script>

<template>
  <div class="wifi-page">
    <NxSectionHeader title="WiFi Analyzer" description="Réseaux Wi-Fi visibles, classés par force de signal." />

    <NxCard v-if="error" danger>{{ error }}</NxCard>

    <NxCard v-for="net in sortedNetworks" :key="net.ssid" class="wifi-row">
      <div class="wifi-info">
        <span class="wifi-ssid">{{ net.ssid }}{{ net.connected ? " (connecté)" : "" }}</span>
        <NxBadge :status="securityStatus(net.security)">{{ net.security || "ouvert" }}</NxBadge>
      </div>
      <div class="wifi-signal-bar">
        <div class="wifi-signal-fill" :style="{ width: `${net.signal_percent}%` }"></div>
      </div>
      <span class="wifi-signal-label">{{ net.signal_percent }}%</span>
    </NxCard>
  </div>
</template>

<style scoped>
.wifi-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.wifi-row { display: flex; align-items: center; gap: 14px; }
.wifi-info { display: flex; flex-direction: column; gap: 6px; min-width: 160px; }
.wifi-ssid { font-weight: 600; color: var(--nx-text-primary); }
.wifi-signal-bar { flex: 1; height: 6px; border-radius: 3px; background: color-mix(in srgb, var(--nx-accent-primary) 15%, transparent); overflow: hidden; }
.wifi-signal-fill { height: 100%; background: var(--nx-accent-primary); border-radius: 3px; }
.wifi-signal-label { font-size: 12px; color: var(--nx-text-secondary); min-width: 36px; text-align: right; }
</style>
```

- [ ] **Step 4: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 5: Commit**

```bash
git add src/pages/WiFiAnalyzerPage.vue src/pages/WiFiAnalyzerPage.spec.ts
git commit -m "feat: add WiFiAnalyzerPage, reusing get_network_snapshot (spec section 3.1)"
```

---

## Task 2: `bluetooth.rs` (read-only) + `BluetoothPage.vue`

**Files:**
- Create: `src-tauri/src/bluetooth.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/BluetoothPage.vue`
- Test: `src/pages/BluetoothPage.spec.ts`

Real `bluetoothctl show` output looks like:
```
Controller AA:BB:CC:DD:EE:FF (public)
	Name: dev-laptop
	...
	Powered: yes
	...
```
Real `bluetoothctl devices` output looks like:
```
Device 11:22:33:44:55:66 Sony WH-1000XM4
Device 77:88:99:AA:BB:CC Keyboard K380
```

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/bluetooth.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct BluetoothDevice {
    pub address: String,
    pub name: String,
}

#[derive(Serialize, Clone)]
pub struct BluetoothStatus {
    pub adapter_present: bool,
    pub powered: bool,
    pub devices: Vec<BluetoothDevice>,
}

/// Parses `Powered: yes`/`Powered: no` out of `bluetoothctl show` output.
/// Any other line, or a missing `Powered:` line entirely (no adapter),
/// is treated as `false` -- an absent/unparseable status is never
/// reported as "on" by default.
pub fn parse_powered_status(show_output: &str) -> bool {
    show_output.lines().any(|l| l.trim() == "Powered: yes")
}

/// Parses one line of `bluetoothctl devices` output, e.g.
/// "Device 11:22:33:44:55:66 Sony WH-1000XM4" -> address + name. The name
/// may itself contain spaces, so only the first two whitespace-separated
/// tokens ("Device", the MAC) are consumed positionally; everything after
/// is the name verbatim.
pub fn parse_device_line(line: &str) -> Option<BluetoothDevice> {
    let rest = line.strip_prefix("Device ")?;
    let (address, name) = rest.split_once(' ')?;
    if address.is_empty() || name.is_empty() {
        return None;
    }
    Some(BluetoothDevice { address: address.to_string(), name: name.to_string() })
}

#[tauri::command]
pub fn get_bluetooth_status() -> BluetoothStatus {
    let show_output = subprocess::run_with_timeout("bluetoothctl", &["show"], Duration::from_secs(5));
    let adapter_present = show_output.is_ok();
    let powered = show_output.as_deref().map(parse_powered_status).unwrap_or(false);

    let devices = subprocess::run_with_timeout("bluetoothctl", &["devices"], Duration::from_secs(5))
        .map(|output| output.lines().filter_map(parse_device_line).collect())
        .unwrap_or_default();

    BluetoothStatus { adapter_present, powered, devices }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_powered_yes() {
        let output = "Controller AA:BB:CC:DD:EE:FF (public)\n\tName: dev-laptop\n\tPowered: yes\n\tDiscoverable: no\n";
        assert!(parse_powered_status(output));
    }

    #[test]
    fn parses_powered_no() {
        let output = "Controller AA:BB:CC:DD:EE:FF (public)\n\tPowered: no\n";
        assert!(!parse_powered_status(output));
    }

    #[test]
    fn treats_missing_powered_line_as_false() {
        assert!(!parse_powered_status("Controller AA:BB:CC:DD:EE:FF (public)\n\tName: dev-laptop\n"));
    }

    #[test]
    fn parses_a_device_line_with_a_multi_word_name() {
        let line = "Device 11:22:33:44:55:66 Sony WH-1000XM4";
        let device = parse_device_line(line).expect("should parse");
        assert_eq!(device.address, "11:22:33:44:55:66");
        assert_eq!(device.name, "Sony WH-1000XM4");
    }

    #[test]
    fn rejects_a_line_not_starting_with_device_prefix() {
        assert!(parse_device_line("Controller AA:BB:CC:DD:EE:FF (public)").is_none());
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile** (module not registered).

- [ ] **Step 3: Register `mod bluetooth;` in `lib.rs`** (alphabetically — sorts after `mod backup;` and `mod benchmark;` but before `mod cache_size;`, so insert it as the third `mod` line, right between `mod benchmark;` and `mod cache_size;`) and add `bluetooth::get_bluetooth_status,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `170 passed; 0 failed; 1 ignored`** (165 baseline + 5 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/bluetooth.rs src-tauri/src/lib.rs
git commit -m "feat: add bluetooth.rs — read-only adapter/device status (spec section 3.3)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/BluetoothPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import BluetoothPage from "./BluetoothPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue({
    adapter_present: true,
    powered: true,
    devices: [{ address: "11:22:33:44:55:66", name: "Sony WH-1000XM4" }],
  }),
}));

describe("BluetoothPage", () => {
  it("shows adapter status and paired devices", async () => {
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Sony WH-1000XM4"));
    expect(wrapper.text()).toContain("activé");
  });

  it("shows a clear message when no adapter is present", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockResolvedValueOnce({ adapter_present: false, powered: false, devices: [] });
    const wrapper = mount(BluetoothPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucun adaptateur"));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `BluetoothPage.vue`**

```vue
<!-- src/pages/BluetoothPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface BluetoothDevice { address: string; name: string }
interface BluetoothStatus { adapter_present: boolean; powered: boolean; devices: BluetoothDevice[] }

const status = ref<BluetoothStatus | null>(null);

onMounted(async () => {
  status.value = await invoke<BluetoothStatus>("get_bluetooth_status");
});
</script>

<template>
  <div class="bt-page">
    <NxSectionHeader title="Bluetooth" description="Statut de l'adaptateur et périphériques appairés (lecture seule)." />

    <div v-if="status && !status.adapter_present" class="bt-empty">Aucun adaptateur Bluetooth détecté.</div>

    <template v-else-if="status">
      <NxCard>
        <NxBadge :status="status.powered ? 'success' : 'warning'">{{ status.powered ? "activé" : "désactivé" }}</NxBadge>
      </NxCard>

      <NxCard v-for="d in status.devices" :key="d.address" class="bt-row">
        <span>{{ d.name }}</span>
        <span class="bt-address">{{ d.address }}</span>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.bt-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.bt-empty { color: var(--nx-text-secondary); }
.bt-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; }
.bt-address { color: var(--nx-text-secondary); font-size: 12px; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit the frontend**

```bash
git add src/pages/BluetoothPage.vue src/pages/BluetoothPage.spec.ts
git commit -m "feat: add BluetoothPage (spec section 3.3)"
```

---

## Task 3: `run_script` command + `scriptsStore.ts` + `ScriptsPage.vue`

**Files:**
- Create: `src-tauri/src/scripts.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/stores/scriptsStore.ts`
- Test: `src/stores/scriptsStore.spec.ts`
- Create: `src/pages/ScriptsPage.vue`
- Test: `src/pages/ScriptsPage.spec.ts`

### Backend

- [ ] **Step 1: Write the failing Rust test**

```rust
// src-tauri/src/scripts.rs
use crate::subprocess;
use std::time::Duration;

/// Runs `content` as a shell script via `sh -c`, with the invoking user's
/// own privileges only -- no elevation, no pkexec. This is not a new
/// privilege boundary: it is conceptually identical to the user opening a
/// terminal and typing the same command themselves. Bounded to 60 seconds
/// -- long enough for a real utility script, short enough that a runaway
/// script can't hang the app indefinitely.
#[tauri::command]
pub fn run_script(content: String) -> Result<String, String> {
    subprocess::run_with_timeout("sh", &["-c", &content], Duration::from_secs(60))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runs_a_simple_script_and_captures_stdout() {
        let result = run_script("echo hello".to_string()).expect("should succeed");
        assert_eq!(result.trim(), "hello");
    }

    #[test]
    fn returns_err_for_a_script_that_exits_nonzero() {
        assert!(run_script("exit 1".to_string()).is_err());
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile.**

- [ ] **Step 3: Register `mod scripts;` in `lib.rs`** (alphabetically, between `mod report;` and `mod security_write;`) and add `scripts::run_script,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `172 passed; 0 failed; 1 ignored`** (170 baseline + 2 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/scripts.rs src-tauri/src/lib.rs
git commit -m "feat: add run_script — non-privileged shell execution as the invoking user (spec section 3.4)"
```

### Frontend store

- [ ] **Step 6: Write the failing store test**

```typescript
// src/stores/scriptsStore.spec.ts
import { describe, it, expect, beforeEach } from "vitest";
import { setActivePinia, createPinia } from "pinia";
import { useScriptsStore } from "./scriptsStore";

describe("scriptsStore", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("starts empty and persists an added script across store instances", () => {
    const store = useScriptsStore();
    expect(store.scripts).toEqual([]);
    store.addScript("Lister /tmp", "ls -la /tmp");

    setActivePinia(createPinia());
    const reloaded = useScriptsStore();
    expect(reloaded.scripts).toEqual([{ name: "Lister /tmp", content: "ls -la /tmp" }]);
  });

  it("removes a script by name", () => {
    const store = useScriptsStore();
    store.addScript("A", "echo a");
    store.addScript("B", "echo b");
    store.removeScript("A");
    expect(store.scripts.map((s) => s.name)).toEqual(["B"]);
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `src/stores/scriptsStore.ts`**, mirroring `preferencesStore.ts`'s exact persistence pattern (`localStorage.getItem`/`setItem`, JSON parse with a safe fallback to an empty default on any parse failure):

```typescript
import { defineStore } from "pinia";

const STORAGE_KEY = "nitrux-scripts";

export interface SavedScript {
  name: string;
  content: string;
}

function readPersistedScripts(): SavedScript[] {
  const stored = localStorage.getItem(STORAGE_KEY);
  if (!stored) return [];
  try {
    const parsed: unknown = JSON.parse(stored);
    return Array.isArray(parsed) ? (parsed as SavedScript[]) : [];
  } catch {
    return [];
  }
}

function persist(scripts: SavedScript[]) {
  localStorage.setItem(STORAGE_KEY, JSON.stringify(scripts));
}

export const useScriptsStore = defineStore("scripts", {
  state: (): { scripts: SavedScript[] } => ({ scripts: readPersistedScripts() }),
  actions: {
    addScript(name: string, content: string) {
      this.scripts.push({ name, content });
      persist(this.scripts);
    },
    removeScript(name: string) {
      this.scripts = this.scripts.filter((s) => s.name !== name);
      persist(this.scripts);
    },
  },
});
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit the store**

```bash
git add src/stores/scriptsStore.ts src/stores/scriptsStore.spec.ts
git commit -m "feat: add scriptsStore — localStorage-persisted saved scripts (spec section 3.4)"
```

### Frontend page

- [ ] **Step 11: Write the failing page test**

```typescript
// src/pages/ScriptsPage.spec.ts
import { describe, it, expect, vi, beforeEach } from "vitest";
import { mount } from "@vue/test-utils";
import { setActivePinia, createPinia } from "pinia";
import ScriptsPage from "./ScriptsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn().mockResolvedValue("hello\n"),
}));

describe("ScriptsPage", () => {
  beforeEach(() => {
    setActivePinia(createPinia());
    localStorage.clear();
  });

  it("saves a new script and lists it", async () => {
    const wrapper = mount(ScriptsPage);
    await wrapper.find("input[placeholder*='Nom']").setValue("Test");
    await wrapper.find("textarea").setValue("echo hello");
    const saveButton = wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!;
    await saveButton.trigger("click");
    expect(wrapper.text()).toContain("Test");
  });

  it("runs a saved script and shows its output", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(ScriptsPage);
    await wrapper.find("input[placeholder*='Nom']").setValue("Test");
    await wrapper.find("textarea").setValue("echo hello");
    await wrapper.findAll("button").find((b) => b.text() === "Enregistrer")!.trigger("click");
    const runButton = wrapper.findAll("button").find((b) => b.text() === "Exécuter")!;
    await runButton.trigger("click");
    await vi.waitFor(() => expect(wrapper.text()).toContain("hello"));
    expect(invoke).toHaveBeenCalledWith("run_script", { content: "echo hello" });
  });
});
```

- [ ] **Step 12: Run it, confirm it FAILS**

- [ ] **Step 13: Write `ScriptsPage.vue`**

```vue
<!-- src/pages/ScriptsPage.vue -->
<script setup lang="ts">
import { ref } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";
import { useScriptsStore } from "@/stores/scriptsStore";

const store = useScriptsStore();

const newName = ref("");
const newContent = ref("");

function saveScript() {
  if (newName.value === "" || newContent.value === "") return;
  store.addScript(newName.value, newContent.value);
  newName.value = "";
  newContent.value = "";
}

const running = ref<string | null>(null);
const outputs = ref<Record<string, string>>({});
const errors = ref<Record<string, string>>({});

async function runScript(name: string, content: string) {
  running.value = name;
  delete errors.value[name];
  try {
    outputs.value[name] = await invoke<string>("run_script", { content });
  } catch (e) {
    errors.value[name] = String(e);
  } finally {
    running.value = null;
  }
}
</script>

<template>
  <div class="scr-page">
    <NxSectionHeader title="Scripts & Snippets" description="Enregistre et exécute des scripts shell avec vos propres privilèges — aucune élévation." />

    <NxCard class="scr-new">
      <NxInput v-model="newName" placeholder="Nom du script..." />
      <textarea v-model="newContent" class="scr-textarea" rows="4" placeholder="Contenu du script..."></textarea>
      <NxButton @click="saveScript">Enregistrer</NxButton>
    </NxCard>

    <NxCard v-for="s in store.scripts" :key="s.name" class="scr-item">
      <div class="scr-item-header">
        <span>{{ s.name }}</span>
        <div class="scr-item-actions">
          <NxButton :disabled="running !== null" @click="runScript(s.name, s.content)">
            {{ running === s.name ? "En cours..." : "Exécuter" }}
          </NxButton>
          <NxButton variant="danger" @click="store.removeScript(s.name)">Supprimer</NxButton>
        </div>
      </div>
      <pre class="scr-content">{{ s.content }}</pre>
      <NxCard v-if="errors[s.name]" danger>{{ errors[s.name] }}</NxCard>
      <pre v-if="outputs[s.name]" class="scr-output">{{ outputs[s.name] }}</pre>
    </NxCard>
  </div>
</template>

<style scoped>
.scr-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.scr-new { display: flex; flex-direction: column; gap: 10px; }
.scr-textarea { width: 100%; padding: 10px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; }
.scr-item { display: flex; flex-direction: column; gap: 8px; }
.scr-item-header { display: flex; justify-content: space-between; align-items: center; }
.scr-item-actions { display: flex; gap: 8px; }
.scr-content, .scr-output { font-size: 12px; margin: 0; white-space: pre-wrap; word-break: break-word; }
.scr-output { color: var(--nx-accent-success); }
</style>
```

- [ ] **Step 14: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 15: Commit the page**

```bash
git add src/pages/ScriptsPage.vue src/pages/ScriptsPage.spec.ts
git commit -m "feat: add ScriptsPage (spec section 3.4)"
```

---

## Task 4: `DnsSwitcherPage.vue`

**Files:**
- Create: `src/pages/DnsSwitcherPage.vue`
- Test: `src/pages/DnsSwitcherPage.spec.ts`

Reuses the already-existing, already-privileged, already-VM-verified `set_dns_servers` command (`network_write.rs`, unchanged) — no new backend.

- [ ] **Step 1: Write the failing test**

```typescript
// src/pages/DnsSwitcherPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import DnsSwitcherPage from "./DnsSwitcherPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_network_snapshot") {
      return Promise.resolve({ wifi_networks: [], listening_ports: [], dns_servers: ["1.1.1.1"], hosts_file: "" });
    }
    if (cmd === "set_dns_servers") return Promise.resolve(null);
    return Promise.resolve(null);
  }),
}));

describe("DnsSwitcherPage", () => {
  it("applies the Cloudflare preset via set_dns_servers", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DnsSwitcherPage);
    const button = wrapper.findAll("button").find((b) => b.text().includes("Cloudflare"))!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("set_dns_servers", { content: "1.1.1.1\n1.0.0.1" }));
  });

  it("applies a manually entered DNS list via set_dns_servers", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    const wrapper = mount(DnsSwitcherPage);
    await wrapper.find("textarea").setValue("9.9.9.9");
    const button = wrapper.findAll("button").find((b) => b.text() === "Appliquer")!;
    await button.trigger("click");
    await vi.waitFor(() => expect(invoke).toHaveBeenCalledWith("set_dns_servers", { content: "9.9.9.9" }));
  });
});
```

- [ ] **Step 2: Run it, confirm it FAILS**

- [ ] **Step 3: Write `DnsSwitcherPage.vue`**

```vue
<!-- src/pages/DnsSwitcherPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxButton from "@/components/ui/NxButton.vue";
import NxBadge from "@/components/ui/NxBadge.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

const PRESETS = [
  { label: "Cloudflare (1.1.1.1)", servers: ["1.1.1.1", "1.0.0.1"] },
  { label: "Google (8.8.8.8)", servers: ["8.8.8.8", "8.8.4.4"] },
  { label: "Quad9 (9.9.9.9)", servers: ["9.9.9.9", "149.112.112.112"] },
];

const currentDns = ref<string[]>([]);
const manualDns = ref("");
const applying = ref(false);
const applyError = ref<string | null>(null);
const applySuccess = ref(false);

async function refresh() {
  const snapshot = await invoke<{ dns_servers: string[] }>("get_network_snapshot");
  currentDns.value = snapshot.dns_servers;
}

onMounted(refresh);

async function apply(content: string) {
  applying.value = true;
  applyError.value = null;
  applySuccess.value = false;
  try {
    await invoke("set_dns_servers", { content });
    applySuccess.value = true;
    await refresh();
  } catch (e) {
    applyError.value = String(e);
  } finally {
    applying.value = false;
  }
}
</script>

<template>
  <div class="dns-page">
    <NxSectionHeader title="DNS Switcher" :description="`Serveurs actuels : ${currentDns.join(', ') || 'aucun'}`" />

    <NxCard v-if="applyError" danger>{{ applyError }}</NxCard>
    <NxBadge v-if="applySuccess" status="success">DNS mis à jour.</NxBadge>

    <NxCard class="dns-presets">
      <NxButton v-for="p in PRESETS" :key="p.label" :disabled="applying" @click="apply(p.servers.join('\n'))">
        {{ p.label }}
      </NxButton>
    </NxCard>

    <NxCard class="dns-manual">
      <textarea v-model="manualDns" class="dns-textarea" rows="4" placeholder="Un serveur DNS par ligne..."></textarea>
      <NxButton :disabled="applying || manualDns === ''" @click="apply(manualDns)">Appliquer</NxButton>
    </NxCard>
  </div>
</template>

<style scoped>
.dns-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.dns-presets { display: flex; gap: 10px; flex-wrap: wrap; }
.dns-manual { display: flex; flex-direction: column; gap: 10px; }
.dns-textarea { width: 100%; padding: 10px; border-radius: var(--nx-style-radius); border: var(--nx-style-border-width) solid var(--nx-style-border-color); background: var(--nx-style-bg); color: var(--nx-text-primary); font-family: monospace; font-size: 12px; }
</style>
```

- [ ] **Step 4: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 5: Commit**

```bash
git add src/pages/DnsSwitcherPage.vue src/pages/DnsSwitcherPage.spec.ts
git commit -m "feat: add DnsSwitcherPage — presets + manual entry via set_dns_servers (spec section 3.2)"
```

---

## Task 5: Extend `categories.ts`'s "reseau" category + wire `App.vue`

**Files:**
- Modify: `src/navigation/categories.ts`
- Modify: `src/navigation/categories.spec.ts`
- Modify: `src/components/nav/AppNav.vue`
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`

- [ ] **Step 1: Read the live `src/navigation/categories.ts`.** The `"reseau"` category currently has 2 pages (`network-overview`, `firewall`). Add 4 more, matching NiTriTe's own ordering (dns-switcher, wifi-analyzer come before bluetooth/scripts in `navigation.ts`, `terminal` is skipped per this plan's scope decision):

```typescript
      { id: "dns-switcher", label: "DNS Switcher", icon: "globe" },
      { id: "wifi-analyzer", label: "WiFi Analyzer", icon: "radio" },
      { id: "bluetooth", label: "Bluetooth", icon: "bluetooth" },
      { id: "scripts", label: "Scripts & Snippets", icon: "file-code" },
```
(inserted right after the existing `network-overview` entry, before `firewall` — matching `navigation.ts`'s own relative ordering where DNS/WiFi come before the later network-adjacent items)

- [ ] **Step 2: Add the 4 new icon names to `AppNav.vue`'s `iconMap`.** Add to the existing `lucide-vue-next` import statement:
```typescript
  Globe, Radio, Bluetooth, FileCode,
```
And to `iconMap`:
```typescript
  globe: Globe,
  radio: Radio,
  bluetooth: Bluetooth,
  "file-code": FileCode,
```

- [ ] **Step 3: Update `categories.spec.ts`** (category count stays 8 — this extends an existing category, doesn't add a new one):
```typescript
  it("includes the 4 new Phase R8 Réseau & Terminal pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("dns-switcher");
    expect(allPageIds).toContain("wifi-analyzer");
    expect(allPageIds).toContain("bluetooth");
    expect(allPageIds).toContain("scripts");
  });
```

- [ ] **Step 4: Read the live `src/App.vue` and `src/App.spec.ts`.** Add 4 new page imports and map entries, and 2 new `App.spec.ts` tests:

```typescript
  it("shows the real BluetoothPage for the bluetooth id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Bluetooth")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Statut de l'adaptateur");
  });

  it("shows the real ScriptsPage for the scripts id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Scripts & Snippets")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("aucune élévation");
  });
```

- [ ] **Step 5: Run `npx vitest run src/App.spec.ts src/navigation/categories.spec.ts`, confirm all pass** (App.spec.ts: 13 tests — 11 from before + 2 new; categories.spec.ts: 7 tests — 6 from before + 1 new).

- [ ] **Step 6: Commit**

```bash
git add src/navigation/categories.ts src/navigation/categories.spec.ts src/components/nav/AppNav.vue src/App.vue src/App.spec.ts
git commit -m "feat: wire App.vue and categories.ts to the 4 new Réseau & Terminal pages (spec section 3)"
```

---

## Task 6: Full verification pass — frontend, backend, and live VM check

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite.**

Expected: baseline entering this plan was 160 (end of R7). Net delta: Task 1 (+2) + Task 2 (+2) + Task 3 (+2 store +2 page = +4) + Task 4 (+2) + Task 5 (+1 categories.spec.ts +2 App.spec.ts) = +13, expected total **173**. Report the real observed total.

- [ ] **Step 2: Type-check.** `npx vue-tsc --noEmit`, expect clean.

- [ ] **Step 3: Confirm the Rust suite.** Expect `172 passed; 0 failed; 1 ignored` (165 R7 baseline + 5 Task2 + 2 Task3 = 172).

- [ ] **Step 4: Build and install on the VM, verify bluetoothctl/run_script against real output.**

Build: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r8-reseau-terminal && npx tauri build 2>&1 | tail -20"`.

Transfer + install on the VM (`172.18.32.124`, user `dev`, password `1998`, SSH helper scripts at `C:\Users\Momo\AppData\Local\Temp\claude\C--Users-Momo\880690b1-319b-40bd-bb2c-957700dc8af4\scratchpad\`, or write an equivalent if that path is gone).

Since this task's only backend additions are read-only (`bluetooth.rs`) and non-privileged (`run_script`, `scripts.rs`) — no `pkexec`/polkit involved at all — verification is simpler than R7's Task 9: just confirm the real commands behave as the parsers assume. Run directly over SSH (no `pkttyagent` needed, since neither of these is privileged):
```
bluetoothctl show
bluetoothctl devices
```
Confirm the output shape matches `parse_powered_status`/`parse_device_line`'s assumptions (or if the VM has no real Bluetooth hardware, confirm `get_bluetooth_status` still degrades gracefully to `adapter_present: false` rather than erroring — check by calling the app's own command via the running instance, or by confirming `bluetoothctl show` itself fails/errors on this VM and that `subprocess::run_with_timeout`'s existing error handling covers that case, which it already does by design).

Also confirm `run_script`'s underlying `sh -c` invocation behaves as expected for a trivial script on the real VM (e.g. `sh -c "echo hello"` produces `hello`).

- [ ] **Step 5: Commit any final cleanup.** No further commit expected if Steps 1-4 all pass clean.

---

## Self-Review

**Spec coverage:** §3.1 (WiFi Analyzer) — Task 1. §3.2 (DNS Switcher) — Task 4. §3.3 (Bluetooth, read-only) — Task 2. §3.4 (Scripts & Snippets) — Task 3. §2/§4 (Terminal explicitly deferred, not built) — confirmed no task in this plan touches PTY/xterm.js/any new dependency.

**Placeholder scan:** No "TBD"/"TODO". The Terminal deferral is explicitly justified in the spec, not a silent omission.

**Type consistency:** `BluetoothDevice`/`BluetoothStatus` (Rust) match `BluetoothPage.vue`'s TypeScript interfaces exactly. `SavedScript` (frontend-only, `scriptsStore.ts`) has no Rust counterpart by design (backend only executes `content: String`, never sees the `name`/persistence concept at all). `Nx*` component props cross-checked against R1's `defineProps` throughout, per every prior phase's discipline — still subject to live re-verification at execution time per R6/R7's own lessons about plan snapshots drifting from live files.
