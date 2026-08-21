# Dure-Sijang App Refactor Design

**Date:** 2026-08-20  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This spec defines the refactoring of `dure_sijang_app.rs` and `dure_sijang_app_stt.rs` to remove legacy code from the Dure-Sijang (Android debloat/scan/apps) feature set after migration to the mycart browser architecture (August 2026).

## Goals

1. **Remove Legacy Code**: Delete unused fields, methods, and types from pre-August 2026 architecture
2. **Clean Comments**: Remove `// LEGACY: Deleted...` markers (git history preserves context)
3. **Simplify Initialization**: Remove setup code for deleted features
4. **Preserve Functionality**: Keep all active mycart browser features, install/update system, and future-use helpers
5. **Maintain Cross-Platform**: Keep Android + Desktop support (remove only ADB/Shizuku device management)

## Approach: Deep Clean (Aggressive)

Remove unused fields/methods AND clean up all legacy comments, unused imports, and dead code paths.

**Why Deep Clean:**
- Clear migration to mycart browser - old features are permanently gone
- Git history preserves migration context - don't need in-code comments
- Clean code easier to maintain and onboard new developers
- Verbose legacy markers are distracting

**Risk Mitigation:**
- All removals confirmed via clarifying questions with user
- Will verify with grep that no code references removed items
- Run `cargo check` and `cargo clippy` after changes
- Test desktop and Android launch

## Design Details

### 1. Fields to Remove

Remove **28 fields** from `DureSijangApp` struct in `dure_sijang_app_stt.rs`:

#### ADB/Shizuku Device Management (9 fields)
```rust
pub adb_devices: Vec<String>,
pub selected_device: Option<String>,
pub current_device: Option<String>,
pub selected_user: Option<i32>,
pub current_user: Option<i32>,
pub shizuku_init_done: bool,
pub shizuku_permission_requested: bool,
pub shizuku_bind_requested: bool,
pub shizuku_error_message: Option<String>,
```

**Reason:** Mycart browser doesn't need Android device management via ADB/Shizuku. Browser runs directly on device.

#### Legacy Feature State (12 fields)
```rust
pub google_play_renderer: RendererStateMachine,
pub fdroid_renderer: RendererStateMachine,
pub apkmirror_renderer: RendererStateMachine,
pub dash_scroll_debloat: f32,
pub dash_scroll_stalkerware: f32,
pub dash_scroll_izzyrisk: f32,
pub dash_scroll_virustotal: f32,
pub dash_scroll_hybridanalysis: f32,
pub dash_scroll_offa: f32,
pub dash_scroll_fmhy: f32,
pub debloat_last_enqueued_version: u64,
pub debloat_last_result_load_time: std::time::Instant,
```

**Reason:** All related to old Dure-Sijang debloat/scan/apps features. Dashboard counters tracked app analysis metrics. No longer needed for mycart browser.

#### Package Loading & Navigation (5 fields)
```rust
pub package_load_progress: Arc<Mutex<Option<f32>>>,
pub package_loading_dialog_open: bool,
pub package_loading_status: String,
pub installer_package_name: Option<String>,
pub show_apps_tab: bool,
```

**Reason:** Package loading was for Android app management. `installer_package_name` determined if app was from Play Store (to hide "Apps" tab). `show_apps_tab` controlled tab visibility. All obsolete for mycart browser navigation via `browser_ui`.

#### Disclaimer & View State (2 fields)
```rust
pub disclaimer_dialog_open: bool,
pub current_view: AppView,
```

**Reason:** `disclaimer_dialog_open` showed startup disclaimer (no longer needed). `current_view` tracked which legacy tab was active (Debloat/Scan/Apps/Usage). Browser uses `browser_ui` for navigation instead.

#### Type Definitions to Remove
```rust
pub struct RendererStateMachine {
    pub is_enabled: bool,
}

pub enum AppView {
    Debloat,
    Scan,
    Apps,
    Usage,
    Browser,
}
```

**Reason:** `RendererStateMachine` managed GooglePlay/FDroid/APKMirror metadata renderers. `AppView` enum defined old tab types. Both obsolete.

### 2. Methods to Remove/Clean

#### Remove Entirely

**Standalone `update()` method** (`dure_sijang_app.rs` lines 515-517)
```rust
pub fn update(&mut self, _ctx: &egui::Context, _frame: &eframe::Frame) {
    log::debug!("update function is called.");
}
```

**Reason:** Unused. App uses `eframe::App::update()` trait impl (line 1060+) instead.

#### Simplify

**`prepare_tabs_controller()`** (`dure_sijang_app.rs` lines 910-926)

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

**Reason:** Logic depended on `installer_package_name` and `show_apps_tab` (both deleted). Mycart browser uses `browser_ui` for tab management. Keep method as no-op for future use.

**`eframe::App::update()` Shizuku polling** (`dure_sijang_app.rs` lines 1153-1201)

Remove entire Shizuku polling block:
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

// ... later at end of method ...

// Use reactive mode: only repaint when actually needed
// Only poll for Shizuku state changes if needed (Android only)
#[cfg(target_os = "android")]
if needs_shizuku_polling {
    ctx.request_repaint_after(std::time::Duration::from_millis(500));
}
```

**Reason:** All Shizuku fields deleted. This polling loop checked for permission/bind state changes. Not needed for mycart browser.

### 3. Imports & Comments to Clean

#### Imports to Remove

**`dure_sijang_app.rs` top imports (lines 17-24):**

Before:
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

After:
```rust
use crate::db::invalidate_cache;
use crate::material_symbol_icons::{ICON_INFO, ICON_REFRESH};
```

**`dure_sijang_app_stt.rs` top imports (lines 1-9):**

Before:
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

After:
```rust
#[cfg(not(target_os = "android"))]
use crate::install_stt::InstallStatus;
```

#### Legacy Comments to Remove

Remove all `// LEGACY: Deleted...` comment blocks:
- Line 20-21: db imports comment
- Line 23-24: models imports comment
- Lines 145-156 (_stt.rs): commented tab types
- Line 205-206 (_stt.rs): package loading thread comment
- Lines 1076-1082: event handling comments in `eframe::App::update()`
- Lines 1122-1124: shared_store context comment (Android first-run)
- Lines 1142-1144: shared_store context comment (Desktop first-run)
- Line 1161: retrieve_adb_devices comment
- Line 1171: retrieve_adb_devices comment

**Rationale:** Git commit `50d04b6 refactor: remove legacy code references for mycart browser` (2026-08-20) and earlier commits preserve the migration history. In-code comments create clutter and duplicate git history.

### 4. Initialization Logic to Simplify

#### `Default::default()` Implementation

**Remove field initialization for deleted fields** (`dure_sijang_app.rs` lines 167-345):

```rust
// Line 199: Remove
let adb_devices = Vec::new();

// Lines 203: Remove
current_view: AppView::Debloat,

// Lines 236-243: Remove
adb_devices: adb_devices,
selected_device: None,
current_device: None,
selected_user: None,
current_user: None,

// Lines 257-262: Remove
package_load_progress: Arc::new(Mutex::new(None)),
disclaimer_dialog_open: true,

// Lines 278-282: Remove
package_loading_dialog_open: false,
package_loading_status: String::new(),

// Lines 290-293: Remove
google_play_renderer: RendererStateMachine::default(),
fdroid_renderer: RendererStateMachine::default(),
apkmirror_renderer: RendererStateMachine::default(),

// Lines 295-315: Remove
shizuku_init_done: false,
shizuku_permission_requested: false,
shizuku_bind_requested: false,
shizuku_error_message: None,
dash_scroll_debloat: 0.0,
dash_scroll_stalkerware: 0.0,
dash_scroll_izzyrisk: 0.0,
dash_scroll_virustotal: 0.0,
dash_scroll_hybridanalysis: 0.0,
dash_scroll_offa: 0.0,
dash_scroll_fmhy: 0.0,
installer_package_name: None,
debloat_last_enqueued_version: 0,
debloat_last_result_load_time: std::time::Instant::now(),
show_apps_tab: true,

// Line 342: Remove
#[cfg(not(target_os = "android"))]
// LEGACY: Deleted - crate::calc::retrieve_adb_devices(&mut app);
```

#### First-run Android Initialization

**Simplify Android first-run init** (`dure_sijang_app.rs` lines 1117-1134):

Before:
```rust
#[cfg(target_os = "android")]
if !self.first_update_done {
    self.first_update_done = true;
    
    // LEGACY: Deleted for mycart browser migration
    // let shared_store = crate::shared_store_stt::get_shared_store();
    // shared_store.set_ui_context(ctx.clone());
    
    log::info!("First update - initializing Shizuku");
    // LEGACY: Deleted - crate::calc::retrieve_adb_devices(self);
    
    if self.settings.autoupdate {
        log::info!("Autoupdate enabled - checking for updates");
        self.check_for_update();
    }
}
```

After:
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

**Reason:** Remove Shizuku initialization log and commented-out `retrieve_adb_devices()` call. Only autoupdate check remains.

### 5. Preserved Functionality

**Keep these as-is (explicitly NOT removing):**

#### Core Browser Functionality
- ✅ `browser_ui: BrowserUI` - Mycart browser UI component
- ✅ `viewmodel: Option<ViewModel>` - MVVM architecture with actors
- ✅ `webviews: HashMap<usize, wry::WebView>` - WebView management for tabs
- ✅ `window_handle: Option<RawWindowHandle>` - Platform integration for webview creation
- ✅ WebView lifecycle methods: `create_webview()`, `destroy_webview()`, `navigate_back()`, `navigate_forward()`, `navigate_reload()`

#### Platform Support
- ✅ `zoom_factor: f32` - Touch device pinch-to-zoom support (Android)
- ✅ All `#[cfg(target_os = "android")]` and `#[cfg(not(target_os = "android"))]` blocks that don't reference deleted fields

#### Install/Update System (Desktop)
- ✅ `install_status: InstallStatus` - Desktop install state
- ✅ `install_dialog_open: bool` - Desktop install dialog state
- ✅ `install_message: String` - Desktop install result message
- ✅ `update_status: String` - Update check result
- ✅ `update_available: bool` - Update availability flag
- ✅ `update_checking: bool` - Update check in-progress flag
- ✅ Methods: `perform_install_action()`, `show_install_dialog()`, `check_for_update()`, `perform_update()`

#### Settings & Dialogs
- ✅ `settings: Settings` - User preferences
- ✅ `dlg_settings: DlgSettings` - Settings dialog state
- ✅ `dlg_about: DlgAbout` - About dialog state
- ✅ `dlg_update: DlgUpdate` - Update dialog state

#### UI Infrastructure
- ✅ Theme system: `apply_saved_theme_preferences()`, `detect_os_theme()`, `apply_theme_by_name()`
- ✅ i18n system: `apply_saved_language()`, `detect_system_language()`
- ✅ Logging system: `append_log()`, `render_logs()`, log buffer and settings
- ✅ Menu system: `show_menus()`, `create_menu_item()`

#### Future-Use Helpers
- ✅ `extract_github_embedded_data(html: &str)` - Parses GitHub HTML for embedded JSON. User confirmed: "will need later"

#### Miscellaneous
- ✅ `first_update_done: bool` - First-run initialization flag
- ✅ All initialization helpers: `init_common()`, `init_egui()`
- ✅ All theme/text style conversion helpers: `string_to_log_level()`, `log_level_to_string()`, etc.

## Files Affected

### 1. `mobile/src/dure_sijang_app_stt.rs`
- Remove 28 struct fields from `DureSijangApp`
- Remove `RendererStateMachine` struct definition
- Remove `AppView` enum definition
- Clean up imports (remove commented-out legacy imports)

### 2. `mobile/src/dure_sijang_app.rs`
- Remove standalone `update()` method (lines 515-517)
- Simplify `prepare_tabs_controller()` to no-op (lines 910-926)
- Remove Shizuku polling from `eframe::App::update()` (lines 1153-1201)
- Remove initialization code for deleted fields in `Default::default()` (lines 167-345)
- Simplify first-run Android initialization (lines 1117-1134)
- Clean up imports (lines 17-24)
- Remove ~10 `// LEGACY: Deleted...` comment blocks throughout

## Impact Assessment

**Lines removed:** ~150-200 lines (estimated 15% reduction of `dure_sijang_app.rs` file size)

**Risk:** Low
- Only removing confirmed unused code
- All removals verified via user clarification questions
- No changes to active browser functionality
- Cross-platform support (Android + Desktop) preserved
- Install/update system preserved
- Future-use helpers preserved

**Testing needed after refactoring:**
1. ✅ `cargo check` - Verify compilation succeeds
2. ✅ `cargo clippy` - Check for new warnings/errors
3. ✅ Desktop launch test - Verify app starts and browser_ui renders
4. ✅ Android launch test - Verify app starts on Android (if build environment available)

**Post-refactor file size estimate:**
- `dure_sijang_app.rs`: ~1150 lines (down from ~1341)
- `dure_sijang_app_stt.rs`: ~200 lines (down from ~257)

## Open Questions

None - all design decisions validated via clarifying questions with user.

## Success Criteria

1. ✅ All 28 identified fields removed from struct
2. ✅ All legacy comments removed (git history preserves context)
3. ✅ All unused imports cleaned up
4. ✅ Initialization logic simplified (no references to deleted fields)
5. ✅ `cargo check` passes
6. ✅ `cargo clippy` has no new warnings
7. ✅ Active browser functionality preserved
8. ✅ Install/update system preserved
9. ✅ Cross-platform support preserved
10. ✅ `extract_github_embedded_data()` preserved for future use

## Next Steps

1. Create implementation plan via `writing-plans` skill
2. Execute refactoring task-by-task
3. Run `cargo check` and `cargo clippy` after each major change
4. Test desktop launch
5. Test Android launch (if available)
6. Commit with message: `refactor(app): deep clean legacy Dure-Sijang code`

## References

- Existing code: `mobile/src/dure_sijang_app.rs` (1341 lines)
- Existing code: `mobile/src/dure_sijang_app_stt.rs` (257 lines)
- Migration context: `docs/mvvm-actor-migration-complete.md`
- Recent commits:
  - `50d04b6 refactor: remove legacy code references for mycart browser`
  - `a989ccd test(browser): add comprehensive WebView integration tests`
  - `f87d527 feat(browser): implement platform-specific WebView creation`
