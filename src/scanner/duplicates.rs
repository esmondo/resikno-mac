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

    #[test]
    fn test_hash_file() {
        // TODO: Add tests with tempfile
    }
}
