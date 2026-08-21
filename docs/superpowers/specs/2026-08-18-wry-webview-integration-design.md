# WebView Integration Design

**Date:** 2026-08-18  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This spec defines the integration of wry WebView into the dure-sijang mycart browser, following strict MVVM architecture. The implementation enables WebView browsing mode alongside the existing API mode, supporting both desktop (Linux/OpenBSD with GTK) and Android platforms.

**Scope:** Specific wry integration architecture based on reference/egui_webview/examples/tabbrowser.rs, adapted to MVVM pattern.

## Goals

1. **WebView Mode**: Implement full website rendering using raw wry (no egui_webview wrapper)
2. **MVVM Compliance**: Maintain strict separation - ViewModel manages state, UI manages webview lifecycle
3. **Cross-Platform**: Support desktop (GTK-based) and Android (wry's built-in Android WebView)
4. **Reference-Inspired**: Adapt proven patterns from reference/egui_webview/examples/tabbrowser.rs
5. **Command/Event Pattern**: Tab lifecycle via ViewModel orchestration, navigation via direct webview control

## Non-Goals

- API mode modifications (already working)
- egui_webview library usage (using raw wry instead)
- Custom JNI bindings (using wry's built-in Android support)
- Embedded webview widgets (webviews managed separately from egui layout)

## Design Decisions

### Decision Summary

| Question | Decision | Rationale |
|---|---|---|
| WebView Library | Raw wry | Maximum control, no extra dependency |
| Architecture | Strict MVVM | Consistency with existing codebase |
| Scope | WebView mode only | API mode already works |
| Platform Support | Desktop + Android | User requires both now |
| WebView Lifecycle | Webviews in DureSijangApp, state in ViewModel | wry requires window handle (UI concern) |
| Tab Management | Command/Event pattern | Matches existing MVVM |
| Navigation | Direct webview control | wry owns history stack |
| Window Handle | eframe's native window | Proven from reference |
| Android Integration | wry's built-in support | Simpler than custom JNI |

## Architecture

### Component Responsibilities

**1. DureSijangApp (UI Layer)**
- Owns `HashMap<usize, wry::WebView>` - actual webview instances
- Initializes GTK on startup (Linux/OpenBSD)
- Creates webviews from eframe window handle
- Handles wry lifecycle (navigation, input, cleanup)
- Calls `BrowserUI::render()` with ViewModel reference
- Polls ViewModel events to create/destroy webviews

**2. ViewModel (State Layer)**
- Tracks tab metadata: `Vec<TabMetadata>` (id, store_url, current_url, title, mode, error)
- Manages active_tab_id
- Processes commands: CreateTab, CloseTab, UpdateCurrentUrl, UpdateTitle, MarkTabFailed
- Emits events: TabCreated, TabClosed, UrlChanged, TitleChanged, TabFailed
- Persists state to database (via existing db_browser.rs)

**3. BrowserUI (Stateless Renderer)**
- Renders tab bar (reads from ViewModel state)
- Delegates to WebViewRenderer for active tab
- Sends commands to ViewModel on user interaction
- Zero-sized struct (no state)

**4. WebViewRenderer (WebView Coordinator)**
- Displays navigation controls (back/forward/reload buttons)
- Calls webview methods directly (no Command/Event round-trip)
- Notifies ViewModel of URL/title changes
- Displays error messages for failed tabs

### Key Pattern

```
Tab Lifecycle:
User Action → UI Command → ViewModel Event → UI creates/destroys webview

Navigation:
User Action → UI calls webview.go_back() → URL change → UI notifies ViewModel
```

## Component Structure

### File Changes

**mobile/src/dure_sijang_app.rs** - Add fields and methods:
```rust
pub struct DureSijangApp {
    // NEW fields
    webviews: HashMap<usize, wry::WebView>,
    window_handle: Option<RawWindowHandle>,
    browser_ui: BrowserUI,
    
    // NEW methods
    fn create_webview(&mut self, tab_id: usize, url: &str) -> anyhow::Result<wry::WebView>;
    fn destroy_webview(&mut self, tab_id: usize);
    fn navigate_back/forward/reload(&mut self, tab_id: usize) -> anyhow::Result<()>;
    fn poll_browser_events(&mut self);
}
```

**mobile/src/browser_ui.rs** - No changes (already correct)

**mobile/src/browser_components/webview_renderer.rs** - Major changes:
- Add `app: &mut DureSijangApp` parameter to `render()`
- Call `app.navigate_back/forward/reload()` directly
- Display error banner if `tab.error` is set

**mobile/src/webview_tab.rs** - Delete (obsolete stub)

**mobile/src/viewmodel/browser.rs** - Add commands/events:
- Commands: UpdateCurrentUrl, UpdateTitle, MarkTabFailed
- Events: UrlChanged, TitleChanged, TabFailed

**mobile/src/models/browser.rs** - Add field:
- `pub error: Option<String>` to TabMetadata

## Data Flow

### Tab Creation
```
1. User clicks "+" → UI calls vm.create_tab()
2. ViewModel generates tab_id, adds TabMetadata, emits TabCreated
3. DureSijangApp polls events, sees TabCreated
4. App calls create_webview(), inserts into HashMap
5. Webview loads URL, fires callback
6. Callback calls vm.update_current_url()
```

### Tab Closure
```
1. User clicks "×" → UI calls vm.close_tab()
2. ViewModel removes TabMetadata, emits TabClosed
3. DureSijangApp polls events, sees TabClosed
4. App removes webview from HashMap (Drop cleans up)
```

### Navigation
```
1. User clicks "◀" → WebViewRenderer calls app.navigate_back()
2. App calls webview.evaluate_script("window.history.back()")
3. Webview navigates, IPC callback fires
4. Callback calls vm.update_current_url()
5. ViewModel updates state, emits UrlChanged
```

## Platform Handling

### Desktop (Linux/OpenBSD + GTK)

**main.rs initialization:**
```rust
#[cfg(any(target_os = "linux", target_os = "openbsd", ...))]
gtk::init().expect("GTK initialization failed");
```

**dure_sijang_app.rs update() loop:**
```rust
// Process GTK events EVERY FRAME
while gtk::events_pending() {
    gtk::main_iteration_do(false);
}
ctx.request_repaint(); // Continuous repainting
```

**WebView creation:**
```rust
WebViewBuilder::new()
    .with_url(url)
    .build_as_child(&window_handle)?
```

### Android

**WebView creation:**
```rust
#[cfg(target_os = "android")]
WebViewBuilder::new()
    .with_url(url)
    .build()? // wry handles JNI automatically
```

**No manual event loop** - Android main loop handles it

## Error Handling

**WebView creation failure:**
```rust
match self.create_webview(tab_id, &url) {
    Ok(webview) => self.webviews.insert(tab_id, webview),
    Err(e) => vm.mark_tab_failed(tab_id, e.to_string()),
}
```

**Navigation errors:**
- Log with tracing::error!
- Display toast/banner (future enhancement)
- Keep tab alive for retry

**Platform init errors:**
- Fail fast with clear message
- "Install gtk3 development libraries" (Linux)
- "Ensure Android System WebView installed" (Android)

**Graceful degradation:**
- Show error message with troubleshooting
- Keep app running for API mode
- User can still access debloat/scan tabs

## Testing Strategy

### Unit Tests
- WebViewRenderer error handling
- ViewModel command/event flow
- TabMetadata.error field

### Integration Tests
- Command/Event flow end-to-end
- State updates
- Database persistence
- Cannot test: wry rendering, GTK loop, real navigation

### Manual Testing Checklist

**Desktop:**
- [ ] GTK init, tab creation, navigation, multiple tabs, cleanup, persistence

**Android:**
- [ ] WebView appears, touch input, lifecycle, keyboard input

**Coverage target:** 80%+ (per Rust ECC rules)

## Open Questions

1. Android Activity context from eframe?
2. IPC callbacks for URL change detection?
3. WebView positioning over egui rect (future)?
4. Tab restoration on app restart?
5. SSL certificate handling for dev servers?

## Success Criteria

- ✅ Desktop: Create tabs, navigate, close tabs
- ✅ Android: Create tabs, touch navigation
- ✅ MVVM maintained
- ✅ Command/Event for tabs, direct for navigation
- ✅ Error handling without crashes
- ✅ GTK + Android WebView support
- ✅ 80%+ test coverage

## References

- Reference: `reference/egui_webview/examples/tabbrowser.rs`
- wry docs: https://docs.rs/wry/
- MVVM: `mobile/src/viewmodel/`, `mobile/src/browser_ui.rs`
- ECC Rust: `/home/wj/.claude/rules/ecc/rust/`
- Skills: `rust-egui-mvvm-core`, `rust-egui-mvvm-threading`

## Next Steps

1. Invoke `writing-plans` skill
2. Create implementation plan with tasks
3. Implement desktop (Linux/OpenBSD)
4. Test desktop
5. Implement Android
6. Test Android
