//! Alpine Linux (`apk`). Also the package manager of postmarketOS and of
//! most musl-based container images.
use super::{InstalledPackage, PackageManager, PackageUpdate};
use crate::subprocess;
use std::time::Duration;

pub struct Apk;

/// Splits an apk "pkgver" (`busybox-1.36.1-r5`) into name and version. The
/// version is the last two dash-separated fields (`<ver>-r<rel>`); package
/// names may themselves contain dashes (`py3-requests`).
pub fn split_apk_pkgver(pkgver: &str) -> Option<(String, String)> {
    let rel = pkgver.rfind("-r")?;
    let ver_start = pkgver[..rel].rfind('-')?;
    let name = &pkgver[..ver_start];
    let version = &pkgver[ver_start + 1..];
    if name.is_empty() || !version.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    Some((name.to_string(), version.to_string()))
}

/// One `apk list --upgradable` line:
/// `curl-8.5.0-r0 x86_64 {curl} (curl) [upgradable from: curl-8.4.0-r0]`.
pub fn parse_apk_upgradable_line(line: &str) -> Option<PackageUpdate> {
    let (name, new_version) = split_apk_pkgver(line.split_whitespace().next()?)?;
    let from = line.split("upgradable from: ").nth(1)?.trim_end_matches(']');
    let (_, current_version) = split_apk_pkgver(from.trim())?;
    Some(PackageUpdate { name, current_version, new_version, source: "apk".to_string() })
}

/// One `apk list --installed` line: `busybox-1.36.1-r5 x86_64 {busybox} (GPL-2.0-only) [installed]`.
pub fn parse_apk_installed_line(line: &str) -> Option<InstalledPackage> {
    let (name, version) = split_apk_pkgver(line.split_whitespace().next()?)?;
    Some(InstalledPackage { name, version })
}

impl PackageManager for Apk {
    fn id(&self) -> &'static str {
        "apk"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        let output = subprocess::run_with_timeout("apk", &["list", "--upgradable"], Duration::from_secs(20))?;
        Ok(output.lines().filter_map(parse_apk_upgradable_line).collect())
    }

    /// `apk` keeps the packages the user asked for in /etc/apk/world;
    /// everything else is a dependency. Only those are listed, like the
    /// "explicitly installed" filters of the other managers.
    fn list_installed(&self) -> Result<Vec<InstalledPackage>, String> {
        let output = subprocess::run_with_timeout("apk", &["list", "--installed"], Duration::from_secs(20))?;
        let world = std::fs::read_to_string("/etc/apk/world").unwrap_or_default();
        let wanted: std::collections::HashSet<&str> =
            world.split_whitespace().map(|w| w.split(['<', '>', '=', '~']).next().unwrap_or(w)).collect();
        Ok(output
            .lines()
            .filter_map(parse_apk_installed_line)
            .filter(|p| wanted.is_empty() || wanted.contains(p.name.as_str()))
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_names_that_contain_dashes() {
        assert_eq!(split_apk_pkgver("py3-requests-2.31.0-r1"), Some(("py3-requests".into(), "2.31.0-r1".into())));
        assert_eq!(split_apk_pkgver("busybox-1.36.1-r5"), Some(("busybox".into(), "1.36.1-r5".into())));
        assert_eq!(split_apk_pkgver("nonsense"), None);
    }

    #[test]
    fn parses_an_upgradable_line() {
        let u = parse_apk_upgradable_line("curl-8.5.0-r0 x86_64 {curl} (curl) [upgradable from: curl-8.4.0-r0]").unwrap();
        assert_eq!((u.name.as_str(), u.current_version.as_str(), u.new_version.as_str()), ("curl", "8.4.0-r0", "8.5.0-r0"));
    }

    #[test]
    fn parses_an_installed_line() {
        let p = parse_apk_installed_line("busybox-1.36.1-r5 x86_64 {busybox} (GPL-2.0-only) [installed]").unwrap();
        assert_eq!((p.name.as_str(), p.version.as_str()), ("busybox", "1.36.1-r5"));
    }
}
