//! Traces the network path to a user-specified host, the Linux equivalent
//! of NiTriTe Windows's traceroute panel (`DiagTabNetTools.vue`). Shells
//! out to the system `traceroute` binary (already relied on non-privileged
//! by `systemToolsCatalog.ts`'s fixed "Traceroute" entry, same reasoning
//! as `ping.rs`) rather than opening a raw socket, which would need root.

use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct TracerouteHop {
    pub hop: u32,
    /// The first responding host/IP seen for this hop, `None` if every
    /// probe timed out (`* * *`). Real routers load-balance across
    /// multiple next hops (ECMP) -- a single hop can show a different host
    /// per probe (captured live, see `real_traceroute_output_regression`
    /// below) -- this deliberately keeps only the first one rather than
    /// modeling per-probe host attribution, matching NiTriTe's own
    /// single-address-per-hop model rather than inventing new complexity
    /// this feature doesn't need.
    pub host: Option<String>,
    /// Every successfully-measured round-trip time for this hop, in probe
    /// order, timed-out probes simply contribute no entry (a hop with 1
    /// timeout out of 3 probes has 2 entries here, not 3 with a placeholder).
    pub times_ms: Vec<f64>,
}

fn validate_traceroute_host(host: &str) -> Result<(), String> {
    if host.trim().is_empty() {
        return Err("hôte vide".to_string());
    }
    if host.starts_with('-') {
        return Err(format!("hôte invalide : {host}"));
    }
    Ok(())
}

/// Parses one hop line of `traceroute` output, e.g. (captured live on this
/// project's own dev machine): `" 1  172.17.208.1 (172.17.208.1)  2.708 ms
/// 2.685 ms  2.124 ms"` (single host, 3 successful probes), `" 2  * * *"`
/// (every probe timed out), or `" 3  158.173.158.125 (158.173.158.125)
/// 30.086 ms 158.173.158.124 (158.173.158.124)  30.175 ms
/// 158.173.158.125 (158.173.158.125)  30.084 ms"` (ECMP: the middle probe
/// hit a different real host than the other two).
///
/// The header line (`"traceroute to 8.8.8.8 (8.8.8.8), ..."`) fails to
/// parse as a hop line because its first token isn't a hop number, which
/// is exactly what makes `filter_map` skip it for free -- no special-cased
/// header check needed.
pub fn parse_traceroute_hop_line(line: &str) -> Option<TracerouteHop> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    let hop: u32 = fields.first()?.parse().ok()?;
    let mut host = None;
    let mut times_ms = Vec::new();
    let mut i = 1;
    while i < fields.len() {
        let tok = fields[i];
        if tok == "*" {
            i += 1;
            continue;
        }
        if i + 1 < fields.len() && fields[i + 1].starts_with('(') && fields[i + 1].ends_with(')') {
            if host.is_none() {
                host = Some(tok.to_string());
            }
            i += 2;
            continue;
        }
        if i + 1 < fields.len() && fields[i + 1] == "ms" {
            if let Ok(t) = tok.parse::<f64>() {
                times_ms.push(t);
            }
            i += 2;
            continue;
        }
        i += 1;
    }
    Some(TracerouteHop { hop, host, times_ms })
}

#[tauri::command]
pub fn traceroute_host(host: String) -> Result<Vec<TracerouteHop>, String> {
    validate_traceroute_host(&host)?;
    let (stdout, stderr, code) = subprocess::run_capturing_exit_code(
        "traceroute",
        &["-m", "15", "-w", "1", &host],
        Duration::from_secs(60),
    )?;
    // Unlike this environment's `ping` build (which exits 0 even on a DNS
    // resolution failure, see ping.rs), `traceroute` here exits non-zero
    // and prints a clear message -- confirmed live against a name that
    // never resolves ("this-does-not-resolve...: Temporary failure in
    // name resolution", exit code 2). The two tools are NOT symmetric;
    // this checks the exit code rather than assuming ping's behavior
    // carries over.
    if code != 0 {
        let message = stdout
            .lines()
            .next()
            .or_else(|| stderr.lines().next())
            .unwrap_or("échec du traceroute")
            .to_string();
        return Err(message);
    }
    Ok(stdout.lines().filter_map(parse_traceroute_hop_line).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_empty_host() {
        assert!(validate_traceroute_host("").is_err());
        assert!(validate_traceroute_host("   ").is_err());
    }

    #[test]
    fn rejects_a_host_starting_with_a_dash() {
        assert!(validate_traceroute_host("-oops").is_err());
    }

    #[test]
    fn accepts_a_reasonable_host() {
        assert!(validate_traceroute_host("8.8.8.8").is_ok());
    }

    #[test]
    fn skips_the_traceroute_header_line() {
        let line = "traceroute to 8.8.8.8 (8.8.8.8), 4 hops max, 60 byte packets";
        assert!(parse_traceroute_hop_line(line).is_none());
    }

    #[test]
    fn traceroute_host_rejects_invalid_input_before_ever_shelling_out() {
        assert!(traceroute_host("".to_string()).is_err());
        assert!(traceroute_host("-oops".to_string()).is_err());
    }
}

/// Not part of the plan's specified test suite — added to prove
/// `parse_traceroute_hop_line` against ACTUAL `traceroute` output. This
/// dev machine doesn't have `traceroute` installed (unlike the target
/// Debian distro `systemToolsCatalog.ts` already assumes it on), so the
/// real binary was extracted without root via `apt-get download` +
/// `dpkg-deb -x` (the technique already established in this project,
/// cf. LESSONS.md cycle 107) purely to capture this output -- never
/// shipped or referenced by the app itself. Kept as a permanent regression
/// test, same rationale as `real_ss_output_regression` in `network.rs`.
#[cfg(test)]
mod real_traceroute_output_regression {
    use super::*;

    #[test]
    fn parses_a_real_single_host_hop_with_three_successful_probes() {
        let line = " 1  172.17.208.1 (172.17.208.1)  2.708 ms  2.685 ms  2.124 ms";
        let hop = parse_traceroute_hop_line(line).expect("should parse");
        assert_eq!(hop.hop, 1);
        assert_eq!(hop.host.as_deref(), Some("172.17.208.1"));
        assert_eq!(hop.times_ms, vec![2.708, 2.685, 2.124]);
    }

    #[test]
    fn parses_a_real_fully_timed_out_hop() {
        let line = " 2  * * *";
        let hop = parse_traceroute_hop_line(line).expect("should parse");
        assert_eq!(hop.hop, 2);
        assert_eq!(hop.host, None);
        assert!(hop.times_ms.is_empty());
    }

    #[test]
    fn parses_a_real_ecmp_hop_keeping_the_first_host_and_every_successful_time() {
        // Regression guard for the actual real-world shape: a real router
        // load-balanced this hop's 3 probes across two different physical
        // next hops (.125, .124, .125) -- a parser assuming one host per
        // hop line would have silently dropped or corrupted this data.
        let line = " 3  158.173.158.125 (158.173.158.125)  30.086 ms 158.173.158.124 (158.173.158.124)  30.175 ms 158.173.158.125 (158.173.158.125)  30.084 ms";
        let hop = parse_traceroute_hop_line(line).expect("should parse");
        assert_eq!(hop.hop, 3);
        assert_eq!(hop.host.as_deref(), Some("158.173.158.125"));
        assert_eq!(hop.times_ms, vec![30.086, 30.175, 30.084]);
    }

    #[test]
    fn parses_a_real_hop_with_two_different_hostnames_across_probes() {
        let line = " 4  vl221.par-itx5-core-1.cdn77.com (79.127.195.33)  30.069 ms  50.540 ms vl221.par-itx5-core-2.cdn77.com (79.127.195.34)  50.404 ms";
        let hop = parse_traceroute_hop_line(line).expect("should parse");
        assert_eq!(hop.hop, 4);
        assert_eq!(hop.host.as_deref(), Some("vl221.par-itx5-core-1.cdn77.com"));
        assert_eq!(hop.times_ms, vec![30.069, 50.540, 50.404]);
    }
}
