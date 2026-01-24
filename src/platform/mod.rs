//! Platform-specific code and abstractions

#[cfg(target_os = "macos")]
pub mod macos;

#[cfg(target_os = "linux")]
pub mod linux;

#[cfg(target_os = "windows")]
pub mod windows;

use std::path::PathBuf;

/// Trait for platform-specific path detection
pub trait PlatformPaths {
    /// Get system cache directories
    fn cache_dirs(&self) -> Vec<PathBuf>;

    /// Get application cache directories
    fn app_cache_dirs(&self) -> Vec<PathBuf>;

    /// Get log directories
    fn log_dirs(&self) -> Vec<PathBuf>;

    /// Get temporary file directories
    fn temp_dirs(&self) -> Vec<PathBuf>;

    /// Get iOS/mobile backup directories
    fn mobile_backup_dirs(&self) -> Vec<PathBuf>;

    /// Get Xcode-related directories (macOS only)
    fn xcode_dirs(&self) -> Vec<PathBuf>;

    /// Get downloads directory
    fn downloads_dir(&self) -> Option<PathBuf>;

    /// Get the resikno-mak config directory
    fn config_dir(&self) -> PathBuf;
}

/// Get the current platform implementation
#[cfg(target_os = "macos")]
pub fn current() -> impl PlatformPaths {
    macos::MacOSPaths::new()
}

#[cfg(target_os = "linux")]
pub fn current() -> impl PlatformPaths {
    linux::LinuxPaths::new()
}

#[cfg(target_os = "windows")]
pub fn current() -> impl PlatformPaths {
    windows::WindowsPaths::new()
}

/// Get home directory
pub fn home_dir() -> Option<PathBuf> {
    directories::UserDirs::new().map(|d| d.home_dir().to_path_buf())
}
