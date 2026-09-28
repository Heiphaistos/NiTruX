//! Void Linux (`xbps`).
use super::{InstalledPackage, PackageManager, PackageUpdate};
use crate::subprocess;
use std::time::Duration;

pub struct Xbps;

/// Splits an xbps pkgver (`bash-5.2.21_1`): the version follows the LAST
/// dash and always ends in `_<revision>`.
pub fn split_xbps_pkgver(pkgver: &str) -> Option<(String, String)> {
    let dash = pkgver.rfind('-')?;
    let (name, version) = (&pkgver[..dash], &pkgver[dash + 1..]);
    if name.is_empty() || !version.contains('_') {
        return None;
    }
    Some((name.to_string(), version.to_string()))
}

/// One `xbps-install -Mun` line: `curl-8.5.0_1 update x86_64 https://repo...`.
/// The dry run does not print the installed version.
pub fn parse_xbps_update_line(line: &str) -> Option<PackageUpdate> {
    let mut fields = line.split_whitespace();
    let (name, new_version) = split_xbps_pkgver(fields.next()?)?;
    if fields.next()? != "update" {
        return None;
    }
    Some(PackageUpdate { name, current_version: String::new(), new_version, source: "xbps".to_string() })
}

impl PackageManager for Xbps {
    fn id(&self) -> &'static str {
        "xbps"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        let output = subprocess::run_with_timeout("xbps-install", &["-Mun"], Duration::from_secs(30))?;
        Ok(output.lines().filter_map(parse_xbps_update_line).collect())
    }

    /// `-m` lists only manually installed packages.
    fn list_installed(&self) -> Result<Vec<InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("xbps-query", &["-m"], Duration::from_secs(20))?;
        Ok(output
            .lines()
            .filter_map(|l| split_xbps_pkgver(l.trim()))
            .map(|(name, version)| InstalledPackage { name, version })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_pkgver() {
        assert_eq!(split_xbps_pkgver("xorg-server-21.1.11_1"), Some(("xorg-server".into(), "21.1.11_1".into())));
        assert_eq!(split_xbps_pkgver("nonsense"), None);
    }

    #[test]
    fn parses_update_lines() {
        let u = parse_xbps_update_line("curl-8.5.0_1 update x86_64 https://repo-default.voidlinux.org/current").unwrap();
        assert_eq!((u.name.as_str(), u.new_version.as_str()), ("curl", "8.5.0_1"));
        assert!(parse_xbps_update_line("curl-8.5.0_1 install x86_64 repo").is_none());
    }
}
