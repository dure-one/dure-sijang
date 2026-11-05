#[cfg(test)]
mod bookmark_duplicate_tests {
    use dure_sijang::browser_stt::BrowserState;

    #[test]
    fn test_add_duplicate_bookmark_returns_ok() {
        // Test that adding duplicate bookmark doesn't error
        // (actual duplicate prevention happens in add_bookmark via is_bookmarked check)
        let mut browser_state = BrowserState::new();
        let title = "Test Page";
        let url = "https://example.com";

        // Act - Add bookmark twice (should not panic or error)
        let result1 = browser_state.add_bookmark(title, url);
        let result2 = browser_state.add_bookmark(title, url);

        // Assert - Both should return Ok
        assert!(result1.is_ok(), "First bookmark add should succeed");
        assert!(result2.is_ok(), "Duplicate bookmark add should not error");
    }
}
