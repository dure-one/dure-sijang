# Dure-Sijang to Dure-Sijang Migration Cleanup Design

**Date:** 2026-08-21  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved  
**Approach:** Complete Atomic Rebrand

## Overview

This spec defines the complete migration cleanup from Dure-Sijang to dure-sijang. The migration implementation is complete (mycart browser functionality), and this cleanup removes all legacy references and establishes consistent branding across documentation, CI/CD, code structure, and deployment artifacts.

## Goals

1. **Complete Rebrand**: Remove all Dure-Sijang references, establish dure-sijang identity
2. **Documentation Focus**: Rewrite docs to emphasize mycart browser (remove legacy debloat/scan features)
3. **Package Restructure**: Change Android package from `pe.nikescar.dure_sijang` to `app.dure.sijang`
4. **GitHub Migration**: Update repository references from `nikescar` to `dure-one` organization
5. **Artifact Consistency**: All release artifacts use `dure-sijang-*` naming

## Design Decisions

### Question 1: Documentation Direction
**Decision:** Full mycart browser replacement (remove legacy features)  
**Rationale:** Dure-sijang is now a mycart browser, not a debloater. Clean slate approach.

### Question 2: GitHub Repository
**Decision:** `github.com/dure-one/dure-sijang`  
**Rationale:** New organization (`dure-one`) aligns with product branding.

### Question 3: Release Artifacts
**Decision:** `dure-sijang-*` pattern for all artifacts  
**Rationale:** Consistent naming, no backwards compatibility needed.

### Question 4: Android Package ID
**Decision:** `app.dure.sijang` (from `pe.nikescar.dure_sijang`)  
**Rationale:** Clean branding, matches `dure.one` domain structure.

### Question 5: Historical References
**Decision:** Remove all (clean slate)  
**Rationale:** No migration notes, no legacy documentation. Fresh start.

## Architecture

### File Inventory

**Total files to modify:** 17 + 1 rename

**Documentation (3 files):**
- `README.md` - Complete rewrite for mycart browser
- `CLAUDE.md` - Remove Dure-Sijang legacy section, update architecture
- `SECURITY.md` - Update process name references

**CI/CD Workflows (3 files):**
- `.github/workflows/release.yml` - Binary name, artifact names, package build
- `.github/workflows/release-googleplay.yml` - Package ID, artifact names
- `.github/workflows/release-msstore.yml` - Binary name, download URLs

**Android Package Structure (5 files + 1 manifest):**
- `mobile/app/src/main/aidl/pe/nikescar/dure_sijang/IShellCallback.aidl` → `app/dure/sijang/`
- `mobile/app/src/main/aidl/pe/nikescar/dure_sijang/IShellService.aidl` → `app/dure/sijang/`
- `mobile/app/src/main/java/pe/nikescar/dure_sijang/IIntentSenderAdaptor.java` → `app/dure/sijang/`
- `mobile/app/src/main/java/pe/nikescar/dure_sijang/IntentSenderUtils.java` → `app/dure/sijang/`
- `mobile/app/src/main/java/pe/nikescar/dure_sijang/ShizukuBridge.java` → `app/dure/sijang/`
- `mobile/app/src/main/AndroidManifest.xml` - Package declaration

**Deployment (3 files + 1 rename):**
- `deploy/flatpak/Dockerfile` - App references, build paths
- `deploy/flatpak/Makefile` - App ID variable
- `deploy/flatpak/pe.nikescar.dure_sijang.desktop` → `app.dure.sijang.desktop` (rename + update)

**Translations (2 files):**
- `mobile/assets/languages/fluent/en-US.ftl` - App title, dialog text, docs URL
- `mobile/assets/languages/fluent/ko-KR.ftl` - App title, dialog text, docs URL

**Build Scripts (3 files):**
- `mobile/build.sh` - Package references
- `mobile/build.fd.sh` - Package references
- `scripts/release.sh` - Binary/artifact name references

### Key Replacements

**Text patterns:**
- `Dure-Sijang` → `Dure-Sijang` (proper noun, capitalized)
- `uad-shizuku` → `dure-sijang` (lowercase, filenames/URLs)
- `uad_shizuku` → `dure_sijang` (underscore variant)

**GitHub URLs:**
- `github.com/dure-one/Dure-Sijang` → `github.com/dure-one/dure-sijang`
- `github.com/dure-one/dure-sijang` → `github.com/dure-one/dure-sijang`

**Android Package:**
- `pe.nikescar.dure_sijang` → `app.dure.sijang`

**Artifact Names:**
- `uad-shizuku.exe` → `dure-sijang.exe`
- `uad-shizuku-all-signed.apk` → `dure-sijang-all-signed.apk`
- `uad-shizuku-all.aab` → `dure-sijang-all.aab`
- `uad-shizuku-<target>` → `dure-sijang-<target>` (all platforms)

## Detailed Changes

### 1. Documentation Restructure

#### README.md - Complete Rewrite

**New Structure:**
```markdown
# Dure-Sijang

Cross-platform mycart browser with dual-mode navigation.

## Features

### Dual-Mode Browsing
- **WebView Mode**: Full website rendering via wry (desktop/Android)
- **API Mode**: Native UI calling mycart REST API directly

### Store Management
- Synchronize mycart stores from dure.one directory
- Multi-tab browsing with persistent sessions
- Bookmarks and browsing history

### Shopping Features
- In-app cart management (API mode)
- Product browsing with native performance
- Offline support with SQLite caching

## Download
[Download table with github.com/dure-one/dure-sijang links]
[Latest Release](https://github.com/dure-one/dure-sijang/releases)

## Usage
1. Install dure-sijang
2. Browse mycart stores from dure.one directory
3. Switch between WebView and API modes per tab
```

**Remove:**
- All debloat/scan/FOSS app installation content
- UAD-NG references
- Stalkerware indicators mentions
- VirusTotal/HybridAnalysis API instructions
- Legacy features section

#### CLAUDE.md Updates

**Remove:**
- "Dure-Sijang debloat/scan/apps actors remain for backward compatibility"
- "Legacy Features (Pre-August 2026)" entire section
- UAD-NG list references in External Dependencies
- Stalkerware IoC references

**Keep:**
- Mycart Browser Architecture section (already correct)
- Browser tables in database documentation
- All August 2026 migration content (Browser/Directory actors)

**Update:**
- Overview section to pure mycart browser description

#### SECURITY.md

Simple find-replace:
```markdown
# Before
We take security bugs in Dure-Sijang seriously.
## The Dure-Sijang Security Notification Process

# After
We take security bugs in dure-sijang seriously.
## The dure-sijang Security Notification Process
```

### 2. Android Package Restructure ⚠️ BREAKING CHANGE

**Impact:** New app identity on Android. Users must uninstall old app and reinstall new one.

#### Directory Structure Change

**Before:**
```
mobile/app/src/main/
├── aidl/pe/nikescar/dure_sijang/
│   ├── IShellCallback.aidl
│   └── IShellService.aidl
└── java/pe/nikescar/dure_sijang/
    ├── IIntentSenderAdaptor.java
    ├── IntentSenderUtils.java
    └── ShizukuBridge.java
```

**After:**
```
mobile/app/src/main/
├── aidl/app/dure/sijang/
│   ├── IShellCallback.aidl
│   └── IShellService.aidl
└── java/app/dure/sijang/
    ├── IIntentSenderAdaptor.java
    ├── IntentSenderUtils.java
    └── ShizukuBridge.java
```

#### Package Declaration Updates

Update inside each Java/AIDL file:
```java
// Before
package pe.nikescar.dure_sijang;

// After
package app.dure.sijang;
```

#### Android Manifest

`mobile/app/src/main/AndroidManifest.xml`:
```xml
<!-- Before -->
<manifest package="pe.nikescar.dure_sijang">

<!-- After -->
<manifest package="app.dure.sijang">
```

#### Rust JNI Bindings

Verify `mobile/src/android_*.rs` files for JNI package path references. Update if needed.

### 3. CI/CD Workflow Updates

#### .github/workflows/release.yml

**Environment variable:**
```yaml
# Before
env:
  BIN_NAME: "uad-shizuku"

# After
env:
  BIN_NAME: "dure-sijang"
```

**Artifact name updates (13+ occurrences):**
- `uad-shizuku.exe` → `dure-sijang.exe`
- `uad-shizuku-all-signed.apk` → `dure-sijang-all-signed.apk`
- `uad-shizuku-all.aab` → `dure-sijang-all.aab`
- `uad-shizuku-aarch64-apple-darwin` → `dure-sijang-aarch64-apple-darwin`
- `uad-shizuku-aarch64-linux-android` → `dure-sijang-aarch64-linux-android`
- `uad-shizuku-aarch64-unknown-linux-musl` → `dure-sijang-aarch64-unknown-linux-musl`
- `uad-shizuku-armv7-linux-androideabi` → `dure-sijang-armv7-linux-androideabi`
- `uad-shizuku-armv7-unknown-linux-musleabihf` → `dure-sijang-armv7-unknown-linux-musleabihf`
- `uad-shizuku-x86_64-apple-darwin` → `dure-sijang-x86_64-apple-darwin`
- `uad-shizuku-x86_64-unknown-linux-musl` → `dure-sijang-x86_64-unknown-linux-musl`
- `uad-shizuku-x86_64-pc-windows-msvc` → `dure-sijang-x86_64-pc-windows-msvc`
- `uad-shizuku-i686-pc-windows-msvc` → `dure-sijang-i686-pc-windows-msvc`
- `uad-shizuku-aarch64-pc-windows-msvc` → `dure-sijang-aarch64-pc-windows-msvc`
- `uad-shizuku-x86_64.exe` → `dure-sijang-x86_64.exe`
- `uad-shizuku-i686.exe` → `dure-sijang-i686.exe`
- `uad-shizuku-aarch64.exe` → `dure-sijang-aarch64.exe`

**Cargo build:**
```bash
# Before
--package uad-shizuku
cargo build --release --package uad-shizuku --target ${{ matrix.platform.target }}

# After
--package dure-sijang
cargo build --release --package dure-sijang --target ${{ matrix.platform.target }}
```

#### .github/workflows/release-googleplay.yml

```yaml
# Before
name: uad-shizuku-all.aab
packageName: pe.nikescar.uad_shizuku
releaseFiles: ./artifacts/uad-shizuku-all.aab

# After
name: dure-sijang-all.aab
packageName: app.dure.sijang
releaseFiles: ./artifacts/dure-sijang-all.aab
```

#### .github/workflows/release-msstore.yml

```yaml
# Before
name: uad-shizuku-.*windows.*
find /tmp/artifacts -name "uad-shizuku.exe" -exec cp {} "$VERSION/uad-shizuku.exe" \;
echo "https://${OWNER_NAME}.github.io/${REPO_NAME#*/}/${VERSION}/uad-shizuku.exe"
echo "- https://${OWNER_NAME}.github.io/${REPO_NAME#*/}/${v}/uad-shizuku.exe"

# After
name: dure-sijang-.*windows.*
find /tmp/artifacts -name "dure-sijang.exe" -exec cp {} "$VERSION/dure-sijang.exe" \;
echo "https://${OWNER_NAME}.github.io/${REPO_NAME#*/}/${VERSION}/dure-sijang.exe"
echo "- https://${OWNER_NAME}.github.io/${REPO_NAME#*/}/${v}/dure-sijang.exe"
```

### 4. Deployment Configuration

#### Flatpak Files

**File rename:**
```bash
# Before
deploy/flatpak/pe.nikescar.dure_sijang.desktop

# After
deploy/flatpak/app.dure.sijang.desktop
```

**deploy/flatpak/Dockerfile:**
```dockerfile
# Before
LABEL maintainer="Dure-Sijang Development Team"
LABEL description="Container for building and testing Dure-Sijang Flatpak"
COPY --chown=builder:builder . /home/builder/uad-shizuku-flatpak/
WORKDIR /home/builder/uad-shizuku-flatpak

# After
LABEL maintainer="dure-sijang Development Team"
LABEL description="Container for building and testing dure-sijang Flatpak"
COPY --chown=builder:builder . /home/builder/dure-sijang-flatpak/
WORKDIR /home/builder/dure-sijang-flatpak
```

**deploy/flatpak/Makefile:**
```makefile
# Before
APP_ID = pe.nikescar.dure_sijang

# After
APP_ID = app.dure.sijang
```

**deploy/flatpak/app.dure.sijang.desktop** (content update after rename):
Update all `pe.nikescar.dure_sijang` references to `app.dure.sijang` inside the file.

### 5. Translation Updates

#### mobile/assets/languages/fluent/en-US.ftl

```fluent
# Before
app-title = Dure-Sijang
install-dlg-step5 = 5. Return to Dure-Sijang and tap Retry Detection
install-dlg-guide-url = https://uad-shizuku.pages.dev/docs/installation

# After
app-title = Dure-Sijang
install-dlg-step5 = 5. Return to dure-sijang and tap Retry Detection
install-dlg-guide-url = https://dure.one/docs/installation
```

#### mobile/assets/languages/fluent/ko-KR.ftl

```fluent
# Before
app-title = Dure-Sijang
install-dlg-step5 = 5. Dure-Sijang로 돌아가서 재감지 탭
install-dlg-guide-url = https://uad-shizuku.pages.dev/docs/kr/docs/installation

# After
app-title = Dure-Sijang
install-dlg-step5 = 5. dure-sijang로 돌아가서 재감지 탭
install-dlg-guide-url = https://dure.one/docs/kr/installation
```

**Note:** Documentation URLs assume migration from `uad-shizuku.pages.dev` to `dure.one`.

### 6. Build Scripts

**mobile/build.sh, mobile/build.fd.sh, scripts/release.sh:**

Pattern replacements:
- `uad-shizuku` → `dure-sijang`
- `uad_shizuku` → `dure_sijang`
- `pe.nikescar.dure_sijang` → `app.dure.sijang`

*(Exact line-by-line changes to be verified during implementation)*

## Validation Strategy

### Pre-Migration Verification

```bash
# Confirm current state
git grep -i "uad.shizuku" | wc -l  # Should show ~45-50 matches

# Backup current branch
git branch backup-pre-rebrand
```

### Post-Migration Checks

#### 1. Code Search Verification

```bash
# Should return ZERO results (except in git history)
git grep -i "uad.shizuku"
git grep -i "nikescar" | grep -v ".git"

# Verify new names present
git grep "app.dure.sijang" | wc -l    # Should show Java package refs
git grep "dure-sijang" | wc -l        # Should show many matches
git grep "github.com/dure-one" | wc -l  # Should show doc links
```

#### 2. Build Validation

```bash
# Desktop build
cargo build --package dure-sijang
cargo clippy --package dure-sijang

# Android build (via gradle)
cd mobile && ./gradlew assembleRelease
# Verify APK package ID: app.dure.sijang
```

#### 3. File Structure Check

```bash
# Verify Android package moved
ls mobile/app/src/main/java/app/dure/sijang/
# Should show: IIntentSenderAdaptor.java, IntentSenderUtils.java, ShizukuBridge.java

ls mobile/app/src/main/aidl/app/dure/sijang/
# Should show: IShellCallback.aidl, IShellService.aidl

# Old structure should be gone
ls mobile/app/src/main/java/pe/nikescar/dure_sijang/  # Should fail
```

#### 4. Flatpak Desktop File

```bash
# Should exist
ls deploy/flatpak/app.dure.sijang.desktop

# Old file should be gone
ls deploy/flatpak/pe.nikescar.dure_sijang.desktop  # Should fail
```

#### 5. Manual Verification Checklist

- [ ] README.md describes mycart browser (no debloat/scan mentions)
- [ ] All download links point to `github.com/dure-one/dure-sijang`
- [ ] CLAUDE.md has no Dure-Sijang legacy references
- [ ] Fluent translations show "Dure-Sijang" app title
- [ ] Android manifest declares `package="app.dure.sijang"`
- [ ] CI/CD workflows use `dure-sijang` artifact names
- [ ] Flatpak uses `app.dure.sijang` app ID
- [ ] Documentation URLs point to `dure.one` domain

## Breaking Changes

### Android Package ID Change

**Impact:** Users with `pe.nikescar.dure_sijang` installed will see `app.dure.sijang` as a completely new app.

**User Migration Required:**
1. Uninstall old app (`pe.nikescar.dure_sijang`)
2. Install new app (`app.dure.sijang`)
3. Reconfigure settings (no automatic data migration)

**Mitigation:**
- Clear communication in release notes
- Documentation update on migration steps
- Consider retaining old package ID in Play Store if continuity is critical (requires decision reversal)

**Decision:** Accepted - clean slate rebrand is more important than upgrade continuity.

## Open Questions

None - all design decisions validated with user.

## Success Criteria

1. ✅ Zero `uad-shizuku` or `Dure-Sijang` references in tracked files (excluding git history)
2. ✅ Zero `pe.nikescar.dure_sijang` references (Android package fully migrated to `app.dure.sijang`)
3. ✅ All GitHub URLs point to `github.com/dure-one/dure-sijang`
4. ✅ All CI/CD artifacts use `dure-sijang-*` naming
5. ✅ README.md describes mycart browser (legacy features removed)
6. ✅ Desktop and Android builds succeed with new naming
7. ✅ Flatpak configuration uses `app.dure.sijang` app ID
8. ✅ Translations show "Dure-Sijang" branding

## Next Steps

1. Invoke `writing-plans` skill to create implementation plan
2. Execute migration in single atomic commit
3. Run validation suite (build + search + manual checks)
4. Commit with message: "refactor: complete rebrand from Dure-Sijang to dure-sijang"
5. Create PR for review

## References

- Current repository: `/home/wj/work/dure-sijang`
- CLAUDE.md: Mycart Browser Architecture (August 2026)
- Recent commits: Browser navigation and UI improvements (feat/browser-fixes branch)
