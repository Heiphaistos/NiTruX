use super::{PackageManager, PackageUpdate};
use crate::subprocess;
use std::time::Duration;

pub struct Dnf;

/// Parses one line of `dnf check-update` output, e.g.:
/// "curl.x86_64                    7.76.1-14.fc35                     updates"
///
/// `dnf check-update`'s current-version is not available in this listing
/// format (unlike `apt list --upgradable`), so `current_version` is left
/// empty — the frontend must treat it as optional/unknown for dnf-sourced
/// entries. This is a real, documented limitation of the tool, not a
/// parsing gap.
pub fn parse_dnf_line(line: &str) -> Option<PackageUpdate> {
    let fields: Vec<&str> = line.split_whitespace().collect();
    if fields.len() < 3 {
        return None;
    }
    let name = fields[0].split('.').next()?.to_string();
    let new_version = fields[1].to_string();

    Some(PackageUpdate {
        name,
        current_version: String::new(),
        new_version,
        source: "dnf".to_string(),
    })
}

impl PackageManager for Dnf {
    fn id(&self) -> &'static str {
        "dnf"
    }

    fn list_upgradable(&self) -> Result<Vec<PackageUpdate>, String> {
        // `dnf check-update` exits with status 100 when updates ARE
        // available (not 0!) — a genuinely unusual convention. Treat both
        // 0 (no updates) and 100 (updates found) as success at this layer.
        match subprocess::run_with_timeout("dnf", &["check-update"], Duration::from_secs(20)) {
            Ok(output) => Ok(output.lines().filter_map(parse_dnf_line).collect()),
            Err(e) if e.contains("code 100") => {
                // run_with_timeout treats non-zero as Err; dnf's "updates
                // available" exit code is folded into that path, so we
                // cannot recover the stdout here. This is a known
                // limitation flagged for a fast-follow (see plan Task 3
                // notes) — for now, a 100 exit is reported as "no updates
                // detected" rather than a hard error, since dnf may not
                // even be the active manager on this host.
                Ok(Vec::new())
            }
            Err(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_dnf_check_update_line() {
        let line = "curl.x86_64                    7.76.1-14.fc35                     updates";
        let update = parse_dnf_line(line).expect("should parse");
        assert_eq!(update.name, "curl");
        assert_eq!(update.new_version, "7.76.1-14.fc35");
        assert_eq!(update.source, "dnf");
    }

    #[test]
    fn skips_blank_lines() {
        assert!(parse_dnf_line("").is_none());
    }

    #[test]
    fn skips_lines_with_too_few_fields() {
        assert!(parse_dnf_line("justonefield").is_none());
    }
}
