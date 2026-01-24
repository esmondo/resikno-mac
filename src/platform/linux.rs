//! Linux-specific paths and operations (placeholder for future)

use std::path::PathBuf;
use super::PlatformPaths;

/// Linux platform implementation
pub struct LinuxPaths {
    home: PathBuf,
}

impl LinuxPaths {
    pub fn new() -> Self {
        let home = directories::UserDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/home/unknown"));

        Self { home }
    }
}

impl Default for LinuxPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformPaths for LinuxPaths {
    fn cache_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.home.join(".cache"),
            PathBuf::from("/var/cache"),
        ]
    }

    fn app_cache_dirs(&self) -> Vec<PathBuf> {
        let cache = self.home.join(".cache");
        vec![
            cache.join("google-chrome"),
            cache.join("mozilla"),
            cache.join("chromium"),
        ]
    }

    fn log_dirs(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from("/var/log"),
            self.home.join(".local/share"),
        ]
    }

    fn temp_dirs(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from("/tmp"),
            PathBuf::from("/var/tmp"),
        ]
    }

    fn mobile_backup_dirs(&self) -> Vec<PathBuf> {
        // Linux typically doesn't have iOS backups
        vec![]
    }

    fn xcode_dirs(&self) -> Vec<PathBuf> {
        // Xcode is macOS only
        vec![]
    }

    fn downloads_dir(&self) -> Option<PathBuf> {
        Some(self.home.join("Downloads"))
    }

    fn config_dir(&self) -> PathBuf {
        self.home.join(".config/resikno-mak")
    }
}
