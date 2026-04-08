//! macOS-specific paths and operations

use std::path::PathBuf;
use super::PlatformPaths;

/// macOS platform implementation
pub struct MacOSPaths {
    home: PathBuf,
}

impl MacOSPaths {
    pub fn new() -> Self {
        let home = directories::UserDirs::new()
            .map(|d| d.home_dir().to_path_buf())
            .unwrap_or_else(|| PathBuf::from("/Users/unknown"));

        Self { home }
    }
}

impl Default for MacOSPaths {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformPaths for MacOSPaths {
    fn cache_dirs(&self) -> Vec<PathBuf> {
        vec![
            // User caches only (system caches need root)
            self.home.join("Library/Caches"),
        ]
    }

    fn app_cache_dirs(&self) -> Vec<PathBuf> {
        let user_caches = self.home.join("Library/Caches");

        // Common app caches
        vec![
            // Browsers
            user_caches.join("com.apple.Safari"),
            user_caches.join("Google/Chrome"),
            user_caches.join("Firefox"),
            user_caches.join("com.microsoft.Edge"),
            // Communication
            user_caches.join("com.tinyspeck.slackmacgap"),
            user_caches.join("com.hnc.Discord"),
            // Development
            user_caches.join("com.microsoft.VSCode"),
            user_caches.join("JetBrains"),
            // Media
            user_caches.join("com.spotify.client"),
            user_caches.join("com.apple.Music"),
            // Dev tool caches (from decisions.md TD-003) - SAFE to delete
            self.home.join(".npm/_cacache"),           // npm cache (often 5GB+)
            self.home.join(".cargo/registry/cache"),   // Cargo crate cache
            self.home.join(".cargo/git/checkouts"),    // Cargo git cache
            self.home.join(".pnpm-store"),             // pnpm global store
            self.home.join(".bun/install/cache"),      // Bun cache
            user_caches.join("electron"),              // Electron framework cache
            user_caches.join("electron-builder"),      // electron-builder cache
            user_caches.join("ms-playwright-go"),      // Playwright browser binaries
        ]
    }

    fn log_dirs(&self) -> Vec<PathBuf> {
        vec![
            // User logs only (system logs need root)
            self.home.join("Library/Logs"),
        ]
    }

    fn temp_dirs(&self) -> Vec<PathBuf> {
        // User temp folders on macOS
        // Use TMPDIR env var which points to /var/folders/xx/xxxxxx/T/
        let mut temps = Vec::new();
        
        // Primary user temp directory from environment
        if let Ok(tmpdir) = std::env::var("TMPDIR") {
            temps.push(PathBuf::from(tmpdir));
        }
        
        // Fallback: macOS-specific temp locations
        temps.push(self.home.join("Library/Containers"));
        
        temps
    }

    fn mobile_backup_dirs(&self) -> Vec<PathBuf> {
        vec![
            self.home.join("Library/Application Support/MobileSync/Backup"),
        ]
    }

    fn xcode_dirs(&self) -> Vec<PathBuf> {
        vec![
            // Derived data (build artifacts)
            self.home.join("Library/Developer/Xcode/DerivedData"),
            // Archives
            self.home.join("Library/Developer/Xcode/Archives"),
            // iOS device support
            self.home.join("Library/Developer/Xcode/iOS DeviceSupport"),
            self.home.join("Library/Developer/Xcode/watchOS DeviceSupport"),
            self.home.join("Library/Developer/Xcode/tvOS DeviceSupport"),
            // Simulators
            self.home.join("Library/Developer/CoreSimulator/Devices"),
        ]
    }

    fn downloads_dir(&self) -> Option<PathBuf> {
        Some(self.home.join("Downloads"))
    }

    fn config_dir(&self) -> PathBuf {
        self.home.join(".resikno-mac")
    }

    fn large_redownload_dirs(&self) -> Vec<PathBuf> {
        let user_caches = self.home.join("Library/Caches");
        vec![
            // From decisions.md - safe but large re-download
            user_caches.join("SiriTTS/BNNSModels"),  // 968 MB ML models
            user_caches.join("Comet"),               // Linear app, 940 MB, will re-sync
        ]
    }
}

/// Get list of commonly safe-to-delete cache identifiers
pub fn safe_cache_identifiers() -> Vec<&'static str> {
    vec![
        // Apple
        "com.apple.Safari",
        "com.apple.dt.Xcode",
        "com.apple.Music",
        "com.apple.podcasts",
        // Browsers
        "com.google.Chrome",
        "org.mozilla.firefox",
        "com.microsoft.Edge",
        // Dev tools
        "com.microsoft.VSCode",
        "com.sublimetext",
        "com.jetbrains",
        // Communication
        "com.tinyspeck.slackmacgap",
        "com.hnc.Discord",
        "us.zoom.xos",
        // Media
        "com.spotify.client",
        "tv.plex.plexamp",
    ]
}

/// Get list of browser-specific cache locations
pub fn browser_caches(home: &PathBuf) -> Vec<(String, PathBuf)> {
    vec![
        ("Safari".to_string(), home.join("Library/Caches/com.apple.Safari")),
        ("Chrome".to_string(), home.join("Library/Caches/Google/Chrome")),
        ("Chrome (Profile)".to_string(), home.join("Library/Application Support/Google/Chrome/Default/Cache")),
        ("Firefox".to_string(), home.join("Library/Caches/Firefox")),
        ("Edge".to_string(), home.join("Library/Caches/com.microsoft.Edge")),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_macos_paths() {
        let paths = MacOSPaths::new();

        // Should have cache dirs
        assert!(!paths.cache_dirs().is_empty());

        // Should have log dirs
        assert!(!paths.log_dirs().is_empty());

        // Config dir should be in home
        let config = paths.config_dir();
        assert!(config.to_string_lossy().contains(".resikno-mac"));
    }
}
