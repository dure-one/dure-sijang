use crate::models::browser::{Bookmark, Tab};
use egui::Id;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Bookmarks,
    History,
    Settings,
    About,
}

#[derive(Debug, Clone)]
pub struct BrowserState {
    pub tabs: Vec<WebTab>,
    pub active_tab_index: Option<usize>,
    pub sidebar_open: bool,
    pub menu_sidebar_open: bool, // NEW: separate menu sidebar
    pub url_input: String,
    pub next_tab_id: usize,
    pub bookmarks: Vec<Bookmark>,
    pub sidebar_tab: SidebarTab,
    pub history_entries: Vec<crate::models::browser::BrowsingHistory>,
    pub resize_toggle_step: u8, // 0=none, 1=toggled, 2=restore
}

#[derive(Debug, Clone)]
pub struct WebTab {
    pub id: Id,
    pub db_id: Option<i32>,
    pub url_bar: String,
    pub title: String,
    // Note: EguiWebView cannot be cloned, so it's stored separately in the app
    // and referenced by id. This struct just holds the tab metadata.
}

impl BrowserState {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            active_tab_index: None,
            sidebar_open: true,
            menu_sidebar_open: false, // Menu sidebar starts closed
            url_input: String::new(),
            next_tab_id: 0,
            bookmarks: Vec::new(),
            sidebar_tab: SidebarTab::Bookmarks,
            history_entries: Vec::new(),
            resize_toggle_step: 0,
        }
    }

    pub fn load_history(&mut self) {
        match crate::db_browser::get_all_history(100) {
            Ok(entries) => {
                self.history_entries = entries;
                log::info!("Loaded {} history entries", self.history_entries.len());
            }
            Err(e) => {
                log::error!("Failed to load history: {}", e);
                self.history_entries = Vec::new();
            }
        }
    }

    pub fn refresh_history(&mut self) {
        self.load_history();
    }

    pub fn should_open_new_tab(&self, history_tab_id: i32) -> bool {
        if let Some(active_idx) = self.active_tab_index {
            if active_idx < self.tabs.len() {
                if let Some(active_db_id) = self.tabs[active_idx].db_id {
                    return history_tab_id != active_db_id;
                }
            }
        }
        true // Default to new tab if can't determine
    }

    pub fn load_from_db() -> Self {
        let mut state = Self::new();

        // Load tabs from database
        if let Ok(db_tabs) = crate::db_browser::get_all_tabs() {
            state.tabs = db_tabs
                .into_iter()
                .map(|tab| WebTab {
                    id: Id::new(format!("tab_{}", tab.id.unwrap_or(0))),
                    db_id: tab.id,
                    url_bar: tab.url.clone(),
                    title: tab.title.clone(),
                })
                .collect();

            if !state.tabs.is_empty() {
                state.active_tab_index = Some(0);
                state.url_input = state.tabs[0].url_bar.clone();
            }
        }

        // Load bookmarks from database
        if let Ok(bookmarks) = crate::db_browser::get_bookmarks() {
            state.bookmarks = bookmarks;
        }

        state.sidebar_tab = SidebarTab::Bookmarks;
        state.load_history();

        state
    }

    pub fn add_tab(&mut self, url: &str, title: &str) -> Id {
        let tab_id = Id::new(format!("tab_{}", self.next_tab_id));

        // Create database entry
        let db_id = crate::db_browser::create_tab(title, url, "webview")
            .ok()
            .and_then(|tab| tab.id);

        let tab = WebTab {
            id: tab_id,
            db_id,
            url_bar: url.to_string(),
            title: title.to_string(),
        };

        self.tabs.push(tab);
        self.active_tab_index = Some(self.tabs.len() - 1);
        self.url_input = url.to_string();
        self.next_tab_id += 1;

        tab_id
    }

    pub fn close_tab(&mut self, idx: usize) {
        if idx >= self.tabs.len() {
            return;
        }

        // Delete from database
        if let Some(db_id) = self.tabs[idx].db_id {
            let _ = crate::db_browser::delete_tab(db_id);
        }

        self.tabs.remove(idx);

        // Update active tab index
        if self.tabs.is_empty() {
            self.active_tab_index = None;
            self.url_input.clear();
        } else if let Some(active) = self.active_tab_index {
            if active >= self.tabs.len() {
                self.active_tab_index = Some(self.tabs.len() - 1);
            } else if idx <= active && active > 0 {
                self.active_tab_index = Some(active - 1);
            }

            if let Some(new_active) = self.active_tab_index {
                self.url_input = self.tabs[new_active].url_bar.clone();
            }
        }
    }

    pub fn update_tab_url(&mut self, idx: usize, url: &str, title: Option<&str>) {
        if idx >= self.tabs.len() {
            return;
        }

        self.tabs[idx].url_bar = url.to_string();
        if let Some(t) = title {
            self.tabs[idx].title = t.to_string();
        }

        // Update database
        if let Some(db_id) = self.tabs[idx].db_id {
            let _ = crate::db_browser::update_tab(db_id, &self.tabs[idx].title, url);

            // Add to history
            let _ = crate::db_browser::add_history(db_id, url, title);
        }

        // Update URL bar if this is the active tab
        if self.active_tab_index == Some(idx) {
            self.url_input = url.to_string();
        }
    }

    pub fn add_bookmark(&mut self, title: &str, url: &str) -> anyhow::Result<()> {
        // Prevent duplicate bookmarks
        if self.is_bookmarked(url) {
            return Ok(());
        }
        crate::db_browser::add_bookmark(title, url, None)?;
        self.bookmarks = crate::db_browser::get_bookmarks()?;
        Ok(())
    }

    pub fn delete_bookmark(&mut self, idx: usize) -> anyhow::Result<()> {
        if idx >= self.bookmarks.len() {
            return Ok(());
        }

        if let Some(db_id) = self.bookmarks[idx].id {
            crate::db_browser::delete_bookmark(db_id)?;
            self.bookmarks = crate::db_browser::get_bookmarks()?;
        }

        Ok(())
    }

    pub fn is_bookmarked(&self, url: &str) -> bool {
        crate::db_browser::bookmark_exists(url).unwrap_or(false)
    }

    /// Force webview resize by toggling sidebar state over two frames
    /// This triggers egui layout recalculation
    pub fn force_sidebar_resize(&mut self) {
        log::info!(
            "Scheduling sidebar resize toggle (current state: {})",
            self.sidebar_open
        );
        self.resize_toggle_step = 1; // Start toggle cycle
    }

    /// Process resize toggle if pending (call each frame)
    pub fn process_resize_toggle(&mut self) {
        match self.resize_toggle_step {
            1 => {
                // Step 1: Toggle to opposite state
                self.sidebar_open = !self.sidebar_open;
                log::info!("Resize toggle step 1: toggled to {}", self.sidebar_open);
                self.resize_toggle_step = 2;
            }
            2 => {
                // Step 2: Restore to original state
                self.sidebar_open = !self.sidebar_open;
                log::info!("Resize toggle step 2: restored to {}", self.sidebar_open);
                self.resize_toggle_step = 0; // Done
            }
            _ => {} // No toggle pending
        }
    }

    /// Explicitly open the sidebar
    pub fn open_sidebar(&mut self) {
        if !self.sidebar_open {
            log::info!("Opening sidebar");
        }
        self.sidebar_open = true;
    }

    /// Explicitly close the sidebar
    pub fn close_sidebar(&mut self) {
        if self.sidebar_open {
            log::info!("Closing sidebar");
        }
        self.sidebar_open = false;
    }

    /// Toggle sidebar state (for hamburger button)
    pub fn toggle_sidebar(&mut self) {
        self.sidebar_open = !self.sidebar_open;
        log::info!("Toggled sidebar to {}", self.sidebar_open);
    }
}
