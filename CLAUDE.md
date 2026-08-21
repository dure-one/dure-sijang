# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 1. Overview

Dure-Sijang is a cross-platform "mycart" designated browser. "mycart" is go-fiber backed web store. any indivisual could have their own store and each-store can be connected very well.
Dure-Sijang will help user to navigate through multiple "mycart" websites with downloading directories from dure.one. this app supports 2 mode. 1. webview mode - which will navigate each website through webview.
2. api mode - which will navagate "mycart" with their api only.

## 2. Features

### Mycart Browser (August 2026)

- **Dual-Mode Browsing**: 
  - **WebView Mode**: Embedded browser using wry for full website rendering
  - **API Mode**: Native egui UI calling mycart REST API directly for optimized performance
- **Store Directory**: Synchronize and browse multiple mycart stores from dure.one directory
- **Tab Management**: Multi-tab browsing with persistent tabs, bookmarks, and history
- **Shopping Cart**: In-app cart management for API mode with product browsing
- **Offline Support**: Database-backed caching for products, carts, and browsing history

## 3. Repository Structure

```
dure-sijang/
├── mobile/                          # Main application workspace
│   ├── src/
│   │   ├── viewmodel/              # MVVM ViewModel layer (Jun-Aug 2026)
│   │   │   ├── mod.rs              # ViewModel struct, channels, state
│   │   │   ├── browser.rs          # BrowserActor (tabs, navigation, mycart API) [NEW Aug 2026]
│   │   │   ├── directory.rs        # DirectoryActor (dure.one sync) [NEW Aug 2026]
│   │   │   ├── debloat.rs          # DebloatActor (packages, UAD lists) [LEGACY]
│   │   │   ├── scan.rs             # ScanActor (VirusTotal, HybridAnalysis) [LEGACY]
│   │   │   ├── apps.rs             # AppsActor (FOSS app lists) [LEGACY]
│   │   │   ├── metadata.rs         # MetadataActor (GooglePlay, F-Droid, etc.) [LEGACY]
│   │   │   └── common.rs           # Shared types (commands, events, state)
│   │   ├── dure_sijang_app.rs      # Main app struct (egui)
│   │   ├── main.rs                 # Desktop entry point
│   │   ├── main_android.rs         # Android entry point
│   │   ├── lib.rs                  # Library exports, Config, Settings
│   │   ├── adb.rs                  # ADB client implementation
│   │   ├── android_shizuku.rs      # Shizuku JNI integration (Android)
│   │   ├── android_*.rs            # Android platform integration modules
│   │   ├── browser_ui.rs           # Browser UI component (tab bar, navigation) [NEW Aug 2026]
│   │   ├── webview_tab.rs          # WebView tab (wry integration) [NEW Aug 2026]
│   │   ├── api_tab.rs              # API mode tab (product grid, cart) [NEW Aug 2026]
│   │   ├── tab_debloat_control.rs  # Debloat UI tab [LEGACY]
│   │   ├── tab_scan_control.rs     # Scan UI tab [LEGACY]
│   │   ├── tab_apps_control.rs     # Apps UI tab [LEGACY]
│   │   ├── tab_usage_control.rs    # Usage tracking tab (stub) [LEGACY]
│   │   ├── dlg_*.rs                # Dialog windows
│   │   ├── api_mycart.rs           # Mycart REST API client [NEW Aug 2026]
│   │   ├── api_virustotal.rs       # VirusTotal API client [LEGACY]
│   │   ├── api_hybridanalysis.rs   # HybridAnalysis API client
│   │   ├── api_googleplay.rs       # Google Play scraper
│   │   ├── api_fdroid.rs           # F-Droid API client
│   │   ├── api_apkmirror.rs        # APKMirror scraper
│   │   ├── api_*.rs                # Other API clients
│   │   ├── db.rs                   # Database connection & migrations
│   │   ├── db_browser.rs           # Browser DB ops (tabs, bookmarks, history) [NEW Aug 2026]
│   │   ├── db_directory.rs         # Store directory DB operations [NEW Aug 2026]
│   │   ├── db_virustotal.rs        # VirusTotal cache DB operations [LEGACY]
│   │   ├── db_hybridanalysis.rs    # HybridAnalysis cache DB operations [LEGACY]
│   │   ├── db_package_cache.rs     # Package metadata DB operations [LEGACY]
│   │   ├── db_googleplay.rs        # Google Play metadata DB operations [LEGACY]
│   │   ├── db_fdroid.rs            # F-Droid metadata DB operations [LEGACY]
│   │   ├── db_apkmirror.rs         # APKMirror metadata DB operations [LEGACY]
│   │   ├── calc.rs                 # Core calculation/processing logic
│   │   ├── calc_*.rs               # Domain-specific business logic
│   │   ├── app_operations_queue.rs # App install/uninstall queue
│   │   ├── shared_store.rs         # LEGACY: Global state (being phased out)
│   │   ├── models.rs               # Data models (exports browser & mycart modules)
│   │   ├── models/
│   │   │   ├── browser.rs          # Browser types (TabState, BrowsingMode, etc.) [NEW Aug 2026]
│   │   │   └── mycart.rs           # Mycart API types (Product, Cart, etc.) [NEW Aug 2026]
│   │   ├── schema.rs               # Diesel schema (auto-generated)
│   │   ├── material_symbol_icons.rs # Material Design icon definitions
│   │   └── *_stt.rs                # State struct modules (paired with main modules)
│   ├── migrations/                 # Diesel SQL migrations (timestamped)
│   ├── tests/                      # Integration tests
│   │   ├── integration/            # Integration test modules
│   │   ├── test_fdroid.rs
│   │   ├── test_hybridanalysis.rs
│   │   └── test_virustotal_db.rs.disabled
│   ├── assets/                     # Embedded resources
│   │   └── languages/fluent/       # i18n translations (en-US, ko-KR)
│   ├── resources/                  # Downloaded at build time
│   │   ├── uad_lists.json          # UAD-NG debloat lists
│   │   └── stalkerware_ioc.yaml    # Stalkerware indicators
│   ├── app/                        # Android app configuration
│   │   └── src/main/               # Android manifest, resources
│   ├── build.rs                    # Build script (downloads resources)
│   ├── Cargo.toml
│   └── diesel.toml                 # Diesel configuration
├── reference/                      # Reference implementations
│   └── bingtray/                   # Similar MVVM project for reference
├── docs/                           # Documentation
│   ├── mvvm-actor-migration-complete.md  # Architecture migration notes
│   ├── next-session-handoff.md     # Session context for Claude
│   └── superpowers/                # Claude Code plans and specs
├── deploy/                         # Build and deployment scripts
├── scripts/                        # Utility scripts
├── fastlane/                       # App store deployment automation
└── Cargo.toml                      # Workspace root
```

## 4. Architecture

### MVVM with Actor-Based Concurrency (June-August 2026)

Dure-Sijang uses MVVM pattern with actors for background processing:

```
┌─────────────────────────────────────────────────────────────┐
│                    DureSijangApp (egui UI)                   │
│  ┌────────────────────────────────────────────────────┐     │
│  │              ViewModel (Command/Event)              │     │
│  │                                                     │     │
│  │  ┌──────────────────────────────────────────┐     │     │
│  │  │       ViewModelState (Read-Only)          │     │     │
│  │  └──────────────────────────────────────────┘     │     │
│  │                                                     │     │
│  │  Command Channels (UI → Actors):                   │     │
│  │                                                     │     │
│  │  Event Channel (Actors → UI):                      │     │
│  └────────────────────────────────────────────────────┘     │
│                                                               │
│  Actors (Background Thread - smol runtime):                  │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐      │
│  │BrowserActor  │  │DirectoryActor│  │DebloatActor  │      │
│  │ • CreateTab  │  │ • SyncDir    │  │ • LoadPkgs   │      │
│  │ • Navigate   │  │ • AddStore   │  │ • LoadUAD    │      │
│  │ • FetchAPI   │  │ • RemoveStore│  │ • Uninstall  │      │
│  │ • Bookmarks  │  │ • Refresh    │  │ [LEGACY]     │      │
│  └──────────────┘  └──────────────┘  └──────────────┘      │
│  ┌──────────────┐  ┌──────────────┐                         │
│  │  ScanActor   │  │MetadataActor │                         │
│  │ • VT Scan    │  │ • GooglePlay │                         │
│  │ • HA Scan    │  │ • FDroid     │                         │
│  │ [LEGACY]     │  │ [LEGACY]     │                         │
│  └──────────────┘  └──────────────┘                         │
└─────────────────────────────────────────────────────────────┘
         │                    │                    │
         ▼                    ▼                    ▼
  ┌──────────────────────────────────────────────────┐
  │          Database Layer (Diesel + SQLite)         │
  │  NEW (Aug 2026):                                  │
  │  • store_directory (dure.one mycart stores)       │
  │  • tabs (browser tabs with mode)                  │
  │  • bookmarks (saved pages)                        │
  │  • browsing_history (navigation history)          │
  │  • cached_products, cached_carts (API mode cache) │
  │  LEGACY:                                          │
  │  • virustotal_results, hybridanalysis_results     │
  │  • package_info_cache                             │
  │  • google_play_apps, fdroid_apps, apkmirror_apps  │
  └──────────────────────────────────────────────────┘
```

**Key Patterns**:
- **Commands**: UI sends commands to actors (e.g., `ScanCommand::ScanPackage`)
- **Events**: Actors emit events consumed by UI (e.g., `ScanEvent::ScanComplete`)
- **State**: `ViewModelState` is the single source of truth, read-only from UI
- **Polling**: UI calls `viewmodel.poll_events()` in `update()` to process events

### Database Layer (Diesel + SQLite)

- **Location**: `mobile/src/db*.rs`
- **Migrations**: Embedded in binary, run automatically on first connection
- **Schema**: Auto-generated `schema.rs` (do not edit manually)
- **Storage Paths**:
  - Desktop: `~/.config/dure_sijang/dbs/` (Linux/macOS), `%APPDATA%\dure_sijang\dbs\` (Windows)
  - Android: `/data/data/app.dure.sijang/dbs/`

**Tables** (August 2026):
- **Browser Tables (NEW)**:
  - `store_directory`: Mycart stores from dure.one
  - `tabs`: Browser tabs with webview/API mode
  - `bookmarks`: Saved mycart pages
  - `browsing_history`: Navigation history
  - `cached_products`: Product metadata cache
  - `cached_carts`: Shopping cart persistence
  - `user_credentials`: Store login tokens
  - `user_preferences`: User settings per store
- **Legacy Tables**:
  - `virustotal_results`: Cached malware scan results
  - `hybridanalysis_results`: Cached sandbox analysis results
  - `package_info_cache`: Android package metadata
  - `google_play_apps`, `fdroid_apps`, `apkmirror_apps`: App metadata

### External Dependencies

**Mycart Browser (NEW - August 2026)**:
- **dure.one Directory**: REST API for mycart store listings
- **Mycart API**: go-fiber REST API for product/cart operations
- **wry 0.56**: Cross-platform webview (GTK on Linux/OpenBSD, WebView2 on Windows, WebKit on macOS)

### Mycart Browser Architecture (August 2026)

**Dual-Mode Browsing**:
- **WebView Mode**: Full website rendering using wry
  - Desktop: GTK-based webview (Linux/OpenBSD), WebView2 (Windows), WebKit (macOS)
  - Android: Android WebView via wry JNI bindings
  - Navigation: Back/forward/reload controls in UI
  - History: All navigation persists to browsing_history table
- **API Mode**: Native egui UI calling mycart REST API
  - Product Grid: 3-column layout with images, names, prices
  - Product Detail: Full product view with description and add-to-cart
  - Shopping Cart: In-memory cart with CartItem tracking
  - Performance: No web rendering overhead, direct API calls

**Store Directory Sync**:
- Fetches store list from `https://dure.one/api/directory`
- Stores in `store_directory` table with name, base_url, category, logo
- DirectoryActor handles sync in background thread
- UI displays store list for quick navigation

**Tab Management**:
- Each tab has unique ID, store_url, current_url, mode (webview/api), title
- Tabs persist to database for session restoration
- UI shows tab bar with active tab highlighting
- Close tab removes from UI but keeps history in database

**Bookmarks & History**:
- Bookmark any page with store_url, page_url, title, description
- History auto-captured on navigation with timestamp
- Both stored in SQLite for offline access

## 5. Build and Test

### Desktop Build

```bash
# Debug build (fast iteration)
cargo build

# Release build (optimized, stripped)
cargo build --release

# Run on desktop
cargo run
```

### Android Build

Android builds are managed via gradle in `deploy/` directory. Requires Android Studio and NDK.

### Testing

```bash
# Run all tests
cargo test

# Run specific test module
cargo test --test viewmodel_tests

# Run with logs visible
RUST_LOG=debug cargo test -- --nocapture
```

### Code Quality

```bash
# Format code
cargo fmt

# Lint
cargo clippy

# Check licenses and dependencies
cargo deny check
```

## 6. Common Development Tasks

### Adding a New Migration

```bash
cd mobile
diesel migration generate add_new_column
# Edit migrations/<timestamp>_add_new_column/up.sql
# Edit migrations/<timestamp>_add_new_column/down.sql
cargo build  # Schema auto-updates
```

### Adding a New Actor Command/Event

1. Add command variant to `viewmodel/<domain>.rs`
2. Add event variant to `viewmodel/common.rs` (`ViewModelEvent` enum)
3. Handle command in actor's `async fn run()` loop
4. Emit event via `event_tx.send()`
5. Handle event in `ViewModel::poll_events()`

### Adding New Translations

Edit `mobile/assets/languages/fluent/{en-US,ko-KR}.ftl` using Fluent syntax.

## 7. Platform-Specific Notes

### Android

- **Entry Point**: `mobile/src/main_android.rs`
- **ADB Alternative**: Uses Shizuku JNI for native Android execution
- **Permissions**: Requires Shizuku permission grant from user

### Desktop

- **Entry Point**: `mobile/src/main.rs`
- **ADB Requirement**: Must have `adb` in PATH or bundled
- **Window Manager**: Uses native OS windowing (GTK on Linux, Win32 on Windows, Cocoa on macOS)

### WASM (Experimental)

- **Target**: `wasm32-unknown-unknown`
- **Limitations**: No ADB support, Diesel uses WASM SQLite VFS
- **Status**: Partially implemented, not production-ready

## 8. Recent Changes

### MVVM Actor Architecture Migration (June 2026)

**Completed**:
- ✅ Migrated from global `SharedStore` singleton to MVVM pattern
- ✅ Implemented 4 actors: Debloat, Scan, Apps, Metadata
- ✅ Centralized state in `ViewModelState`
- ✅ Added comprehensive integration tests
- ✅ Scanner states now in ViewModel (VirusTotal, HybridAnalysis)
- ✅ Metadata cache now in ViewModel (GooglePlay, F-Droid, APKMirror)
- ✅ Stalkerware indicators loaded into ViewModel

**Legacy Code**:
- `shared_store.rs` still exists for texture caches (egui constraint)
- New code should use ViewModel, not SharedStore

**Documentation**: See `docs/mvvm-actor-migration-complete.md`

### Mycart Browser Migration (August 2026)

**Completed**:
- ✅ Added 2 new actors: BrowserActor, DirectoryActor
- ✅ Implemented dual-mode browsing (WebView via wry + API mode)
- ✅ Created 8 new database tables for browser functionality
- ✅ Built mycart REST API client (api_mycart.rs)
- ✅ Implemented browser UI components (browser_ui, webview_tab, api_tab)
- ✅ Database operations for tabs, bookmarks, history, directory
- ✅ Store directory sync from dure.one
- ✅ Tab persistence and restoration
- ✅ Product grid and cart UI for API mode

**Architecture**:
- `viewmodel/browser.rs`: Tab management, navigation, mycart API calls
- `viewmodel/directory.rs`: dure.one directory sync
- `browser_ui.rs`: Main browser UI with tab bar
- `webview_tab.rs`: wry webview integration (desktop/Android)
- `api_tab.rs`: Native product grid and cart UI
- `models/browser.rs`: Browser types (TabState, BrowsingMode, Bookmark, HistoryEntry)
- `models/mycart.rs`: Mycart API types (Product, Cart, CartItem, DirectoryResponse)

**Database Schema**:
- `store_directory`: Mycart stores from dure.one
- `tabs`: Browser tabs with mode (webview/api)
- `bookmarks`: Saved pages
- `browsing_history`: Navigation history
- `cached_products`, `cached_carts`: API mode cache
- `user_credentials`, `user_preferences`: Per-store settings

**Dependencies Added**:
- `wry = "0.56"`: Cross-platform webview
- `http = "1"`: HTTP types for wry integration
- `gtk = "0.18"` (OpenBSD): GTK backend for webview

## 9. API Keys and Configuration

User settings stored in:
- Desktop: `~/.config/dure_sijang/settings.txt`
- Android: `/data/data/app.dure.sijang/files/settings.txt`

## 10. Code Style Guidelines

- Use `tracing::` macros for logging (`info!`, `warn!`, `error!`)
- Prefix Android JNI modules with `android_*`
- Prefix API clients with `api_*`
- Prefix business logic with `calc_*`
- Prefix database modules with `db_*`
- Prefix dialogs with `dlg_*`
- Prefix tabs with `tab_*`
- Suffix state structs with `_stt` module (e.g., `scan_stt.rs` for `ScanState`)
- All new async operations should use actors in `viewmodel/`
- Do not add new dependencies to `SharedStore` - use ViewModel instead

## 11. Testing Guidelines

- Integration tests for actors in `mobile/tests/`
- Unit tests in same file as implementation (inline `#[cfg(test)]`)
- Mock ADB responses using in-memory fixtures
- Database tests use temporary SQLite files
- UI tests are manual (egui integration testing is limited)

## 12. Known Limitations

- Android lifecycle handling needs improvement (app freezes after sleep)
- APKMirror renderer app ID search is not accurate
- Web version blocked by Diesel/ureq WASM compatibility
- Some metadata fetchers may fail due to website structure changes
