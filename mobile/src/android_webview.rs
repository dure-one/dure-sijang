/// Android WebView integration using wry
///
/// This module provides WebView functionality on Android using the wry crate.
/// WebViews are created in Rust and attached to WebViewActivity via JNI for
/// Activity Embedding split-screen support.

#[cfg(target_os = "android")]
use wry::{WebViewBuilder, WebView};
#[cfg(target_os = "android")]
use raw_window_handle::HasWindowHandle;
use std::collections::HashMap;
#[cfg(target_os = "android")]
use std::time::SystemTime;

/// Manages WebView instances on Android with LRU caching
#[cfg(target_os = "android")]
pub struct AndroidWebViewManager {
    /// Map of tab IDs to WebView instances
    webviews: HashMap<i32, WebView>,
    /// Last access time for LRU eviction
    last_access: HashMap<i32, SystemTime>,
    /// Active webview ID
    active_webview: Option<i32>,
    /// Maximum cached WebViews
    max_cache_size: usize,
}

#[cfg(target_os = "android")]
impl AndroidWebViewManager {
    const MAX_CACHED_WEBVIEWS: usize = 10;

    /// Create a new WebView manager
    pub fn new() -> Self {
        Self {
            webviews: HashMap::new(),
            last_access: HashMap::new(),
            active_webview: None,
            max_cache_size: Self::MAX_CACHED_WEBVIEWS,
        }
    }

    /// Get or create a WebView for a tab (with LRU caching)
    pub fn get_or_create_webview(
        &mut self,
        tab_id: i32,
        url: &str,
        window: &impl HasWindowHandle,
    ) -> anyhow::Result<&WebView> {
        log::info!("get_or_create_webview: tab_id={}, url={}", tab_id, url);

        // Return cached WebView if exists
        if self.webviews.contains_key(&tab_id) {
            self.last_access.insert(tab_id, SystemTime::now());
            log::info!("Returning cached wry WebView for tab {}", tab_id);
            return Ok(self.webviews.get(&tab_id).unwrap());
        }

        // Check cache size limit and evict LRU if needed
        if self.webviews.len() >= self.max_cache_size {
            self.evict_lru();
        }

        // Create WebView using wry
        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_user_agent("Mozilla/5.0 (Linux; Android) DureSijang/1.0")
            .with_accept_first_mouse(true)
            .build_as_child(window)?;

        log::info!("Created new wry WebView for tab {} (cache size: {})",
                   tab_id, self.webviews.len() + 1);

        // Cache the WebView
        self.webviews.insert(tab_id, webview);
        self.last_access.insert(tab_id, SystemTime::now());
        self.active_webview = Some(tab_id);

        Ok(self.webviews.get(&tab_id).unwrap())
    }

    /// Switch to a different tab's WebView
    pub fn switch_to_webview(&mut self, tab_id: i32) -> anyhow::Result<&WebView> {
        log::info!("switch_to_webview: from {:?} to {}", self.active_webview, tab_id);

        if !self.webviews.contains_key(&tab_id) {
            anyhow::bail!("No cached wry WebView for tab {}", tab_id);
        }

        self.last_access.insert(tab_id, SystemTime::now());
        self.active_webview = Some(tab_id);

        log::info!("Switched to tab {}", tab_id);
        Ok(self.webviews.get(&tab_id).unwrap())
    }

    /// Remove a WebView
    pub fn remove_webview(&mut self, tab_id: i32) {
        log::info!("remove_webview: tab_id={}", tab_id);

        if let Some(_webview) = self.webviews.remove(&tab_id) {
            self.last_access.remove(&tab_id);
            log::info!("Removed wry WebView for tab {} (remaining: {})",
                       tab_id, self.webviews.len());

            // If this was the active webview, clear it
            if self.active_webview == Some(tab_id) {
                self.active_webview = None;
            }
        }
    }

    /// Clear all cached WebViews
    pub fn clear_cache(&mut self) {
        log::info!("clear_cache: destroying {} cached wry WebViews", self.webviews.len());

        self.webviews.clear();
        self.last_access.clear();
        self.active_webview = None;

        log::info!("wry WebView cache cleared");
    }

    /// Evict least recently used WebView
    fn evict_lru(&mut self) {
        if self.last_access.is_empty() {
            return;
        }

        // Find LRU tab
        let lru_tab_id = self.last_access
            .iter()
            .min_by_key(|(_, time)| *time)
            .map(|(id, _)| *id);

        if let Some(tab_id) = lru_tab_id {
            log::info!("Evicting LRU wry WebView for tab {}", tab_id);
            self.remove_webview(tab_id);
        }
    }

    /// Navigate a WebView to a URL
    pub fn navigate(&mut self, tab_id: i32, url: &str) -> anyhow::Result<()> {
        if let Some(webview) = self.webviews.get_mut(&tab_id) {
            webview.load_url(url);
            log::info!("Navigated tab {} to {}", tab_id, url);
            Ok(())
        } else {
            anyhow::bail!("WebView not found for tab {}", tab_id)
        }
    }

    /// Navigate current active WebView to URL
    pub fn navigate_current(&mut self, url: &str) -> anyhow::Result<()> {
        if let Some(tab_id) = self.active_webview {
            self.navigate(tab_id, url)
        } else {
            anyhow::bail!("No active wry WebView to navigate")
        }
    }

    /// Set the active WebView
    pub fn set_active(&mut self, tab_id: i32) {
        self.active_webview = Some(tab_id);
        self.last_access.insert(tab_id, SystemTime::now());
    }

    /// Get the active WebView ID
    pub fn get_active(&self) -> Option<i32> {
        self.active_webview
    }

    /// Check if a WebView exists for a tab
    pub fn has_webview(&self, tab_id: i32) -> bool {
        self.webviews.contains_key(&tab_id)
    }

    /// Get cache size
    pub fn cache_size(&self) -> usize {
        self.webviews.len()
    }
}

#[cfg(not(target_os = "android"))]
pub struct AndroidWebViewManager;

#[cfg(not(target_os = "android"))]
impl AndroidWebViewManager {
    pub fn new() -> Self {
        Self
    }
}
