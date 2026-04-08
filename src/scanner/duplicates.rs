//! Duplicate file detection using content hashing

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::PathBuf;
use anyhow::Result;
use sha2::{Sha256, Digest};

/// Find duplicate files in the given directories
pub fn find_duplicates(paths: &[PathBuf], min_size: u64) -> Result<Vec<DuplicateGroup>> {
    // Step 1: Group files by size (quick filter)
    let size_groups = group_by_size(paths, min_size)?;

    // Step 2: For files with same size, compare hashes
    let mut duplicates = Vec::new();

    for (size, files) in size_groups {
        if files.len() < 2 {
            continue;
        }

        let hash_groups = group_by_hash(&files)?;

        for (hash, group) in hash_groups {
            if group.len() >= 2 {
                duplicates.push(DuplicateGroup {
                    hash,
                    size,
                    files: group,
                });
            }
        }
    }

    Ok(duplicates)
}

/// A group of duplicate files
#[derive(Debug, Clone)]
pub struct DuplicateGroup {
    pub hash: String,
    pub size: u64,
    pub files: Vec<PathBuf>,
}

impl DuplicateGroup {
    /// Space that could be recovered by keeping only one copy
    pub fn recoverable_space(&self) -> u64 {
        self.size * (self.files.len() as u64 - 1)
    }
}

/// Group files by size
fn group_by_size(paths: &[PathBuf], min_size: u64) -> Result<HashMap<u64, Vec<PathBuf>>> {
    let mut groups: HashMap<u64, Vec<PathBuf>> = HashMap::new();

    for path in paths {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    let size = metadata.len();
                    if size >= min_size {
                        groups
                            .entry(size)
                            .or_default()
                            .push(entry.path().to_path_buf());
                    }
                }
            }
        }
    }

    Ok(groups)
}

/// Group files by their content hash
fn group_by_hash(files: &[PathBuf]) -> Result<HashMap<String, Vec<PathBuf>>> {
    let mut groups: HashMap<String, Vec<PathBuf>> = HashMap::new();

    for file in files {
        if let Ok(hash) = hash_file(file) {
            groups.entry(hash).or_default().push(file.clone());
        }
    }

    Ok(groups)
}

/// Calculate SHA256 hash of a file
fn hash_file(path: &PathBuf) -> Result<String> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(format!("{:x}", hasher.finalize()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn test_hash_file_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("test.txt");
        
        // Create a test file
        let mut file = std::fs::File::create(&file_path).unwrap();
        file.write_all(b"hello world").unwrap();
        drop(file);
        
        // Hash it
        let hash1 = hash_file(&file_path).unwrap();
        let hash2 = hash_file(&file_path).unwrap();
        
        // Same file should have same hash
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64); // SHA256 is 64 hex chars
    }

    #[test]
    fn test_hash_file_different_content() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file1 = temp_dir.path().join("test1.txt");
        let file2 = temp_dir.path().join("test2.txt");
        
        // Create two files with different content
        std::fs::write(&file1, "content A").unwrap();
        std::fs::write(&file2, "content B").unwrap();
        
        // Different content should have different hashes
        let hash1 = hash_file(&file1).unwrap();
        let hash2 = hash_file(&file2).unwrap();
        
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_duplicate_group_recoverable_space() {
        let group = DuplicateGroup {
            hash: "abc123".to_string(),
            size: 1024,
            files: vec![
                PathBuf::from("/file1.txt"),
                PathBuf::from("/file2.txt"),
                PathBuf::from("/file3.txt"),
            ],
        };
        
        // Recoverable space = size * (n - 1)
        assert_eq!(group.recoverable_space(), 1024 * 2);
    }

    #[test]
    fn test_group_by_size_basic() {
        let temp_dir = tempfile::tempdir().unwrap();
        
        // Create files with different sizes
        let file1 = temp_dir.path().join("file1.txt");
        let file2 = temp_dir.path().join("file2.txt");
        let file3 = temp_dir.path().join("file3.txt");
        
        std::fs::write(&file1, "a").unwrap(); // 1 byte
        std::fs::write(&file2, "a").unwrap(); // 1 byte
        std::fs::write(&file3, "bb").unwrap(); // 2 bytes
        
        let groups = group_by_size(&[temp_dir.path().to_path_buf()], 1).unwrap();
        
        // Should have 2 groups: size 1 and size 2
        assert_eq!(groups.len(), 2);
        assert_eq!(groups[&1].len(), 2); // Two 1-byte files
        assert_eq!(groups[&2].len(), 1); // One 2-byte file
    }
}
