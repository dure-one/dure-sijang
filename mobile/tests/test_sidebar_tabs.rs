/// Tests for sidebar tab functionality
use dure_sijang::browser_stt::SidebarTab;

#[test]
fn test_sidebar_tab_has_about_variant() {
    // About variant should exist
    let about_tab = SidebarTab::About;
    assert_eq!(about_tab, SidebarTab::About);
}

#[test]
fn test_sidebar_tab_has_bookmarks_variant() {
    let bookmarks_tab = SidebarTab::Bookmarks;
    assert_eq!(bookmarks_tab, SidebarTab::Bookmarks);
}

#[test]
fn test_sidebar_tab_has_history_variant() {
    let history_tab = SidebarTab::History;
    assert_eq!(history_tab, SidebarTab::History);
}

#[test]
fn test_sidebar_tab_has_settings_variant() {
    let settings_tab = SidebarTab::Settings;
    assert_eq!(settings_tab, SidebarTab::Settings);
}

#[test]
fn test_sidebar_tab_copy_trait() {
    let tab1 = SidebarTab::About;
    let tab2 = tab1; // Should copy, not move
    assert_eq!(tab1, tab2);
}

#[test]
fn test_sidebar_tab_partial_eq() {
    assert_eq!(SidebarTab::About, SidebarTab::About);
    assert_ne!(SidebarTab::About, SidebarTab::Settings);
    assert_ne!(SidebarTab::Bookmarks, SidebarTab::History);
}
