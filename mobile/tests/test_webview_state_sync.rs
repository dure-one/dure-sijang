//! Integration tests for WebView state synchronization commands and events

use dure_sijang::viewmodel::{BrowserActor, BrowserCommand, BrowserEvent, ViewModelEvent};
use smol::channel::unbounded;

#[test]
fn test_update_current_url_emits_url_changed_event() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_url_sync.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act
        cmd_tx
            .send(BrowserCommand::UpdateCurrentUrl {
                tab_id: 1,
                url: "https://test.mycart/products".to_string(),
            })
            .await
            .unwrap();

        // Assert
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::UrlChanged { tab_id, url }) => {
                assert_eq!(tab_id, 1);
                assert_eq!(url, "https://test.mycart/products");
            }
            _ => panic!("Expected UrlChanged event, got {:?}", event),
        }
    });
}

#[test]
fn test_update_title_emits_title_changed_event() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_title_sync.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act
        cmd_tx
            .send(BrowserCommand::UpdateTitle {
                tab_id: 1,
                title: "Products Page".to_string(),
            })
            .await
            .unwrap();

        // Assert
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TitleChanged { tab_id, title }) => {
                assert_eq!(tab_id, 1);
                assert_eq!(title, "Products Page");
            }
            _ => panic!("Expected TitleChanged event, got {:?}", event),
        }
    });
}

#[test]
fn test_mark_tab_failed_emits_tab_failed_event() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_tab_failed.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act
        cmd_tx
            .send(BrowserCommand::MarkTabFailed {
                tab_id: 1,
                error: "Network timeout".to_string(),
            })
            .await
            .unwrap();

        // Assert
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TabFailed { tab_id, error }) => {
                assert_eq!(tab_id, 1);
                assert_eq!(error, "Network timeout");
            }
            _ => panic!("Expected TabFailed event, got {:?}", event),
        }
    });
}

#[test]
fn test_multiple_webview_state_updates() {
    smol::block_on(async {
        // Arrange
        let (cmd_tx, cmd_rx) = unbounded();
        let (event_tx, event_rx) = unbounded();
        let temp_dir = std::env::temp_dir();
        let db_path = temp_dir.join("test_multiple_updates.db");

        let actor = BrowserActor::new(cmd_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();

        // Act - send multiple state sync commands
        cmd_tx
            .send(BrowserCommand::UpdateCurrentUrl {
                tab_id: 1,
                url: "https://test.mycart/products".to_string(),
            })
            .await
            .unwrap();

        cmd_tx
            .send(BrowserCommand::UpdateTitle {
                tab_id: 1,
                title: "Products".to_string(),
            })
            .await
            .unwrap();

        // Assert - receive events in order
        let event1 = event_rx.recv().await.unwrap();
        assert!(matches!(
            event1,
            ViewModelEvent::Browser(BrowserEvent::UrlChanged { .. })
        ));

        let event2 = event_rx.recv().await.unwrap();
        assert!(matches!(
            event2,
            ViewModelEvent::Browser(BrowserEvent::TitleChanged { .. })
        ));
    });
}
