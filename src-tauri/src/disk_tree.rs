//! Per-directory size breakdown, the "where did my disk space go" view.
//!
//! `DiskVisualizerPage` could show per-mountpoint usage and a flat list of
//! large files, but nothing between the two: a full home directory with no
//! single huge file in it looked empty. This walks one level at a time via
//! `du`, so the user drills down instead of waiting on a full-tree scan.

use crate::subprocess;
use serde::Serialize;
use std::time::Duration;

#[derive(Serialize, Clone, PartialEq, Debug)]
pub struct TreeEntry {
    pub path: String,
    pub size_bytes: u64,
}

/// Parses `du -b --max-depth=1` output: `<bytes>\t<path>` per line. The
/// last line is the directory that was asked about (its own total), which
/// the caller reports separately rather than as one of its children.
pub fn parse_du_output(output: &str, root: &str) -> (Vec<TreeEntry>, u64) {
    let mut children = Vec::new();
    let mut total = 0;
    for line in output.lines() {
        let Some((size, path)) = line.split_once('\t') else { continue };
        let Ok(size_bytes) = size.trim().parse::<u64>() else { continue };
        let path = path.trim_end_matches('\n');
        if path == root {
            total = size_bytes;
            continue;
        }
        children.push(TreeEntry { path: path.to_string(), size_bytes });
    }
    children.sort_by(|a, b| b.size_bytes.cmp(&a.size_bytes));
    (children, total)
}

#[derive(Serialize, Clone)]
pub struct DirectoryBreakdown {
    pub root: String,
    pub total_bytes: u64,
    pub children: Vec<TreeEntry>,
    /// `du` exits non-zero when it could not read *some* subdirectory, yet
    /// still prints every directory it did read. Reporting that as a hard
    /// failure would throw away a correct breakdown over one unreadable
    /// folder, so the sizes come back with the warning attached.
    pub warning: Option<String>,
}

fn validate_directory(path: &str) -> Result<(), String> {
    if !path.starts_with('/') {
        return Err(format!("chemin absolu attendu : {path}"));
    }
    if !std::path::Path::new(path).is_dir() {
        return Err(format!("dossier introuvable : {path}"));
    }
    Ok(())
}

/// Sizes the immediate children of `directory`.
///
/// `-x` keeps the scan on one filesystem: without it, sizing `/` walks into
/// every mounted disk, network share and container overlay, which is both
/// slow and wrong for a "what is filling this disk" answer.
#[tauri::command]
pub fn get_directory_breakdown(directory: String) -> Result<DirectoryBreakdown, String> {
    validate_directory(&directory)?;
    let (stdout, stderr, code) = subprocess::run_capturing_exit_code(
        "du",
        &["-b", "-x", "--max-depth=1", directory.as_str()],
        Duration::from_secs(120),
    )?;

    let (children, total_bytes) = parse_du_output(&stdout, &directory);
    if children.is_empty() && total_bytes == 0 {
        return Err(format!(
            "du n'a rien pu mesurer dans {directory} (code {code}) : {}",
            stderr.trim()
        ));
    }
    let warning = (code != 0).then(|| {
        let detail = stderr.lines().next().unwrap_or("").trim().to_string();
        format!("Certains dossiers n'ont pas pu être lus : {detail}")
    });

    Ok(DirectoryBreakdown { root: directory, total_bytes, children, warning })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_real_du_output_biggest_first() {
        let output = "4096\t/home/dev/.cache\n1073741824\t/home/dev/videos\n52428800\t/home/dev/documents\n1073798720\t/home/dev\n";
        let (children, total) = parse_du_output(output, "/home/dev");
        assert_eq!(total, 1_073_798_720, "the root's own line is the total, not a child");
        assert_eq!(children.len(), 3);
        assert_eq!(children[0].path, "/home/dev/videos", "biggest first");
        assert_eq!(children[2].size_bytes, 4096);
    }

    #[test]
    fn ignores_lines_du_did_not_produce() {
        // Real runs interleave nothing on stdout, but a locale-mangled or
        // partially-written line must be skipped, not parsed into a 0-byte
        // entry that would look like an empty directory.
        let (children, total) = parse_du_output("not a du line\nxyz\t/tmp/a\n100\t/tmp\n", "/tmp");
        assert_eq!(total, 100);
        assert!(children.is_empty());
    }

    #[test]
    fn refuses_a_relative_or_missing_directory() {
        assert!(validate_directory("relative/path").is_err());
        assert!(validate_directory("/definitely/not/here/nitrux").is_err());
    }

    #[test]
    fn sizes_a_real_directory_on_this_machine() {
        let scratch = std::env::temp_dir().join(format!("nitrux-tree-test-{}", std::process::id()));
        let child = scratch.join("sub");
        std::fs::create_dir_all(&child).unwrap();
        std::fs::write(child.join("file.bin"), vec![0u8; 40_000]).unwrap();

        let breakdown = get_directory_breakdown(scratch.to_string_lossy().into_owned())
            .expect("du should measure a directory we just created");
        assert!(breakdown.total_bytes >= 40_000);
        assert_eq!(breakdown.children.len(), 1);
        assert!(breakdown.children[0].path.ends_with("sub"));
        assert!(breakdown.children[0].size_bytes >= 40_000);

        std::fs::remove_dir_all(&scratch).ok();
    }
}
