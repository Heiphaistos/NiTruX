//! Resolves a user-specified host against a chosen DNS record type, the
//! Linux equivalent of NiTriTe Windows's DNS lookup panel
//! (`DiagTabNetTools.vue`). Shells out to `dig +short` (a standard
//! `dnsutils` binary, already assumed present the same way `ping`/
//! `traceroute` already are in `systemToolsCatalog.ts`) rather than
//! hand-rolling a DNS resolver -- `+short` gives one record per line with
//! no header/footer text to parse.

use crate::subprocess;
use std::time::Duration;

const QUERY_TYPES: [&str; 6] = ["A", "AAAA", "MX", "TXT", "CNAME", "NS"];

fn validate_dns_lookup_input(host: &str, query_type: &str) -> Result<(), String> {
    if host.trim().is_empty() {
        return Err("hôte vide".to_string());
    }
    if host.starts_with('-') {
        return Err(format!("hôte invalide : {host}"));
    }
    if !QUERY_TYPES.contains(&query_type) {
        return Err(format!("type d'enregistrement inconnu : {query_type}"));
    }
    Ok(())
}

/// `dig +short` prints one record per line and nothing else -- no header,
/// no footer, an entirely empty stdout for a name with no records of the
/// requested type (including NXDOMAIN, verified live: exit code 0, no
/// output, no error text). Blank lines are filtered defensively even
/// though none were observed in real captures, since a stray blank line
/// would otherwise render as an empty row in the UI.
fn parse_dig_short_output(output: &str) -> Vec<String> {
    output.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

#[tauri::command]
pub fn dns_lookup(host: String, query_type: String) -> Result<Vec<String>, String> {
    validate_dns_lookup_input(&host, &query_type)?;
    let output = subprocess::run_with_timeout(
        "dig",
        &["+short", &query_type, &host],
        Duration::from_secs(10),
    )?;
    Ok(parse_dig_short_output(&output))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_an_empty_host() {
        assert!(validate_dns_lookup_input("", "A").is_err());
        assert!(validate_dns_lookup_input("   ", "A").is_err());
    }

    #[test]
    fn rejects_a_host_starting_with_a_dash() {
        assert!(validate_dns_lookup_input("-oops", "A").is_err());
    }

    #[test]
    fn rejects_an_unknown_query_type() {
        assert!(validate_dns_lookup_input("example.com", "HACK").is_err());
        assert!(validate_dns_lookup_input("example.com", "").is_err());
    }

    #[test]
    fn accepts_every_supported_query_type_with_a_reasonable_host() {
        for t in QUERY_TYPES {
            assert!(validate_dns_lookup_input("example.com", t).is_ok());
        }
    }

    #[test]
    fn parses_a_real_single_a_record() {
        // Captured live from this project's own dev machine.
        assert_eq!(parse_dig_short_output("172.217.22.110\n"), vec!["172.217.22.110".to_string()]);
    }

    #[test]
    fn parses_real_multiple_txt_records() {
        // Captured live: dig quotes each TXT record's value.
        let output = "\"v=spf1 include:_spf.google.com ~all\"\n\"google-site-verification=abc\"\n";
        assert_eq!(
            parse_dig_short_output(output),
            vec![
                "\"v=spf1 include:_spf.google.com ~all\"".to_string(),
                "\"google-site-verification=abc\"".to_string(),
            ]
        );
    }

    #[test]
    fn parses_a_real_cname_chain_as_two_lines() {
        // Captured live: `dig +short A www.github.com` resolves through a
        // CNAME first, both lines are real, useful records to show, not
        // noise to collapse.
        assert_eq!(
            parse_dig_short_output("github.com.\n140.82.121.3\n"),
            vec!["github.com.".to_string(), "140.82.121.3".to_string()]
        );
    }

    #[test]
    fn empty_output_parses_to_an_empty_list_not_an_error() {
        // Captured live: a name with no records of the requested type
        // (including NXDOMAIN) exits 0 with empty stdout -- "no records
        // found" is a legitimate, distinct outcome from a real failure.
        assert_eq!(parse_dig_short_output(""), Vec::<String>::new());
    }

    #[test]
    fn dns_lookup_rejects_invalid_input_before_ever_shelling_out() {
        assert!(dns_lookup("".to_string(), "A".to_string()).is_err());
        assert!(dns_lookup("example.com".to_string(), "BOGUS".to_string()).is_err());
    }

    #[test]
    fn dns_lookup_resolves_a_real_name_on_this_host() {
        // Real end-to-end call, not a mock: localhost's own reverse
        // resolution aside, a well-known name should resolve to at least
        // one A record wherever this test runs.
        let records = dns_lookup("localhost".to_string(), "A".to_string()).expect("should succeed");
        assert!(records.iter().any(|r| r == "127.0.0.1"), "expected localhost to resolve to 127.0.0.1, got {records:?}");
    }
}
