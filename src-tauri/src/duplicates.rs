use serde::Serialize;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct DuplicateGroup {
    pub hash: String,
    pub paths: Vec<String>,
    pub size_bytes: u64,
}

fn hash_file(path: &Path) -> std::io::Result<String> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Scans `dir` recursively (not following symlinks, to avoid cycles) and
/// groups regular files with identical SHA-256 content hashes. Directories
/// that can't be read (permission denied, etc.) are silently skipped rather
/// than aborting the whole scan.
///
/// Two passes: first group by file size (cheap, no file reads) and drop any
/// size bucket with a single file -- a file can never share a hash with a
/// file of a different size, so it can never have a duplicate and hashing
/// it (reading its full content) would be wasted work. Only files that
/// share a size with at least one other file get actually hashed. On a
/// directory of mostly-unique large files (the common case: photos,
/// videos), this avoids reading their content at all.
pub fn find_duplicates(dir: &Path) -> Result<Vec<DuplicateGroup>, String> {
    if !dir.is_dir() {
        return Err(format!("{} n'est pas un dossier accessible", dir.display()));
    }

    let mut by_size: HashMap<u64, Vec<std::path::PathBuf>> = HashMap::new();
    let mut stack = vec![dir.to_path_buf()];

    while let Some(current) = stack.pop() {
        let entries = match fs::read_dir(&current) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let file_type = match entry.file_type() {
                Ok(t) => t,
                Err(_) => continue,
            };
            if file_type.is_symlink() {
                continue;
            }
            if file_type.is_dir() {
                stack.push(path);
            } else if file_type.is_file() {
                let size = entry.metadata().map(|m| m.len()).unwrap_or(0);
                by_size.entry(size).or_default().push(path);
            }
        }
    }

    let mut by_hash: HashMap<String, (Vec<String>, u64)> = HashMap::new();
    for (size, paths) in by_size.into_iter().filter(|(_, paths)| paths.len() > 1) {
        for path in paths {
            if let Ok(hash) = hash_file(&path) {
                let entry = by_hash.entry(hash).or_insert_with(|| (Vec::new(), size));
                entry.0.push(path.to_string_lossy().into_owned());
            }
        }
    }

    Ok(by_hash
        .into_iter()
        .filter(|(_, (paths, _))| paths.len() > 1)
        .map(|(hash, (paths, size_bytes))| DuplicateGroup { hash, paths, size_bytes })
        .collect())
}

#[tauri::command]
pub fn find_duplicate_files(directory: String) -> Result<Vec<DuplicateGroup>, String> {
    find_duplicates(std::path::Path::new(&directory))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn groups_files_with_identical_content_by_hash() {
        let dir = std::env::temp_dir().join(format!("nitrux-dup-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let a = dir.join("a.txt");
        let b = dir.join("b.txt");
        let c = dir.join("c.txt");
        std::fs::File::create(&a).unwrap().write_all(b"same content").unwrap();
        std::fs::File::create(&b).unwrap().write_all(b"same content").unwrap();
        std::fs::File::create(&c).unwrap().write_all(b"different content").unwrap();

        let groups = find_duplicates(&dir).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(groups.len(), 1, "expected exactly one duplicate group");
        assert_eq!(groups[0].paths.len(), 2);
    }

    #[test]
    fn returns_no_groups_when_all_files_are_unique() {
        let dir = std::env::temp_dir().join(format!("nitrux-dup-unique-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(dir.join("x.txt")).unwrap().write_all(b"x").unwrap();
        std::fs::File::create(dir.join("y.txt")).unwrap().write_all(b"y").unwrap();

        let groups = find_duplicates(&dir).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert!(groups.is_empty());
    }

    #[test]
    fn size_bucketing_does_not_cross_contaminate_groups() {
        // Two files sharing a size (5 bytes) but different content must NOT
        // be grouped with each other, and a same-size-different-content file
        // in one bucket must not leak into a duplicate group from another
        // bucket -- exercises the two-pass group-by-size-then-hash path
        // across multiple concurrent size buckets, not just the single-file
        // (never hashed) and single-duplicate-pair cases the other tests
        // already cover.
        let dir = std::env::temp_dir().join(format!("nitrux-dup-buckets-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(dir.join("a1.txt")).unwrap().write_all(b"aaaaa").unwrap();
        std::fs::File::create(dir.join("a2.txt")).unwrap().write_all(b"aaaaa").unwrap();
        std::fs::File::create(dir.join("a3.txt")).unwrap().write_all(b"bbbbb").unwrap();
        std::fs::File::create(dir.join("c1.txt")).unwrap().write_all(b"cc").unwrap();
        std::fs::File::create(dir.join("c2.txt")).unwrap().write_all(b"cc").unwrap();

        let mut groups = find_duplicates(&dir).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();
        groups.sort_by_key(|g| g.size_bytes);

        assert_eq!(groups.len(), 2, "expected exactly two duplicate groups");
        assert_eq!(groups[0].size_bytes, 2);
        assert_eq!(groups[0].paths.len(), 2);
        assert_eq!(groups[1].size_bytes, 5);
        assert_eq!(groups[1].paths.len(), 2);
    }

    #[test]
    fn errors_on_nonexistent_directory() {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/xyz");
        assert!(find_duplicates(bogus).is_err());
    }
}
