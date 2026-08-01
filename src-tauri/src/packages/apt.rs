use super::{PackageManager, PackageUpdate};
use crate::subprocess;
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

impl PackageManager for Apt {
    fn id(&self) -> &'static str {
        "apt"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        let output = subprocess::run_with_timeout("apt", &["list", "--upgradable"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_apt_line).collect())
    }

    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("dpkg", &["-l"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_dpkg_l_line).collect())
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
}
