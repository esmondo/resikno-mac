//! Windows-specific paths and operations (placeholder for future)

use std::path::PathBuf;
use super::PlatformPaths;

/// Windows platform implementation
pub struct WindowsPaths {
    home: PathBuf,
    local_app_data: PathBuf,
}

impl WindowsPaths {
    pub fn new() -> Self {
        let home = directories::UserDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("C:\\Users\\unknown"));

        let local_app_data = std::env::var("LOCALAPPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|_| home.join("AppData\\Local"));

        Self { home, local_app_data }
    }
}

impl Default for WindowsPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformPaths for WindowsPaths {
    fn cache_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.local_app_data.join("Temp"),
            self.local_app_data.clone(),
        ]
    }

    fn app_cache_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.local_app_data.join("Google\\Chrome\\User Data\\Default\\Cache"),
            self.local_app_data.join("Mozilla\\Firefox\\Profiles"),
            self.local_app_data.join("Microsoft\\Edge\\User Data\\Default\\Cache"),
        ]
    }

    fn log_dirs(&self) -> Vec<PathBuf> {
        vec![
            PathBuf::from("C:\\Windows\\Logs"),
            self.local_app_data.join("CrashDumps"),
        ]
    }

    fn temp_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.local_app_data.join("Temp"),
            PathBuf::from("C:\\Windows\\Temp"),
        ]
    }

    fn mobile_backup_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.home.join("Apple\\MobileSync\\Backup"),
            self.local_app_data.join("Apple Computer\\MobileSync\\Backup"),
        ]
    }

    fn xcode_dirs(&self) -> Vec<PathBuf> {
        // Xcode is macOS only
        vec![]
    }

    fn downloads_dir(&self) -> Option<PathBuf> {
        Some(self.home.join("Downloads"))
    }

    fn config_dir(&self) -> PathBuf {
        self.local_app_data.join("resikno-mac")
    }

    fn large_redownload_dirs(&self) -> Vec<PathBuf> {
        // TODO: Add Windows-specific large re-download paths
        vec![]
    }
}
