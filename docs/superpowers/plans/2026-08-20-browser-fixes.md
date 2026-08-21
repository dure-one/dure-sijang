# Browser Fixes Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Fix broken sidebar toggle and webview rendering in tabbed browser using egui_webview library

**Architecture:** Replace unused `HashMap<usize, wry::WebView>` with `HashMap<Id, EguiWebView>` for widget storage. Maintain MVVM separation (BrowserState for metadata, DureSijangApp for widgets). Use conditional rendering for sidebar collapse/expand.

**Tech Stack:** Rust, egui 0.33, egui_webview (local path dependency), wry (GTK backend on OpenBSD)

## Global Constraints

- Use `egui_webview::EguiWebView` (not raw `wry::WebView`)
- Maintain egui 0.33 compatibility (egui-material3 dependency)
- All changes in `mobile/src/dure_sijang_app.rs` and `mobile/src/dure_sijang_app_stt.rs`
- No breaking changes to existing public APIs
- Follow Rust ECC coding style: no `.unwrap()`, use `?` operator, immutable by default
- Test on OpenBSD (primary platform) before committing

---

### Task 1: Update App Struct for egui_webview Storage

**Files:**
- Modify: `mobile/src/dure_sijang_app_stt.rs:166-167` (remove wry fields)
- Modify: `mobile/src/dure_sijang_app_stt.rs:166` (add webview_widgets field)
- Modify: `mobile/src/dure_sijang_app.rs:~200` (update Default::default())

**Interfaces:**
- Consumes: None (foundational change)
- Produces: `pub webview_widgets: HashMap<egui::Id, egui_webview::EguiWebView>` field available in DureSijangApp

- [ ] **Step 1: Remove unused wry fields from struct**

Edit `mobile/src/dure_sijang_app_stt.rs` around line 166-167:

```rust
// REMOVE these lines:
pub webviews: std::collections::HashMap<usize, wry::WebView>,
pub window_handle: Option<raw_window_handle::RawWindowHandle>,
```

Expected: Compilation will fail (these fields are referenced in Default::default())

- [ ] **Step 2: Add webview_widgets field**

At the same location (line 166), add:

```rust
// WebView widgets (stored separately from BrowserState metadata)
pub webview_widgets: std::collections::HashMap<egui::Id, egui_webview::EguiWebView>,
```

Expected: Compilation still fails (need to initialize in Default)

- [ ] **Step 3: Find Default::default() initialization**

```bash
cd mobile
grep -n "webviews:" src/dure_sijang_app.rs
```

Expected: Shows line number where old fields are initialized

- [ ] **Step 4: Update Default::default() initialization**

In `mobile/src/dure_sijang_app.rs`, replace the old initializations with:

```rust
webview_widgets: std::collections::HashMap::new(),
```

Remove any lines initializing `webviews` or `window_handle`.

Expected: Code compiles successfully

- [ ] **Step 5: Verify compilation**

```bash
cd mobile
cargo check --message-format=short
```

Expected: No errors (warnings about unused field are OK for now)

- [ ] **Step 6: Commit**

```bash
git add mobile/src/dure_sijang_app_stt.rs mobile/src/dure_sijang_app.rs
git commit -m "refactor: replace wry HashMap with egui_webview HashMap

- Remove unused webviews: HashMap<usize, wry::WebView>
- Remove window_handle: Option<RawWindowHandle>
- Add webview_widgets: HashMap<Id, EguiWebView>
- Initialize as empty in Default::default()

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 2: Fix Sidebar Toggle with Conditional Rendering

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1183-1218` (wrap SidePanel in conditional)

**Interfaces:**
- Consumes: `self.browser_state.sidebar_open: bool` (already exists)
- Produces: Working sidebar collapse/expand behavior

- [ ] **Step 1: Locate sidebar rendering code**

```bash
cd mobile
grep -n "SidePanel::left.*browser_sidebar" src/dure_sijang_app.rs
```

Expected: Shows line ~1183

- [ ] **Step 2: Read current sidebar code**

Read `mobile/src/dure_sijang_app.rs` lines 1183-1218 to see the full SidePanel block.

Expected: Starts with `SidePanel::left("browser_sidebar").resizable(true)...`

- [ ] **Step 3: Wrap SidePanel in conditional**

Edit `mobile/src/dure_sijang_app.rs` at line 1183. Wrap the ENTIRE SidePanel block in:

```rust
// 1. Left Sidebar Panel (collapsible, resizable)
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

            let mut bookmark_to_navigate: Option<(String, String)> = None;
            let mut bookmark_to_delete: Option<usize> = None;

            ScrollArea::vertical().show(ui, |ui| {
                for (idx, bookmark) in self.browser_state.bookmarks.iter().enumerate() {
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
                if let Some(active_idx) = self.browser_state.active_tab_index {
                    self.browser_state.update_tab_url(active_idx, &url, Some(&title));
                }
            }
            if let Some(idx) = bookmark_to_delete {
                let _ = self.browser_state.delete_bookmark(idx);
            }
        });
}
```

Key change: `if self.browser_state.sidebar_open {` before the SidePanel, and closing `}` after it.

- [ ] **Step 4: Verify compilation**

```bash
cd mobile
cargo check --message-format=short
```

Expected: No errors

- [ ] **Step 5: Test sidebar toggle (manual)**

```bash
cd mobile
cargo run
```

1. Create a tab (click "+ New Tab")
2. Click sidebar toggle button (should show "◀◀")
3. Verify sidebar disappears
4. Click toggle again (should show "▶▶")
5. Verify sidebar reappears

Expected: Sidebar collapses/expands, content area resizes

- [ ] **Step 6: Commit**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "fix: sidebar toggle now collapses/expands panel

Wrap SidePanel in conditional based on sidebar_open flag.
When false, sidebar hidden and content uses full width.

Fixes broken sidebar toggle button behavior.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 3: Create EguiWebView Widgets in add_browser_tab()

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1162-1165` (add webview creation)

**Interfaces:**
- Consumes: `tab_id: egui::Id` from `browser_state.add_tab()`
- Produces: `EguiWebView` widget stored in `self.webview_widgets[tab_id]`

- [ ] **Step 1: Locate add_browser_tab method**

```bash
cd mobile
grep -n "pub fn add_browser_tab" src/dure_sijang_app.rs
```

Expected: Shows line ~1162

- [ ] **Step 2: Read current implementation**

Read `mobile/src/dure_sijang_app.rs` lines 1162-1165.

Expected: Only creates tab in browser_state, no webview creation

- [ ] **Step 3: Add webview creation**

Edit `mobile/src/dure_sijang_app.rs` at line 1162. Replace the entire function:

```rust
pub fn add_browser_tab(&mut self, ctx: &egui::Context, frame: &eframe::Frame, url: &str) {
    // Create tab metadata
    let tab_id = self.browser_state.add_tab(url, url);
    
    // Create EguiWebView widget
    use egui_webview::EguiWebView;
    let view = EguiWebView::new(ctx, tab_id, frame, |builder| {
        builder.with_url(url)
    });
    
    // Store webview in HashMap
    self.webview_widgets.insert(tab_id, view);
    log::info!("Added browser tab {:?} with URL: {}", tab_id, url);
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd mobile
cargo check --message-format=short
```

Expected: No errors

- [ ] **Step 5: Test tab creation (manual)**

```bash
cd mobile
cargo run
```

1. Click "+ New Tab" button
2. Check terminal logs for "Added browser tab" message
3. Verify tab appears in tab bar (though webview won't render yet)

Expected: Tab created, log message shows tab_id and URL

- [ ] **Step 6: Commit**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "feat: create EguiWebView widget when adding browser tab

Store webview in webview_widgets HashMap keyed by tab ID.
Uses egui_webview builder pattern with initial URL.

Tab metadata and webview widget now created together.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 4: Render EguiWebView Widgets in Browser Content Area

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1336-1348` (replace TODO with rendering loop)

**Interfaces:**
- Consumes: `self.webview_widgets: HashMap<Id, EguiWebView>`, `self.browser_state.tabs`, `self.browser_state.active_tab_index`
- Produces: Rendered webviews, URL bar updates via `WebViewEvent::Loaded`

- [ ] **Step 1: Locate TODO placeholder**

```bash
cd mobile
grep -n "TODO.*egui_webview" src/dure_sijang_app.rs
```

Expected: Shows line ~1337

- [ ] **Step 2: Read current placeholder code**

Read `mobile/src/dure_sijang_app.rs` lines 1336-1348.

Expected: Shows "WebView content will render here" placeholder

- [ ] **Step 3: Replace TODO with webview rendering loop**

Edit `mobile/src/dure_sijang_app.rs` at line 1336. Replace the placeholder section with:

```rust
                ui.separator();

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

- [ ] **Step 4: Verify compilation**

```bash
cd mobile
cargo check --message-format=short
```

Expected: No errors

- [ ] **Step 5: Test webview rendering (manual)**

```bash
cd mobile
cargo run
```

1. Create new tab (click "+ New Tab")
2. Verify webview shows default page (https://dure.app)
3. Enter different URL in address bar, click "Go"
4. Verify page loads in webview
5. Verify URL bar updates when page loads

Expected: Webview displays pages, URL bar syncs with loaded page

- [ ] **Step 6: Test tab switching (manual)**

1. Create 3 tabs with different URLs
2. Click between tabs
3. Verify correct webview shown for each tab

Expected: Each tab shows its own webview content

- [ ] **Step 7: Commit**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "feat: render EguiWebView widgets in browser content area

Replace TODO placeholder with rendering loop:
- All tabs rendered (webview lifecycle requirement)
- Only active tab visible (zero size for inactive)
- WebViewEvent::Loaded updates URL bar
- push_id prevents egui ID conflicts

Webviews now display loaded pages.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Clean Up Webview on Tab Closure

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1171-1174` (add webview cleanup)

**Interfaces:**
- Consumes: `tab_id: egui::Id` from `self.browser_state.tabs[idx].id`
- Produces: Webview removed from HashMap, memory freed via Drop

- [ ] **Step 1: Locate close_browser_tab method**

```bash
cd mobile
grep -n "pub fn close_browser_tab" src/dure_sijang_app.rs
```

Expected: Shows line ~1171

- [ ] **Step 2: Read current implementation**

Read `mobile/src/dure_sijang_app.rs` lines 1171-1174.

Expected: Only closes tab in browser_state, no webview cleanup

- [ ] **Step 3: Add webview cleanup**

Edit `mobile/src/dure_sijang_app.rs` at line 1171. Replace the entire function:

```rust
pub fn close_browser_tab(&mut self, idx: usize) {
    if idx >= self.browser_state.tabs.len() {
        return;
    }
    
    // Remove webview widget before removing tab metadata
    let tab_id = self.browser_state.tabs[idx].id;
    if let Some(_view) = self.webview_widgets.remove(&tab_id) {
        log::info!("Destroyed webview for tab {:?}", tab_id);
        // Drop will handle cleanup
    }
    
    // Remove tab metadata
    self.browser_state.close_tab(idx);
    log::info!("Closed browser tab at index {}", idx);
}
```

- [ ] **Step 4: Verify compilation**

```bash
cd mobile
cargo check --message-format=short
```

Expected: No errors

- [ ] **Step 5: Test tab closure (manual)**

```bash
cd mobile
cargo run
```

1. Create 3 tabs
2. Close middle tab (click × button)
3. Verify no crash
4. Check terminal logs for "Destroyed webview" message
5. Verify active tab selection updates correctly
6. Close all tabs
7. Verify "No tabs open" state shown

Expected: Tabs close cleanly, no memory leaks, correct state updates

- [ ] **Step 6: Commit**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "feat: clean up webview when closing browser tab

Remove EguiWebView from HashMap before closing tab.
Drop trait handles native webview destruction.

Prevents memory leaks when tabs are closed.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: Fix Navigation Methods to Use webview_widgets

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1098-1150` (update navigate_back, navigate_forward, navigate_reload)
- Modify: `mobile/src/dure_sijang_app.rs:1231-1244` (fix toolbar button calls)

**Interfaces:**
- Consumes: `idx: usize` (tab index), `self.webview_widgets: HashMap<Id, EguiWebView>`
- Produces: Fixed navigation methods using EguiWebView API (back(), forward(), reload())

- [ ] **Step 1: Locate navigation methods**

```bash
cd mobile
grep -n "pub fn navigate_back\|pub fn navigate_forward\|pub fn navigate_reload" src/dure_sijang_app.rs
```

Expected: Shows lines ~1098, ~1119, ~1140

- [ ] **Step 2: Update navigate_back method**

Edit `mobile/src/dure_sijang_app.rs` at line 1098. Replace the entire function:

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
```

- [ ] **Step 3: Update navigate_forward method**

Edit `mobile/src/dure_sijang_app.rs` at line ~1119. Replace the entire function:

```rust
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
```

- [ ] **Step 4: Update navigate_reload method**

Edit `mobile/src/dure_sijang_app.rs` at line ~1140. Replace the entire function:

```rust
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

- [ ] **Step 5: Find toolbar button calls**

```bash
cd mobile
grep -n "TODO.*back navigation\|TODO.*forward navigation" src/dure_sijang_app.rs
```

Expected: Shows lines ~1233, ~1241

- [ ] **Step 6: Fix back button call**

Edit `mobile/src/dure_sijang_app.rs` at line ~1231. Replace:

```rust
// Back button
if ui.button("◀").clicked() {
    if let Some(idx) = self.browser_state.active_tab_index {
        // TODO: Implement back navigation via egui_webview
        log::info!("Back navigation for tab {}", idx);
    }
}
```

With:

```rust
// Back button
if ui.button("◀").clicked() {
    if let Some(idx) = self.browser_state.active_tab_index {
        if let Err(e) = self.navigate_back(idx) {
            log::warn!("Back navigation failed: {}", e);
        }
    }
}
```

- [ ] **Step 7: Fix forward button call**

Edit `mobile/src/dure_sijang_app.rs` at line ~1239. Replace:

```rust
// Forward button
if ui.button("▶").clicked() {
    if let Some(idx) = self.browser_state.active_tab_index {
        // TODO: Implement forward navigation via egui_webview
        log::info!("Forward navigation for tab {}", idx);
    }
}
```

With:

```rust
// Forward button
if ui.button("▶").clicked() {
    if let Some(idx) = self.browser_state.active_tab_index {
        if let Err(e) = self.navigate_forward(idx) {
            log::warn!("Forward navigation failed: {}", e);
        }
    }
}
```

- [ ] **Step 8: Verify compilation**

```bash
cd mobile
cargo check --message-format=short
```

Expected: No errors

- [ ] **Step 9: Test navigation (manual)**

```bash
cd mobile
cargo run
```

1. Create tab, navigate to page with links
2. Click link (webview navigates to new page)
3. Click back button (◀)
4. Verify returns to previous page
5. Click forward button (▶)
6. Verify returns to linked page
7. Verify URL bar updates

Expected: Back/forward navigation works, URL bar syncs

- [ ] **Step 10: Commit**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "fix: navigation methods now use webview_widgets HashMap

Update navigate_back, navigate_forward, navigate_reload:
- Accept idx: usize instead of tab_id
- Look up EguiWebView from webview_widgets
- Call view.back(), view.forward(), view.reload()

Connect toolbar buttons to navigation methods.
Remove TODO comments.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: Remove Obsolete wry WebView Methods

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1024-1089` (delete create_webview, destroy_webview)

**Interfaces:**
- Consumes: None (cleanup task)
- Produces: Removed dead code, cleaner codebase

- [ ] **Step 1: Locate obsolete methods**

```bash
cd mobile
grep -n "pub fn create_webview\|pub fn destroy_webview" src/dure_sijang_app.rs
```

Expected: Shows lines ~1024 and ~1080

- [ ] **Step 2: Verify methods are unused**

```bash
cd mobile
grep -n "create_webview\|destroy_webview" src/dure_sijang_app.rs | grep -v "pub fn"
```

Expected: No results (methods not called anywhere)

- [ ] **Step 3: Delete create_webview method**

Delete `mobile/src/dure_sijang_app.rs` lines 1024-1075 (entire `pub fn create_webview()` method).

This includes:
- Doc comment ("/// Create a new webview...")
- Function signature
- GTK TODO block
- Android block
- Default platform block
- Return statement

- [ ] **Step 4: Delete destroy_webview method**

Delete `mobile/src/dure_sijang_app.rs` lines ~1077-1089 (entire `pub fn destroy_webview()` method).

This includes:
- Doc comment ("/// Destroy the webview...")
- Function signature
- HashMap remove logic
- Log statements

- [ ] **Step 5: Delete obsolete comment section**

Delete the comment line that says:
```rust
// ===== WebView Management Methods (NEW - Task 5) =====
```

This was above the deleted methods.

- [ ] **Step 6: Verify compilation**

```bash
cd mobile
cargo check --message-format=short
```

Expected: No errors

- [ ] **Step 7: Run full test suite**

```bash
cd mobile
cargo test
```

Expected: All tests pass (or show same failures as before if any)

- [ ] **Step 8: Commit**

```bash
git add mobile/src/dure_sijang_app.rs
git commit -m "refactor: remove obsolete wry webview methods

Delete create_webview() and destroy_webview() methods.
These were for raw wry::WebView integration which is
replaced by egui_webview.

EguiWebView::new() and HashMap::remove() handle
webview lifecycle directly now.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Final Verification

After completing all tasks, run this final verification checklist:

### Compilation Check

```bash
cd mobile
cargo clean
cargo build --release
```

Expected: Clean build with no errors, no clippy warnings

### Manual Test Suite

Run all 6 manual tests from the spec:

1. **Sidebar Toggle**: ◀◀ / ▶▶ button collapses/expands sidebar
2. **WebView Display**: URL bar + Go button loads pages
3. **Tab Switching**: Click between 3 tabs, correct content shown
4. **Navigation**: Back (◀) / Forward (▶) buttons work
5. **Tab Closure**: Close middle tab, close all tabs
6. **Bookmarks**: Add bookmark, collapse sidebar, expand, click bookmark

Expected: All tests pass

### Code Quality Check

```bash
cd mobile
cargo clippy --all-targets -- -D warnings
cargo fmt -- --check
```

Expected: No clippy lints, code properly formatted

### Platform Test (OpenBSD)

```bash
cd mobile
cargo run
```

Verify webview renders using GTK/WebKit backend on OpenBSD.

Expected: Webview displays pages (no "GTK not supported" error)

---

## Success Criteria

- ✅ Sidebar toggle works (conditional rendering)
- ✅ Webviews render loaded pages
- ✅ URL bar updates when pages load
- ✅ Tab switching shows correct webview
- ✅ Back/forward/reload buttons work
- ✅ Tab closure cleans up webviews
- ✅ Bookmarks navigate active tab
- ✅ No compilation errors or warnings
- ✅ No crashes during testing
- ✅ Works on OpenBSD with GTK backend

---

## Troubleshooting

### Issue: "WebView not yet supported on GTK platforms"

**Cause**: Old wry code still present  
**Fix**: Verify Task 7 completed (obsolete methods deleted)

### Issue: Webview shows blank/empty

**Cause**: URL not loading or webview not created  
**Fix**: Check Task 3 (webview creation) and Task 4 (rendering loop)

### Issue: Sidebar toggle doesn't work

**Cause**: Conditional wrapper missing  
**Fix**: Verify Task 2 completed (if statement added)

### Issue: Tabs don't close cleanly

**Cause**: Webview not removed from HashMap  
**Fix**: Check Task 5 (cleanup in close_browser_tab)

### Issue: Back/forward buttons don't work

**Cause**: Navigation methods not updated or not connected  
**Fix**: Verify Task 6 completed (both methods and button calls)
