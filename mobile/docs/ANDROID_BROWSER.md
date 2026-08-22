# Android Browser with Activity Embedding

## Overview

The Android browser uses Activity Embedding to display egui browser controls and wry WebView simultaneously in a split-screen layout.

## Architecture

- **NativeActivity (30% top)**: egui UI with tab bar, URL input, navigation controls
- **WebViewActivity (70% bottom)**: wry WebView rendering web content
- **JNI Bridge**: Rust ↔ Kotlin communication via `android_activity_embedding` module

## Requirements

- **Minimum Android SDK**: API 31 (Android 12L)
- **Dependency**: `androidx.window:window:1.2.0`
- **Pattern**: Single active WebView (destroy old, create new on tab switch)

## Components

### Kotlin Components

**Files:**
- `app/src/main/java/app/dure/sijang/WebViewActivity.java` - Hosts wry WebView
- `app/src/main/java/app/dure/sijang/WryWebViewBridge.java` - JNI bridge methods
- `app/src/main/res/xml/main_split_config.xml` - Activity Embedding config

**Methods:**
- `WryWebViewBridge.launchWebView(activity, url, tabId)` - Create WebViewActivity
- `WryWebViewBridge.destroyWebView()` - Finish current WebViewActivity
- `WryWebViewBridge.navigateWebView(url)` - Navigate to new URL

### Rust Components

**Files:**
- `mobile/src/android_activity_embedding.rs` - JNI wrapper functions

**Functions:**
```rust
pub fn launch_webview_activity(url: &str, tab_id: i32) -> anyhow::Result<()>
pub fn destroy_webview_activity() -> anyhow::Result<()>
pub fn navigate_webview(url: &str) -> anyhow::Result<()>
```

## User Flows

### App Launch
1. `DureSijangApp::new()` calls `initialize_browser()`
2. Creates 2 default tabs to https://dure.app (metadata only)
3. First `update()` launches WebViewActivity for tab 0
4. User sees tab bar (top) + WebView (bottom) simultaneously

### Create Tab
1. User clicks "+" button
2. `add_browser_tab()` creates tab metadata
3. Android: Destroys old WebViewActivity, launches new one
4. New tab becomes active, WebView loads https://dure.app

### Switch Tab
1. User clicks different tab in tab bar
2. Android: Destroys current WebViewActivity
3. Launches WebViewActivity for clicked tab's URL
4. Updates active_tab_index and url_input

### Navigate URL
1. User types URL and presses Enter (or clicks Go)
2. Updates tab metadata
3. Android: Calls `navigate_webview(url)` via JNI
4. Kotlin: `currentActivity.loadUrl(url)`
5. WebView navigates to new page

### Close Tab
1. User clicks "×" on tab
2. If active tab: Destroys WebViewActivity
3. Removes tab metadata
4. Calculates new active tab (adjacent tab)
5. Launches WebViewActivity for new active tab

## Constraints

- **Max Tabs**: 20 (prevents memory issues)
- **Single WebView**: Only one WebViewActivity exists at a time
- **Split Ratio**: 30% controls, 70% WebView (configured in main_split_config.xml)

## Error Handling

### WebView Launch Failure
- Logs error
- Removes tab metadata
- Shows toast: "Failed to open tab"
- App remains functional

### JNI Errors
- Logs detailed error with class/method name
- Disables browser functionality
- Shows one-time warning dialog
- Other app features remain usable

### Tab Limit Reached
- Disables "+" button
- Shows tooltip: "Maximum 20 tabs. Close some tabs to create new ones."
- Re-enables when tabs < 20

## Testing

### Manual Test Checklist
- [ ] App launches with 2 tabs visible
- [ ] Both tab bar and WebView visible simultaneously
- [ ] Can create new tabs (+ button works)
- [ ] Can switch between tabs (content changes)
- [ ] Can navigate URLs (Enter and Go button)
- [ ] Can close tabs (× button works)
- [ ] Tab limit enforced (+ disabled at 20 tabs)
- [ ] Error scenarios handled gracefully

### Logcat Monitoring
```bash
adb logcat | grep -E "WryWebViewBridge|WebViewActivity|dure_sijang"
```

Watch for:
- "launchWebView: url=..." - WebView launch
- "destroyWebView called" - WebView destruction
- "Successfully launched WebViewActivity" - Success
- "Failed to launch WebViewActivity" - Errors

## Development

### Building for Android
```bash
cd deploy
./gradlew assembleDebug
adb install ../app/build/outputs/apk/debug/app-debug.apk
```

### Viewing Logs
```bash
adb logcat -s WryWebViewBridge WebViewActivity RustStdoutStderr
```

### Testing Activity Embedding
Check if device supports Activity Embedding:
```bash
adb shell getprop ro.build.version.sdk
# Should return >= 31 for Android 12L+
```

## Troubleshooting

### WebView doesn't appear
- Check logcat for "Failed to launch WebViewActivity"
- Verify AndroidManifest.xml has WebViewActivity declaration
- Verify PROPERTY_ACTIVITY_EMBEDDING_SPLITS_ENABLED is set

### Split screen not working
- Verify device is Android 12L+ (API 31+)
- Check main_split_config.xml splitRatio is 0.3
- Ensure window dependency is added to build.gradle

### Tab switching crashes app
- Check if destroy_webview_activity() is called before launch
- Verify JNI bridge methods are working (check logcat)
- Ensure only one WebViewActivity exists at a time

## References

- [Activity Embedding Documentation](https://developer.android.com/guide/topics/large-screens/activity-embedding)
- [wry WebView](https://docs.rs/wry/0.56.0/wry/)
- Design Spec: `docs/superpowers/specs/2026-08-23-browser-ui-fixes-design.md`
- Implementation Plan: `docs/superpowers/plans/2026-08-23-android-browser-activity-embedding.md`
