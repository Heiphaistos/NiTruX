# NiTruX Phase 4 Part 1 — Network Diagnostics (read-only) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the "Réseau, sécurité & maintenance" pillar's read-only network diagnostics: Wi-Fi status, local listening ports, DNS configuration, `/etc/hosts` contents (all aggregated into one snapshot, mirroring `system.rs`'s `SystemSnapshot` pattern from Phase 1), a bounded local TCP port scanner, and read-only Docker container/image listing — exposed via a new `NetworkPage.vue`.

**Architecture:** `network.rs` aggregates four cheap read-only queries (Wi-Fi, listening ports, DNS, hosts file) into one `NetworkSnapshot` struct behind a single command, exactly mirroring `system.rs`'s `SystemSnapshot`/`get_system_snapshot` shape from Phase 1. `portscan.rs` and `docker.rs` are separate modules since they're distinct, heavier-weight operations (a scan takes real time; Docker may not be installed) that shouldn't block the cheap snapshot. All subprocess calls go through the established `subprocess::run_with_timeout` + `Result<T, String>` convention.

**Tech Stack:** Same as Phase 1/2/3 — Tauri v2, Rust, Vue 3 + TypeScript + Pinia, Vitest, `cargo test`.

**Scope note — READ-ONLY by design, intentional, same reasoning as Phase 2/3:** Editing `/etc/hosts`, changing DNS servers, modifying the firewall (UFW), and the design spec's "bouton de dépannage" (restart NetworkManager/PipeWire, etc.) are all explicitly OUT of scope for this plan — they're either privileged writes (polkit/pkexec, human-reviewed design required) or destructive-adjacent (a bad DNS/hosts write can break network access entirely). This plan covers only diagnostics: reading current state and a bounded, safe local port scan.

**Security note on the port scanner:** `portscan.rs` implements a bounded TCP-connect scan (a standard, well-understood system-administration diagnostic technique — the same category of tool as `nmap`, which the design spec's own `entry-point-analyzer`/dual-use-tooling guidance treats as legitimate for a user's own system administration app). To keep it unambiguously safe and non-abusable: the scan targets a caller-specified host (defaulting to `127.0.0.1` in the frontend), is capped to a bounded, small port list (not an unbounded range), uses a short per-port connect timeout, and the whole operation itself is wrapped in an overall timeout. This is a diagnostic tool for the user's own machine/network, not a mass-scanning or exploitation tool — no code in this task probes for vulnerabilities, brute-forces credentials, or targets anything beyond what the user explicitly enters.

---

## File Structure

```
src-tauri/src/
├── network.rs      # NetworkSnapshot: wifi status + listening ports + DNS + /etc/hosts (all read-only)
├── portscan.rs       # Bounded TCP-connect port scanner
└── docker.rs            # Read-only `docker ps`/`docker images` listing
src/
└── pages/
    └── NetworkPage.vue   # Snapshot view + port scanner + Docker tab
```

---

## Task 1: Network snapshot (Wi-Fi + listening ports + DNS + hosts file)

**Files:**
- Create: `src-tauri/src/network.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/network.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nmcli_wifi_line_into_network() {
        let line = "*:MyHomeWifi:WPA2:78";
        let net = parse_nmcli_wifi_line(line).expect("should parse");
        assert_eq!(net.ssid, "MyHomeWifi");
        assert_eq!(net.security, "WPA2");
        assert_eq!(net.signal_percent, 78);
        assert!(net.connected);
    }

    #[test]
    fn parses_disconnected_nmcli_wifi_line() {
        let line = ":OtherNetwork:WPA2:45";
        let net = parse_nmcli_wifi_line(line).expect("should parse");
        assert!(!net.connected);
    }

    #[test]
    fn skips_malformed_nmcli_line() {
        assert!(parse_nmcli_wifi_line("not:enough").is_none());
    }

    #[test]
    fn parses_ss_listening_port_line() {
        let line = "LISTEN 0      128          0.0.0.0:22         0.0.0.0:*    users:((\"sshd\",pid=1234,fd=3))";
        let port = parse_ss_line(line).expect("should parse");
        assert_eq!(port.port, 22);
        assert_eq!(port.process.as_deref(), Some("sshd"));
    }

    #[test]
    fn skips_ss_header_line() {
        assert!(parse_ss_line("Netid  State   Recv-Q  Send-Q   Local Address:Port").is_none());
    }

    #[test]
    fn parses_resolv_conf_nameserver_line() {
        let content = "# comment\nnameserver 8.8.8.8\nnameserver 1.1.1.1\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()]);
    }

    #[test]
    fn ignores_non_nameserver_resolv_conf_lines() {
        let content = "search example.com\noptions rotate\n";
        assert!(parse_resolv_conf(content).is_empty());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test network:: 2>&1 | tail -40`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `network.rs`**

```rust
// src-tauri/src/network.rs
use crate::subprocess;
use serde::Serialize;
use std::fs;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct WifiNetwork {
    pub ssid: String,
    pub security: String,
    pub signal_percent: u8,
    pub connected: bool,
}

#[derive(Serialize, Clone)]
pub struct ListeningPort {
    pub port: u16,
    pub process: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct NetworkSnapshot {
    pub wifi_networks: Vec<WifiNetwork>,
    pub listening_ports: Vec<ListeningPort>,
    pub dns_servers: Vec<String>,
    pub hosts_file: String,
}

/// Parses one line of `nmcli -t -f IN-USE,SSID,SECURITY,SIGNAL dev wifi`
/// output, e.g. "*:MyHomeWifi:WPA2:78" (connected) or ":OtherNetwork:WPA2:45".
pub fn parse_nmcli_wifi_line(line: &str) -> Option<WifiNetwork> {
    let fields: Vec<&str> = line.split(':').collect();
    if fields.len() != 4 {
        return None;
    }
    Some(WifiNetwork {
        connected: fields[0] == "*",
        ssid: fields[1].to_string(),
        security: fields[2].to_string(),
        signal_percent: fields[3].parse().ok()?,
    })
}

/// Parses one line of `ss -tulnp` output for the local port and owning
/// process name, e.g.:
/// "LISTEN 0  128  0.0.0.0:22  0.0.0.0:*  users:((\"sshd\",pid=1234,fd=3))"
pub fn parse_ss_line(line: &str) -> Option<ListeningPort> {
    if !line.starts_with("LISTEN") {
        return None;
    }
    let fields: Vec<&str> = line.split_whitespace().collect();
    let local_addr = fields.get(4)?;
    let port: u16 = local_addr.rsplit(':').next()?.parse().ok()?;
    let process = line
        .find("users:((\"")
        .map(|idx| &line[idx + 9..])
        .and_then(|rest| rest.split('"').next())
        .map(|s| s.to_string());
    Some(ListeningPort { port, process })
}

/// Extracts `nameserver` entries from `/etc/resolv.conf` content, in order.
pub fn parse_resolv_conf(content: &str) -> Vec<String> {
    content
        .lines()
        .filter_map(|line| line.trim().strip_prefix("nameserver "))
        .map(|s| s.trim().to_string())
        .collect()
}

fn read_wifi_networks() -> Vec<WifiNetwork> {
    subprocess::run_with_timeout(
        "nmcli",
        &["-t", "-f", "IN-USE,SSID,SECURITY,SIGNAL", "dev", "wifi"],
        Duration::from_secs(10),
    )
    .map(|output| output.lines().filter_map(parse_nmcli_wifi_line).collect())
    .unwrap_or_default()
}

fn read_listening_ports() -> Vec<ListeningPort> {
    subprocess::run_with_timeout("ss", &["-tulnp"], Duration::from_secs(5))
        .map(|output| output.lines().filter_map(parse_ss_line).collect())
        .unwrap_or_default()
}

fn read_dns_servers() -> Vec<String> {
    fs::read_to_string("/etc/resolv.conf")
        .map(|content| parse_resolv_conf(&content))
        .unwrap_or_default()
}

fn read_hosts_file() -> String {
    fs::read_to_string("/etc/hosts").unwrap_or_default()
}

#[tauri::command]
pub fn get_network_snapshot() -> NetworkSnapshot {
    NetworkSnapshot {
        wifi_networks: read_wifi_networks(),
        listening_ports: read_listening_ports(),
        dns_servers: read_dns_servers(),
        hosts_file: read_hosts_file(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nmcli_wifi_line_into_network() {
        let line = "*:MyHomeWifi:WPA2:78";
        let net = parse_nmcli_wifi_line(line).expect("should parse");
        assert_eq!(net.ssid, "MyHomeWifi");
        assert_eq!(net.security, "WPA2");
        assert_eq!(net.signal_percent, 78);
        assert!(net.connected);
    }

    #[test]
    fn parses_disconnected_nmcli_wifi_line() {
        let line = ":OtherNetwork:WPA2:45";
        let net = parse_nmcli_wifi_line(line).expect("should parse");
        assert!(!net.connected);
    }

    #[test]
    fn skips_malformed_nmcli_line() {
        assert!(parse_nmcli_wifi_line("not:enough").is_none());
    }

    #[test]
    fn parses_ss_listening_port_line() {
        let line = "LISTEN 0      128          0.0.0.0:22         0.0.0.0:*    users:((\"sshd\",pid=1234,fd=3))";
        let port = parse_ss_line(line).expect("should parse");
        assert_eq!(port.port, 22);
        assert_eq!(port.process.as_deref(), Some("sshd"));
    }

    #[test]
    fn skips_ss_header_line() {
        assert!(parse_ss_line("Netid  State   Recv-Q  Send-Q   Local Address:Port").is_none());
    }

    #[test]
    fn parses_resolv_conf_nameserver_line() {
        let content = "# comment\nnameserver 8.8.8.8\nnameserver 1.1.1.1\n";
        let servers = parse_resolv_conf(content);
        assert_eq!(servers, vec!["8.8.8.8".to_string(), "1.1.1.1".to_string()]);
    }

    #[test]
    fn ignores_non_nameserver_resolv_conf_lines() {
        let content = "search example.com\noptions rotate\n";
        assert!(parse_resolv_conf(content).is_empty());
    }
}
```

Note `get_network_snapshot()` is intentionally infallible (`NetworkSnapshot`, not `Result<NetworkSnapshot, String>`) — each of the four sub-queries independently defaults to empty (`unwrap_or_default()`) on failure (e.g. no `nmcli` on a wired-only machine, no wifi hardware, `/etc/hosts` unreadable), matching `packages::universal::list_universal_updates()`'s "optional supplement, never blocks the rest" philosophy from Phase 2 Task 5 — a machine with no Wi-Fi shouldn't make the whole network page unusable.

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test network:: 2>&1 | tail -40`
Expected: PASS (7 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod network;` and `network::get_network_snapshot` to `generate_handler!`, additively alongside all existing Phase 1-3 modules/commands (`disks`, `drivers`, `duplicates`, `hardware`, `hashcheck`, `largefiles`, `logs`, `packages`, `sensors`, `smart`, `subprocess`, `system`).

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 60 (pre-existing) + 7 = 67 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/network.rs src-tauri/src/lib.rs
git commit -m "feat: network snapshot (wifi, listening ports, DNS, hosts file)"
```

---

## Task 2: Bounded local port scanner

**Files:**
- Create: `src-tauri/src/portscan.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/portscan.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn detects_an_open_port_on_localhost() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("should bind");
        let port = listener.local_addr().unwrap().port();

        let results = scan_ports("127.0.0.1", &[port]);
        drop(listener);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, port);
        assert!(results[0].open);
    }

    #[test]
    fn reports_a_closed_port_as_not_open() {
        // Port 1 is a reserved/unassigned port extremely unlikely to have
        // anything listening on it in any test environment.
        let results = scan_ports("127.0.0.1", &[1]);
        assert_eq!(results.len(), 1);
        assert!(!results[0].open);
    }

    #[test]
    fn caps_the_port_list_to_a_safe_maximum() {
        let too_many: Vec<u16> = (1..=2000).collect();
        let err = validate_port_list(&too_many).expect_err("should reject");
        assert!(err.contains("maximum"));
    }

    #[test]
    fn accepts_a_reasonable_port_list() {
        assert!(validate_port_list(&[22, 80, 443]).is_ok());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test portscan:: 2>&1 | tail -40`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `portscan.rs`**

```rust
// src-tauri/src/portscan.rs
use serde::Serialize;
use std::net::{TcpStream, ToSocketAddrs};
use std::time::Duration;

const MAX_PORTS_PER_SCAN: usize = 200;
const CONNECT_TIMEOUT: Duration = Duration::from_millis(300);

#[derive(Serialize, Clone)]
pub struct PortResult {
    pub port: u16,
    pub open: bool,
}

/// Rejects unreasonably large scan requests up front, before any network
/// activity — this is what keeps the scanner "bounded" rather than a
/// potential DoS/abuse vector against the target host.
pub fn validate_port_list(ports: &[u16]) -> Result<(), String> {
    if ports.len() > MAX_PORTS_PER_SCAN {
        return Err(format!(
            "trop de ports demandés ({}), maximum {MAX_PORTS_PER_SCAN} par scan",
            ports.len()
        ));
    }
    Ok(())
}

/// Attempts a short TCP connect to each port in `ports` on `host`,
/// sequentially, each bounded by `CONNECT_TIMEOUT`. Does not distinguish
/// "host unreachable" from "port closed" — both report `open: false`,
/// which is the correct, safe default for a diagnostic tool (no need to
/// leak network topology details beyond open/closed on the requested host).
pub fn scan_ports(host: &str, ports: &[u16]) -> Vec<PortResult> {
    ports
        .iter()
        .map(|&port| {
            let open = format!("{host}:{port}")
                .to_socket_addrs()
                .ok()
                .and_then(|mut addrs| addrs.next())
                .map(|addr| TcpStream::connect_timeout(&addr, CONNECT_TIMEOUT).is_ok())
                .unwrap_or(false);
            PortResult { port, open }
        })
        .collect()
}

#[tauri::command]
pub fn scan_ports_cmd(host: String, ports: Vec<u16>) -> Result<Vec<PortResult>, String> {
    validate_port_list(&ports)?;
    Ok(scan_ports(&host, &ports))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::TcpListener;

    #[test]
    fn detects_an_open_port_on_localhost() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("should bind");
        let port = listener.local_addr().unwrap().port();

        let results = scan_ports("127.0.0.1", &[port]);
        drop(listener);

        assert_eq!(results.len(), 1);
        assert_eq!(results[0].port, port);
        assert!(results[0].open);
    }

    #[test]
    fn reports_a_closed_port_as_not_open() {
        let results = scan_ports("127.0.0.1", &[1]);
        assert_eq!(results.len(), 1);
        assert!(!results[0].open);
    }

    #[test]
    fn caps_the_port_list_to_a_safe_maximum() {
        let too_many: Vec<u16> = (1..=2000).collect();
        let err = validate_port_list(&too_many).expect_err("should reject");
        assert!(err.contains("maximum"));
    }

    #[test]
    fn accepts_a_reasonable_port_list() {
        assert!(validate_port_list(&[22, 80, 443]).is_ok());
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test portscan:: 2>&1 | tail -40`
Expected: PASS (4 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod portscan;` and `portscan::scan_ports_cmd` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 67 + 4 = 71 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/portscan.rs src-tauri/src/lib.rs
git commit -m "feat: bounded local TCP port scanner"
```

---

## Task 3: Read-only Docker listing

**Files:**
- Create: `src-tauri/src/docker.rs`
- Modify: `src-tauri/src/lib.rs`

- [ ] **Step 1: Write the failing tests**

```rust
// src-tauri/src/docker.rs (test module, written first)
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_ps_json_line() {
        let line = r#"{"ID":"a1b2c3d4","Image":"nginx:latest","Names":"web","Status":"Up 2 hours"}"#;
        let c = parse_container_line(line).expect("should parse");
        assert_eq!(c.id, "a1b2c3d4");
        assert_eq!(c.image, "nginx:latest");
        assert_eq!(c.name, "web");
        assert_eq!(c.status, "Up 2 hours");
    }

    #[test]
    fn skips_unparseable_container_line() {
        assert!(parse_container_line("not json").is_none());
    }

    #[test]
    fn parses_docker_images_json_line() {
        let line = r#"{"ID":"e5f6a7b8","Repository":"nginx","Tag":"latest","Size":"142MB"}"#;
        let img = parse_image_line(line).expect("should parse");
        assert_eq!(img.id, "e5f6a7b8");
        assert_eq!(img.repository, "nginx");
        assert_eq!(img.tag, "latest");
        assert_eq!(img.size, "142MB");
    }

    #[test]
    fn skips_unparseable_image_line() {
        assert!(parse_image_line("not json").is_none());
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cd src-tauri && cargo test docker:: 2>&1 | tail -40`
Expected: FAIL — module doesn't exist

- [ ] **Step 3: Implement `docker.rs`**

```rust
// src-tauri/src/docker.rs
use crate::subprocess;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct Container {
    pub id: String,
    pub image: String,
    pub name: String,
    pub status: String,
}

#[derive(Serialize, Clone)]
pub struct DockerImage {
    pub id: String,
    pub repository: String,
    pub tag: String,
    pub size: String,
}

#[derive(Serialize, Clone)]
pub struct DockerSnapshot {
    pub available: bool,
    pub containers: Vec<Container>,
    pub images: Vec<DockerImage>,
}

#[derive(Deserialize)]
struct RawContainer {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Image")]
    image: String,
    #[serde(rename = "Names")]
    names: String,
    #[serde(rename = "Status")]
    status: String,
}

#[derive(Deserialize)]
struct RawImage {
    #[serde(rename = "ID")]
    id: String,
    #[serde(rename = "Repository")]
    repository: String,
    #[serde(rename = "Tag")]
    tag: String,
    #[serde(rename = "Size")]
    size: String,
}

pub fn parse_container_line(line: &str) -> Option<Container> {
    let raw: RawContainer = serde_json::from_str(line).ok()?;
    Some(Container {
        id: raw.id,
        image: raw.image,
        name: raw.names,
        status: raw.status,
    })
}

pub fn parse_image_line(line: &str) -> Option<DockerImage> {
    let raw: RawImage = serde_json::from_str(line).ok()?;
    Some(DockerImage {
        id: raw.id,
        repository: raw.repository,
        tag: raw.tag,
        size: raw.size,
    })
}

/// Infallible by design, same rationale as `network::get_network_snapshot`:
/// Docker not being installed is a normal, common case (not every NiTruX
/// user runs Docker), reflected via `available: false` rather than an error
/// the frontend has to specifically handle.
#[tauri::command]
pub fn get_docker_snapshot() -> DockerSnapshot {
    let containers = subprocess::run_with_timeout(
        "docker",
        &["ps", "-a", "--format", "{{json .}}"],
        Duration::from_secs(10),
    );
    let available = containers.is_ok();

    let containers = containers
        .map(|output| output.lines().filter_map(parse_container_line).collect())
        .unwrap_or_default();

    let images = subprocess::run_with_timeout("docker", &["images", "--format", "{{json .}}"], Duration::from_secs(10))
        .map(|output| output.lines().filter_map(parse_image_line).collect())
        .unwrap_or_default();

    DockerSnapshot { available, containers, images }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_docker_ps_json_line() {
        let line = r#"{"ID":"a1b2c3d4","Image":"nginx:latest","Names":"web","Status":"Up 2 hours"}"#;
        let c = parse_container_line(line).expect("should parse");
        assert_eq!(c.id, "a1b2c3d4");
        assert_eq!(c.image, "nginx:latest");
        assert_eq!(c.name, "web");
        assert_eq!(c.status, "Up 2 hours");
    }

    #[test]
    fn skips_unparseable_container_line() {
        assert!(parse_container_line("not json").is_none());
    }

    #[test]
    fn parses_docker_images_json_line() {
        let line = r#"{"ID":"e5f6a7b8","Repository":"nginx","Tag":"latest","Size":"142MB"}"#;
        let img = parse_image_line(line).expect("should parse");
        assert_eq!(img.id, "e5f6a7b8");
        assert_eq!(img.repository, "nginx");
        assert_eq!(img.tag, "latest");
        assert_eq!(img.size, "142MB");
    }

    #[test]
    fn skips_unparseable_image_line() {
        assert!(parse_image_line("not json").is_none());
    }
}
```

Note `available` is derived from whether the `docker ps` call succeeded at all (binary present + daemon reachable + permission to talk to the socket), not just binary presence — a more accurate "can I actually use Docker" signal than `binary_exists` would give, since `docker` being installed but the daemon not running (or the current user not in the `docker` group) is a common real-world state worth distinguishing from "not installed at all" conceptually, even though this task doesn't surface the distinction to the frontend beyond the boolean (that granularity is a reasonable fast-follow, not needed now).

- [ ] **Step 4: Run tests to verify they pass**

Run: `cd src-tauri && cargo test docker:: 2>&1 | tail -40`
Expected: PASS (4 tests)

- [ ] **Step 5: Register the command**

Modify `src-tauri/src/lib.rs` — add `mod docker;` and `docker::get_docker_snapshot` to `generate_handler!`.

- [ ] **Step 6: Run the full test suite**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: PASS — 71 + 4 = 75 passed, 1 ignored. `cargo build` 0 warnings.

- [ ] **Step 7: Commit**

```bash
git add src-tauri/src/docker.rs src-tauri/src/lib.rs
git commit -m "feat: read-only Docker container/image listing"
```

---

## Task 4: `NetworkPage.vue` frontend

**Files:**
- Create: `src/pages/NetworkPage.vue`

- [ ] **Step 1: Build the tabbed page**

Follows the established tabbed pattern (`ThemeEditorPage.vue`, `DisksPage.vue`) and the established `try/catch` + visible `error` ref pattern for the scan action (the snapshot fetches are infallible per Tasks 1/3's design, so no error ref is needed for those — only the port scan, which can genuinely fail via `validate_port_list`'s `Err`).

```vue
<!-- src/pages/NetworkPage.vue -->
<script setup lang="ts">
import { ref, onMounted } from "vue";
import { invoke } from "@tauri-apps/api/core";

interface WifiNetwork { ssid: string; security: string; signal_percent: number; connected: boolean }
interface ListeningPort { port: number; process: string | null }
interface NetworkSnapshot { wifi_networks: WifiNetwork[]; listening_ports: ListeningPort[]; dns_servers: string[]; hosts_file: string }
interface PortResult { port: number; open: boolean }
interface Container { id: string; image: string; name: string; status: string }
interface DockerImageInfo { id: string; repository: string; tag: string; size: string }
interface DockerSnapshot { available: boolean; containers: Container[]; images: DockerImageInfo[] }

type Tab = "overview" | "portscan" | "docker";
const activeTab = ref<Tab>("overview");

const snapshot = ref<NetworkSnapshot | null>(null);
const docker = ref<DockerSnapshot | null>(null);

onMounted(async () => {
  snapshot.value = await invoke<NetworkSnapshot>("get_network_snapshot");
  docker.value = await invoke<DockerSnapshot>("get_docker_snapshot");
});

const scanHost = ref("127.0.0.1");
const scanPortsInput = ref("22,80,443,3000,8080");
const scanResults = ref<PortResult[]>([]);
const scanError = ref<string | null>(null);
const scanning = ref(false);

async function runScan() {
  scanning.value = true;
  scanError.value = null;
  try {
    const ports = scanPortsInput.value
      .split(",")
      .map((p) => parseInt(p.trim(), 10))
      .filter((p) => !Number.isNaN(p));
    scanResults.value = await invoke<PortResult[]>("scan_ports_cmd", { host: scanHost.value, ports });
  } catch (e) {
    scanError.value = String(e);
  } finally {
    scanning.value = false;
  }
}
</script>

<template>
  <div class="net-page">
    <h1>Réseau</h1>

    <div class="net-tabs">
      <button :class="{ active: activeTab === 'overview' }" @click="activeTab = 'overview'">Vue d'ensemble</button>
      <button :class="{ active: activeTab === 'portscan' }" @click="activeTab = 'portscan'">Scanner de ports</button>
      <button :class="{ active: activeTab === 'docker' }" @click="activeTab = 'docker'">Docker</button>
    </div>

    <section v-if="activeTab === 'overview' && snapshot" class="net-panel">
      <h2>Wi-Fi</h2>
      <div v-for="w in snapshot.wifi_networks" :key="w.ssid" class="net-row">
        <span>{{ w.ssid }}{{ w.connected ? " (connecté)" : "" }}</span>
        <span>{{ w.security }} · {{ w.signal_percent }}%</span>
      </div>

      <h2>Ports en écoute</h2>
      <div v-for="p in snapshot.listening_ports" :key="p.port" class="net-row">
        <span>{{ p.port }}</span>
        <span>{{ p.process ?? "?" }}</span>
      </div>

      <h2>Serveurs DNS</h2>
      <div v-for="d in snapshot.dns_servers" :key="d" class="net-row"><span>{{ d }}</span></div>

      <h2>/etc/hosts</h2>
      <pre class="net-hosts">{{ snapshot.hosts_file }}</pre>
    </section>

    <section v-else-if="activeTab === 'portscan'" class="net-panel">
      <div class="net-form-row">
        <input v-model="scanHost" class="net-input" placeholder="Hôte (ex: 127.0.0.1)" />
        <input v-model="scanPortsInput" class="net-input" placeholder="Ports, séparés par virgule" />
        <button :disabled="scanning" @click="runScan">{{ scanning ? "Scan..." : "Scanner" }}</button>
      </div>
      <div v-if="scanError" class="net-error">{{ scanError }}</div>
      <div v-for="r in scanResults" :key="r.port" class="net-row">
        <span>{{ r.port }}</span>
        <span :class="r.open ? 'net-open' : 'net-closed'">{{ r.open ? "ouvert" : "fermé" }}</span>
      </div>
    </section>

    <section v-else-if="activeTab === 'docker'" class="net-panel">
      <div v-if="!docker?.available" class="net-empty">Docker n'est pas disponible sur ce système.</div>
      <template v-else>
        <h2>Conteneurs</h2>
        <div v-for="c in docker.containers" :key="c.id" class="net-row">
          <span>{{ c.name }} ({{ c.image }})</span>
          <span>{{ c.status }}</span>
        </div>
        <h2>Images</h2>
        <div v-for="i in docker.images" :key="i.id" class="net-row">
          <span>{{ i.repository }}:{{ i.tag }}</span>
          <span>{{ i.size }}</span>
        </div>
      </template>
    </section>
  </div>
</template>

<style scoped>
.net-page { padding: 24px; color: var(--nx-text-primary); }
.net-tabs { display: flex; gap: 8px; margin: 16px 0; }
.net-tabs button { padding: 8px 14px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-secondary); cursor: pointer; }
.net-tabs button.active { color: var(--nx-text-primary); border-color: var(--nx-accent-primary); }
.net-panel h2 { font-size: 14px; margin: 16px 0 6px; color: var(--nx-text-secondary); }
.net-row { display: flex; justify-content: space-between; padding: 6px 0; font-size: 13px; border-bottom: 1px solid var(--nx-border); }
.net-hosts { background: var(--nx-bg-elevated); border: 1px solid var(--nx-border); border-radius: 8px; padding: 12px; font-size: 12px; overflow: auto; max-height: 200px; }
.net-form-row { display: flex; gap: 10px; align-items: center; }
.net-input { flex: 1; padding: 8px 10px; border-radius: 8px; border: 1px solid var(--nx-border); background: var(--nx-bg-elevated); color: var(--nx-text-primary); }
.net-error { margin-top: 10px; padding: 10px 14px; border-radius: 8px; background: color-mix(in srgb, var(--nx-accent-danger) 15%, transparent); border: 1px solid var(--nx-accent-danger); }
.net-open { color: var(--nx-accent-success); }
.net-closed { color: var(--nx-text-secondary); }
.net-empty { color: var(--nx-text-secondary); margin-top: 12px; }
</style>
```

- [ ] **Step 2: Type-check**

Run: `npx vue-tsc --noEmit`
Expected: clean.

- [ ] **Step 3: Commit**

```bash
git add src/pages/NetworkPage.vue
git commit -m "feat: NetworkPage.vue with overview, port scanner, Docker tabs"
```

---

## Task 5: Wire `NetworkPage` into `App.vue` navigation, final verification

**Files:**
- Modify: `src/App.vue`

- [ ] **Step 1: Add the page to the nav**

Read the current `src/App.vue` (7 pages: `dashboard`/`hardware`/`drivers`/`logs`/`theme-editor`/`packages`/`disks`). Add `"network"` to `PageId`, import `NetworkPage`, add to the `pages` map, add an 8th nav button ("Réseau") matching the established pattern exactly.

- [ ] **Step 2: Run the full test suite**

Run: `npm run test` — expect 25 passed (unchanged).
Run: `cd src-tauri && cargo test` — expect 75 passed, 1 ignored.
Run: `npx vue-tsc --noEmit` — expect clean.

- [ ] **Step 3: Manual GUI verification in WSL2**

Same established technique (boot `npm run tauri dev`, confirm real window/surface via `/proc/<pid>/fd`, check dev log for errors, kill processes). Since the app defaults to Dashboard, use the same scratch-test workaround established in Phase 2/3's final tasks to prove `get_network_snapshot()` and `get_docker_snapshot()` work end-to-end against this WSL2 host's real state (expect: `nmcli` likely absent in WSL2 → empty wifi list, not an error; `ss` should work and return real listening ports; `/etc/resolv.conf`/`/etc/hosts` should read real content; `docker` likely absent → `available: false`) — revert the scratch test before committing. Do NOT invoke the port scanner against anything other than `127.0.0.1` during this verification — that's the safe, intended default and there's no reason to scan anything else to prove the feature works.

- [ ] **Step 4: Commit**

```bash
git add src/App.vue
git commit -m "feat: wire NetworkPage into app navigation"
```
