//! Cache directory detection and scanning

use std::path::PathBuf;
use anyhow::Result;
use crate::platform::PlatformPaths;

/// Scan cache directories and return their sizes
pub fn scan_caches<P: PlatformPaths>(platform: &P) -> Result<Vec<(PathBuf, u64)>> {
    let cache_dirs = platform.cache_dirs();
    let mut results = Vec::new();

    for dir in cache_dirs {
        if dir.exists() {
            let size = calculate_dir_size(&dir)?;
            results.push((dir, size));
        }
    }

    Ok(results)
}

/// Calculate total physical disk usage of a directory recursively.
/// See `scanner::file_disk_usage` for why we use block count, not length.
fn calculate_dir_size(path: &PathBuf) -> Result<u64> {
    use crate::scanner::file_disk_usage;
    let mut total = 0u64;

    if path.is_file() {
        return Ok(file_disk_usage(&path.metadata()?));
    }

    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| file_disk_usage(&m)).unwrap_or(0);
        }
    }

    Ok(total)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_dir_size() {
        // TODO: Add tests with tempfile
    }
}
