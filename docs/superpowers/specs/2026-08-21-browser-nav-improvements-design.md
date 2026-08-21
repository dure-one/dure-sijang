# Browser Navigation and UI Improvements Design

**Date:** 2026-08-21  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved  
**Complexity:** Medium

## Overview

This design improves the dure-sijang browser by fixing non-functional navigation buttons, adding browsing history to the sidebar, modernizing button styling with Material Design, and streamlining the toolbar UI by removing visual separators.

## Requirements

### Functional Requirements

1. **Forward/Previous Navigation**
   - Wire up existing `navigate_back()` and `navigate_forward()` methods to UI buttons
   - Visually disable buttons when navigation history unavailable
   - Each tab maintains independent navigation stack (handled by webview)

2. **Browsing History in Sidebar**
   - Display last 100 browsing history entries
   - Tabbed sidebar interface: "Bookmarks" | "History" tabs at top
   - Smart navigation: open in current tab if history from same tab, new tab otherwise
   - Material Design tabs for switching views

3. **UI Modernization**
   - Replace all `ui.button()` with `MaterialButton::small()`
   - Use Material Design icons: `arrow_back`, `arrow_forward`, `menu`, `bookmark`, `history`
   - Remove all vertical separators (`ui.separator()`) from toolbar
   - Cleaner, more compact toolbar design

### Non-Functional Requirements

- **Performance:** Database queries <1ms for 100 history entries
- **Memory:** ~10KB additional for history cache
- **Maintainability:** Follow existing MVVM patterns in codebase

## Architectural Decisions

### Decision 1: State-Only History (Approach A)

**Options Considered:**
- **A) State-Only (Selected):** Load history into `BrowserState`, refresh on navigation
- **B) Cached History:** Cache with lazy loading and invalidation
- **C) Per-Tab Navigation Stacks:** Mirror webview state in Rust

**Decision:** Approach A - State-Only History

**Rationale:**
- Simplicity: Minimal code changes, easy to debug
- Performance: SQLite queries are fast enough (<1ms for 100 rows)
- Maintainability: Matches existing bookmark pattern
- KISS Principle: Avoid premature optimization

### Decision 2: Per-Tab Navigation via Webview

**Decision:** Use `egui_webview::EguiWebView`'s internal navigation stack

**Rationale:**
- Each webview widget already maintains back/forward history
- No need to duplicate this state in Rust
- Query webview for button enable/disable state
- Zero additional memory overhead

### Decision 3: Material Design Components

**Decision:** Use `MaterialButton::small()` for all toolbar buttons

**Rationale:**
- Consistent with existing hamburger menu button
- Better touch targets for mobile (Android support)
- Modern, clean aesthetic
- Part of existing `egui_material3` dependency

## Design Details

### 1. State Management Changes

#### File: `mobile/src/browser_stt.rs`

**New Types:**
```rust
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SidebarTab {
    Bookmarks,
    History,
}
```

**BrowserState Modifications:**
```rust
pub struct BrowserState {
    // ... existing fields ...
    pub sidebar_tab: SidebarTab,           // Active sidebar tab
    pub history_entries: Vec<BrowsingHistory>,  // Last 100 entries
}
```

**New Methods:**

1. **`load_history(&mut self)`**
   - Calls `db_browser::get_all_history(100)`
   - Stores result in `history_entries`
   - Called on app startup and after navigation
   - Error handling: logs error, sets empty vec

2. **`refresh_history(&mut self)`**
   - Alias for `load_history()`
   - Called after any navigation event
   - Keeps history list synchronized

3. **`should_open_new_tab(&self, history_tab_id: i32) -> bool`**
   - Smart navigation logic
   - Returns `true` if history entry from different tab
   - Returns `false` if history from currently active tab
   - Defaults to `true` if unable to determine

**Updated Methods:**

- **`load_from_db()`**: Initialize `sidebar_tab: SidebarTab::Bookmarks`, call `load_history()`
- **`new()`**: Initialize `sidebar_tab: SidebarTab::Bookmarks`, `history_entries: Vec::new()`

### 2. UI Component Changes

#### File: `mobile/src/dure_sijang_app.rs`

**Sidebar Panel (Lines 1097-1140 replacement):**

```rust
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
```

**Toolbar Panel (Lines 1142-1226 modifications):**

Changes:
- Replace `ui.button("≡")` → `MaterialButton::filled(icon("menu")).small()`
- Replace `ui.button("◀◀"/"▶▶")` → `MaterialButton::filled("chevron_left"/"chevron_right").small()`
- Replace `ui.button("◀")` → `MaterialButton::filled(icon("arrow_back")).small()` + wire to `navigate_back()`
- Replace `ui.button("▶")` → `MaterialButton::filled(icon("arrow_forward")).small()` + wire to `navigate_forward()`
- Replace `ui.button("Go")` → `MaterialButton::filled("Go").small()`
- Replace `ui.button("⭐")` → `MaterialButton::filled(icon("bookmark")).small()`
- Remove all `ui.separator()` calls between buttons
- Add `ui.add_enabled(can_go_back, ...)` for back button
- Add `ui.add_enabled(can_go_forward, ...)` for forward button
- Add `self.browser_state.refresh_history()` after Go button navigation

**New Helper Functions:**

1. **`check_can_navigate_back(app: &DureSijangApp) -> bool`**
   ```rust
   fn check_can_navigate_back(app: &DureSijangApp) -> bool {
       let Some(idx) = app.browser_state.active_tab_index else {
           return false;
       };
       
       if idx >= app.browser_state.tabs.len() {
           return false;
       }
       
       let tab_id = app.browser_state.tabs[idx].id;
       
       // Check if webview exists and can go back
       if let Some(view) = app.webview_widgets.get(&tab_id) {
           // Note: egui_webview may not expose can_go_back()
           // Fallback: always enable and let navigate_back() handle failure
           true
       } else {
           false
       }
   }
   ```

2. **`check_can_navigate_forward(app: &DureSijangApp) -> bool`**
   - Same logic as `check_can_navigate_back`
   - Returns `true` if webview exists

3. **`render_bookmarks_list(app: &mut DureSijangApp, ui: &mut Ui, frame: &mut Frame)`**
   - Extract existing bookmark rendering code (lines 1108-1138)
   - No logic changes, just code organization

4. **`render_history_list(app: &mut DureSijangApp, ui: &mut Ui, frame: &mut Frame)`**
   ```rust
   fn render_history_list(app: &mut DureSijangApp, ui: &mut Ui, frame: &mut Frame) {
       let mut url_to_navigate: Option<String> = None;
       
       if app.browser_state.history_entries.is_empty() {
           ui.centered_and_justified(|ui| {
               ui.label("No browsing history yet");
           });
           return;
       }
       
       ScrollArea::vertical().show(ui, |ui| {
           for entry in &app.browser_state.history_entries {
               let title = entry.title.as_deref().unwrap_or(&entry.url);
               if ui.button(title).clicked() {
                   url_to_navigate = Some(entry.url.clone());
                   
                   // Smart navigation logic
                   if app.browser_state.should_open_new_tab(entry.tab_id) {
                       // Open in new tab
                       app.add_browser_tab(ui.ctx(), frame, &entry.url);
                   } else {
                       // Navigate current tab (handled below)
                   }
               }
           }
       });
       
       // Navigate current tab if same-tab history clicked
       if let Some(url) = url_to_navigate {
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
   ```

### 3. Database Layer

**No changes required.** Existing functions sufficient:

- `db_browser::get_all_history(100)` - Fetch last 100 entries
- `db_browser::add_history(tab_id, url, title)` - Already called on navigation
- `browsing_history` table already tracks `tab_id` for smart navigation

### 4. Material Design Icons

**Icon Mapping:**

| Component | Icon Name | Constant |
|-----------|-----------|----------|
| Hamburger menu | `"menu"` | `ICON_MENU` |
| Sidebar toggle (open) | `"chevron_left"` | `ICON_CHEVRON_LEFT` |
| Sidebar toggle (closed) | `"chevron_right"` | `ICON_CHEVRON_RIGHT` |
| Back button | `"arrow_back"` | `ICON_ARROW_BACK` |
| Forward button | `"arrow_forward"` | `ICON_ARROW_FORWARD` |
| Bookmark button | `"bookmark"` | `ICON_BOOKMARK` |

**Usage:**
```rust
use crate::material_symbol_icons::icon;

MaterialButton::filled(icon("arrow_back")).small()
```

## Data Flow

### Flow 1: Back/Forward Navigation

```
1. User clicks back button (MaterialButton::small with arrow_back icon)
   ↓
2. check_can_navigate_back() queries active tab's webview widget
   ↓
3. If enabled: navigate_back(active_tab_index) called
   ↓
4. Webview navigates internally (wry handles history stack)
   ↓
5. WebViewEvent::Loaded fires with new URL
   ↓
6. update_tab_url() updates database + tab metadata
   ↓
7. refresh_history() reloads last 100 entries from DB
   ↓
8. UI re-renders with updated history sidebar
```

### Flow 2: History Sidebar Click

```
1. User switches to History tab in sidebar
   ↓
2. sidebar_tab = SidebarTab::History
   ↓
3. render_history_list() displays history_entries
   ↓
4. User clicks history entry
   ↓
5. should_open_new_tab(history.tab_id) checks if same tab
   ↓
6a. Same tab → navigate_to_url(active_tab, url)
    OR
6b. Different tab → add_browser_tab(ctx, frame, url)
   ↓
7. WebViewEvent::Loaded fires
   ↓
8. refresh_history() updates list
```

### Flow 3: Initial Load

```
1. App startup: DureSijangApp::default()
   ↓
2. BrowserState::load_from_db() called
   ↓
3. Load tabs from database
   ↓
4. Load bookmarks from database
   ↓
5. load_history() - fetch last 100 entries
   ↓
6. sidebar_tab = SidebarTab::Bookmarks (default)
   ↓
7. UI renders with populated state
```

### Flow 4: URL Bar Navigation (Updated)

```
1. User types URL and presses Enter (or clicks Go)
   ↓
2. update_tab_url() updates database + metadata
   ↓
3. webview.load_url() navigates the page
   ↓
4. refresh_history() called immediately  ← NEW
   ↓
5. History sidebar updates if visible
```

## Error Handling

### 1. Webview Navigation Failures

**Scenario:** `navigate_back()` or `navigate_forward()` fails

**Handling:**
```rust
if back_btn.clicked() {
    if let Some(idx) = self.browser_state.active_tab_index {
        if let Err(e) = self.navigate_back(idx) {
            log::warn!("Back navigation failed: {}", e);
            // UI already shows button as enabled, no visual feedback needed
            // Webview state unchanged, user can retry
        }
    }
}
```

**Result:** Log warning, no crash. Button stays enabled for retry.

### 2. Database Query Failures

**Scenario:** `get_all_history()` fails (disk error, corruption)

**Handling:**
```rust
pub fn load_history(&mut self) {
    match crate::db_browser::get_all_history(100) {
        Ok(entries) => self.history_entries = entries,
        Err(e) => {
            log::error!("Failed to load history: {}", e);
            self.history_entries = Vec::new();  // Show empty list
        }
    }
}
```

**Result:** Empty history list shown, app continues functioning.

### 3. No Active Tab

**Scenario:** User clicks back/forward with no tabs open

**Handling:**
```rust
fn check_can_navigate_back(app: &DureSijangApp) -> bool {
    let Some(idx) = app.browser_state.active_tab_index else {
        return false;  // No active tab = disable button
    };
    // ... check webview state ...
}
```

**Result:** Buttons automatically disabled when no tabs.

### 4. Webview Widget Not Found

**Scenario:** Tab metadata exists but webview widget destroyed

**Handling:**
```rust
pub fn navigate_back(&mut self, idx: usize) -> anyhow::Result<()> {
    // ... bounds check ...
    let tab_id = self.browser_state.tabs[idx].id;
    
    let Some(view) = self.webview_widgets.get(&tab_id) else {
        anyhow::bail!("WebView not found for tab {:?}", tab_id)
    };
    
    view.back();
    Ok(())
}
```

**Result:** Returns error (logged), no crash.

### 5. History Entry from Deleted Tab

**Scenario:** User clicks history from a tab that was closed

**Handling:**
```rust
// In render_history_list():
if app.browser_state.should_open_new_tab(entry.tab_id) {
    // Tab doesn't exist anymore OR different tab
    app.add_browser_tab(ui.ctx(), frame, &entry.url);
} else {
    // Same tab still exists, navigate in place
    // ... navigate logic ...
}
```

**Result:** Opens in new tab if original tab closed.

## Edge Cases

| Case | Behavior |
|------|----------|
| Empty history (first run) | Show "No history yet" message in sidebar |
| 100+ history entries | Limit to most recent 100 (database query handles this) |
| Sidebar closed | Tabs still work, history still updates |
| Rapid navigation | Each navigation refreshes history - might see slight lag |
| Bookmark same URL twice | Allowed (database permits duplicates) |
| Navigate to invalid URL | Webview shows error page, history still records it |

## Performance Analysis

### Database Query Frequency

- **Startup:** 3 queries (tabs, bookmarks, history)
- **Per navigation:** 2 queries (update tab, add history entry) + 1 query (refresh history list)
- **Sidebar tab switch:** 0 queries (already loaded in memory)
- **History click:** Same as navigation (2-3 queries)

### Expected Performance

- **SQLite query time:** <1ms for 100 rows on modern hardware
- **Memory overhead:** ~10KB for 100 history entries (100 bytes avg per entry)
- **UI responsiveness:** No noticeable lag

### Bottleneck Analysis

**Potential bottleneck:** Refreshing history on every navigation

**Mitigation:**
- SQLite is fast enough for this use case
- If performance issues arise, switch to Approach B (cached history)
- Current design makes future optimization easy

## Testing Strategy

### Unit Tests

1. **`BrowserState::load_history()`**
   - Test successful load with mock database
   - Test error handling (database failure)
   - Verify 100-entry limit

2. **`BrowserState::should_open_new_tab()`**
   - Test same tab ID → false
   - Test different tab ID → true
   - Test no active tab → true

3. **Helper functions**
   - `check_can_navigate_back()` with various states
   - `check_can_navigate_forward()` with various states

### Integration Tests

1. **Navigation flow**
   - Create tab → navigate → check history updated
   - Verify back button enables after navigation
   - Verify forward button enables after back navigation

2. **Sidebar interaction**
   - Switch between Bookmarks and History tabs
   - Click history entry from same tab
   - Click history entry from different tab

3. **Database persistence**
   - Navigate → restart app → verify history persisted
   - Close tab → verify history entries remain

### Manual Testing Checklist

- [ ] Back button disabled on fresh tab
- [ ] Back button enabled after navigation
- [ ] Forward button enabled after back navigation
- [ ] History tab shows recent navigation
- [ ] Clicking same-tab history navigates in place
- [ ] Clicking different-tab history opens new tab
- [ ] All buttons use Material Design styling
- [ ] No vertical separators in toolbar
- [ ] Sidebar toggle icon changes (chevron_left/right)
- [ ] Empty history shows helpful message

## Migration Plan

### Phase 1: State Management (No UI Changes)

1. Add `SidebarTab` enum to `browser_stt.rs`
2. Add `sidebar_tab` and `history_entries` fields to `BrowserState`
3. Implement `load_history()`, `refresh_history()`, `should_open_new_tab()`
4. Update `load_from_db()` and `new()`

**Verification:** Compile successfully, no runtime changes

### Phase 2: Helper Functions (No UI Changes)

1. Implement `check_can_navigate_back()`
2. Implement `check_can_navigate_forward()`
3. Extract `render_bookmarks_list()` from existing code
4. Implement `render_history_list()`

**Verification:** Compile successfully, bookmarks still work

### Phase 3: Toolbar Updates (Visible Changes)

1. Replace all buttons with `MaterialButton::small()`
2. Wire back/forward buttons to navigation methods
3. Add button enable/disable logic
4. Remove all `ui.separator()` calls
5. Add `refresh_history()` call after Go button navigation

**Verification:** Navigation works, buttons styled correctly, no separators

### Phase 4: Sidebar Updates (Final)

1. Replace sidebar content with tabbed interface
2. Call `render_bookmarks_list()` for Bookmarks tab
3. Call `render_history_list()` for History tab
4. Test sidebar tab switching

**Verification:** Full functionality, all features working

## Future Enhancements

### Potential Improvements (Out of Scope)

1. **Search/filter history** - Add search box above history list
2. **Clear history button** - One-click history deletion
3. **Date grouping** - Group history by "Today", "Yesterday", "This Week"
4. **Favicon caching** - Show site icons next to history entries
5. **Infinite scroll** - Load more than 100 entries on demand
6. **History export** - Export as JSON/CSV
7. **Configurable limit** - Let user choose 50/100/200 entries

## Success Criteria

✅ Forward/back buttons functional and visually indicate enabled/disabled state  
✅ History sidebar shows last 100 entries with smart navigation  
✅ Tabbed sidebar with Bookmarks and History views  
✅ All buttons use Material Design small buttons  
✅ No vertical separators in toolbar  
✅ No performance degradation (queries <1ms)  
✅ All existing bookmark functionality preserved  
✅ Error handling prevents crashes  

## References

- Existing implementation: `mobile/src/dure_sijang_app.rs` lines 1090-1329
- Database operations: `mobile/src/db_browser.rs`
- State management: `mobile/src/browser_stt.rs`
- Material icons: `mobile/src/material_symbol_icons.rs`
- Browser models: `mobile/src/models/browser.rs`
