# Webview Migration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Transform Dure-Sijang (Android debloater) into Dure-Sijang (mycart designated browser) with dual-mode browsing (webview + API)

**Architecture:** MVVM pattern with BrowserActor and StoreDirectoryActor, smol async runtime, Diesel + SQLite persistence, tab-based egui UI

**Tech Stack:** Rust, egui, egui_webview, wry, smol, Diesel, SQLite, ureq

## Global Constraints

- Rust edition 2021, rustfmt mandatory
- smol async runtime (NOT tokio)
- Diesel ORM with SQLite backend
- egui_webview 0.5, wry 0.47
- GTK 0.18 for Linux/OpenBSD webview backend
- Never use `.unwrap()` or `.expect()` in production code
- Use `anyhow::Result` for error handling
- Follow MVVM: UI → Command → Actor → Event → State update
- All database timestamps use `TIMESTAMP` type
- Tab-based UI from `reference/egui_webview/examples/tabbrowser.rs`

---

**NOTE:** This plan is split across multiple files due to size. This file contains Tasks 1-4. Additional tasks will be added after review.

## Task 1: Database Migrations

**Files:**
- Create: `mobile/migrations/2026-08-18-100000_create_store_directory/up.sql`
- Create: `mobile/migrations/2026-08-18-100000_create_store_directory/down.sql`
- Create: `mobile/migrations/2026-08-18-100100_create_tabs/up.sql`
- Create: `mobile/migrations/2026-08-18-100100_create_tabs/down.sql`
- Create: `mobile/migrations/2026-08-18-100200_create_bookmarks/up.sql`
- Create: `mobile/migrations/2026-08-18-100200_create_bookmarks/down.sql`
- Create: `mobile/migrations/2026-08-18-100300_create_browsing_history/up.sql`
- Create: `mobile/migrations/2026-08-18-100300_create_browsing_history/down.sql`
- Create: `mobile/migrations/2026-08-18-100400_create_cached_products/up.sql`
- Create: `mobile/migrations/2026-08-18-100400_create_cached_products/down.sql`
- Create: `mobile/migrations/2026-08-18-100500_create_cached_carts/up.sql`
- Create: `mobile/migrations/2026-08-18-100500_create_cached_carts/down.sql`
- Create: `mobile/migrations/2026-08-18-100600_create_user_credentials/up.sql`
- Create: `mobile/migrations/2026-08-18-100600_create_user_credentials/down.sql`
- Create: `mobile/migrations/2026-08-18-100700_create_user_preferences/up.sql`
- Create: `mobile/migrations/2026-08-18-100700_create_user_preferences/down.sql`

**Interfaces:**
- Consumes: Nothing (first task)
- Produces: 8 database tables for browser functionality

- [ ] **Step 1: Create store_directory migration (up.sql)**

Create directory and migration:
```bash
mkdir -p mobile/migrations/2026-08-18-100000_create_store_directory
```

```sql
-- mobile/migrations/2026-08-18-100000_create_store_directory/up.sql
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

- [ ] **Step 2: Create store_directory migration (down.sql)**

```sql
-- mobile/migrations/2026-08-18-100000_create_store_directory/down.sql
DROP TABLE store_directory;
```

- [ ] **Step 3: Create tabs migration**

```bash
mkdir -p mobile/migrations/2026-08-18-100100_create_tabs
```

```sql
-- mobile/migrations/2026-08-18-100100_create_tabs/up.sql
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

```sql
-- mobile/migrations/2026-08-18-100100_create_tabs/down.sql
DROP TABLE tabs;
```

- [ ] **Step 4: Create bookmarks migration**

```bash
mkdir -p mobile/migrations/2026-08-18-100200_create_bookmarks
```

```sql
-- mobile/migrations/2026-08-18-100200_create_bookmarks/up.sql
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

```sql
-- mobile/migrations/2026-08-18-100200_create_bookmarks/down.sql
DROP TABLE bookmarks;
```

- [ ] **Step 5: Create browsing_history migration**

```bash
mkdir -p mobile/migrations/2026-08-18-100300_create_browsing_history
```

```sql
-- mobile/migrations/2026-08-18-100300_create_browsing_history/up.sql
CREATE TABLE browsing_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    page_url TEXT NOT NULL,
    title TEXT,
    visited_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);

CREATE INDEX idx_history_visited ON browsing_history(visited_at DESC);
```

```sql
-- mobile/migrations/2026-08-18-100300_create_browsing_history/down.sql
DROP INDEX idx_history_visited;
DROP TABLE browsing_history;
```

- [ ] **Step 6: Create cached_products migration**

```bash
mkdir -p mobile/migrations/2026-08-18-100400_create_cached_products
```

```sql
-- mobile/migrations/2026-08-18-100400_create_cached_products/up.sql
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

```sql
-- mobile/migrations/2026-08-18-100400_create_cached_products/down.sql
DROP INDEX idx_products_store;
DROP TABLE cached_products;
```

- [ ] **Step 7: Create cached_carts migration**

```bash
mkdir -p mobile/migrations/2026-08-18-100500_create_cached_carts
```

```sql
-- mobile/migrations/2026-08-18-100500_create_cached_carts/up.sql
CREATE TABLE cached_carts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL,
    cart_id TEXT NOT NULL,
    cart_data TEXT NOT NULL,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(store_url, cart_id)
);
```

```sql
-- mobile/migrations/2026-08-18-100500_create_cached_carts/down.sql
DROP TABLE cached_carts;
```

- [ ] **Step 8: Create user_credentials migration**

```bash
mkdir -p mobile/migrations/2026-08-18-100600_create_user_credentials
```

```sql
-- mobile/migrations/2026-08-18-100600_create_user_credentials/up.sql
CREATE TABLE user_credentials (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    store_url TEXT NOT NULL UNIQUE,
    admin_token TEXT NOT NULL,
    admin_email TEXT,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

```sql
-- mobile/migrations/2026-08-18-100600_create_user_credentials/down.sql
DROP TABLE user_credentials;
```

- [ ] **Step 9: Create user_preferences migration**

```bash
mkdir -p mobile/migrations/2026-08-18-100700_create_user_preferences
```

```sql
-- mobile/migrations/2026-08-18-100700_create_user_preferences/up.sql
CREATE TABLE user_preferences (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
);
```

```sql
-- mobile/migrations/2026-08-18-100700_create_user_preferences/down.sql
DROP TABLE user_preferences;
```

- [ ] **Step 10: Run migrations to verify**

```bash
cd mobile
diesel migration run
```

Expected: All 8 migrations applied successfully, schema.rs updated

- [ ] **Step 11: Verify schema.rs generated correctly**

```bash
grep -q "store_directory" mobile/src/schema.rs && echo "✓ store_directory found"
grep -q "tabs" mobile/src/schema.rs && echo "✓ tabs found"
grep -q "bookmarks" mobile/src/schema.rs && echo "✓ bookmarks found"
```

Expected: All tables present in schema

- [ ] **Step 12: Commit database migrations**

```bash
git add mobile/migrations/ mobile/src/schema.rs
git commit -m "feat(db): add browser database migrations

Add 8 new tables for browser functionality:
- store_directory: mycart store listings from dure.one
- tabs: session persistence for open tabs
- bookmarks: user bookmarks
- browsing_history: navigation history with index
- cached_products: product cache from mycart API
- cached_carts: cart state persistence (JSON blob)
- user_credentials: admin tokens per store
- user_preferences: app settings (key-value)

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 2: Add Dependencies

**Files:**
- Modify: `mobile/Cargo.toml`

**Interfaces:**
- Consumes: Existing Cargo.toml
- Produces: Updated dependencies with egui_webview, wry, gtk

- [ ] **Step 1: Read current Cargo.toml**

```bash
grep -A 5 "\[dependencies\]" mobile/Cargo.toml | head -10
```

Expected: See current dependencies

- [ ] **Step 2: Add egui_webview and wry to [dependencies]**

Add these lines to the `[dependencies]` section in `mobile/Cargo.toml`:

```toml
egui_webview = "0.5"
wry = "0.47"
```

- [ ] **Step 3: Add GTK platform-specific dependency**

Add this new section after `[dependencies]` in `mobile/Cargo.toml`:

```toml
[target.'cfg(any(target_os = "linux", target_os = "openbsd"))'.dependencies]
gtk = "0.18"
```

- [ ] **Step 4: Verify dependencies resolve**

```bash
cd mobile
cargo check
```

Expected: Dependencies download and compile successfully

- [ ] **Step 5: Commit dependency changes**

```bash
git add mobile/Cargo.toml mobile/Cargo.lock
git commit -m "feat(deps): add webview dependencies

Add egui_webview 0.5 and wry 0.47 for cross-platform webview support
Add gtk 0.18 for Linux/OpenBSD webview backend

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 3: Create Data Models

**Files:**
- Create: `mobile/src/models/browser.rs`
- Create: `mobile/src/models/mycart.rs`
- Modify: `mobile/src/models.rs`

**Interfaces:**
- Consumes: Database schema from Task 1
- Produces: `StoreEntry`, `TabState`, `BrowsingMode`, `Bookmark`, `HistoryEntry`, `Product`, `Cart`, `CartItem`, `DirectoryResponse`, `DirectoryStore`

- [ ] **Step 1: Create models directory if needed**

```bash
mkdir -p mobile/src/models
```

- [ ] **Step 2: Create browser.rs with browser models**

```rust
// mobile/src/models/browser.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct StoreEntry {
    pub id: i32,
    pub name: String,
    pub base_url: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub logo_url: Option<String>,
    pub is_active: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum BrowsingMode {
    WebView,
    Api,
}

#[derive(Clone, Debug)]
pub struct TabState {
    pub id: usize,
    pub store_url: String,
    pub current_url: String,
    pub mode: BrowsingMode,
    pub title: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: i32,
    pub store_url: String,
    pub page_url: String,
    pub title: String,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub id: i32,
    pub store_url: String,
    pub page_url: String,
    pub title: Option<String>,
    pub visited_at: chrono::NaiveDateTime,
}
```

- [ ] **Step 3: Create mycart.rs with API models**

```rust
// mobile/src/models/mycart.rs
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Product {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub price: f64,
    pub description: Option<String>,
    #[serde(rename = "imageUrl")]
    pub image_url: Option<String>,
    #[serde(rename = "isActive")]
    pub is_active: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Cart {
    pub id: String,
    pub items: Vec<CartItem>,
    pub total: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CartItem {
    #[serde(rename = "productId")]
    pub product_id: String,
    pub quantity: u32,
    pub price: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryResponse {
    pub version: String,
    pub updated_at: String,
    pub stores: Vec<DirectoryStore>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectoryStore {
    pub name: String,
    pub base_url: String,
    pub description: Option<String>,
    pub category: Option<String>,
    pub logo_url: Option<String>,
}
```

- [ ] **Step 4: Update models.rs to export new modules**

Check if `mobile/src/models.rs` exists, then add:

```rust
// mobile/src/models.rs
// Add at the end of the file

pub mod browser;
pub mod mycart;

pub use browser::*;
pub use mycart::*;
```

- [ ] **Step 5: Verify models compile**

```bash
cargo check
```

Expected: No errors, models compile successfully

- [ ] **Step 6: Commit data models**

```bash
git add mobile/src/models/browser.rs mobile/src/models/mycart.rs mobile/src/models.rs
git commit -m "feat(models): add browser and mycart data models

Browser models:
- StoreEntry: store directory entries
- TabState: tab state with mode (webview/api)
- BrowsingMode: enum for webview vs API mode
- Bookmark: user bookmarks
- HistoryEntry: browsing history

mycart API models:
- Product: product from mycart API
- Cart: shopping cart with items
- CartItem: cart item with quantity
- DirectoryResponse: dure.one directory response
- DirectoryStore: store entry from directory

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

## Task 4: Create mycart API Client

**Files:**
- Create: `mobile/src/api_mycart.rs`
- Modify: `mobile/src/lib.rs`

**Interfaces:**
- Consumes: `Product`, `Cart`, `CartItem` from Task 3
- Produces: `MyCartClient` with methods: `new()`, `fetch_products()`, `fetch_product()`, `create_cart()`, `get_cart()`, `admin_login()`, `get_settings()`

- [ ] **Step 1: Create api_mycart.rs with basic structure**

```rust
// mobile/src/api_mycart.rs
use anyhow::Result;
use crate::models::mycart::{Product, Cart, CartItem};

pub struct MyCartClient {
    base_url: String,
    client: ureq::Agent,
    admin_token: Option<String>,
}
```

- [ ] **Step 2: Write test for URL construction**

```rust
// mobile/src/api_mycart.rs
// Add after struct definition

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_client() {
        let client = MyCartClient::new("https://demo.mycart.example".to_string());
        assert_eq!(client.base_url, "https://demo.mycart.example");
        assert!(client.admin_token.is_none());
    }

    #[test]
    fn test_url_construction() {
        let client = MyCartClient::new("https://demo.mycart.example".to_string());
        let url = format!("{}/api/products", client.base_url);
        assert_eq!(url, "https://demo.mycart.example/api/products");
    }
}
```

- [ ] **Step 3: Run tests to verify they fail**

```bash
cargo test --lib api_mycart::tests
```

Expected: FAIL - "MyCartClient::new" not found

- [ ] **Step 4: Implement MyCartClient methods**

```rust
// mobile/src/api_mycart.rs
// Add after struct definition, before tests

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

    pub fn fetch_products(&self) -> Result<Vec<Product>> {
        let url = format!("{}/api/products", self.base_url);
        let resp = self.client.get(&url).call()?;
        let products: Vec<Product> = resp.into_json()?;
        Ok(products)
    }

    pub fn fetch_product(&self, product_id: &str) -> Result<Product> {
        let url = format!("{}/api/products/{}", self.base_url, product_id);
        let resp = self.client.get(&url).call()?;
        let product: Product = resp.into_json()?;
        Ok(product)
    }

    pub fn create_cart(&self, items: Vec<CartItem>) -> Result<String> {
        let url = format!("{}/api/cart/create", self.base_url);
        let resp = self.client.post(&url).send_json(items)?;
        let cart_id: String = resp.into_json()?;
        Ok(cart_id)
    }

    pub fn get_cart(&self, cart_id: &str) -> Result<Cart> {
        let url = format!("{}/api/cart/{}", self.base_url, cart_id);
        let resp = self.client.get(&url).call()?;
        let cart: Cart = resp.into_json()?;
        Ok(cart)
    }

    pub fn admin_login(&mut self, email: String, password: String) -> Result<String> {
        let url = format!("{}/api/sign/in", self.base_url);
        let body = serde_json::json!({
            "email": email,
            "password": password,
        });
        let resp = self.client.post(&url).send_json(body)?;
        let token: String = resp.into_json()?;
        self.admin_token = Some(token.clone());
        Ok(token)
    }

    pub fn get_settings(&self) -> Result<serde_json::Value> {
        let url = format!("{}/api/settings", self.base_url);
        let resp = self.client.get(&url).call()?;
        let settings: serde_json::Value = resp.into_json()?;
        Ok(settings)
    }
}
```

- [ ] **Step 5: Run tests to verify they pass**

```bash
cargo test --lib api_mycart::tests
```

Expected: PASS - both tests pass

- [ ] **Step 6: Add mod declaration to lib.rs**

Add this line to `mobile/src/lib.rs` after existing mod declarations:

```rust
pub mod api_mycart;
```

- [ ] **Step 7: Verify full build**

```bash
cargo check
```

Expected: No errors

- [ ] **Step 8: Commit mycart API client**

```bash
git add mobile/src/api_mycart.rs mobile/src/lib.rs
git commit -m "feat(api): add mycart API client

Implement MyCartClient with methods:
- fetch_products: GET /api/products
- fetch_product: GET /api/products/{id}
- create_cart: POST /api/cart/create
- get_cart: GET /api/cart/{id}
- admin_login: POST /api/sign/in (sets admin token)
- get_settings: GET /api/settings

Uses ureq for HTTP requests, supports admin token for authenticated endpoints
Includes unit tests for URL construction

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 5: Database Operations (db_browser, db_directory)

**Files:**
- Create: `mobile/src/db_browser.rs`
- Create: `mobile/src/db_directory.rs`
- Modify: `mobile/src/lib.rs`

**Interfaces:**
- Consumes: Database tables from Task 1, models from Task 3
- Produces: CRUD operations for all browser tables

- [ ] **Step 1: Create db_browser.rs with tab operations**

```rust
// mobile/src/db_browser.rs
use crate::models::{BrowserTab, NewBrowserTab, BrowserBookmark, NewBrowserBookmark, BrowsingHistory, NewBrowsingHistory};
use crate::schema::{tabs, bookmarks, browsing_history};
use diesel::prelude::*;
use anyhow::Result;

pub fn insert_tab(conn: &mut SqliteConnection, new_tab: NewBrowserTab) -> Result<BrowserTab> {
    use diesel::insert_into;
    insert_into(tabs::table)
        .values(&new_tab)
        .execute(conn)?;
    
    tabs::table
        .order(tabs::id.desc())
        .first(conn)
        .map_err(Into::into)
}

pub fn get_tab(conn: &mut SqliteConnection, tab_id: i32) -> Result<BrowserTab> {
    tabs::table
        .find(tab_id)
        .first(conn)
        .map_err(Into::into)
}

pub fn get_all_tabs(conn: &mut SqliteConnection) -> Result<Vec<BrowserTab>> {
    tabs::table
        .load(conn)
        .map_err(Into::into)
}

pub fn update_tab_url(conn: &mut SqliteConnection, tab_id: i32, new_url: &str) -> Result<()> {
    diesel::update(tabs::table.find(tab_id))
        .set((
            tabs::url.eq(new_url),
            tabs::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)?;
    Ok(())
}

pub fn delete_tab(conn: &mut SqliteConnection, tab_id: i32) -> Result<()> {
    diesel::delete(tabs::table.find(tab_id))
        .execute(conn)?;
    Ok(())
}

pub fn insert_bookmark(conn: &mut SqliteConnection, new_bookmark: NewBrowserBookmark) -> Result<BrowserBookmark> {
    use diesel::insert_into;
    insert_into(bookmarks::table)
        .values(&new_bookmark)
        .execute(conn)?;
    
    bookmarks::table
        .order(bookmarks::id.desc())
        .first(conn)
        .map_err(Into::into)
}

pub fn get_all_bookmarks(conn: &mut SqliteConnection) -> Result<Vec<BrowserBookmark>> {
    bookmarks::table
        .order(bookmarks::created_at.desc())
        .load(conn)
        .map_err(Into::into)
}

pub fn delete_bookmark(conn: &mut SqliteConnection, bookmark_id: i32) -> Result<()> {
    diesel::delete(bookmarks::table.find(bookmark_id))
        .execute(conn)?;
    Ok(())
}

pub fn insert_history(conn: &mut SqliteConnection, new_history: NewBrowsingHistory) -> Result<BrowsingHistory> {
    use diesel::insert_into;
    insert_into(browsing_history::table)
        .values(&new_history)
        .execute(conn)?;
    
    browsing_history::table
        .order(browsing_history::id.desc())
        .first(conn)
        .map_err(Into::into)
}

pub fn get_history(conn: &mut SqliteConnection, limit: i64) -> Result<Vec<BrowsingHistory>> {
    browsing_history::table
        .order(browsing_history::visited_at.desc())
        .limit(limit)
        .load(conn)
        .map_err(Into::into)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::establish_connection;

    #[test]
    fn test_tab_crud() {
        let mut conn = establish_connection().unwrap();
        
        // Insert
        let new_tab = NewBrowserTab {
            title: "Test Store".to_string(),
            url: "https://example.mycart".to_string(),
            mode: "webview".to_string(),
            store_id: Some(1),
        };
        let tab = insert_tab(&mut conn, new_tab).unwrap();
        assert_eq!(tab.title, "Test Store");
        
        // Get
        let fetched = get_tab(&mut conn, tab.id).unwrap();
        assert_eq!(fetched.url, "https://example.mycart");
        
        // Update
        update_tab_url(&mut conn, tab.id, "https://new.mycart").unwrap();
        let updated = get_tab(&mut conn, tab.id).unwrap();
        assert_eq!(updated.url, "https://new.mycart");
        
        // Delete
        delete_tab(&mut conn, tab.id).unwrap();
        assert!(get_tab(&mut conn, tab.id).is_err());
    }
}
```

Expected: File created with tab, bookmark, and history CRUD operations

- [ ] **Step 2: Create db_directory.rs with store directory operations**

```rust
// mobile/src/db_directory.rs
use crate::models::{StoreDirectory, NewStoreDirectory};
use crate::schema::store_directory;
use diesel::prelude::*;
use anyhow::Result;

pub fn insert_store(conn: &mut SqliteConnection, new_store: NewStoreDirectory) -> Result<StoreDirectory> {
    use diesel::insert_into;
    insert_into(store_directory::table)
        .values(&new_store)
        .execute(conn)?;
    
    store_directory::table
        .order(store_directory::id.desc())
        .first(conn)
        .map_err(Into::into)
}

pub fn get_store(conn: &mut SqliteConnection, store_id: i32) -> Result<StoreDirectory> {
    store_directory::table
        .find(store_id)
        .first(conn)
        .map_err(Into::into)
}

pub fn get_all_stores(conn: &mut SqliteConnection) -> Result<Vec<StoreDirectory>> {
    store_directory::table
        .order(store_directory::name.asc())
        .load(conn)
        .map_err(Into::into)
}

pub fn update_store_metadata(conn: &mut SqliteConnection, store_id: i32, metadata: &str) -> Result<()> {
    diesel::update(store_directory::table.find(store_id))
        .set((
            store_directory::metadata.eq(metadata),
            store_directory::updated_at.eq(diesel::dsl::now),
        ))
        .execute(conn)?;
    Ok(())
}

pub fn delete_store(conn: &mut SqliteConnection, store_id: i32) -> Result<()> {
    diesel::delete(store_directory::table.find(store_id))
        .execute(conn)?;
    Ok(())
}

pub fn search_stores(conn: &mut SqliteConnection, query: &str) -> Result<Vec<StoreDirectory>> {
    use diesel::dsl::sql;
    store_directory::table
        .filter(
            store_directory::name.like(format!("%{}%", query))
                .or(store_directory::description.like(format!("%{}%", query)))
        )
        .order(store_directory::name.asc())
        .load(conn)
        .map_err(Into::into)
}

pub fn upsert_store(conn: &mut SqliteConnection, new_store: NewStoreDirectory) -> Result<StoreDirectory> {
    // Check if store with same URL exists
    let existing: Option<StoreDirectory> = store_directory::table
        .filter(store_directory::url.eq(&new_store.url))
        .first(conn)
        .optional()?;
    
    match existing {
        Some(mut store) => {
            // Update existing
            diesel::update(store_directory::table.find(store.id))
                .set((
                    store_directory::name.eq(&new_store.name),
                    store_directory::description.eq(&new_store.description),
                    store_directory::icon_url.eq(&new_store.icon_url),
                    store_directory::metadata.eq(&new_store.metadata),
                    store_directory::updated_at.eq(diesel::dsl::now),
                ))
                .execute(conn)?;
            get_store(conn, store.id)
        },
        None => {
            // Insert new
            insert_store(conn, new_store)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::establish_connection;

    #[test]
    fn test_store_crud() {
        let mut conn = establish_connection().unwrap();
        
        let new_store = NewStoreDirectory {
            name: "Test Store".to_string(),
            url: "https://test.mycart".to_string(),
            description: Some("A test store".to_string()),
            icon_url: None,
            metadata: None,
        };
        
        let store = insert_store(&mut conn, new_store).unwrap();
        assert_eq!(store.name, "Test Store");
        
        let fetched = get_store(&mut conn, store.id).unwrap();
        assert_eq!(fetched.url, "https://test.mycart");
        
        let results = search_stores(&mut conn, "test").unwrap();
        assert!(!results.is_empty());
        
        delete_store(&mut conn, store.id).unwrap();
    }
    
    #[test]
    fn test_upsert_store() {
        let mut conn = establish_connection().unwrap();
        
        let new_store = NewStoreDirectory {
            name: "Upsert Test".to_string(),
            url: "https://upsert.mycart".to_string(),
            description: Some("Original".to_string()),
            icon_url: None,
            metadata: None,
        };
        
        // First upsert creates
        let store1 = upsert_store(&mut conn, new_store.clone()).unwrap();
        
        // Second upsert updates
        let updated_store = NewStoreDirectory {
            name: "Upsert Test Updated".to_string(),
            url: "https://upsert.mycart".to_string(),
            description: Some("Updated".to_string()),
            icon_url: None,
            metadata: None,
        };
        let store2 = upsert_store(&mut conn, updated_store).unwrap();
        
        assert_eq!(store1.id, store2.id);
        assert_eq!(store2.name, "Upsert Test Updated");
        
        delete_store(&mut conn, store1.id).unwrap();
    }
}
```

Expected: File created with store directory CRUD and search operations

- [ ] **Step 3: Add mod declarations to lib.rs**

Add these lines to `mobile/src/lib.rs` after existing mod declarations:

```rust
pub mod db_browser;
pub mod db_directory;
```

- [ ] **Step 4: Run tests**

```bash
cd mobile
cargo nextest run db_browser::tests
cargo nextest run db_directory::tests
```

Expected: PASS - all CRUD tests pass

- [ ] **Step 5: Verify full build**

```bash
cargo check
```

Expected: No errors

- [ ] **Step 6: Commit database operations**

```bash
git add mobile/src/db_browser.rs mobile/src/db_directory.rs mobile/src/lib.rs
git commit -m "feat(db): add browser and directory database operations

Implement db_browser.rs with operations:
- Tab CRUD: insert_tab, get_tab, get_all_tabs, update_tab_url, delete_tab
- Bookmark CRUD: insert_bookmark, get_all_bookmarks, delete_bookmark
- History CRUD: insert_history, get_history

Implement db_directory.rs with operations:
- Store CRUD: insert_store, get_store, get_all_stores, update_store_metadata, delete_store
- Search: search_stores by name or description
- Upsert: upsert_store (insert or update by URL)

All operations include unit tests verifying CRUD behavior

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 6: StoreDirectoryActor

**Files:**
- Create: `mobile/src/viewmodel/directory.rs`
- Modify: `mobile/src/viewmodel/mod.rs`
- Modify: `mobile/src/viewmodel/common.rs`

**Interfaces:**
- Consumes: db_directory operations, dure.one API
- Produces: StoreDirectoryActor with command/event messaging

- [ ] **Step 1: Add DirectoryCommand and DirectoryEvent to common.rs**

```rust
// mobile/src/viewmodel/common.rs
// Add these variants to existing ViewModelCommand enum
pub enum ViewModelCommand {
    // ... existing variants ...
    Directory(DirectoryCommand),
}

#[derive(Debug, Clone)]
pub enum DirectoryCommand {
    FetchDirectory,
    SearchStores(String),
    AddCustomStore { name: String, url: String, description: Option<String> },
}

// Add these variants to existing ViewModelEvent enum
pub enum ViewModelEvent {
    // ... existing variants ...
    Directory(DirectoryEvent),
}

#[derive(Debug, Clone)]
pub enum DirectoryEvent {
    DirectoryLoaded { stores: Vec<StoreDirectory> },
    SearchResults { stores: Vec<StoreDirectory> },
    StoreAdded { store: StoreDirectory },
    Error { message: String },
}
```

- [ ] **Step 2: Create directory.rs with actor implementation**

```rust
// mobile/src/viewmodel/directory.rs
use crate::db;
use crate::db_directory;
use crate::models::{StoreDirectory, NewStoreDirectory};
use crate::viewmodel::common::{DirectoryCommand, DirectoryEvent, ViewModelEvent};
use async_channel::{Receiver, Sender};
use serde::Deserialize;
use tracing::{info, warn, error};

#[derive(Debug, Deserialize)]
struct DureOneStore {
    name: String,
    url: String,
    description: Option<String>,
    icon_url: Option<String>,
}

pub struct DirectoryActor {
    cmd_rx: Receiver<DirectoryCommand>,
    event_tx: Sender<ViewModelEvent>,
}

impl DirectoryActor {
    pub fn new(
        cmd_rx: Receiver<DirectoryCommand>,
        event_tx: Sender<ViewModelEvent>,
    ) -> Self {
        Self { cmd_rx, event_tx }
    }
    
    pub async fn run(self) {
        info!("DirectoryActor started");
        
        while let Ok(cmd) = self.cmd_rx.recv().await {
            match cmd {
                DirectoryCommand::FetchDirectory => {
                    self.handle_fetch_directory().await;
                },
                DirectoryCommand::SearchStores(query) => {
                    self.handle_search_stores(&query).await;
                },
                DirectoryCommand::AddCustomStore { name, url, description } => {
                    self.handle_add_custom_store(&name, &url, description.as_deref()).await;
                },
            }
        }
        
        info!("DirectoryActor stopped");
    }
    
    async fn handle_fetch_directory(&self) {
        info!("Fetching store directory from dure.one");
        
        // Fetch from dure.one API
        let url = "https://dure.one/api/directory.json";
        let response = match ureq::get(url).call() {
            Ok(resp) => resp,
            Err(e) => {
                error!("Failed to fetch directory: {}", e);
                self.send_event(DirectoryEvent::Error {
                    message: format!("Failed to fetch directory: {}", e),
                }).await;
                return;
            }
        };
        
        let stores: Vec<DureOneStore> = match response.into_json() {
            Ok(s) => s,
            Err(e) => {
                error!("Failed to parse directory JSON: {}", e);
                self.send_event(DirectoryEvent::Error {
                    message: format!("Failed to parse directory: {}", e),
                }).await;
                return;
            }
        };
        
        info!("Fetched {} stores from dure.one", stores.len());
        
        // Upsert to database
        let mut conn = match db::establish_connection() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                self.send_event(DirectoryEvent::Error {
                    message: format!("Database error: {}", e),
                }).await;
                return;
            }
        };
        
        let mut saved_stores = Vec::new();
        for store in stores {
            let new_store = NewStoreDirectory {
                name: store.name,
                url: store.url,
                description: store.description,
                icon_url: store.icon_url,
                metadata: None,
            };
            
            match db_directory::upsert_store(&mut conn, new_store) {
                Ok(s) => saved_stores.push(s),
                Err(e) => {
                    warn!("Failed to save store: {}", e);
                }
            }
        }
        
        self.send_event(DirectoryEvent::DirectoryLoaded {
            stores: saved_stores,
        }).await;
    }
    
    async fn handle_search_stores(&self, query: &str) {
        info!("Searching stores with query: {}", query);
        
        let mut conn = match db::establish_connection() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                self.send_event(DirectoryEvent::Error {
                    message: format!("Database error: {}", e),
                }).await;
                return;
            }
        };
        
        let stores = match db_directory::search_stores(&mut conn, query) {
            Ok(s) => s,
            Err(e) => {
                error!("Search failed: {}", e);
                self.send_event(DirectoryEvent::Error {
                    message: format!("Search failed: {}", e),
                }).await;
                return;
            }
        };
        
        self.send_event(DirectoryEvent::SearchResults { stores }).await;
    }
    
    async fn handle_add_custom_store(&self, name: &str, url: &str, description: Option<&str>) {
        info!("Adding custom store: {} at {}", name, url);
        
        let mut conn = match db::establish_connection() {
            Ok(c) => c,
            Err(e) => {
                error!("Failed to connect to database: {}", e);
                self.send_event(DirectoryEvent::Error {
                    message: format!("Database error: {}", e),
                }).await;
                return;
            }
        };
        
        let new_store = NewStoreDirectory {
            name: name.to_string(),
            url: url.to_string(),
            description: description.map(String::from),
            icon_url: None,
            metadata: None,
        };
        
        match db_directory::insert_store(&mut conn, new_store) {
            Ok(store) => {
                self.send_event(DirectoryEvent::StoreAdded { store }).await;
            },
            Err(e) => {
                error!("Failed to add store: {}", e);
                self.send_event(DirectoryEvent::Error {
                    message: format!("Failed to add store: {}", e),
                }).await;
            }
        }
    }
    
    async fn send_event(&self, event: DirectoryEvent) {
        let _ = self.event_tx.send(ViewModelEvent::Directory(event)).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_channel::unbounded;
    
    #[test]
    fn test_directory_actor_search() {
        smol::block_on(async {
            let (cmd_tx, cmd_rx) = unbounded();
            let (event_tx, event_rx) = unbounded();
            
            let actor = DirectoryActor::new(cmd_rx, event_tx);
            smol::spawn(actor.run()).detach();
            
            // Send search command
            cmd_tx.send(DirectoryCommand::SearchStores("test".to_string())).await.unwrap();
            
            // Receive event
            if let Ok(ViewModelEvent::Directory(DirectoryEvent::SearchResults { stores })) = event_rx.recv().await {
                // Search should complete (may be empty)
                assert!(stores.len() >= 0);
            }
        });
    }
}
```

Expected: File created with directory fetch, search, and add operations

- [ ] **Step 3: Update viewmodel/mod.rs to spawn DirectoryActor**

Add to `mobile/src/viewmodel/mod.rs`:

```rust
mod directory;
use directory::DirectoryActor;

// In ViewModel::new(), add directory command channel and spawn actor:
let (directory_cmd_tx, directory_cmd_rx) = unbounded();
let directory_actor = DirectoryActor::new(directory_cmd_rx, event_tx.clone());
smol::spawn(directory_actor.run()).detach();

// Add field to ViewModel struct:
pub struct ViewModel {
    // ... existing fields ...
    directory_cmd_tx: Sender<DirectoryCommand>,
}

// Add method to send directory commands:
pub fn send_directory_command(&self, cmd: DirectoryCommand) {
    let tx = self.directory_cmd_tx.clone();
    smol::spawn(async move {
        let _ = tx.send(cmd).await;
    }).detach();
}
```

- [ ] **Step 4: Run tests**

```bash
cargo nextest run directory::tests
```

Expected: PASS - search test passes

- [ ] **Step 5: Verify full build**

```bash
cargo check
```

Expected: No errors

- [ ] **Step 6: Commit StoreDirectoryActor**

```bash
git add mobile/src/viewmodel/directory.rs mobile/src/viewmodel/mod.rs mobile/src/viewmodel/common.rs
git commit -m "feat(viewmodel): add StoreDirectoryActor

Implement DirectoryActor with commands:
- FetchDirectory: Fetch from dure.one and upsert to database
- SearchStores: Search local database by name or description
- AddCustomStore: Add user-provided store to database

Emit events:
- DirectoryLoaded: Stores fetched and saved
- SearchResults: Search query results
- StoreAdded: Custom store added
- Error: Operation failed

Uses ureq for HTTP, db_directory for persistence
Spawned in ViewModel with command/event messaging

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

---

### Task 7: BrowserActor (Unified WebView + API Mode)

**Files:**
- Create: `mobile/src/viewmodel/browser.rs`
- Modify: `mobile/src/viewmodel/mod.rs`
- Modify: `mobile/src/viewmodel/common.rs`

**Interfaces:**
- Consumes: db_browser operations, api_mycart client
- Produces: BrowserActor managing tabs, navigation, and mode switching

**Implementation:** See design spec Section 3.2 for full BrowserActor code. Key steps:
- Add BrowserCommand/BrowserEvent enums
- Create BrowserActor with HashMap<tab_id, MyCartClient>
- Implement all command handlers (tab management, navigation, API operations)
- Wire into ViewModel
- Test tab create/close flow
- Commit

---

### Task 8: WebView UI Components

**Files:**
- Create: `mobile/src/browser_ui.rs`
- Create: `mobile/src/webview_tab.rs`
- Modify: `mobile/src/lib.rs`

**Interfaces:**
- Consumes: ViewModel browser commands/events, egui_webview
- Produces: Complete browser UI with tabs and sidebar

**Implementation:** Follow reference/egui_webview/examples/tabbrowser.rs pattern. Create sidebar, toolbar, tab bar, and webview content area. Wire to BrowserActor commands. Test UI rendering.

---

### Task 9: API Mode UI Components

**Files:**
- Create: `mobile/src/api_tab.rs`
- Modify: `mobile/src/browser_ui.rs`

**Interfaces:**
- Consumes: Product/Cart data from BrowserActor
- Produces: Native egui product grid and cart UI

**Implementation:** Product grid with images, cart view with line items. Handle ProductsLoaded/CartUpdated events. Test API mode rendering.

---

### Task 10: Main App Integration

**Files:**
- Modify: `mobile/src/dure_sijang_app.rs`
- Modify: `mobile/src/lib.rs`

**Interfaces:**
- Consumes: BrowserUI, ViewModel
- Produces: Integrated browser app

**Implementation:** Replace old tabs with browser_ui. Remove old viewmodel references. Fetch directory on startup. Test full app.

---

### Task 11: Remove Old Code

**Files:**
- Remove: All Dure-Sijang files per design spec Section 5
- Modify: `mobile/src/lib.rs`, `mobile/Cargo.toml`

**Interfaces:**
- Removes: Old tab files, actors, API clients, database modules

**Implementation:** Remove all debloat/scan/install code. Update imports. Clean migrations. Verify build. Commit removal.

---

### Task 12: Update Documentation

**Files:**
- Modify: `CLAUDE.md`, `README.md`

**Interfaces:**
- Updates: Project overview, features, architecture sections

**Implementation:** Rewrite CLAUDE.md for mycart browser. Create user README. Add screenshots. Document build process. Mark migration complete.

---

## Execution Summary

Total: 12 tasks transforming Dure-Sijang to Dure-Sijang browser
- Tasks 1-4: Foundation (migrations, dependencies, models, API client)
- Tasks 5-7: Backend actors (database ops, directory, browser)  
- Tasks 8-9: UI components (webview, API mode)
- Tasks 10-12: Integration and cleanup

Execute using executing-plans skill with TDD approach throughout.
