//! Large and old file detection

use std::path::PathBuf;
use std::time::{Duration, SystemTime};
use anyhow::Result;
use chrono::{DateTime, Utc};

/// Find files within the specified size range
///
/// # Arguments
/// * `paths` - Directories to search
/// * `min_size_bytes` - Minimum file size (0 = no minimum)
/// * `max_size_bytes` - Maximum file size (0 = no maximum)
pub fn find_large_files(paths: &[PathBuf], min_size_bytes: u64, max_size_bytes: u64) -> Result<Vec<LargeFile>> {
    let mut results = Vec::new();

    for path in paths {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    let size = metadata.len();
                    let meets_min = min_size_bytes == 0 || size >= min_size_bytes;
                    let meets_max = max_size_bytes == 0 || size <= max_size_bytes;

                    if meets_min && meets_max {
                        let last_accessed = metadata
                            .accessed()
                            .ok()
                            .map(|t| DateTime::<Utc>::from(t));
                        let last_modified = metadata
                            .modified()
                            .ok()
                            .map(|t| DateTime::<Utc>::from(t));

                        results.push(LargeFile {
                            path: entry.path().to_path_buf(),
                            size,
                            last_accessed,
                            last_modified,
                        });
                    }
                }
            }
        }
    }

    // Sort by size descending
    results.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(results)
}

/// Find files not accessed in the specified number of days
pub fn find_old_files(paths: &[PathBuf], days_old: u32) -> Result<Vec<LargeFile>> {
    let cutoff = SystemTime::now() - Duration::from_secs(days_old as u64 * 24 * 60 * 60);
    let mut results = Vec::new();

    for path in paths {
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    if let Ok(accessed) = metadata.accessed() {
                        if accessed < cutoff {
                            let last_accessed = Some(DateTime::<Utc>::from(accessed));
                            let last_modified = metadata
                                .modified()
                                .ok()
                                .map(|t| DateTime::<Utc>::from(t));

                            results.push(LargeFile {
                                path: entry.path().to_path_buf(),
                                size: metadata.len(),
                                last_accessed,
                                last_modified,
                            });
                        }
                    }
                }
            }
        }
    }

    // Sort by size descending
    results.sort_by(|a, b| b.size.cmp(&a.size));

    Ok(results)
}

/// A large or old file
#[derive(Debug, Clone)]
pub struct LargeFile {
    pub path: PathBuf,
    pub size: u64,
    pub last_accessed: Option<DateTime<Utc>>,
    pub last_modified: Option<DateTime<Utc>>,
}

impl LargeFile {
    pub fn human_size(&self) -> String {
        bytesize::ByteSize(self.size).to_string()
    }

    pub fn days_since_access(&self) -> Option<i64> {
        self.last_accessed.map(|t| {
            (Utc::now() - t).num_days()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_find_large_files() {
        // TODO: Add tests with tempfile
    }
}
