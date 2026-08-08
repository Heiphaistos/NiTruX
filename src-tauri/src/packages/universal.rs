use super::{binary_exists, PackageUpdate};
use crate::subprocess;
use std::time::Duration;

/// Parses one tab-separated line of `flatpak remote-ls --updates` output.
/// Real column order (confirmed live on the VM, flatpak 1.16.6): display
/// name, then app ID, then version, then branch, then arch -- e.g.
/// "GitKraken\tcom.axosoft.GitKraken\t12.3.1\tstable\tx86_64". This
/// function only reads fields[0] (name) and fields[2] (version), both of
/// which the previous synthetic doc example/test fixture (app ID first,
/// display name second, "flathub" as a 5th "remote" column) had backwards
/// relative to the real tool -- coincidentally harmless since neither the
/// swapped fields[0]/[1] order nor the mislabeled unused fields[4] changed
/// what gets extracted, but corrected here so a future reader doesn't
/// trust the wrong column semantics. Runtime/platform packages (e.g.
/// "org.gnome.Platform") have an empty version field in real output --
/// left as an empty string, a known limitation mirroring dnf.rs's/
/// zypper.rs's own documented version-column gaps, not a parsing bug.
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
/// Skips the header row (starts with "Name"). Also skips the plain-sentence
/// summary snap prints instead of a table when there is nothing to
/// refresh -- confirmed live on the VM: `snap refresh --list` exits 0 and
/// prints exactly "All snaps up to date." (no header, no table) in that
/// case. That sentence has the same 5-whitespace-token shape as a real
/// update row, so a field-count check alone can't tell them apart; instead
/// require the would-be version field to contain a digit, which every real
/// snap version does ("121.0", "2000", "2.76.1") and no English word in
/// that sentence ever does.
pub fn parse_snap_line(line: &str) -> Option<PackageUpdate> {
    if line.trim_start().starts_with("Name") {
        return None;
    }
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 2 {
        return None;
    }
    if !fields[1].chars().any(|c| c.is_ascii_digit()) {
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
        // Real column order captured live from `flatpak remote-ls
        // --updates` on the VM (flatpak 1.16.6): name, app ID, version,
        // branch, arch -- not the app-ID-first order this fixture used to
        // assume.
        let line = "GitKraken\tcom.axosoft.GitKraken\t12.3.1\tstable\tx86_64";
        let update = parse_flatpak_line(line).expect("should parse");
        assert_eq!(update.name, "GitKraken");
        assert_eq!(update.new_version, "12.3.1");
        assert_eq!(update.source, "flatpak");
    }

    #[test]
    fn handles_a_runtime_platform_line_with_an_empty_version_field() {
        // Real output captured live on the VM: runtime/platform packages
        // report an empty version field (only a branch number). Must not
        // panic or drop the entry -- current_version/new_version limits
        // are already a known, accepted gap (mirrors dnf.rs/zypper.rs).
        let line = "GNOME Application Platform version 49\torg.gnome.Platform\t\t49\tx86_64";
        let update = parse_flatpak_line(line).expect("should still parse despite the empty version field");
        assert_eq!(update.name, "GNOME Application Platform version 49");
        assert_eq!(update.new_version, "");
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

    #[test]
    fn skips_the_all_up_to_date_sentence_snap_prints_when_there_is_nothing_to_refresh() {
        // Real output captured live from `snap refresh --list` on the VM
        // (snapd 2.76.1) when no snap has an update: no header, no table,
        // just this one sentence, exit code 0. Without a fix this parses as
        // a fake update for a package named "All" version "snaps".
        assert!(parse_snap_line("All snaps up to date.").is_none());
    }
}
