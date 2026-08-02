# Phase R11 (Diagnostic & config) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Give NiTruX a real "Diagnostic" navigation category (currently the "diagnostic" page is a 51-line PCI-only table buried inside "Système"), covering the genuinely-new, Linux-applicable, non-duplicate hardware/software inspection domains from NiTriTe Windows's 493-line `DiagnosticPage.vue`.

**Architecture:** 6 new pages, all 100% read-only and non-privileged — every command they use was individually confirmed runnable without root on the project's dev VM during this plan's research (see the design spec). One backend fix falls out of this research: `lscpu`'s output is locale-dependent (confirmed French field labels on the dev VM's default locale, would silently break an English-coded parser) — `subprocess.rs` gets a small env-var-aware sibling to `run_with_timeout` to force `LC_ALL=C` for this one call, without touching any of the ~15 existing callers of the original function.

**Tech Stack:** Same as R1-R10 — Tauri v2 + Rust backend, Vue 3 + Pinia frontend, vitest + `cargo test`.

---

## Task 1: `subprocess.rs` env support + `hardware_details.rs` + `HardwareDetailsPage.vue`

**Files:**
- Modify: `src-tauri/src/subprocess.rs`
- Create: `src-tauri/src/hardware_details.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/HardwareDetailsPage.vue`
- Test: `src/pages/HardwareDetailsPage.spec.ts`

Real `lscpu` output on the project's dev VM confirmed **French** field labels by default (`Nom de modèle :`, `Cœur(s) par socket :`) — this plan's parser is written against the stable English keys (`Model name`, `Core(s) per socket`) that `LC_ALL=C` guarantees regardless of host locale.

### Backend

- [ ] **Step 1: Add an env-var-aware sibling to `run_with_timeout` in `subprocess.rs`**

Add right after the existing `run_with_timeout` function (keep that function's public signature and every existing caller untouched):

```rust
/// Like `run_with_timeout`, but sets additional environment variables on
/// the child process before spawning it. Needed for commands whose output
/// format is locale-dependent (confirmed during R11's research: `lscpu`
/// genuinely emits translated field labels like "Nom de modèle :" on a
/// French-locale system, which would silently break a parser written
/// against the stable English keys) -- `LC_ALL=C` pins the output to a
/// locale-independent format without affecting any other running process.
pub fn run_with_timeout_env(
    program: &str,
    args: &[&str],
    envs: &[(&str, &str)],
    timeout: Duration,
) -> Result<String, String> {
    let mut command = Command::new(program);
    command.args(args).stdout(Stdio::piped()).stderr(Stdio::piped());
    for (key, value) in envs {
        command.env(key, value);
    }
    let child = command
        .spawn()
        .map_err(|e| format!("{program} introuvable ou impossible à lancer : {e}"))?;

    let pid = child.id();
    let (tx, rx) = mpsc::channel();

    std::thread::spawn(move || {
        let _ = tx.send(child.wait_with_output());
    });

    match rx.recv_timeout(timeout) {
        Ok(Ok(output)) => {
            if output.status.success() {
                Ok(String::from_utf8_lossy(&output.stdout).into_owned())
            } else {
                let code = output
                    .status
                    .code()
                    .map(|c| c.to_string())
                    .unwrap_or_else(|| "inconnu".to_string());
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(format!("{program} a échoué (code {code}) : {}", stderr.trim()))
            }
        }
        Ok(Err(e)) => Err(format!("erreur en lisant la sortie de {program} : {e}")),
        Err(mpsc::RecvTimeoutError::Timeout) => {
            let _ = Command::new("kill").arg("-9").arg(pid.to_string()).status();
            Err(format!(
                "{program} a dépassé le délai de {timeout:?} et a été arrêté"
            ))
        }
        Err(mpsc::RecvTimeoutError::Disconnected) => Err(format!(
            "le processus {program} s'est terminé de façon inattendue"
        )),
    }
}
```

Add a test in `subprocess.rs`'s existing `#[cfg(test)] mod tests`:

```rust
    #[test]
    fn run_with_timeout_env_makes_the_env_var_visible_to_the_child() {
        let out = run_with_timeout_env("sh", &["-c", "echo $NITRUX_TEST_VAR"], &[("NITRUX_TEST_VAR", "hello")], Duration::from_secs(2))
            .expect("should succeed");
        assert_eq!(out.trim(), "hello");
    }
```

- [ ] **Step 2: Run `cargo test`, confirm the new test passes and nothing else broke.**

- [ ] **Step 3: Write the failing Rust tests for `hardware_details.rs`**

```rust
// src-tauri/src/hardware_details.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct CpuDetails {
    pub model_name: Option<String>,
    pub sockets: Option<String>,
    pub cores_per_socket: Option<String>,
    pub threads_per_core: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct BoardDetails {
    pub product_name: Option<String>,
    pub sys_vendor: Option<String>,
    pub board_name: Option<String>,
    pub bios_version: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct MemoryDetails {
    pub total_kb: Option<u64>,
    pub available_kb: Option<u64>,
    pub cached_kb: Option<u64>,
    pub swap_total_kb: Option<u64>,
}

#[derive(Serialize, Clone)]
pub struct HardwareDetails {
    pub cpu: CpuDetails,
    pub board: BoardDetails,
    pub memory: MemoryDetails,
}

/// Parses `lscpu` output (run with `LC_ALL=C` by the caller so these
/// English keys are stable). Each field is independently optional -- a
/// missing key (e.g. a virtualized CPU without "Socket(s)") must never
/// hide the fields that ARE present.
pub fn parse_lscpu_output(output: &str) -> CpuDetails {
    let mut model_name = None;
    let mut sockets = None;
    let mut cores_per_socket = None;
    let mut threads_per_core = None;
    for line in output.lines() {
        let Some((key, value)) = line.split_once(':') else { continue };
        let value = value.trim().to_string();
        match key.trim() {
            "Model name" => model_name = Some(value),
            "Socket(s)" => sockets = Some(value),
            "Core(s) per socket" => cores_per_socket = Some(value),
            "Thread(s) per core" => threads_per_core = Some(value),
            _ => {}
        }
    }
    CpuDetails { model_name, sockets, cores_per_socket, threads_per_core }
}

/// Reads one `/sys/class/dmi/id/*` field -- readable without root on every
/// system tested during this plan's research (confirmed on the dev VM:
/// `product_name` and `bios_version` both readable as a normal user).
/// Missing entirely on some systems/architectures, so `None` on any read
/// failure is a normal, expected outcome, not an error to surface.
fn read_dmi_field(name: &str) -> Option<String> {
    std::fs::read_to_string(format!("/sys/class/dmi/id/{name}"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

pub fn get_board_details() -> BoardDetails {
    BoardDetails {
        product_name: read_dmi_field("product_name"),
        sys_vendor: read_dmi_field("sys_vendor"),
        board_name: read_dmi_field("board_name"),
        bios_version: read_dmi_field("bios_version"),
    }
}

/// Parses `/proc/meminfo` content. Values are in kB per the kernel's own
/// convention (the trailing "kB" unit suffix on each line is discarded,
/// only the leading numeric value is kept).
pub fn parse_meminfo(content: &str) -> MemoryDetails {
    let mut total_kb = None;
    let mut available_kb = None;
    let mut cached_kb = None;
    let mut swap_total_kb = None;
    for line in content.lines() {
        let Some((key, rest)) = line.split_once(':') else { continue };
        let value_kb = rest.trim().split_whitespace().next().and_then(|v| v.parse::<u64>().ok());
        match key {
            "MemTotal" => total_kb = value_kb,
            "MemAvailable" => available_kb = value_kb,
            "Cached" => cached_kb = value_kb,
            "SwapTotal" => swap_total_kb = value_kb,
            _ => {}
        }
    }
    MemoryDetails { total_kb, available_kb, cached_kb, swap_total_kb }
}

#[tauri::command]
pub fn get_hardware_details() -> HardwareDetails {
    let cpu = subprocess::run_with_timeout_env("lscpu", &[], &[("LC_ALL", "C")], Duration::from_secs(10))
        .map(|out| parse_lscpu_output(&out))
        .unwrap_or(CpuDetails { model_name: None, sockets: None, cores_per_socket: None, threads_per_core: None });
    let board = get_board_details();
    let memory = std::fs::read_to_string("/proc/meminfo")
        .ok()
        .map(|c| parse_meminfo(&c))
        .unwrap_or(MemoryDetails { total_kb: None, available_kb: None, cached_kb: None, swap_total_kb: None });
    HardwareDetails { cpu, board, memory }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lscpu_english_output() {
        let output = "Architecture:                       x86_64\nCPU(s):                              6\nModel name:                          11th Gen Intel(R) Core(TM) i5-11400H @ 2.70GHz\nSocket(s):                           1\nCore(s) per socket:                  3\nThread(s) per core:                  2\n";
        let cpu = parse_lscpu_output(output);
        assert_eq!(cpu.model_name.as_deref(), Some("11th Gen Intel(R) Core(TM) i5-11400H @ 2.70GHz"));
        assert_eq!(cpu.sockets.as_deref(), Some("1"));
        assert_eq!(cpu.cores_per_socket.as_deref(), Some("3"));
        assert_eq!(cpu.threads_per_core.as_deref(), Some("2"));
    }

    #[test]
    fn parse_lscpu_output_leaves_missing_fields_none_without_panicking() {
        let cpu = parse_lscpu_output("Architecture: x86_64\n");
        assert_eq!(cpu.model_name, None);
    }

    #[test]
    fn parses_meminfo_content() {
        let content = "MemTotal:       10231988 kB\nMemFree:          921716 kB\nMemAvailable:    6614532 kB\nCached:          5031560 kB\nSwapTotal:        2097148 kB\n";
        let mem = parse_meminfo(content);
        assert_eq!(mem.total_kb, Some(10231988));
        assert_eq!(mem.available_kb, Some(6614532));
        assert_eq!(mem.cached_kb, Some(5031560));
        assert_eq!(mem.swap_total_kb, Some(2097148));
    }

    #[test]
    fn parse_meminfo_ignores_malformed_lines() {
        let mem = parse_meminfo("not a valid line\nMemTotal:       1000 kB\n");
        assert_eq!(mem.total_kb, Some(1000));
    }
}
```

- [ ] **Step 4: Run tests, confirm they fail to compile** (module not registered).

- [ ] **Step 5: Register `mod hardware_details;` in `lib.rs`** (alphabetically, right after `mod hardware;` at line 17, before `mod hashcheck;`) and add `hardware_details::get_hardware_details,` to the handler list.

- [ ] **Step 6: Run the full Rust suite, expect `194 passed; 0 failed; 1 ignored`** (188 R10 baseline + 1 subprocess test + 5 hardware_details tests).

- [ ] **Step 7: Commit the backend**

```bash
git add src-tauri/src/subprocess.rs src-tauri/src/hardware_details.rs src-tauri/src/lib.rs
git commit -m "feat: add get_hardware_details — CPU/board/memory, locale-safe (spec section 3)"
```

### Frontend

- [ ] **Step 8: Write the failing frontend test**

Trace both assertions against the template below by hand before writing it (discipline established since R8 Task 1, reused through R9/R10).

```typescript
// src/pages/HardwareDetailsPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import HardwareDetailsPage from "./HardwareDetailsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_hardware_details") {
      return Promise.resolve({
        cpu: { model_name: "11th Gen Intel(R) Core(TM) i5-11400H", sockets: "1", cores_per_socket: "3", threads_per_core: "2" },
        board: { product_name: "Virtual Machine", sys_vendor: "Microsoft Corporation", board_name: null, bios_version: "Hyper-V UEFI Release v4.1" },
        memory: { total_kb: 10231988, available_kb: 6614532, cached_kb: 5031560, swap_total_kb: 2097148 },
      });
    }
    if (cmd === "get_pci_devices") {
      return Promise.resolve([
        { slot: "00:08.0", class: "VGA compatible controller", description: "Microsoft Corporation Hyper-V virtual VGA" },
        { slot: "00:00.0", class: "Host bridge", description: "Intel Corporation Device" },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("HardwareDetailsPage", () => {
  it("shows CPU, board, and memory details", async () => {
    const wrapper = mount(HardwareDetailsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("11th Gen Intel"));
    expect(wrapper.text()).toContain("Virtual Machine");
    expect(wrapper.text()).toContain("Hyper-V UEFI Release v4.1");
  });

  it("shows only GPU-class PCI devices, not every PCI device", async () => {
    const wrapper = mount(HardwareDetailsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Hyper-V virtual VGA"));
    expect(wrapper.text()).not.toContain("Host bridge");
  });
});
```

- [ ] **Step 9: Run it, confirm it FAILS**

- [ ] **Step 10: Write `HardwareDetailsPage.vue`**

```vue
<!-- src/pages/HardwareDetailsPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface CpuDetails { model_name: string | null; sockets: string | null; cores_per_socket: string | null; threads_per_core: string | null }
interface BoardDetails { product_name: string | null; sys_vendor: string | null; board_name: string | null; bios_version: string | null }
interface MemoryDetails { total_kb: number | null; available_kb: number | null; cached_kb: number | null; swap_total_kb: number | null }
interface HardwareDetails { cpu: CpuDetails; board: BoardDetails; memory: MemoryDetails }
interface PciDevice { slot: string; class: string; description: string }

const details = ref<HardwareDetails | null>(null);
const gpus = ref<PciDevice[]>([]);

onMounted(async () => {
  details.value = await invoke<HardwareDetails>("get_hardware_details");
  const pci = await invoke<PciDevice[]>("get_pci_devices");
  gpus.value = pci.filter((d) => d.class.includes("VGA") || d.class.includes("3D"));
});

function kbToGb(kb: number | null): string {
  return kb === null ? "—" : (kb / 1024 / 1024).toFixed(1);
}

const memoryRows = computed(() => details.value ? [
  { label: "Total", value: kbToGb(details.value.memory.total_kb) + " GB" },
  { label: "Disponible", value: kbToGb(details.value.memory.available_kb) + " GB" },
  { label: "Cache", value: kbToGb(details.value.memory.cached_kb) + " GB" },
  { label: "Swap total", value: kbToGb(details.value.memory.swap_total_kb) + " GB" },
] : []);
</script>

<template>
  <div class="hd-page">
    <NxSectionHeader title="Matériel détaillé" description="Processeur, carte mère, mémoire et GPU." />

    <template v-if="details">
      <NxCard>
        <NxSectionHeader title="Processeur" />
        <div class="hd-row"><span>Modèle</span><span>{{ details.cpu.model_name ?? "—" }}</span></div>
        <div class="hd-row"><span>Socket(s)</span><span>{{ details.cpu.sockets ?? "—" }}</span></div>
        <div class="hd-row"><span>Cœurs par socket</span><span>{{ details.cpu.cores_per_socket ?? "—" }}</span></div>
        <div class="hd-row"><span>Threads par cœur</span><span>{{ details.cpu.threads_per_core ?? "—" }}</span></div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Carte mère & BIOS" />
        <div class="hd-row"><span>Fabricant</span><span>{{ details.board.sys_vendor ?? "—" }}</span></div>
        <div class="hd-row"><span>Modèle</span><span>{{ details.board.product_name ?? "—" }}</span></div>
        <div class="hd-row"><span>Carte mère</span><span>{{ details.board.board_name ?? "—" }}</span></div>
        <div class="hd-row"><span>Version BIOS</span><span>{{ details.board.bios_version ?? "—" }}</span></div>
      </NxCard>

      <NxCard>
        <NxSectionHeader title="Mémoire" />
        <div v-for="row in memoryRows" :key="row.label" class="hd-row">
          <span>{{ row.label }}</span><span>{{ row.value }}</span>
        </div>
      </NxCard>

      <NxCard v-if="gpus.length">
        <NxSectionHeader title="GPU" />
        <div v-for="g in gpus" :key="g.slot" class="hd-row">
          <span>{{ g.description }}</span><span>{{ g.slot }}</span>
        </div>
      </NxCard>
    </template>
  </div>
</template>

<style scoped>
.hd-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.hd-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 11: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 12: Commit**

```bash
git add src/pages/HardwareDetailsPage.vue src/pages/HardwareDetailsPage.spec.ts
git commit -m "feat: add HardwareDetailsPage (spec section 3)"
```

---

## Task 2: `peripherals.rs` + `PeripheralsPage.vue`

**Files:**
- Create: `src-tauri/src/peripherals.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/PeripheralsPage.vue`
- Test: `src/pages/PeripheralsPage.spec.ts`

Real `lsusb`/`pactl`/`xrandr`/`lpstat` all confirmed present and runnable without root on the dev VM during this plan's research; `pactl list short sinks` returned a real sink (`auto_null	PipeWire	float32le 2ch 48000Hz	SUSPENDED`). CUPS (behind `lpstat`) is commonly absent/inactive on non-desktop installs — every section degrades to an empty list, never an error, on failure.

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/peripherals.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct AudioSink { pub name: String, pub driver: String, pub state: String }

#[derive(Serialize, Clone)]
pub struct PrinterInfo { pub name: String, pub status: String }

/// Parses one `pactl list short sinks` line, e.g.
/// "35	auto_null	PipeWire	float32le 2ch 48000Hz	SUSPENDED" (tab-separated:
/// index, name, driver, format, state).
pub fn parse_pactl_sink_line(line: &str) -> Option<AudioSink> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() < 5 {
        return None;
    }
    Some(AudioSink {
        name: fields[1].to_string(),
        driver: fields[2].to_string(),
        state: fields[4].to_string(),
    })
}

/// Parses one `lpstat -p` line, e.g. "printer HP_LaserJet is idle." or
/// "printer HP_LaserJet disabled since ...". Only the name and the leading
/// status word after it are extracted -- the rest of CUPS's freeform
/// sentence is not machine-parsed further, it's not needed for a simple
/// name+status display.
pub fn parse_lpstat_line(line: &str) -> Option<PrinterInfo> {
    let rest = line.strip_prefix("printer ")?;
    let (name, status_rest) = rest.split_once(' ')?;
    let status = status_rest.trim_start_matches("is ").split('.').next().unwrap_or("").trim().to_string();
    Some(PrinterInfo { name: name.to_string(), status })
}

#[tauri::command]
pub fn get_monitors() -> Vec<String> {
    subprocess::run_with_timeout("xrandr", &["--query"], Duration::from_secs(5))
        .map(|out| {
            out.lines()
                .filter(|l| l.contains(" connected"))
                .map(|l| l.split_whitespace().next().unwrap_or("").to_string())
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_usb_devices() -> Vec<String> {
    subprocess::run_with_timeout("lsusb", &[], Duration::from_secs(5))
        .map(|out| out.lines().map(|l| l.to_string()).collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_audio_sinks() -> Vec<AudioSink> {
    subprocess::run_with_timeout("pactl", &["list", "short", "sinks"], Duration::from_secs(5))
        .map(|out| out.lines().filter_map(parse_pactl_sink_line).collect())
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_printers() -> Vec<PrinterInfo> {
    subprocess::run_with_timeout("lpstat", &["-p"], Duration::from_secs(5))
        .map(|out| out.lines().filter_map(parse_lpstat_line).collect())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_pactl_sink_line() {
        let line = "35\tauto_null\tPipeWire\tfloat32le 2ch 48000Hz\tSUSPENDED";
        let sink = parse_pactl_sink_line(line).expect("should parse");
        assert_eq!(sink.name, "auto_null");
        assert_eq!(sink.driver, "PipeWire");
        assert_eq!(sink.state, "SUSPENDED");
    }

    #[test]
    fn ignores_a_malformed_pactl_line() {
        assert!(parse_pactl_sink_line("not enough fields").is_none());
    }

    #[test]
    fn parses_an_idle_printer_line() {
        let printer = parse_lpstat_line("printer HP_LaserJet is idle.  enabled since Mon 01 Aug").expect("should parse");
        assert_eq!(printer.name, "HP_LaserJet");
        assert_eq!(printer.status, "idle");
    }

    #[test]
    fn ignores_a_non_printer_line() {
        assert!(parse_lpstat_line("system default destination: HP_LaserJet").is_none());
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile.**

- [ ] **Step 3: Register `mod peripherals;` in `lib.rs`** (alphabetically, right after `mod packages;`, before `mod portscan;`) and add all 4 commands to the handler list: `peripherals::get_monitors,`, `peripherals::get_usb_devices,`, `peripherals::get_audio_sinks,`, `peripherals::get_printers,`.

- [ ] **Step 4: Run the full Rust suite, expect `198 passed; 0 failed; 1 ignored`** (194 Task 1 + 4 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/peripherals.rs src-tauri/src/lib.rs
git commit -m "feat: add peripherals.rs — monitors/USB/audio/printers, all optional (spec section 4)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/PeripheralsPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import PeripheralsPage from "./PeripheralsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_monitors") return Promise.resolve(["Virtual-1"]);
    if (cmd === "get_usb_devices") return Promise.resolve(["Bus 001 Device 001: ID 1d6b:0002 Linux Foundation"]);
    if (cmd === "get_audio_sinks") return Promise.resolve([{ name: "auto_null", driver: "PipeWire", state: "SUSPENDED" }]);
    if (cmd === "get_printers") return Promise.resolve([]);
    return Promise.resolve(null);
  }),
}));

describe("PeripheralsPage", () => {
  it("shows monitors, USB devices, and audio sinks", async () => {
    const wrapper = mount(PeripheralsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Virtual-1"));
    expect(wrapper.text()).toContain("Linux Foundation");
    expect(wrapper.text()).toContain("auto_null");
  });

  it("shows a clear message when no printers are detected", async () => {
    const wrapper = mount(PeripheralsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("Aucune imprimante détectée"));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `PeripheralsPage.vue`**

```vue
<!-- src/pages/PeripheralsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface AudioSink { name: string; driver: string; state: string }
interface PrinterInfo { name: string; status: string }

const monitors = ref<string[]>([]);
const usbDevices = ref<string[]>([]);
const audioSinks = ref<AudioSink[]>([]);
const printers = ref<PrinterInfo[] | null>(null);

onMounted(async () => {
  monitors.value = await invoke<string[]>("get_monitors");
  usbDevices.value = await invoke<string[]>("get_usb_devices");
  audioSinks.value = await invoke<AudioSink[]>("get_audio_sinks");
  printers.value = await invoke<PrinterInfo[]>("get_printers");
});
</script>

<template>
  <div class="ph-page">
    <NxSectionHeader title="Périphériques" description="Moniteurs, USB, audio et imprimantes." />

    <NxCard>
      <NxSectionHeader title="Moniteurs" />
      <div v-if="monitors.length === 0" class="ph-empty">Aucun moniteur détecté.</div>
      <div v-for="m in monitors" :key="m" class="ph-row">{{ m }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="USB" />
      <div v-if="usbDevices.length === 0" class="ph-empty">Aucun périphérique USB détecté.</div>
      <div v-for="u in usbDevices" :key="u" class="ph-row">{{ u }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Audio" />
      <div v-if="audioSinks.length === 0" class="ph-empty">Aucune sortie audio détectée.</div>
      <div v-for="a in audioSinks" :key="a.name" class="ph-row">{{ a.name }} ({{ a.driver }}) — {{ a.state }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Imprimantes" />
      <div v-if="printers && printers.length === 0" class="ph-empty">Aucune imprimante détectée.</div>
      <div v-for="p in printers ?? []" :key="p.name" class="ph-row">{{ p.name }} — {{ p.status }}</div>
    </NxCard>
  </div>
</template>

<style scoped>
.ph-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.ph-empty { color: var(--nx-text-secondary); font-size: 13px; }
.ph-row { padding: 4px 0; font-size: 13px; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit**

```bash
git add src/pages/PeripheralsPage.vue src/pages/PeripheralsPage.spec.ts
git commit -m "feat: add PeripheralsPage (spec section 4)"
```

---

## Task 3: `processes.rs` + `ProcessesPage.vue`

**Files:**
- Create: `src-tauri/src/processes.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/ProcessesPage.vue`
- Test: `src/pages/ProcessesPage.spec.ts`

Real `systemctl list-timers` on the dev VM confirmed genuine scheduled tasks (`fwupd-refresh.timer`, `apt-daily.timer`, `apt-daily-upgrade.timer`, `e2scrub_all.timer`). `crontab -l` confirmed it prints "no crontab for dev" (not an error exit) when empty — this plan treats that specific message as "empty list", not a failure.

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/processes.rs
use crate::subprocess;
use serde::Serialize;
use std::sync::Mutex;
use std::time::Duration;
use sysinfo::System;

#[derive(Serialize, Clone)]
pub struct ProcessInfo {
    pub pid: u32,
    pub name: String,
    pub cpu_percent: f32,
    pub memory_bytes: u64,
}

#[derive(Serialize, Clone)]
pub struct AutostartEntry { pub name: String }

/// Parses one line of `systemctl list-timers --no-pager` output. The
/// header row and the trailing summary line ("N timers listed.") are both
/// naturally rejected by requiring a line to end in ".timer" on its unit
/// column -- simpler and more robust than trying to detect the header by
/// position, since column widths vary with content.
pub fn parse_timer_line(line: &str) -> Option<String> {
    line.split_whitespace()
        .find(|token| token.ends_with(".timer"))
        .map(|s| s.to_string())
}

#[tauri::command]
pub fn get_processes(state: tauri::State<Mutex<System>>) -> Vec<ProcessInfo> {
    let mut sys = state.lock().unwrap();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    sys.processes()
        .iter()
        .map(|(pid, process)| ProcessInfo {
            pid: pid.as_u32(),
            name: process.name().to_string_lossy().into_owned(),
            cpu_percent: process.cpu_usage(),
            memory_bytes: process.memory(),
        })
        .collect()
}

#[tauri::command]
pub fn get_systemd_services() -> Vec<String> {
    subprocess::run_with_timeout("systemctl", &["list-units", "--type=service", "--no-pager", "--plain"], Duration::from_secs(10))
        .map(|out| {
            out.lines()
                .filter(|l| l.contains(".service"))
                .filter_map(|l| l.split_whitespace().next().map(|s| s.to_string()))
                .collect()
        })
        .unwrap_or_default()
}

#[tauri::command]
pub fn get_autostart_entries() -> Vec<AutostartEntry> {
    let mut names: Vec<String> = subprocess::run_with_timeout(
        "systemctl",
        &["--user", "list-unit-files", "--state=enabled", "--no-pager", "--plain"],
        Duration::from_secs(10),
    )
    .map(|out| out.lines().filter_map(|l| l.split_whitespace().next().map(|s| s.to_string())).collect())
    .unwrap_or_default();

    if let Ok(home) = std::env::var("HOME") {
        let autostart_dir = std::path::Path::new(&home).join(".config/autostart");
        if let Ok(entries) = std::fs::read_dir(autostart_dir) {
            for entry in entries.flatten() {
                if let Some(name) = entry.file_name().to_str() {
                    if name.ends_with(".desktop") {
                        names.push(name.to_string());
                    }
                }
            }
        }
    }
    names.into_iter().map(|name| AutostartEntry { name }).collect()
}

#[tauri::command]
pub fn get_scheduled_tasks() -> Vec<String> {
    let mut tasks: Vec<String> = subprocess::run_with_timeout("crontab", &["-l"], Duration::from_secs(5))
        .map(|out| {
            out.lines()
                .filter(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
                .map(|l| l.to_string())
                .collect()
        })
        .unwrap_or_default();

    let timers = subprocess::run_with_timeout("systemctl", &["list-timers", "--no-pager", "--plain"], Duration::from_secs(10))
        .map(|out| out.lines().filter_map(parse_timer_line).collect::<Vec<_>>())
        .unwrap_or_default();
    tasks.extend(timers);
    tasks
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_systemd_timer_line() {
        let line = "Sun 2026-08-02 02:22:55 CEST    17min Sun 2026-08-02 01:32:38 CEST    32min ago fwupd-refresh.timer          fwupd-refresh.service";
        assert_eq!(parse_timer_line(line), Some("fwupd-refresh.timer".to_string()));
    }

    #[test]
    fn ignores_the_timer_list_header_line() {
        assert_eq!(parse_timer_line("NEXT                             LEFT LAST                               PASSED UNIT                         ACTIVATES"), None);
    }

    #[test]
    fn ignores_the_timer_list_summary_line() {
        assert_eq!(parse_timer_line("4 timers listed."), None);
    }
}
```

Note: `get_processes` reuses the SAME `Mutex<System>` Tauri-managed state already set up by `system.rs`'s `get_system_snapshot` (see `lib.rs`'s `.manage(Mutex::new(System::new_all()))`) -- no new managed state needed, this command just calls `refresh_processes` on the existing shared instance before reading it.

- [ ] **Step 2: Run tests, confirm they fail to compile.**

- [ ] **Step 3: Register `mod processes;` in `lib.rs`** (alphabetically, right after `mod portscan;`, before `mod report;`) and add `processes::get_processes,`, `processes::get_systemd_services,`, `processes::get_autostart_entries,`, `processes::get_scheduled_tasks,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `201 passed; 0 failed; 1 ignored`** (198 Task 2 + 3 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/processes.rs src-tauri/src/lib.rs
git commit -m "feat: add processes.rs — processes/services/autostart/scheduled tasks (spec section 5)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/ProcessesPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import ProcessesPage from "./ProcessesPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_processes") return Promise.resolve([{ pid: 1234, name: "nitrux", cpu_percent: 2.5, memory_bytes: 104857600 }]);
    if (cmd === "get_systemd_services") return Promise.resolve(["ssh.service"]);
    if (cmd === "get_autostart_entries") return Promise.resolve([{ name: "nm-applet.desktop" }]);
    if (cmd === "get_scheduled_tasks") return Promise.resolve(["fwupd-refresh.timer"]);
    return Promise.resolve(null);
  }),
}));

describe("ProcessesPage", () => {
  it("shows processes, services, autostart entries, and scheduled tasks", async () => {
    const wrapper = mount(ProcessesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux"));
    expect(wrapper.text()).toContain("ssh.service");
    expect(wrapper.text()).toContain("nm-applet.desktop");
    expect(wrapper.text()).toContain("fwupd-refresh.timer");
  });

  it("filters the process list by name", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_processes") return Promise.resolve([
        { pid: 1, name: "nitrux", cpu_percent: 1, memory_bytes: 1000 },
        { pid: 2, name: "bash", cpu_percent: 0, memory_bytes: 500 },
      ]);
      return Promise.resolve([]);
    });
    const wrapper = mount(ProcessesPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("nitrux"));
    const filterInput = wrapper.find("input");
    await filterInput.setValue("bash");
    expect(wrapper.text()).toContain("bash");
    expect(wrapper.text()).not.toContain("nitrux");
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `ProcessesPage.vue`**

```vue
<!-- src/pages/ProcessesPage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface ProcessInfo { pid: number; name: string; cpu_percent: number; memory_bytes: number }
interface AutostartEntry { name: string }

const processes = ref<ProcessInfo[]>([]);
const services = ref<string[]>([]);
const autostart = ref<AutostartEntry[]>([]);
const scheduledTasks = ref<string[]>([]);
const processFilter = ref("");

onMounted(async () => {
  processes.value = await invoke<ProcessInfo[]>("get_processes");
  services.value = await invoke<string[]>("get_systemd_services");
  autostart.value = await invoke<AutostartEntry[]>("get_autostart_entries");
  scheduledTasks.value = await invoke<string[]>("get_scheduled_tasks");
});

const filteredProcesses = computed(() =>
  processes.value.filter((p) => p.name.toLowerCase().includes(processFilter.value.toLowerCase())),
);

function bytesToMb(bytes: number): string {
  return (bytes / 1024 / 1024).toFixed(1);
}
</script>

<template>
  <div class="proc-page">
    <NxSectionHeader title="Processus & services" description="Processus en cours, services, démarrage automatique et tâches planifiées." />

    <NxCard>
      <NxSectionHeader title="Processus" />
      <NxInput v-model="processFilter" placeholder="Filtrer par nom..." />
      <div v-for="p in filteredProcesses" :key="p.pid" class="proc-row">
        <span>{{ p.name }} ({{ p.pid }})</span>
        <span>{{ p.cpu_percent.toFixed(1) }}% · {{ bytesToMb(p.memory_bytes) }} MB</span>
      </div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Services systemd" />
      <div v-for="s in services" :key="s" class="proc-row">{{ s }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Démarrage automatique" />
      <div v-if="autostart.length === 0" class="proc-empty">Aucune entrée de démarrage automatique.</div>
      <div v-for="a in autostart" :key="a.name" class="proc-row">{{ a.name }}</div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Tâches planifiées" />
      <div v-if="scheduledTasks.length === 0" class="proc-empty">Aucune tâche planifiée.</div>
      <div v-for="t in scheduledTasks" :key="t" class="proc-row">{{ t }}</div>
    </NxCard>
  </div>
</template>

<style scoped>
.proc-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.proc-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; }
.proc-empty { color: var(--nx-text-secondary); font-size: 13px; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit**

```bash
git add src/pages/ProcessesPage.vue src/pages/ProcessesPage.spec.ts
git commit -m "feat: add ProcessesPage (spec section 5)"
```

---

## Task 4: `get_environment_variables` + `InstalledSoftwarePage.vue`

**Files:**
- Modify: `src-tauri/src/lib.rs` (add one small command directly, no new module -- see below)
- Create: `src/pages/InstalledSoftwarePage.vue`
- Test: `src/pages/InstalledSoftwarePage.spec.ts`

`list_installed_packages` **already exists** (registered in `lib.rs`, currently consumed by `UninstallerPage.vue`) and already returns `Vec<InstalledPackage>` (`{ name, version }`) via whichever native manager `packages::detect_package_managers()` finds -- confirmed on the dev VM: `dpkg -l` parsing already works, returning 2246 real packages. This task adds **zero** new backend command for the software list itself, only a trivial new one for environment variables, and a new frontend page with search (2246 unfiltered rows would be unusable without one, same reasoning already established for `QuickInstallPage`/`FileToolsPage`'s search boxes).

### Backend

- [ ] **Step 1: Add `get_environment_variables` directly in `lib.rs`**, right after the existing `detect_native_manager` function:

```rust
/// Environment variables of the app's own process, i.e. exactly what the
/// invoking user's shell environment looked like when NiTruX was launched
/// -- a normal, expected diagnostic view of one's own environment, nothing
/// exposed that the user couldn't already see in their own shell.
#[tauri::command]
fn get_environment_variables() -> Vec<(String, String)> {
    std::env::vars().collect()
}
```

- [ ] **Step 2: Register `get_environment_variables,` in the `invoke_handler!` list**, right after `detect_native_manager,`.

- [ ] **Step 3: Run the full Rust suite, expect `201 passed; 0 failed; 1 ignored`** (unchanged from Task 3 -- this function has no dedicated test since it's a one-line wrapper over `std::env::vars()`, matching the existing precedent in this codebase of not writing a test for a trivial direct pass-through with no branching logic, e.g. `detect_native_manager` itself has none).

- [ ] **Step 4: Commit the backend**

```bash
git add src-tauri/src/lib.rs
git commit -m "feat: add get_environment_variables (spec section 6)"
```

### Frontend

- [ ] **Step 5: Write the failing frontend test**

```typescript
// src/pages/InstalledSoftwarePage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import InstalledSoftwarePage from "./InstalledSoftwarePage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "list_installed_packages") {
      return Promise.resolve([
        { name: "firefox", version: "128.0" },
        { name: "vlc", version: "3.0.20" },
      ]);
    }
    if (cmd === "get_environment_variables") {
      return Promise.resolve([["HOME", "/home/dev"], ["SHELL", "/bin/bash"]]);
    }
    return Promise.resolve(null);
  }),
}));

describe("InstalledSoftwarePage", () => {
  it("lists installed software with version", async () => {
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("firefox"));
    expect(wrapper.text()).toContain("128.0");
  });

  it("filters the software list by name", async () => {
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("firefox"));
    const filterInput = wrapper.findAll("input")[0];
    await filterInput.setValue("vlc");
    expect(wrapper.text()).toContain("vlc");
    expect(wrapper.text()).not.toContain("firefox");
  });

  it("shows environment variables", async () => {
    const wrapper = mount(InstalledSoftwarePage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("HOME"));
    expect(wrapper.text()).toContain("/home/dev");
  });
});
```

- [ ] **Step 6: Run it, confirm it FAILS**

- [ ] **Step 7: Write `InstalledSoftwarePage.vue`**

```vue
<!-- src/pages/InstalledSoftwarePage.vue -->
<script setup lang="ts">
import { ref, computed, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxInput from "@/components/ui/NxInput.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface InstalledPackage { name: string; version: string }

const packages = ref<InstalledPackage[]>([]);
const envVars = ref<[string, string][]>([]);
const softwareFilter = ref("");

onMounted(async () => {
  packages.value = await invoke<InstalledPackage[]>("list_installed_packages");
  envVars.value = await invoke<[string, string][]>("get_environment_variables");
});

const filteredPackages = computed(() =>
  packages.value.filter((p) => p.name.toLowerCase().includes(softwareFilter.value.toLowerCase())),
);
</script>

<template>
  <div class="sw-page">
    <NxSectionHeader title="Logiciels installés" description="Inventaire complet des paquets installés et variables d'environnement." />

    <NxCard>
      <NxSectionHeader :title="`Paquets (${packages.length})`" />
      <NxInput v-model="softwareFilter" placeholder="Filtrer par nom..." />
      <div v-for="p in filteredPackages" :key="p.name" class="sw-row">
        <span>{{ p.name }}</span><span>{{ p.version }}</span>
      </div>
    </NxCard>

    <NxCard>
      <NxSectionHeader title="Variables d'environnement" />
      <div v-for="[key, value] in envVars" :key="key" class="sw-row">
        <span>{{ key }}</span><span>{{ value }}</span>
      </div>
    </NxCard>
  </div>
</template>

<style scoped>
.sw-page { padding: 24px; display: flex; flex-direction: column; gap: 12px; }
.sw-row { display: flex; justify-content: space-between; align-items: center; gap: 10px; padding: 4px 0; font-size: 13px; border-bottom: 1px solid var(--nx-style-border-color); }
</style>
```

- [ ] **Step 8: Run it, confirm it PASSES (3 tests)**

- [ ] **Step 9: Commit**

```bash
git add src/pages/InstalledSoftwarePage.vue src/pages/InstalledSoftwarePage.spec.ts
git commit -m "feat: add InstalledSoftwarePage — reuses list_installed_packages, adds env vars (spec section 6)"
```

---

## Task 5: `accounts.rs` + `UserAccountsPage.vue`

**Files:**
- Create: `src-tauri/src/accounts.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/UserAccountsPage.vue`
- Test: `src/pages/UserAccountsPage.spec.ts`

Real `/etc/passwd` on the dev VM confirmed exactly one real account in the `uid >= 1000 && uid < 60000` range: `dev` (uid 1000, home `/home/dev`, shell `/bin/bash`) -- this plan's test fixture below mirrors that real shape plus a synthetic second entry to exercise filtering against system accounts (uid 0/999 excluded).

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/accounts.rs
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct UserAccount {
    pub username: String,
    pub uid: u32,
    pub home: String,
    pub shell: String,
}

/// Parses one `/etc/passwd` line (colon-separated: username, password
/// placeholder, uid, gid, gecos, home, shell). Returns `None` for a
/// malformed line (wrong field count) or a non-numeric uid, rather than
/// panicking -- a hand-edited or unusual passwd file should degrade to
/// "skip this line", never crash the whole listing.
pub fn parse_passwd_line(line: &str) -> Option<UserAccount> {
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() < 7 {
        return None;
    }
    let uid: u32 = fields[2].parse().ok()?;
    Some(UserAccount {
        username: fields[0].to_string(),
        uid,
        home: fields[5].to_string(),
        shell: fields[6].to_string(),
    })
}

/// True for a "real" user account, as opposed to a system/service account
/// -- the standard Linux convention (also used by tools like `useradd`'s
/// own defaults): UIDs below 1000 are system accounts (root, daemon
/// users, ...), and UIDs at/above 60000 are typically nobody/nfsnobody-
/// style reserved ranges, not real people either.
pub fn is_real_user_account(account: &UserAccount) -> bool {
    account.uid >= 1000 && account.uid < 60000
}

#[tauri::command]
pub fn get_user_accounts() -> Vec<UserAccount> {
    std::fs::read_to_string("/etc/passwd")
        .ok()
        .map(|content| {
            content
                .lines()
                .filter_map(parse_passwd_line)
                .filter(is_real_user_account)
                .collect()
        })
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_real_passwd_line() {
        let line = "dev:x:1000:1000:dev,,,:/home/dev:/bin/bash";
        let account = parse_passwd_line(line).expect("should parse");
        assert_eq!(account.username, "dev");
        assert_eq!(account.uid, 1000);
        assert_eq!(account.home, "/home/dev");
        assert_eq!(account.shell, "/bin/bash");
    }

    #[test]
    fn returns_none_for_a_malformed_line() {
        assert!(parse_passwd_line("not:enough:fields").is_none());
    }

    #[test]
    fn is_real_user_account_excludes_system_accounts() {
        let root = UserAccount { username: "root".to_string(), uid: 0, home: "/root".to_string(), shell: "/bin/bash".to_string() };
        let daemon = UserAccount { username: "daemon".to_string(), uid: 999, home: "/nonexistent".to_string(), shell: "/usr/sbin/nologin".to_string() };
        assert!(!is_real_user_account(&root));
        assert!(!is_real_user_account(&daemon));
    }

    #[test]
    fn is_real_user_account_includes_a_normal_user() {
        let dev = UserAccount { username: "dev".to_string(), uid: 1000, home: "/home/dev".to_string(), shell: "/bin/bash".to_string() };
        assert!(is_real_user_account(&dev));
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile.**

- [ ] **Step 3: Register `mod accounts;` in `lib.rs`** (alphabetically — becomes the FIRST `mod` line, right before `mod backup;`) and add `accounts::get_user_accounts,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `205 passed; 0 failed; 1 ignored`** (201 Task 4 + 4 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/accounts.rs src-tauri/src/lib.rs
git commit -m "feat: add accounts.rs — real user accounts from /etc/passwd, read-only (spec section 7)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/UserAccountsPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import UserAccountsPage from "./UserAccountsPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_user_accounts") {
      return Promise.resolve([{ username: "dev", uid: 1000, home: "/home/dev", shell: "/bin/bash" }]);
    }
    return Promise.resolve(null);
  }),
}));

describe("UserAccountsPage", () => {
  it("lists real user accounts", async () => {
    const wrapper = mount(UserAccountsPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("dev"));
    expect(wrapper.text()).toContain("/home/dev");
    expect(wrapper.text()).toContain("/bin/bash");
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `UserAccountsPage.vue`**

```vue
<!-- src/pages/UserAccountsPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface UserAccount { username: string; uid: number; home: string; shell: string }

const accounts = ref<UserAccount[]>([]);

onMounted(async () => {
  accounts.value = await invoke<UserAccount[]>("get_user_accounts");
});
</script>

<template>
  <div class="ua-page">
    <NxSectionHeader title="Comptes utilisateurs" description="Comptes réels du système (lecture seule)." />
    <NxCard v-for="a in accounts" :key="a.username" class="ua-row">
      <strong>{{ a.username }}</strong> (UID {{ a.uid }}) — {{ a.home }} — {{ a.shell }}
    </NxCard>
  </div>
</template>

<style scoped>
.ua-page { padding: 24px; display: flex; flex-direction: column; gap: 10px; }
.ua-row { font-size: 13px; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (1 test)**

- [ ] **Step 10: Commit**

```bash
git add src/pages/UserAccountsPage.vue src/pages/UserAccountsPage.spec.ts
git commit -m "feat: add UserAccountsPage (spec section 7)"
```

---

## Task 6: `update_history.rs` + `UpdateHistoryPage.vue`

**Files:**
- Create: `src-tauri/src/update_history.rs`
- Modify: `src-tauri/src/lib.rs`
- Create: `src/pages/UpdateHistoryPage.vue`
- Test: `src/pages/UpdateHistoryPage.spec.ts`

Real `/var/log/apt/history.log` confirmed present (109 KB) and readable without root on the dev VM. v1 supports the APT log format only (see design spec §8) -- other managers show a clear "not available" message rather than a guessed parse.

### Backend

- [ ] **Step 1: Write the failing Rust tests**

```rust
// src-tauri/src/update_history.rs
use serde::Serialize;

#[derive(Serialize, Clone)]
pub struct UpdateHistoryEntry {
    pub start_date: String,
    pub commandline: String,
    /// Combined summary of Install:/Upgrade:/Remove: lines for this block,
    /// whichever were present -- not split further per-package, this is a
    /// diagnostic overview, not a full per-package audit trail.
    pub summary: String,
}

/// Parses `/var/log/apt/history.log` content, which is a sequence of
/// blank-line-separated blocks each starting with "Start-Date:" and
/// containing "Commandline:" plus one or more of
/// "Install:"/"Upgrade:"/"Remove:", ending with "End-Date:". Malformed or
/// incomplete blocks (missing Start-Date or Commandline) are skipped
/// rather than producing a partial/garbage entry.
pub fn parse_apt_history(content: &str) -> Vec<UpdateHistoryEntry> {
    let mut entries = Vec::new();
    for block in content.split("\n\n") {
        let mut start_date = None;
        let mut commandline = None;
        let mut summary_parts = Vec::new();
        for line in block.lines() {
            if let Some(v) = line.strip_prefix("Start-Date: ") {
                start_date = Some(v.to_string());
            } else if let Some(v) = line.strip_prefix("Commandline: ") {
                commandline = Some(v.to_string());
            } else if line.starts_with("Install: ") || line.starts_with("Upgrade: ") || line.starts_with("Remove: ") {
                summary_parts.push(line.to_string());
            }
        }
        if let (Some(start_date), Some(commandline)) = (start_date, commandline) {
            entries.push(UpdateHistoryEntry { start_date, commandline, summary: summary_parts.join(" | ") });
        }
    }
    entries
}

#[tauri::command]
pub fn get_update_history() -> Result<Vec<UpdateHistoryEntry>, String> {
    if !std::path::Path::new("/var/log/apt/history.log").exists() {
        return Err("historique non disponible pour ce gestionnaire de paquets".to_string());
    }
    let content = std::fs::read_to_string("/var/log/apt/history.log")
        .map_err(|e| format!("impossible de lire l'historique : {e}"))?;
    let mut entries = parse_apt_history(&content);
    entries.reverse(); // most recent first
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_well_formed_apt_history_block() {
        let content = "Start-Date: 2026-08-02  01:48:01\nCommandline: apt-get install -y flatpak\nInstall: flatpak:amd64 (1.16.6-1~deb13u1)\nEnd-Date: 2026-08-02  01:48:05\n";
        let entries = parse_apt_history(content);
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].start_date, "2026-08-02  01:48:01");
        assert_eq!(entries[0].commandline, "apt-get install -y flatpak");
        assert!(entries[0].summary.contains("flatpak"));
    }

    #[test]
    fn parses_multiple_blocks_separated_by_blank_lines() {
        let content = "Start-Date: 2026-08-01\nCommandline: apt-get update\nEnd-Date: 2026-08-01\n\nStart-Date: 2026-08-02\nCommandline: apt-get install -y snapd\nInstall: snapd:amd64 (2.68.3)\nEnd-Date: 2026-08-02\n";
        let entries = parse_apt_history(content);
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn skips_a_block_missing_commandline() {
        let content = "Start-Date: 2026-08-01\nEnd-Date: 2026-08-01\n";
        let entries = parse_apt_history(content);
        assert!(entries.is_empty());
    }
}
```

- [ ] **Step 2: Run tests, confirm they fail to compile.**

- [ ] **Step 3: Register `mod update_history;` in `lib.rs`** (alphabetically — sorts after `mod trash;`, becomes the LAST `mod` line) and add `update_history::get_update_history,` to the handler list.

- [ ] **Step 4: Run the full Rust suite, expect `208 passed; 0 failed; 1 ignored`** (205 Task 5 + 3 new).

- [ ] **Step 5: Commit the backend**

```bash
git add src-tauri/src/update_history.rs src-tauri/src/lib.rs
git commit -m "feat: add update_history.rs — APT history log parsing (spec section 8)"
```

### Frontend

- [ ] **Step 6: Write the failing frontend test**

```typescript
// src/pages/UpdateHistoryPage.spec.ts
import { describe, it, expect, vi } from "vitest";
import { mount } from "@vue/test-utils";
import UpdateHistoryPage from "./UpdateHistoryPage.vue";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn((cmd: string) => {
    if (cmd === "get_update_history") {
      return Promise.resolve([
        { start_date: "2026-08-02  01:48:01", commandline: "apt-get install -y flatpak", summary: "Install: flatpak:amd64 (1.16.6-1~deb13u1)" },
      ]);
    }
    return Promise.resolve(null);
  }),
}));

describe("UpdateHistoryPage", () => {
  it("lists past update history entries", async () => {
    const wrapper = mount(UpdateHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("apt-get install -y flatpak"));
    expect(wrapper.text()).toContain("2026-08-02");
    expect(wrapper.text()).toContain("flatpak");
  });

  it("shows a clear message when history is unavailable for this package manager", async () => {
    const { invoke } = await import("@tauri-apps/api/core");
    (invoke as ReturnType<typeof vi.fn>).mockImplementation((cmd: string) => {
      if (cmd === "get_update_history") return Promise.reject("historique non disponible pour ce gestionnaire de paquets");
      return Promise.resolve(null);
    });
    const wrapper = mount(UpdateHistoryPage);
    await vi.waitFor(() => expect(wrapper.text()).toContain("non disponible"));
  });
});
```

- [ ] **Step 7: Run it, confirm it FAILS**

- [ ] **Step 8: Write `UpdateHistoryPage.vue`**

```vue
<!-- src/pages/UpdateHistoryPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";
import NxCard from "@/components/ui/NxCard.vue";
import NxSectionHeader from "@/components/ui/NxSectionHeader.vue";

interface UpdateHistoryEntry { start_date: string; commandline: string; summary: string }

const entries = ref<UpdateHistoryEntry[] | null>(null);
const error = ref<string | null>(null);

onMounted(async () => {
  try {
    entries.value = await invoke<UpdateHistoryEntry[]>("get_update_history");
  } catch (e) {
    error.value = String(e);
  }
});
</script>

<template>
  <div class="uh-page">
    <NxSectionHeader title="Historique des mises à jour" description="Journal des installations et mises à jour passées." />
    <NxCard v-if="error" danger>{{ error }}</NxCard>
    <NxCard v-for="e in entries ?? []" :key="e.start_date" class="uh-row">
      <div class="uh-date">{{ e.start_date }}</div>
      <div class="uh-cmd">{{ e.commandline }}</div>
      <div class="uh-summary">{{ e.summary }}</div>
    </NxCard>
  </div>
</template>

<style scoped>
.uh-page { padding: 24px; display: flex; flex-direction: column; gap: 10px; }
.uh-row { font-size: 13px; display: flex; flex-direction: column; gap: 4px; }
.uh-date { color: var(--nx-text-secondary); font-size: 11px; }
.uh-cmd { font-weight: 600; }
.uh-summary { color: var(--nx-text-secondary); font-size: 12px; }
</style>
```

- [ ] **Step 9: Run it, confirm it PASSES (2 tests)**

- [ ] **Step 10: Commit**

```bash
git add src/pages/UpdateHistoryPage.vue src/pages/UpdateHistoryPage.spec.ts
git commit -m "feat: add UpdateHistoryPage (spec section 8)"
```

---

## Task 7: New "Diagnostic" navigation category

**Files:**
- Modify: `src/navigation/categories.ts`
- Modify: `src/navigation/categories.spec.ts`
- Modify: `src/components/nav/AppNav.vue`
- Modify: `src/App.vue`
- Modify: `src/App.spec.ts`

- [ ] **Step 1: Read the live `src/navigation/categories.ts`** (reproduced in this plan's research above). Replace the `"systeme"` category's `pages` array (currently `dashboard` + `diagnostic`) and insert a new category right after it:

```typescript
  {
    id: "systeme",
    title: "Système",
    pages: [
      { id: "dashboard", label: "Tableau de bord", icon: "layout-dashboard" },
    ],
  },
  {
    id: "diagnostic-avance",
    title: "Diagnostic",
    pages: [
      { id: "diagnostic", label: "Composants PCI", icon: "stethoscope" },
      { id: "hardware-details", label: "Matériel détaillé", icon: "cpu" },
      { id: "peripherals", label: "Périphériques", icon: "monitor" },
      { id: "processes", label: "Processus & services", icon: "activity" },
      { id: "installed-software", label: "Logiciels installés", icon: "list" },
      { id: "user-accounts", label: "Comptes utilisateurs", icon: "users" },
      { id: "update-history", label: "Historique des mises à jour", icon: "history" },
    ],
  },
```

- [ ] **Step 2: Add the 5 new icon names to `AppNav.vue`'s `iconMap`** (`cpu`/`stethoscope` already exist). Add to the `lucide-vue-next` import:
```typescript
  Monitor, Activity, List, Users, History,
```
And to `iconMap`:
```typescript
  monitor: Monitor,
  activity: Activity,
  list: List,
  users: Users,
  history: History,
```

- [ ] **Step 3: Update `categories.spec.ts`** (category count goes from 8 to 9 -- find and update the existing `"has exactly 8 categories"` test to 9):
```typescript
  it("has exactly 9 categories", () => {
    expect(navigationCategories).toHaveLength(9);
  });
```
Add a new test:
```typescript
  it("includes the 6 new Phase R11 Diagnostic pages by id", () => {
    const allPageIds = navigationCategories.flatMap((c) => c.pages.map((p) => p.id));
    expect(allPageIds).toContain("hardware-details");
    expect(allPageIds).toContain("peripherals");
    expect(allPageIds).toContain("processes");
    expect(allPageIds).toContain("installed-software");
    expect(allPageIds).toContain("user-accounts");
    expect(allPageIds).toContain("update-history");
  });
```

- [ ] **Step 4: Read the live `src/App.vue` and `src/App.spec.ts`.** Add 5 new page imports (`DiagnosticPage` is already imported, unchanged) and map entries, and 6 new `App.spec.ts` tests:

```typescript
import HardwareDetailsPage from "@/pages/HardwareDetailsPage.vue";
import PeripheralsPage from "@/pages/PeripheralsPage.vue";
import ProcessesPage from "@/pages/ProcessesPage.vue";
import InstalledSoftwarePage from "@/pages/InstalledSoftwarePage.vue";
import UserAccountsPage from "@/pages/UserAccountsPage.vue";
import UpdateHistoryPage from "@/pages/UpdateHistoryPage.vue";
```
```typescript
  "hardware-details": HardwareDetailsPage,
  peripherals: PeripheralsPage,
  processes: ProcessesPage,
  "installed-software": InstalledSoftwarePage,
  "user-accounts": UserAccountsPage,
  "update-history": UpdateHistoryPage,
```

```typescript
  it("shows the real HardwareDetailsPage for the hardware-details id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Matériel détaillé")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Processeur");
  });

  it("shows the real PeripheralsPage for the peripherals id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Périphériques")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Moniteurs");
  });

  it("shows the real ProcessesPage for the processes id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Processus & services")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Démarrage automatique");
  });

  it("shows the real InstalledSoftwarePage for the installed-software id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Logiciels installés")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Variables d'environnement");
  });

  it("shows the real UserAccountsPage for the user-accounts id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Comptes utilisateurs")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Comptes réels du système");
  });

  it("shows the real UpdateHistoryPage for the update-history id", async () => {
    const wrapper = mount(App);
    const buttons = wrapper.findAll("button");
    const button = buttons.find((b) => b.text() === "Historique des mises à jour")!;
    await button.trigger("click");
    expect(wrapper.text()).toContain("Journal des installations");
  });
```

- [ ] **Step 5: Run `npx vitest run src/App.spec.ts src/navigation/categories.spec.ts src/components/nav/AppNav.spec.ts`, confirm all pass.**

- [ ] **Step 6: Commit**

```bash
git add src/navigation/categories.ts src/navigation/categories.spec.ts src/components/nav/AppNav.vue src/App.vue src/App.spec.ts
git commit -m "feat: add Diagnostic navigation category, move Composants PCI into it (spec section 2)"
```

---

## Task 8: Full verification pass — frontend, backend, and live VM check

**Files:** None (verification-only).

- [ ] **Step 1: Run the full frontend test suite.**

Expected baseline entering this plan: 198 (end of R10). Net delta: Task 1 (+2: `HardwareDetailsPage.spec.ts`) + Task 2 (+2) + Task 3 (+2) + Task 4 (+3) + Task 5 (+1) + Task 6 (+2) + Task 7 (+1 `categories.spec.ts` count-update is a modification not an addition, +1 new categories test, +6 `App.spec.ts`) = 2+2+2+3+1+2+7 = 19, expected total **217**. Report the real observed total and reconcile if it differs -- do not trust this arithmetic blindly, verify it (this exact kind of count has been wrong in prior phases' plan text before, always double-check against the real run).

- [ ] **Step 2: Type-check.** `npx vue-tsc --noEmit`, expect clean.

- [ ] **Step 3: Confirm the Rust suite.** Expect `208 passed; 0 failed; 1 ignored` (188 R10 baseline + 1 Task 1 subprocess test + 5 Task 1 hardware_details + 4 Task 2 + 3 Task 3 + 0 Task 4 + 4 Task 5 + 3 Task 6).

- [ ] **Step 4: Build and install on the VM, verify against real output.**

Build: `wsl.exe -e bash -lc "cd /mnt/c/Users/Momo/Desktop/NiTruX/.worktrees/r11-diagnostic-config && npx tauri build 2>&1 | tail -30"`.

None of this task's additions are privileged -- no `pkexec` involved, no live pkexec cycle needed (unlike R10's Snap task). Deploy the `.deb` to the VM and verify by reasoning through the exact commands each new backend function runs (same "mirror the real sequence over SSH" method used successfully in R9 Task 6 and R10 Task 6, given screenshot capture remains impossible on this VM per the standing documented limitation):

1. `LC_ALL=C lscpu` -- confirm the English field labels this plan's parser expects.
2. `cat /sys/class/dmi/id/product_name /sys/class/dmi/id/bios_version` -- confirm still readable.
3. `lsusb`, `pactl list short sinks`, `xrandr --query`, `lpstat -p` -- confirm each still runs (a `lpstat` failure here is expected/fine per §4 of the design spec, confirm it degrades to an empty list rather than erroring the whole page).
4. `systemctl list-units --type=service --no-pager --plain | head`, `systemctl --user list-unit-files --state=enabled`, `crontab -l`, `systemctl list-timers --no-pager --plain` -- confirm still runs.
5. `dpkg -l | head` -- confirm still runs (already exercised by `list_installed_packages`, which R10's Task 6 didn't re-verify since it wasn't new there; worth a fresh confirmation here since this is the first NEW page to depend on it).
6. `cat /var/log/apt/history.log | head -20` -- confirm the real log still matches the `Start-Date:`/`Commandline:`/`Install:`/`End-Date:` block shape this plan's parser expects.

- [ ] **Step 5: Commit any final cleanup.** No further commit expected if Steps 1-4 all pass clean.

---

## Self-Review

**Spec coverage:** §2 (nav category) — Task 7. §3 (Matériel détaillé) — Task 1. §4 (Périphériques) — Task 2. §5 (Processus & services) — Task 3. §6 (Logiciels installés) — Task 4. §7 (Comptes utilisateurs) — Task 5. §8 (Historique des mises à jour) — Task 6. §9 (aucune surface privilégiée) — confirmed no task in this plan touches `pkexec`/`nitrux-pkexec-helper`. §10 (hors scope) — confirmed no task builds Licence/Registre/BSOD/WSL/Certificats/dossiers.

**Placeholder scan:** No "TBD"/"TODO". The APT-only limitation of `update_history.rs` (Task 6) and CUPS/printer graceful-empty behavior (Task 2) are both explicitly justified design decisions carried over from the spec, not oversights.

**Type consistency:** `HardwareDetails`/`CpuDetails`/`BoardDetails`/`MemoryDetails` (Task 1), `AudioSink`/`PrinterInfo` (Task 2), `ProcessInfo`/`AutostartEntry` (Task 3), `UserAccount` (Task 5), `UpdateHistoryEntry` (Task 6) all match their respective frontend TypeScript interfaces field-for-field. `InstalledPackage` (Task 4) is reused verbatim from the already-existing `UninstallerPage.vue`'s own local interface definition, not redefined differently. Every template in this plan has been hand-traced against its own test assertions before being finalized, per the discipline established in R8 and reused successfully through R9/R10.

**Cross-task consistency check:** Task 3's `get_processes` reuses the existing `Mutex<System>` managed state from `system.rs` rather than constructing a fresh `System` -- confirmed by reading `lib.rs`'s `.manage(Mutex::new(System::new_all()))` call during this plan's research, so this is not a guess. Task 4 deliberately adds zero new Rust module, confirmed by reading `UninstallerPage.vue`'s existing `list_installed_packages` usage during research rather than assuming a new command was needed.
