use super::{PackageManager, PackageUpdate};
use crate::subprocess;
use std::time::Duration;

pub struct Pacman;

/// Parses one line of `pacman -Qu` output, e.g.: "curl 7.81.0-1 -> 8.4.0-1"
pub fn parse_pacman_line(line: &str) -> Option<PackageUpdate> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() != 4 || fields[2] != "->" {
        return None;
    }
    Some(PackageUpdate {
        name: fields[0].to_string(),
        current_version: fields[1].to_string(),
        new_version: fields[3].to_string(),
        source: "pacman".to_string(),
    })
}

impl PackageManager for Pacman {
    fn id(&self) -> &'static str {
        "pacman"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        // `pacman -Qu` exits non-zero (1) when there are NO updates
        // available (the inverse convention from dnf!) — treat that
        // specific case as an empty, successful list rather than an error.
        match subprocess::run_with_timeout("pacman", &["-Qu"], Duration::from_secs(15)) {
            Ok(output) => Ok(output.lines().filter_map(parse_pacman_line).collect()),
            Err(e) if e.contains("code 1") => Ok(Vec::new()),
            Err(e) => Err(e),
        }
    }

    fn list_installed(&self) -> Result<Vec<super::InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("pacman", &["-Q"], Duration::from_secs(15))?;
        Ok(output.lines().filter_map(parse_pacman_q_line).collect())
    }
}

/// Parses one line of `pacman -Q` output, e.g. "curl 8.4.0-1".
pub fn parse_pacman_q_line(line: &str) -> Option<super::InstalledPackage> {
    let mut parts = line.split_whitespace();
    let name = parts.next()?.to_string();
    let version = parts.next()?.to_string();
    Some(super::InstalledPackage { name, version })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_pacman_qu_line() {
        let line = "curl 7.81.0-1 -> 8.4.0-1";
        let update = parse_pacman_line(line).expect("should parse");
        assert_eq!(update.name, "curl");
        assert_eq!(update.current_version, "7.81.0-1");
        assert_eq!(update.new_version, "8.4.0-1");
        assert_eq!(update.source, "pacman");
    }

    #[test]
    fn skips_lines_without_arrow() {
        assert!(parse_pacman_line("curl 7.81.0-1").is_none());
    }

    #[test]
    fn skips_empty_lines() {
        assert!(parse_pacman_line("").is_none());
    }

    #[test]
    fn parses_pacman_q_line() {
        let line = "curl 8.4.0-1";
        let pkg = super::parse_pacman_q_line(line).expect("should parse");
        assert_eq!(pkg.name, "curl");
        assert_eq!(pkg.version, "8.4.0-1");
    }

    #[test]
    fn skips_blank_pacman_q_lines() {
        assert!(super::parse_pacman_q_line("").is_none());
    }
}
