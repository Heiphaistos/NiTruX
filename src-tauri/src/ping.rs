//! Ping a user-specified host, the Linux equivalent of NiTriTe Windows's
//! dedicated Ping tool (`DiagTabNetTools.vue`'s ping panel). Shells out to
//! the system `ping` binary (already relied on non-privileged by
//! `systemToolsCatalog.ts`'s fixed "Test de connectivité" entry) rather
//! than opening a raw ICMP socket, which on Linux requires either root or
//! a `ping_group_range` sysctl this app has no business assuming is
//! configured.

use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct PingSummary {
    pub packets_sent: u32,
    pub packets_received: u32,
    pub loss_percent: f64,
    pub min_ms: Option<f64>,
    pub avg_ms: Option<f64>,
    pub max_ms: Option<f64>,
}

/// A host starting with `-` would otherwise be passed through to `ping` as
/// a single argv element (never shell-interpreted, so never a command
/// injection risk) but could still be misread by `ping` itself as an
/// unknown option, producing a confusing raw CLI error instead of this
/// command's own clear message.
fn validate_ping_host(host: &str) -> Result<(), String> {
    if host.trim().is_empty() {
        return Err("hôte vide".to_string());
    }
    if host.starts_with('-') {
        return Err(format!("hôte invalide : {host}"));
    }
    Ok(())
}

/// Finds `ping`'s stats line, e.g.
/// `"4 packets transmitted, 4 received, 0% packet loss, time 2997ms"` --
/// or, captured live against a real unreachable host on this project's own
/// dev machine, `"4 packets transmitted, 0 received, +4 errors, 100%
/// packet loss, time 3013ms"` (an extra `+N errors,` segment appears for
/// ICMP "Destination Host Unreachable" replies). Splitting on `,` and
/// matching each segment by its own suffix, rather than assuming a fixed
/// segment count/order, tolerates that extra segment without extra cases.
fn parse_ping_stats_line(line: &str) -> Option<(u32, u32, f64)> {
    let mut sent = None;
    let mut received = None;
    let mut loss = None;
    for segment in line.split(',') {
        let segment = segment.trim();
        if let Some(n) = segment.strip_suffix(" packets transmitted") {
            sent = n.trim().parse().ok();
        } else if let Some(n) = segment.strip_suffix(" received") {
            received = n.trim().parse().ok();
        } else if let Some(pct) = segment.strip_suffix("% packet loss") {
            loss = pct.trim().parse().ok();
        }
    }
    Some((sent?, received?, loss?))
}

/// Parses `"rtt min/avg/max/mdev = 15.984/18.368/22.885/2.673 ms"`, present
/// only when at least one packet was actually received -- absent entirely
/// (not zeroed) on 100% loss, which is why the summary's *_ms fields are
/// `Option`, not defaulted to 0.0 (a 0ms round-trip would misleadingly
/// read as "instant", not "unknown").
fn parse_rtt_line(output: &str) -> Option<(f64, f64, f64)> {
    let line = output.lines().find(|l| l.trim_start().starts_with("rtt "))?;
    let values = line.split('=').nth(1)?.trim().strip_suffix(" ms")?;
    let mut parts = values.split('/');
    let min = parts.next()?.parse().ok()?;
    let avg = parts.next()?.parse().ok()?;
    let max = parts.next()?.parse().ok()?;
    Some((min, avg, max))
}

/// `combined` is `ping`'s stdout+stderr concatenated: a resolution failure
/// (`"ping: this-does-not-resolve.invalid: Name or service not known"`,
/// captured live) never produces a stats line at all, and this project's
/// `ping` build exits 0 even in that case -- confirmed live, not assumed --
/// so a missing stats line, not the exit code, is what signals failure here.
fn parse_ping_output(combined: &str) -> Result<PingSummary, String> {
    let stats_line = combined
        .lines()
        .find(|l| l.contains("packets transmitted"))
        .ok_or_else(|| {
            combined
                .lines()
                .find(|l| l.trim_start().starts_with("ping:"))
                .map(str::trim)
                .unwrap_or("aucune réponse de ping")
                .to_string()
        })?;
    let (packets_sent, packets_received, loss_percent) = parse_ping_stats_line(stats_line)
        .ok_or_else(|| "impossible d'interpréter la ligne de statistiques ping".to_string())?;
    let (min_ms, avg_ms, max_ms) = match parse_rtt_line(combined) {
        Some((min, avg, max)) => (Some(min), Some(avg), Some(max)),
        None => (None, None, None),
    };
    Ok(PingSummary { packets_sent, packets_received, loss_percent, min_ms, avg_ms, max_ms })
}

#[tauri::command]
pub fn ping_host(host: String) -> Result<PingSummary, String> {
    validate_ping_host(&host)?;
    let (stdout, stderr, _code) = subprocess::run_capturing_exit_code(
        "ping",
        &["-c", "4", "-W", "2", &host],
        Duration::from_secs(12),
    )?;
    parse_ping_output(&format!("{stdout}\n{stderr}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_empty_host() {
        assert!(validate_ping_host("").is_err());
        assert!(validate_ping_host("   ").is_err());
    }

    #[test]
    fn rejects_a_host_starting_with_a_dash() {
        assert!(validate_ping_host("-c").is_err());
    }

    #[test]
    fn accepts_a_reasonable_host() {
        assert!(validate_ping_host("8.8.8.8").is_ok());
        assert!(validate_ping_host("example.com").is_ok());
    }

    #[test]
    fn parses_a_real_successful_ping_output() {
        // Captured live against 8.8.8.8 from this project's own dev machine.
        let output = "PING 8.8.8.8 (8.8.8.8) 56(84) bytes of data.\n\
                       64 bytes from 8.8.8.8: icmp_seq=1 ttl=115 time=16.0 ms\n\
                       64 bytes from 8.8.8.8: icmp_seq=2 ttl=115 time=17.0 ms\n\
                       64 bytes from 8.8.8.8: icmp_seq=3 ttl=115 time=17.6 ms\n\
                       64 bytes from 8.8.8.8: icmp_seq=4 ttl=115 time=22.9 ms\n\
                       \n\
                       --- 8.8.8.8 ping statistics ---\n\
                       4 packets transmitted, 4 received, 0% packet loss, time 2997ms\n\
                       rtt min/avg/max/mdev = 15.984/18.368/22.885/2.673 ms\n";
        let summary = parse_ping_output(output).expect("should parse");
        assert_eq!(summary.packets_sent, 4);
        assert_eq!(summary.packets_received, 4);
        assert_eq!(summary.loss_percent, 0.0);
        assert_eq!(summary.min_ms, Some(15.984));
        assert_eq!(summary.avg_ms, Some(18.368));
        assert_eq!(summary.max_ms, Some(22.885));
    }

    #[test]
    fn parses_a_real_unreachable_host_output_including_the_extra_errors_segment() {
        // Captured live against an unreachable LAN address from this
        // project's own dev machine -- the "+4 errors," segment (from ICMP
        // "Destination Host Unreachable" replies) is exactly the kind of
        // real-world shape a hand-written test literal would miss.
        let output = "PING 10.255.255.1 (10.255.255.1) 56(84) bytes of data.\n\
                       From 192.168.1.254 icmp_seq=1 Destination Host Unreachable\n\
                       From 192.168.1.254 icmp_seq=2 Destination Host Unreachable\n\
                       From 192.168.1.254 icmp_seq=3 Destination Host Unreachable\n\
                       From 192.168.1.254 icmp_seq=4 Destination Host Unreachable\n\
                       \n\
                       --- 10.255.255.1 ping statistics ---\n\
                       4 packets transmitted, 0 received, +4 errors, 100% packet loss, time 3013ms\n";
        let summary = parse_ping_output(output).expect("should parse despite the extra errors segment");
        assert_eq!(summary.packets_sent, 4);
        assert_eq!(summary.packets_received, 0);
        assert_eq!(summary.loss_percent, 100.0);
        // No rtt line at all on 100% loss -- must stay None, not 0.0.
        assert_eq!(summary.min_ms, None);
        assert_eq!(summary.avg_ms, None);
        assert_eq!(summary.max_ms, None);
    }

    #[test]
    fn surfaces_a_real_dns_resolution_failure_as_a_clear_error_instead_of_a_bogus_empty_summary() {
        // Captured live: this project's own `ping` build exits 0 even when
        // the hostname never resolves, so the absence of a stats line (not
        // the exit code) is what this parser keys off.
        let output = "ping: this-does-not-resolve.invalid: Name or service not known\n";
        let err = parse_ping_output(output).expect_err("should surface as an error, not a fabricated summary");
        assert!(err.contains("Name or service not known"));
    }

    #[test]
    fn ping_host_rejects_an_invalid_host_before_ever_shelling_out() {
        assert!(ping_host("-oops".to_string()).is_err());
    }

    #[test]
    fn ping_host_pings_a_real_reachable_address_on_this_host() {
        // Real end-to-end call, not a mock: localhost always responds.
        let summary = ping_host("127.0.0.1".to_string()).expect("pinging localhost should succeed");
        assert_eq!(summary.packets_sent, 4);
        assert!(summary.packets_received > 0, "localhost should reply to at least one echo request");
        assert!(summary.min_ms.is_some());
    }
}
