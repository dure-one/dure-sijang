# Dure-Sijang App Refactor Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Remove 28 unused fields, 2 type definitions, and legacy comments from dure_sijang_app.rs and dure_sijang_app_stt.rs after Dure-Sijang to mycart browser migration.

**Architecture:** Deep clean refactoring - remove ADB/Shizuku device management fields (9), legacy renderer state machines (3), dashboard scroll offsets (7), package loading state (3), debloat performance fields (2), disclaimer/view state (4), and all legacy comment markers. Preserve active browser functionality, install/update system, and cross-platform support.

**Tech Stack:** Rust, egui, eframe, wry

## Global Constraints

- Rust edition 2021 (`rustfmt --edition 2021`)
- Cross-platform support: Android + Desktop (OpenBSD, Linux, Windows, macOS)
- Preserve: browser_ui, viewmodel, webviews, install/update system, extract_github_embedded_data()
- Remove: All ADB/Shizuku device management, legacy renderer state, dashboard counters, package loading
- Verify: `cargo check` and `cargo clippy` after each task
- No placeholders or TODOs in final code

---

## Task 1: Clean up dure_sijang_app_stt.rs Type Definitions and Imports

**Files:**
- Modify: `mobile/src/dure_sijang_app_stt.rs:1-257`

**Interfaces:**
- Consumes: Nothing (first task)
- Produces: Cleaned imports, `RendererStateMachine` and `AppView` removed

- [ ] **Step 1: Remove legacy import comments (lines 1-9)**

Navigate to `mobile/src/dure_sijang_app_stt.rs` and replace lines 1-9:

**Before:**
```rust
// LEGACY: Deleted for mycart browser migration
// use crate::adb::UserInfo;
#[cfg(not(target_os = "android"))]
use crate::install_stt::InstallStatus;
// LEGACY: Deleted modules from pre-August 2026 mycart browser migration
// use crate::tab_apps_control::TabAppsControl;
// use crate::tab_debloat::TabDebloat;
// use crate::tab_scan_control::TabScanControl;
// use crate::tab_usage_control::TabUsageControl;
```

**After:**
```rust
#[cfg(not(target_os = "android"))]
use crate::install_stt::InstallStatus;
```

Expected: Lines 1-9 reduced to 2 lines (import + cfg attribute)

- [ ] **Step 2: Remove RendererStateMachine struct definition (lines 19-23)**

Find and delete the `RendererStateMachine` struct:

```rust
/// State machine for renderer lifecycle management
#[derive(Default)]
pub struct RendererStateMachine {
    /// Whether the renderer is currently enabled
    pub is_enabled: bool,
}
```

Expected: Struct definition completely removed

- [ ] **Step 3: Remove AppView enum definition (lines 250-257)**

Find and delete the `AppView` enum at the end of the file:

```rust
pub enum AppView {
    Debloat,
    Scan,
    Apps,
    Usage,
    Browser,
}
```

Expected: Enum definition completely removed

- [ ] **Step 4: Verify syntax**

Run rustfmt and check for syntax errors:

```bash
cd /home/wj/work/dure-sijang/mobile
rustfmt --edition 2021 src/dure_sijang_app_stt.rs
cargo check --message-format=short 2>&1 | head -20
```

Expected: rustfmt succeeds, cargo check may show errors about missing fields (fixed in next task)

- [ ] **Step 5: Commit type definitions cleanup**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app_stt.rs
git commit -m "$(cat <<'EOF'
refactor(app): remove legacy type definitions and imports

Remove RendererStateMachine, AppView enum, and legacy import comments.
Part 1 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created with message

---

## Task 2: Remove 28 Unused Fields from DureSijangApp Struct

**Files:**
- Modify: `mobile/src/dure_sijang_app_stt.rs:106-249`

**Interfaces:**
- Consumes: Type definitions removed in Task 1
- Produces: DureSijangApp struct with 28 fewer fields

- [ ] **Step 1: Remove ADB/Shizuku device management fields**

In the `DureSijangApp` struct, find and delete these 9 fields:

```rust
pub adb_devices: Vec<String>,
pub selected_device: Option<String>,
pub current_device: Option<String>,

// LEGACY: Deleted for mycart browser migration
// pub adb_users: Vec<UserInfo>,
pub selected_user: Option<i32>, // None means "All Users"
pub current_user: Option<i32>,
```

And later in the struct:

```rust
// Shizuku state tracking (Android)
pub shizuku_init_done: bool,
pub shizuku_permission_requested: bool,
pub shizuku_bind_requested: bool,
pub shizuku_error_message: Option<String>,
```

Expected: 9 fields removed (adb_devices, selected_device, current_device, selected_user, current_user, shizuku_init_done, shizuku_permission_requested, shizuku_bind_requested, shizuku_error_message)

- [ ] **Step 2: Remove legacy renderer state machine fields**

Delete these 3 fields:

```rust
// Renderer state machines
pub google_play_renderer: RendererStateMachine,
pub fdroid_renderer: RendererStateMachine,
pub apkmirror_renderer: RendererStateMachine,
```

Expected: 3 renderer fields removed

- [ ] **Step 3: Remove dashboard scroll offset fields**

Delete these 7 fields:

```rust
// Dashboard counter scroll offsets
pub dash_scroll_debloat: f32,
pub dash_scroll_stalkerware: f32,
pub dash_scroll_izzyrisk: f32,
pub dash_scroll_virustotal: f32,
pub dash_scroll_hybridanalysis: f32,
pub dash_scroll_offa: f32,
pub dash_scroll_fmhy: f32,
```

Expected: 7 dashboard scroll fields removed

- [ ] **Step 4: Remove package loading and navigation fields**

Delete these 5 fields:

```rust
// Progress tracking for background tasks
pub package_load_progress: std::sync::Arc<std::sync::Mutex<Option<f32>>>,
```

And:

```rust
// Package loading state
// LEGACY: Deleted for mycart browser migration
// pub package_loading_thread:
//     Option<std::thread::JoinHandle<(Vec<crate::adb::PackageFingerprint>, Option<UadNgLists>)>>,
pub package_loading_dialog_open: bool,
pub package_loading_status: String,
```

And:

```rust
// Installer package name (Android) - cached for UI decisions
pub installer_package_name: Option<String>,
```

And:

```rust
// Tab controller state (shared between mobile and desktop UI)
pub show_apps_tab: bool,
```

Expected: 5 fields removed (package_load_progress, package_loading_dialog_open, package_loading_status, installer_package_name, show_apps_tab)

- [ ] **Step 5: Remove debloat performance optimization fields**

Delete these 2 fields:

```rust
// Debloat tab performance optimization
pub debloat_last_enqueued_version: u64,
pub debloat_last_result_load_time: std::time::Instant,
```

Expected: 2 debloat fields removed

- [ ] **Step 6: Remove disclaimer and view state fields**

Delete these 2 fields:

```rust
// Disclaimer dialog state
pub disclaimer_dialog_open: bool,
```

And near the top of the struct:

```rust
pub current_view: AppView,
```

Expected: 2 fields removed (disclaimer_dialog_open, current_view)

- [ ] **Step 7: Remove legacy comment block about tab types**

Find and delete this comment block (around line 145-156):

```rust
// LEGACY: Tab types removed for mycart browser migration
// NOTE: installed_packages and uad_ng_lists are now in shared_store_stt::SharedStore
// Access via: crate::shared_store_stt::get_shared_store()
// pub tab_debloat: TabDebloat, // REMOVED
// pub tab_scan_control: TabScanControl, // REMOVED
// pub tab_usage_control: TabUsageControl, // REMOVED
// pub tab_apps_control: TabAppsControl, // REMOVED
```

Expected: Legacy comment block removed

- [ ] **Step 8: Remove package loading thread comment**

Find and delete this comment (around line 205-206):

```rust
// LEGACY: Deleted for mycart browser migration
// pub package_loading_thread:
//     Option<std::thread::JoinHandle<(Vec<crate::adb::PackageFingerprint>, Option<UadNgLists>)>>,
```

Expected: Comment removed (field already commented out)

- [ ] **Step 9: Verify syntax and format**

```bash
cd /home/wj/work/dure-sijang/mobile
rustfmt --edition 2021 src/dure_sijang_app_stt.rs
cargo check --message-format=short 2>&1 | head -30
```

Expected: rustfmt succeeds, cargo check shows errors about field initialization (fixed in Task 7)

- [ ] **Step 10: Commit struct field removal**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app_stt.rs
git commit -m "$(cat <<'EOF'
refactor(app): remove 28 unused fields from DureSijangApp

Remove ADB/Shizuku device management fields (9), legacy renderer
state machines (3), dashboard scroll offsets (7), package loading
state (3), debloat performance fields (2), and disclaimer/view
state (4).

Part 2 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created

---

## Task 3: Clean up dure_sijang_app.rs Imports and Top-Level Comments

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:17-24`

**Interfaces:**
- Consumes: Struct fields removed in Task 2
- Produces: Clean import block without legacy comments

- [ ] **Step 1: Simplify import block (lines 17-24)**

Replace the import block:

**Before:**
```rust
use crate::db::{
    invalidate_cache,
};
// LEGACY: Deleted for mycart browser migration
// use crate::db_package_cache::get_cached_packages_with_apk;
use crate::material_symbol_icons::{ICON_INFO, ICON_REFRESH};
// LEGACY: Deleted for mycart browser migration
// use crate::models::PackageInfoCache;
```

**After:**
```rust
use crate::db::invalidate_cache;
use crate::material_symbol_icons::{ICON_INFO, ICON_REFRESH};
```

Expected: Import block reduced from 8 lines to 2 lines

- [ ] **Step 2: Format and verify**

```bash
cd /home/wj/work/dure-sijang/mobile
rustfmt --edition 2021 src/dure_sijang_app.rs
cargo check --message-format=short 2>&1 | head -20
```

Expected: rustfmt succeeds, cargo check shows field initialization errors (fixed in Task 7)

- [ ] **Step 3: Commit import cleanup**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app.rs
git commit -m "$(cat <<'EOF'
refactor(app): clean up imports and remove legacy comments

Simplify import blocks and remove LEGACY comment markers.

Part 3 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created

---

## Task 4: Remove Standalone update() Method

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:515-517`

**Interfaces:**
- Consumes: Import cleanup from Task 3
- Produces: Standalone update() method removed

- [ ] **Step 1: Remove standalone update() method (lines 515-517)**

Find and delete this method entirely:

```rust
pub fn update(&mut self, _ctx: &egui::Context, _frame: &eframe::Frame) {
    log::debug!("update function is called.");
}
```

**Reason:** This method is unused. The app uses the `eframe::App::update()` trait implementation (starting around line 1060) instead.

Expected: Method completely removed

- [ ] **Step 2: Verify removal doesn't break trait impl**

Check that the `eframe::App::update()` trait implementation still exists:

```bash
cd /home/wj/work/dure-sijang/mobile
grep -n "impl eframe::App for DureSijangApp" src/dure_sijang_app.rs
grep -A 2 "impl eframe::App for DureSijangApp" src/dure_sijang_app.rs | grep "fn update"
```

Expected: `impl eframe::App` found around line 1059, `fn update` exists in trait impl

- [ ] **Step 3: Format and check**

```bash
rustfmt --edition 2021 src/dure_sijang_app.rs
cargo check --message-format=short 2>&1 | head -20
```

Expected: rustfmt succeeds, cargo check still shows field init errors

- [ ] **Step 4: Commit method removal**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app.rs
git commit -m "$(cat <<'EOF'
refactor(app): remove unused standalone update() method

This method was unused. App uses eframe::App::update() trait impl instead.

Part 4 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created

---

## Task 5: Simplify prepare_tabs_controller() to No-Op

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:910-926`

**Interfaces:**
- Consumes: Standalone update() removed in Task 4
- Produces: prepare_tabs_controller() simplified to no-op

- [ ] **Step 1: Replace prepare_tabs_controller() implementation**

Find the method (around lines 910-926) and replace it:

**Before:**
```rust
fn prepare_tabs_controller(&mut self) {
    // On Android: hide apps tab if installed from Google Play Store
    // On other platforms: always show apps tab
    #[cfg(target_os = "android")]
    {
        self.show_apps_tab = !matches!(
            self.installer_package_name.as_deref(),
            Some("com.android.vending")
        );
    }
    #[cfg(not(target_os = "android"))]
    {
        self.show_apps_tab = true;
    }
}
```

**After:**
```rust
fn prepare_tabs_controller(&mut self) {
    // No-op for mycart browser - tab management handled by browser_ui
    // Method preserved for future tab controller logic
}
```

Expected: Method body reduced to just a comment

- [ ] **Step 2: Format and check**

```bash
cd /home/wj/work/dure-sijang/mobile
rustfmt --edition 2021 src/dure_sijang_app.rs
cargo check --message-format=short 2>&1 | head -20
```

Expected: rustfmt succeeds, cargo check still shows field init errors

- [ ] **Step 3: Commit simplification**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app.rs
git commit -m "$(cat <<'EOF'
refactor(app): simplify prepare_tabs_controller() to no-op

Removed logic that depended on installer_package_name and show_apps_tab
(both deleted). Tab management now handled by browser_ui. Method preserved
for future use.

Part 5 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created

---

## Task 6: Remove Shizuku Polling from eframe::App::update()

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1153-1201`

**Interfaces:**
- Consumes: prepare_tabs_controller() simplified in Task 5
- Produces: Shizuku polling block removed from eframe::App::update()

- [ ] **Step 1: Locate and remove Shizuku polling block (lines 1153-1180)**

Find and delete the entire Shizuku polling section:

```rust
// Poll Shizuku state: auto-retry when permission granted or service bound
#[cfg(target_os = "android")]
let needs_shizuku_polling = {
    let mut needs_polling = false;
    if self.shizuku_permission_requested && self.adb_devices.is_empty() {
        let perm_state = crate::android_shizuku::shizuku_get_permission_state();
        if perm_state == 2 {
            // Permission granted, retry device detection
            self.shizuku_permission_requested = false;
            // LEGACY: Deleted - crate::calc::retrieve_adb_devices(self);
        } else {
            needs_polling = true;
        }
    }
    if self.shizuku_bind_requested && self.adb_devices.is_empty() {
        let bind_state = crate::android_shizuku::shizuku_get_bind_state();
        if bind_state == 2 {
            // Service bound, retry device detection
            self.shizuku_bind_requested = false;
            // LEGACY: Deleted - crate::calc::retrieve_adb_devices(self);
        } else if bind_state == 3 {
            // Bind failed, stop polling
            self.shizuku_bind_requested = false;
        } else {
            needs_polling = true;
        }
    }
    needs_polling
};
```

Expected: Entire block removed

- [ ] **Step 2: Remove reactive repaint trigger (lines 1196-1201)**

Find and delete this block near the end of the update() method:

```rust
// Use reactive mode: only repaint when actually needed
// Only poll for Shizuku state changes if needed (Android only)
#[cfg(target_os = "android")]
if needs_shizuku_polling {
    ctx.request_repaint_after(std::time::Duration::from_millis(500));
}
```

Expected: Repaint trigger removed

- [ ] **Step 3: Format and check**

```bash
cd /home/wj/work/dure-sijang/mobile
rustfmt --edition 2021 src/dure_sijang_app.rs
cargo check --message-format=short 2>&1 | head -30
```

Expected: rustfmt succeeds, cargo check still shows field init errors

- [ ] **Step 4: Commit Shizuku polling removal**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app.rs
git commit -m "$(cat <<'EOF'
refactor(app): remove Shizuku polling from eframe::App::update()

Remove Android Shizuku permission/bind state polling logic that depended
on deleted fields (shizuku_permission_requested, shizuku_bind_requested,
adb_devices).

Part 6 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created

---

## Task 7: Clean Up Default::default() Field Initialization

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:167-345`

**Interfaces:**
- Consumes: Shizuku polling removed in Task 6
- Produces: Default::default() without initialization of deleted fields

- [ ] **Step 1: Remove adb_devices variable (line 199)**

Find and delete:

```rust
// Initialize ADB devices
let adb_devices = Vec::new();
```

Expected: Variable declaration removed

- [ ] **Step 2: Remove current_view field init (line 203)**

Find and delete:

```rust
current_view: AppView::Debloat,
```

Expected: Field init removed

- [ ] **Step 3: Remove ADB/Shizuku field inits (lines 236-243)**

Find and delete:

```rust
adb_devices: adb_devices,
selected_device: None,
current_device: None,

// LEGACY: Deleted for mycart browser migration
// adb_users: Vec::<UserInfo>::new(),
selected_user: None,
current_user: None,
```

Expected: 5 field inits removed

- [ ] **Step 4: Remove package_load_progress and disclaimer_dialog_open (lines 257-262)**

Find and delete:

```rust
package_load_progress: Arc::new(Mutex::new(None)),

// Disclaimer dialog (shows on startup)
disclaimer_dialog_open: true,
```

Expected: 2 field inits removed

- [ ] **Step 5: Remove package loading dialog fields (lines 278-282)**

Find and delete:

```rust
// Package loading state
// LEGACY: Deleted for mycart browser migration
// package_loading_thread: None,
package_loading_dialog_open: false,
package_loading_status: String::new(),
```

Expected: 2 field inits removed, comment removed

- [ ] **Step 6: Remove renderer state machine inits (lines 290-293)**

Find and delete:

```rust
// Renderer state machines (LEGACY - for mycart migration)
google_play_renderer: RendererStateMachine::default(),
fdroid_renderer: RendererStateMachine::default(),
apkmirror_renderer: RendererStateMachine::default(),
```

Expected: 3 field inits removed

- [ ] **Step 7: Remove Shizuku, dashboard, installer, debloat inits (lines 295-315)**

Find and delete:

```rust
// Shizuku state (Android)
shizuku_init_done: false,
shizuku_permission_requested: false,
shizuku_bind_requested: false,
shizuku_error_message: None,

// Dashboard scroll offsets (LEGACY - for mycart migration)
dash_scroll_debloat: 0.0,
dash_scroll_stalkerware: 0.0,
dash_scroll_izzyrisk: 0.0,
dash_scroll_virustotal: 0.0,
dash_scroll_hybridanalysis: 0.0,
dash_scroll_offa: 0.0,
dash_scroll_fmhy: 0.0,

// Installer package name (Android)
installer_package_name: None,

// Debloat performance optimization (LEGACY)
debloat_last_enqueued_version: 0,
debloat_last_result_load_time: std::time::Instant::now(),

// Tab controller state (shared between mobile and desktop UI)
show_apps_tab: true,
```

Expected: 15 field inits removed

- [ ] **Step 8: Remove retrieve_adb_devices comment (line 342)**

Find and delete:

```rust
// Don't call retrieve_adb_devices() here on Android - it will be called
// on first update when the Android context is fully ready
#[cfg(not(target_os = "android"))]
// LEGACY: Deleted - crate::calc::retrieve_adb_devices(&mut app);
```

Expected: Comment block removed

- [ ] **Step 9: Format and check compilation**

```bash
cd /home/wj/work/dure-sijang/mobile
rustfmt --edition 2021 src/dure_sijang_app.rs
cargo check --message-format=short
```

Expected: rustfmt succeeds, cargo check should now compile successfully (all field init errors fixed)

- [ ] **Step 10: Commit Default initialization cleanup**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app.rs
git commit -m "$(cat <<'EOF'
refactor(app): remove initialization of deleted fields in Default::default()

Remove initialization code for 28 deleted fields including ADB devices,
Shizuku state, renderer state machines, dashboard scroll offsets, package
loading state, and debloat performance tracking.

Part 7 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created

---

## Task 8: Simplify First-Run Initialization and Remove Remaining Legacy Comments

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs:1076-1150`

**Interfaces:**
- Consumes: Default initialization cleaned in Task 7
- Produces: Android/Desktop first-run init simplified, all LEGACY comments removed

- [ ] **Step 1: Remove event handling legacy comments (lines 1076-1082)**

Find and delete this comment block in the event polling section:

```rust
// crate::viewmodel::ViewModelEvent::Scan(scan_event) => { // DELETED
//     self.tab_scan_control.apply_scan_event(scan_event);
// }
// crate::viewmodel::ViewModelEvent::Apps(apps_event) => { // DELETED
//     self.tab_apps_control.apply_apps_event(apps_event);
// }
```

Expected: Commented-out match arms removed

- [ ] **Step 2: Simplify Android first-run init (lines 1117-1134)**

Replace Android first-run initialization block:

**Before:**
```rust
#[cfg(target_os = "android")]
if !self.first_update_done {
    self.first_update_done = true;

    // Set UI context for background threads to request repaints
    // LEGACY: Deleted for mycart browser migration
    // let shared_store = crate::shared_store_stt::get_shared_store();
    // shared_store.set_ui_context(ctx.clone());

    log::info!("First update - initializing Shizuku");
    // LEGACY: Deleted - crate::calc::retrieve_adb_devices(self);

    // Check for updates if autoupdate is enabled
    if self.settings.autoupdate {
        log::info!("Autoupdate enabled - checking for updates");
        self.check_for_update();
    }
}
```

**After:**
```rust
#[cfg(target_os = "android")]
if !self.first_update_done {
    self.first_update_done = true;

    if self.settings.autoupdate {
        log::info!("Autoupdate enabled - checking for updates");
        self.check_for_update();
    }
}
```

Expected: Shizuku log and comments removed

- [ ] **Step 3: Simplify Desktop first-run init (lines 1136-1150)**

Replace Desktop first-run initialization block:

**Before:**
```rust
#[cfg(not(target_os = "android"))]
if !self.first_update_done {
    self.first_update_done = true;

    // Set UI context for background threads to request repaints
    // LEGACY: Deleted for mycart browser migration
    // let shared_store = crate::shared_store_stt::get_shared_store();
    // shared_store.set_ui_context(ctx.clone());

    // Check for updates if autoupdate is enabled
    if self.settings.autoupdate {
        log::info!("Autoupdate enabled - checking for updates");
        self.check_for_update();
    }
}
```

**After:**
```rust
#[cfg(not(target_os = "android"))]
if !self.first_update_done {
    self.first_update_done = true;

    if self.settings.autoupdate {
        log::info!("Autoupdate enabled - checking for updates");
        self.check_for_update();
    }
}
```

Expected: Shared_store comment removed

- [ ] **Step 4: Search for any remaining LEGACY comments**

```bash
cd /home/wj/work/dure-sijang/mobile
grep -n "LEGACY" src/dure_sijang_app.rs
```

Expected: No matches found (all LEGACY comments removed)

- [ ] **Step 5: Format and verify**

```bash
rustfmt --edition 2021 src/dure_sijang_app.rs
cargo check --message-format=short
```

Expected: rustfmt succeeds, cargo check compiles successfully

- [ ] **Step 6: Commit first-run init cleanup and LEGACY comment removal**

```bash
cd /home/wj/work/dure-sijang
git add mobile/src/dure_sijang_app.rs
git commit -m "$(cat <<'EOF'
refactor(app): simplify first-run init and remove all LEGACY comments

Simplify Android and Desktop first-run initialization by removing Shizuku
logging and shared_store comments. Remove all remaining LEGACY comment
markers throughout the file.

Part 8 of deep clean refactoring for mycart browser migration.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Commit created

---

## Task 9: Verify Compilation, Run Clippy, and Create Summary

**Files:**
- Verify: `mobile/src/dure_sijang_app.rs`
- Verify: `mobile/src/dure_sijang_app_stt.rs`

**Interfaces:**
- Consumes: All refactoring from Tasks 1-8 complete
- Produces: Verified clean compilation with no clippy warnings

- [ ] **Step 1: Run cargo check on entire workspace**

```bash
cd /home/wj/work/dure-sijang
cargo check --workspace --message-format=short
```

Expected: ✅ Finished `dev` [unoptimized + debuginfo] target(s)

- [ ] **Step 2: Run cargo clippy**

```bash
cargo clippy --workspace -- -D warnings
```

Expected: ✅ No warnings (or only unrelated warnings)

- [ ] **Step 3: Verify file size reduction**

```bash
wc -l mobile/src/dure_sijang_app.rs mobile/src/dure_sijang_app_stt.rs
```

Expected:
- `dure_sijang_app.rs`: ~1150 lines (down from ~1341)
- `dure_sijang_app_stt.rs`: ~200 lines (down from ~257)

- [ ] **Step 4: Verify preserved functionality**

```bash
cd /home/wj/work/dure-sijang/mobile
grep "browser_ui: BrowserUI" src/dure_sijang_app_stt.rs
grep "viewmodel: Option<ViewModel>" src/dure_sijang_app_stt.rs
grep "webviews: HashMap" src/dure_sijang_app_stt.rs
grep "extract_github_embedded_data" src/dure_sijang_app.rs
grep "install_status:" src/dure_sijang_app_stt.rs
grep "update_status:" src/dure_sijang_app_stt.rs
```

Expected: All 6 greps return matches

- [ ] **Step 5: Create refactoring summary**

```bash
cd /home/wj/work/dure-sijang
echo "=== Dure-Sijang App Refactoring Summary ==="
echo ""
echo "Files modified:"
git diff --stat 50d04b6..HEAD mobile/src/dure_sijang_app.rs mobile/src/dure_sijang_app_stt.rs
echo ""
echo "Commits created (last 8):"
git log --oneline --no-decorate -8
echo ""
echo "Verification:"
cargo check --workspace 2>&1 | grep "Finished" || echo "❌ Check failed"
cargo clippy --workspace -- -D warnings 2>&1 | tail -1
```

Expected: Summary showing changes, commits, verification status

- [ ] **Step 6: Final verification commit**

```bash
cd /home/wj/work/dure-sijang
git commit --allow-empty -m "$(cat <<'EOF'
refactor(app): deep clean refactoring complete

✅ 28 fields removed from DureSijangApp
✅ 2 type definitions removed (RendererStateMachine, AppView)
✅ All LEGACY comment markers removed
✅ cargo check passes
✅ cargo clippy clean
✅ File size reduced by ~150-200 lines
✅ Preserved functionality verified

Deep clean migration from Dure-Sijang to mycart browser complete.

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected: Final commit created

---

## Execution Handoff

Plan complete and saved to `docs/superpowers/plans/2026-08-20-dure-sijang-app-refactor.md`. Two execution options:

**1. Subagent-Driven (recommended)** - I dispatch a fresh subagent per task, review between tasks, fast iteration

**2. Inline Execution** - Execute tasks in this session using executing-plans, batch execution with checkpoints

Which approach?
