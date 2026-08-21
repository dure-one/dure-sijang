# Webview Migration Design: Dure-Sijang to Dure-Sijang

**Date:** 2026-08-18  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

This specification defines the complete migration from Dure-Sijang (Android debloater) to Dure-Sijang (mycart designated browser). The migration transforms the application from an Android device management tool into a cross-platform browser for navigating multiple mycart e-commerce stores with two browsing modes: webview and API.

## Goals

1. **Complete transformation**: Remove all Dure-Sijang functionality (debloat, scan, install)
2. **Dual-mode browsing**: Support both webview mode (embedded browser) and API mode (native UI)
3. **Store directory**: Integrate with dure.one to fetch and manage mycart store listings
4. **MVVM preservation**: Maintain existing MVVM architecture pattern with smol async runtime
5. **Tab-based UI**: Implement multi-tab browser interface from reference implementation
6. **Comprehensive persistence**: Store bookmarks, history, credentials, carts, and session state

## Architecture

### High-Level Structure

```
┌─────────────────────────────────────────────────────────────┐
│                  DureSijangApp (egui UI)                     │
│  ┌────────────────────────────────────────────────────┐     │
│  │              ViewModel (Command/Event)              │     │
│  │                                                     │     │
│  │  ┌──────────────────────────────────────────┐     │     │
│  │  │       ViewModelState (Read-Only)          │     │     │
│  │  │  • store_directory: Vec<StoreEntry>      │     │     │
│  │  │  • tabs: Vec<TabState>                   │     │     │
│  │  │  • active_tab_idx: Option<usize>         │     │     │
│  │  │  • bookmarks: Vec<Bookmark>              │     │     │
│  │  │  • history: Vec<HistoryEntry>            │     │     │
│  │  │  • cached_products: HashMap<...>         │     │     │
│  │  │  • cached_carts: HashMap<...>            │     │     │
│  │  └──────────────────────────────────────────┘     │     │
│  │                                                     │     │
│  │  Command Channels (UI → Actors):                   │     │
│  │  • browser_tx  → BrowserActor                     │     │
│  │  • directory_tx → StoreDirectoryActor             │     │
│  │                                                     │     │
│  │  Event Channel (Actors → UI):                      │     │
│  │  • event_rx ← All Actors                          │     │
│  └────────────────────────────────────────────────────┘     │
│                                                               │
│  Actors (Background Thread - smol runtime):                  │
│  ┌──────────────────────┐  ┌───────────────────────┐       │
│  │    BrowserActor      │  │ StoreDirectoryActor   │       │
│  │ • OpenTab            │  │ • FetchDirectory      │       │
│  │ • CloseTab           │  │ • RefreshDirectory    │       │
│  │ • Navigate           │  │ • CacheDirectory      │       │
│  │ • ToggleMode         │  └───────────────────────┘       │
│  │ • FetchProducts      │                                   │
│  │ • AddToCart          │                                   │
│  │ • Bookmark           │                                   │
│  └──────────────────────┘                                   │
└─────────────────────────────────────────────────────────────┘
         │                    │
         ▼                    ▼
  ┌──────────────────────────────────────────────────┐
  │          Database Layer (Diesel + SQLite)         │
  │  • store_directory                               │
  │  • tabs (session persistence)                    │
  │  • bookmarks                                     │
  │  • browsing_history                              │
  │  • cached_products                               │
  │  • cached_carts                                  │
  │  • user_credentials (admin tokens)               │
  │  • user_preferences                              │
  └──────────────────────────────────────────────────┘
```

### Key Changes from Dure-Sijang

**Removed:**
- DebloatActor, ScanActor, AppsActor, MetadataActor
- All debloat/scan/install UI tabs
- UAD-NG lists, VirusTotal, HybridAnalysis integrations

**Added:**
- BrowserActor (webview + API mode)
- StoreDirectoryActor (fetch directory from dure.one)
- Tab-based browser UI (from reference/tabbrowser.rs)
- mycart API client integration

**Retained:**
- MVVM architecture pattern
- smol async runtime
- Diesel + SQLite persistence
- Command/Event messaging

## Data Models

### Database Schema

#### 1. store_directory
```sql
CREATE TABLE store_directory (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    base_url TEXT NOT NULL UNIQUE,
    description TEXT,
    category TEXT,
    logo_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    last_synced_at TIMESTAMP,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### 2. tabs
```sql
CREATE TABLE tabs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    tab_index INTEGER NOT NULL,
    store_url TEXT NOT NULL,
    current_url TEXT NOT NULL,
    mode TEXT NOT NULL CHECK(mode IN ('webview', 'api')),
    title TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### 3. bookmarks
```sql
CREATE TABLE bookmarks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    page_url TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_url, page_url)
);
```

#### 4. browsing_history
```sql
CREATE TABLE browsing_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    page_url TEXT NOT NULL,
    title TEXT,
    visited_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
CREATE INDEX idx_history_visited ON browsing_history(visited_at DESC);
```

#### 5. cached_products
```sql
CREATE TABLE cached_products (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    product_id TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT NOT NULL,
    price REAL NOT NULL,
    description TEXT,
    image_url TEXT,
    is_active BOOLEAN NOT NULL DEFAULT 1,
    cached_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_url, product_id)
);
CREATE INDEX idx_products_store ON cached_products(store_url);
```

#### 6. cached_carts
```sql
CREATE TABLE cached_carts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    cart_id TEXT NOT NULL,
    cart_data TEXT NOT NULL, -- JSON blob
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_url, cart_id)
);
```

#### 7. user_credentials
```sql
CREATE TABLE user_credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL UNIQUE,
    admin_token TEXT NOT NULL,
    admin_email TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

#### 8. user_preferences
```sql
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

### Rust Data Models

#### StoreEntry
```rust
#[derive(Clone, Debug)]
pub struct StoreEntry {
    pub id: i32,
    pub name: String,
    pub base_url: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub logo_url: Option<String>,
    pub is_active: bool,
}
```

#### TabState
```rust
#[derive(Clone, Debug)]
pub struct TabState {
    pub id: usize,
    pub store_url: String,
    pub current_url: String,
    pub mode: BrowsingMode,
    pub title: Option<String>,
    pub webview: Option<EguiWebView>, // Only for webview mode
}

#[derive(Clone, Debug, PartialEq)]
pub enum BrowsingMode {
    WebView,
    Api,
}
```

#### Product (mycart API)
```rust
#[derive(Clone, Debug, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub price: f64,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub is_active: bool,
}
```

#### Cart (mycart API)
```rust
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cart {
    pub id: String,
    pub items: Vec<CartItem>,
    pub total: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CartItem {
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
}
```

#### Bookmark
```rust
#[derive(Clone, Debug)]
pub struct Bookmark {
    pub id: i32,
    pub store_url: String,
    pub page_url: String,
    pub title: String,
    pub description: Option<String>,
}
```

#### HistoryEntry
```rust
#[derive(Clone, Debug)]
pub struct HistoryEntry {
    pub id: i32,
    pub store_url: String,
    pub page_url: String,
    pub title: Option<String>,
    pub visited_at: chrono::NaiveDateTime,
}
```

## Actor Design

### BrowserActor

**Commands** (UI → Actor):
```rust
pub enum BrowserCommand {
    // Tab management
    OpenTab { store_url: String, mode: BrowsingMode },
    CloseTab { tab_id: usize },
    SwitchTab { tab_id: usize },
    
    // Navigation (webview mode)
    Navigate { tab_id: usize, url: String },
    GoBack { tab_id: usize },
    GoForward { tab_id: usize },
    
    // Mode switching
    ToggleMode { tab_id: usize, new_mode: BrowsingMode },
    
    // API mode operations
    FetchProducts { tab_id: usize, store_url: String },
    FetchProductDetails { tab_id: usize, product_id: String },
    AddToCart { tab_id: usize, product_id: String, quantity: u32 },
    ViewCart { tab_id: usize },
    Checkout { tab_id: usize, cart_id: String },
    
    // Admin operations (requires auth token)
    AdminLogin { store_url: String, email: String, password: String },
    AdminCreateProduct { store_url: String, name: String, price: f64 },
    
    // Bookmarks & history
    AddBookmark { store_url: String, page_url: String, title: String },
    RemoveBookmark { bookmark_id: i32 },
    LoadBookmarks,
    LoadHistory,
    
    // Session persistence
    SaveSession,
    RestoreSession,
}
```

**Events** (Actor → UI):
```rust
pub enum BrowserEvent {
    // Tab events
    TabOpened { tab_id: usize, tab_state: TabState },
    TabClosed { tab_id: usize },
    TabUpdated { tab_id: usize, tab_state: TabState },
    
    // Navigation events (webview mode)
    PageLoaded { tab_id: usize, url: String, title: String },
    NavigationError { tab_id: usize, error: String },
    
    // API mode events
    ProductsLoaded { tab_id: usize, products: Vec<Product> },
    ProductDetailsLoaded { tab_id: usize, product: Product },
    CartUpdated { tab_id: usize, cart: Cart },
    CheckoutComplete { tab_id: usize, order_id: String },
    
    // Auth events
    LoginSuccess { store_url: String, token: String },
    LoginFailed { store_url: String, error: String },
    
    // Bookmarks & history
    BookmarksLoaded { bookmarks: Vec<Bookmark> },
    HistoryLoaded { history: Vec<HistoryEntry> },
    
    // Session events
    SessionRestored { tabs: Vec<TabState> },
    
    // Errors
    Error { operation: String, message: String },
}
```

**Implementation Notes:**
- Actor runs in background thread with smol async runtime
- Manages tab lifecycle (create/destroy webviews, API state)
- Coordinates with database layer for persistence
- Emits events for all state changes
- Handles both webview and API mode operations

### StoreDirectoryActor

**Commands**:
```rust
pub enum DirectoryCommand {
    FetchDirectory,          // Fetch from dure.one
    RefreshDirectory,        // Force refresh
    AddCustomStore { name: String, base_url: String },
    RemoveStore { store_id: i32 },
    ToggleStoreActive { store_id: i32 },
}
```

**Events**:
```rust
pub enum DirectoryEvent {
    DirectoryLoaded { stores: Vec<StoreEntry> },
    DirectoryUpdated { stores: Vec<StoreEntry> },
    StoreAdded { store: StoreEntry },
    StoreRemoved { store_id: i32 },
    Error { operation: String, message: String },
}
```

**Implementation Notes:**
- Fetches directory from `https://dure.one/api/directory.json`
- Caches directory in SQLite
- Supports custom store additions
- Handles offline mode (use cached directory)

## dure.one Directory API

**Endpoint**: `https://dure.one/api/directory.json`

**Response Format**:
```json
{
  "version": "1.0",
  "updated_at": "2026-08-18T12:00:00Z",
  "stores": [
    {
      "name": "Demo Store",
      "base_url": "https://demo.mycart.example",
      "description": "Example mycart store",
      "category": "electronics",
      "logo_url": "https://demo.mycart.example/logo.png"
    },
    {
      "name": "Fashion Store",
      "base_url": "https://fashion.mycart.example",
      "description": "Clothing and accessories",
      "category": "fashion",
      "logo_url": "https://fashion.mycart.example/logo.png"
    }
  ]
}
```

## mycart API Integration

**Base URL**: Each store has its own base URL (from directory)

**Key Endpoints** (from swagger spec):

### Public Endpoints (no auth)
- `GET /api/products` - List active products
- `GET /api/products/{product_id}` - Get product details
- `POST /api/cart/create` - Create cart, returns cart_id
- `GET /api/cart/{cart_id}` - Get cart details
- `GET /api/pages/{page_slug}` - Get custom page content
- `GET /api/settings` - Get store settings (name, currency, etc.)

### Admin Endpoints (BearerAuth required)
- `POST /api/sign/in` - Login with email/password, returns JWT token
- `POST /api/_/products` - Create product
- `PATCH /api/_/products/{product_id}` - Update product
- `DELETE /api/_/products/{product_id}` - Delete product
- `GET /api/_/carts` - List all carts (admin view)

**API Client** (`mobile/src/api_mycart.rs`):
```rust
pub struct MyCartClient {
    base_url: String,
    client: ureq::Agent,
    admin_token: Option<String>,
}

impl MyCartClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            client: ureq::Agent::new(),
            admin_token: None,
        }
    }
    
    pub fn set_admin_token(&mut self, token: String) {
        self.admin_token = Some(token);
    }
    
    pub fn fetch_products(&self) -> Result<Vec<Product>, anyhow::Error> {
        let url = format!("{}/api/products", self.base_url);
        let resp = self.client.get(&url).call()?;
        let products: Vec<Product> = resp.into_json()?;
        Ok(products)
    }
    
    pub fn create_cart(&self, items: Vec<CartItem>) -> Result<String, anyhow::Error> {
        let url = format!("{}/api/cart/create", self.base_url);
        let resp = self.client.post(&url).send_json(items)?;
        let cart_id: String = resp.into_json()?;
        Ok(cart_id)
    }
    
    pub fn admin_login(&mut self, email: String, password: String) -> Result<String, anyhow::Error> {
        let url = format!("{}/api/sign/in", self.base_url);
        let resp = self.client.post(&url)
            .send_json(serde_json::json!({
                "email": email,
                "password": password,
            }))?;
        let token: String = resp.into_json()?;
        self.admin_token = Some(token.clone());
        Ok(token)
    }
}
```

## UI Structure

### File Structure

```
mobile/src/
├── dure_sijang_app.rs          # Main app (UPDATE: integrate BrowserUI)
├── browser_ui.rs               # NEW: Tab browser UI (from tabbrowser.rs)
├── webview_tab.rs              # NEW: WebView tab wrapper
├── api_tab.rs                  # NEW: API mode tab (native egui widgets)
├── viewmodel/
│   ├── mod.rs                  # UPDATE: Remove old actors, add new
│   ├── browser.rs              # NEW: BrowserActor
│   ├── directory.rs            # NEW: StoreDirectoryActor
│   ├── common.rs               # UPDATE: New event types
│   ├── debloat.rs              # REMOVE
│   ├── scan.rs                 # REMOVE
│   ├── apps.rs                 # REMOVE
│   └── metadata.rs             # REMOVE
├── api_mycart.rs               # NEW: mycart API client
├── db_browser.rs               # NEW: Browser-related DB ops
├── db_directory.rs             # NEW: Directory DB ops
└── models.rs                   # UPDATE: Add browser models
```

### BrowserUI Component

**Based on** `reference/egui_webview/examples/tabbrowser.rs`

**Layout**:
```
┌────────────────────────────────────────────────────────┐
│ Sidebar Toggle │ ◀ ▶ │ URL: [_______________] │ Go │ ⭐ │  ← Toolbar
├─────────────┬──────────────────────────────────────────┤
│             │ Tab 1 │ Tab 2 │ Tab 3 │ + │              │  ← Tab Bar
│   Stores    ├──────────────────────────────────────────┤
│   ========  │                                          │
│   ☑ Demo    │                                          │
│   ☑ Fashion │         Active Tab Content               │  ← Content
│   ☐ Food    │         (Webview or API mode)            │
│             │                                          │
│   + Custom  │                                          │
│             │                                          │
└─────────────┴──────────────────────────────────────────┘
```

**Key Components**:

1. **Left Sidebar** (collapsible, resizable 80-200px)
   - Store directory list
   - Checkboxes to show/hide stores
   - Search/filter stores
   - "+ Custom Store" button

2. **Top Toolbar**
   - Sidebar toggle (◀◀ / ▶▶)
   - Back button (◀)
   - Forward button (▶)
   - URL input field
   - Go button
   - Mode toggle (Webview ⟷ API)
   - Bookmark button (⭐)

3. **Tab Bar**
   - Horizontal scrollable tabs
   - Tab buttons (selectable, show title)
   - Close buttons (×) per tab
   - + button to add new tab
   - Empty state when no tabs open

4. **Central Content Area**
   - **Webview mode**: Renders `EguiWebView` widget
   - **API mode**: Native egui widgets (product grid, cart view)
   - Only active tab visible
   - All webviews remain in memory (for faster switching)

### Mode Switching Flow

```
User clicks "Toggle Mode" button
    ↓
UI sends BrowserCommand::ToggleMode { tab_id, new_mode }
    ↓
BrowserActor receives command
    ↓
If switching to API mode:
    - Destroy webview widget
    - Fetch products via mycart API (GET /api/products)
    - Emit BrowserEvent::ProductsLoaded
    ↓
If switching to WebView mode:
    - Create new EguiWebView
    - Load current URL in webview
    - Emit BrowserEvent::TabUpdated
    ↓
UI updates tab state in ViewModelState
    ↓
UI re-renders with new mode
```

### API Mode UI Rendering

**Product Grid** (`mobile/src/api_tab.rs`):
```rust
pub fn render_products(ui: &mut egui::Ui, products: &[Product], viewmodel: &ViewModel) {
    egui::ScrollArea::vertical().show(ui, |ui| {
        egui::Grid::new("product_grid")
            .num_columns(3)
            .spacing([20.0, 20.0])
            .show(ui, |ui| {
                for (idx, product) in products.iter().enumerate() {
                    if idx > 0 && idx % 3 == 0 {
                        ui.end_row();
                    }
                    
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            // Product image (if available)
                            if let Some(img_url) = &product.image_url {
                                ui.label(format!("🖼️ {}", img_url));
                            }
                            
                            // Product name
                            ui.heading(&product.name);
                            
                            // Price
                            ui.label(format!("${:.2}", product.price));
                            
                            // Add to cart button
                            if ui.button("Add to Cart").clicked() {
                                viewmodel.add_to_cart(product.id.clone(), 1).ok();
                            }
                        });
                    });
                }
            });
    });
}
```

**Cart View** (`mobile/src/api_tab.rs`):
```rust
pub fn render_cart(ui: &mut egui::Ui, cart: &Cart, viewmodel: &ViewModel) {
    ui.heading("Shopping Cart");
    ui.separator();
    
    for item in &cart.items {
        ui.horizontal(|ui| {
            ui.label(&item.product_id);
            ui.label(format!("x{}", item.quantity));
            ui.label(format!("${:.2}", item.price * item.quantity as f64));
        });
    }
    
    ui.separator();
    ui.horizontal(|ui| {
        ui.label("Total:");
        ui.label(format!("${:.2}", cart.total));
    });
    
    if ui.button("Checkout").clicked() {
        viewmodel.checkout(cart.id.clone()).ok();
    }
}
```

## Migration Plan

### Phase 1: Database Migration

**Create migrations** (in order):
1. `create_store_directory.sql` - Store directory table
2. `create_tabs.sql` - Tabs table
3. `create_bookmarks.sql` - Bookmarks table
4. `create_browsing_history.sql` - History table
5. `create_cached_products.sql` - Product cache table
6. `create_cached_carts.sql` - Cart cache table
7. `create_user_credentials.sql` - Credentials table
8. `create_user_preferences.sql` - Preferences table

**Drop old tables** (in migration down.sql):
- `virustotal_results`
- `hybridanalysis_results`
- `package_info_cache`
- `google_play_apps`
- `fdroid_apps`
- `apkmirror_apps`

### Phase 2: Remove Dure-Sijang Code

**Files to DELETE:**
```
mobile/src/viewmodel/debloat.rs
mobile/src/viewmodel/scan.rs
mobile/src/viewmodel/apps.rs
mobile/src/viewmodel/metadata.rs
mobile/src/tab_debloat_control.rs
mobile/src/tab_scan_control.rs
mobile/src/tab_apps_control.rs
mobile/src/tab_usage_control.rs
mobile/src/tab_*_stt.rs
mobile/src/adb.rs
mobile/src/adb_stt.rs
mobile/src/android_shizuku.rs
mobile/src/android_*.rs
mobile/src/api_virustotal.rs
mobile/src/api_hybridanalysis.rs
mobile/src/api_googleplay.rs
mobile/src/api_fdroid.rs
mobile/src/api_apkmirror.rs
mobile/src/db_virustotal.rs
mobile/src/db_hybridanalysis.rs
mobile/src/db_package_cache.rs
mobile/src/db_googleplay.rs
mobile/src/db_fdroid.rs
mobile/src/db_apkmirror.rs
mobile/src/calc_virustotal_stt.rs
mobile/src/calc_hybridanalysis_stt.rs
mobile/src/calc_stalkerware_stt.rs
mobile/src/calc_*.rs
mobile/src/app_operations_queue.rs
mobile/src/dlg_*.rs
mobile/resources/uad_lists.json
mobile/resources/stalkerware_ioc.yaml
```

**Files to UPDATE:**
```
mobile/src/viewmodel/mod.rs
mobile/src/viewmodel/common.rs
mobile/src/dure_sijang_app.rs
mobile/src/lib.rs
mobile/src/models.rs
mobile/src/db.rs
mobile/Cargo.toml
```

### Phase 3: Add Browser Code

**New Files to CREATE:**
```
mobile/src/viewmodel/browser.rs
mobile/src/viewmodel/directory.rs
mobile/src/browser_ui.rs
mobile/src/webview_tab.rs
mobile/src/api_tab.rs
mobile/src/api_mycart.rs
mobile/src/db_browser.rs
mobile/src/db_directory.rs
mobile/src/models/browser.rs
mobile/src/models/mycart.rs
```

### Phase 4: Update Documentation

**CLAUDE.md Updates:**
- Section 1: Overview - describe mycart browser
- Section 2: Features - webview/API mode browsing
- Section 3: Repository Structure - browser modules
- Section 4: Architecture - BrowserActor/DirectoryActor
- Section 8: Recent Changes - webview migration
- Section 10: Code Style - remove adb/calc prefixes

**README.md Updates:**
- Title: "Dure-Sijang"
- Subtitle: "mycart designated browser"
- Features: webview/API mode browsing
- Usage: mycart browsing instructions
- Settings: browser settings
- Remove: Debloat/Scan/App sections

### Phase 5: Dependencies

**Add to Cargo.toml:**
```toml
[dependencies]
egui_webview = "0.5"
wry = "0.47"
gtk = { version = "0.18", optional = true }

[target.'cfg(any(target_os = "linux", target_os = "openbsd"))'.dependencies]
gtk = "0.18"
```

**Remove:**
- Dependencies only used for ADB/debloat (verify before removing)

### Migration Execution Order

1. Create database migrations (Phase 1)
2. Add new browser files (Phase 3)
3. Update ViewModel to use new actors (Phase 2)
4. Update DureSijangApp to use BrowserUI (Phase 2)
5. Test webview mode
6. Test API mode
7. Delete old files (Phase 2)
8. Update documentation (Phase 4)
9. Final integration testing

## Testing Strategy

### Unit Tests

**BrowserActor** (`mobile/src/viewmodel/browser.rs`):
- `test_open_tab_webview_mode` - Open tab in webview mode
- `test_open_tab_api_mode` - Open tab in API mode
- `test_toggle_mode` - Switch between modes
- `test_close_tab` - Close tab correctly
- `test_bookmark_management` - Add/remove bookmarks

**StoreDirectoryActor** (`mobile/src/viewmodel/directory.rs`):
- `test_fetch_directory` - Fetch from dure.one
- `test_add_custom_store` - Add custom store
- `test_toggle_store_active` - Enable/disable stores

**mycart API Client** (`mobile/src/api_mycart.rs`):
- `test_fetch_products` - Mock API, verify parsing
- `test_create_cart` - Cart creation
- `test_admin_login` - Authentication

### Integration Tests

**Browser Integration** (`mobile/tests/browser_integration.rs`):
```rust
#[test]
fn test_browser_lifecycle() {
    smol::block_on(async {
        // Create channels
        // Spawn BrowserActor
        // Send OpenTab command
        // Verify TabOpened event
        // Send FetchProducts command
        // Verify ProductsLoaded event
    });
}
```

**Database Integration** (`mobile/tests/db_browser_integration.rs`):
- `test_bookmark_persistence` - Save/load bookmarks
- `test_session_restore` - Restore tabs after restart

### Manual Testing Checklist

**Webview Mode:**
- [ ] Open tab in webview mode
- [ ] Navigate to mycart store
- [ ] Use back/forward buttons
- [ ] Enter URL in address bar
- [ ] Add bookmark
- [ ] Close tab
- [ ] Reopen from bookmark

**API Mode:**
- [ ] Open tab in API mode
- [ ] Fetch products list
- [ ] View product details
- [ ] Add product to cart
- [ ] View cart
- [ ] Admin login
- [ ] Create/edit product (admin)

**Mode Switching:**
- [ ] Toggle webview → API
- [ ] Toggle API → webview
- [ ] Verify state persists

**Store Directory:**
- [ ] Fetch directory from dure.one
- [ ] Add custom store
- [ ] Remove store
- [ ] Enable/disable store

**Session Persistence:**
- [ ] Open multiple tabs
- [ ] Close app
- [ ] Reopen app
- [ ] Verify tabs restored

**Cross-Platform:**
- [ ] Linux (GTK webview)
- [ ] macOS (WebKit webview)
- [ ] Windows (WebView2)
- [ ] Android (WebView)

### Error Handling Tests

**Network Errors:**
- [ ] Directory fetch fails
- [ ] API request fails
- [ ] Timeout handling

**Invalid Data:**
- [ ] Malformed directory JSON
- [ ] Invalid mycart API response
- [ ] Missing required fields

**State Errors:**
- [ ] Close non-existent tab
- [ ] Navigate on closed tab
- [ ] Toggle mode on invalid tab

## Implementation Notes

### Critical WebView Requirements

From `reference/egui_webview/examples/tabbrowser.rs`:

**1. GTK Initialization (Linux/OpenBSD)**
```rust
#[cfg(any(target_os = "linux", target_os = "openbsd"))]
gtk::init().expect("Failed to initialize GTK");
```

**2. GTK Event Processing (in main loop)**
```rust
#[cfg(any(target_os = "linux", target_os = "openbsd"))]
{
    while gtk::events_pending() {
        gtk::main_iteration_do(false);
    }
}
```

**3. Continuous Repaint (CRITICAL)**
```rust
// Without this, webview input has 3+ second delays
ctx.request_repaint();
```

**4. WebView Lifecycle**
```rust
// Initialize once at app start
init_webview(ctx);

// At end of each frame
webview_end_frame(ctx);
```

### Database Connection Pooling

- Use single database connection per application instance
- Migrations run automatically on first connection
- All DB operations use async smol runtime

### MVVM Best Practices

- UI never mutates state directly
- All state changes via Command → Actor → Event → State update
- ViewModelState is read-only from UI
- Use `poll_events()` in `update()` to process events

### Error Handling

- Use `anyhow::Result` for all fallible operations
- Emit `BrowserEvent::Error` or `DirectoryEvent::Error` for user-facing errors
- Log detailed errors with `log::error!`
- Never `.unwrap()` in production code

## Success Criteria

1. ✅ All Dure-Sijang code removed
2. ✅ BrowserActor and StoreDirectoryActor implemented
3. ✅ Tab-based UI with webview mode working
4. ✅ API mode displaying products and cart
5. ✅ Directory fetching from dure.one
6. ✅ Bookmarks and history persistence
7. ✅ Session restore on app restart
8. ✅ CLAUDE.md and README.md updated
9. ✅ All tests passing
10. ✅ Cross-platform builds successful

## Open Questions

None - all design decisions validated with user.

## References

- Reference webview implementation: `reference/egui_webview/examples/tabbrowser.rs`
- Reference webview (single window): `reference/egui_webview/examples/webview.rs`
- Current MVVM architecture: `mobile/src/viewmodel/mod.rs`
- mycart API specification: https://nikescar.github.io/mycart/swagger/swagger.json
- egui_webview documentation: https://docs.rs/egui_webview
- wry documentation: https://docs.rs/wry
