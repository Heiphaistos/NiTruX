use serde::Serialize;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct CacheSizeReport {
    pub user_cache_bytes: u64,
    pub package_cache_bytes: Option<u64>,
}

/// Recursively sums file sizes under `path`. Missing directories, and any
/// individual file/subdirectory that can't be read (permission denied,
/// broken symlink, race with concurrent deletion), are silently skipped
/// rather than failing the whole walk -- a best-effort size estimate is
/// more useful here than an all-or-nothing error, since this is purely
/// informational (no cleanup decision is made automatically from it).
pub fn directory_size_bytes(path: &Path) -> u64 {
    let Ok(entries) = std::fs::read_dir(path) else {
        return 0;
    };
    let mut total = 0u64;
    for entry in entries.flatten() {
        let Ok(metadata) = entry.metadata() else { continue };
        if metadata.is_dir() {
            total += directory_size_bytes(&entry.path());
        } else {
            total += metadata.len();
        }
    }
    total
}

fn package_cache_dir() -> Option<&'static str> {
    if Path::new("/var/cache/apt/archives").is_dir() {
        Some("/var/cache/apt/archives")
    } else if Path::new("/var/cache/dnf").is_dir() {
        Some("/var/cache/dnf")
    } else if Path::new("/var/cache/pacman/pkg").is_dir() {
        Some("/var/cache/pacman/pkg")
    } else if Path::new("/var/cache/zypp").is_dir() {
        Some("/var/cache/zypp")
    } else {
        None
    }
}

#[tauri::command]
pub fn get_cache_size_report() -> CacheSizeReport {
    let home = std::env::var("HOME").unwrap_or_default();
    let user_cache_bytes = directory_size_bytes(Path::new(&home).join(".cache").as_path());
    // Package manager cache dirs are typically root-owned and NOT readable
    // by an unprivileged user (e.g. /var/cache/apt/archives is root:root
    // 0700 on Debian) -- `directory_size_bytes`'s permission-denied handling
    // means this naturally reports 0 rather than erroring in that case, so
    // the frontend distinguishes "no known cache dir for this distro"
    // (`None`) from "cache dir exists but unreadable, reports as ~0 bytes"
    // (`Some(0)`) only loosely -- both are honestly represented as "0 o"
    // to the user rather than a confusing permission error for a read-only
    // informational number.
    let package_cache_bytes = package_cache_dir().map(|dir| directory_size_bytes(Path::new(dir)));

    CacheSizeReport { user_cache_bytes, package_cache_bytes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn sums_file_sizes_in_a_flat_directory() {
        let dir = std::env::temp_dir().join(format!("nitrux-test-cache-size-{}", std::process::id()));
        fs::create_dir_all(&dir).unwrap();
        fs::write(dir.join("a.txt"), b"12345").unwrap();
        fs::write(dir.join("b.txt"), b"1234567890").unwrap();
        let total = directory_size_bytes(&dir);
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(total, 15);
    }

    #[test]
    fn recurses_into_subdirectories() {
        let dir = std::env::temp_dir().join(format!("nitrux-test-cache-size-nested-{}", std::process::id()));
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(dir.join("sub").join("c.txt"), b"abc").unwrap();
        let total = directory_size_bytes(&dir);
        fs::remove_dir_all(&dir).unwrap();
        assert_eq!(total, 3);
    }

    #[test]
    fn returns_zero_for_a_nonexistent_directory_rather_than_erroring() {
        let dir = std::env::temp_dir().join("nitrux-test-cache-size-does-not-exist");
        assert_eq!(directory_size_bytes(&dir), 0);
    }
}
