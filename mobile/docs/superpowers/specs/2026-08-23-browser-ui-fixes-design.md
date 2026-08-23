# Browser UI Fixes for Android - Design Specification

**Date:** 2026-08-23  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Problem Statement

Android browser is completely non-functional:
1. No way to create new tabs (no "+" button)
2. URL navigation doesn't work (Go button and Enter do nothing)
3. No tabs created on app launch
4. Desktop works correctly with 2 auto-created tabs and full UI

**Goal:** Make Android browser match desktop functionality with tab UI and WebView visible simultaneously on the same screen.

## Architecture Overview

### Approach: Activity Embedding

Use Android's Activity Embedding feature (Android 12L+) to display egui UI and wry WebView simultaneously in a split-screen layout.

```
┌─────────────────────────────────────────────────────┐
│ NativeActivity (Rust + egui)                        │
│ ┌─────────────────────────────────────────────┐    │
│ │ Browser Controls (30% of screen)            │    │
│ │ • Tab bar: [dure.app] [dure.app] [+]        │    │
│ │ • URL input + Go button                     │    │
│ │ • Back/Forward/Bookmark buttons             │    │
│ └─────────────────────────────────────────────┘    │
│ ┌─────────────────────────────────────────────┐    │
│ │ WebViewActivity (embedded, 70% of screen)   │    │
│ │                                              │    │
│ │    wry WebView renders page content here    │    │
│ │                                              │    │
│ └─────────────────────────────────────────────┘    │
└─────────────────────────────────────────────────────┘
```

### Key Architectural Decisions

1. **Two Activities:** 
   - NativeActivity (Rust/egui) - browser controls
   - WebViewActivity (Kotlin/wry) - web content

2. **Split Ratio:** 30% top (controls) / 70% bottom (content)

3. **Tab Switching Strategy:** 
   - Only one WebView active at a time
   - Destroy old WebViewActivity, launch new one when switching tabs
   - Matches mobile browser pattern (memory efficient)

4. **Initial State:** 
   - Auto-create 2 tabs to https://dure.app on launch
   - Launch WebViewActivity for first tab (active by default)

5. **Communication:** 
   - JNI bridge for Rust ↔ Kotlin/Java
   - Rust calls Kotlin to launch/destroy/navigate WebViewActivity

### Why Activity Embedding?

**Problem with single-activity approach:**
- wry WebViews on Android are native Views that overlay the window
- Cannot be hidden/shown like desktop's egui_webview widgets
- Would cover the entire window, hiding tab bar and controls

**Activity Embedding solution:**
- Android manages layout and positioning automatically
- Tab UI and WebView both visible simultaneously
- Clean separation of concerns
- Proper activity lifecycle for WebView

## Components

### New Files to Create

#### 1. `mobile/app/src/main/java/app/dure/sijang/WebViewActivity.kt`

Kotlin Activity that hosts wry WebView.

**Responsibilities:**
- Receive URL via Intent extras (`intent.getStringExtra("url")`)
- Receive tab ID via Intent extras (`intent.getIntExtra("tab_id")`)
- Create native wry WebView instance on onCreate
- Handle activity lifecycle (onDestroy, onPause, onResume)
- Clean up WebView resources on destroy

**Key Methods:**
```kotlin
class WebViewActivity : AppCompatActivity() {
    private var wryWebView: WryWebView? = null
    
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        val url = intent.getStringExtra("url") ?: "https://dure.app"
        val tabId = intent.getIntExtra("tab_id", -1)
        
        wryWebView = WryWebView(this, url)
        setContentView(wryWebView)
    }
    
    override fun onDestroy() {
        wryWebView?.destroy()
        super.onDestroy()
    }
    
    fun loadUrl(url: String) {
        wryWebView?.loadUrl(url)
    }
}
```

#### 2. `mobile/app/src/main/java/app/dure/sijang/WryWebViewBridge.kt`

JNI bridge class for Rust ↔ Kotlin communication.

**Responsibilities:**
- Provide static methods callable from Rust via JNI
- Launch WebViewActivity with URL and tab ID
- Navigate existing WebViewActivity to new URL
- Destroy WebViewActivity when tab closed

**Key Methods:**
```kotlin
object WryWebViewBridge {
    private var currentActivity: WebViewActivity? = null
    
    @JvmStatic
    fun launchWebView(activity: Activity, url: String, tabId: Int) {
        val intent = Intent(activity, WebViewActivity::class.java).apply {
            putExtra("url", url)
            putExtra("tab_id", tabId)
        }
        activity.startActivity(intent)
    }
    
    @JvmStatic
    fun destroyWebView(activity: Activity) {
        currentActivity?.finish()
        currentActivity = null
    }
    
    @JvmStatic
    fun navigateWebView(url: String) {
        currentActivity?.loadUrl(url)
    }
}
```

#### 3. `mobile/app/src/main/res/xml/main_split_config.xml`

Activity Embedding split rules configuration.

**Content:**
```xml
<?xml version="1.0" encoding="utf-8"?>
<split-config xmlns:window="http://schemas.android.com/apk/res-auto">
    <SplitPairRule
        window:splitRatio="0.3"
        window:splitLayoutDirection="topToBottom"
        window:finishPrimaryWithSecondary="never"
        window:finishSecondaryWithPrimary="always">
        
        <!-- Top 30% = NativeActivity (egui controls) -->
        <!-- Bottom 70% = WebViewActivity (wry WebView) -->
        
        <SplitPairFilter
            window:primaryActivityName=".NativeActivity"
            window:secondaryActivityName=".WebViewActivity"/>
    </SplitPairRule>
</split-config>
```

**Configuration Details:**
- `splitRatio="0.3"` - Primary (NativeActivity) takes 30% of screen
- `splitLayoutDirection="topToBottom"` - Primary on top, secondary below
- `finishPrimaryWithSecondary="never"` - Keep egui UI when WebView closes
- `finishSecondaryWithPrimary="always"` - Close WebView if egui app closes

#### 4. `mobile/src/android_activity_embedding.rs`

Rust JNI wrapper for calling Kotlin bridge methods.

**Responsibilities:**
- Safe Rust API for launching/destroying/navigating WebViewActivity
- Handle JNI errors gracefully
- Log all operations for debugging

**Key Functions:**
```rust
#[cfg(target_os = "android")]
pub fn launch_webview_activity(url: &str, tab_id: i32) -> anyhow::Result<()> {
    use jni::objects::{JObject, JValue, JString};
    use jni::JavaVM;
    
    let vm = unsafe { JavaVM::from_raw(ndk_context::android_context().vm().cast())? };
    let mut env = vm.attach_current_thread()?;
    
    let activity = get_native_activity(&mut env)?;
    let url_jstring = env.new_string(url)?;
    
    env.call_static_method(
        "app/dure/sijang/WryWebViewBridge",
        "launchWebView",
        "(Landroid/app/Activity;Ljava/lang/String;I)V",
        &[
            JValue::Object(&activity),
            JValue::Object(&url_jstring),
            JValue::Int(tab_id),
        ],
    )?;
    
    log::info!("Launched WebViewActivity for tab {} with URL: {}", tab_id, url);
    Ok(())
}

#[cfg(target_os = "android")]
pub fn destroy_webview_activity() -> anyhow::Result<()> {
    // Similar JNI call to destroyWebView
}

#[cfg(target_os = "android")]
pub fn navigate_webview(url: &str) -> anyhow::Result<()> {
    // Similar JNI call to navigateWebView
}

#[cfg(not(target_os = "android"))]
pub fn launch_webview_activity(_url: &str, _tab_id: i32) -> anyhow::Result<()> {
    Ok(()) // No-op on non-Android
}
```

### Files to Modify

#### 5. `mobile/src/dure_sijang_app.rs`

**Changes:**

1. **Remove platform guards from UI elements:**
   - Remove `#[cfg(not(target_os = "android"))]` from "+" button (lines 1486, 1523)
   - Remove guards from tab creation logic
   - Keep guards only for desktop-specific egui_webview rendering

2. **Modify `add_browser_tab()` for Android:**
```rust
pub fn add_browser_tab(&mut self, ctx: &egui::Context, frame: &eframe::Frame, url: &str) {
    // Create tab metadata
    let tab_id = self.browser_state.add_tab(url, url);
    
    #[cfg(not(target_os = "android"))]
    {
        // Desktop: Create egui_webview widget
        let view = egui_webview::EguiWebView::new(url, frame);
        self.webview_widgets.insert(tab_id, view);
    }
    
    #[cfg(target_os = "android")]
    {
        // Android: Launch WebViewActivity via JNI
        if let Err(e) = crate::android_activity_embedding::launch_webview_activity(
            url,
            self.browser_state.tabs.len() as i32 - 1,
        ) {
            log::error!("Failed to launch WebViewActivity: {}", e);
            self.browser_state.close_tab(self.browser_state.tabs.len() - 1);
        }
    }
}
```

3. **Add initialization method called from `new()`:**
```rust
fn initialize_browser(&mut self) {
    // If no tabs exist (first launch), create 2 default tabs
    if self.browser_state.tabs.is_empty() {
        log::info!("First launch - creating 2 default tabs");
        
        // Create 2 tabs to dure.app (metadata only, no WebView yet)
        self.browser_state.add_tab("https://dure.app", "dure.app");
        self.browser_state.add_tab("https://dure.app", "dure.app");
        
        // Set first tab as active
        self.browser_state.active_tab_index = Some(0);
        self.browser_state.url_input = "https://dure.app".to_string();
    }
}
```

Call from `DureSijangApp::new()`:
```rust
pub fn new(cc: &eframe::CreationContext) -> Self {
    let mut app = Self::default();
    app.initialize_browser();
    app
}
```

4. **Add first update() logic to launch initial WebView:**
```rust
impl eframe::App for DureSijangApp {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // On first update, launch WebViewActivity for active tab (Android only)
        #[cfg(target_os = "android")]
        if !self.first_update_done {
            self.first_update_done = true;
            
            if let Some(active_idx) = self.browser_state.active_tab_index {
                if active_idx < self.browser_state.tabs.len() {
                    let url = self.browser_state.tabs[active_idx].url_bar.clone();
                    if let Err(e) = crate::android_activity_embedding::launch_webview_activity(
                        &url,
                        active_idx as i32,
                    ) {
                        log::error!("Failed to launch initial WebView: {}", e);
                    }
                }
            }
        }
        
        // Rest of update logic...
    }
}
```

5. **Modify tab switching to destroy/launch WebViewActivity:**
```rust
// In render_browser_ui() where tabs are clicked
if ui.selectable_label(is_active, display_title).clicked() {
    log::info!("Switching active tab from {:?} to {}", self.browser_state.active_tab_index, idx);
    
    #[cfg(target_os = "android")]
    {
        // Destroy old WebViewActivity
        let _ = crate::android_activity_embedding::destroy_webview_activity();
        
        // Launch new WebViewActivity for clicked tab
        let url = self.browser_state.tabs[idx].url_bar.clone();
        if let Err(e) = crate::android_activity_embedding::launch_webview_activity(&url, idx as i32) {
            log::error!("Failed to switch tab: {}", e);
        }
    }
    
    self.browser_state.active_tab_index = Some(idx);
    self.browser_state.url_input = tab.url_bar.clone();
}
```

#### 6. `mobile/src/lib.rs`

**Add module:**
```rust
#[cfg(target_os = "android")]
mod android_activity_embedding;
```

#### 7. `mobile/app/src/main/AndroidManifest.xml`

**Add WebViewActivity declaration:**
```xml
<activity
    android:name=".WebViewActivity"
    android:configChanges="orientation|screenSize|keyboardHidden"
    android:exported="false" />

<property
    android:name="android.window.PROPERTY_ACTIVITY_EMBEDDING_SPLITS_ENABLED"
    android:value="true" />
```

**Purpose:**
- `configChanges` prevents Activity recreation on rotation
- `exported="false"` - only launchable from within app
- `PROPERTY_ACTIVITY_EMBEDDING_SPLITS_ENABLED` - enables split-screen

#### 8. `mobile/app/build.gradle`

**Add dependency:**
```gradle
dependencies {
    // Existing dependencies...
    implementation 'androidx.window:window:1.2.0'
}
```

**Purpose:** Provides Activity Embedding APIs (backward compatible to API 24)

## Data Flow

### Flow 1: App Launch (Initial State)

```
1. main_android.rs calls DureSijangApp::new()
   ↓
2. DureSijangApp::default() creates empty browser_state
   ↓
3. initialize_browser() called
   ↓
4. Checks if browser_state.tabs is empty
   ↓
5. Creates 2 tabs: browser_state.add_tab("https://dure.app", "dure.app") x2
   ↓
6. Sets active_tab_index = Some(0)
   ↓
7. First update() called
   ↓
8. Detects !first_update_done on Android
   ↓
9. JNI call: launch_webview_activity("https://dure.app", 0)
   ↓
10. Kotlin: Intent created, WebViewActivity starts
    ↓
11. WebViewActivity.onCreate(): Creates wry WebView with URL
    ↓
12. Activity Embedding: Splits screen 30/70
    ↓
13. User sees: Tab bar (30%) + WebView with dure.app (70%)
```

### Flow 2: User Creates New Tab (Clicks "+")

```
1. User clicks "+" button in tab bar
   ↓
2. add_browser_tab(ctx, frame, "https://dure.app") called
   ↓
3. browser_state.add_tab() creates metadata, returns tab_id
   ↓
4. Database: create_tab() persists to SQLite
   ↓
5. Sets active_tab_index to new tab (last index)
   ↓
6. Android path: destroy_webview_activity() via JNI
   ↓
7. Kotlin: currentActivity.finish() destroys old WebViewActivity
   ↓
8. JNI: launch_webview_activity("https://dure.app", new_tab_id)
   ↓
9. Kotlin: New WebViewActivity starts with fresh wry WebView
   ↓
10. Updates url_input field to "https://dure.app"
    ↓
11. User sees new tab active with fresh WebView
```

### Flow 3: User Navigates URL (Enter/Go button)

```
1. User types URL in input field
   ↓
2. User presses Enter OR clicks "Go" button
   ↓
3. Get active_tab_index (must be Some)
   ↓
4. ensure_protocol(&url_input) adds "https://" if missing
   ↓
5. browser_state.update_tab_url(active_idx, url, None)
   ↓
6. Database: update tab's current_url field
   ↓
7. Android path: navigate_webview(url) via JNI
   ↓
8. Kotlin: currentActivity.loadUrl(url)
   ↓
9. wry WebView: webview.load_url(url)
   ↓
10. WebView navigates to new URL
    ↓
11. Database: Insert into browsing_history table
    ↓
12. browser_state.refresh_history() reloads history entries
```

### Flow 4: User Switches Tabs (Clicks tab)

```
1. User clicks different tab in tab bar
   ↓
2. Tab click handler detects idx != active_tab_index
   ↓
3. Android path: destroy_webview_activity() via JNI
   ↓
4. Kotlin: currentActivity.finish()
   ↓
5. Old WebViewActivity destroyed, WebView cleaned up
   ↓
6. Get clicked tab's URL: tabs[idx].url_bar
   ↓
7. JNI: launch_webview_activity(url, idx)
   ↓
8. Kotlin: New WebViewActivity starts
   ↓
9. wry WebView created with clicked tab's URL
   ↓
10. Updates active_tab_index = Some(idx)
    ↓
11. Updates url_input = tabs[idx].url_bar
    ↓
12. User sees content from selected tab
```

### Flow 5: User Closes Tab (Clicks "×")

```
1. User clicks "×" on tab
   ↓
2. Stores tab_to_close = Some(idx) (avoid borrow conflict)
   ↓
3. After tab bar render: process tab_to_close
   ↓
4. close_browser_tab(idx) called
   ↓
5. Android path: Check if closing active tab
   ↓
6. If active: destroy_webview_activity() via JNI
   ↓
7. browser_state.close_tab(idx)
   ↓
8. Database: delete_tab(db_id)
   ↓
9. Removes tab from tabs Vec
   ↓
10. If was active tab: Calculate new active (adjacent tab)
    ↓
11. If new active exists: launch_webview_activity for it
    ↓
12. If no tabs left: active_tab_index = None, show empty state
```

## Error Handling

### Error Scenarios and Recovery Strategies

#### 1. WebViewActivity Launch Failure

**Scenario:** JNI call fails or Activity won't start

**Detection:**
```rust
if let Err(e) = launch_webview_activity(url, tab_id) {
    // Error handling here
}
```

**Handling:**
- Log error with full details: `log::error!("Failed to launch WebViewActivity: {}", e)`
- Show toast notification to user: "Failed to open tab"
- Remove tab metadata: `browser_state.close_tab(idx)`
- If it was the only tab, show empty state with "Retry" button
- Don't crash the app - egui UI remains functional
- User can try creating new tab or navigating different URL

#### 2. JNI Bridge Errors

**Scenario:** Can't find Java class/method, JNI environment unavailable

**Detection:**
```rust
let vm = JavaVM::from_raw(...)
    .map_err(|e| anyhow!("JNI VM unavailable: {}", e))?;

env.call_static_method(...)
    .map_err(|e| anyhow!("JNI method call failed: {}", e))?;
```

**Handling:**
- Log detailed error with class/method name
- Disable Android browser functionality (set flag: `browser_available = false`)
- Show one-time warning dialog: "Browser requires Android 12L+ with Activity Embedding support"
- UI shows grayed-out "+" button with tooltip explaining limitation
- App remains usable without browser (other features work)

#### 3. Activity Embedding Not Supported (Android < 12L)

**Scenario:** Device doesn't support Activity Embedding

**Detection:**
```kotlin
// In WryWebViewBridge initialization
val windowManager = WindowManager(context)
val supported = windowManager.isSplitSupported()
```

**Handling - Option A (Recommended):**
- Detect at app startup via WindowManager.isSplitSupported()
- If unsupported: Show dialog once explaining requirement
- Disable browser UI entirely (hide browser tab/menu item)
- Keep other app features functional

**Handling - Option B (Graceful Degradation):**
- Fall back to full-screen WebView mode
- Hide tab bar when WebView is active
- Add "Back to tabs" button to return to tab selection UI
- Similar to traditional mobile browser UX

**Implementation:**
```rust
#[cfg(target_os = "android")]
fn check_activity_embedding_support() -> bool {
    // JNI call to WindowManager.isSplitSupported()
    // Return false if API < 31 or not supported
}

// In DureSijangApp initialization
#[cfg(target_os = "android")]
let browser_available = check_activity_embedding_support();
```

#### 4. WebView Navigation Failure

**Scenario:** URL is invalid, unreachable, or DNS fails

**Handling:**
- wry WebView shows built-in error page (no custom handling needed)
- Log navigation error: `log::warn!("Navigation failed to {}: {}", url, error)`
- Don't destroy WebView or close tab
- User can edit URL and retry
- Error page provides standard browser error UX

#### 5. Tab Database Corruption

**Scenario:** SQLite read fails or returns malformed data

**Detection:**
```rust
match db_browser::get_all_tabs() {
    Ok(tabs) => { /* process tabs */ }
    Err(e) => {
        log::error!("Failed to load tabs from DB: {}", e);
        // Recovery here
    }
}
```

**Handling:**
- Log error with stack trace
- Fall back to empty browser state (no tabs)
- Re-initialize with clean state: `BrowserState::new()`
- Create 2 default tabs as if first launch
- Don't block app startup
- User data (bookmarks, history) may be lost but app works

#### 6. Memory Pressure (Too Many Tabs)

**Scenario:** User creates 50+ tabs, system runs out of memory

**Prevention:**
```rust
const MAX_TABS: usize = 20;

if self.browser_state.tabs.len() >= MAX_TABS {
    // Disable "+" button
    ui.add_enabled(false, MaterialButton::filled("+").small())
        .on_disabled_hover_text("Maximum 20 tabs. Close some tabs to create new ones.");
    return;
}
```

**Handling:**
- Set reasonable limit: 20 tabs max
- When limit reached:
  - Disable "+" button
  - Show tooltip: "Maximum 20 tabs. Close some tabs to create new ones."
- When approaching limit (15+ tabs):
  - Show warning banner: "You have many tabs open. Consider closing unused tabs."
- Protect against memory exhaustion

#### 7. WebViewActivity Destroyed by System

**Scenario:** Android kills WebViewActivity to reclaim memory (low memory condition)

**Detection:**
```kotlin
// In WebViewActivity
override fun onDestroy() {
    super.onDestroy()
    // Notify Rust via JNI callback
    WryWebViewBridge.notifyDestroyed(tabId)
}
```

**Handling:**
- Detect via JNI callback from onDestroy
- Update Rust state: WebView no longer exists for this tab
- UI shows placeholder in WebView area: "Tab suspended - tap to reload"
- On user interaction (tap placeholder or switch to tab):
  - Recreate WebViewActivity with same URL
  - Resume normal operation
- Keep tab metadata intact (URL, title, bookmarks)

### Error Logging Strategy

**All errors logged with context:**
```rust
log::error!("Failed to launch WebViewActivity for tab {}: {}", tab_id, e);
log::warn!("Navigation to {} failed, showing error page", url);
log::info!("WebViewActivity destroyed for tab {} (system reclaim)", tab_id);
```

**Levels:**
- `error!` - Operation failed, user impacted
- `warn!` - Degraded behavior, user might notice
- `info!` - Normal operation events for debugging

**User-Facing Messages:**
- Toast notifications for recoverable errors
- Dialogs for critical errors requiring user action
- Tooltips for disabled features explaining why

## Testing Strategy

### Manual Testing Checklist

#### Phase 1: Basic Functionality
- [ ] App launches successfully on Android device/emulator
- [ ] 2 tabs appear in tab bar automatically on first launch
- [ ] Both tabs show "dure.app" as title
- [ ] Tab bar visible at top (~30% of screen)
- [ ] WebView visible at bottom (~70% of screen)
- [ ] Can see both tab UI and web content simultaneously
- [ ] First tab is highlighted as active
- [ ] URL input shows "https://dure.app"
- [ ] Web page renders correctly in WebView

#### Phase 2: Tab Management
- [ ] Click "+" button creates new tab
- [ ] New tab loads https://dure.app by default
- [ ] New tab becomes active automatically (highlighted)
- [ ] Can switch between tabs by clicking in tab bar
- [ ] Switching tabs shows different content (verify by navigating tabs to different URLs first)
- [ ] Old WebView is destroyed when switching (verify in logcat)
- [ ] New WebView is created for newly active tab
- [ ] Tab content persists when switching back (URL is preserved)
- [ ] Click "×" on tab closes it
- [ ] Active tab adjusts correctly when closing tabs
- [ ] Closing last tab shows empty state: "No tabs open" message
- [ ] Empty state shows "+" button to create first tab

#### Phase 3: Navigation
- [ ] Type URL in input field and press Enter
- [ ] WebView navigates to entered URL
- [ ] Click "Go" button navigates to URL
- [ ] URL input updates when switching tabs (shows active tab's URL)
- [ ] Can navigate to http:// URLs
- [ ] Can navigate to https:// URLs
- [ ] URLs without protocol auto-add https://
- [ ] Invalid URLs show error page in WebView (not crash)
- [ ] Error page allows retry (can edit URL and navigate again)

#### Phase 4: WebView Functionality
- [ ] Web pages render correctly (HTML, CSS)
- [ ] Can scroll web content vertically
- [ ] Can scroll web content horizontally (if needed)
- [ ] Can click links (navigate within WebView)
- [ ] JavaScript executes correctly (test with interactive sites)
- [ ] Images load and display
- [ ] Videos can be played (if applicable)
- [ ] Form inputs work (can type in web forms)
- [ ] Form submission works
- [ ] Back button in tab bar navigates WebView history (desktop only for now)

#### Phase 5: Bookmarks & History
- [ ] Click bookmark button (⭐) saves current page
- [ ] Bookmarks appear in sidebar under "Bookmarks" tab
- [ ] Click bookmark in sidebar opens URL in current tab
- [ ] Bookmark shows correct title and URL
- [ ] Switch to "History" tab in sidebar
- [ ] History shows all navigations with timestamps
- [ ] Click history entry opens URL in current tab (or new tab based on settings)
- [ ] History persists across app restarts

#### Phase 6: Error Scenarios
- [ ] Navigate to invalid URL (e.g., "notaurl") - shows error, doesn't crash
- [ ] Navigate to unreachable URL (e.g., "https://thisdomaindoesnotexist123.com") - shows error page
- [ ] Force-stop WebViewActivity from Android settings - app handles gracefully
- [ ] Rapidly create 10 tabs in a row - all created successfully, no crashes
- [ ] Create 20 tabs (limit) - "+" button becomes disabled
- [ ] Tooltip on disabled "+" explains limit
- [ ] Close a tab to get below limit - "+" button re-enables
- [ ] Rapidly switch between tabs - WebViews created/destroyed correctly
- [ ] Rotate device - tab state persists, WebView recreates
- [ ] Put app in background and return - tabs restored correctly
- [ ] Kill app from recent apps and restart - tabs restored from database

#### Phase 7: Platform Differences
- [ ] Desktop: Works with existing egui_webview (no changes)
- [ ] Desktop: Can still create, switch, close tabs
- [ ] Android API 31+: Activity Embedding works as designed
- [ ] Android API < 31: Shows unsupported message OR falls back gracefully
- [ ] Android API < 31: App doesn't crash, other features work

#### Phase 8: Activity Embedding Specific
- [ ] Split screen shows correct ratio (30/70)
- [ ] Tab bar is always visible even when scrolling web content
- [ ] Tab bar is always interactive (can switch tabs while page loads)
- [ ] WebView area updates immediately when switching tabs
- [ ] Both activities remain responsive (no blocking)
- [ ] Switching tabs feels smooth (acceptable delay)
- [ ] Memory usage is reasonable (check Android profiler)

### Automated Testing (Future Enhancements)

#### Unit Tests

**browser_stt.rs:**
```rust
#[test]
fn test_add_tab_creates_metadata() {
    let mut state = BrowserState::new();
    let tab_id = state.add_tab("https://example.com", "Example");
    
    assert_eq!(state.tabs.len(), 1);
    assert_eq!(state.tabs[0].url_bar, "https://example.com");
    assert_eq!(state.tabs[0].title, "Example");
    assert_eq!(state.active_tab_index, Some(0));
}

#[test]
fn test_close_tab_updates_indices() {
    let mut state = BrowserState::new();
    state.add_tab("https://a.com", "A");
    state.add_tab("https://b.com", "B");
    state.add_tab("https://c.com", "C");
    
    state.close_tab(1); // Close middle tab
    
    assert_eq!(state.tabs.len(), 2);
    assert_eq!(state.tabs[0].url_bar, "https://a.com");
    assert_eq!(state.tabs[1].url_bar, "https://c.com");
}
```

**dure_sijang_app.rs:**
```rust
#[test]
fn test_ensure_protocol_adds_https() {
    assert_eq!(DureSijangApp::ensure_protocol("example.com"), "https://example.com");
    assert_eq!(DureSijangApp::ensure_protocol("https://example.com"), "https://example.com");
    assert_eq!(DureSijangApp::ensure_protocol("http://example.com"), "http://example.com");
}
```

#### Integration Tests

**JNI Bridge Mock Tests:**
```rust
#[cfg(test)]
mod android_activity_embedding_tests {
    #[test]
    fn test_launch_webview_with_mocked_jni() {
        // Mock JNI calls
        // Verify correct method signatures used
        // Verify parameters passed correctly
    }
}
```

**Database Tests:**
```rust
#[test]
fn test_create_and_load_tabs_from_db() {
    // Create temporary database
    // Create tabs
    // Load tabs
    // Verify data matches
}
```

### Performance Testing

**Metrics to measure:**
- Tab creation time (should be < 500ms)
- Tab switching time (should be < 300ms)
- Memory usage per tab (should be reasonable)
- WebView initialization time
- App startup time with 10 saved tabs

### Acceptance Criteria

#### Must Have (P0) - Blocking Release

1. ✅ Android shows tab UI and WebView simultaneously on same screen
2. ✅ Can create new tabs with "+" button on Android
3. ✅ Can switch between tabs by clicking tab bar
4. ✅ Can close tabs with "×" button
5. ✅ URL navigation works (Enter key and Go button)
6. ✅ Initial launch shows 2 tabs to https://dure.app automatically
7. ✅ No crashes during normal tab operations
8. ✅ Desktop functionality remains unchanged (no regressions)

#### Should Have (P1) - Important for UX

9. ✅ Bookmarks work (save and restore)
10. ✅ History works (record and display)
11. ✅ Graceful error handling for common failures
12. ✅ Tab limit prevents memory exhaustion
13. ✅ WebView content persists when switching tabs (same URL)
14. ✅ Works on Android 12L+ devices (API 31+)

#### Nice to Have (P2) - Future Enhancements

15. ⚠️ Fallback mode for Android < 12L (graceful degradation)
16. ⚠️ Tab state fully persists across app restarts
17. ⚠️ Back/Forward buttons work on Android
18. ⚠️ Loading indicators for slow pages
19. ⚠️ Pull-to-refresh in WebView

### Known Limitations

**Platform Requirements:**
- ❗ Requires Android 12L+ (API 31) for Activity Embedding
- ❗ Devices with API < 31 will show unsupported message or use fallback

**Functional Limitations:**
- ⚠️ Only one WebView active at a time (others destroyed)
- ⚠️ Tab switching has slight delay (~200-300ms for destroy/create)
- ⚠️ Cannot keep multiple WebViews loaded simultaneously (memory tradeoff)
- ⚠️ Back/Forward navigation buttons desktop-only initially (Android TBD)

**UX Differences from Desktop:**
- Desktop: Multiple WebViews can exist simultaneously, instant switching
- Android: Only active tab has WebView, switching recreates WebView
- Rationale: Memory efficiency on mobile devices

## Implementation Phases

### Phase 1: Android Infrastructure (Kotlin/Java)
**Goal:** Set up Activity Embedding and JNI bridge

**Tasks:**
1. Create `WebViewActivity.kt` with wry WebView integration
2. Create `WryWebViewBridge.kt` with static JNI methods
3. Create `main_split_config.xml` with 30/70 split rules
4. Update `AndroidManifest.xml` with activity and property
5. Update `build.gradle` with WindowManager dependency
6. Test: Launch WebViewActivity manually, verify split screen works

### Phase 2: Rust JNI Wrapper
**Goal:** Create safe Rust API for calling Kotlin bridge

**Tasks:**
1. Create `android_activity_embedding.rs` module
2. Implement `launch_webview_activity(url, tab_id)` with JNI calls
3. Implement `destroy_webview_activity()` with JNI calls
4. Implement `navigate_webview(url)` with JNI calls
5. Add error handling and logging
6. Test: Call from Rust test harness, verify JNI bridge works

### Phase 3: Browser UI Cleanup
**Goal:** Remove Android platform guards from browser UI

**Tasks:**
1. Remove `#[cfg(not(target_os = "android"))]` from "+" button
2. Remove guards from tab creation UI elements
3. Keep guards for desktop-specific egui_webview rendering
4. Test: Verify UI elements visible on Android (may not function yet)

### Phase 4: Tab Creation Integration
**Goal:** Wire up "+" button and initial tabs for Android

**Tasks:**
1. Modify `add_browser_tab()` to call `launch_webview_activity()` on Android
2. Add `initialize_browser()` method to create 2 default tabs
3. Call `initialize_browser()` from `DureSijangApp::new()`
4. Add first `update()` logic to launch initial WebView
5. Test: App launches with 2 tabs, clicking "+" creates new tab

### Phase 5: Tab Switching & Closing
**Goal:** Implement destroy/recreate pattern for tab switching

**Tasks:**
1. Modify tab click handler to call destroy + launch
2. Modify tab close handler to destroy WebView
3. Update `close_browser_tab()` for Android path
4. Handle edge cases (last tab, active tab closed)
5. Test: Switch tabs, close tabs, verify WebViews created/destroyed

### Phase 6: URL Navigation
**Goal:** Make Go button and Enter key work on Android

**Tasks:**
1. Verify `navigate_webview()` is called in Go button handler
2. Verify `navigate_webview()` is called in Enter key handler
3. Ensure `ensure_protocol()` is applied to URLs
4. Update history after navigation
5. Test: Enter URL and navigate, click Go button

### Phase 7: Testing & Polish
**Goal:** Verify all functionality, fix bugs, handle errors

**Tasks:**
1. Run through manual testing checklist (all phases)
2. Fix any bugs discovered
3. Add error handling for edge cases
4. Test on multiple Android versions (if possible)
5. Test with slow network (verify loading states)
6. Verify no memory leaks (Android profiler)
7. Document known limitations

### Phase 8: Documentation
**Goal:** Update docs and provide usage guidance

**Tasks:**
1. Update CLAUDE.md with new architecture
2. Add troubleshooting guide for common issues
3. Document Android version requirements
4. Update README with screenshots/demo

## Success Metrics

**Quantitative:**
- 0 crashes during normal tab operations
- < 500ms tab creation time
- < 300ms tab switching time
- Works on 100% of Android 12L+ devices tested
- 20 tab limit prevents memory issues

**Qualitative:**
- Tab UI and WebView both visible simultaneously ✅
- UX feels responsive and smooth
- Error messages are clear and actionable
- Desktop functionality unchanged (no regressions)
- Code is maintainable and well-documented

## Appendix

### Alternative Approaches Considered

**Approach 1: Single Activity with Native View Positioning**
- Use JNI to manually position wry WebView within window bounds
- Pros: Simpler than Activity Embedding
- Cons: Requires low-level JNI for View.layout(), fragile, doesn't use Android's window management

**Approach 2: Full-Screen WebView Toggle**
- Show either tab bar OR WebView, not both simultaneously
- Pros: Simple, works on all Android versions
- Cons: Doesn't match desktop UX, more clicks to navigate

**Approach 3: Single Active WebView (Current)**
- Desktop uses egui_webview (multiple simultaneous)
- Android uses Activity Embedding (one active WebView)
- Pros: Matches desktop UX, uses platform features correctly
- Cons: Requires Android 12L+, more complex implementation

**Decision:** Approach 3 chosen for best UX and proper platform integration

### Technical Debt

**Current compromises:**
1. Android-specific code in Kotlin/Java (not pure Rust)
2. JNI bridge adds complexity
3. Only one WebView at a time (vs desktop's multiple)
4. Activity Embedding requires API 31+

**Future improvements:**
1. Implement fallback mode for older Android versions
2. Cache WebView state for faster tab switching
3. Preload adjacent tabs in background (if memory allows)
4. Add pull-to-refresh gesture
5. Implement back/forward navigation on Android

### References

- [Android Activity Embedding Guide](https://developer.android.com/guide/topics/large-screens/activity-embedding)
- [Jetpack WindowManager Documentation](https://developer.android.com/jetpack/androidx/releases/window)
- [wry WebView Documentation](https://docs.rs/wry/latest/wry/)
- [JNI in Rust Documentation](https://docs.rs/jni/latest/jni/)
