# Plan: WebView Cleanup Fixes

**Complexity**: Medium
**Branch**: feature/webview-cleanup-fixes  
**Date**: 2026-11-05

## Summary

Fix two webview lifecycle issues:
1. **Last tab closed**: Webview stays visible instead of being destroyed/hidden
2. **Sidebar toggle**: Webview position doesn't update when sidebar opens/closes

## Patterns to Mirror

| Category | Source | Pattern |
|----------|--------|---------|
| Webview cleanup | `dure_sijang_app.rs:1141-1145` | Remove from HashMap and let Drop handle destruction |
| Logging | `dure_sijang_app.rs:1134` | `log::error!()` for state changes |
| Platform conditionals | `dure_sijang_app.rs:1139` | `#[cfg(not(target_os = "android"))]` |
| HashMap operations | `dure_sijang_app.rs:1141-1142` | `.remove()` returns Option, check before cleanup |
| Empty state handling | `dure_sijang_app.rs:1730-1746` | Check `tabs.is_empty()` for empty state UI |

## Root Cause Analysis

### Problem 1: Last Tab Closed - Webview Stays Visible

**Location**: `mobile/src/dure_sijang_app.rs:1128-1177` (`close_browser_tab`)

**Current Flow**:
```rust
// Line 1141-1145
if let Some(_view) = self.webview_widgets.remove(&tab_id) {
    self.webview_last_access.remove(&tab_id);
    log::error!("Destroyed webview for tab {:?}", tab_id);
}

// Line 1166
} else {
    None  // active_tab_index becomes None when last tab closed
};
```

**Issue**:
- Closes specific tab's webview (line 1141)
- Sets `active_tab_index = None` when no tabs left (line 1166)
- But doesn't clear OTHER cached webviews in `webview_widgets` HashMap
- Rendering code at line 1761 checks `active_tab_index.is_some()` but doesn't explicitly hide webviews when None

**Evidence**:
- `webview_widgets` can have up to `MAX_CACHED_WEBVIEWS` (10) entries (line 1768)
- Only the closed tab's webview is removed, others stay in cache
- Empty state UI at line 1730-1746 shows "No tabs open" but doesn't hide webviews

### Problem 2: Sidebar Toggle - Webview Position Doesn't Update

**Location**: `mobile/src/dure_sijang_app.rs:1369-1799` (`render_browser_ui`)

**Current Flow**:
```rust
// Line 1389-1456: Sidebar rendered
if self.browser_state.sidebar_open {
    SidePanel::left("browser_sidebar")
        .resizable(is_desktop)
        .default_width(200.0)
        ...
}

// Line 1571: Sidebar toggle
if items_button.clicked() {
    self.browser_state.sidebar_open = !self.browser_state.sidebar_open;
}

// Line 1729-1792: Webview rendered in CentralPanel
CentralPanel::default().show_inside(ui, |ui| {
    ...
    let size = ui.available_size();  // Size calculated AFTER sidebar
    view.ui(ui, size);  // Webview rendered with size
```

**Issue**:
- Sidebar toggle changes `sidebar_open` (line 1571)
- Next frame, `SidePanel` is hidden/shown (line 1389)
- `CentralPanel` available space changes
- `ui.available_size()` returns new size (line 1788)
- But `EguiWebView::ui()` in `egui_webview/mod.rs:366-465`:
  - Only updates bounds when `bounds_changed` (line 447)
  - Bounds comparison uses `last_bounds` cached value (line 428-445)
  - If bounds comparison fails to detect change, native window doesn't reposition

**Root Cause**:
- The `bounds_changed` check in `egui_webview/mod.rs:428-445` might not detect position changes when only X offset changes (sidebar width)
- OR the bounds update happens but native window repositioning lags behind egui layout

## Files to Change

| File | Action | Why |
|------|--------|-----|
| `mobile/src/dure_sijang_app.rs` | UPDATE | Clear all webviews when last tab closed |
| `mobile/src/egui_webview/mod.rs` | UPDATE | Force bounds recalculation on layout changes |

## Implementation Tasks

### Task 1: Clear All Webviews When Last Tab Closed

**File**: `mobile/src/dure_sijang_app.rs:1128-1177`

**Problem**: After closing last tab, cached webviews still exist in HashMap and might be visible

**Change 1** - Add cleanup after `close_tab()`:
```rust
// After line 1157: self.browser_state.close_tab(idx);
// Add:

// If no tabs left, clear all cached webviews (desktop-only)
#[cfg(not(target_os = "android"))]
if self.browser_state.tabs.is_empty() {
    log::error!("Last tab closed - clearing all {} cached webviews", 
               self.webview_widgets.len());
    self.webview_widgets.clear();
    self.webview_last_access.clear();
}
```

**Rationale**:
- When `tabs.is_empty()` is true, no webview should be visible
- Clear entire HashMap to prevent orphaned webviews
- Drop trait will handle native window destruction for each webview
- Mirrors the cleanup pattern at line 1141-1145

**Mirror**: HashMap cleanup pattern (line 1141-1145), logging pattern (line 1134)

**Validate**:
```bash
cargo build
cargo run
# 1. Create a tab
# 2. Close the tab (should be last tab)
# 3. Verify no webview visible, "No tabs open" message shown
```

### Task 2: Force Webview Bounds Update on Sidebar Toggle

**File**: `mobile/src/egui_webview/mod.rs:366-465` (`EguiWebView::ui()`)

**Problem**: Webview doesn't reposition when sidebar toggles because bounds comparison might miss X-offset-only changes

**Change 1** - Always update bounds on first frames after change:
```rust
// After line 445: } else {
//     true // First time, always update
// };

// Replace with:
} else {
    true // First time, always update
};

// ALWAYS update bounds if size OR position changed
// This handles sidebar toggle where X offset changes but size might not
if bounds_changed {
    self.view.set_bounds(new_bounds).ok();
    self.last_bounds = Some(new_bounds);
    log::trace!("Updated webview bounds: pos=({},{}) size={}x{}", 
               physical_rect.min.x as i32, physical_rect.min.y as i32,
               physical_rect.width() as u32, physical_rect.height() as u32);
}
```

**Alternative Approach** (if above doesn't work):
Add a "force update" flag when sidebar toggles:

In `dure_sijang_app.rs` after line 1571:
```rust
if items_button.clicked() {
    self.browser_state.sidebar_open = !self.browser_state.sidebar_open;
    // Mark that layout changed - force webview bounds update next frame
    self.webview_force_update = true;
}
```

Then in `egui_webview/mod.rs`, pass force flag and bypass cache check.

**Rationale**:
- Current bounds comparison at line 428-445 might not detect X-offset-only changes
- Sidebar toggle changes X position (not just size) of CentralPanel
- Adding logging helps debug future layout issues
- Force update ensures native window repositions

**Mirror**: Bounds update pattern (line 447-450), logging with trace level (throughout module)

**Validate**:
```bash
cargo build
cargo run
# 1. Open a tab
# 2. Toggle sidebar closed (hamburger button)
# 3. Verify webview expands to fill space
# 4. Toggle sidebar open
# 5. Verify webview shrinks and repositions correctly
```

### Task 3: Add Diagnostic Logging

**Files**: Both `dure_sijang_app.rs` and `egui_webview/mod.rs`

**Add logging** to trace webview lifecycle:

In `dure_sijang_app.rs` after webview clear (from Task 1):
```rust
log::error!("Last tab closed - cleared all webviews (was: {})", count);
```

In `egui_webview/mod.rs` at bounds update (from Task 2):
```rust
log::trace!("Webview bounds updated: {:?} -> {:?}", self.last_bounds, new_bounds);
```

**Validate**: Check logs show expected messages during tab close and sidebar toggle

## Branch Strategy

```bash
# Create feature branch
git checkout main
git pull
git checkout -b feature/webview-cleanup-fixes

# Implement fixes
# ... make changes ...

# Commit each fix separately
git add mobile/src/dure_sijang_app.rs
git commit -m "fix: clear all webviews when last tab closed

When closing the last browser tab, clear entire webview_widgets HashMap
to ensure no orphaned webviews remain visible. Previously only the closed
tab's webview was removed, leaving other cached webviews in memory.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

git add mobile/src/egui_webview/mod.rs
git commit -m "fix: ensure webview repositions on sidebar toggle

Force webview bounds update when sidebar opens/closes. The previous
bounds comparison didn't reliably detect X-offset-only changes when
sidebar width changed, causing webview to stay in wrong position.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"

# Test
cargo build
cargo run

# Push and create PR when ready
git push -u origin feature/webview-cleanup-fixes
```

## Validation Commands

```bash
# Build
cd mobile
cargo clean
cargo build

# Test scenario 1: Last tab close
cargo run
# UI Actions:
# 1. Click "+ New Tab" to create a tab
# 2. Click "X" on the tab to close it
# Expected: Webview disappears, "No tabs open" message visible
# Previous: Webview stayed visible as ghost window

# Test scenario 2: Sidebar toggle
cargo run
# UI Actions:
# 1. Create a tab (webview visible)
# 2. Click hamburger menu to close sidebar
# Expected: Webview expands left to fill sidebar space
# Previous: Webview stayed in same position, leaving gap
# 3. Click hamburger menu to open sidebar
# Expected: Webview shrinks and moves right
# Previous: Webview stayed expanded, overlapping sidebar

# Check logs
cargo run 2>&1 | grep -E "webview|bounds|tab closed"
```

## Expected Behavior Changes

### Before (Current):

**Last Tab Close**:
```
User: Creates tab → Webview appears
User: Closes tab → "No tabs open" message shows
Problem: Webview window still visible as ghost (from cache)
HashMap: {tab_0: WebView, tab_1: WebView} → {tab_1: WebView}  ❌
```

**Sidebar Toggle**:
```
Sidebar Open (200px): [Sidebar][Webview at X=200]
User: Closes sidebar
Sidebar Closed: [____][Webview at X=200]  ❌ (gap left)
Expected: [Webview at X=0]
```

### After (Fixed):

**Last Tab Close**:
```
User: Creates tab → Webview appears
User: Closes tab → "No tabs open" message shows
Fixed: All webviews destroyed, window cleaned up
HashMap: {tab_0: WebView} → {}  ✅
Log: "Last tab closed - cleared all 1 cached webviews"
```

**Sidebar Toggle**:
```
Sidebar Open (200px): [Sidebar][Webview at X=200]
User: Closes sidebar
Sidebar Closed: [Webview at X=0]  ✅ (fills space)
User: Opens sidebar
Sidebar Open: [Sidebar][Webview at X=200]  ✅ (repositions)
Log: "Updated webview bounds: pos=(0,64) size=800x600"
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Clearing HashMap causes use-after-free | LOW | High | HashMap owns Arc<WebView>, clear() drops all Arcs, Drop trait handles cleanup |
| Bounds update causes flicker | MEDIUM | Low | egui repaints are fast, native window repositioning should be smooth |
| Breaking Android webview | LOW | High | All changes are `#[cfg(not(target_os = "android"))]` desktop-only |
| Over-eager bounds update impacts performance | LOW | Medium | Bounds only update when position/size actually changed, trace logging for monitoring |

## Acceptance Criteria

- [ ] Task 1 complete: Webviews cleared when last tab closed
- [ ] Task 2 complete: Webview repositions on sidebar toggle
- [ ] Task 3 complete: Diagnostic logging added
- [ ] `cargo build` succeeds
- [ ] Manual test 1: Last tab close hides webview
- [ ] Manual test 2: Sidebar toggle repositions webview correctly
- [ ] Logs show expected cleanup messages
- [ ] No regression on Android (conditional compilation)
- [ ] Feature branch created and commits follow message format
- [ ] Ready for PR when all tests pass

## Notes

- **ponytail**: Minimal changes - HashMap clear, bounds update flag
- **Scope**: Desktop-only fixes (Android uses different lifecycle)
- **Safety**: All changes wrapped in `#[cfg(not(target_os = "android"))]`
- **Testing**: Manual UI testing required - webview positioning can't be unit tested
