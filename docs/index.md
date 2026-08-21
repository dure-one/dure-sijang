---
title: About
---

# Introduction

Dure-Sijang is a cross-platform mycart browser for browsing and shopping across multiple mycart stores. It provides dual-mode browsing (WebView and API) with integrated shopping cart management.

## Key Features

### Dual-Mode Browsing

Browse mycart stores using two different modes:
- **WebView Mode**: Full website rendering with embedded browser (desktop and Android)
- **API Mode**: Native UI calling mycart REST API directly for optimized performance

### Store Directory

Synchronize and browse multiple mycart stores from the [dure.one](https://dure.one) directory. Easily discover and navigate between different stores.

### Shopping Cart Management

- In-app cart management with real-time updates
- Product browsing with image previews
- Add to cart functionality in API mode
- Cart persistence across sessions

### Tab Management

- Multi-tab browsing with persistent tabs
- Bookmarks for quick access to favorite stores
- Navigation history tracking
- Session restoration

### Cross-Platform Support

- **Desktop**: Linux, macOS, and Windows with GTK/WebView2/WebKit
- **Android**: Native APK with Android WebView integration

## Technology

Dure-Sijang is built with:
- **Rust**: Core application logic
- **egui**: Cross-platform UI framework
- **wry**: Cross-platform webview library
- **Diesel + SQLite**: Local data persistence
- **smol**: Async runtime for background tasks