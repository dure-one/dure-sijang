use dure_sijang::browser_stt::BrowserState;
use dure_sijang::db_browser;

/// Helper: Clear all tabs from database
fn clear_all_tabs() {
    if let Ok(tabs) = db_browser::get_all_tabs() {
        for tab in tabs {
            if let Some(id) = tab.id {
                let _ = db_browser::delete_tab(id);
            }
        }
    }
}

/// Test: Load tabs from database
#[test]
fn test_load_tabs_from_database() {
    clear_all_tabs();

    // Arrange: Create 3 tabs in database
    let tab1 = db_browser::create_tab("Tab 1", "https://example.com/1", "webview");
    let tab2 = db_browser::create_tab("Tab 2", "https://example.com/2", "webview");
    let tab3 = db_browser::create_tab("Tab 3", "https://example.com/3", "webview");

    assert!(tab1.is_ok(), "Failed to create tab1");
    assert!(tab2.is_ok(), "Failed to create tab2");
    assert!(tab3.is_ok(), "Failed to create tab3");

    // Act: Load from database
    let state = BrowserState::load_from_db();

    // Assert: 3 tabs loaded
    assert_eq!(state.tabs.len(), 3, "Should load 3 tabs from database");
    assert_eq!(
        state.active_tab_index,
        Some(0),
        "First tab should be active"
    );
    assert_eq!(
        state.url_input, "https://example.com/1",
        "URL bar should show first tab URL"
    );

    // Verify tab data
    assert_eq!(state.tabs[0].url_bar, "https://example.com/1");
    assert_eq!(state.tabs[0].title, "Tab 1");
    assert_eq!(state.tabs[1].url_bar, "https://example.com/2");
    assert_eq!(state.tabs[2].url_bar, "https://example.com/3");

    // Cleanup
    for tab in &state.tabs {
        if let Some(db_id) = tab.db_id {
            let _ = db_browser::delete_tab(db_id);
        }
    }
}

/// Test: Empty database returns empty state (defaults created by initialize_browser)
#[test]
fn test_empty_database_loads_empty_state() {
    clear_all_tabs();

    // Act: Load from empty database
    let state = BrowserState::load_from_db();

    // Assert: No tabs loaded
    assert_eq!(
        state.tabs.len(),
        0,
        "Should load 0 tabs from empty database"
    );
    assert_eq!(state.active_tab_index, None, "No active tab when empty");
    assert!(state.url_input.is_empty(), "URL bar should be empty");
}

/// Test: Session restoration preserves tab order
#[test]
fn test_tab_order_preserved() {
    clear_all_tabs();

    // Arrange: Create tabs in specific order
    let _ = db_browser::create_tab("First", "https://first.com", "webview");
    std::thread::sleep(std::time::Duration::from_millis(10)); // Ensure different created_at
    let _ = db_browser::create_tab("Second", "https://second.com", "webview");
    std::thread::sleep(std::time::Duration::from_millis(10));
    let _ = db_browser::create_tab("Third", "https://third.com", "webview");

    // Act: Load from database
    let state = BrowserState::load_from_db();

    // Assert: Order preserved
    assert_eq!(state.tabs.len(), 3);
    assert_eq!(state.tabs[0].title, "First");
    assert_eq!(state.tabs[1].title, "Second");
    assert_eq!(state.tabs[2].title, "Third");

    // Cleanup
    for tab in &state.tabs {
        if let Some(db_id) = tab.db_id {
            let _ = db_browser::delete_tab(db_id);
        }
    }
}
