//! Writing an image file onto a whole disk: an ISO onto a USB stick
//! (bootable key) or a "Cloner" image back onto a disk. Both go through the
//! `disk-image` pkexec action; the helper re-checks everything and refuses
//! the running system's disk and any disk with something mounted.

use crate::disk_write::{validate_disk_device, validate_image_dest_path};
use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

const PKEXEC_DISK_IMAGE: &str = "/usr/bin/nitrux-pkexec-disk-image";

#[derive(Serialize, Clone, Debug, PartialEq)]
pub struct WholeDisk {
    pub path: String,
    pub size_bytes: u64,
    pub model: String,
    /// "usb", "nvme", "sata"... empty when unknown.
    pub transport: String,
}

/// `lsblk -J -b -d -o PATH,SIZE,MODEL,TRAN,TYPE`: whole disks only (no
/// zram/loop/rom), sizes in bytes.
fn parse_lsblk_disks(json: &str) -> Vec<WholeDisk> {
    let Ok(value) = serde_json::from_str::<serde_json::Value>(json) else { return Vec::new() };
    value
        .get("blockdevices")
        .and_then(|d| d.as_array())
        .into_iter()
        .flatten()
        .filter(|d| d.get("type").and_then(|t| t.as_str()) == Some("disk"))
        .filter_map(|d| {
            let path = d.get("path")?.as_str()?.to_string();
            if path.starts_with("/dev/zram") {
                return None;
            }
            let size_bytes = d.get("size").and_then(|s| s.as_u64().or_else(|| s.as_str()?.parse().ok())).unwrap_or(0);
            let text = |k: &str| d.get(k).and_then(|v| v.as_str()).unwrap_or("").trim().to_string();
            Some(WholeDisk { path, size_bytes, model: text("model"), transport: text("tran") })
        })
        .collect()
}

#[tauri::command]
pub fn list_whole_disks() -> Result<Vec<WholeDisk>, String> {
    let out = subprocess::run_with_timeout("lsblk", &["-J", "-b", "-d", "-o", "PATH,SIZE,MODEL,TRAN,TYPE"], Duration::from_secs(10))?;
    Ok(parse_lsblk_disks(&out))
}

fn validate_source(image: &str) -> Result<(), String> {
    validate_image_dest_path(image)?;
    if !std::path::Path::new(image).is_file() {
        return Err(format!("fichier image introuvable : {image}"));
    }
    Ok(())
}

/// Hours for a large image on a slow stick; includes typing the password.
const WRITE_TIMEOUT: Duration = Duration::from_secs(4 * 3600);

#[tauri::command]
pub async fn write_iso_to_usb(iso: String, device: String) -> Result<String, String> {
    validate_source(&iso)?;
    validate_disk_device(&device)?;
    tauri::async_runtime::spawn_blocking(move || {
        subprocess::run_with_timeout("pkexec", &[PKEXEC_DISK_IMAGE, "write-iso", &iso, &device], WRITE_TIMEOUT)
    })
    .await
    .map_err(|e| format!("tâche interrompue : {e}"))?
}

#[tauri::command]
pub async fn restore_disk_image(image: String, device: String) -> Result<String, String> {
    validate_source(&image)?;
    validate_disk_device(&device)?;
    tauri::async_runtime::spawn_blocking(move || {
        subprocess::run_with_timeout("pkexec", &[PKEXEC_DISK_IMAGE, "restore-image", &image, &device], WRITE_TIMEOUT)
    })
    .await
    .map_err(|e| format!("tâche interrompue : {e}"))?
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_real_disks_and_drops_zram() {
        // Shape of `lsblk -J -b -d -o PATH,SIZE,MODEL,TRAN,TYPE` on Omarchy, plus a USB stick.
        let json = r#"{"blockdevices":[
            {"path":"/dev/zram0","size":7948206080,"model":null,"tran":null,"type":"disk"},
            {"path":"/dev/nvme0n1","size":512110190592,"model":"KBG40ZNV512G KIOXIA","tran":"nvme","type":"disk"},
            {"path":"/dev/sdb","size":"16008609792","model":"SanDisk Ultra ","tran":"usb","type":"disk"},
            {"path":"/dev/sr0","size":1073741312,"model":"DVD","tran":"sata","type":"rom"}
        ]}"#;
        let disks = parse_lsblk_disks(json);
        assert_eq!(disks.len(), 2);
        assert_eq!(disks[1], WholeDisk { path: "/dev/sdb".into(), size_bytes: 16008609792, model: "SanDisk Ultra".into(), transport: "usb".into() });
    }

    #[test]
    fn refuses_bad_sources_and_partitions_before_any_prompt() {
        assert!(validate_source("relative.iso").is_err());
        assert!(validate_source("/tmp/../etc/shadow").is_err());
        assert!(validate_source("/nonexistent/debian.iso").is_err());
        assert!(validate_disk_device("/dev/sdb1").is_err());
    }
}
