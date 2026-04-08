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

/// Restore a restore point
/// 
/// Since files are moved to Trash (not backed up), this attempts to restore from Trash.
/// Returns the number of items successfully restored.
pub fn restore_restore_point(restore_point_id: &str) -> Result<(usize, usize)> {
    let restore_path = restore_dir()?.join(restore_point_id);
    let manifest_path = restore_path.join("manifest.json");
    
    if !manifest_path.exists() {
        anyhow::bail!("Restore point '{}' not found", restore_point_id);
    }
    
    // Read manifest
    let manifest_json = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_json)?;
    
    let mut restored = 0;
    let mut failed = 0;
    
    for entry in &manifest.items {
        // Check if file still exists (wasn't actually deleted)
        if entry.path.exists() {
            restored += 1; // Already there
            continue;
        }
        
        // Try to restore from Trash
        match restore_from_trash(&entry.path) {
            Ok(true) => restored += 1,
            Ok(false) => failed += 1,
            Err(_) => failed += 1,
        }
    }
    
    Ok((restored, failed))
}

/// Attempt to restore a file from Trash
/// 
/// On macOS, Trash is at ~/.Trash/ but Finder manages it.
/// This function attempts to find the file in Trash by name and restore it.
#[cfg(target_os = "macos")]
fn restore_from_trash(original_path: &PathBuf) -> Result<bool> {
    use std::process::Command;
    
    let file_name = original_path.file_name()
        .ok_or_else(|| anyhow::anyhow!("Invalid path"))?
        .to_string_lossy();
    
    // Try to find the file in Trash
    let trash_path = directories::UserDirs::new()
        .map(|d| d.home_dir().join(".Trash").join(file_name.as_ref()));
    
    if let Some(trash_file) = trash_path {
        if trash_file.exists() {
            // Ensure parent directory exists
            if let Some(parent) = original_path.parent() {
                fs::create_dir_all(parent)?;
            }
            
            // Move from Trash back to original location
            fs::rename(&trash_file, original_path)?;
            return Ok(true);
        }
    }
    
    // Try using AppleScript to restore from Trash via Finder
    // This handles cases where the file might have been renamed in Trash
    let script = format!(
        r#"tell application "Finder"
            set trashItems to every item of trash
            repeat with i in trashItems
                if name of i is "{}" then
                    move i to folder "{}" of (path to home folder)
                    return "restored"
                end if
            end repeat
            return "not_found"
        end tell"#,
        file_name,
        original_path.parent()
            .and_then(|p| p.strip_prefix(directories::UserDirs::new()?.home_dir()).ok())
            .map(|p| p.to_string_lossy().replace('/', ":"))
            .unwrap_or_default()
    );
    
    let output = Command::new("osascript")
        .arg("-e")
        .arg(&script)
        .output();
    
    match output {
        Ok(out) => {
            let result = String::from_utf8_lossy(&out.stdout);
            Ok(result.trim() == "restored")
        }
        Err(_) => Ok(false),
    }
}

#[cfg(not(target_os = "macos"))]
fn restore_from_trash(_original_path: &PathBuf) -> Result<bool> {
    // Non-macOS: restoration not implemented
    Ok(false)
}

/// Get detailed information about a restore point
pub fn get_restore_point_details(restore_point_id: &str) -> Result<Manifest> {
    let restore_path = restore_dir()?.join(restore_point_id);
    let manifest_path = restore_path.join("manifest.json");
    
    if !manifest_path.exists() {
        anyhow::bail!("Restore point '{}' not found", restore_point_id);
    }
    
    let manifest_json = fs::read_to_string(&manifest_path)?;
    let manifest: Manifest = serde_json::from_str(&manifest_json)?;
    
    Ok(manifest)
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
