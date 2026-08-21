//! Comprehensive integration tests for WebView integration
//!
//! Tests the complete MVVM Command/Event flow:
//! User action → Command → Actor → Event → State update → UI update

use dure_sijang::models::browser::{BrowsingMode, TabMetadata};
use dure_sijang::viewmodel::{BrowserActor, BrowserCommand, BrowserEvent, ViewModelEvent};
use smol::channel::unbounded;

// =============================================================================
// Tab Lifecycle Tests
// =============================================================================

#[test]
fn test_create_tab_creates_webview_metadata() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_create_tab_webview.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act - send CreateTab command with WebView mode
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        // Assert - TabCreated event emitted
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated {
                tab_id,
                store_url,
                mode,
            }) => {
                assert_eq!(tab_id, 1); // First tab should have ID 1
                assert_eq!(store_url, "https://test.mycart");
                assert!(matches!(mode, BrowsingMode::WebView));
            }
            _ => panic!("Expected TabCreated event, got {:?}", event),
        }
    });
}

#[test]
fn test_create_multiple_tabs_increments_id() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_multiple_tabs.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act - create 3 tabs
        for i in 1..=3 {
            cmd_tx
                .send(BrowserCommand::CreateTab {
                    store_url: format!("https://store{}.mycart", i),
                    mode: BrowsingMode::WebView,
                })
                .await
                .unwrap();
        }

        // Assert - tab IDs should be 1, 2, 3
        for expected_id in 1..=3 {
            let event = event_rx.recv().await.unwrap();
            match event {
                ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => {
                    assert_eq!(tab_id, expected_id);
                }
                _ => panic!("Expected TabCreated event"),
            }
        }
    });
}

#[test]
fn test_navigate_updates_url_in_state() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_navigate_url.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create a tab first
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        // Wait for TabCreated event
        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated event"),
        };

        // Act - update current URL (simulating webview navigation)
        cmd_tx
            .send(BrowserCommand::UpdateCurrentUrl {
                tab_id,
                url: "https://test.mycart/products".to_string(),
            })
            .await
            .unwrap();

        // Assert - UrlChanged event emitted
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::UrlChanged { tab_id: id, url }) => {
                assert_eq!(id, tab_id);
                assert_eq!(url, "https://test.mycart/products");
            }
            _ => panic!("Expected UrlChanged event, got {:?}", event),
        }
    });
}

#[test]
fn test_failed_webview_creation_sets_error() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_failed_webview.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create a tab
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        // Wait for TabCreated
        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Act - mark tab as failed (simulating webview creation error)
        cmd_tx
            .send(BrowserCommand::MarkTabFailed {
                tab_id,
                error: "No window handle available".to_string(),
            })
            .await
            .unwrap();

        // Assert - TabFailed event emitted
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TabFailed { tab_id: id, error }) => {
                assert_eq!(id, tab_id);
                assert_eq!(error, "No window handle available");
            }
            _ => panic!("Expected TabFailed event, got {:?}", event),
        }
    });
}

#[test]
fn test_close_tab_destroys_webview() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_close_tab.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create a tab
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        // Wait for TabCreated
        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Act - close the tab
        cmd_tx
            .send(BrowserCommand::CloseTab { tab_id })
            .await
            .unwrap();

        // Assert - TabClosed event emitted
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TabClosed { tab_id: id }) => {
                assert_eq!(id, tab_id);
            }
            _ => panic!("Expected TabClosed event, got {:?}", event),
        }
    });
}

// =============================================================================
// Navigation Flow Tests
// =============================================================================

#[test]
fn test_navigation_state_synchronization() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_navigation_sync.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create tab
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Act - simulate navigation sequence (like webview.go_back() triggering URL change)
        let urls = vec![
            "https://test.mycart/products",
            "https://test.mycart/products/123",
            "https://test.mycart/cart",
        ];

        for url in &urls {
            cmd_tx
                .send(BrowserCommand::UpdateCurrentUrl {
                    tab_id,
                    url: url.to_string(),
                })
                .await
                .unwrap();
        }

        // Assert - verify each URL change emitted event
        for expected_url in &urls {
            let event = event_rx.recv().await.unwrap();
            match event {
                ViewModelEvent::Browser(BrowserEvent::UrlChanged { url, .. }) => {
                    assert_eq!(&url, expected_url);
                }
                _ => panic!("Expected UrlChanged event"),
            }
        }
    });
}

#[test]
fn test_navigation_controls_update_title() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_title_update.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create tab
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Act - update title (simulating webview document.title change)
        cmd_tx
            .send(BrowserCommand::UpdateTitle {
                tab_id,
                title: "Product Catalog".to_string(),
            })
            .await
            .unwrap();

        // Assert - TitleChanged event emitted
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TitleChanged { tab_id: id, title }) => {
                assert_eq!(id, tab_id);
                assert_eq!(title, "Product Catalog");
            }
            _ => panic!("Expected TitleChanged event, got {:?}", event),
        }
    });
}

// =============================================================================
// Error Handling Tests
// =============================================================================

#[test]
fn test_error_recovery_flow() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_error_recovery.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create tab
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Act - mark tab as failed
        cmd_tx
            .send(BrowserCommand::MarkTabFailed {
                tab_id,
                error: "Connection timeout".to_string(),
            })
            .await
            .unwrap();

        // Wait for TabFailed event
        let event = event_rx.recv().await.unwrap();
        assert!(matches!(
            event,
            ViewModelEvent::Browser(BrowserEvent::TabFailed { .. })
        ));

        // Act - retry by clearing error (empty string clears error)
        cmd_tx
            .send(BrowserCommand::MarkTabFailed {
                tab_id,
                error: "".to_string(),
            })
            .await
            .unwrap();

        // Assert - TabFailed event with empty error (signals retry)
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TabFailed { error, .. }) => {
                assert_eq!(error, "");
            }
            _ => panic!("Expected TabFailed event with empty error"),
        }
    });
}

#[test]
fn test_multiple_tabs_with_mixed_states() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_mixed_states.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act - create 3 tabs
        for i in 1..=3 {
            cmd_tx
                .send(BrowserCommand::CreateTab {
                    store_url: format!("https://store{}.mycart", i),
                    mode: BrowsingMode::WebView,
                })
                .await
                .unwrap();
        }

        // Wait for all TabCreated events
        let mut tab_ids = vec![];
        for _ in 1..=3 {
            let event = event_rx.recv().await.unwrap();
            match event {
                ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => {
                    tab_ids.push(tab_id);
                }
                _ => panic!("Expected TabCreated"),
            }
        }

        // Act - fail tab 1, navigate tab 2, update title on tab 3
        cmd_tx
            .send(BrowserCommand::MarkTabFailed {
                tab_id: tab_ids[0],
                error: "Failed to load".to_string(),
            })
            .await
            .unwrap();

        cmd_tx
            .send(BrowserCommand::UpdateCurrentUrl {
                tab_id: tab_ids[1],
                url: "https://store2.mycart/products".to_string(),
            })
            .await
            .unwrap();

        cmd_tx
            .send(BrowserCommand::UpdateTitle {
                tab_id: tab_ids[2],
                title: "Store 3".to_string(),
            })
            .await
            .unwrap();

        // Assert - verify events for each tab
        let mut events_received = 0;
        for _ in 0..3 {
            let event = event_rx.recv().await.unwrap();
            match event {
                ViewModelEvent::Browser(BrowserEvent::TabFailed { tab_id, error }) => {
                    assert_eq!(tab_id, tab_ids[0]);
                    assert_eq!(error, "Failed to load");
                    events_received += 1;
                }
                ViewModelEvent::Browser(BrowserEvent::UrlChanged { tab_id, url }) => {
                    assert_eq!(tab_id, tab_ids[1]);
                    assert_eq!(url, "https://store2.mycart/products");
                    events_received += 1;
                }
                ViewModelEvent::Browser(BrowserEvent::TitleChanged { tab_id, title }) => {
                    assert_eq!(tab_id, tab_ids[2]);
                    assert_eq!(title, "Store 3");
                    events_received += 1;
                }
                _ => panic!("Unexpected event: {:?}", event),
            }
        }
        assert_eq!(events_received, 3);
    });
}

// =============================================================================
// End-to-End MVVM Pattern Tests
// =============================================================================

#[test]
fn test_complete_tab_lifecycle_mvvm_pattern() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_complete_lifecycle.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act - complete lifecycle: Create → Navigate → Update Title → Close
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Navigate
        cmd_tx
            .send(BrowserCommand::UpdateCurrentUrl {
                tab_id,
                url: "https://test.mycart/products".to_string(),
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        assert!(matches!(
            event,
            ViewModelEvent::Browser(BrowserEvent::UrlChanged { .. })
        ));

        // Update title
        cmd_tx
            .send(BrowserCommand::UpdateTitle {
                tab_id,
                title: "Products".to_string(),
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        assert!(matches!(
            event,
            ViewModelEvent::Browser(BrowserEvent::TitleChanged { .. })
        ));

        // Close tab
        cmd_tx
            .send(BrowserCommand::CloseTab { tab_id })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        assert!(matches!(
            event,
            ViewModelEvent::Browser(BrowserEvent::TabClosed { .. })
        ));
    });
}

#[test]
fn test_rapid_command_sequence() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_rapid_commands.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create tab
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Act - send rapid sequence of URL updates (simulating fast navigation)
        for i in 0..10 {
            cmd_tx
                .send(BrowserCommand::UpdateCurrentUrl {
                    tab_id,
                    url: format!("https://test.mycart/page{}", i),
                })
                .await
                .unwrap();
        }

        // Assert - all events should be received in order
        for i in 0..10 {
            let event = event_rx.recv().await.unwrap();
            match event {
                ViewModelEvent::Browser(BrowserEvent::UrlChanged { url, .. }) => {
                    assert_eq!(url, format!("https://test.mycart/page{}", i));
                }
                _ => panic!("Expected UrlChanged event"),
            }
        }
    });
}

// =============================================================================
// State Consistency Tests
// =============================================================================

#[test]
fn test_state_consistency_after_error() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_state_consistency.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Create tab
        cmd_tx
            .send(BrowserCommand::CreateTab {
                store_url: "https://test.mycart".to_string(),
                mode: BrowsingMode::WebView,
            })
            .await
            .unwrap();

        let event = event_rx.recv().await.unwrap();
        let tab_id = match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated { tab_id, .. }) => tab_id,
            _ => panic!("Expected TabCreated"),
        };

        // Act - set error, then try to update URL (should still work)
        cmd_tx
            .send(BrowserCommand::MarkTabFailed {
                tab_id,
                error: "Temporary error".to_string(),
            })
            .await
            .unwrap();

        let _ = event_rx.recv().await.unwrap(); // TabFailed event

        cmd_tx
            .send(BrowserCommand::UpdateCurrentUrl {
                tab_id,
                url: "https://test.mycart/products".to_string(),
            })
            .await
            .unwrap();

        // Assert - URL update should still work even when tab has error
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::UrlChanged { url, .. }) => {
                assert_eq!(url, "https://test.mycart/products");
            }
            _ => panic!("Expected UrlChanged event, got {:?}", event),
        }
    });
}
