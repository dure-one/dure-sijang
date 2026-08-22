/// Android WebView integration using wry
///
/// This module provides WebView functionality on Android using the wry crate.
/// Unlike desktop which uses egui_webview, Android directly integrates wry WebViews
/// as native Android View components.

#[cfg(target_os = "android")]
use wry::{WebViewBuilder, WebView};
#[cfg(target_os = "android")]
use raw_window_handle::HasWindowHandle;
use std::collections::HashMap;

/// Manages WebView instances on Android
#[cfg(target_os = "android")]
pub struct AndroidWebViewManager {
    /// Map of tab IDs to WebView instances
    webviews: HashMap<egui::Id, WebView>,
    /// Active webview ID
    active_webview: Option<egui::Id>,
}

#[cfg(target_os = "android")]
impl AndroidWebViewManager {
    /// Create a new WebView manager
    pub fn new() -> Self {
        Self {
            webviews: HashMap::new(),
            active_webview: None,
        }
    }

    /// Create a new WebView for a tab
    pub fn create_webview(
        &mut self,
        tab_id: egui::Id,
        url: &str,
        window: &impl HasWindowHandle,
    ) -> anyhow::Result<()> {
        log::info!("Creating Android WebView for tab {:?} with URL: {}", tab_id, url);

        // Create WebView using wry
        let webview = WebViewBuilder::new()
            .with_url(url)
            .with_user_agent("Mozilla/5.0 (Linux; Android) DureSijang/1.0")
            .with_accept_first_mouse(true)
            .build_as_child(window)?;

        self.webviews.insert(tab_id, webview);
        self.active_webview = Some(tab_id);

        log::info!("Android WebView created successfully for tab {:?}", tab_id);
        Ok(())
    }

    /// Remove a WebView
    pub fn remove_webview(&mut self, tab_id: &egui::Id) {
        if let Some(_webview) = self.webviews.remove(tab_id) {
            log::info!("Removed Android WebView for tab {:?}", tab_id);

            // If this was the active webview, clear it
            if self.active_webview == Some(*tab_id) {
                self.active_webview = None;
            }
        }
    }

    /// Navigate a WebView to a URL
    pub fn navigate(&mut self, tab_id: &egui::Id, url: &str) -> anyhow::Result<()> {
        if let Some(webview) = self.webviews.get_mut(tab_id) {
            webview.load_url(url);
            log::info!("Navigated tab {:?} to {}", tab_id, url);
            Ok(())
        } else {
            anyhow::bail!("WebView not found for tab {:?}", tab_id)
        }
    }

    /// Set the active WebView
    pub fn set_active(&mut self, tab_id: egui::Id) {
        self.active_webview = Some(tab_id);
        // TODO: Show/hide WebViews based on active state
    }

    /// Get the active WebView ID
    pub fn get_active(&self) -> Option<egui::Id> {
        self.active_webview
    }

    /// Check if a WebView exists for a tab
    pub fn has_webview(&self, tab_id: &egui::Id) -> bool {
        self.webviews.contains_key(tab_id)
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
