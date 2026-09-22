//! Downloads a hosts-format blocklist so the Hosts editor can merge it.
//!
//! The file is fetched, validated and handed back to the frontend as text;
//! nothing is written here. The actual write still goes through the single
//! privileged `write_hosts_file` path, which keeps one audited place where
//! `/etc/hosts` can change.

use crate::subprocess;
use std::time::Duration;

/// Entries a blocklist is allowed to point at. A list that mapped a domain
/// to a real address would be a redirection, not a block.
const BLOCKHOLE_IPS: &[&str] = &["0.0.0.0", "127.0.0.1", "::1", "::"];

/// Rejects anything that is not a plain public HTTPS URL.
///
/// This URL comes from the frontend and is handed to `curl`, so it is an
/// SSRF sink: without this check a "blocklist URL" could point at
/// `http://169.254.169.254/...` (cloud metadata), at a service bound to
/// localhost, or at a private LAN host, and the response would be shown
/// back to the user.
pub fn validate_blocklist_url(url: &str) -> Result<(), String> {
    let Some(rest) = url.strip_prefix("https://") else {
        return Err("l'URL doit commencer par https://".to_string());
    };
    if url.len() > 2048 {
        return Err("URL trop longue".to_string());
    }
    // A leading `-` would reach curl as a flag, and credentials in the
    // authority (`user@host`) hide the real host from a naive reader.
    if url.starts_with("https://-") || rest.split('/').next().unwrap_or("").contains('@') {
        return Err(format!("URL refusée : {url}"));
    }
    let host = rest
        .split(['/', '?', '#'])
        .next()
        .unwrap_or("")
        .split(':')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    if host.is_empty() {
        return Err("URL sans nom d'hôte".to_string());
    }
    let is_private = host == "localhost"
        || host.ends_with(".localhost")
        || host.ends_with(".local")
        || host.ends_with(".internal")
        || host == "0.0.0.0"
        || host.starts_with("127.")
        || host.starts_with("10.")
        || host.starts_with("192.168.")
        || host.starts_with("169.254.")
        || host.starts_with("[")
        || (host.starts_with("172.")
            && host
                .split('.')
                .nth(1)
                .and_then(|o| o.parse::<u8>().ok())
                .is_some_and(|o| (16..=31).contains(&o)));
    if is_private {
        return Err(format!("URL refusée : {host} est une adresse locale ou privée"));
    }
    Ok(())
}

/// Keeps only the lines that really are blocklist entries, and returns the
/// domains. A blocklist is a hosts file, so it also carries comments, its
/// own `localhost` entries and occasional junk -- merging those into the
/// user's `/etc/hosts` would at best add noise and at worst overwrite their
/// own loopback lines.
pub fn extract_blocked_domains(content: &str) -> Vec<String> {
    let mut domains = Vec::new();
    for line in content.lines() {
        let line = line.split('#').next().unwrap_or("").trim();
        if line.is_empty() {
            continue;
        }
        let mut fields = line.split_whitespace();
        let Some(ip) = fields.next() else { continue };
        if !BLOCKHOLE_IPS.contains(&ip) {
            continue;
        }
        for domain in fields {
            let domain = domain.trim().to_ascii_lowercase();
            // `0.0.0.0 localhost` appears in several published lists and
            // would break name resolution for the user's own machine.
            if domain == "localhost" || domain.starts_with("localhost.") || domain.is_empty() {
                continue;
            }
            if domain.len() <= 253 && domain.contains('.') && !domain.contains('/') {
                domains.push(domain);
            }
        }
    }
    domains.sort();
    domains.dedup();
    domains
}

#[tauri::command]
pub fn download_hosts_blocklist(url: String) -> Result<Vec<String>, String> {
    validate_blocklist_url(&url)?;
    let output = subprocess::run_with_timeout(
        "curl",
        &[
            "--proto",
            "=https",
            "--location",
            "--max-redirs",
            "3",
            "--max-filesize",
            "20000000",
            "--max-time",
            "60",
            "--silent",
            "--show-error",
            "--fail",
            url.as_str(),
        ],
        Duration::from_secs(90),
    )?;
    let domains = extract_blocked_domains(&output);
    if domains.is_empty() {
        return Err("aucune entrée de blocage trouvée : ce fichier n'est pas une liste au format hosts".to_string());
    }
    Ok(domains)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_a_real_published_blocklist_url() {
        assert!(validate_blocklist_url(
            "https://raw.githubusercontent.com/StevenBlack/hosts/master/hosts"
        )
        .is_ok());
    }

    #[test]
    fn refuses_every_ssrf_shaped_url() {
        for bad in [
            "http://example.com/hosts",              // plaintext
            "https://169.254.169.254/latest/meta-data", // cloud metadata
            "https://localhost/hosts",
            "https://127.0.0.1:8080/hosts",
            "https://192.168.1.1/hosts",
            "https://172.20.0.5/hosts",
            "https://10.0.0.1/hosts",
            "https://router.local/hosts",
            "https://evil.com@internal-host/hosts", // credentials hide the host
            "file:///etc/shadow",
        ] {
            assert!(validate_blocklist_url(bad).is_err(), "{bad} should be refused");
        }
    }

    #[test]
    fn allows_a_public_host_whose_name_merely_starts_like_a_private_range() {
        // 172.15.x is public; only 172.16-31 is private.
        assert!(validate_blocklist_url("https://172.15.1.1/hosts").is_ok());
    }

    #[test]
    fn extracts_domains_from_a_real_blocklist_excerpt() {
        let content = "# Title: StevenBlack/hosts\n\n0.0.0.0 ads.example.com\n0.0.0.0 tracker.example.net # inline comment\n127.0.0.1 malware.example.org\n\n";
        let domains = extract_blocked_domains(content);
        assert_eq!(domains, vec!["ads.example.com", "malware.example.org", "tracker.example.net"]);
    }

    #[test]
    fn never_imports_localhost_or_a_real_redirection() {
        // `0.0.0.0 localhost` really does appear in published lists, and a
        // line pointing at a routable address is a redirect, not a block.
        let content = "0.0.0.0 localhost\n127.0.0.1 localhost.localdomain\n93.184.216.34 example.com\n0.0.0.0 ads.example.com\n";
        assert_eq!(extract_blocked_domains(content), vec!["ads.example.com"]);
    }

    #[test]
    fn deduplicates_repeated_domains() {
        let content = "0.0.0.0 a.example.com\n0.0.0.0 a.example.com\n0.0.0.0 b.example.com\n";
        assert_eq!(extract_blocked_domains(content), vec!["a.example.com", "b.example.com"]);
    }
}
