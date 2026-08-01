use super::{PackageManager, PackageUpdate};
use crate::subprocess;
use std::time::Duration;

pub struct Zypper;

/// Parses one line of `zypper list-updates` output, e.g.:
/// "v | curl | package | 8.4.0-1.1 | 7.81.0-1.1 | x86_64"
/// (Status | Name | Type | Available Version | Installed Version | Arch)
pub fn parse_zypper_line(line: &str) -> Option<PackageUpdate> {
    let fields: Vec<&str> = line.split('|').map(|f| f.trim()).collect();
    if fields.len() != 6 || fields[0] != "v" {
        return None;
    }
    Some(PackageUpdate {
        name: fields[1].to_string(),
        new_version: fields[3].to_string(),
        current_version: fields[4].to_string(),
        source: "zypper".to_string(),
    })
}

impl PackageManager for Zypper {
    fn id(&self) -> &'static str {
        "zypper"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        let output = subprocess::run_with_timeout("zypper", &["list-updates"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_zypper_line).collect())
    }

    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("zypper", &["se", "--installed-only"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_zypper_installed_line).collect())
    }
}

/// Parses one line of `zypper se --installed-only` output, e.g.:
/// "i | curl | package | x86_64 | Main Repository"
/// (Status | Name | Type | Arch | Repository -- note this has 5 fields,
/// one fewer than `list-updates`'s 6, and no version column at all; zypper
/// requires a separate `zypper info <pkg>` call per package for version,
/// which this scan deliberately skips for performance -- version is left
/// empty for zypper-sourced installed packages, a real, documented
/// limitation, not a parsing gap, mirroring dnf.rs's existing precedent
/// for `list_upgradable`'s own current_version limitation.)
pub fn parse_zypper_installed_line(line: &str) -> Option<super::InstalledPackage> {
    let fields: Vec<&str> = line.split('|').map(|f| f.trim()).collect();
    if fields.len() != 5 || fields[0] != "i" {
        return None;
    }
    Some(super::InstalledPackage { name: fields[1].to_string(), version: String::new() })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_zypper_lu_line() {
        let line = "v | curl | package | 8.4.0-1.1 | 7.81.0-1.1 | x86_64";
        let update = parse_zypper_line(line).expect("should parse");
        assert_eq!(update.name, "curl");
        assert_eq!(update.new_version, "8.4.0-1.1");
        assert_eq!(update.current_version, "7.81.0-1.1");
        assert_eq!(update.source, "zypper");
    }

    #[test]
    fn skips_header_and_separator_lines() {
        assert!(parse_zypper_line("S | Name | Type | Available | Installed | Arch").is_none());
        assert!(parse_zypper_line("--+------+------+-----------+-----------+------").is_none());
    }

    #[test]
    fn skips_lines_with_wrong_field_count() {
        assert!(parse_zypper_line("v | curl | package").is_none());
    }

    #[test]
    fn parses_zypper_installed_line() {
        let line = "i | curl | package | x86_64 | Main Repository";
        let pkg = super::parse_zypper_installed_line(line).expect("should parse");
        assert_eq!(pkg.name, "curl");
        assert_eq!(pkg.version, "");
    }

    #[test]
    fn skips_zypper_installed_header_and_separator_lines() {
        assert!(super::parse_zypper_installed_line("S | Name | Type | Arch | Repository").is_none());
        assert!(super::parse_zypper_installed_line("--+------+------+------+------------").is_none());
    }
}
