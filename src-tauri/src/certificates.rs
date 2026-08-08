//! Lists installed CA certificates (`/etc/ssl/certs/*.pem`), the Linux
//! equivalent of NiTriTe Windows's certificate store viewer
//! (`DiagTabCertificates.vue`), flagging expired/expiring-soon entries.
//! Read-only, non-privileged -- every user can read `/etc/ssl/certs` on a
//! standard Debian install. Shells out to `openssl x509` (already a system
//! binary every one of this app's other network/security features already
//! assumes present) instead of adding a new X.509-parsing crate dependency.

use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

const EXPIRING_SOON_SECS: u64 = 30 * 24 * 60 * 60; // 30 days

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct CertEntry {
    pub filename: String,
    pub subject: String,
    pub issuer: String,
    pub not_after: String,
    pub is_expired: bool,
    pub is_expiring_soon: bool,
}

/// Parses the combined stdout of
/// `openssl x509 -in <file> -noout -subject -issuer -enddate`, e.g.:
/// ```text
/// subject=CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES
/// issuer=CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES
/// notAfter=Dec 31 09:37:37 2030 GMT
/// ```
/// Returns `None` if any of the three fields is missing -- deliberately
/// avoids parsing the `notAfter` value itself (OpenSSL's ASN1_TIME text
/// format has irregular whitespace, e.g. "May  5" for single-digit days)
/// since only `-checkend`'s exit code, not the parsed date, is used to
/// determine expiry state.
fn parse_openssl_output(output: &str) -> Option<(String, String, String)> {
    let mut subject = None;
    let mut issuer = None;
    let mut not_after = None;
    for line in output.lines() {
        if let Some(v) = line.strip_prefix("subject=") {
            subject = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("issuer=") {
            issuer = Some(v.trim().to_string());
        } else if let Some(v) = line.strip_prefix("notAfter=") {
            not_after = Some(v.trim().to_string());
        }
    }
    Some((subject?, issuer?, not_after?))
}

/// `openssl x509 -checkend N` exits non-zero if the certificate is already
/// expired or will expire within N seconds, zero otherwise.
fn is_within_checkend_window(exit_code: i32) -> bool {
    exit_code != 0
}

fn checkend(path: &str, seconds: u64) -> bool {
    subprocess::run_capturing_exit_code(
        "openssl",
        &["x509", "-in", path, "-noout", "-checkend", &seconds.to_string()],
        Duration::from_secs(5),
    )
    .map(|(_, _, code)| is_within_checkend_window(code))
    .unwrap_or(false)
}

fn read_one_certificate(path: &str, filename: &str) -> Option<CertEntry> {
    let (stdout, _, code) = subprocess::run_capturing_exit_code(
        "openssl",
        &["x509", "-in", path, "-noout", "-subject", "-issuer", "-enddate", "-checkend", "0"],
        Duration::from_secs(5),
    )
    .ok()?;
    let (subject, issuer, not_after) = parse_openssl_output(&stdout)?;
    let is_expired = is_within_checkend_window(code);
    // Already-expired certs are trivially also "expiring soon" -- skip the
    // second openssl call in that case rather than asking a question whose
    // answer is already known.
    let is_expiring_soon = !is_expired && checkend(path, EXPIRING_SOON_SECS);
    Some(CertEntry { filename: filename.to_string(), subject, issuer, not_after, is_expired, is_expiring_soon })
}

/// Only `*.pem`-suffixed entries are read: `/etc/ssl/certs` also contains
/// hash-indexed symlinks (e.g. `002c0b4f.0`) OpenSSL itself uses for fast
/// lookup, each pointing to the very same certificate as its `.pem`
/// sibling -- including them would double-list every certificate under two
/// different filenames.
#[tauri::command]
pub fn get_certificates() -> Vec<CertEntry> {
    let entries = match std::fs::read_dir("/etc/ssl/certs") {
        Ok(entries) => entries,
        Err(_) => return vec![],
    };
    let mut certs: Vec<CertEntry> = entries
        .flatten()
        .filter_map(|entry| {
            let filename = entry.file_name().to_string_lossy().into_owned();
            if !filename.ends_with(".pem") {
                return None;
            }
            let path = entry.path().to_string_lossy().into_owned();
            read_one_certificate(&path, &filename)
        })
        .collect();
    certs.sort_by(|a, b| a.subject.cmp(&b.subject));
    certs
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_openssl_x509_output() {
        // Captured live from this project's own dev machine's real
        // /etc/ssl/certs/ACCVRAIZ1.pem, not a hand-constructed guess at
        // OpenSSL's output format.
        let output = "subject=CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES\n\
                       issuer=CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES\n\
                       notAfter=Dec 31 09:37:37 2030 GMT\n";
        let (subject, issuer, not_after) = parse_openssl_output(output).expect("should parse");
        assert_eq!(subject, "CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES");
        assert_eq!(issuer, "CN = ACCVRAIZ1, OU = PKIACCV, O = ACCV, C = ES");
        assert_eq!(not_after, "Dec 31 09:37:37 2030 GMT");
    }

    #[test]
    fn rejects_output_missing_a_required_field() {
        assert!(parse_openssl_output("subject=CN = Test\nissuer=CN = Test\n").is_none());
        assert!(parse_openssl_output("").is_none());
    }

    #[test]
    fn is_within_checkend_window_matches_openssls_documented_exit_code_contract() {
        // openssl x509 -checkend: exit 0 means the cert will NOT expire
        // within the window, non-zero means it will (or already has).
        assert!(!is_within_checkend_window(0));
        assert!(is_within_checkend_window(1));
    }

    #[test]
    fn get_certificates_reads_real_ca_certificates_on_this_host() {
        // Real end-to-end call, not a mock: any standard Debian/Ubuntu
        // install has a populated /etc/ssl/certs, including this project's
        // own WSL2 dev environment (121 certs at time of writing).
        let certs = get_certificates();
        assert!(!certs.is_empty(), "expected at least one CA certificate on this host");
        for cert in &certs {
            assert!(!cert.subject.is_empty(), "{}: subject should never be empty", cert.filename);
            assert!(!cert.issuer.is_empty(), "{}: issuer should never be empty", cert.filename);
            assert!(!cert.not_after.is_empty(), "{}: not_after should never be empty", cert.filename);
        }
        // Every real system CA certificate on a maintained distro is
        // long-lived and currently valid -- a fully-expired default trust
        // store would itself be a distro-level problem, not a plausible
        // fixture state to design around.
        assert!(certs.iter().any(|c| !c.is_expired), "expected at least one non-expired real CA certificate");
    }

    #[test]
    fn get_certificates_never_lists_the_hash_indexed_symlink_duplicate_of_a_pem_file() {
        let certs = get_certificates();
        let filenames: Vec<&str> = certs.iter().map(|c| c.filename.as_str()).collect();
        assert!(filenames.iter().all(|f| f.ends_with(".pem")), "every listed filename must be a real .pem, not an OpenSSL hash-lookup alias");
    }
}
