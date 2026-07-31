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
