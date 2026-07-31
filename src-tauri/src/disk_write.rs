//! Tauri commands for the privileged disk-write operations (format,
//! extend, clone), shelling out to `pkexec` against the dedicated helper
//! binaries installed by `packaging/nitrux-pkexec-helper` (see that
//! script's header comment for the full contract). Each command validates
//! its input here too, even though the helper script re-validates
//! independently as root and never trusts the caller — this Rust-side
//! check exists purely to give the user a clear, immediate error instead
//! of an opaque failure from `pkexec`/the helper for obviously malformed
//! input.
//!
//! This is the highest-risk area of the app: `format_partition` runs
//! `mkfs`, an irreversible data-destroying operation. The validation here
//! is the FIRST line of defense, not the only one — the shell helper
//! re-validates independently as the real safety net (it additionally
//! requires the path to resolve to a real, unmounted block device, which
//! this unprivileged process cannot meaningfully check).

use crate::subprocess;
use std::time::Duration;

const PKEXEC_FORMAT_PARTITION: &str = "/usr/bin/nitrux-pkexec-format-partition";
const PKEXEC_EXTEND_PARTITION: &str = "/usr/bin/nitrux-pkexec-extend-partition";
const PKEXEC_CLONE_DISK: &str = "/usr/bin/nitrux-pkexec-clone-disk";

/// Mirrors the helper script's own device-shape check. A partition device
/// must end in a number distinguishing it from its parent whole-disk
/// device -- accepting a whole-disk path here would let format_partition
/// wipe an entire partition table instead of one filesystem.
///
/// This is a defense-in-depth check, not the sole safety net: the
/// privileged helper script independently re-validates AND additionally
/// requires the path to resolve to a real block device (a check this
/// unprivileged Rust code cannot meaningfully perform, since the caller
/// could be validating a path that doesn't exist yet or isn't accessible
/// pre-authorization). Never assume this function alone makes the
/// downstream `pkexec` call safe.
pub fn validate_partition_device(device: &str) -> Result<(), String> {
    let looks_like_partition = (device.starts_with("/dev/sd") || device.starts_with("/dev/vd"))
        && device.len() > 8
        && device.chars().last().is_some_and(|c| c.is_ascii_digit());
    let looks_like_nvme_partition = device.starts_with("/dev/nvme")
        && device.contains('n')
        && device.contains('p')
        && device.chars().last().is_some_and(|c| c.is_ascii_digit());
    let looks_like_mmc_partition = device.starts_with("/dev/mmcblk")
        && device.contains('p')
        && device.chars().last().is_some_and(|c| c.is_ascii_digit());
    if !(looks_like_partition || looks_like_nvme_partition || looks_like_mmc_partition) {
        return Err(format!(
            "chemin de partition invalide (attendu par ex. /dev/sda1, /dev/nvme0n1p1) : {device}"
        ));
    }
    if !device.chars().all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '-') {
        return Err(format!("le chemin contient des caractères non autorisés : {device}"));
    }
    Ok(())
}

/// Mirrors the helper script's own `validate_disk_device`: a whole-disk
/// path, never a partition. Used to scope `extend_partition`'s parent
/// disk and `clone_disk`'s source, both of which read/resize at the
/// whole-disk level rather than a single filesystem.
pub fn validate_disk_device(device: &str) -> Result<(), String> {
    let is_known_whole_disk_shape = (device.starts_with("/dev/sd") || device.starts_with("/dev/vd"))
        && device.len() == 8
        && device.chars().last().is_some_and(|c| c.is_ascii_alphabetic());
    let is_nvme_whole_disk =
        device.starts_with("/dev/nvme") && device.contains('n') && !device.contains('p');
    let is_mmc_whole_disk = device.starts_with("/dev/mmcblk")
        && !device.contains('p')
        && device.chars().last().is_some_and(|c| c.is_ascii_digit());
    let is_loop_whole_disk = device.starts_with("/dev/loop")
        && device.len() > "/dev/loop".len()
        && device["/dev/loop".len()..].bytes().all(|b| b.is_ascii_digit());
    if !(is_known_whole_disk_shape || is_nvme_whole_disk || is_mmc_whole_disk || is_loop_whole_disk) {
        return Err(format!(
            "chemin de disque invalide (attendu par ex. /dev/sda, /dev/nvme0n1) : {device}"
        ));
    }
    if !device.chars().all(|c| c.is_ascii_alphanumeric() || c == '/' || c == '_' || c == '-') {
        return Err(format!("le chemin contient des caractères non autorisés : {device}"));
    }
    Ok(())
}

pub fn validate_fstype(fstype: &str) -> Result<(), String> {
    match fstype {
        "ext4" | "btrfs" | "xfs" | "vfat" => Ok(()),
        other => Err(format!("système de fichiers non pris en charge : {other}")),
    }
}

pub fn validate_partition_number(n: &str) -> Result<(), String> {
    if n.is_empty() || !n.bytes().all(|b| b.is_ascii_digit()) {
        return Err(format!("numéro de partition invalide : {n}"));
    }
    Ok(())
}

/// Mirrors the helper script's own `validate_image_dest_path`.
pub fn validate_image_dest_path(path: &str) -> Result<(), String> {
    if path.is_empty() {
        return Err("chemin de destination vide".to_string());
    }
    if !path.starts_with('/') {
        return Err(format!("le chemin doit être absolu : {path}"));
    }
    if path.contains("..") {
        return Err(format!("le chemin ne doit pas contenir '..' : {path}"));
    }
    const DISALLOWED: [char; 10] = ['*', '?', '$', '`', ';', '|', '&', '<', '>', '\n'];
    if path.chars().any(|c| DISALLOWED.contains(&c)) {
        return Err(format!("le chemin contient des caractères non autorisés : {path}"));
    }
    Ok(())
}

/// Formats `device` with `fstype`. IRREVERSIBLE -- destroys all data on
/// the target partition. Both arguments are validated here AND
/// independently re-validated inside the privileged helper script, which
/// additionally refuses a mounted or root-adjacent device at execution
/// time (a check that cannot be meaningfully done from this unprivileged
/// process, since mount state can change between validation and
/// execution).
#[tauri::command]
pub fn format_partition(device: String, fstype: String) -> Result<String, String> {
    validate_partition_device(&device)?;
    validate_fstype(&fstype)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_FORMAT_PARTITION, "format-partition", &device, &fstype],
        Duration::from_secs(300),
    )
}

/// Grows `device` (a partition) to fill trailing free space on `disk` at
/// partition number `part_number`, then grows its filesystem to match.
/// Growing-only: never shrinks, never touches existing data.
#[tauri::command]
pub fn extend_partition(device: String, disk: String, part_number: String) -> Result<String, String> {
    validate_partition_device(&device)?;
    validate_disk_device(&disk)?;
    validate_partition_number(&part_number)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_EXTEND_PARTITION, "extend-partition", &device, &disk, &part_number],
        Duration::from_secs(300),
    )
}

/// Clones `source_disk` to the image file at `dest_path`. Non-destructive
/// to the source; a large, slow, real disk-read operation, hence the very
/// generous timeout (a full disk clone can take a long time depending on
/// disk size -- this mirrors the reasoning behind other generous timeouts
/// elsewhere in this codebase, e.g. upgrade_all_packages/create_snapshot).
#[tauri::command]
pub fn clone_disk(source_disk: String, dest_path: String) -> Result<String, String> {
    validate_disk_device(&source_disk)?;
    validate_image_dest_path(&dest_path)?;
    subprocess::run_with_timeout(
        "pkexec",
        &[PKEXEC_CLONE_DISK, "clone-disk", &source_disk, &dest_path],
        Duration::from_secs(3600),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_well_formed_partition_devices() {
        assert!(validate_partition_device("/dev/sda1").is_ok());
        assert!(validate_partition_device("/dev/nvme0n1p1").is_ok());
        assert!(validate_partition_device("/dev/vdb2").is_ok());
    }

    #[test]
    fn rejects_whole_disk_devices() {
        assert!(validate_partition_device("/dev/sda").is_err());
        assert!(validate_partition_device("/dev/nvme0n1").is_err());
    }

    #[test]
    fn rejects_malformed_or_malicious_partition_device() {
        assert!(validate_partition_device("").is_err());
        assert!(validate_partition_device("/dev/sda1; rm -rf /").is_err());
        assert!(validate_partition_device("sda1").is_err());
        assert!(validate_partition_device("/etc/passwd").is_err());
    }

    #[test]
    fn accepts_supported_fstypes() {
        assert!(validate_fstype("ext4").is_ok());
        assert!(validate_fstype("btrfs").is_ok());
        assert!(validate_fstype("xfs").is_ok());
        assert!(validate_fstype("vfat").is_ok());
    }

    #[test]
    fn rejects_unsupported_fstype() {
        assert!(validate_fstype("ntfs").is_err());
        assert!(validate_fstype("").is_err());
        assert!(validate_fstype("ext4; rm -rf /").is_err());
    }

    #[test]
    fn accepts_well_formed_disk_devices() {
        assert!(validate_disk_device("/dev/sda").is_ok());
        assert!(validate_disk_device("/dev/nvme0n1").is_ok());
    }

    #[test]
    fn rejects_partition_device_as_disk_device() {
        assert!(validate_disk_device("/dev/sda1").is_err());
    }

    #[test]
    fn accepts_well_formed_image_dest_path() {
        assert!(validate_image_dest_path("/home/dev/backup.img").is_ok());
        assert!(validate_image_dest_path("/mnt/backups/disk.img").is_ok());
    }

    #[test]
    fn rejects_malformed_image_dest_path() {
        assert!(validate_image_dest_path("").is_err());
        assert!(validate_image_dest_path("relative.img").is_err());
        assert!(validate_image_dest_path("/tmp/../etc/shadow").is_err());
        assert!(validate_image_dest_path("/tmp/evil;rm -rf /").is_err());
    }

    #[test]
    fn accepts_well_formed_partition_number() {
        assert!(validate_partition_number("1").is_ok());
        assert!(validate_partition_number("12").is_ok());
    }

    #[test]
    fn rejects_malformed_partition_number() {
        assert!(validate_partition_number("").is_err());
        assert!(validate_partition_number("0x1").is_err());
        assert!(validate_partition_number("1; rm -rf /").is_err());
    }
}
