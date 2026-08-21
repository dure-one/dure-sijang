# Tabbed Browser Fixes Design

**Date:** 2026-08-20  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This spec defines fixes for two broken features in the dure-sijang tabbed browser:

1. **Bookmarks sidebar toggle** - Button changes state but sidebar always visible
2. **WebView rendering** - Tabs created but webview not displayed (TODO placeholder)

**Solution approach**: Use `egui_webview` library (already partially integrated) instead of raw `wry::WebView`. Minimal code changes (~50 lines) for maximum impact.

## Goals

- Fix sidebar collapse/expand toggle
- Render functional webviews in browser tabs
- Maintain existing MVVM architecture (BrowserState for metadata, DureSijangApp for widgets)
- Use proven `egui_webview` library instead of manual wry integration
- Work on all platforms (desktop Linux/OpenBSD, Android)

## Current State Analysis

### Issue 1: Sidebar Toggle Broken

**Location**: `mobile/src/dure_sijang_app.rs:1183-1218`

**Problem**:
```rust
SidePanel::left("browser_sidebar")
    .show_inside(ui, |ui| {
        // bookmarks UI
    });
```

`show_inside()` always renders the panel, ignoring `self.browser_state.sidebar_open` flag. The toggle button (line 1224) correctly flips the flag, but the panel doesn't respect it.

**Root cause**: No conditional rendering - panel always shown.

### Issue 2: WebView Not Rendering

**Location**: `mobile/src/dure_sijang_app.rs:1337-1347`

**Problem**:
```rust
// Browser content area
// TODO: Render egui_webview widgets here
ui.centered_and_justified(|ui| {
    ui.label("WebView content will render here");
});
```

**Related issues**:
- `add_browser_tab()` (line 1162) only updates `browser_state`, doesn't create webviews
- `webviews: HashMap<usize, wry::WebView>` field (dure_sijang_app_stt.rs:166) is unused
- Raw `wry::WebView` requires manual positioning (GTK TODO at line 1044)
- `egui_webview::init_webview()` is called (line 992) but no `EguiWebView` widgets created

**Root cause**: Incomplete port from reference code - initialization exists but widget creation missing.

## Design Decisions

### Decision 1: Use egui_webview Library

**Rationale**:
- Already partially integrated (`init_webview`, `webview_end_frame` called)
- Reference implementation (tabbrowser.rs) is proven and works
- Handles all platform integration (GTK, WebKit, WebView2)
- Much simpler than raw wry (30 lines vs 200+)

**Rejected alternatives**:
- Raw `wry::WebView`: Too complex, GTK integration broken (line 1044-1049 TODO)
- Hybrid approach: Unnecessary complexity

### Decision 2: Separate Storage for Widgets

**Rationale**:
- `EguiWebView` is not `Clone`
- `BrowserState` is `Clone` (used for MVVM pattern)
- Keep metadata (BrowserState) separate from widgets (DureSijangApp)

**Architecture**:
```rust
// Metadata (Clone, database-backed)
pub browser_state: BrowserState {
    tabs: Vec<WebTab>,  // id, url, title
    active_tab_index: Option<usize>,
    sidebar_open: bool,
}

// Widgets (not Clone, runtime only)
pub webview_widgets: HashMap<Id, EguiWebView>,
```

### Decision 3: Conditional Sidebar Rendering

**Rationale**:
- egui 0.33 may not have `show_collapsible()` (added in 0.36)
- Conditional `if` is version-agnostic and simple
- No animation, but functional

**Alternative considered**: `show_animated_inside()` if available in 0.33.

## Architecture Changes

### File: `mobile/src/dure_sijang_app_stt.rs`

**Remove** (lines 166-167):
```rust
pub webviews: std::collections::HashMap<usize, wry::WebView>,
pub window_handle: Option<raw_window_handle::RawWindowHandle>,
```

**Add**:
```rust
// WebView widgets (stored separately from BrowserState metadata)
pub webview_widgets: std::collections::HashMap<egui::Id, egui_webview::EguiWebView>,
```

**Initialize** in `Default::default()`:
```rust
webview_widgets: std::collections::HashMap::new(),
```

### File: `mobile/src/dure_sijang_app.rs`

**Changes**:
1. Wrap sidebar in conditional (line 1183)
2. Create `EguiWebView` in `add_browser_tab()` (line 1162)
3. Render webviews in `render_browser_ui()` (line 1337)
4. Destroy webview in `close_browser_tab()` (line 1171)
5. Update navigation methods to use `webview_widgets` (lines 1098-1150)

## Implementation Details

### Fix 1: Sidebar Toggle

**Location**: `render_browser_ui()` at line 1182

**Change**:
```rust
// Wrap entire SidePanel in conditional
if self.browser_state.sidebar_open {
    SidePanel::left("browser_sidebar")
        .resizable(true)
        .default_width(100.0)
        .show_separator_line(true)
        .show_inside(ui, |ui| {
            ui.vertical_centered(|ui| {
                ui.heading("📚 Bookmarks");
            });
            ui.separator();
            
            // ... bookmark list (unchanged)
        });
}
```

**Behavior**:
- `sidebar_open = true`: Sidebar visible, resizable
- `sidebar_open = false`: Sidebar hidden, content uses full width
- Toggle button (line 1224) already works correctly

### Fix 2: WebView Creation

**Location**: `add_browser_tab()` at line 1162

**Change**:
```rust
pub fn add_browser_tab(&mut self, ctx: &egui::Context, frame: &eframe::Frame, url: &str) {
    // Create tab metadata (unchanged)
    let tab_id = self.browser_state.add_tab(url, url);
    
    // NEW: Create EguiWebView widget
    use egui_webview::EguiWebView;
    let view = EguiWebView::new(ctx, tab_id, frame, |builder| {
        builder.with_url(url)
    });
    
    self.webview_widgets.insert(tab_id, view);
    log::info!("Added browser tab {:?} with URL: {}", tab_id, url);
}
```

**Key points**:
- `tab_id` from `browser_state.add_tab()` is used as HashMap key
- `EguiWebView::new()` takes egui context, ID, frame, and builder closure
- Builder pattern: `.with_url(url)` sets initial URL
- Webview stored in `webview_widgets` HashMap

### Fix 3: WebView Rendering

**Location**: `render_browser_ui()` at line 1336

**Replace TODO placeholder with**:
```rust
// Browser content area - render all webviews, show only active
for (idx, tab) in self.browser_state.tabs.iter_mut().enumerate() {
    let is_active = Some(idx) == self.browser_state.active_tab_index;
    
    // Size control: active tab gets full space, inactive tabs get zero
    let size = if is_active {
        ui.available_size()
    } else {
        egui::vec2(0.0, 0.0)
    };
    
    // Render webview (if exists for this tab)
    if let Some(view) = self.webview_widgets.get_mut(&tab.id) {
        ui.push_id(tab.id, |ui| {
            let response = view.ui(ui, size);
            
            // Handle navigation events (active tab only)
            if is_active {
                for event in response.events {
                    if let egui_webview::WebViewEvent::Loaded(new_url) = event {
                        self.browser_state.update_tab_url(idx, &new_url, None);
                    }
                }
            }
        });
    }
}
```

**Key points**:
- All tabs rendered (webview lifecycle requirement)
- Only active tab visible (size = 0 for inactive)
- `push_id(tab.id)` prevents egui ID conflicts
- `WebViewEvent::Loaded` updates URL bar when page loads
- Events only processed for active tab (performance)

### Fix 4: Tab Cleanup

**Location**: `close_browser_tab()` at line 1171

**Change**:
```rust
pub fn close_browser_tab(&mut self, idx: usize) {
    if idx >= self.browser_state.tabs.len() {
        return;
    }
    
    // NEW: Remove webview widget before removing tab metadata
    let tab_id = self.browser_state.tabs[idx].id;
    if let Some(_view) = self.webview_widgets.remove(&tab_id) {
        log::info!("Destroyed webview for tab {:?}", tab_id);
        // Drop will handle cleanup
    }
    
    // Remove tab metadata (unchanged)
    self.browser_state.close_tab(idx);
    log::info!("Closed browser tab at index {}", idx);
}
```

**Cleanup behavior**:
- Webview removed from HashMap first
- Drop trait handles native webview destruction
- Then metadata removed from BrowserState

### Fix 5: Navigation Methods

**Location**: Lines 1098-1150 (back/forward/reload methods)

**Current code uses**:
```rust
self.webviews.get(&tab_id)  // Wrong - uses raw wry HashMap
```

**Change to**:
```rust
pub fn navigate_back(&mut self, idx: usize) -> anyhow::Result<()> {
    if idx >= self.browser_state.tabs.len() {
        anyhow::bail!("Tab index {} out of bounds", idx);
    }
    
    let tab_id = self.browser_state.tabs[idx].id;
    if let Some(view) = self.webview_widgets.get(&tab_id) {
        view.back();
        Ok(())
    } else {
        anyhow::bail!("WebView not found for tab {:?}", tab_id)
    }
}

pub fn navigate_forward(&mut self, idx: usize) -> anyhow::Result<()> {
    if idx >= self.browser_state.tabs.len() {
        anyhow::bail!("Tab index {} out of bounds", idx);
    }
    
    let tab_id = self.browser_state.tabs[idx].id;
    if let Some(view) = self.webview_widgets.get(&tab_id) {
        view.forward();
        Ok(())
    } else {
        anyhow::bail!("WebView not found for tab {:?}", tab_id)
    }
}

pub fn navigate_reload(&mut self, idx: usize) -> anyhow::Result<()> {
    if idx >= self.browser_state.tabs.len() {
        anyhow::bail!("Tab index {} out of bounds", idx);
    }
    
    let tab_id = self.browser_state.tabs[idx].id;
    if let Some(view) = self.webview_widgets.get(&tab_id) {
        view.reload();
        Ok(())
    } else {
        anyhow::bail!("WebView not found for tab {:?}", tab_id)
    }
}
```

**Note**: These methods are called from toolbar buttons (lines 1231-1244). Change parameter from `tab_id: usize` to `idx: usize` to match.

## Error Handling

### GTK Platform Support

**Old issue** (line 1044-1049):
```rust
#[cfg(target_os = "openbsd")]
let webview = {
    // TODO: GTK webview integration requires GTK widget container
    return Err(anyhow::anyhow!(
        "WebView not yet supported on GTK platforms"
    ));
};
```

**Resolution**: `egui_webview` handles GTK/WebKit integration internally. No platform-specific code needed.

**Verification**: Test on OpenBSD (primary platform per CLAUDE.md).

### WebView Creation Failures

`EguiWebView::new()` does not return `Result` - internal errors are logged by the library. If webview fails to initialize:

- Tab metadata still created
- Webview missing from `webview_widgets` HashMap
- Rendering gracefully skips (`if let Some(view) = ...` guard)
- User sees tab bar but empty content area

**Improvement opportunity**: Add fallback UI when webview missing:
```rust
if let Some(view) = self.webview_widgets.get_mut(&tab.id) {
    view.ui(ui, size);
} else {
    ui.centered_and_justified(|ui| {
        ui.label("⚠️ WebView failed to load");
    });
}
```

## Testing Plan

### Manual Testing

**Test 1: Sidebar Toggle**
1. Launch app, create tab
2. Click sidebar toggle button (◀◀ / ▶▶)
3. Verify sidebar collapses/expands
4. Verify content area resizes to use freed space

**Test 2: WebView Display**
1. Create new tab
2. Enter URL in address bar, click "Go"
3. Verify webview loads and displays page
4. Verify URL bar updates when page loads

**Test 3: Tab Switching**
1. Create 3 tabs with different URLs
2. Click between tabs
3. Verify correct webview shown for each tab
4. Verify URL bar updates to match active tab

**Test 4: Navigation**
1. Load page with links
2. Click link (webview navigates)
3. Click back button (◀)
4. Click forward button (▶)
5. Verify URL bar updates

**Test 5: Tab Closure**
1. Create 3 tabs
2. Close middle tab
3. Verify no crash
4. Verify active tab selection updated correctly
5. Close all tabs
6. Verify "No tabs open" state shown

**Test 6: Bookmarks**
1. Load page
2. Click bookmark button (⭐)
3. Collapse sidebar, verify bookmarks hidden
4. Expand sidebar, click bookmark
5. Verify active tab navigates to bookmarked URL

### Platform Testing

- **Desktop Linux/OpenBSD**: Full test suite
- **Desktop Windows/macOS**: Test 1-5 (if available)
- **Android**: Test 1-5 (webview should use Android WebView)

### Performance Testing

**Tab count stress test**:
1. Create 10 tabs
2. Switch between tabs
3. Verify UI remains responsive
4. Monitor memory usage

**Expected**: Each webview is ~50-100MB. 10 tabs = ~500MB-1GB. Should be acceptable on desktop, monitor on Android.

## Success Criteria

### Functional Requirements

- ✅ Sidebar toggle button collapses/expands sidebar
- ✅ WebView displays loaded pages
- ✅ URL bar shows current page URL
- ✅ Tab switching shows correct webview
- ✅ Back/forward/reload buttons work
- ✅ Tab closure cleans up webview
- ✅ Bookmarks navigate active tab

### Non-Functional Requirements

- ✅ No crashes when creating/closing tabs
- ✅ Works on OpenBSD (primary platform)
- ✅ UI remains responsive with multiple tabs
- ✅ Code compiles without warnings
- ✅ No clippy lints introduced

### Code Quality

- ✅ Follows Rust ECC coding style (immutability, error handling)
- ✅ Minimal changes (~50 lines)
- ✅ No breaking changes to existing APIs
- ✅ Logging for debug visibility

## Migration Notes

### Removed Code

**dure_sijang_app_stt.rs**:
- `webviews: HashMap<usize, wry::WebView>` - unused, replaced by `webview_widgets`
- `window_handle: Option<RawWindowHandle>` - not needed for egui_webview

**dure_sijang_app.rs**:
- Lines 1024-1075: `create_webview()` method - not needed (EguiWebView::new() is simpler)
- Lines 1077-1089: `destroy_webview()` method - replaced by HashMap::remove()

### Behavioral Changes

**Before**:
- Sidebar always visible (broken toggle)
- Webview placeholder text (not functional)

**After**:
- Sidebar respects toggle state
- Functional webview with navigation

**Breaking changes**: None (external APIs unchanged)

## Future Enhancements

### V2 Features (Out of Scope)

1. **Animated sidebar collapse** - Use `show_animated_inside()` if egui 0.33 supports it
2. **Tab dragging** - Reorder tabs via drag-and-drop
3. **Tab persistence** - Restore tabs on app restart (database already supports this)
4. **Webview settings** - User-agent, JavaScript toggle, zoom level
5. **Developer tools** - Right-click → Inspect (if wry supports DevTools)

## References

- Reference implementation: `reference/egui_webview/examples/tabbrowser.rs`
- egui_webview dependency: `mobile/Cargo.toml:egui_webview = { path = "../reference/egui_webview" }`
- Browser state: `mobile/src/browser_stt.rs`
- Database layer: `mobile/src/db_browser.rs`
