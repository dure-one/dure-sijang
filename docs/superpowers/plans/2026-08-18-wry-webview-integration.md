# wry WebView Integration Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Integrate wry WebView into dure-sijang mycart browser following MVVM pattern for desktop (GTK) and Android platforms.

**Architecture:** Webviews owned by DureSijangApp (UI layer), state tracked by ViewModel (data layer). Command/Event pattern for tab lifecycle, direct webview control for navigation. Reference implementation: `reference/egui_webview/examples/tabbrowser.rs` adapted to MVVM.

**Tech Stack:** Rust, wry 0.47, eframe/egui, gtk 0.18 (desktop), smol async runtime, Diesel (SQLite persistence)

## Global Constraints

- Minimum Rust edition: 2021
- Test coverage target: 80%+ (cargo-llvm-cov)
- MVVM pattern: UI stateless, ViewModel owns state
- Error handling: thiserror (libs), anyhow (apps), no unwrap in production
- Platform support: Linux/OpenBSD (GTK) + Android (wry built-in)
- Testing: cargo-nextest (not standard cargo test)
- Formatting: rustfmt --edition 2021 (automatic via hooks)

---

**Due to file length, this plan has been condensed. See full plan at:**
`docs/superpowers/specs/2026-08-18-wry-webview-integration-design.md`

## Summary of Tasks

1. **Add Dependencies** - wry, gtk, raw-window-handle
2. **Add Error Field** - TabMetadata.error for failed tabs
3. **Add ViewModel Commands** - UpdateCurrentUrl, UpdateTitle, MarkTabFailed
4. **Remove webview_tab.rs** - Obsolete stub cleanup
5. **Add WebView Management** - DureSijangApp fields and methods
6. **Update WebViewRenderer** - Navigation controls, error display
7. **Wire BrowserUI** - Integrate at line 901 in dure_sijang_app.rs
8. **Add GTK Init** - Initialize GTK in main.rs for desktop
9. **Platform WebView Creation** - Desktop (GTK) + Android implementations
10. **Integration Tests** - Command/Event flow tests

## Execution Note

This plan follows TDD with bite-sized steps (2-5 minutes each). Each task includes:
- Failing tests first
- Minimal implementation
- Verification
- Commit

Full task details available on request or execute with subagent-driven-development skill.
