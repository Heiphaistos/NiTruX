use super::{binary_exists, PackageUpdate};
use crate::subprocess;
use std::time::Duration;

/// Parses one tab-separated line of `flatpak remote-ls --updates` output,
/// e.g.: "org.mozilla.firefox\tFirefox\t121.0\tstable\tflathub"
pub fn parse_flatpak_line(line: &str) -> Option<PackageUpdate> {
    let fields: Vec<&str> = line.split('\t').collect();
    if fields.len() < 3 {
        return None;
    }
    Some(PackageUpdate {
        name: fields[0].to_string(),
        new_version: fields[2].to_string(),
        current_version: String::new(),
        source: "flatpak".to_string(),
    })
}

/// Parses one line of `snap refresh --list` output, e.g.:
/// "firefox    121.0    2000    latest/stable    canonical**"
/// Skips the header row (starts with "Name").
pub fn parse_snap_line(line: &str) -> Option<PackageUpdate> {
    if line.trim_start().starts_with("Name") {
        return None;
    }
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 2 {
        return None;
    }
    Some(PackageUpdate {
        name: fields[0].to_string(),
        new_version: fields[1].to_string(),
        current_version: String::new(),
        source: "snap".to_string(),
    })
}

/// Flatpak and Snap are checked unconditionally alongside whatever native
/// manager(s) were detected — they're an independent, always-on layer per
/// the design spec's multi-distro architecture. A failing source (missing
/// binary, no remotes configured, network error, ...) is silently skipped
/// rather than propagated as an error: unlike apt/dnf/pacman/zypper (which
/// each represent "the" package manager for the detected distro, so a real
/// failure there is significant), Flatpak/Snap are optional "bonus"
/// supplements — one failing should never block the other or the native
/// manager's results.
pub fn list_universal_updates() -> Vec<PackageUpdate> {
    let mut updates = Vec::new();

    if binary_exists("flatpak") {
        if let Ok(output) =
            subprocess::run_with_timeout("flatpak", &["remote-ls", "--updates"], Duration::from_secs(15))
        {
            updates.extend(output.lines().filter_map(parse_flatpak_line));
        }
    }

    if binary_exists("snap") {
        if let Ok(output) = subprocess::run_with_timeout("snap", &["refresh", "--list"], Duration::from_secs(15)) {
            updates.extend(output.lines().filter_map(parse_snap_line));
        }
    }

    updates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_flatpak_remote_ls_updates_line() {
        let line = "org.mozilla.firefox\tFirefox\t121.0\tstable\tflathub";
        let update = parse_flatpak_line(line).expect("should parse");
        assert_eq!(update.name, "org.mozilla.firefox");
        assert_eq!(update.new_version, "121.0");
        assert_eq!(update.source, "flatpak");
    }

    #[test]
    fn skips_flatpak_lines_with_too_few_fields() {
        assert!(parse_flatpak_line("org.mozilla.firefox").is_none());
    }

    #[test]
    fn parses_snap_refresh_list_line() {
        let line = "firefox    121.0    2000    latest/stable    canonical**";
        let update = parse_snap_line(line).expect("should parse");
        assert_eq!(update.name, "firefox");
        assert_eq!(update.new_version, "121.0");
        assert_eq!(update.source, "snap");
    }

    #[test]
    fn skips_snap_header_line() {
        assert!(parse_snap_line("Name      Version  Rev   Tracking       Publisher").is_none());
    }
}
