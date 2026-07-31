use serde::{Deserialize, Serialize};
use sha1::Sha1;
use sha2::{Digest, Sha256};
use std::fmt::Write as _;
use std::fs;
use std::io::Read;
use std::path::Path;

#[derive(Serialize, Deserialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
pub enum HashAlgorithm {
    Md5,
    Sha1,
    Sha256,
}

fn hash_with<D: Digest>(mut hasher: D, path: &Path) -> Result<String, String> {
    let mut file = fs::File::open(path).map_err(|e| format!("impossible d'ouvrir {} : {e}", path.display()))?;
    let mut buf = [0u8; 65536];
    loop {
        let n = file
            .read(&mut buf)
            .map_err(|e| format!("erreur de lecture de {} : {e}", path.display()))?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    // `format!("{:x}", ...)` doesn't work generically here: `LowerHex` for
    // `GenericArray<u8, D::OutputSize>` isn't implemented for every output
    // size when `D` is a type parameter, so hex-encode the digest bytes by
    // hand instead.
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut hex, "{byte:02x}").expect("writing to a String cannot fail");
    }
    Ok(hex)
}

pub fn compute_hash(path: &Path, algorithm: HashAlgorithm) -> Result<String, String> {
    match algorithm {
        HashAlgorithm::Md5 => hash_with(md5::Md5::new(), path),
        HashAlgorithm::Sha1 => hash_with(Sha1::new(), path),
        HashAlgorithm::Sha256 => hash_with(Sha256::new(), path),
    }
}

pub fn verify_hash(path: &Path, algorithm: HashAlgorithm, expected: &str) -> Result<bool, String> {
    let actual = compute_hash(path, algorithm)?;
    Ok(actual.eq_ignore_ascii_case(expected.trim()))
}

#[tauri::command]
pub fn compute_file_hash(path: String, algorithm: HashAlgorithm) -> Result<String, String> {
    compute_hash(std::path::Path::new(&path), algorithm)
}

#[tauri::command]
pub fn verify_file_hash(path: String, algorithm: HashAlgorithm, expected: String) -> Result<bool, String> {
    verify_hash(std::path::Path::new(&path), algorithm, &expected)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn computes_sha256_of_a_known_file() {
        let path = std::env::temp_dir().join(format!("nitrux-hash-test-{}.txt", std::process::id()));
        std::fs::File::create(&path).unwrap().write_all(b"hello world").unwrap();

        let result = compute_hash(&path, HashAlgorithm::Sha256).expect("should hash");
        std::fs::remove_file(&path).ok();

        // Known SHA-256 of "hello world" (verified independently via
        // sha256sum, openssl dgst, and Python hashlib — the plan's quoted
        // vector was missing its trailing "9" character).
        assert_eq!(result, "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9");
    }

    #[test]
    fn errors_on_nonexistent_file() {
        let bogus = std::path::Path::new("/definitely/not/a/real/file/xyz.txt");
        assert!(compute_hash(bogus, HashAlgorithm::Sha256).is_err());
    }

    #[test]
    fn matches_expected_hash_case_insensitively() {
        let path = std::env::temp_dir().join(format!("nitrux-hash-match-{}.txt", std::process::id()));
        std::fs::File::create(&path).unwrap().write_all(b"hello world").unwrap();

        let matches = verify_hash(
            &path,
            HashAlgorithm::Sha256,
            "B94D27B9934D3E08A52E52D7DA7DABFAC484EFE37A5380EE9088F7ACE2EFCDE9",
        )
        .expect("should verify");
        std::fs::remove_file(&path).ok();

        assert!(matches);
    }
}
