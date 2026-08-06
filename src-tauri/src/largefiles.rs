use serde::Serialize;
use std::fs;
use std::path::Path;

#[derive(Serialize, Clone)]
pub struct LargeFile {
    pub path: String,
    pub size_bytes: u64,
}

/// Scans `dir` recursively (not following symlinks) for files at or above
/// `min_size_bytes`, sorted largest-first. Unreadable subdirectories are
/// silently skipped, matching `duplicates::find_duplicates`'s behavior.
pub fn find_large_files(dir: &Path, min_size_bytes: u64) -> Result<Vec<LargeFile>, String> {
    if !dir.is_dir() {
        return Err(format!("{} n'est pas un dossier accessible", dir.display()));
    }

    let mut results = Vec::new();
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
                if let Ok(metadata) = entry.metadata() {
                    if metadata.len() >= min_size_bytes {
                        results.push(LargeFile {
                            path: path.to_string_lossy().into_owned(),
                            size_bytes: metadata.len(),
                        });
                    }
                }
            }
        }
    }

    results.sort_by_key(|f| std::cmp::Reverse(f.size_bytes));
    Ok(results)
}

#[tauri::command]
pub fn find_large_files_cmd(directory: String, min_size_bytes: u64) -> Result<Vec<LargeFile>, String> {
    find_large_files(std::path::Path::new(&directory), min_size_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn finds_files_above_the_size_threshold() {
        let dir = std::env::temp_dir().join(format!("nitrux-large-test-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let small = dir.join("small.txt");
        let big = dir.join("big.txt");
        std::fs::File::create(&small).unwrap().write_all(&[0u8; 10]).unwrap();
        std::fs::File::create(&big).unwrap().write_all(&[0u8; 1000]).unwrap();

        let results = find_large_files(&dir, 100).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(results.len(), 1);
        assert!(results[0].path.ends_with("big.txt"));
        assert_eq!(results[0].size_bytes, 1000);
    }

    #[test]
    fn returns_results_sorted_largest_first() {
        let dir = std::env::temp_dir().join(format!("nitrux-large-sort-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::File::create(dir.join("a.bin")).unwrap().write_all(&vec![0u8; 500]).unwrap();
        std::fs::File::create(dir.join("b.bin")).unwrap().write_all(&vec![0u8; 2000]).unwrap();

        let results = find_large_files(&dir, 100).expect("should scan");
        std::fs::remove_dir_all(&dir).ok();

        assert_eq!(results.len(), 2);
        assert!(results[0].size_bytes > results[1].size_bytes);
    }

    #[test]
    fn errors_on_nonexistent_directory() {
        let bogus = std::path::Path::new("/definitely/not/a/real/path/xyz");
        assert!(find_large_files(bogus, 0).is_err());
    }
}
