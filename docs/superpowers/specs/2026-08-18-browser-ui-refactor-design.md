# Browser UI MVVM Refactor Design

**Date:** 2026-08-18  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

Refactor the dure-sijang browser UI (`browser_ui.rs`, `api_tab.rs`, `webview_tab.rs`) to follow the existing MVVM pattern used by Debloat, Scan, and Apps actors. This eliminates borrow checker errors caused by mixing state ownership and rendering logic in UI components.

**Problem:** Current browser code violates MVVM by storing state directly in UI structs (`BrowserUI`, `ApiTabState`), which causes borrow checker conflicts when rendering methods need `&mut self` while iterating over immutable borrows.

**Solution:** Move all business logic state to `ViewModelState`, make UI components stateless renderers that read from state and send commands to `ViewModel`.

## Goals

1. **Eliminate borrow checker errors** - Fix all 5 E0502 errors in browser code
2. **Match existing MVVM pattern** - Consistent with Debloat/Scan/Apps actors
3. **Maintain functionality** - No regression in browser features
4. **Enable future features** - Bookmarks, history, sync fit naturally into this pattern
5. **Follow ECC Rust rules** - Stateless components, immutable data flow, no excessive cloning

## Design Decisions

### Question 1: Tab State Scope
**Decision:** Hybrid approach
- **In-memory (ViewModelState)**: Current products, selected product, cart items, active tab index, loading states
- **Database-backed (via BrowserActor)**: Open tabs list, bookmarks, history, store directory
- **Pattern**: On app start, BrowserActor loads tabs from DB → emits `TabsLoaded` event → ViewModelState updates

**Rationale:** Fast reactive state in memory, persistent user data in database. Matches existing pattern (e.g., `filtered_packages` in ViewModelState).

### Question 2: WebView Handle Management
**Decision:** Keep webview handles in BrowserUI (Option A)
- `ViewModelState` holds tab metadata (id, url, title, mode) - the WHAT
- `BrowserUI` holds webview handles (rendering resources) - the HOW

**Rationale:** Follows egui's established pattern where UI structs own rendering resources (like textures). WebView handles are NOT `Send`, must stay on main thread.

### Question 3: Component Granularity
**Decision:** Hierarchical decomposition (Option B+)
- Primary split by mode (WebView vs API)
- Decompose complex API mode into sub-components (ProductGrid, ProductDetail, CartView)
- Keep simple components together (tab bar stays in BrowserUI)

**Rationale:** API mode is complex (~290 lines), decomposing into ~60-80 line components follows ECC rule "200-400 lines typical, 800 max".

## Architecture

### Current Architecture (Broken)
```
UI Layer (owns state + resources)
├── BrowserUI { tabs: Vec<TabState>, webview_handles }
└── ApiTabState { products, selected_product, cart_items }
    ❌ Problem: &mut self during rendering causes borrow conflicts
```

### New Architecture (MVVM)
```
ViewModelState (read-only from UI)
├── browser_tabs: Vec<TabMetadata>
├── active_tab_id: Option<usize>
├── tab_products: HashMap<usize, Vec<Product>>
├── selected_product: HashMap<usize, Option<Product>>
└── cart_items: HashMap<usize, Vec<CartItem>>

BrowserUI (owns rendering resources only)
├── webview_handles: HashMap<usize, wry::WebView>
└── Delegates to stateless renderers:
    ├── WebViewRenderer (nav controls)
    └── ApiRenderer (delegates to)
        ├── ProductGrid
        ├── ProductDetail
        └── CartView

BrowserActor (background thread)
├── Handles commands: CreateTab, CloseTab, FetchProducts
├── Emits events: TabCreated, TabClosed, ProductsLoaded
└── Manages database: tabs, bookmarks, history
```

### Data Flow Pattern
```
User clicks "Add to Cart"
  → UI sends: viewmodel.add_to_cart(tab_id, product_id)
  → BrowserActor receives command
  → BrowserActor updates database + in-memory state
  → BrowserActor emits: CartUpdated(tab_id, cart_items)
  → ViewModel polls event, updates ViewModelState
  → UI re-renders with new state (automatic via egui)
```

## Component Structure

### File Organization
```
mobile/src/
├── viewmodel/
│   ├── browser.rs        [MODIFY] Add state fields to ViewModelState
│   └── mod.rs            [MODIFY] Add browser state to ViewModelState
│
├── browser_ui.rs         [MODIFY] Remove state, keep only webview_handles
├── webview_renderer.rs   [NEW] Stateless WebView mode renderer
├── api_renderer.rs       [NEW] Stateless API mode coordinator
│
└── browser_components/   [NEW DIRECTORY]
    ├── mod.rs            [NEW] Export all components
    ├── product_grid.rs   [NEW] Grid layout of products
    ├── product_detail.rs [NEW] Single product view
    └── cart_view.rs      [NEW] Shopping cart UI
```

### Component Signatures (All Stateless)

#### BrowserUI (owns webview handles only)
```rust
pub struct BrowserUI {
    webview_handles: HashMap<usize, wry::WebView>,
}

impl BrowserUI {
    pub fn render(&mut self, ctx: &egui::Context, state: &ViewModelState, vm: &ViewModel) {
        // Read active tab from state, delegate to renderers
    }
}
```

#### WebViewRenderer (pure function)
```rust
pub struct WebViewRenderer;

impl WebViewRenderer {
    pub fn render(ui: &mut egui::Ui, tab: &TabMetadata, handle: &wry::WebView) {
        // Navigation controls + webview display
    }
}
```

#### ApiRenderer (delegates to sub-components)
```rust
pub struct ApiRenderer;

impl ApiRenderer {
    pub fn render(ui: &mut egui::Ui, state: &ViewModelState, vm: &ViewModel, tab_id: usize) {
        match state.get_selected_product(tab_id) {
            Some(product) => ProductDetail::render(ui, state, vm, tab_id, product),
            None => ProductGrid::render(ui, state, vm, tab_id),
        }
    }
}
```

#### ProductGrid (stateless)
```rust
pub struct ProductGrid;

impl ProductGrid {
    pub fn render(ui: &mut egui::Ui, state: &ViewModelState, vm: &ViewModel, tab_id: usize) {
        let products = state.get_tab_products(tab_id);
        for product in products {
            if ui.button("View").clicked() {
                vm.select_product(tab_id, product.id.clone());
            }
        }
    }
}
```

### Size Estimates
- `browser_ui.rs`: ~80 lines (down from current 150+)
- `webview_renderer.rs`: ~60 lines
- `api_renderer.rs`: ~50 lines (coordinator)
- `product_grid.rs`: ~80 lines
- `product_detail.rs`: ~70 lines
- `cart_view.rs`: ~60 lines

**Total: ~400 lines across 6 focused files** (vs current 450 lines in 3 tangled files)

## State Management

### ViewModelState Extensions

```rust
// viewmodel/mod.rs - add to ViewModelState
pub struct ViewModelState {
    // ... existing fields (packages, uad_ng_lists, etc.) ...
    
    // === NEW: Browser state ===
    pub browser_tabs: Vec<TabMetadata>,
    pub active_tab_id: Option<usize>,
    pub tab_products: HashMap<usize, Vec<Product>>,
    pub selected_product: HashMap<usize, Option<Product>>,
    pub cart_items: HashMap<usize, Vec<CartItem>>,
    pub tab_loading: HashMap<usize, bool>,
    pub browser_error_message: Option<String>,
}

#[derive(Clone, Debug)]
pub struct TabMetadata {
    pub id: usize,
    pub store_url: String,
    pub current_url: String,
    pub title: Option<String>,
    pub mode: BrowsingMode,
}
```

### State Access Helpers

```rust
impl ViewModelState {
    pub fn get_active_tab(&self) -> Option<&TabMetadata> {
        self.active_tab_id
            .and_then(|id| self.browser_tabs.iter().find(|t| t.id == id))
    }
    
    pub fn get_tab_products(&self, tab_id: usize) -> &[Product] {
        self.tab_products.get(&tab_id).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    pub fn get_selected_product(&self, tab_id: usize) -> Option<&Product> {
        self.selected_product.get(&tab_id).and_then(|p| p.as_ref())
    }
    
    pub fn get_cart_items(&self, tab_id: usize) -> &[CartItem] {
        self.cart_items.get(&tab_id).map(|v| v.as_slice()).unwrap_or(&[])
    }
    
    pub fn is_tab_loading(&self, tab_id: usize) -> bool {
        self.tab_loading.get(&tab_id).copied().unwrap_or(false)
    }
}
```

### State Updates (Event-Driven)

```rust
// viewmodel/mod.rs - ViewModel::poll_events()
impl ViewModel {
    pub fn poll_events(&mut self) {
        while let Ok(event) = self.event_rx.try_recv() {
            match event {
                ViewModelEvent::Browser(browser_event) => {
                    self.handle_browser_event(browser_event);
                }
                // ... other event types ...
            }
        }
    }
    
    fn handle_browser_event(&mut self, event: BrowserEvent) {
        match event {
            BrowserEvent::TabCreated(tab) => {
                self.state.browser_tabs.push(tab.clone());
                self.state.active_tab_id = Some(tab.id);
            }
            BrowserEvent::TabClosed(tab_id) => {
                self.state.browser_tabs.retain(|t| t.id != tab_id);
                self.state.tab_products.remove(&tab_id);
                self.state.selected_product.remove(&tab_id);
                self.state.cart_items.remove(&tab_id);
            }
            BrowserEvent::ProductsLoaded(tab_id, products) => {
                self.state.tab_products.insert(tab_id, products);
                self.state.tab_loading.insert(tab_id, false);
            }
            BrowserEvent::ProductSelected(tab_id, product) => {
                self.state.selected_product.insert(tab_id, Some(product));
            }
            BrowserEvent::CartUpdated(tab_id, items) => {
                self.state.cart_items.insert(tab_id, items);
            }
            BrowserEvent::Error { tab_id, message } => {
                self.state.browser_error_message = Some(message);
                log::warn!("Browser error (tab {:?}): {}", tab_id, 
                          self.state.browser_error_message.as_ref().unwrap());
            }
        }
    }
}
```

### Persistence Strategy

**On app start:**
1. `BrowserActor` loads tabs from database
2. Emits `TabsLoaded(Vec<TabMetadata>)` event
3. `ViewModelState.browser_tabs` populated
4. UI renders tabs automatically

**On tab change:**
1. UI sends `CreateTab(store_url, mode)` command
2. `BrowserActor` inserts into database
3. Emits `TabCreated(TabMetadata)` event
4. `ViewModelState` updated
5. UI re-renders

## Commands & Events

### BrowserCommand (UI → Actor)

```rust
// viewmodel/browser.rs
pub enum BrowserCommand {
    // Tab management
    CreateTab { store_url: String, mode: BrowsingMode },
    CloseTab { tab_id: usize },
    SwitchTab { tab_id: usize },
    
    // Navigation (WebView mode)
    Navigate { tab_id: usize, url: String },
    GoBack { tab_id: usize },
    GoForward { tab_id: usize },
    Reload { tab_id: usize },
    
    // Product fetching (API mode)
    FetchProducts { tab_id: usize, store_url: String },
    FetchProduct { tab_id: usize, product_id: String },
    
    // Product interaction
    SelectProduct { tab_id: usize, product_id: Option<String> },
    AddToCart { tab_id: usize, product_id: String },
    RemoveFromCart { tab_id: usize, product_id: String },
    
    // Bookmarks
    AddBookmark { store_url: String, page_url: String, title: String },
    RemoveBookmark { bookmark_id: i32 },
    LoadBookmarks,
}
```

### BrowserEvent (Actor → UI)

```rust
// viewmodel/common.rs - add to ViewModelEvent enum
pub enum ViewModelEvent {
    // ... existing variants (Debloat, Scan, Apps, Metadata) ...
    Browser(BrowserEvent),
}

pub enum BrowserEvent {
    // Tab lifecycle
    TabCreated(TabMetadata),
    TabClosed(usize),
    TabsLoaded(Vec<TabMetadata>),
    
    // Product data
    ProductsLoaded { tab_id: usize, products: Vec<Product> },
    ProductLoaded { tab_id: usize, product: Product },
    ProductSelected { tab_id: usize, product: Option<Product> },
    
    // Cart updates
    CartUpdated { tab_id: usize, items: Vec<CartItem> },
    
    // Bookmarks
    BookmarksLoaded(Vec<Bookmark>),
    BookmarkAdded(Bookmark),
    
    // Errors
    Error { tab_id: Option<usize>, message: String },
}
```

### ViewModel Public API (UI Convenience Methods)

```rust
// viewmodel/mod.rs
impl ViewModel {
    // Tab management
    pub fn create_tab(&self, store_url: String, mode: BrowsingMode) -> Result<()> {
        self.browser_tx.try_send(BrowserCommand::CreateTab { store_url, mode })?;
        Ok(())
    }
    
    pub fn close_tab(&self, tab_id: usize) -> Result<()> {
        self.browser_tx.try_send(BrowserCommand::CloseTab { tab_id })?;
        Ok(())
    }
    
    // Product operations
    pub fn fetch_products(&self, tab_id: usize, store_url: String) -> Result<()> {
        self.browser_tx.try_send(BrowserCommand::FetchProducts { tab_id, store_url })?;
        Ok(())
    }
    
    pub fn select_product(&self, tab_id: usize, product_id: Option<String>) -> Result<()> {
        self.browser_tx.try_send(BrowserCommand::SelectProduct { tab_id, product_id })?;
        Ok(())
    }
    
    pub fn add_to_cart(&self, tab_id: usize, product_id: String) -> Result<()> {
        self.browser_tx.try_send(BrowserCommand::AddToCart { tab_id, product_id })?;
        Ok(())
    }
    
    pub fn clear_browser_error(&self) -> Result<()> {
        self.browser_tx.try_send(BrowserCommand::ClearError)?;
        Ok(())
    }
}
```

## Error Handling

### Error Strategy

Following **ECC Rust rules**:
- Use `anyhow::Result` for application code (not library)
- Use `?` operator for error propagation
- Provide context with `.context()`
- Emit error events instead of panicking

### BrowserActor Error Handling

```rust
// viewmodel/browser.rs
impl BrowserActor {
    async fn handle_command(&self, cmd: BrowserCommand) {
        match cmd {
            BrowserCommand::FetchProducts { tab_id, store_url } => {
                match self.fetch_products_internal(&store_url).await {
                    Ok(products) => {
                        let _ = self.event_tx.send(ViewModelEvent::Browser(
                            BrowserEvent::ProductsLoaded { tab_id, products }
                        )).await;
                    }
                    Err(e) => {
                        log::error!("Failed to fetch products for tab {}: {}", tab_id, e);
                        let _ = self.event_tx.send(ViewModelEvent::Browser(
                            BrowserEvent::Error {
                                tab_id: Some(tab_id),
                                message: format!("Failed to load products: {}", e),
                            }
                        )).await;
                    }
                }
            }
            // ... other commands ...
        }
    }
    
    async fn fetch_products_internal(&self, store_url: &str) -> anyhow::Result<Vec<Product>> {
        let response = self.mycart_client
            .get_products(store_url)
            .await
            .context("HTTP request failed")?;
        
        let products = response
            .parse_json()
            .context("Failed to parse product JSON")?;
        
        Ok(products)
    }
}
```

### UI Error Display

```rust
// browser_ui.rs
impl BrowserUI {
    pub fn render(&mut self, ctx: &egui::Context, state: &ViewModelState, vm: &ViewModel) {
        // Show error toast if present
        if let Some(error_msg) = &state.browser_error_message {
            egui::Window::new("Error")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.colored_label(egui::Color32::RED, error_msg);
                    if ui.button("OK").clicked() {
                        let _ = vm.clear_browser_error();
                    }
                });
        }
        
        // ... rest of rendering ...
    }
}
```

### Database Error Handling

```rust
// db_browser.rs
pub fn insert_tab(conn: &mut SqliteConnection, tab: &TabMetadata) -> anyhow::Result<()> {
    use crate::schema::tabs::dsl::*;
    
    diesel::insert_into(tabs)
        .values((
            id.eq(tab.id as i32),
            store_url.eq(&tab.store_url),
            current_url.eq(&tab.current_url),
            title.eq(&tab.title),
            mode.eq(tab.mode.to_string()),
        ))
        .execute(conn)
        .context("Failed to insert tab into database")?;
    
    Ok(())
}
```

## Testing Strategy

### Test Organization

Following **ECC Rust testing rules**:
- Unit tests in same file as implementation
- Integration tests in `tests/` directory
- Minimum 80% coverage (checked with `cargo llvm-cov`)
- Use AAA pattern (Arrange-Act-Assert)
- Use `cargo-nextest` (NOT standard `cargo test`)

### Unit Tests (Stateless Components)

```rust
// browser_components/product_grid.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_product_grid_renders_empty_state() {
        // Arrange
        let mut state = ViewModelState::default();
        state.tab_products.insert(1, vec![]);
        
        // Act
        let rendered = test_helpers::render_component(|ui| {
            ProductGrid::render(ui, &state, &mock_viewmodel(), 1);
        });
        
        // Assert
        assert!(rendered.contains("No products"));
    }
    
    #[test]
    fn test_product_grid_renders_products() {
        // Arrange
        let mut state = ViewModelState::default();
        state.tab_products.insert(1, vec![
            Product { 
                id: "p1".into(), 
                name: "Test Product".into(), 
                slug: "test-product".into(),
                price: 10.0,
                description: None,
                image_url: None,
                is_active: true,
            },
        ]);
        
        // Act - verify no panics
        let _ = test_helpers::render_component(|ui| {
            ProductGrid::render(ui, &state, &mock_viewmodel(), 1);
        });
        
        // Assert - rendering succeeded without panic
    }
}
```

### Integration Tests (Actor Communication)

```rust
// tests/browser_actor_test.rs
use dure_sijang::viewmodel::*;

#[test]
fn test_browser_actor_create_tab() {
    smol::block_on(async {
        // Arrange
        let (browser_tx, browser_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();
        let db_path = test_helpers::temp_db_path();
        
        let actor = BrowserActor::new(browser_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();
        
        // Act
        browser_tx.send(BrowserCommand::CreateTab {
            store_url: "https://test.mycart".into(),
            mode: BrowsingMode::Api,
        }).await.unwrap();
        
        // Assert
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::TabCreated(tab)) => {
                assert_eq!(tab.store_url, "https://test.mycart");
                assert_eq!(tab.mode, BrowsingMode::Api);
            }
            _ => panic!("Expected TabCreated event"),
        }
    })
}

#[test]
fn test_browser_actor_fetch_products() {
    smol::block_on(async {
        // Arrange
        let (browser_tx, browser_rx) = smol::channel::unbounded();
        let (event_tx, event_rx) = smol::channel::unbounded();
        let db_path = test_helpers::temp_db_path();
        
        let actor = BrowserActor::new(browser_rx, event_tx, db_path);
        smol::spawn(actor.run()).detach();
        
        // Create tab first
        let tab_id = 1;
        
        // Act
        browser_tx.send(BrowserCommand::FetchProducts {
            tab_id,
            store_url: "https://test.mycart".into(),
        }).await.unwrap();
        
        // Assert
        let event = event_rx.recv().await.unwrap();
        match event {
            ViewModelEvent::Browser(BrowserEvent::ProductsLoaded { tab_id: id, products }) => {
                assert_eq!(id, tab_id);
                assert!(!products.is_empty());
            }
            ViewModelEvent::Browser(BrowserEvent::Error { .. }) => {
                // OK - network might fail in test environment
            }
            _ => panic!("Expected ProductsLoaded or Error event"),
        }
    })
}
```

### State Helper Tests

```rust
// viewmodel/mod.rs
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_get_active_tab() {
        // Arrange
        let mut state = ViewModelState::default();
        state.browser_tabs.push(TabMetadata {
            id: 1,
            store_url: "https://test".into(),
            current_url: "https://test".into(),
            title: Some("Test".into()),
            mode: BrowsingMode::Api,
        });
        state.active_tab_id = Some(1);
        
        // Act
        let active = state.get_active_tab();
        
        // Assert
        assert!(active.is_some());
        assert_eq!(active.unwrap().id, 1);
    }
    
    #[test]
    fn test_get_tab_products_empty() {
        // Arrange
        let state = ViewModelState::default();
        
        // Act
        let products = state.get_tab_products(1);
        
        // Assert
        assert_eq!(products.len(), 0);
    }
}
```

### Coverage Target

- **Minimum: 80%** (enforced by `cargo llvm-cov --fail-under-lines 80`)
- **Unit tests**: All stateless component render functions
- **Integration tests**: All BrowserActor commands
- **State tests**: All ViewModelState helper methods

### Test Execution

```bash
# Run all tests
cargo nextest run

# Run with coverage
cargo llvm-cov --html
open target/llvm-cov/html/index.html

# Fail if below 80%
cargo llvm-cov --fail-under-lines 80
```

## Implementation Phases

### Phase 1: State Migration
1. Add browser state fields to `ViewModelState`
2. Add state helper methods (`get_active_tab`, `get_tab_products`, etc.)
3. Update `ViewModel::handle_browser_event()` to populate state
4. Add unit tests for state helpers

### Phase 2: Component Creation
1. Create `browser_components/` directory
2. Implement `ProductGrid`, `ProductDetail`, `CartView` (stateless)
3. Implement `WebViewRenderer`, `ApiRenderer` (stateless)
4. Add unit tests for each component

### Phase 3: BrowserUI Refactor
1. Remove state fields from `BrowserUI` (keep only `webview_handles`)
2. Update `render()` to read from `ViewModelState` and delegate to renderers
3. Update all UI event handlers to send commands to `ViewModel`

### Phase 4: Command/Event Wiring
1. Add new command variants to `BrowserCommand`
2. Add new event variants to `BrowserEvent`
3. Add ViewModel convenience methods (`select_product`, `add_to_cart`, etc.)
4. Update `BrowserActor` to handle new commands

### Phase 5: Integration & Testing
1. Run integration tests for actor communication
2. Run coverage check (`cargo llvm-cov --fail-under-lines 80`)
3. Manual UI testing for regression
4. Fix any edge cases discovered during testing

### Phase 6: Cleanup
1. Delete `api_tab.rs` (replaced by components)
2. Delete `webview_tab.rs` (replaced by `WebViewRenderer`)
3. Update imports in `dure_sijang_app.rs`
4. Final coverage check

## Migration Path

### Files to Modify
- `mobile/src/viewmodel/mod.rs` - Add browser state to ViewModelState
- `mobile/src/viewmodel/browser.rs` - Add new commands/events
- `mobile/src/viewmodel/common.rs` - Add BrowserEvent to ViewModelEvent
- `mobile/src/browser_ui.rs` - Refactor to stateless renderer
- `mobile/src/dure_sijang_app.rs` - Update browser UI integration

### Files to Create
- `mobile/src/webview_renderer.rs` - WebView mode renderer
- `mobile/src/api_renderer.rs` - API mode coordinator
- `mobile/src/browser_components/mod.rs` - Component exports
- `mobile/src/browser_components/product_grid.rs` - Product grid
- `mobile/src/browser_components/product_detail.rs` - Product detail
- `mobile/src/browser_components/cart_view.rs` - Shopping cart

### Files to Delete
- `mobile/src/api_tab.rs` - Replaced by components
- `mobile/src/webview_tab.rs` - Replaced by WebViewRenderer

## Success Criteria

1. ✅ **All borrow checker errors resolved** - Cargo check passes without E0502 errors
2. ✅ **All tests pass** - `cargo nextest run` succeeds
3. ✅ **80%+ test coverage** - `cargo llvm-cov --fail-under-lines 80` passes
4. ✅ **No functionality regression** - Manual testing confirms all browser features work
5. ✅ **Matches MVVM pattern** - Consistent with Debloat/Scan/Apps actors
6. ✅ **Code quality** - No new clippy warnings, rustfmt passes

## Open Questions

None - all design decisions validated with user.

## References

- Existing MVVM actors: `mobile/src/viewmodel/debloat.rs`, `scan.rs`, `apps.rs`
- ECC Rust rules: `/home/wj/.claude/rules/ecc/rust/`
- MVVM skills: `rust-egui-mvvm-core`, `rust-egui-mvvm-threading`
- Existing browser code: `mobile/src/browser_ui.rs`, `api_tab.rs`, `webview_tab.rs`
