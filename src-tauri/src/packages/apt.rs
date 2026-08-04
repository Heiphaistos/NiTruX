use super::{PackageManager, PackageUpdate};
use crate::subprocess;
use std::collections::HashSet;
use std::time::Duration;

pub struct Apt;

/// Parses one line of `apt list --upgradable` output, e.g.:
/// "curl/jammy-updates 7.81.0-1ubuntu1.20 amd64 [upgradable from: 7.81.0-1ubuntu1.19]"
pub fn parse_apt_line(line: &str) -> Option<PackageUpdate> {
    if !line.contains("[upgradable from:") {
        return None;
    }
    let mut parts = line.split_whitespace();
    let name_and_repo = parts.next()?;
    let name = name_and_repo.split('/').next()?.to_string();
    let new_version = parts.next()?.to_string();
    // Skip architecture token, then find "from:" and take the next token,
    // stripping the trailing ']'.
    let rest: Vec<&str> = parts.collect();
    let from_idx = rest.iter().position(|t| *t == "from:")?;
    let current_version = rest.get(from_idx + 1)?.trim_end_matches(']').to_string();

    Some(PackageUpdate {
        name,
        current_version,
        new_version,
        source: "apt".to_string(),
    })
}

/// Parses one line of `dpkg -l` output. Only lines whose status field is
/// exactly "ii" (installed) count -- other statuses ("rc" = removed but
/// config files remain, "un" = unknown, etc.) are not currently-installed
/// packages and are skipped, same as header/separator lines that don't
/// start with a 2-letter status code at all.
pub fn parse_dpkg_l_line(line: &str) -> Option<super::InstalledPackage> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 3 || fields[0] != "ii" {
        return None;
    }
    Some(super::InstalledPackage { name: fields[1].to_string(), version: fields[2].to_string() })
}

/// Parses `apt-mark showmanual` output: one package name per line, no
/// version. This is the set of packages the user (or an install script
/// acting on their behalf) explicitly asked for -- everything else `dpkg
/// -l` reports is a transitively-pulled dependency (on a typical desktop,
/// this is the overwhelming majority: verified live, 64 manually-installed
/// vs. 1241 total `dpkg -l` entries on this project's own dev system).
/// Showing all 1241 in an "installed software" list buries the ~60 the
/// user actually cares about.
pub fn parse_manual_names(output: &str) -> HashSet<String> {
    output.lines().map(str::trim).filter(|l| !l.is_empty()).map(str::to_string).collect()
}

impl PackageManager for Apt {
    fn id(&self) -> &'static str {
        "apt"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        let output = subprocess::run_with_timeout("apt", &["list", "--upgradable"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_apt_line).collect())
    }

    /// Restricted to manually-installed packages (see `parse_manual_names`'s
    /// doc comment) -- an "installed software" or "uninstall" list showing
    /// every transitive library dependency alongside real applications is
    /// far less useful than the ~5% of packages the user actually chose.
    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let manual_names = parse_manual_names(&subprocess::run_with_timeout(
            "apt-mark",
            &["showmanual"],
            Duration::from_secs(15),
        )?);
        let output = subprocess::run_with_timeout("dpkg", &["-l"], Duration::from_secs(15))?;
        Ok(output
            .lines()
            .filter_map(parse_dpkg_l_line)
            .filter(|pkg| manual_names.contains(&pkg.name))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_apt_list_upgradable_line() {
        let line = "curl/jammy-updates 7.81.0-1ubuntu1.20 amd64 [upgradable from: 7.81.0-1ubuntu1.19]";
        let update = parse_apt_line(line).expect("should parse");
        assert_eq!(update.name, "curl");
        assert_eq!(update.new_version, "7.81.0-1ubuntu1.20");
        assert_eq!(update.current_version, "7.81.0-1ubuntu1.19");
        assert_eq!(update.source, "apt");
    }

    #[test]
    fn skips_the_listing_header_line() {
        assert!(parse_apt_line("Listing... Done").is_none());
    }

    #[test]
    fn skips_malformed_lines() {
        assert!(parse_apt_line("not a valid apt line at all").is_none());
    }

    #[test]
    fn parses_dpkg_l_installed_line() {
        let line = "ii  adduser                                3.118ubuntu5                            all          add and remove users and groups";
        let pkg = parse_dpkg_l_line(line).expect("should parse");
        assert_eq!(pkg.name, "adduser");
        assert_eq!(pkg.version, "3.118ubuntu5");
    }

    #[test]
    fn skips_non_installed_dpkg_status_lines() {
        assert!(parse_dpkg_l_line("rc  old-package  1.0  all  description").is_none());
    }

    #[test]
    fn skips_dpkg_l_header_lines() {
        assert!(parse_dpkg_l_line("Desired=Unknown/Install/Remove/Purge/Hold").is_none());
        assert!(parse_dpkg_l_line("+++-======================================").is_none());
    }

    #[test]
    fn parses_manual_names_from_apt_mark_showmanual_output() {
        let output = "curl\nfirefox-esr\ngit\n";
        let names = parse_manual_names(output);
        assert_eq!(names.len(), 3);
        assert!(names.contains("curl"));
        assert!(names.contains("firefox-esr"));
        assert!(names.contains("git"));
    }

    #[test]
    fn parse_manual_names_ignores_blank_lines() {
        let names = parse_manual_names("curl\n\n\ngit\n");
        assert_eq!(names.len(), 2);
    }

    #[test]
    fn filtering_dpkg_output_by_manual_names_excludes_transitive_dependencies() {
        // Reproduces the actual reported bug: a dependency-only package
        // (libcurl4, never in apt-mark showmanual's output) must not
        // appear in the filtered result even though dpkg -l reports it as
        // installed, while a manually-installed package must.
        let dpkg_output = "\
ii  curl                                8.5.0-2ubuntu10                         amd64        command line tool for transferring data
ii  libcurl4                            8.5.0-2ubuntu10                         amd64        easy-to-use client-side URL transfer library
";
        let manual_names = parse_manual_names("curl\n");
        let filtered: Vec<crate::packages::InstalledPackage> = dpkg_output
            .lines()
            .filter_map(parse_dpkg_l_line)
            .filter(|pkg| manual_names.contains(&pkg.name))
            .collect();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "curl");
    }
}
