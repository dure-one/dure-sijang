# Fix Cargo Schema Mismatch Design

**Date:** 2026-08-18  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

Fix the cargo build error caused by a schema mismatch between database migrations and Rust structs in `db_browser.rs`. The migrations use a normalized design with foreign keys (`store_id`, `tab_id`), while the Rust code expects a denormalized design with URL strings (`store_url`, `page_url`).

## Problem Statement

Running `cargo run` fails with compilation errors:

```
error[E0425]: cannot find type `store_url` in module `bookmarks`
error[E0425]: cannot find type `page_url` in module `bookmarks`
error[E0425]: cannot find type `description` in module `bookmarks`
```

**Root Cause:**

Database migrations (created 2026-08-18) define:
- `bookmarks` table: `id`, `title`, `url`, `store_id` (FK), `created_at`
- `browsing_history` table: `id`, `tab_id` (FK), `url`, `title`, `visited_at`

Rust structs in `db_browser.rs` expect:
- `DbBookmark`: `id`, `store_url`, `page_url`, `title`, `description`, `created_at`
- `DbHistoryEntry`: `id`, `store_url`, `page_url`, `title`, `visited_at`

## Goals

1. Make the code compile by aligning Rust structs with database schema
2. Maintain database normalization (use foreign keys, not duplicate data)
3. Follow existing patterns in the codebase (`tabs` table uses `store_id`)
4. Update all affected database operations and tests

## Non-Goals

- Changing the database schema (migrations stay as-is)
- Adding new features or fields
- Migrating existing data (this is new code with no production data)

## Design Decisions

### Decision 1: Update Rust Code vs Update Schema

**Options:**
- A) Update Rust code to match normalized schema ✅ **Selected**
- B) Update schema to match denormalized Rust code
- C) Hybrid approach (add description column only)

**Rationale for A:**
- Schema follows proper database normalization
- Simpler fix (one file change vs multiple migrations)
- Consistent with existing `tabs` table design pattern
- No data migration needed
- Better for future features (can JOIN with store_directory for store metadata)

### Decision 2: History Query Strategy

**Options:**
- A) Rename `get_history_for_store` to `get_history_for_tab` ✅ **Selected**
- B) Keep function name but add JOIN with tabs table

**Rationale for A:**
- More direct - history is inherently tab-specific
- Simpler query (no JOIN needed)
- Clearer API semantics
- Matches the schema's foreign key relationship

## Architecture

### File Changes

**Files to modify:** 1 file
- `mobile/src/db_browser.rs` - Update structs, operations, and tests

**No migration changes needed.**

### Data Model

**Bookmarks Table:**
```
bookmarks
├── id (PK)
├── title
├── url (full page URL)
├── store_id (FK -> store_directory.id, nullable)
└── created_at
```

**Browsing History Table:**
```
browsing_history
├── id (PK)
├── tab_id (FK -> tabs.id, NOT NULL)
├── url (full page URL)
├── title (nullable)
└── visited_at
```

**Relationship:**
- Bookmarks → Store Directory (many-to-one, nullable)
- Browsing History → Tabs (many-to-one, cascading delete)
- Tabs → Store Directory (many-to-one, nullable)

### Struct Changes

#### DbBookmark

**Before:**
```rust
pub struct DbBookmark {
    pub id: Option<i32>,
    pub store_url: String,      // ❌ doesn't exist
    pub page_url: String,        // ❌ doesn't exist
    pub title: String,
    pub description: Option<String>, // ❌ doesn't exist
    pub created_at: chrono::NaiveDateTime,
}
```

**After:**
```rust
pub struct DbBookmark {
    pub id: Option<i32>,
    pub title: String,
    pub url: String,             // ✅ matches schema
    pub store_id: Option<i32>,   // ✅ FK to store_directory
    pub created_at: chrono::NaiveDateTime,
}
```

#### NewBookmark

**Before:**
```rust
pub struct NewBookmark {
    pub store_url: String,
    pub page_url: String,
    pub title: String,
    pub description: Option<String>,
}
```

**After:**
```rust
pub struct NewBookmark {
    pub title: String,
    pub url: String,
    pub store_id: Option<i32>,
}
```

#### DbHistoryEntry

**Before:**
```rust
pub struct DbHistoryEntry {
    pub id: Option<i32>,
    pub store_url: String,       // ❌ doesn't exist
    pub page_url: String,        // ❌ doesn't exist
    pub title: Option<String>,
    pub visited_at: chrono::NaiveDateTime,
}
```

**After:**
```rust
pub struct DbHistoryEntry {
    pub id: Option<i32>,
    pub tab_id: i32,             // ✅ FK to tabs table
    pub url: String,             // ✅ matches schema
    pub title: Option<String>,
    pub visited_at: chrono::NaiveDateTime,
}
```

#### NewHistoryEntry

**Before:**
```rust
pub struct NewHistoryEntry {
    pub store_url: String,
    pub page_url: String,
    pub title: Option<String>,
}
```

**After:**
```rust
pub struct NewHistoryEntry {
    pub tab_id: i32,
    pub url: String,
    pub title: Option<String>,
}
```

### Function Changes

#### get_bookmarks_for_store

**Before:**
```rust
pub fn get_bookmarks_for_store(conn: &mut SqliteConnection, store: &str) -> Result<Vec<DbBookmark>> {
    let results = bookmarks
        .filter(store_url.eq(store))  // ❌ column doesn't exist
        .order(created_at.desc())
        .load::<DbBookmark>(conn)?;
    Ok(results)
}
```

**After:**
```rust
pub fn get_bookmarks_for_store(conn: &mut SqliteConnection, store_id_param: i32) -> Result<Vec<DbBookmark>> {
    use crate::schema::bookmarks::dsl::*;
    
    let results = bookmarks
        .filter(store_id.eq(Some(store_id_param)))  // ✅ filter by FK
        .order(created_at.desc())
        .load::<DbBookmark>(conn)?;
    Ok(results)
}
```

**Changes:**
- Parameter type: `&str` → `i32` (store ID instead of URL)
- Filter column: `store_url` → `store_id`
- Wrap in `Some()` since `store_id` is nullable

#### get_history_for_store → get_history_for_tab

**Before:**
```rust
pub fn get_history_for_store(conn: &mut SqliteConnection, store: &str) -> Result<Vec<DbHistoryEntry>> {
    let results = browsing_history
        .filter(store_url.eq(store))  // ❌ column doesn't exist
        .order(visited_at.desc())
        .load::<DbHistoryEntry>(conn)?;
    Ok(results)
}
```

**After:**
```rust
pub fn get_history_for_tab(conn: &mut SqliteConnection, tab_id_param: i32) -> Result<Vec<DbHistoryEntry>> {
    use crate::schema::browsing_history::dsl::*;
    
    let results = browsing_history
        .filter(tab_id.eq(tab_id_param))  // ✅ filter by FK
        .order(visited_at.desc())
        .load::<DbHistoryEntry>(conn)?;
    Ok(results)
}
```

**Changes:**
- Function name: `get_history_for_store` → `get_history_for_tab`
- Parameter: `store: &str` → `tab_id_param: i32`
- Filter column: `store_url` → `tab_id`
- Semantics: History is tab-specific, not store-specific

### Test Updates

All three test functions need updates:

#### test_bookmark_crud (lines 244-273)

**Changes:**
```rust
// Before
let new_bookmark = NewBookmark {
    store_url: "https://test.mycart.example".to_string(),
    page_url: "https://test.mycart.example/products/123".to_string(),
    title: "Cool Product".to_string(),
    description: Some("A very cool product".to_string()),
};

let store_bookmarks = get_bookmarks_for_store(conn, "https://test.mycart.example")?;

// After
let new_bookmark = NewBookmark {
    title: "Cool Product".to_string(),
    url: "https://test.mycart.example/products/123".to_string(),
    store_id: None,  // Or Some(1) if referencing a store
};

// Note: If testing with store_id, need to insert into store_directory first
let store_bookmarks = get_bookmarks_for_store(conn, 1)?;  // Use store_id
```

#### test_history_crud (lines 275-303)

**Changes:**
```rust
// Before
let new_entry = NewHistoryEntry {
    store_url: "https://test.mycart.example".to_string(),
    page_url: "https://test.mycart.example/about".to_string(),
    title: Some("About Us".to_string()),
};

let store_history = get_history_for_store(conn, "https://test.mycart.example")?;

// After
// First create a tab (history requires tab_id)
let tab_id = insert_tab(conn, NewTab {
    title: "Test Tab".to_string(),
    url: "https://test.mycart.example".to_string(),
    mode: "webview".to_string(),
    store_id: None,
})?;

let new_entry = NewHistoryEntry {
    tab_id,
    url: "https://test.mycart.example/about".to_string(),
    title: Some("About Us".to_string()),
};

let tab_history = get_history_for_tab(conn, tab_id)?;
```

## Error Handling

### Edge Cases

1. **Bookmarks with NULL store_id**
   - Schema allows `store_id` to be NULL
   - Valid case: User bookmarks a page without knowing the store
   - Function `get_bookmarks_for_store` will not return these (only bookmarks with matching store_id)

2. **History entries cascade delete**
   - When a tab is deleted, all its history entries are deleted (CASCADE)
   - Schema enforces `tab_id NOT NULL` - history always belongs to a tab
   - No orphaned history entries possible

3. **Foreign key validation**
   - SQLite enforces foreign key constraints (if enabled)
   - Inserting bookmark/history with non-existent store_id/tab_id will fail
   - Error propagates as `anyhow::Error` (application-level code uses anyhow per ECC Rust rules)

### Error Propagation

All functions already use `anyhow::Result<T>`:
- Correct for application-level code (per ECC Rust coding-style rules)
- Diesel errors automatically convert to anyhow
- No changes needed to error handling strategy

## Testing Strategy

### Unit Tests

Update existing tests in `db_browser.rs`:
1. `test_tab_crud` - No changes needed (tabs already correct)
2. `test_bookmark_crud` - Update to use new fields
3. `test_history_crud` - Update to use new fields, create tab first

### Additional Test Cases

Add tests for edge cases:
1. Test bookmark with `store_id = None`
2. Test cascade delete of history when tab is deleted
3. Test `get_bookmarks_for_store` returns only matching bookmarks

### Test Execution

```bash
cd mobile
cargo test db_browser::tests
```

All tests use `test_transaction` which auto-rolls back (good isolation).

## Implementation Plan

### Phase 1: Update Structs
1. Update `DbBookmark` struct (lines 28-37)
2. Update `NewBookmark` struct (lines 39-46)
3. Update `DbHistoryEntry` struct (lines 48-56)
4. Update `NewHistoryEntry` struct (lines 58-64)

### Phase 2: Update Functions
1. Update `get_bookmarks_for_store` (lines 140-149)
2. Rename `get_history_for_store` to `get_history_for_tab` (lines 186-195)

### Phase 3: Update Tests
1. Update `test_bookmark_crud` (lines 244-273)
2. Update `test_history_crud` (lines 275-303)

### Phase 4: Verify
1. Run `cargo check` - should compile
2. Run `cargo test db_browser::tests` - should pass
3. Run `cargo run` - should start without errors

## Risks

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|------------|
| Breaking code that uses these structs elsewhere | Low | High | Search codebase for usages before changing |
| Tests fail after updates | Medium | Low | Fix tests incrementally, use test_transaction |
| Foreign key constraint violations | Low | Medium | SQLite may not have FK enforcement enabled by default |

## Success Criteria

- ✅ `cargo check` passes with no errors
- ✅ `cargo test db_browser::tests` passes
- ✅ `cargo run` starts successfully
- ✅ All struct fields match database schema exactly
- ✅ No compilation errors related to schema mismatch

## Future Enhancements

**Not in scope for this fix, but possible future work:**

1. Add `description` field to bookmarks via migration
2. Add helper function to get store URL from `store_id` (JOIN with store_directory)
3. Add function to get all history for a store (JOIN through tabs → browsing_history)
4. Add indexes on foreign key columns for query performance

## References

- Database schema: `mobile/src/schema.rs`
- Migrations: `mobile/migrations/2026-08-18-*`
- ECC Rust rules: `/home/wj/.claude/rules/ecc/rust/`
- CLAUDE.md: `CLAUDE.md` (MVVM architecture section)
