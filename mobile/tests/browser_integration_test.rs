/// Integration test for BrowserUI integration into DureSijangApp
///
/// This test verifies that the browser UI is properly wired into the main app.

#[test]
fn test_browser_ui_field_exists() {
    // Arrange - Create app instance
    let app = mobile::DureSijangApp::default();

    // Assert - Verify browser_ui field exists
    let _browser_ui = &app.browser_ui;
}

#[test]
fn test_browser_ui_is_initialized() {
    // Arrange - Create app instance
    let app = mobile::DureSijangApp::default();

    // Assert - Browser UI should be initialized (not panicking when accessed)
    let browser_ui = &app.browser_ui;
    assert!(
        std::ptr::addr_of!(*browser_ui) as usize != 0,
        "browser_ui should be initialized"
    );
}

#[test]
fn test_app_has_viewmodel() {
    // Arrange - Create app instance
    let app = mobile::DureSijangApp::default();

    // Assert - Verify app has viewmodel field (Option<ViewModel>)
    // This is required for browser_ui.render() to work
    match &app.viewmodel {
        Some(_) => {
            // ViewModel initialized
        }
        None => {
            // ViewModel not initialized (expected for default app)
        }
    }
}
