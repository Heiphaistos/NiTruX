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
    /// `BootOrder:` from efibootmgr, e.g. ["0003", "0002"].
    pub efi_boot_order: Option<Vec<String>>,
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

/// `/etc/default/grub`, then `/etc/default/grub.d/*.cfg` in name order:
/// update-grub sources them after the main file, so on Debian/Ubuntu cloud
/// images a snippet's GRUB_TIMEOUT=0 is what really applies. The parser
/// keeps the last value of each key, exactly like the shell sourcing.
fn read_grub_defaults_from(root: &std::path::Path) -> Option<GrubDefaults> {
    let mut content = std::fs::read_to_string(root.join("etc/default/grub")).ok()?;
    let mut snippets: Vec<_> = std::fs::read_dir(root.join("etc/default/grub.d"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "cfg"))
        .collect();
    snippets.sort();
    for snippet in snippets {
        content.push('\n');
        content.push_str(&std::fs::read_to_string(snippet).unwrap_or_default());
    }
    Some(parse_grub_defaults(&content))
}

fn read_grub_defaults() -> Option<GrubDefaults> {
    read_grub_defaults_from(std::path::Path::new("/"))
}

pub fn parse_boot_order(efibootmgr: &str) -> Option<Vec<String>> {
    let line = efibootmgr.lines().find_map(|l| l.strip_prefix("BootOrder:"))?;
    Some(line.trim().split(',').filter(|e| !e.is_empty()).map(str::to_string).collect())
}

#[tauri::command]
pub fn get_boot_manager_snapshot() -> BootManagerSnapshot {
    let efi = subprocess::run_with_timeout("efibootmgr", &[], Duration::from_secs(5)).ok();
    BootManagerSnapshot {
        grub: read_grub_defaults(),
        efi_entries: efi.as_deref().map(|o| o.lines().filter_map(parse_efibootmgr_line).collect()),
        efi_boot_order: efi.as_deref().and_then(parse_boot_order),
    }
}

// Changes go through the system-tools pkexec action, whose helper re-checks
// every value (digits, 'saved', existing 4-hex EFI entries).
const PKEXEC_SYSTEM_TOOLS: &str = "/usr/bin/nitrux-pkexec-system-tools";
const CHANGE_TIMEOUT: Duration = Duration::from_secs(180); // password + grub regeneration

fn system_tool(args: &[&str]) -> Result<String, String> {
    let mut full = vec![PKEXEC_SYSTEM_TOOLS, "system-tool"];
    full.extend_from_slice(args);
    subprocess::run_with_timeout("pkexec", &full, CHANGE_TIMEOUT)
}

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct GrubEntry {
    pub index: u32,
    pub title: String,
}

/// "N<TAB>title" lines from the helper's grub-entries read.
pub fn parse_grub_entries(out: &str) -> Vec<GrubEntry> {
    out.lines()
        .filter_map(|l| {
            let (n, title) = l.split_once('\t')?;
            Some(GrubEntry { index: n.trim().parse().ok()?, title: title.trim().to_string() })
        })
        .collect()
}

#[tauri::command]
pub fn get_grub_entries() -> Result<Vec<GrubEntry>, String> {
    // grub.cfg is root-only on several distributions: read-only inventory action.
    let out = subprocess::run_with_timeout(
        "pkexec",
        &["/usr/bin/nitrux-pkexec-inventory", "inventory", "grub-entries"],
        Duration::from_secs(60),
    )?;
    Ok(parse_grub_entries(&out))
}

#[tauri::command]
pub fn set_grub_timeout(seconds: u32) -> Result<String, String> {
    if seconds > 60 {
        return Err("délai maximal : 60 secondes".to_string());
    }
    system_tool(&["grub-timeout", &seconds.to_string()])
}

#[tauri::command]
pub fn set_grub_default(entry: String) -> Result<String, String> {
    if entry != "saved" && (entry.is_empty() || !entry.chars().all(|c| c.is_ascii_digit())) {
        return Err(format!("entrée invalide : {entry}"));
    }
    system_tool(&["grub-default", &entry])
}

#[tauri::command]
pub fn grub_reboot_once(index: u32) -> Result<String, String> {
    system_tool(&["grub-reboot", &index.to_string()])
}

pub fn validate_boot_order(order: &[String]) -> Result<String, String> {
    if order.is_empty() || !order.iter().all(|e| e.len() == 4 && e.chars().all(|c| c.is_ascii_hexdigit())) {
        return Err("ordre EFI invalide".to_string());
    }
    Ok(order.join(","))
}

#[tauri::command]
pub fn set_efi_boot_order(order: Vec<String>) -> Result<String, String> {
    let joined = validate_boot_order(&order)?;
    system_tool(&["efi-bootorder", &joined])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_boot_order_and_grub_entries() {
        let efi = "BootCurrent: 0003\nTimeout: 0 seconds\nBootOrder: 0003,0002,2001\nBoot0002* Windows Boot Manager\n";
        assert_eq!(parse_boot_order(efi), Some(vec!["0003".into(), "0002".into(), "2001".into()]));
        assert_eq!(parse_boot_order("BootCurrent: 0003\n"), None);
        // Output captured from the helper on a Debian 13 VM.
        let entries = parse_grub_entries("0\tDebian GNU/Linux\n1\tAdvanced options for Debian GNU/Linux (sous-menu)\n");
        assert_eq!(entries[1], GrubEntry { index: 1, title: "Advanced options for Debian GNU/Linux (sous-menu)".into() });
    }

    #[test]
    fn rejects_bad_values_before_any_password_prompt() {
        assert!(set_grub_timeout(61).is_err());
        assert!(set_grub_default("1;reboot".into()).is_err());
        assert!(set_grub_default(String::new()).is_err());
        assert!(validate_boot_order(&["0001".into(), "zz;r".into()]).is_err());
        assert!(validate_boot_order(&[]).is_err());
        assert_eq!(validate_boot_order(&["0003".into(), "0002".into()]).unwrap(), "0003,0002");
    }

    #[test]
    fn grub_d_snippets_override_the_main_file_in_name_order() {
        // Layout of the Debian 13 cloud image, where the snippet wins.
        let root = std::env::temp_dir().join(format!("nitrux-grubd-{}", std::process::id()));
        std::fs::create_dir_all(root.join("etc/default/grub.d")).unwrap();
        std::fs::write(root.join("etc/default/grub"), "GRUB_DEFAULT=0\nGRUB_TIMEOUT=5\n").unwrap();
        std::fs::write(root.join("etc/default/grub.d/15_timeout.cfg"), "GRUB_TIMEOUT=0\n").unwrap();
        std::fs::write(root.join("etc/default/grub.d/99-nitrux.cfg"), "GRUB_TIMEOUT=7\n").unwrap();
        std::fs::write(root.join("etc/default/grub.d/notes.txt"), "GRUB_TIMEOUT=99\n").unwrap();
        let d = read_grub_defaults_from(&root).unwrap();
        std::fs::remove_dir_all(&root).unwrap();
        assert_eq!(d.timeout_seconds.as_deref(), Some("7"));
        assert_eq!(d.default_entry.as_deref(), Some("0"));
    }

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
