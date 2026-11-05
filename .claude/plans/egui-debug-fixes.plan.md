# Plan: egui Debug Fixes - X11 Panic & RFC5424 Logging

**Complexity**: Medium
**Branch**: feature/egui-debug-fixes
**Date**: 2026-11-05

## Summary

Fix two critical issues:
1. **X11 GLXBadWindow panic**: Remove unsafe `set_visible(false)` call in `EguiWebView::drop()` that attempts X11 interaction after window destruction
2. **RFC5424 syslog format**: Refactor logging to comply with RFC5424 standard for better syslog integration

## Patterns to Mirror

| Category | Source | Pattern |
|----------|--------|---------|
| Error handling | `mobile/src/egui_webview/mod.rs:74` | `.ok()` to swallow non-critical errors |
| Logging | `mobile/src/dure_sijang_app.rs:172-175` | `log::error!()`, `log::info!()`, etc. |
| Platform conditionals | `mobile/src/egui_webview/mod.rs:410` | `#[cfg(not(target_os = "android"))]` |
| Log format (current) | `mobile/src/log_capture.rs:62` | `[LEVEL] target: message` |
| Level conversion | `mobile/src/log_capture.rs:65-71` | Match expression for level strings |

## Root Cause Analysis

### Problem 1: X11 GLXBadWindow Panic

**Error**:
```
X11 error: XError { description: "GLXBadWindow", error_code: 170, request_code: 152, minor_code: 32 }
thread 'main' panicked at winit-0.30.13/src/platform_impl/linux/x11/event_processor.rs:1324:49:
Failed to unfocus input context: XError { description: "GLXBadWindow", error_code: 170 }
```

**Location**: `mobile/src/egui_webview/mod.rs:68-76`

```rust
impl Drop for EguiWebView {
    fn drop(&mut self) {
        println!("🗑️  Dropping EguiWebView {:?}", self.id);
        println!("   Arc strong count: {}", Arc::strong_count(&self.view));

        // Explicitly hide the webview before dropping
        self.view.set_visible(false).ok();  // ← LINE 74: UNSAFE CALL
    }
}
```

**Why it panics**:
- `set_visible(false)` attempts to unfocus X11 input context
- X11 window may already be destroyed by winit during shutdown
- GLX context is invalid (GLXBadWindow error code 170)
- Panic occurs because winit uses `.unwrap()` on X11 operations

**Evidence**:
- Arc strong count: 2 (should be 1) → extra reference in `GlobalWebViewState`
- "Initialized egui_webview for browser" → followed immediately by panic
- `webview_end_frame()` already manages visibility cleanup (line 509-535)

### Problem 2: RFC5424 Format

**Current format** (`log_capture.rs:62`):
```
[ERROR] dure_sijang::dure_sijang_app: === System Information ===
```

**RFC5424 format** (IETF RFC 5424):
```
<priority>version timestamp hostname app-name procid msgid structured-data msg
```

Example:
```
<134>1 2026-11-05T12:34:56.789+00:00 hostname dure-sijang 12345 dure_sijang::app - === System Information ===
```

**Priority calculation**:
- Priority = (Facility * 8) + Severity
- Facility 16 (local0) for user-level messages
- Severity: 0=Emergency, 3=Error, 4=Warning, 6=Info, 7=Debug

## Files to Change

| File | Action | Why |
|------|--------|-----|
| `mobile/src/egui_webview/mod.rs` | UPDATE | Remove unsafe `set_visible(false)` from Drop impl (line 74) |
| `mobile/src/log_capture.rs` | UPDATE | Implement RFC5424 formatter in CombinedLogger::log() |
| `mobile/Cargo.toml` | UPDATE | Add `hostname = "0.4"` and `chrono` (already present) |

## Implementation Tasks

### Task 1: Remove Unsafe set_visible Call

**File**: `mobile/src/egui_webview/mod.rs:68-76`

**Change**:
```rust
impl Drop for EguiWebView {
    fn drop(&mut self) {
        println!("🗑️  Dropping EguiWebView {:?}", self.id);
        println!("   Arc strong count: {}", Arc::strong_count(&self.view));

        // Don't call set_visible(false) - X11 window may already be destroyed
        // Visibility cleanup is handled by webview_end_frame() during normal operation
    }
}
```

**Rationale**:
- `set_visible(false)` in Drop is redundant - cleanup happens in `webview_end_frame()`
- Attempting X11 operations during Drop is unsafe when window may be destroyed
- Keeping debug println for visibility into Arc refcount issues

**Mirror**: Error swallowing with `.ok()` throughout module, but here we remove the call entirely

**Validate**:
```bash
cargo build
cargo run  # Close window - should not panic
```

### Task 2: Implement RFC5424 Syslog Format

**File**: `mobile/src/log_capture.rs:46-94`

**Add helper function** (after line 23):
```rust
use std::process;

/// Convert log level to RFC5424 severity (0-7)
fn level_to_severity(level: log::Level) -> u8 {
    match level {
        log::Level::Error => 3,   // Error
        log::Level::Warn => 4,    // Warning
        log::Level::Info => 6,    // Informational
        log::Level::Debug => 7,   // Debug
        log::Level::Trace => 7,   // Debug
    }
}

/// Format log in RFC5424 syslog format
/// <priority>version timestamp hostname app-name procid msgid structured-data msg
fn format_rfc5424(level: log::Level, target: &str, message: &str) -> String {
    // Priority = (facility * 8) + severity
    // Facility 16 = local0 (user-level)
    let facility = 16;
    let severity = level_to_severity(level);
    let priority = (facility * 8) + severity;
    
    // Timestamp in ISO8601 with milliseconds
    let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%S%.3fZ");
    
    // Hostname (fallback to "localhost" if unavailable)
    let hostname = hostname::get()
        .ok()
        .and_then(|h| h.into_string().ok())
        .unwrap_or_else(|| "localhost".to_string());
    
    // Process ID
    let procid = process::id();
    
    // Format: <priority>version timestamp hostname app-name procid msgid - msg
    format!(
        "<{}>1 {} {} dure-sijang {} {} - {}",
        priority, timestamp, hostname, procid, target, message
    )
}
```

**Update CombinedLogger::log()** (lines 46-94):
```rust
fn log(&self, record: &log::Record) {
    if !self.enabled(record.metadata()) {
        return;
    }

    let level = record.level();
    let target = record.target();
    let message = format!("{}", record.args());

    // Skip empty messages
    if message.trim().is_empty() {
        return;
    }

    // Format log line based on platform
    #[cfg(target_os = "android")]
    let log_line = {
        // Android: Keep simple format for logcat compatibility
        format!("[{}] {}: {}", level, target, message)
    };

    #[cfg(not(target_os = "android"))]
    let log_line = {
        // Desktop: Use RFC5424 syslog format
        format_rfc5424(level, target, &message)
    };

    // Convert level to string for in-app log viewer
    let level_str = match level {
        log::Level::Error => "ERROR",
        log::Level::Warn => "WARN",
        log::Level::Info => "INFO",
        log::Level::Debug => "DEBUG",
        log::Level::Trace => "TRACE",
    };

    // Append to global log buffer
    crate::dure_sijang_app::append_log(level_str, log_line.clone());

    // Platform-specific output
    #[cfg(target_os = "android")]
    {
        match level {
            log::Level::Error => eprintln!("{}", log_line),
            _ => println!("{}", log_line),
        }
    }

    #[cfg(not(target_os = "android"))]
    {
        // Desktop: Output RFC5424 format
        match level {
            log::Level::Error => eprintln!("{}", log_line),
            _ => println!("{}", log_line),
        }
    }
}
```

**Mirror**: 
- Existing level conversion pattern (line 65-71)
- Platform conditional formatting (line 77-93)
- Error message handling (line 56-59)

**Validate**:
```bash
cargo build
cargo run 2>&1 | head -5
# Should see RFC5424 format:
# <134>1 2026-11-05T12:34:56.789Z hostname dure-sijang 12345 dure_sijang - Starting
```

### Task 3: Add hostname Dependency

**File**: `mobile/Cargo.toml`

**Add after line 60** (after `crossbeam-queue`):
```toml
hostname = "0.4"
```

**Note**: `chrono` already present at line 54 with `serde` feature - no change needed

**Validate**:
```bash
cargo update
cargo check
```

## Branch Strategy

```bash
# Create feature branch from main
git checkout -b feature/egui-debug-fixes

# Implement fixes
# ... make changes ...

# Commit with descriptive messages
git add mobile/src/egui_webview/mod.rs
git commit -m "fix: remove unsafe set_visible call from EguiWebView Drop

Fixes X11 GLXBadWindow panic when window is destroyed.
The set_visible(false) call attempted X11 operations on an already-destroyed
window context during shutdown. Cleanup is already handled by webview_end_frame()."

git add mobile/src/log_capture.rs mobile/Cargo.toml
git commit -m "feat: implement RFC5424 syslog format for logging

Refactor CombinedLogger to output RFC5424-compliant syslog messages.
Format: <priority>version timestamp hostname app-name procid msgid - msg

Android keeps simple format for logcat compatibility."

# Test
cargo build
cargo run

# Push and create PR
git push -u origin feature/egui-debug-fixes
```

## Validation Commands

```bash
# Full build and test
cd mobile
cargo clean
cargo build
cargo test

# Run and verify no X11 panic
cargo run
# 1. Open app
# 2. Create a browser tab
# 3. Close window
# 4. Should NOT see GLXBadWindow panic

# Check log format
cargo run 2>&1 | head -10
# Should see RFC5424 format like:
# <134>1 2026-11-05T12:34:56.789Z yourhostname dure-sijang 12345 dure_sijang - Starting
# <131>1 2026-11-05T12:34:57.123Z yourhostname dure-sijang 12345 dure_sijang::app - System Information
```

## Expected Output Changes

### Before (Current):
```
[ERROR] dure_sijang::dure_sijang_app: === System Information ===
[ERROR] dure_sijang::dure_sijang_app: OS: openbsd
[ERROR] dure_sijang::dure_sijang_app: Architecture: x86_64
```

### After (RFC5424):
```
<131>1 2026-11-05T12:34:56.789Z openbsd-host dure-sijang 12345 dure_sijang::dure_sijang_app - === System Information ===
<131>1 2026-11-05T12:34:56.790Z openbsd-host dure-sijang 12345 dure_sijang::dure_sijang_app - OS: openbsd
<131>1 2026-11-05T12:34:56.791Z openbsd-host dure-sijang 12345 dure_sijang::dure_sijang_app - Architecture: x86_64
```

Priority breakdown:
- `<131>` = (16 * 8) + 3 = ERROR (facility local0, severity 3)
- `<132>` = (16 * 8) + 4 = WARN
- `<134>` = (16 * 8) + 6 = INFO
- `<135>` = (16 * 8) + 7 = DEBUG/TRACE

## Risks & Mitigations

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Removing set_visible breaks cleanup | LOW | Medium | webview_end_frame() already handles cleanup; Drop call was redundant |
| RFC5424 breaks Android logcat | MEDIUM | Medium | Keep simple format on Android path (#[cfg(target_os = "android")]) |
| hostname crate fails on OpenBSD | LOW | Low | Fallback to "localhost" if hostname::get() fails |
| Breaking log parsers | MEDIUM | Medium | RFC5424 is standard; document change; old parsers need update |
| Performance impact of timestamp/hostname | LOW | Low | Minimal - only affects logging path, not hot code |

## Acceptance Criteria

- [ ] Task 1 complete: set_visible removed from Drop
- [ ] Task 2 complete: RFC5424 format implemented
- [ ] Task 3 complete: hostname dependency added
- [ ] `cargo build` succeeds with no errors
- [ ] `cargo test` passes (if tests exist)
- [ ] App runs without X11 panic on window close
- [ ] Logs output in RFC5424 format on desktop
- [ ] Logs remain simple format on Android
- [ ] Priority values correct for each log level
- [ ] Hostname appears in logs (not "localhost" on real systems)
- [ ] Feature branch created and ready for PR

## Notes

- **ponytail**: Minimal changes - removed one unsafe line, reformatted logging
- **No new abstractions**: Reusing existing patterns (level conversion, platform conditionals)
- **Backward compatibility**: Android keeps original format for logcat
- **Standard compliance**: RFC5424 is IETF standard, widely supported by syslog daemons
