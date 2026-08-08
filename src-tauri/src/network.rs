//! Aggregates four cheap, read-only network diagnostics (Wi-Fi status,
//! listening ports, DNS servers, `/etc/hosts`) into one `NetworkSnapshot`,
//! mirroring `system.rs`'s `SystemSnapshot`/`get_system_snapshot` shape.

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

/// Splits one line of `nmcli -t` output on `:`, honoring nmcli's terse-mode
/// escaping (on by default -- see `nmcli(1)`'s `--escape` option): a literal
/// `:` or `\` inside a field's own value is backslash-escaped (`\:`, `\\`),
/// so a naive `str::split(':')` would incorrectly fragment a field whose
/// value contains a colon -- a real-world SSID like "Office:Guest" would
/// silently drop that network from the list instead of just having a colon
/// in its name.
fn split_nmcli_terse_fields(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut chars = line.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.peek() {
                Some(':') | Some('\\') => {
                    current.push(chars.next().unwrap());
                    continue;
                }
                _ => current.push(c),
            }
        } else if c == ':' {
            fields.push(std::mem::take(&mut current));
        } else {
            current.push(c);
        }
    }
    fields.push(current);
    fields
}

/// Parses one line of `nmcli -t -f IN-USE,SSID,SECURITY,SIGNAL dev wifi`
/// output, e.g. "*:MyHomeWifi:WPA2:78" (connected) or ":OtherNetwork:WPA2:45".
pub fn parse_nmcli_wifi_line(line: &str) -> Option<WifiNetwork> {
    let fields = split_nmcli_terse_fields(line);
    if fields.len() != 4 {
        return None;
    }
    Some(WifiNetwork {
        connected: fields[0] == "*",
        ssid: fields[1].clone(),
        security: fields[2].clone(),
        signal_percent: fields[3].parse().ok()?,
    })
}

/// Parses one line of `ss -tulnp` output for the local port and owning
/// process name.
///
/// Real `ss -tulnp` output (verified on this machine, iproute2 5.15) has a
/// leading Netid column (`tcp`/`udp`) before the state column, e.g.:
/// `"tcp   LISTEN 0      4096    127.0.0.53%lo:53         0.0.0.0:*"`.
/// The local-address field is always 3 tokens after the `LISTEN` token
/// (Recv-Q, Send-Q, then Local Address:Port) regardless of whether a Netid
/// column precedes it, so we locate `LISTEN` positionally rather than
/// assuming a fixed field index — this also correctly skips header lines
/// (no `LISTEN` token present) and non-LISTEN rows (e.g. `UNCONN` UDP
/// sockets).
pub fn parse_ss_line(line: &str) -> Option<ListeningPort> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let listen_idx = fields.iter().position(|&f| f == "LISTEN")?;
    let local_addr = fields.get(listen_idx + 3)?;
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

/// Intentionally infallible: each of the four sub-queries independently
/// degrades to empty/default on failure (e.g. no `nmcli` on a wired-only
/// machine, no Wi-Fi hardware, `/etc/hosts` unreadable), matching
/// `packages::universal::list_universal_updates()`'s "optional supplement,
/// never blocks the rest" philosophy from Phase 2 Task 5 — a machine with
/// no Wi-Fi shouldn't make the whole network page unusable.
#[tauri::command]
pub fn get_network_snapshot() -> NetworkSnapshot {
    NetworkSnapshot {
        wifi_networks: read_wifi_networks(),
        listening_ports: read_listening_ports(),
        dns_servers: read_dns_servers(),
        hosts_file: read_hosts_file(),
    }
}

#[derive(Serialize, Clone)]
pub struct NetworkInterface {
    pub name: String,
    pub mac_address: Option<String>,
    pub rx_bytes_per_sec: u64,
    pub tx_bytes_per_sec: u64,
}

/// Parses one non-header `/proc/net/dev` line into (interface name,
/// cumulative rx_bytes, cumulative tx_bytes) -- confirmed live against
/// this project's own dev machine's real kernel output, e.g.
/// "  eth0:  210412     857    0 ... 500       25033     377 ...": the
/// interface name is followed by ':' with no guaranteed space before it,
/// rx_bytes is the first numeric field after the colon, and tx_bytes is
/// the 9th (index 8) -- the Receive block always has exactly 8 columns
/// (bytes/packets/errs/drop/fifo/frame/compressed/multicast) ahead of the
/// Transmit block's own bytes column, a fixed kernel format, not
/// something that varies by interface.
pub fn parse_proc_net_dev_line(line: &str) -> Option<(String, u64, u64)> {
    let (name, rest) = line.trim().split_once(':')?;
    let fields: Vec<&str> = rest.split_whitespace().collect();
    if fields.len() < 9 {
        return None;
    }
    let rx_bytes: u64 = fields[0].parse().ok()?;
    let tx_bytes: u64 = fields[8].parse().ok()?;
    Some((name.trim().to_string(), rx_bytes, tx_bytes))
}

fn read_interface_byte_counters() -> std::collections::HashMap<String, (u64, u64)> {
    fs::read_to_string("/proc/net/dev")
        .ok()
        .map(|content| content.lines().filter_map(parse_proc_net_dev_line).map(|(name, rx, tx)| (name, (rx, tx))).collect())
        .unwrap_or_default()
}

/// Readable without root on every system tested during this feature's
/// research (confirmed live: `/sys/class/net/eth0/address` readable as a
/// normal user) -- absent entirely is a normal, expected outcome for a
/// virtual/loopback-only interface, not an error to surface.
fn read_mac_address(iface: &str) -> Option<String> {
    fs::read_to_string(format!("/sys/class/net/{iface}/address"))
        .ok()
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

/// Bounds the throughput sample: two `/proc/net/dev` reads separated by
/// this real wall-clock delay, the same delta-over-time approach
/// `system::build_snapshot`'s CPU usage relies on (see that function's
/// doc comment) -- except here both samples are taken within this one
/// command rather than across separate polls, since this page has no
/// existing polling loop to reuse a previous sample from. Short enough
/// not to make a one-time page load noticeably slower.
const THROUGHPUT_SAMPLE_WINDOW: Duration = Duration::from_millis(300);

/// Lists real network interfaces (loopback excluded -- it is not a
/// physical network path and its "throughput" is not meaningful to a
/// user asking about their network hardware) with MAC address and
/// instantaneous rx/tx throughput.
#[tauri::command]
pub fn get_network_interfaces() -> Vec<NetworkInterface> {
    let before = read_interface_byte_counters();
    std::thread::sleep(THROUGHPUT_SAMPLE_WINDOW);
    let after = read_interface_byte_counters();
    let window_secs = THROUGHPUT_SAMPLE_WINDOW.as_secs_f64();

    before
        .iter()
        .filter(|(name, _)| name.as_str() != "lo")
        .map(|(name, &(rx0, tx0))| {
            let (rx1, tx1) = after.get(name).copied().unwrap_or((rx0, tx0));
            NetworkInterface {
                name: name.clone(),
                mac_address: read_mac_address(name),
                rx_bytes_per_sec: (rx1.saturating_sub(rx0) as f64 / window_secs) as u64,
                tx_bytes_per_sec: (tx1.saturating_sub(tx0) as f64 / window_secs) as u64,
            }
        })
        .collect()
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
    fn parses_an_open_network_with_an_empty_security_field_as_an_empty_string() {
        // `nmcli -t` (terse, machine-parseable mode) represents a genuinely
        // absent field value as nothing between its two delimiters, not a
        // placeholder like the "--" used in nmcli's default human-readable
        // table output -- so an open (unencrypted) network's SECURITY
        // column is an empty string here, not "--"/"None"/"Open". This
        // exact scenario (the one this whole feature exists to flag to the
        // user, cf. WiFiAnalyzerPage.vue::securityStatus's `security === ""`
        // check) previously had zero test coverage on the Rust parsing
        // side, even though the frontend's badge logic was already tested
        // against it -- this closes that gap. Not independently verifiable
        // against real WiFi hardware in this environment (no WiFi adapter
        // on the dev VM, cf. .loop/JOURNAL.md cycle 308), but pins the
        // documented terse-mode contract as a regression guard.
        let line = "*:OpenNetwork::45";
        let net = parse_nmcli_wifi_line(line).expect("should parse");
        assert_eq!(net.ssid, "OpenNetwork");
        assert_eq!(net.security, "");
        assert_eq!(net.signal_percent, 45);
    }

    #[test]
    fn parses_an_ssid_containing_a_colon_escaped_by_nmcli_terse_mode() {
        // nmcli -t escapes a literal ':' in a field's own value as '\:' by
        // default (--escape defaults to yes) -- a real SSID like
        // "Office:Guest" appears on the wire as "Office\:Guest", not
        // "Office:Guest" unescaped.
        let line = r"*:Office\:Guest:WPA2:60";
        let net = parse_nmcli_wifi_line(line).expect("should parse an SSID with an escaped colon");
        assert_eq!(net.ssid, "Office:Guest");
        assert_eq!(net.security, "WPA2");
        assert_eq!(net.signal_percent, 60);
    }

    #[test]
    fn parses_an_ssid_containing_a_literal_backslash_escaped_by_nmcli_terse_mode() {
        let line = r"*:Cabin\\Wifi:WPA2:50";
        let net = parse_nmcli_wifi_line(line).expect("should parse an SSID with an escaped backslash");
        assert_eq!(net.ssid, r"Cabin\Wifi");
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

    #[test]
    fn parses_a_real_proc_net_dev_line() {
        // Captured live from this project's own dev machine's real
        // /proc/net/dev, not a hand-constructed guess at the format.
        let line = "  eth0:  210412     857    0    0    0     0          0       500    25033     377    0    8    0     0       0          0";
        let (name, rx, tx) = parse_proc_net_dev_line(line).expect("should parse");
        assert_eq!(name, "eth0");
        assert_eq!(rx, 210412);
        assert_eq!(tx, 25033);
    }

    #[test]
    fn parses_the_loopback_proc_net_dev_line_too() {
        let line = "    lo: 17413491    8001    0    0    0     0          0         0 17413491    8001    0    0    0     0       0          0";
        let (name, rx, tx) = parse_proc_net_dev_line(line).expect("should parse");
        assert_eq!(name, "lo");
        assert_eq!(rx, 17413491);
        assert_eq!(tx, 17413491);
    }

    #[test]
    fn skips_the_proc_net_dev_header_lines() {
        assert!(parse_proc_net_dev_line("Inter-|   Receive                                                |  Transmit").is_none());
        assert!(parse_proc_net_dev_line(" face |bytes    packets errs drop fifo frame compressed multicast|bytes    packets errs drop fifo colls carrier compressed").is_none());
    }

    #[test]
    fn skips_malformed_proc_net_dev_lines() {
        assert!(parse_proc_net_dev_line("not a valid line at all").is_none());
        assert!(parse_proc_net_dev_line("eth0: 123 456").is_none());
    }

    #[test]
    fn get_network_interfaces_excludes_loopback_and_reads_real_data_on_this_host() {
        // Real end-to-end call, not a mock: on any Linux host (including
        // this project's own WSL2 dev environment) at least one interface
        // besides loopback should be present and readable.
        let interfaces = get_network_interfaces();
        assert!(!interfaces.iter().any(|i| i.name == "lo"), "loopback must be excluded");
        assert!(!interfaces.is_empty(), "expected at least one non-loopback interface on this host");
    }
}

/// Not part of the plan's specified test suite — added to prove
/// `parse_ss_line` against the ACTUAL `ss -tulnp` output captured on this
/// WSL2/Ubuntu machine (iproute2 5.15), which includes a leading Netid
/// column the plan's own hand-written test literal omits. Kept as a
/// permanent regression test.
#[cfg(test)]
mod real_ss_output_regression {
    use super::*;

    #[test]
    fn parses_real_machine_ss_line_with_netid_column() {
        let line = "tcp   LISTEN 0      4096    127.0.0.53%lo:53         0.0.0.0:*          ";
        let port = parse_ss_line(line).expect("should parse real ss line with netid column");
        assert_eq!(port.port, 53);
    }

    #[test]
    fn skips_real_udp_unconn_line() {
        let line = "udp   UNCONN 0      0       127.0.0.53%lo:53         0.0.0.0:*          ";
        assert!(parse_ss_line(line).is_none());
    }
}
