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

#[cfg(test)]
mod tests {
    use super::*;

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
