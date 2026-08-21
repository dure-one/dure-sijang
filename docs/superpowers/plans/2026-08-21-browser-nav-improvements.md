# Browser Navigation and UI Improvements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix non-functional forward/previous navigation buttons, add browsing history sidebar with tabbed interface, modernize all toolbar buttons with Material Design, and remove vertical separators.

**Architecture:** State-Only history loading (fetch 100 entries from database on startup and after navigation). Per-tab navigation handled by webview widgets internally. Tabbed sidebar switches between Bookmarks and History views using Material Design selectable labels.

**Tech Stack:** Rust, egui, egui_material3, egui_webview, Diesel (SQLite), smol async runtime

## Global Constraints

- Follow Rust 2021 edition conventions (snake_case, CamelCase, SCREAMING_SNAKE_CASE)
- Use `anyhow::Result` for error handling (application code)
- Never use `.unwrap()` or `.expect()` in production code
- All database operations through Diesel ORM
- Material Design icons via `crate::material_symbol_icons::icon()`
- Use `MaterialButton::small()` for all toolbar buttons
- Follow existing MVVM patterns (state in BrowserState, rendering in app)
- Commit frequently with descriptive messages
- Test coverage: manual UI testing for egui components

---

## File Structure

### Files to Modify

1. **`mobile/src/browser_stt.rs`** (State Management)
   - Add `SidebarTab` enum (Bookmarks | History)
   - Add `sidebar_tab` and `history_entries` fields to `BrowserState`
   - Add `load_history()`, `refresh_history()`, `should_open_new_tab()` methods
   - Update `new()` and `load_from_db()`

2. **`mobile/src/dure_sijang_app.rs`** (UI Rendering)
   - Add `check_can_navigate_back()` helper function
   - Add `check_can_navigate_forward()` helper function
   - Add `render_bookmarks_list()` helper function (extract existing code)
   - Add `render_history_list()` helper function (new)
   - Modify `render_browser_ui()` method:
     - Lines 1097-1140: Replace sidebar with tabbed interface
     - Lines 1142-1226: Replace all buttons with Material Design, wire navigation, remove separators

### Files Referenced (No Changes)

- `mobile/src/db_browser.rs` - Database operations (already complete)
- `mobile/src/models/browser.rs` - Data models (already complete)
- `mobile/src/material_symbol_icons.rs` - Icon definitions (already complete)

---

### Task 1: Add State Management for History

**Files:**
- Modify: `mobile/src/browser_stt.rs:1-167`

**Interfaces:**
- Consumes: `crate::db_browser::get_all_history(limit: i64) -> Result<Vec<BrowsingHistory>>`
- Consumes: `crate::models::browser::BrowsingHistory` struct
- Produces: `SidebarTab` enum (Copy, PartialEq, Debug, Clone)
- Produces: `BrowserState::sidebar_tab: SidebarTab`
- Produces: `BrowserState::history_entries: Vec<BrowsingHistory>`
- Produces: `BrowserState::load_history(&mut self)`
- Produces: `BrowserState::refresh_history(&mut self)`
- Produces: `BrowserState::should_open_new_tab(&self, history_tab_id: i32) -> bool`

- [ ] **Step 1: Read current browser_stt.rs to understand structure**

```bash
head -50 mobile/src/browser_stt.rs
```

Expected: See existing struct fields and imports

- [ ] **Step 2: Add SidebarTab enum after imports**

Add after line 2 (after `use crate::models::browser::{Tab, Bookmark};`):

```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Bookmarks,
    History,
}
```

- [ ] **Step 3: Add new fields to BrowserState struct**

In the `BrowserState` struct (around line 5), add before the closing brace:

```rust
    pub sidebar_tab: SidebarTab,
    pub history_entries: Vec<crate::models::browser::BrowsingHistory>,
```

- [ ] **Step 4: Update BrowserState::new() to initialize new fields**

In the `new()` method (around line 26), add before the closing brace:

```rust
            sidebar_tab: SidebarTab::Bookmarks,
            history_entries: Vec::new(),
```

- [ ] **Step 5: Add load_history() method to BrowserState impl block**

Add after the `new()` method (around line 35):

```rust
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
        true  // Default to new tab if can't determine
    }
```

- [ ] **Step 6: Update load_from_db() to initialize sidebar_tab and load history**

In the `load_from_db()` method (around line 61), add before the final `state` return:

```rust
        state.sidebar_tab = SidebarTab::Bookmarks;
        state.load_history();
```

- [ ] **Step 7: Verify compilation**

```bash
cargo check --package mobile
```

Expected: Compilation succeeds with no errors

- [ ] **Step 8: Commit state management changes**

```bash
git add mobile/src/browser_stt.rs
git commit -m "feat(browser): add state management for history sidebar

- Add SidebarTab enum (Bookmarks | History)
- Add sidebar_tab and history_entries fields to BrowserState
- Implement load_history() with error handling
- Implement refresh_history() as alias
- Implement should_open_new_tab() for smart navigation
- Initialize fields in new() and load_from_db()

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Add Helper Functions for UI Rendering

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1090-1329`

**Interfaces:**
- Consumes: `DureSijangApp::browser_state: BrowserState`
- Consumes: `DureSijangApp::webview_widgets: HashMap<Id, EguiWebView>`
- Consumes: `BrowserState::sidebar_tab: SidebarTab`
- Consumes: `BrowserState::history_entries: Vec<BrowsingHistory>`
- Consumes: `BrowserState::should_open_new_tab(history_tab_id: i32) -> bool`
- Produces: `check_can_navigate_back(app: &DureSijangApp) -> bool`
- Produces: `check_can_navigate_forward(app: &DureSijangApp) -> bool`
- Produces: `render_bookmarks_list(app: &mut DureSijangApp, ui: &mut Ui, frame: &mut Frame)`
- Produces: `render_history_list(app: &mut DureSijangApp, ui: &mut Ui, frame: &mut Frame)`

- [ ] **Step 1: Add check_can_navigate_back() function before render_browser_ui()**

Add before line 1093 (before `pub fn render_browser_ui`):

```rust
/// Check if the active tab can navigate back
fn check_can_navigate_back(app: &DureSijangApp) -> bool {
    let Some(idx) = app.browser_state.active_tab_index else {
        return false;
    };
    
    if idx >= app.browser_state.tabs.len() {
        return false;
    }
    
    let tab_id = app.browser_state.tabs[idx].id;
    
    // Check if webview exists (egui_webview doesn't expose can_go_back())
    app.webview_widgets.get(&tab_id).is_some()
}

/// Check if the active tab can navigate forward
fn check_can_navigate_forward(app: &DureSijangApp) -> bool {
    let Some(idx) = app.browser_state.active_tab_index else {
        return false;
    };
    
    if idx >= app.browser_state.tabs.len() {
        return false;
    }
    
    let tab_id = app.browser_state.tabs[idx].id;
    
    // Check if webview exists (egui_webview doesn't expose can_go_forward())
    app.webview_widgets.get(&tab_id).is_some()
}
```

- [ ] **Step 2: Add render_bookmarks_list() function (extract from existing code)**

Add after the navigation check functions:

```rust
/// Render the bookmarks list in the sidebar
fn render_bookmarks_list(app: &mut DureSijangApp, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::ScrollArea;
    
    let mut bookmark_to_navigate: Option<(String, String)> = None;
    let mut bookmark_to_delete: Option<usize> = None;

    ScrollArea::vertical().show(ui, |ui| {
        for (idx, bookmark) in app.browser_state.bookmarks.iter().enumerate() {
            ui.horizontal(|ui| {
                if ui.button(&bookmark.title).clicked() {
                    bookmark_to_navigate = Some((bookmark.url.clone(), bookmark.title.clone()));
                }
                if ui.small_button("×").clicked() {
                    bookmark_to_delete = Some(idx);
                }
            });
        }
    });

    // Handle bookmark actions after rendering
    if let Some((url, title)) = bookmark_to_navigate {
        // Create new tab with bookmarked URL
        app.add_browser_tab(ui.ctx(), frame, &url);

        // Update title for the newly created tab
        if let Some(last_idx) = app.browser_state.tabs.len().checked_sub(1) {
            app.browser_state.tabs[last_idx].title = title;
        }

        log::info!("Opened bookmark in new tab: {}", url);
    }
    if let Some(idx) = bookmark_to_delete {
        let _ = app.browser_state.delete_bookmark(idx);
    }
}
```

- [ ] **Step 3: Add render_history_list() function**

Add after render_bookmarks_list():

```rust
/// Render the browsing history list in the sidebar
fn render_history_list(app: &mut DureSijangApp, ui: &mut egui::Ui, frame: &mut eframe::Frame) {
    use egui::ScrollArea;
    
    if app.browser_state.history_entries.is_empty() {
        ui.centered_and_justified(|ui| {
            ui.label("No browsing history yet");
        });
        return;
    }
    
    let mut url_to_navigate: Option<(String, i32)> = None;
    
    ScrollArea::vertical().show(ui, |ui| {
        for entry in &app.browser_state.history_entries {
            let title = entry.title.as_deref().unwrap_or(&entry.url);
            if ui.button(title).clicked() {
                url_to_navigate = Some((entry.url.clone(), entry.tab_id));
            }
        }
    });
    
    // Handle history click after rendering
    if let Some((url, tab_id)) = url_to_navigate {
        if app.browser_state.should_open_new_tab(tab_id) {
            // Open in new tab (different tab or tab doesn't exist)
            app.add_browser_tab(ui.ctx(), frame, &url);
            log::info!("Opened history in new tab: {}", url);
        } else {
            // Navigate current tab (same tab)
            if let Some(idx) = app.browser_state.active_tab_index {
                app.browser_state.update_tab_url(idx, &url, None);
                let tab_id = app.browser_state.tabs[idx].id;
                if let Some(view) = app.webview_widgets.get(&tab_id) {
                    let _ = view.view.load_url(&url);
                    log::info!("Navigated to history URL: {}", url);
                }
            }
        }
    }
}
```

- [ ] **Step 4: Import SidebarTab enum at top of file**

Add to the existing imports section (around line 19):

```rust
use crate::browser_stt::SidebarTab;
```

- [ ] **Step 5: Verify compilation**

```bash
cargo check --package mobile
```

Expected: Compilation succeeds with no errors

- [ ] **Step 6: Commit helper functions**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "feat(browser): add helper functions for navigation and rendering

- Add check_can_navigate_back() to check if back navigation available
- Add check_can_navigate_forward() to check if forward navigation available
- Extract render_bookmarks_list() from inline code for reusability
- Add render_history_list() with smart navigation logic
- Import SidebarTab enum for tabbed sidebar

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Update Toolbar with Material Design Buttons

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1142-1226`

**Interfaces:**
- Consumes: `check_can_navigate_back(app: &DureSijangApp) -> bool`
- Consumes: `check_can_navigate_forward(app: &DureSijangApp) -> bool`
- Consumes: `DureSijangApp::navigate_back(idx: usize) -> anyhow::Result<()>`
- Consumes: `DureSijangApp::navigate_forward(idx: usize) -> anyhow::Result<()>`
- Consumes: `BrowserState::refresh_history(&mut self)`
- Consumes: `crate::material_symbol_icons::icon(name: &str) -> &'static str`
- Produces: Updated toolbar UI with Material buttons and working navigation

- [ ] **Step 1: Replace hamburger menu button with Material Design**

Find line 1146 (`let items_button = ui.add(MaterialButton::filled("≡").small());`) and verify it's already using MaterialButton.

Update to use Material icon:

```rust
                let items_button = ui.add(
                    MaterialButton::filled(crate::material_symbol_icons::icon("menu"))
                        .small()
                );
```

- [ ] **Step 2: Remove separator after hamburger menu**

Delete line 1153 (`ui.separator();`)

- [ ] **Step 3: Replace sidebar toggle button with Material Design and dynamic icon**

Replace lines 1155-1158 with:

```rust
                // Sidebar toggle
                let sidebar_icon = if self.browser_state.sidebar_open {
                    crate::material_symbol_icons::icon("chevron_left")
                } else {
                    crate::material_symbol_icons::icon("chevron_right")
                };
                if ui.add(MaterialButton::filled(sidebar_icon).small()).clicked() {
                    self.browser_state.sidebar_open = !self.browser_state.sidebar_open;
                }
```

- [ ] **Step 4: Remove separator after sidebar toggle**

Delete the `ui.separator();` line after sidebar toggle

- [ ] **Step 5: Replace back button and wire to navigate_back()**

Replace lines 1162-1168 (back button) with:

```rust
                // Back button
                let can_go_back = check_can_navigate_back(self);
                let back_btn = ui.add_enabled(
                    can_go_back,
                    MaterialButton::filled(crate::material_symbol_icons::icon("arrow_back"))
                        .small()
                );
                if back_btn.clicked() {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        if let Err(e) = self.navigate_back(idx) {
                            log::warn!("Back navigation failed: {}", e);
                        } else {
                            self.browser_state.refresh_history();
                        }
                    }
                }
```

- [ ] **Step 6: Replace forward button and wire to navigate_forward()**

Replace lines 1170-1176 (forward button) with:

```rust
                // Forward button
                let can_go_forward = check_can_navigate_forward(self);
                let fwd_btn = ui.add_enabled(
                    can_go_forward,
                    MaterialButton::filled(crate::material_symbol_icons::icon("arrow_forward"))
                        .small()
                );
                if fwd_btn.clicked() {
                    if let Some(idx) = self.browser_state.active_tab_index {
                        if let Err(e) = self.navigate_forward(idx) {
                            log::warn!("Forward navigation failed: {}", e);
                        } else {
                            self.browser_state.refresh_history();
                        }
                    }
                }
```

- [ ] **Step 7: Remove separator before URL input**

Delete the `ui.separator();` line before URL input (around line 1178)

- [ ] **Step 8: Replace Go button with Material Design**

Replace line 1203 (Go button) with:

```rust
                if ui.add(MaterialButton::filled("Go").small()).clicked() {
```

- [ ] **Step 9: Add refresh_history() call after Go button navigation**

After the webview navigation in the Go button block (around line 1211), add:

```rust
                            self.browser_state.refresh_history();
```

- [ ] **Step 10: Replace bookmark button with Material Design icon**

Replace line 1218 (bookmark button) with:

```rust
                // Bookmark button
                let bookmark_icon = crate::material_symbol_icons::icon("bookmark");
                if ui.add(MaterialButton::filled(bookmark_icon).small()).clicked() {
```

- [ ] **Step 11: Verify compilation**

```bash
cargo check --package mobile
```

Expected: Compilation succeeds with no errors

- [ ] **Step 12: Commit toolbar updates**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "feat(browser): modernize toolbar with Material Design buttons

- Replace all toolbar buttons with MaterialButton::small()
- Use Material icons (menu, chevron_left/right, arrow_back/forward, bookmark)
- Wire back/forward buttons to navigate_back()/navigate_forward()
- Add button enable/disable logic based on navigation state
- Call refresh_history() after navigation
- Remove all vertical separators for cleaner UI

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Update Sidebar with Tabbed Interface

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1097-1140`

**Interfaces:**
- Consumes: `SidebarTab` enum
- Consumes: `BrowserState::sidebar_tab: SidebarTab`
- Consumes: `render_bookmarks_list(app, ui, frame)`
- Consumes: `render_history_list(app, ui, frame)`
- Produces: Tabbed sidebar UI with Bookmarks and History views

- [ ] **Step 1: Replace entire sidebar panel content with tabbed interface**

Replace lines 1097-1140 (entire sidebar panel) with:

```rust
        // 1. Left Sidebar Panel (collapsible, resizable)
        if self.browser_state.sidebar_open {
            SidePanel::left("browser_sidebar")
                .resizable(true)
                .default_width(200.0)
                .show_separator_line(true)
                .show_inside(ui, |ui| {
                    // Material Design tab switcher
                    ui.horizontal(|ui| {
                        if ui.selectable_label(
                            self.browser_state.sidebar_tab == SidebarTab::Bookmarks,
                            "📚 Bookmarks"
                        ).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::Bookmarks;
                        }
                        
                        if ui.selectable_label(
                            self.browser_state.sidebar_tab == SidebarTab::History,
                            "🕒 History"
                        ).clicked() {
                            self.browser_state.sidebar_tab = SidebarTab::History;
                        }
                    });
                    ui.separator();
                    
                    // Content area based on active tab
                    match self.browser_state.sidebar_tab {
                        SidebarTab::Bookmarks => render_bookmarks_list(self, ui, frame),
                        SidebarTab::History => render_history_list(self, ui, frame),
                    }
                });
        }
```

- [ ] **Step 2: Verify compilation**

```bash
cargo check --package mobile
```

Expected: Compilation succeeds with no errors

- [ ] **Step 3: Build the application**

```bash
cargo build --package mobile
```

Expected: Build succeeds

- [ ] **Step 4: Commit sidebar updates**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "feat(browser): add tabbed sidebar interface

- Replace bookmarks-only sidebar with tabbed interface
- Add Bookmarks and History tab selectors
- Use selectable_label for Material Design tab switching
- Call render_bookmarks_list() for Bookmarks tab
- Call render_history_list() for History tab
- Maintain resizable and collapsible sidebar behavior

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Manual Testing and Verification

**Files:**
- Test: Application runtime behavior

**Interfaces:**
- Validates: All requirements from design spec
- Validates: Navigation buttons functional
- Validates: History sidebar displays and navigates correctly
- Validates: Material Design buttons render correctly

- [ ] **Step 1: Run the application**

```bash
cargo run --package mobile
```

Expected: Application launches without errors

- [ ] **Step 2: Test sidebar tab switching**

Action:
1. Click "🕒 History" tab in sidebar
2. Click "📚 Bookmarks" tab in sidebar

Expected: Sidebar content switches between history list and bookmarks list

- [ ] **Step 3: Navigate and check history updates**

Action:
1. Navigate to https://dure.app
2. Click a link on the page
3. Switch to History tab in sidebar

Expected: New entries appear in history list

- [ ] **Step 4: Test back navigation**

Action:
1. Click the back button (arrow_back icon)

Expected: Browser navigates back to previous page, history refreshes

- [ ] **Step 5: Test forward navigation**

Action:
1. After going back, click the forward button (arrow_forward icon)

Expected: Browser navigates forward to next page

- [ ] **Step 6: Test Material Design button styling**

Action: Observe all toolbar buttons

Expected:
- ✅ Hamburger menu: Material "menu" icon, small button
- ✅ Sidebar toggle: "chevron_left" or "chevron_right" icon, small button
- ✅ Back button: "arrow_back" icon, small button
- ✅ Forward button: "arrow_forward" icon, small button
- ✅ Go button: Text "Go", small button
- ✅ Bookmark button: "bookmark" icon, small button

- [ ] **Step 7: Test no vertical separators**

Action: Observe toolbar layout

Expected: No vertical separator lines between buttons, clean compact layout

- [ ] **Step 8: Test history navigation - same tab**

Action:
1. Navigate to several pages in the same tab
2. Switch to History tab in sidebar
3. Click a history entry from the current tab

Expected: Current tab navigates to that URL

- [ ] **Step 9: Test history navigation - new tab**

Action:
1. Click a history entry from a different tab

Expected: Opens a new tab with that URL

- [ ] **Step 10: Test empty history state**

Action:
1. Fresh install or cleared history
2. Switch to History tab

Expected: Shows "No browsing history yet" message

- [ ] **Step 11: Test bookmark functionality preserved**

Action:
1. Switch to Bookmarks tab
2. Click a bookmark
3. Delete a bookmark with × button

Expected: Bookmarks still work exactly as before

- [ ] **Step 12: Final verification commit**

```bash
git commit --allow-empty -m "test(browser): manual testing complete

All requirements verified:
- ✅ Forward/back navigation functional
- ✅ History sidebar with tabbed interface  
- ✅ Material Design buttons throughout
- ✅ Vertical separators removed
- ✅ Smart history navigation working
- ✅ Bookmarks functionality preserved

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Execution Summary

**Total Tasks:** 5
**Estimated Time:** 2-3 hours

**Task Dependencies:**
- Task 1 → Task 2 (state fields needed for helpers)
- Task 2 → Task 3, 4 (helpers needed for UI)
- Task 3, 4 → Task 5 (UI needed for testing)
