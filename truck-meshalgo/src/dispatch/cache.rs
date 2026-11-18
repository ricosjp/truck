//! Persistent dot-file cache for calibrated kernel dispatch thresholds and hardware metadata.

use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

use super::metadata::SystemMetadata;
use super::profile::DispatchProfile;

/// Current schema version of the kernel calibration cache.
pub const CURRENT_CACHE_VERSION: u32 = 8;

/// Default filename for the persistent dot file cache.
pub const DEFAULT_CACHE_FILENAME: &str = ".truck_kernel_cache.json";

/// Cached calibration payload serialized to disk.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelCache {
    /// Schema version for cache invalidation when data structures evolve.
    pub cache_version: u32,
    /// Unix timestamp when calibration was performed.
    pub timestamp_secs: u64,
    /// Hardware fingerprint (CPU model & cores, GPU device details).
    pub hardware: SystemMetadata,
    /// Optimal problem-size thresholds and recorded measurements.
    pub profile: DispatchProfile,
}

impl KernelCache {
    /// Creates a new cache record for the current hardware.
    pub fn new(profile: DispatchProfile) -> Self {
        let timestamp_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        Self {
            cache_version: CURRENT_CACHE_VERSION,
            timestamp_secs,
            hardware: SystemMetadata::current(),
            profile,
        }
    }

    /// Validates whether the cache record is still valid for the current runtime.
    pub fn is_valid_for_current_system(&self) -> bool {
        if self.cache_version != CURRENT_CACHE_VERSION {
            return false;
        }
        let current_hw = SystemMetadata::current();
        self.hardware.matches(&current_hw)
    }
}

/// Resolves the filesystem path to the dot-file cache.
pub fn cache_file_path() -> PathBuf {
    if let Ok(custom_path) = std::env::var("TRUCK_KERNEL_CACHE_FILE") {
        return PathBuf::from(custom_path);
    }
    // Prefer standard cache directories: $XDG_CACHE_HOME/truck or $HOME/.cache/truck
    if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
        let dir = PathBuf::from(xdg_cache).join("truck");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("kernel_cache.json");
    }
    if let Ok(home) = std::env::var("HOME") {
        let dir = PathBuf::from(home).join(".cache").join("truck");
        let _ = std::fs::create_dir_all(&dir);
        return dir.join("kernel_cache.json");
    }
    // Default to .truck_kernel_cache.json in the current working directory as fallback
    PathBuf::from(DEFAULT_CACHE_FILENAME)
}

/// Reads the cached dispatch profile if the dot file exists and hardware metadata matches.
pub fn load_cached_profile() -> Option<DispatchProfile> {
    let path = cache_file_path();
    load_cached_profile_from_path(&path)
}

/// Reads the cached dispatch profile from an explicit path.
pub fn load_cached_profile_from_path(path: &Path) -> Option<DispatchProfile> {
    if !path.exists() {
        return None;
    }

    let content = std::fs::read_to_string(path).ok()?;
    let cache: KernelCache = serde_json::from_str(&content).ok()?;

    if cache.is_valid_for_current_system() {
        Some(cache.profile)
    } else {
        None
    }
}

/// Persists the calibrated profile and system hardware metadata to the dot file cache.
pub fn save_cached_profile(profile: &DispatchProfile) -> std::io::Result<()> {
    let path = cache_file_path();
    save_cached_profile_to_path(profile, &path)
}

/// Persists the calibrated profile and hardware metadata to a specific file path.
pub fn save_cached_profile_to_path(profile: &DispatchProfile, path: &Path) -> std::io::Result<()> {
    let cache = KernelCache::new(profile.clone());
    let json_text = serde_json::to_string_pretty(&cache)
        .map_err(|e| std::io::Error::other(e.to_string()))?;
    
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() && !parent.exists() {
            let _ = std::fs::create_dir_all(parent);
        }
    }

    std::fs::write(path, json_text)
}

/// Clears the persistent cache dot file if it exists.
pub fn clear_cache() -> std::io::Result<()> {
    let path = cache_file_path();
    if path.exists() {
        std::fs::remove_file(path)?;
    }
    Ok(())
}
