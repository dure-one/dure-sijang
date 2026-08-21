// Minimal integration test to verify wry, gtk, and raw-window-handle dependencies
// This test is independent of the main library compilation state
//
// Task 1: Add Dependencies for wry WebView integration

#[test]
fn test_wry_available() {
    // Compile-time check: wry::WebView type exists
    use wry::WebView;

    // If this compiles, wry dependency is correctly added
    let _phantom: Option<WebView> = None;
}

#[cfg(any(target_os = "linux", target_os = "openbsd", target_os = "freebsd"))]
#[test]
fn test_gtk_available_on_linux() {
    // Compile-time check: gtk::Application type exists on Linux/OpenBSD
    use gtk::Application;

    // If this compiles on Linux/OpenBSD, gtk dependency is correctly added
    let _phantom: Option<Application> = None;
}

#[test]
fn test_raw_window_handle_available() {
    // Compile-time check: raw-window-handle types exist
    use raw_window_handle::RawWindowHandle;

    // If this compiles, raw-window-handle dependency is correctly added
    let _phantom: Option<RawWindowHandle> = None;
}

#[test]
fn test_wry_webview_builder() {
    // Verify we can import WebViewBuilder
    use wry::WebViewBuilder;

    // If this compiles, wry::WebViewBuilder is accessible
    let _phantom: Option<WebViewBuilder> = None;
}
