# Dure-Sijang to Dure-Sijang Migration Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Complete rebrand from Dure-Sijang to dure-sijang by removing all legacy references and updating branding across 17 files plus 1 rename.

**Architecture:** Phased migration with validation at each step. Documentation first, then Android package restructure (breaking change), followed by CI/CD, deployment, translations, and build scripts. Final validation ensures zero legacy references remain.

**Tech Stack:** Markdown, YAML (GitHub Actions), Java/AIDL (Android), Dockerfile, Makefile, Fluent (i18n), Bash scripts

## Global Constraints

- All `Dure-Sijang` → `Dure-Sijang` (proper noun)
- All `uad-shizuku` → `dure-sijang` (lowercase)
- All `uad_shizuku` → `dure_sijang` (underscore)
- All `github.com/dure-one/dure-sijang` → `github.com/dure-one/dure-sijang`
- All `pe.nikescar.dure_sijang` → `app.dure.sijang`
- README.md: mycart browser focus, remove debloat/scan/FOSS legacy features
- Breaking change: Android package ID change requires user reinstall

---

### Task 1: Pre-Migration Backup and Verification

**Files:**
- Read: All project files (via git grep)
- Create: `backup-pre-rebrand` git branch

**Interfaces:**
- Consumes: Current git state
- Produces: Baseline count of Dure-Sijang references (~45-50 expected), backup branch

- [ ] **Step 1: Count current Dure-Sijang references**

```bash
git grep -i "uad.shizuku" | wc -l
```

Expected output: ~45-50 matches

- [ ] **Step 2: Create backup branch**

```bash
git branch backup-pre-rebrand
```

Expected output: (no output, success silent)

- [ ] **Step 3: Verify backup branch exists**

```bash
git branch | grep backup-pre-rebrand
```

Expected output: `  backup-pre-rebrand`

- [ ] **Step 4: List all files with Dure-Sijang references**

```bash
git grep -l -i "uad.shizuku" | sort
```

Expected output: List of 17 files (workflows, docs, translations, build scripts, Android files)

---

### Task 2: Documentation Restructure

**Files:**
- Modify: `README.md` (complete rewrite)
- Modify: `CLAUDE.md` (remove legacy, update overview)
- Modify: `SECURITY.md` (find-replace process name)

**Interfaces:**
- Consumes: Current README structure (debloat/scan/FOSS focus)
- Produces: Mycart browser-focused documentation with `github.com/dure-one/dure-sijang` links

- [ ] **Step 1: Rewrite README.md**

Replace entire content with:

```markdown
<img src="./imgs/logo110.png" alt="drawing" width="120"/>

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

| Arch          | Windows        | MacOS         | Linux        | Android        |
|:--------------|:--------------:|:-------------:|:------------:|:--------------:|
| X86_64(AMD64) | [GUI](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-x86_64.exe) | [GUI](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-x86_64-apple-darwin.tar.gz) | [GUI](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-x86_64-unknown-linux-musl.tar.gz) | [APK](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-all-signed.apk) |
| I686(x86)     | [GUI](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-i686.exe) | - | - | - |
| AARCH64(ARM64)| [GUI](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-aarch64.exe) | [GUI](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-aarch64-apple-darwin.tar.gz) | [GUI](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-aarch64-linux-android.tar.gz) | [APK](https://github.com/dure-one/dure-sijang/releases/latest/download/dure-sijang-all-signed.apk) |

[Latest Release](https://github.com/dure-one/dure-sijang/releases)

## Usage

1. Install dure-sijang
2. Browse mycart stores from dure.one directory
3. Switch between WebView and API modes per tab

## Star History

[![Star History Chart](https://api.star-history.com/image?repos=nikescar/dure-sijang&type=date&legend=top-left)](https://www.star-history.com/)
```

- [ ] **Step 2: Verify README.md no longer mentions Dure-Sijang**

```bash
grep -i "uad" README.md
```

Expected output: (no output - zero matches)

- [ ] **Step 3: Update CLAUDE.md - Remove legacy features section**

Find and remove the entire "### Legacy Features (Pre-August 2026)" section and its subsections (Debloat, Scan, Install, Metadata).

- [ ] **Step 4: Update CLAUDE.md - Remove Dure-Sijang references**

Find and remove these specific lines:
- "- Dure-Sijang debloat/scan/apps actors remain for backward compatibility" (in Features section)
- UAD-NG references in External Dependencies section
- Stalkerware IoC references in External Dependencies section

- [ ] **Step 5: Update CLAUDE.md - Overview section**

Replace Overview section (after line 10) with:

```markdown
Dure-Sijang is a cross-platform "mycart" designated browser. "mycart" is go-fiber backed web store. any indivisual could have their own store and each-store can be connected very well.
Dure-Sijang will help user to navigate through multiple "mycart" websites with downloading directories from dure.one. this app supports 2 mode. 1. webview mode - which will navigate each website through webview.
2. api mode - which will navagate "mycart" with their api only.
```

- [ ] **Step 6: Verify CLAUDE.md no longer mentions Dure-Sijang**

```bash
grep -i "uad" CLAUDE.md
```

Expected output: (no output - zero matches except possibly in git workflow examples)

- [ ] **Step 7: Update SECURITY.md**

Replace all occurrences:

```bash
# Find current content
grep -n "Dure-Sijang" SECURITY.md
```

Then replace:
- "Dure-Sijang" → "dure-sijang" (2 occurrences expected)

Final SECURITY.md should start with:
```markdown
We take security bugs in dure-sijang seriously. We appreciate your efforts to responsibly disclose your findings, and will make every effort to acknowledge your contributions.

## The dure-sijang Security Notification Process
```

- [ ] **Step 8: Verify SECURITY.md changes**

```bash
grep -i "uad" SECURITY.md
```

Expected output: (no output - zero matches)

- [ ] **Step 9: Commit documentation changes**

```bash
git add README.md CLAUDE.md SECURITY.md
git commit -m "$(cat <<'EOF'
docs: rebrand from Dure-Sijang to dure-sijang

- README: rewrite for mycart browser focus, remove debloat/scan legacy
- CLAUDE: remove Dure-Sijang legacy features section
- SECURITY: update process name references
- All GitHub URLs now point to dure-one/dure-sijang

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected output: Commit SHA with "3 files changed"

---

### Task 3: Android Package Restructure ⚠️ BREAKING CHANGE

**Files:**
- Move: `mobile/app/src/main/aidl/pe/nikescar/dure_sijang/IShellCallback.aidl` → `mobile/app/src/main/aidl/app/dure/sijang/`
- Move: `mobile/app/src/main/aidl/pe/nikescar/dure_sijang/IShellService.aidl` → `mobile/app/src/main/aidl/app/dure/sijang/`
- Move: `mobile/app/src/main/java/pe/nikescar/dure_sijang/IIntentSenderAdaptor.java` → `mobile/app/src/main/java/app/dure/sijang/`
- Move: `mobile/app/src/main/java/pe/nikescar/dure_sijang/IntentSenderUtils.java` → `mobile/app/src/main/java/app/dure/sijang/`
- Move: `mobile/app/src/main/java/pe/nikescar/dure_sijang/ShizukuBridge.java` → `mobile/app/src/main/java/app/dure/sijang/`
- Modify: `mobile/app/src/main/AndroidManifest.xml`

**Interfaces:**
- Consumes: Java/AIDL files with `package pe.nikescar.dure_sijang;`
- Produces: Java/AIDL files with `package app.dure.sijang;`, Android manifest with `app.dure.sijang` package ID

- [ ] **Step 1: Create new directory structure**

```bash
mkdir -p mobile/app/src/main/aidl/app/dure/sijang
mkdir -p mobile/app/src/main/java/app/dure/sijang
```

Expected output: (no output, success silent)

- [ ] **Step 2: Move and update IShellCallback.aidl**

```bash
# Copy file
cp mobile/app/src/main/aidl/pe/nikescar/dure_sijang/IShellCallback.aidl \
   mobile/app/src/main/aidl/app/dure/sijang/IShellCallback.aidl

# Update package declaration
sed -i 's/package pe.nikescar.dure_sijang;/package app.dure.sijang;/' \
   mobile/app/src/main/aidl/app/dure/sijang/IShellCallback.aidl
```

Expected output: (no output, file updated)

- [ ] **Step 3: Verify IShellCallback.aidl package**

```bash
head -1 mobile/app/src/main/aidl/app/dure/sijang/IShellCallback.aidl
```

Expected output: `package app.dure.sijang;`

- [ ] **Step 4: Move and update IShellService.aidl**

```bash
cp mobile/app/src/main/aidl/pe/nikescar/dure_sijang/IShellService.aidl \
   mobile/app/src/main/aidl/app/dure/sijang/IShellService.aidl

sed -i 's/package pe.nikescar.dure_sijang;/package app.dure.sijang;/' \
   mobile/app/src/main/aidl/app/dure/sijang/IShellService.aidl
```

- [ ] **Step 5: Verify IShellService.aidl package**

```bash
head -1 mobile/app/src/main/aidl/app/dure/sijang/IShellService.aidl
```

Expected output: `package app.dure.sijang;`

- [ ] **Step 6: Move and update IIntentSenderAdaptor.java**

```bash
cp mobile/app/src/main/java/pe/nikescar/dure_sijang/IIntentSenderAdaptor.java \
   mobile/app/src/main/java/app/dure/sijang/IIntentSenderAdaptor.java

sed -i 's/package pe.nikescar.dure_sijang;/package app.dure.sijang;/' \
   mobile/app/src/main/java/app/dure/sijang/IIntentSenderAdaptor.java
```

- [ ] **Step 7: Verify IIntentSenderAdaptor.java package**

```bash
head -1 mobile/app/src/main/java/app/dure/sijang/IIntentSenderAdaptor.java
```

Expected output: `package app.dure.sijang;`

- [ ] **Step 8: Move and update IntentSenderUtils.java**

```bash
cp mobile/app/src/main/java/pe/nikescar/dure_sijang/IntentSenderUtils.java \
   mobile/app/src/main/java/app/dure/sijang/IntentSenderUtils.java

sed -i 's/package pe.nikescar.dure_sijang;/package app.dure.sijang;/' \
   mobile/app/src/main/java/app/dure/sijang/IntentSenderUtils.java
```

- [ ] **Step 9: Verify IntentSenderUtils.java package**

```bash
head -1 mobile/app/src/main/java/app/dure/sijang/IntentSenderUtils.java
```

Expected output: `package app.dure.sijang;`

- [ ] **Step 10: Move and update ShizukuBridge.java**

```bash
cp mobile/app/src/main/java/pe/nikescar/dure_sijang/ShizukuBridge.java \
   mobile/app/src/main/java/app/dure/sijang/ShizukuBridge.java

sed -i 's/package pe.nikescar.dure_sijang;/package app.dure.sijang;/' \
   mobile/app/src/main/java/app/dure/sijang/ShizukuBridge.java
```

- [ ] **Step 11: Verify ShizukuBridge.java package**

```bash
head -1 mobile/app/src/main/java/app/dure/sijang/ShizukuBridge.java
```

Expected output: `package app.dure.sijang;`

- [ ] **Step 12: Remove old directory structure**

```bash
rm -rf mobile/app/src/main/aidl/pe/
rm -rf mobile/app/src/main/java/pe/
```

Expected output: (no output, directories deleted)

- [ ] **Step 13: Verify old structure is gone**

```bash
ls mobile/app/src/main/java/pe/ 2>&1
```

Expected output: `ls: cannot access 'mobile/app/src/main/java/pe/': No such file or directory`

- [ ] **Step 14: Update AndroidManifest.xml package declaration**

Find and replace in `mobile/app/src/main/AndroidManifest.xml`:

```xml
<!-- Before -->
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="pe.nikescar.dure_sijang">

<!-- After -->
<manifest xmlns:android="http://schemas.android.com/apk/res/android"
    package="app.dure.sijang">
```

Command:
```bash
sed -i 's/package="pe.nikescar.dure_sijang"/package="app.dure.sijang"/' \
   mobile/app/src/main/AndroidManifest.xml
```

- [ ] **Step 15: Verify AndroidManifest.xml package**

```bash
grep 'package=' mobile/app/src/main/AndroidManifest.xml
```

Expected output: `    package="app.dure.sijang">`

- [ ] **Step 16: Check for any remaining pe.nikescar references in mobile/**

```bash
git grep "pe.nikescar" mobile/
```

Expected output: (no output - zero matches)

- [ ] **Step 17: Commit Android package restructure**

```bash
git add mobile/app/src/main/
git commit -m "$(cat <<'EOF'
refactor(android): migrate package from pe.nikescar.dure_sijang to app.dure.sijang

BREAKING CHANGE: Android package ID changed. Users must uninstall old app
(pe.nikescar.dure_sijang) and install new app (app.dure.sijang).

- Moved all Java/AIDL files to app/dure/sijang/ package structure
- Updated package declarations in all files
- Updated AndroidManifest.xml package attribute
- Deleted old pe/nikescar/dure_sijang/ directory structure

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected output: Commit SHA with "5 files moved, 1 file changed" or similar

---

### Task 4: CI/CD Workflow Updates

**Files:**
- Modify: `.github/workflows/release.yml`
- Modify: `.github/workflows/release-googleplay.yml`
- Modify: `.github/workflows/release-msstore.yml`

**Interfaces:**
- Consumes: Workflow files with `uad-shizuku` artifact names, `pe.nikescar.uad_shizuku` package
- Produces: Workflow files with `dure-sijang` artifacts, `app.dure.sijang` package

- [ ] **Step 1: Update release.yml BIN_NAME**

```bash
sed -i 's/BIN_NAME: "uad-shizuku"/BIN_NAME: "dure-sijang"/' .github/workflows/release.yml
```

- [ ] **Step 2: Update release.yml artifact names (all occurrences)**

```bash
sed -i 's/uad-shizuku/dure-sijang/g' .github/workflows/release.yml
```

- [ ] **Step 3: Update release.yml cargo package name**

```bash
# Should already be updated by previous sed, verify
grep "package dure-sijang" .github/workflows/release.yml | head -1
```

Expected output: Line containing `--package dure-sijang`

- [ ] **Step 4: Verify release.yml has no uad-shizuku references**

```bash
grep -i "uad.shizuku" .github/workflows/release.yml
```

Expected output: (no output - zero matches)

- [ ] **Step 5: Update release-googleplay.yml artifact name**

```bash
sed -i 's/uad-shizuku/dure-sijang/g' .github/workflows/release-googleplay.yml
```

- [ ] **Step 6: Update release-googleplay.yml package name**

```bash
sed -i 's/pe.nikescar.uad_shizuku/app.dure.sijang/' .github/workflows/release-googleplay.yml
```

- [ ] **Step 7: Verify release-googleplay.yml changes**

```bash
grep "packageName:" .github/workflows/release-googleplay.yml
```

Expected output: Line containing `packageName: app.dure.sijang`

- [ ] **Step 8: Verify release-googleplay.yml has no uad-shizuku**

```bash
grep -i "uad.shizuku" .github/workflows/release-googleplay.yml
```

Expected output: (no output - zero matches)

- [ ] **Step 9: Update release-msstore.yml artifact names**

```bash
sed -i 's/uad-shizuku/dure-sijang/g' .github/workflows/release-msstore.yml
```

- [ ] **Step 10: Verify release-msstore.yml has no uad-shizuku**

```bash
grep -i "uad.shizuku" .github/workflows/release-msstore.yml
```

Expected output: (no output - zero matches)

- [ ] **Step 11: Commit CI/CD workflow updates**

```bash
git add .github/workflows/
git commit -m "$(cat <<'EOF'
ci: update workflows for dure-sijang rebrand

- release.yml: BIN_NAME and all artifact names uad-shizuku → dure-sijang
- release-googleplay.yml: package pe.nikescar.uad_shizuku → app.dure.sijang
- release-msstore.yml: all artifact names uad-shizuku → dure-sijang
- Cargo package builds now use --package dure-sijang

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected output: Commit SHA with "3 files changed"

---

### Task 5: Deployment Configuration Updates

**Files:**
- Modify: `deploy/flatpak/Dockerfile`
- Modify: `deploy/flatpak/Makefile`
- Rename: `deploy/flatpak/pe.nikescar.dure_sijang.desktop` → `deploy/flatpak/app.dure.sijang.desktop`
- Modify: `deploy/flatpak/app.dure.sijang.desktop` (after rename)

**Interfaces:**
- Consumes: Flatpak files with `Dure-Sijang` references, `pe.nikescar.dure_sijang` app ID
- Produces: Flatpak files with `dure-sijang` references, `app.dure.sijang` app ID

- [ ] **Step 1: Update Dockerfile maintainer and description**

```bash
sed -i 's/Dure-Sijang/dure-sijang/g' deploy/flatpak/Dockerfile
```

- [ ] **Step 2: Update Dockerfile workdir paths**

```bash
sed -i 's/uad-shizuku/dure-sijang/g' deploy/flatpak/Dockerfile
```

- [ ] **Step 3: Verify Dockerfile has no Dure-Sijang**

```bash
grep -i "uad" deploy/flatpak/Dockerfile
```

Expected output: (no output - zero matches)

- [ ] **Step 4: Update Makefile APP_ID**

```bash
sed -i 's/APP_ID = pe.nikescar.dure_sijang/APP_ID = app.dure.sijang/' deploy/flatpak/Makefile
```

- [ ] **Step 5: Verify Makefile APP_ID**

```bash
grep "APP_ID" deploy/flatpak/Makefile
```

Expected output: `APP_ID = app.dure.sijang`

- [ ] **Step 6: Rename desktop file**

```bash
git mv deploy/flatpak/pe.nikescar.dure_sijang.desktop deploy/flatpak/app.dure.sijang.desktop
```

Expected output: (no output, file renamed)

- [ ] **Step 7: Update desktop file contents**

```bash
sed -i 's/pe.nikescar.dure_sijang/app.dure.sijang/g' deploy/flatpak/app.dure.sijang.desktop
```

- [ ] **Step 8: Verify desktop file has new app ID**

```bash
grep "app.dure.sijang" deploy/flatpak/app.dure.sijang.desktop | head -1
```

Expected output: Line containing `app.dure.sijang`

- [ ] **Step 9: Verify deploy/flatpak/ has no pe.nikescar references**

```bash
git grep "pe.nikescar" deploy/flatpak/
```

Expected output: (no output - zero matches)

- [ ] **Step 10: Commit deployment configuration**

```bash
git add deploy/flatpak/
git commit -m "$(cat <<'EOF'
build(flatpak): rebrand from pe.nikescar.dure_sijang to app.dure.sijang

- Dockerfile: update maintainer, description, and workdir paths
- Makefile: APP_ID pe.nikescar.dure_sijang → app.dure.sijang
- Renamed desktop file and updated app ID references inside

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected output: Commit SHA with "3 files changed, 1 file renamed"

---

### Task 6: Translation Updates

**Files:**
- Modify: `mobile/assets/languages/fluent/en-US.ftl`
- Modify: `mobile/assets/languages/fluent/ko-KR.ftl`

**Interfaces:**
- Consumes: Fluent files with `Dure-Sijang` app title, `uad-shizuku.pages.dev` docs URL
- Produces: Fluent files with `Dure-Sijang` app title, `dure.one` docs URL

- [ ] **Step 1: Update English app-title**

```bash
sed -i 's/app-title = Dure-Sijang/app-title = Dure-Sijang/' mobile/assets/languages/fluent/en-US.ftl
```

- [ ] **Step 2: Update English install dialog step 5**

```bash
sed -i 's/Return to Dure-Sijang/Return to dure-sijang/' mobile/assets/languages/fluent/en-US.ftl
```

- [ ] **Step 3: Update English docs URL**

```bash
sed -i 's|https://uad-shizuku.pages.dev/docs/installation|https://dure.one/docs/installation|' mobile/assets/languages/fluent/en-US.ftl
```

- [ ] **Step 4: Verify English translation updates**

```bash
grep -E "(app-title|install-dlg-step5|install-dlg-guide-url)" mobile/assets/languages/fluent/en-US.ftl
```

Expected output:
```
app-title = Dure-Sijang
install-dlg-step5 = 5. Return to dure-sijang and tap Retry Detection
install-dlg-guide-url = https://dure.one/docs/installation
```

- [ ] **Step 5: Update Korean app-title**

```bash
sed -i 's/app-title = Dure-Sijang/app-title = Dure-Sijang/' mobile/assets/languages/fluent/ko-KR.ftl
```

- [ ] **Step 6: Update Korean install dialog step 5**

```bash
sed -i 's/Dure-Sijang로/dure-sijang로/' mobile/assets/languages/fluent/ko-KR.ftl
```

- [ ] **Step 7: Update Korean docs URL**

```bash
sed -i 's|https://uad-shizuku.pages.dev/docs/kr/docs/installation|https://dure.one/docs/kr/installation|' mobile/assets/languages/fluent/ko-KR.ftl
```

- [ ] **Step 8: Verify Korean translation updates**

```bash
grep -E "(app-title|install-dlg-step5|install-dlg-guide-url)" mobile/assets/languages/fluent/ko-KR.ftl
```

Expected output:
```
app-title = Dure-Sijang
install-dlg-step5 = 5. dure-sijang로 돌아가서 재감지 탭
install-dlg-guide-url = https://dure.one/docs/kr/installation
```

- [ ] **Step 9: Verify no UAD references in fluent files**

```bash
grep -i "uad" mobile/assets/languages/fluent/*.ftl
```

Expected output: (no output - zero matches)

- [ ] **Step 10: Commit translation updates**

```bash
git add mobile/assets/languages/fluent/
git commit -m "$(cat <<'EOF'
i18n: rebrand app title and docs URLs

- App title: Dure-Sijang → Dure-Sijang (en-US, ko-KR)
- Install dialog: updated app name references
- Docs URLs: uad-shizuku.pages.dev → dure.one

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected output: Commit SHA with "2 files changed"

---

### Task 7: Build Script Updates

**Files:**
- Modify: `mobile/build.sh`
- Modify: `mobile/build.fd.sh`
- Modify: `scripts/release.sh`

**Interfaces:**
- Consumes: Build scripts with `uad-shizuku`, `uad_shizuku`, `pe.nikescar.dure_sijang` references
- Produces: Build scripts with `dure-sijang`, `dure_sijang`, `app.dure.sijang` references

- [ ] **Step 1: Check mobile/build.sh for UAD references**

```bash
grep -n -i "uad" mobile/build.sh
```

Expected output: Line numbers with matches (if any)

- [ ] **Step 2: Update mobile/build.sh**

```bash
sed -i 's/uad-shizuku/dure-sijang/g' mobile/build.sh
sed -i 's/uad_shizuku/dure_sijang/g' mobile/build.sh
sed -i 's/pe.nikescar.dure_sijang/app.dure.sijang/g' mobile/build.sh
```

- [ ] **Step 3: Verify mobile/build.sh has no UAD references**

```bash
grep -i "uad" mobile/build.sh
```

Expected output: (no output - zero matches)

- [ ] **Step 4: Check mobile/build.fd.sh for UAD references**

```bash
grep -n -i "uad" mobile/build.fd.sh
```

Expected output: Line numbers with matches (if any)

- [ ] **Step 5: Update mobile/build.fd.sh**

```bash
sed -i 's/uad-shizuku/dure-sijang/g' mobile/build.fd.sh
sed -i 's/uad_shizuku/dure_sijang/g' mobile/build.fd.sh
sed -i 's/pe.nikescar.dure_sijang/app.dure.sijang/g' mobile/build.fd.sh
```

- [ ] **Step 6: Verify mobile/build.fd.sh has no UAD references**

```bash
grep -i "uad" mobile/build.fd.sh
```

Expected output: (no output - zero matches)

- [ ] **Step 7: Check scripts/release.sh for UAD references**

```bash
grep -n -i "uad" scripts/release.sh
```

Expected output: Line numbers with matches (if any)

- [ ] **Step 8: Update scripts/release.sh**

```bash
sed -i 's/uad-shizuku/dure-sijang/g' scripts/release.sh
sed -i 's/uad_shizuku/dure_sijang/g' scripts/release.sh
sed -i 's/pe.nikescar.dure_sijang/app.dure.sijang/g' scripts/release.sh
```

- [ ] **Step 9: Verify scripts/release.sh has no UAD references**

```bash
grep -i "uad" scripts/release.sh
```

Expected output: (no output - zero matches)

- [ ] **Step 10: Commit build script updates**

```bash
git add mobile/build.sh mobile/build.fd.sh scripts/release.sh
git commit -m "$(cat <<'EOF'
build: update build scripts for dure-sijang rebrand

- mobile/build.sh: all uad-shizuku → dure-sijang references
- mobile/build.fd.sh: all uad-shizuku → dure-sijang references
- scripts/release.sh: all uad-shizuku → dure-sijang references
- Package ID: pe.nikescar.dure_sijang → app.dure.sijang

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>
EOF
)"
```

Expected output: Commit SHA with "3 files changed"

---

### Task 8: Final Validation and Verification

**Files:**
- Read: All project files (via git grep)
- Verify: Build success, file structure, manual checklist

**Interfaces:**
- Consumes: All rebrand changes from Tasks 2-7
- Produces: Validation report, zero Dure-Sijang references confirmed

- [ ] **Step 1: Search for any remaining uad-shizuku references**

```bash
git grep -i "uad.shizuku"
```

Expected output: (no output - zero matches in tracked files)

- [ ] **Step 2: Search for any remaining nikescar references**

```bash
git grep -i "nikescar" | grep -v ".git"
```

Expected output: (no output or only Star History chart URL - acceptable)

- [ ] **Step 3: Count app.dure.sijang references**

```bash
git grep "app.dure.sijang" | wc -l
```

Expected output: 5+ matches (Java files, AndroidManifest, workflows, flatpak)

- [ ] **Step 4: Count dure-sijang references**

```bash
git grep "dure-sijang" | wc -l
```

Expected output: 50+ matches (workflows, docs, scripts, translations)

- [ ] **Step 5: Count github.com/dure-one references**

```bash
git grep "github.com/dure-one" | wc -l
```

Expected output: 10+ matches (README download links)

- [ ] **Step 6: Verify Android package structure**

```bash
ls mobile/app/src/main/java/app/dure/sijang/
```

Expected output:
```
IIntentSenderAdaptor.java
IntentSenderUtils.java
ShizukuBridge.java
```

- [ ] **Step 7: Verify AIDL structure**

```bash
ls mobile/app/src/main/aidl/app/dure/sijang/
```

Expected output:
```
IShellCallback.aidl
IShellService.aidl
```

- [ ] **Step 8: Verify old structure is gone**

```bash
ls mobile/app/src/main/java/pe/ 2>&1
```

Expected output: `No such file or directory`

- [ ] **Step 9: Verify Flatpak desktop file**

```bash
ls deploy/flatpak/app.dure.sijang.desktop
```

Expected output: `deploy/flatpak/app.dure.sijang.desktop`

- [ ] **Step 10: Verify old Flatpak desktop file is gone**

```bash
ls deploy/flatpak/pe.nikescar.dure_sijang.desktop 2>&1
```

Expected output: `No such file or directory`

- [ ] **Step 11: Test desktop build**

```bash
cargo build --package dure-sijang
```

Expected output: Build completes successfully with "Finished" message

- [ ] **Step 12: Test clippy**

```bash
cargo clippy --package dure-sijang
```

Expected output: No errors, possible warnings acceptable

- [ ] **Step 13: Manual verification checklist**

Confirm each item:

- [ ] README.md describes mycart browser (no debloat/scan mentions)
- [ ] All download links point to `github.com/dure-one/dure-sijang`
- [ ] CLAUDE.md has no Dure-Sijang legacy references
- [ ] Fluent translations show "Dure-Sijang" app title
- [ ] Android manifest declares `package="app.dure.sijang"`
- [ ] CI/CD workflows use `dure-sijang` artifact names
- [ ] Flatpak uses `app.dure.sijang` app ID
- [ ] Documentation URLs point to `dure.one` domain

- [ ] **Step 14: Generate final validation report**

```bash
echo "=== Migration Validation Report ===" > /tmp/validation-report.txt
echo "" >> /tmp/validation-report.txt
echo "Dure-Sijang references: $(git grep -i 'uad.shizuku' | wc -l)" >> /tmp/validation-report.txt
echo "nikescar references: $(git grep -i 'nikescar' | grep -v '.git' | wc -l)" >> /tmp/validation-report.txt
echo "app.dure.sijang references: $(git grep 'app.dure.sijang' | wc -l)" >> /tmp/validation-report.txt
echo "dure-sijang references: $(git grep 'dure-sijang' | wc -l)" >> /tmp/validation-report.txt
echo "dure-one GitHub refs: $(git grep 'github.com/dure-one' | wc -l)" >> /tmp/validation-report.txt
echo "" >> /tmp/validation-report.txt
echo "Android package structure:" >> /tmp/validation-report.txt
echo "  Java files: $(ls mobile/app/src/main/java/app/dure/sijang/ | wc -l)" >> /tmp/validation-report.txt
echo "  AIDL files: $(ls mobile/app/src/main/aidl/app/dure/sijang/ | wc -l)" >> /tmp/validation-report.txt
echo "" >> /tmp/validation-report.txt
echo "Build status: $(cargo build --package dure-sijang 2>&1 | tail -1)" >> /tmp/validation-report.txt

cat /tmp/validation-report.txt
```

Expected output:
```
=== Migration Validation Report ===

Dure-Sijang references: 0
nikescar references: 0 (or 1 if Star History URL)
app.dure.sijang references: 5+
dure-sijang references: 50+
dure-one GitHub refs: 10+

Android package structure:
  Java files: 3
  AIDL files: 2

Build status: Finished ...
```

- [ ] **Step 15: Show commit summary**

```bash
git log --oneline backup-pre-rebrand..HEAD
```

Expected output: List of 6 commits (docs, android, ci, flatpak, i18n, build scripts)
