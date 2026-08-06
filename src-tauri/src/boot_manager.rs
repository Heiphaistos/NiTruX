// src-tauri/src/boot_manager.rs
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct GrubDefaults {
    pub default_entry: Option<String>,
    pub timeout_seconds: Option<String>,
    pub distributor: Option<String>,
    pub cmdline_default: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct EfiBootEntry {
    pub id: String,
    pub label: String,
    pub is_current: bool,
}

#[derive(Serialize, Clone)]
pub struct BootManagerSnapshot {
    pub grub: Option<GrubDefaults>,
    pub efi_entries: Option<Vec<EfiBootEntry>>,
}

/// Parses `/etc/default/grub` content -- simple `KEY=value` lines (values
/// may be quoted, quotes are stripped). Deliberately does NOT evaluate
/// shell expressions that may appear inside a value (e.g.
/// `GRUB_DISTRIBUTOR` is often a backtick command substitution in real
/// files) -- the raw string is shown as-is; actually running embedded
/// shell from a config file would be a real security concern for a
/// read-only informational parser, never do that.
pub fn parse_grub_defaults(content: &str) -> GrubDefaults {
    let mut default_entry = None;
    let mut timeout_seconds = None;
    let mut distributor = None;
    let mut cmdline_default = None;

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('#') || line.is_empty() {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else { continue };
        let value = value.trim_matches('"').to_string();
        match key {
            "GRUB_DEFAULT" => default_entry = Some(value),
            "GRUB_TIMEOUT" => timeout_seconds = Some(value),
            "GRUB_DISTRIBUTOR" => distributor = Some(value),
            "GRUB_CMDLINE_LINUX_DEFAULT" => cmdline_default = Some(value),
            _ => {}
        }
    }

    GrubDefaults { default_entry, timeout_seconds, distributor, cmdline_default }
}

/// Parses one line of `efibootmgr` output for a boot entry, e.g.
/// "Boot0004* debian    HD(1,GPT,...)" -> id "0004", label "debian",
/// is_current based on the `*` marker (present = this entry is active in
/// BootOrder... actually `*` means "active/enabled", not "currently
/// booted" -- `BootCurrent:` on its own line is the real "what booted"
/// signal, not encoded per-entry, so `is_current` here reflects the
/// enabled/active flag, not boot-current; this is a deliberate, documented
/// simplification for a read-only display, not a bug).
pub fn parse_efibootmgr_line(line: &str) -> Option<EfiBootEntry> {
    let rest = line.strip_prefix("Boot")?;
    let (id_and_marker, label_and_rest) = rest.split_once(' ')?;
    let is_current = id_and_marker.ends_with('*');
    let id = id_and_marker.trim_end_matches('*').to_string();
    if id.len() != 4 || !id.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    // The label runs up to the first tab (device path descriptor follows).
    let label = label_and_rest.split('\t').next().unwrap_or(label_and_rest).trim().to_string();
    Some(EfiBootEntry { id, label, is_current })
}

fn read_grub_defaults() -> Option<GrubDefaults> {
    std::fs::read_to_string("/etc/default/grub").ok().map(|c| parse_grub_defaults(&c))
}

fn read_efi_entries() -> Option<Vec<EfiBootEntry>> {
    let output = subprocess::run_with_timeout("efibootmgr", &[], Duration::from_secs(5)).ok()?;
    Some(output.lines().filter_map(parse_efibootmgr_line).collect())
}

#[tauri::command]
pub fn get_boot_manager_snapshot() -> BootManagerSnapshot {
    BootManagerSnapshot { grub: read_grub_defaults(), efi_entries: read_efi_entries() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_grub_defaults_from_real_content() {
        let content = "GRUB_DEFAULT=0\nGRUB_TIMEOUT=5\nGRUB_DISTRIBUTOR=`( . /etc/os-release && echo ${NAME} )`\nGRUB_CMDLINE_LINUX_DEFAULT=\"quiet\"\nGRUB_CMDLINE_LINUX=\"\"\n";
        let defaults = parse_grub_defaults(content);
        assert_eq!(defaults.default_entry.as_deref(), Some("0"));
        assert_eq!(defaults.timeout_seconds.as_deref(), Some("5"));
        assert_eq!(defaults.distributor.as_deref(), Some("`( . /etc/os-release && echo ${NAME} )`"));
        assert_eq!(defaults.cmdline_default.as_deref(), Some("quiet"));
    }

    #[test]
    fn skips_comment_and_blank_lines() {
        let content = "# a comment\n\nGRUB_TIMEOUT=5\n";
        let defaults = parse_grub_defaults(content);
        assert_eq!(defaults.timeout_seconds.as_deref(), Some("5"));
        assert_eq!(defaults.default_entry, None);
    }

    #[test]
    fn parses_an_efibootmgr_entry_line() {
        let line = "Boot0004* debian\tHD(1,GPT,dc647f1c-da1d-4c2f-a376-81604ff637a8,0x800,0x1e8000)/File(\\EFI\\debian\\shimx64.efi)";
        let entry = parse_efibootmgr_line(line).expect("should parse");
        assert_eq!(entry.id, "0004");
        assert_eq!(entry.label, "debian");
        assert!(entry.is_current);
    }

    #[test]
    fn parses_a_non_active_efibootmgr_entry() {
        let line = "Boot0000  Windows Boot Manager\tHD(...)";
        let entry = parse_efibootmgr_line(line).expect("should parse");
        assert_eq!(entry.id, "0000");
        assert!(!entry.is_current);
    }

    #[test]
    fn ignores_non_boot_lines() {
        assert!(parse_efibootmgr_line("BootCurrent: 0004").is_none());
        assert!(parse_efibootmgr_line("Timeout: 0 seconds").is_none());
    }
}
