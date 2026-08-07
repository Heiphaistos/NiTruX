use crate::subprocess;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Serialize, Clone)]
pub struct Partition {
    pub name: String,
    pub size: String,
    pub mountpoint: Option<String>,
}

#[derive(Serialize, Clone)]
pub struct Disk {
    pub name: String,
    pub size: String,
    pub partitions: Vec<Partition>,
}

#[derive(Serialize, Clone)]
pub struct UsageEntry {
    pub mountpoint: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub used_percent: u8,
}

#[derive(Deserialize)]
struct RawBlockDevice {
    name: String,
    size: String,
    #[serde(rename = "type")]
    device_type: String,
    mountpoint: Option<String>,
    #[serde(default)]
    children: Vec<RawBlockDevice>,
}

#[derive(Deserialize)]
struct RawLsblkOutput {
    blockdevices: Vec<RawBlockDevice>,
}

pub fn parse_lsblk_json(json: &str) -> Option<Vec<Disk>> {
    let raw: RawLsblkOutput = serde_json::from_str(json).ok()?;
    Some(
        raw.blockdevices
            .into_iter()
            .filter(|d| d.device_type == "disk")
            .map(|d| Disk {
                name: d.name,
                size: d.size,
                partitions: d
                    .children
                    .into_iter()
                    .map(|c| Partition {
                        name: c.name,
                        size: c.size,
                        mountpoint: c.mountpoint,
                    })
                    .collect(),
            })
            .collect(),
    )
}

/// Parses one line of `df --output=source,size,used,avail,pcent,target` output
/// (POSIX `-P` block-size defaults to 1K blocks), e.g.:
/// "/dev/sda1  536870912  214748364  322122548  40%  /"
///
/// The mountpoint (last column) may itself contain spaces (e.g. removable
/// media mounted by volume label, "/media/user/My Passport") -- it is taken
/// as everything remaining after the first 5 fixed-width columns, not a
/// single whitespace-delimited token.
///
/// Only sources starting with `/dev/` are real block devices -- reproduced
/// live (`df -k -P` on this project's own dev machine) that unfiltered `df`
/// output is dominated by pseudo-filesystems with non-`/dev/` sources
/// (`tmpfs`, `none`, `rootfs`, `overlay`, ...), several of which are
/// actively misleading here: every mounted Snap package shows up as a
/// `snapfuse` source permanently pinned at "100%" (a read-only squashfs
/// image is always exactly as full as its own size), which a user glancing
/// at a "Disques & partitions" usage list would very reasonably read as "my
/// disk is full" rather than "an installed Snap app exists". Mirrors
/// `list_disks`'s own `device_type == "disk"` filter, which already excludes
/// non-disk entries from `lsblk` -- this closes the equivalent gap on the
/// `df`-based usage view, which had no filter at all.
pub fn parse_df_line(line: &str) -> Option<UsageEntry> {
    let mut fields = line.split_whitespace();
    let source = fields.next()?;
    if !source.starts_with("/dev/") {
        return None;
    }
    let total_kb: u64 = fields.next()?.parse().ok()?;
    let used_kb: u64 = fields.next()?.parse().ok()?;
    let _avail = fields.next()?;
    let used_percent: u8 = fields.next()?.trim_end_matches('%').parse().ok()?;
    let mountpoint: String = fields.collect::<Vec<_>>().join(" ");
    if mountpoint.is_empty() {
        return None;
    }
    Some(UsageEntry {
        mountpoint,
        total_bytes: total_kb * 1024,
        used_bytes: used_kb * 1024,
        used_percent,
    })
}

#[tauri::command]
pub fn list_disks() -> Result<Vec<Disk>, String> {
    let output = subprocess::run_with_timeout("lsblk", &["-J", "-o", "NAME,SIZE,TYPE,MOUNTPOINT"], Duration::from_secs(10))?;
    parse_lsblk_json(&output).ok_or_else(|| "impossible de lire la sortie de lsblk".to_string())
}

#[tauri::command]
pub fn list_disk_usage() -> Result<Vec<UsageEntry>, String> {
    let output = subprocess::run_with_timeout("df", &["-k", "-P"], Duration::from_secs(10))?;
    Ok(output.lines().filter_map(parse_df_line).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_lsblk_json_into_disks() {
        let json = r#"{
            "blockdevices": [
                {"name":"sda","size":"512G","type":"disk","mountpoint":null,
                 "children":[{"name":"sda1","size":"511G","type":"part","mountpoint":"/"}]}
            ]
        }"#;
        let disks = parse_lsblk_json(json).expect("should parse");
        assert_eq!(disks.len(), 1);
        assert_eq!(disks[0].name, "sda");
        assert_eq!(disks[0].size, "512G");
        assert_eq!(disks[0].partitions.len(), 1);
        assert_eq!(disks[0].partitions[0].name, "sda1");
        assert_eq!(disks[0].partitions[0].mountpoint.as_deref(), Some("/"));
    }

    #[test]
    fn handles_disk_with_no_partitions() {
        let json = r#"{"blockdevices":[{"name":"sdb","size":"1T","type":"disk","mountpoint":null}]}"#;
        let disks = parse_lsblk_json(json).expect("should parse");
        assert_eq!(disks.len(), 1);
        assert!(disks[0].partitions.is_empty());
    }

    #[test]
    fn rejects_invalid_json() {
        assert!(parse_lsblk_json("not json").is_none());
    }

    #[test]
    fn parses_df_line_into_usage_entry() {
        let line = "/dev/sda1  536870912  214748364  322122548  40%  /";
        let entry = parse_df_line(line).expect("should parse");
        assert_eq!(entry.mountpoint, "/");
        assert_eq!(entry.used_percent, 40);
        assert_eq!(entry.total_bytes, 536870912 * 1024);
        assert_eq!(entry.used_bytes, 214748364 * 1024);
    }

    #[test]
    fn skips_df_header_line() {
        assert!(parse_df_line("Filesystem  1K-blocks  Used  Available  Use%  Mounted on").is_none());
    }

    // Regression guard for the actual bug: reproduced live via `df -k -P` on
    // this project's own dev machine, which included exactly these
    // pseudo-filesystem lines alongside the one real `/dev/sdd` entry.
    // Before this fix, every one of these was surfaced as a real disk usage
    // row -- most misleadingly, `snapfuse` sources are always pinned at
    // "100%" (a read-only squashfs image being exactly as full as itself is
    // normal, not a low-disk-space warning).
    #[test]
    fn rejects_non_dev_pseudo_filesystem_sources() {
        assert!(parse_df_line("none               8130884         0   8130884       0% /usr/lib/modules/6.18.33.2").is_none());
        assert!(parse_df_line("tmpfs              1626176        20   1626156       1% /run/user/1000").is_none());
        assert!(parse_df_line("rootfs             8125128      2772   8122356       1% /init").is_none());
        assert!(parse_df_line("snapfuse             10240     10240         0     100% /snap/nmap/4838").is_none());
        assert!(parse_df_line("drivers          499152892 406887896  92264996      82% /usr/lib/wsl/drivers").is_none());
        assert!(parse_df_line(r"C:\              499152892 406887896  92264996      82% /mnt/c").is_none());
    }

    #[test]
    fn parses_df_line_with_space_in_mountpoint() {
        let line = "/dev/sdb1  1000000  500000  500000  50%  /media/dev/My Passport";
        let entry = parse_df_line(line).expect("should parse mountpoint containing a space");
        assert_eq!(entry.mountpoint, "/media/dev/My Passport");
    }
}
