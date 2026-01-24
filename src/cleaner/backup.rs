//! Restore point creation and management

use std::fs;
use std::path::PathBuf;
use anyhow::Result;
use chrono::Utc;
use serde::{Deserialize, Serialize};

use super::RestorePoint;

/// Manifest entry for a deleted file
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestEntry {
    pub path: PathBuf,
    pub size: u64,
    pub category: String,
    pub backed_up: bool,
    pub reason: String,
}

/// Full manifest for a restore point
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Manifest {
    pub timestamp: chrono::DateTime<Utc>,
    pub total_size: u64,
    pub items: Vec<ManifestEntry>,
}

/// Get the restore directory path
pub fn restore_dir() -> Result<PathBuf> {
    let home = directories::UserDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?
        .home_dir()
        .to_path_buf();

    Ok(home.join(".resikno-mak").join("restore"))
}

/// Create a restore point before deletion
pub fn create_restore_point(paths: &[PathBuf]) -> Result<RestorePoint> {
    let timestamp = Utc::now();
    let id = timestamp.format("%Y-%m-%d_%H%M%S").to_string();

    let restore_path = restore_dir()?.join(&id);
    fs::create_dir_all(&restore_path)?;

    // Build manifest
    let mut manifest = Manifest {
        timestamp,
        total_size: 0,
        items: Vec::new(),
    };

    for path in paths {
        if let Ok(metadata) = path.metadata() {
            let size = if metadata.is_file() {
                metadata.len()
            } else {
                calculate_dir_size(path)?
            };

            manifest.total_size += size;
            manifest.items.push(ManifestEntry {
                path: path.clone(),
                size,
                category: categorize_path(path),
                backed_up: false, // Caches don't need backup
                reason: get_deletion_reason(path),
            });
        }
    }

    // Write manifest
    let manifest_path = restore_path.join("manifest.json");
    let manifest_json = serde_json::to_string_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_json)?;

    Ok(RestorePoint {
        id,
        timestamp,
        manifest_path,
        total_size: manifest.total_size,
        item_count: manifest.items.len(),
    })
}

/// List available restore points
pub fn list_restore_points() -> Result<Vec<RestorePoint>> {
    let restore_path = restore_dir()?;
    let mut points = Vec::new();

    if !restore_path.exists() {
        return Ok(points);
    }

    for entry in fs::read_dir(restore_path)? {
        let entry = entry?;
        if entry.file_type()?.is_dir() {
            let manifest_path = entry.path().join("manifest.json");
            if manifest_path.exists() {
                let manifest_json = fs::read_to_string(&manifest_path)?;
                let manifest: Manifest = serde_json::from_str(&manifest_json)?;

                points.push(RestorePoint {
                    id: entry.file_name().to_string_lossy().to_string(),
                    timestamp: manifest.timestamp,
                    manifest_path,
                    total_size: manifest.total_size,
                    item_count: manifest.items.len(),
                });
            }
        }
    }

    // Sort by timestamp descending (newest first)
    points.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

    Ok(points)
}

/// Calculate total size of a directory
fn calculate_dir_size(path: &PathBuf) -> Result<u64> {
    let mut total = 0;
    for entry in walkdir::WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.file_type().is_file() {
            total += entry.metadata().map(|m| m.len()).unwrap_or(0);
        }
    }
    Ok(total)
}

/// Categorize a path for the manifest
fn categorize_path(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("cache") {
        "cache".to_string()
    } else if path_str.contains("log") {
        "log".to_string()
    } else if path_str.contains("tmp") || path_str.contains("temp") {
        "temp".to_string()
    } else {
        "other".to_string()
    }
}

/// Get human-readable reason for deletion
fn get_deletion_reason(path: &PathBuf) -> String {
    let path_str = path.to_string_lossy().to_lowercase();

    if path_str.contains("cache") {
        "Cache files - regenerated automatically by applications".to_string()
    } else if path_str.contains("log") {
        "Log files - historical data, usually not needed".to_string()
    } else if path_str.contains("tmp") || path_str.contains("temp") {
        "Temporary files - safe to remove".to_string()
    } else {
        "User selected for cleanup".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_categorize_path() {
        let cache_path = PathBuf::from("/Users/test/Library/Caches/com.app");
        assert_eq!(categorize_path(&cache_path), "cache");

        let log_path = PathBuf::from("/var/log/system.log");
        assert_eq!(categorize_path(&log_path), "log");
    }
}
