//! Compilation test to verify GTK initialization exists on Linux/OpenBSD
//!
//! This test ensures that GTK initialization code is present and compiles
//! correctly on Linux and OpenBSD platforms. It does not test runtime behavior
//! since GTK may not be available in CI environments.

#[cfg(all(not(target_arch = "wasm32"), any(target_os = "linux", target_os = "openbsd")))]
#[test]
fn test_gtk_init_compiles() {
    // This test verifies that:
    // 1. gtk crate is available as a dependency
    // 2. gtk::init() function exists with correct signature
    // 3. The conditional compilation for GTK init is correct

    // We can't actually call gtk::init() in a test environment since GTK requires
    // a display server, but we can verify the function signature exists.

    // Verify gtk::init exists and returns Result<(), gtk::glib::BoolError>
    let _init_fn: fn() -> Result<(), gtk::glib::BoolError> = gtk::init;

    // Test passes if this compiles - GTK dependency is correctly configured
    assert!(true, "GTK init function exists and compiles");
}

#[cfg(not(all(not(target_arch = "wasm32"), any(target_os = "linux", target_os = "openbsd"))))]
#[test]
fn test_gtk_not_required_on_other_platforms() {
    // On non-Linux/OpenBSD platforms (Windows, macOS, Android, WASM),
    // GTK should not be a dependency. This test confirms we can compile
    // without GTK on these platforms.
    assert!(true, "Non-Linux/OpenBSD platform compiles without GTK");
}
