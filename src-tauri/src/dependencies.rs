use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct MissingDependency {
    pub binary: String,
    pub missing_library: String,
}

/// A small, fixed set of common system binaries -- scanning every
/// executable on the system would be slow and mostly redundant (most
/// binaries share the same handful of core libraries); this list mirrors
/// the kind of "did something break after an update" spot-check NiTriTe's
/// own Dépendances page performs, not an exhaustive audit.
const BINARIES_TO_CHECK: &[&str] = &[
    "/bin/bash", "/bin/ls", "/usr/bin/apt", "/usr/bin/systemctl",
    "/usr/bin/python3", "/usr/bin/curl", "/usr/bin/git",
];

/// Parses one line of `ldd <binary>` output, returning the missing library
/// name if this line reports one (`=> not found`), or `None` for a
/// resolved dependency / the dynamic linker line / a malformed line.
pub fn parse_ldd_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    if !trimmed.ends_with("=> not found") {
        return None;
    }
    trimmed.split_whitespace().next().map(|s| s.to_string())
}

fn check_binary(binary: &str) -> Vec<MissingDependency> {
    if !std::path::Path::new(binary).exists() {
        return Vec::new();
    }
    let Ok(output) = subprocess::run_with_timeout("ldd", &[binary], Duration::from_secs(5)) else {
        return Vec::new();
    };
    output
        .lines()
        .filter_map(parse_ldd_line)
        .map(|missing_library| MissingDependency { binary: binary.to_string(), missing_library })
        .collect()
}

#[tauri::command]
pub fn scan_missing_dependencies() -> Vec<MissingDependency> {
    BINARIES_TO_CHECK.iter().flat_map(|b| check_binary(b)).collect()
}

/// One external tool NiTruX shells out to, and whether this system has it.
#[derive(Serialize, Clone)]
pub struct ToolStatus {
    pub binary: String,
    /// Package that provides it on *this* distribution, when the family is
    /// known -- that is what makes the "Installer" button possible.
    pub package: Option<String>,
    pub feature: String,
    pub installed: bool,
}

/// Inventory of every external tool NiTruX depends on, installed or not.
///
/// NiTruX deliberately declares almost none of these as hard package
/// dependencies (a user with no printer should not be forced to install
/// CUPS), so on a fresh system a large share of the pages depend on tools
/// that simply are not there. Without this inventory the only way to find
/// out was to open each page and read an error -- which is exactly what
/// "three quarters of the app does not work" looks like from the outside.
#[tauri::command]
pub fn check_required_tools() -> Vec<ToolStatus> {
    subprocess::EXTERNAL_TOOLS
        .iter()
        .map(|t| ToolStatus {
            binary: t.binary.to_string(),
            package: subprocess::package_for_this_system(t.binary).map(str::to_string),
            feature: t.feature.to_string(),
            installed: subprocess::binary_in_path(t.binary),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports_every_known_tool_with_its_feature_label() {
        let tools = check_required_tools();
        assert_eq!(tools.len(), subprocess::EXTERNAL_TOOLS.len());
        // `sh` is guaranteed present anywhere this test can run, so at least
        // one entry must report installed -- guards against a PATH-scan that
        // silently always answers false.
        assert!(
            tools.iter().any(|t| t.installed),
            "no tool detected at all -- PATH scan is probably broken"
        );
        assert!(tools.iter().all(|t| !t.feature.is_empty()));
    }

    #[test]
    fn parses_a_missing_dependency_line() {
        let line = "\tlibfoo.so.3 => not found";
        assert_eq!(parse_ldd_line(line), Some("libfoo.so.3".to_string()));
    }

    #[test]
    fn ignores_a_resolved_dependency_line() {
        let line = "\tlibc.so.6 => /lib/x86_64-linux-gnu/libc.so.6 (0x00007f0000000000)";
        assert_eq!(parse_ldd_line(line), None);
    }

    #[test]
    fn ignores_the_dynamic_linker_line() {
        let line = "\t/lib64/ld-linux-x86-64.so.2 (0x00007f0000000000)";
        assert_eq!(parse_ldd_line(line), None);
    }

    #[test]
    fn ignores_the_vdso_line() {
        let line = "\tlinux-vdso.so.1 (0x00007ffd00000000)";
        assert_eq!(parse_ldd_line(line), None);
    }
}
