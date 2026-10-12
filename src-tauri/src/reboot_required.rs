//! Whether an update is waiting for a reboot to take effect.

use std::path::Path;

/// Why a reboot is needed, or `None`. Two signals cover every family:
/// Debian/Ubuntu's `/run/reboot-required` (written by the packages that
/// need it), and on every distribution a kernel update, which removes or
/// replaces the running kernel's modules directory -- the next module load
/// (USB stick, VPN...) then fails until the reboot.
fn reboot_reason(root: &Path, running_kernel: &str) -> Option<String> {
    let flag = root.join("run/reboot-required");
    if flag.exists() {
        let pkgs = std::fs::read_to_string(root.join("run/reboot-required.pkgs")).unwrap_or_default();
        let pkgs: Vec<&str> = pkgs.lines().filter(|l| !l.trim().is_empty()).collect();
        return Some(if pkgs.is_empty() {
            "des mises à jour installées demandent un redémarrage".to_string()
        } else {
            format!("des mises à jour installées demandent un redémarrage ({})", pkgs.join(", "))
        });
    }
    let kernel = running_kernel.trim();
    if !kernel.is_empty() && root.join("usr/lib/modules").is_dir() && !root.join("usr/lib/modules").join(kernel).is_dir() {
        return Some(format!("le noyau a été mis à jour (en cours : {kernel}) : redémarrez pour l'utiliser"));
    }
    None
}

#[tauri::command]
pub fn get_reboot_required() -> Option<String> {
    let kernel = std::fs::read_to_string("/proc/sys/kernel/osrelease").unwrap_or_default();
    reboot_reason(Path::new("/"), &kernel)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let root = std::env::temp_dir().join(format!("nitrux-reboot-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("usr/lib/modules/6.1.0")).unwrap();
        std::fs::create_dir_all(root.join("run")).unwrap();
        root
    }

    #[test]
    fn nothing_to_do_when_the_running_kernel_still_has_its_modules() {
        let root = scratch("ok");
        assert_eq!(reboot_reason(&root, "6.1.0\n"), None);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn a_removed_modules_directory_means_the_kernel_was_updated() {
        let root = scratch("kernel");
        assert!(reboot_reason(&root, "6.0.9").unwrap().contains("6.0.9"));
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn debian_flag_lists_the_packages() {
        let root = scratch("deb");
        std::fs::write(root.join("run/reboot-required"), "").unwrap();
        std::fs::write(root.join("run/reboot-required.pkgs"), "linux-image-6.1\nlibc6\n").unwrap();
        assert!(reboot_reason(&root, "6.1.0").unwrap().contains("linux-image-6.1, libc6"));
        std::fs::remove_dir_all(root).unwrap();
    }
}
