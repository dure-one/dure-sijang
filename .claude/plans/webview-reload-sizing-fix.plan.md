# Plan: WebView Reload Sizing Fix

**Complexity**: Medium
**Branch**: feature/webview-reload-sizing (new)
**Base**: feature/webview-cleanup-fixes (current)
**Date**: 2026-11-05

## Summary

Fix webview appearing as small square after reload (e.g., when settings dialog closes). The fix involves cherry-picking commit b8bcae1 which defers webview visibility until first bounds are set, preventing the 0x0 size flash.

## Problem Analysis

### Root Cause

**Current Behavior** (without b8bcae1):
```
Timeline:
1. WebView created → set_visible(true) with 0x0 bounds
2. Small square appears on screen (0x0 or minimal size)
3. First frame renders → set_bounds() called with correct size
4. WebView stays small (bounds set but doesn't resize properly)
5. User toggles sidebar → forces bounds recalculation → fixes size
```

**Fixed Behavior** (with b8bcae1):
```
Timeline:
1. WebView created → set_visible(false) → hidden
2. First frame renders → set_bounds() called with correct size
3. After bounds set → set_visible(true) → shows at correct size
4. No small square, no sidebar toggle needed
```

### Evidence

**Commit b8bcae1** (on separate branch):
```
fix: defer WebView visibility until first bounds set

Prevents small square flash on WebView creation by:
- Adding is_visible flag to EguiWebView struct
- Starting WebView hidden (set_visible(false))
- Making visible after first set_bounds() in ui()
```

**Changes in b8bcae1**:
1. Add `is_visible: bool` field to `EguiWebView` struct
2. Change `new()` to call `set_visible(false)` instead of `set_visible(true)`
3. In `ui()` method, call `set_visible(true)` AFTER first `set_bounds()`

### Why Sidebar Toggle Fixes It

From `dure_sijang_app.rs:1571`:
```rust
if items_button.clicked() {
    self.browser_state.sidebar_open = !self.browser_state.sidebar_open;
}
```

When sidebar toggles:
1. `SidePanel` width changes (200px ↔ 0px)
2. `CentralPanel` available space changes
3. Next frame: `ui.available_size()` returns new size
4. `bounds_changed = true` (size difference detected)
5. `set_bounds(new_bounds)` called with CORRECT full size
6. WebView finally resizes to proper dimensions

## Patterns to Mirror

| Category | Source | Pattern |
|----------|--------|---------|
| Visibility defer | `b8bcae1:egui_webview/mod.rs:182-184` | Start hidden, show after first bounds |
| Flag tracking | `b8bcae1:egui_webview/mod.rs:47` | Add `is_visible: bool` to struct |
| Bounds-triggered visibility | `b8bcae1:egui_webview/mod.rs:455-461` | Check `!is_visible` in bounds update block |
| Android handling | `b8bcae1:egui_webview/mod.rs:468-474` | Separate Android visibility logic |

## Files to Change

| File | Action | Why |
|------|--------|-----|
| `.git` | CHERRY-PICK | Apply commit b8bcae1 to current branch |
| `mobile/src/egui_webview/mod.rs` | UPDATE (via cherry-pick) | Add is_visible flag and defer visibility |

## Implementation Tasks

### Task 0: Verify Current Branch State

**Check**: All changes committed on `feature/webview-cleanup-fixes`

**Validate**:
```bash
git status
# Should show: nothing to commit, working tree clean (except .claude/)
```

### Task 1: Create New Feature Branch

**Action**: Branch from current `feature/webview-cleanup-fixes` to preserve cleanup fixes

```bash
git checkout -b feature/webview-reload-sizing
```

**Rationale**: Keep cleanup fixes and add reload fix on top

### Task 2: Cherry-Pick Visibility Defer Fix

**Action**: Apply commit b8bcae1 to current branch

```bash
git cherry-pick b8bcae1
```

**Expected Changes** (from b8bcae1):

**File**: `mobile/src/egui_webview/mod.rs`

**Change 1** - Add `is_visible` field to struct (line ~49):
```rust
#[cfg(not(target_os = "android"))]
pub struct EguiWebView {
    pub view: Arc<wry::WebView>,
    id: Id,
    events: Arc<Mutex<VecDeque<WebViewEvent>>>,
    #[allow(dead_code)]
    context: Context,
    last_bounds: Option<wry::Rect>,
    is_visible: bool,  // NEW: Track visibility state
}

#[cfg(target_os = "android")]
pub struct EguiWebView {
    pub view: Arc<android_webview::AndroidWebView>,
    id: Id,
    events: Arc<Mutex<VecDeque<WebViewEvent>>>,
    #[allow(dead_code)]
    context: Context,
    is_visible: bool,  // NEW: Track visibility state
}
```

**Change 2** - Start hidden in `new_with_container()` (line ~184):
```rust
// Start hidden - will be shown after first bounds set in ui()
web_view.set_visible(false).ok();
```

**Change 3** - Start hidden in `new()` (line ~266):
```rust
// Start hidden - will be shown after first bounds set in ui()
web_view.set_visible(false).ok();
```

**Change 4** - Initialize `is_visible: false` (lines ~199, ~282, ~326):
```rust
Self {
    events,
    view: web_view,
    id,
    context: ctx.clone(),
    last_bounds: None,
    is_visible: false,  // NEW
}
```

**Change 5** - Show after first bounds set (desktop, line ~458):
```rust
if bounds_changed {
    self.view.set_bounds(new_bounds).ok();
    self.last_bounds = Some(new_bounds);
    log::trace!("Updated webview bounds: pos=({},{}) size={}x{}", ...);
    
    // NEW: Make visible after first bounds set (prevents small square flash)
    if !self.is_visible {
        self.view.set_visible(true).ok();
        self.is_visible = true;
        log::error!("WebView made visible after first bounds set");
    }
}
```

**Change 6** - Show on first ui() (Android, line ~471):
```rust
#[cfg(target_os = "android")]
{
    // Android WebView bounds are managed by the view hierarchy
    // Make visible on first ui() call
    if !self.is_visible {
        self.view.set_visible(true).ok();
        self.is_visible = true;
        log::error!("Android WebView made visible on first ui()");
    }
}
```

**Mirror**: 
- Flag pattern from existing `last_bounds: Option<Rect>`
- Logging pattern from our previous commit (4bdf9cb)
- Platform conditional pattern `#[cfg(not(target_os = "android"))]`

**Validate**:
```bash
# Cherry-pick should succeed
git cherry-pick b8bcae1

# Check for conflicts
git status

# If conflicts, resolve and continue:
# git add <conflicted-files>
# git cherry-pick --continue
```

### Task 3: Resolve Conflicts (if any)

**Likely Conflict**: Line ~450 where we added logging in commit 4bdf9cb

**Our version** (4bdf9cb):
```rust
if bounds_changed {
    self.view.set_bounds(new_bounds).ok();
    self.last_bounds = Some(new_bounds);
    log::trace!("Updated webview bounds: pos=({},{}) size={}x{}", ...);
}
```

**Incoming version** (b8bcae1):
```rust
if bounds_changed {
    self.view.set_bounds(new_bounds).ok();
    self.last_bounds = Some(new_bounds);
    // Make visible after first bounds set (prevents small square flash)
    if !self.is_visible {
        self.view.set_visible(true).ok();
        self.is_visible = true;
    }
}
```

**Merged version**:
```rust
if bounds_changed {
    self.view.set_bounds(new_bounds).ok();
    self.last_bounds = Some(new_bounds);
    log::trace!("Updated webview bounds: pos=({},{}) size={}x{}", 
               physical_rect.min.x as i32, physical_rect.min.y as i32,
               physical_rect.width() as u32, physical_rect.height() as u32);
    
    // Make visible after first bounds set (prevents small square flash)
    if !self.is_visible {
        self.view.set_visible(true).ok();
        self.is_visible = true;
        log::error!("WebView made visible after first bounds set");
    }
}
```

**Validate**: Ensure both logging and visibility logic are present

### Task 4: Build and Test

**Build**:
```bash
cd mobile
cargo build
```

**Expected**: Build succeeds with no errors

**Validate**: No compilation errors

## Branch Strategy

```bash
# 0. Verify current state
git status  # Should be clean on feature/webview-cleanup-fixes

# 1. Create new branch
git checkout -b feature/webview-reload-sizing

# 2. Cherry-pick visibility fix
git cherry-pick b8bcae1

# 3. Resolve conflicts if needed
# (see Task 3 above)

# 4. Test
cargo build
cargo run

# 5. Commit message already from cherry-pick:
# "fix: defer WebView visibility until first bounds set"
```

## Validation Commands

```bash
# Build
cd mobile
cargo clean
cargo build

# Test scenario: Settings dialog close
cargo run
# UI Actions:
# 1. Create a browser tab
# 2. Open Settings (gear icon)
# 3. Change any setting (e.g., theme)
# 4. Close Settings dialog
# Expected: WebView appears at full size immediately
# Previous: WebView appeared as small square, needed sidebar toggle to fix

# Test scenario: New tab creation
cargo run
# UI Actions:
# 1. Click "+ New Tab"
# Expected: WebView appears at full size immediately
# Previous: WebView flashed as small square before resizing

# Check logs for visibility messages
RUST_LOG=error cargo run 2>&1 | grep "made visible"
# Should see: "WebView made visible after first bounds set"
```

## Expected Behavior Changes

### Before (Current - without b8bcae1):

**Settings Dialog Close**:
```
User: Opens Settings → WebView hidden (z-index issue)
User: Closes Settings → WebView recreated
Problem: WebView appears as small square (0x0 or minimal size)
Workaround: Toggle sidebar → forces bounds update → correct size

Timeline:
Frame 0: new() → set_visible(true) with 0x0 bounds
Frame 1: ui() → set_bounds(800x600) but already visible at wrong size
Frames 2+: Stuck at small size until layout changes
```

**New Tab Creation**:
```
User: Clicks "+ New Tab"
Problem: Brief flash of small square before full size
Timeline:
Frame 0: new() → set_visible(true) with 0x0 bounds → 📦 small square
Frame 1: ui() → set_bounds(800x600) → resize
```

### After (Fixed - with b8bcae1):

**Settings Dialog Close**:
```
User: Opens Settings → WebView hidden (z-index issue)
User: Closes Settings → WebView recreated
Fixed: WebView appears at correct size immediately

Timeline:
Frame 0: new() → set_visible(false) → hidden
Frame 1: ui() → set_bounds(800x600) → set_visible(true) → ✅ full size
No sidebar toggle needed!
```

**New Tab Creation**:
```
User: Clicks "+ New Tab"
Fixed: No flash, appears at full size

Timeline:
Frame 0: new() → set_visible(false) → hidden
Frame 1: ui() → set_bounds(800x600) → set_visible(true) → ✅ full size
```

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Cherry-pick conflicts with our logging commit | HIGH | Low | Manually merge both changes (Task 3) |
| is_visible flag causes double-visibility calls | LOW | Low | Flag prevents redundant calls |
| Android visibility timing different | MEDIUM | Medium | Separate Android path in b8bcae1 handles this |
| Breaking existing functionality | LOW | High | Cherry-pick is battle-tested (commit b8bcae1) |

## Acceptance Criteria

- [ ] Task 0: Current branch state verified (clean)
- [ ] Task 1: New branch created from feature/webview-cleanup-fixes
- [ ] Task 2: Cherry-pick b8bcae1 applied successfully
- [ ] Task 3: Conflicts resolved (if any)
- [ ] Task 4: Build succeeds
- [ ] Manual test 1: Settings dialog close shows full-size webview
- [ ] Manual test 2: New tab shows full-size webview without flash
- [ ] Manual test 3: No small square appears in any scenario
- [ ] Logs show "WebView made visible after first bounds set"
- [ ] No regression in previous fixes (cleanup, Drop fix)

## Commit History After Implementation

```
* <new> fix: defer WebView visibility until first bounds set (cherry-picked from b8bcae1)
* 4bdf9cb fix: add diagnostic logging for webview bounds updates
* be5cbed fix: clear all webviews when last tab closed
* 98926fc fix: remove unsafe set_visible call from EguiWebView Drop
* e771329 feat: vendor egui_webview and implement z-index control
```

## Notes

- **ponytail**: Zero new code - cherry-pick existing tested fix
- **Safety**: b8bcae1 already battle-tested on separate branch
- **Approach**: Cherry-pick > reimplementation (DRY principle)
- **Testing**: Manual UI testing required - small square issue can't be unit tested
- **Desktop-only**: Changes affect both desktop and Android (separate paths)
