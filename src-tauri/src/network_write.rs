//! Writing network configuration (`/etc/hosts`, `/etc/resolv.conf`) and
//! firewall rules requires root, so these commands shell out via `pkexec`
//! to the packaged `nitrux-pkexec-helper` script (see
//! `packaging/nitrux-pkexec-helper`) rather than writing the files or
//! calling `ufw` directly from the (unprivileged) app process.

use std::io::Write;
use std::process::{Command, Stdio};
use std::time::Duration;

use base64::{engine::general_purpose::STANDARD, Engine as _};

const PKEXEC_HELPER: &str = "/usr/bin/nitrux-pkexec-helper";
const WRITE_TIMEOUT: Duration = Duration::from_secs(15);

/// A `/etc/hosts` file with no content, or one missing a `localhost`
/// mapping, would break basic name resolution on the target machine —
/// reject those before they ever reach the privileged helper.
fn validate_hosts_content(content: &str) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("le contenu du fichier hosts est vide".to_string());
    }
    let has_localhost = content.lines().any(|line| {
        let trimmed = line.trim_start();
        (trimmed.starts_with("127.") || trimmed.starts_with("::1")) && trimmed.contains("localhost")
    });
    if !has_localhost {
        return Err("le contenu du fichier hosts ne contient aucune entrée localhost (127.x ou ::1)".to_string());
    }
    Ok(())
}

/// A `resolv.conf` with no `nameserver` line leaves the system unable to
/// resolve any DNS name — reject that before it reaches the privileged
/// helper.
fn validate_dns_content(content: &str) -> Result<(), String> {
    if content.trim().is_empty() {
        return Err("le contenu de resolv.conf est vide".to_string());
    }
    let has_nameserver = content
        .lines()
        .any(|line| line.trim_start().starts_with("nameserver "));
    if !has_nameserver {
        return Err("le contenu de resolv.conf ne contient aucune ligne nameserver".to_string());
    }
    Ok(())
}

/// Mirrors the helper script's own `validate_port_proto`, checked here too
/// so obviously-malformed input gets a clear error from the app itself
/// rather than an opaque failure from `pkexec`/the helper.
fn validate_port_proto(value: &str) -> Result<(), String> {
    let Some((port, proto)) = value.split_once('/') else {
        return Err(format!(
            "format port/protocole invalide (attendu par ex. 22/tcp) : {value}"
        ));
    };
    if port.is_empty() || !port.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!(
            "format port/protocole invalide (attendu par ex. 22/tcp) : {value}"
        ));
    }
    if proto != "tcp" && proto != "udp" {
        return Err(format!(
            "format port/protocole invalide (attendu par ex. 22/tcp) : {value}"
        ));
    }
    Ok(())
}

/// Base64-encodes `content` and pipes it to `pkexec <helper> <subcommand>`
/// on stdin, as the helper's `write-hosts`/`set-dns` subcommands expect.
///
/// Unlike `subprocess::run_with_timeout`, this does not yet enforce the
/// `timeout` argument with a kill-on-expiry watchdog — it relies on
/// `pkexec`'s own prompt/exec behavior returning in bounded time. Unifying
/// this with the argv-only timeout helper in `subprocess.rs` is a
/// reasonable future improvement, not something this task builds.
fn run_pkexec_with_stdin(subcommand: &str, content: &str, _timeout: Duration) -> Result<String, String> {
    let encoded = STANDARD.encode(content.as_bytes());

    let mut child = Command::new("pkexec")
        .args([PKEXEC_HELPER, subcommand])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("pkexec introuvable ou impossible à lancer : {e}"))?;

    {
        let stdin = child
            .stdin
            .as_mut()
            .ok_or_else(|| "impossible d'ouvrir le stdin du processus pkexec".to_string())?;
        stdin
            .write_all(encoded.as_bytes())
            .map_err(|e| format!("erreur en écrivant sur le stdin de pkexec : {e}"))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("erreur en lisant la sortie de pkexec : {e}"))?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).into_owned())
    } else {
        let code = output
            .status
            .code()
            .map(|c| c.to_string())
            .unwrap_or_else(|| "inconnu".to_string());
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("pkexec a échoué (code {code}) : {}", stderr.trim()))
    }
}

#[tauri::command]
pub fn write_hosts_file(content: String) -> Result<(), String> {
    validate_hosts_content(&content)?;
    run_pkexec_with_stdin("write-hosts", &content, WRITE_TIMEOUT)?;
    Ok(())
}

#[tauri::command]
pub fn set_dns_servers(content: String) -> Result<(), String> {
    validate_dns_content(&content)?;
    run_pkexec_with_stdin("set-dns", &content, WRITE_TIMEOUT)?;
    Ok(())
}

#[tauri::command]
pub fn add_firewall_rule(port_proto: String) -> Result<String, String> {
    validate_port_proto(&port_proto)?;
    crate::subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_HELPER, "firewall-rule", "add", &port_proto],
        Duration::from_secs(15),
    )
}

#[tauri::command]
pub fn remove_firewall_rule(port_proto: String) -> Result<String, String> {
    validate_port_proto(&port_proto)?;
    crate::subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_HELPER, "firewall-rule", "remove", &port_proto],
        Duration::from_secs(15),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_reasonable_hosts_content() {
        let content = "127.0.0.1 localhost\n::1 localhost\n";
        assert!(validate_hosts_content(content).is_ok());
    }

    #[test]
    fn rejects_empty_hosts_content() {
        assert!(validate_hosts_content("").is_err());
    }

    #[test]
    fn rejects_hosts_content_missing_localhost_entry() {
        let content = "93.184.216.34 example.com\n";
        assert!(validate_hosts_content(content).is_err());
    }

    #[test]
    fn accepts_reasonable_dns_content() {
        let content = "nameserver 1.1.1.1\nnameserver 8.8.8.8\n";
        assert!(validate_dns_content(content).is_ok());
    }

    #[test]
    fn rejects_empty_dns_content() {
        assert!(validate_dns_content("").is_err());
    }

    #[test]
    fn rejects_dns_content_with_no_nameserver_lines() {
        let content = "search example.com\noptions rotate\n";
        assert!(validate_dns_content(content).is_err());
    }

    #[test]
    fn accepts_well_formed_port_proto() {
        assert!(validate_port_proto("22/tcp").is_ok());
        assert!(validate_port_proto("53/udp").is_ok());
        assert!(validate_port_proto("8080/tcp").is_ok());
    }

    #[test]
    fn rejects_malformed_port_proto() {
        assert!(validate_port_proto("").is_err());
        assert!(validate_port_proto("22").is_err());
        assert!(validate_port_proto("22/tcp; rm -rf /").is_err());
        assert!(validate_port_proto("abc/tcp").is_err());
        assert!(validate_port_proto("22/ftp").is_err());
    }
}
