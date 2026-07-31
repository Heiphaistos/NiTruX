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
}
