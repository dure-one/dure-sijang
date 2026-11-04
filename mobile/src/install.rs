//! Update checking for desktop platforms (Windows/Linux/macOS)
//!
//! Provides functionality for:
//! - Checking for updates from GitHub releases
//! - Comparing current version with latest release

use crate::install_stt::{GitHubRelease, UpdateInfo};
use std::env;

const APP_NAME: &str = "dure-sijang";
const GITHUB_REPO: &str = "dure-one/dure-sijang";
const CURRENT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// Check for updates from GitHub releases
pub fn check_for_updates() -> Result<UpdateInfo, String> {
    let api_url = format!("https://api.github.com/repos/{}/releases/latest", GITHUB_REPO);

    log::info!("Checking for updates from: {}", api_url);

    let response = ureq::get(&api_url)
        .set("User-Agent", &format!("{}/{}", APP_NAME, CURRENT_VERSION))
        .call()
        .map_err(|e| format!("Failed to fetch release info: {}", e))?;

    let release: GitHubRelease = serde_json::from_reader(response.into_reader())
        .map_err(|e| format!("Failed to parse release JSON: {}", e))?;

    let latest_version = release.tag_name.trim_start_matches('v');
    let current_version = CURRENT_VERSION;

    let available = latest_version != current_version;

    log::info!("Current version: {}", current_version);
    log::info!("Latest version: {}", latest_version);
    log::info!("Update available: {}", available);

    // Find download URL for current platform
    let download_url = if available {
        find_download_url_for_platform(&release.assets)
            .unwrap_or_else(|| format!("https://github.com/{}/releases/latest", GITHUB_REPO))
    } else {
        String::new()
    };

    Ok(UpdateInfo {
        available,
        current_version: current_version.to_string(),
        latest_version: latest_version.to_string(),
        download_url,
        release_notes: release.body.unwrap_or_default(),
    })
}

/// Find the appropriate download URL for the current platform
fn find_download_url_for_platform(assets: &[crate::install_stt::GitHubAsset]) -> Option<String> {
    #[cfg(target_os = "windows")]
    let platform_keywords = vec!["windows", "win64", "x86_64-pc-windows"];

    #[cfg(target_os = "linux")]
    let platform_keywords = vec!["linux", "x86_64-unknown-linux"];

    #[cfg(target_os = "macos")]
    let platform_keywords = vec!["macos", "darwin", "x86_64-apple-darwin", "aarch64-apple-darwin"];

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    let platform_keywords: Vec<&str> = vec![];

    for asset in assets {
        let name_lower = asset.name.to_lowercase();
        if platform_keywords.iter().any(|k| name_lower.contains(k)) {
            return Some(asset.browser_download_url.clone());
        }
    }

    None
}

/// Get current version
pub fn get_current_version() -> &'static str {
    CURRENT_VERSION
}
