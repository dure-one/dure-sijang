# GitHub Workflows Update Design

**Date:** 2026-08-21  
**Author:** Claude Sonnet 4.5  
**Status:** Design Approved

## Overview

Update all GitHub Actions workflows by copying the latest versions from `reference/uad-shizuku/.github/` and adapting them for the dure-sijang project. This brings in several critical improvements including FreeBSD support, robust Windows code signing validation, a Python-based VirusTotal scanner with 409 error handling, and various workflow enhancements.

## Goals

1. **Complete replacement** of all `.github/workflows/` files with reference versions
2. **Add Python VirusTotal scanner** to handle already-scanned files gracefully
3. **Enable FreeBSD builds** with pkg-config synthesis fixes
4. **Improve Windows signing** with pre-signing certificate validation
5. **Replace all name references** from "uad-shizuku" to "dure-sijang"
6. **Use GITHUB_TOKEN** instead of RELEASE_TOKEN for releases

## Design Decisions

### Question 1: Update Scope
**Decision:** Complete replacement (Option A)  
**Rationale:** User confirmed all improvements are desired, Android configuration is ready, and reference workflows are battle-tested.

### Question 2: Binary Name
**Decision:** Replace all "uad-shizuku" with "dure-sijang"  
**Rationale:** Project package name is already `dure-sijang` in Cargo.toml.

### Question 3: Android Configuration
**Decision:** Keep existing paths and secrets  
**Rationale:** User confirmed `./mobile/app/` paths are correct and all secrets (STORE_PASSWORD, KEY_PASSWORD, KEY_ALIAS, KEYSTORE_BASE64) are configured.

### Question 4: Release Token
**Decision:** Use GITHUB_TOKEN instead of RELEASE_TOKEN  
**Rationale:** Simpler (automatically provided), consistent with reference, sufficient permissions for release creation.

## Architecture

### File Structure

```
.github/
├── scripts/                         # NEW DIRECTORY
│   └── virustotal_scan.py          # NEW - Python VT scanner with 409 handling
└── workflows/
    ├── release.yml                  # REPLACE - Main build & release
    ├── release-googleplay.yml       # REPLACE - Google Play deployment
    ├── release-msstore.yml          # REPLACE - Microsoft Store deployment
    ├── release-snap.yml             # REPLACE - Snap package deployment
    └── vite.docs.yml               # REPLACE - Documentation site build
```

### Name Transformations

All workflow files will have these replacements applied:

| Find | Replace | Context |
|------|---------|---------|
| `uad-shizuku` | `dure-sijang` | Binary names, file paths, artifact names |
| `uad_shizuku` | `dure_sijang` | Rust package names (underscore form) |
| `BIN_NAME: "uad-shizuku"` | `BIN_NAME: "dure-sijang"` | Environment variables |
| `--package uad-shizuku` | `--package dure-sijang` | Cargo build commands |
| `token: ${{ secrets.RELEASE_TOKEN }}` | `token: ${{ secrets.GITHUB_TOKEN }}` | Release creation (release.yml only) |

### Key Improvements from Reference

#### 1. FreeBSD Support (release.yml)

**Current state:** FreeBSD build commented out (lines 63-68)  
**Reference state:** Enabled with pkg-config synthesis

**Changes:**
- Uncomment FreeBSD x86_64 platform entry
- Add pkg-config synthesis for bzip2/zlib (lines 226-248)
  - Fixes missing pkg-config metadata for FreeBSD base libraries
  - Prevents pango/cairo build failures from transitive dependencies
- Use FreeBSD 14.2 VM with vmactions/freebsd-vm@v1

**Benefit:** Enables native FreeBSD binary builds without cross-compilation issues.

#### 2. Enhanced Windows Code Signing (release.yml)

**Current state:** Basic certificate decode and sign (lines 233-272)  
**Reference state:** Pre-signing validation with detailed error messages (lines 255-321)

**Changes:**
- Validate certificate before signing attempt:
  - Check private key presence (`HasPrivateKey` property)
  - Check expiration date (`NotAfter < Get-Date`)
  - Display certificate metadata (Subject, Thumbprint, EKU)
- Fail fast with clear error messages if certificate invalid
- Keep `continue-on-error: true` (step skips if cert missing)

**Benefit:** Prevents silent signing failures, easier troubleshooting for certificate issues.

#### 3. Windows Binary Renaming (release.yml)

**Current state:** Rename during artifact preparation (lines 447-456)  
**Reference state:** Dedicated rename step before upload (lines 323-345)

**Changes:**
- Add "Windows - Rename Binary by Architecture" step
- Rename before artifact upload:
  - `dure-sijang.exe` → `dure-sijang-x86_64.exe` (for x86_64 builds)
  - `dure-sijang.exe` → `dure-sijang-i686.exe` (for i686 builds)
  - `dure-sijang.exe` → `dure-sijang-aarch64.exe` (for aarch64 builds)
- Artifact preparation step expects renamed files

**Benefit:** Cleaner separation of concerns, easier to track renamed binaries in artifacts.

#### 4. Python VirusTotal Scanner (release.yml + new script)

**Current state:** GitHub Action `crazy-max/ghaction-virustotal@v4` (line 319)  
**Reference state:** Python script with custom 409 handling (lines 391-419)

**Changes:**
- Add `.github/scripts/virustotal_scan.py` (198 lines)
- Replace VirusTotal GitHub Action step with:
  - Setup Python 3.11
  - Install requests library
  - Run virustotal_scan.py with file list
- Script features:
  - Computes SHA256 hash before upload
  - Handles 409 errors (file already scanned) by fetching existing report
  - Handles 429 errors (rate limit) with 60s retry
  - Configurable rate limit via `VT_REQUEST_RATE` env var (default: 4/min)
  - Sets `GITHUB_OUTPUT` variable with `file=url` pairs

**Benefit:** Graceful handling of already-scanned files (no failures), better rate limiting control, more robust error handling.

#### 5. Explicit Permissions (release.yml)

**Current state:** No permissions block in release job  
**Reference state:** Explicit `permissions: contents: write` (lines 378-379)

**Changes:**
- Add permissions block to release job:
```yaml
permissions:
  contents: write
```

**Benefit:** Explicit declaration prevents permission issues, follows GitHub Actions best practices.

## Implementation Details

### GitHub Secrets Required

| Secret | Purpose | Status |
|--------|---------|--------|
| `GITHUB_TOKEN` | Release creation, code checkout | Auto-provided by GitHub |
| `VT_API_KEY` | VirusTotal scanning | Must be set (or scan skips with continue-on-error) |
| `STORE_PASSWORD` | Android keystore password | User confirmed configured |
| `KEY_PASSWORD` | Android key password | User confirmed configured |
| `KEY_ALIAS` | Android key alias | User confirmed configured |
| `KEYSTORE_BASE64` | Android keystore (base64-encoded) | User confirmed configured |
| `WINDOWS_CERT_P12_BASE64` | Windows code signing cert | Optional - step skips if missing |
| `WINDOWS_CERT_PASSWORD` | Windows cert password | Optional - step skips if missing |

### Platform Build Targets

Total: 13 platform targets

**Linux (5 targets):**
- x86_64-unknown-linux-musl
- i686-unknown-linux-musl
- aarch64-unknown-linux-musl
- armv7-unknown-linux-musleabihf
- arm-unknown-linux-musleabihf

**FreeBSD (1 target - NEW):**
- x86_64-unknown-freebsd

**Android (3 targets):**
- aarch64-linux-android (binary)
- aarch64-linux-android (APK + AAB)
- armv7-linux-androideabi (binary)

**macOS (2 targets):**
- x86_64-apple-darwin
- aarch64-apple-darwin

**Windows (3 targets):**
- x86_64-pc-windows-msvc
- i686-pc-windows-msvc
- aarch64-pc-windows-msvc

### Android Build Dependencies

**Required files:**
- `./mobile/build.gh.sh` - Android build script (must exist)
- `./mobile/app/` - Android app directory
- Keystore will be synthesized from `KEYSTORE_BASE64` secret at runtime

**Build outputs:**
- `./mobile/app/build/outputs/apk/release/app-release.apk` → renamed to `dure-sijang-all-signed.apk`
- `./mobile/app/build/outputs/bundle/release/app-release.aab` → renamed to `dure-sijang-all.aab`

### Workflow Triggers

All workflows support these triggers:

**release.yml:**
- `push: tags: v*` - Tag push triggers full build + release
- `push: branches: test` - Test branch triggers build without release
- `workflow_dispatch` - Manual trigger with platform selection

**Other workflows:**
- Specific to their deployment targets (Google Play, MS Store, Snap, docs)

## Validation Strategy

### Pre-Implementation Checks

1. **Verify Android build script exists:**
```bash
test -f ./mobile/build.gh.sh && echo "✓ Found" || echo "✗ Missing"
```

2. **Check GITHUB_TOKEN permissions:**
   - Repository Settings → Actions → General → Workflow permissions
   - Ensure "Read and write permissions" is enabled

3. **Confirm secrets configuration:**
   - Repository Settings → Secrets and variables → Actions
   - Verify VT_API_KEY, STORE_PASSWORD, KEY_PASSWORD, KEY_ALIAS, KEYSTORE_BASE64

### Post-Implementation Validation

**1. Syntax Validation:**
```bash
# Install actionlint (if not available)
brew install actionlint  # or download from GitHub releases

# Validate all workflows
actionlint .github/workflows/*.yml
```

**2. Test Run Options:**

**Option A: Test branch (recommended first)**
```bash
git checkout -b test
git push origin test
# Triggers build without creating release
```

**Option B: Test tag**
```bash
git tag v0.0.21-test
git push origin v0.0.21-test
# Triggers full build + release creation
```

**Option C: Manual trigger**
- GitHub → Actions → "Build and Release" → Run workflow
- Select platform (e.g., "linux" for quick test)

**3. Expected Outputs:**

**Successful build produces:**
- 13 platform artifacts in "Actions" tab
- VirusTotal scan results (or skip message if VT_API_KEY missing)
- GitHub release with:
  - Changelog from CHANGELOG.md
  - VirusTotal report links
  - All platform binaries (tar.gz for Unix, .exe for Windows, .apk/.aab for Android)

**4. Validation Checklist:**

- [ ] All 13 platform builds complete successfully
- [ ] FreeBSD build completes (may take 10-15 min in VM)
- [ ] Windows binaries renamed correctly (x86_64.exe, i686.exe, aarch64.exe)
- [ ] Android APK + AAB artifacts uploaded
- [ ] VirusTotal scan completes or skips gracefully
- [ ] Release created with proper formatting
- [ ] Binary names use "dure-sijang" (not "uad-shizuku")

### Rollback Plan

**If issues arise:**
1. Revert commit: `git revert HEAD`
2. Push revert: `git push origin main`
3. Workflows automatically revert to previous version

**Git history preserves:**
- All previous workflow versions
- Can cherry-pick specific fixes if needed

## Edge Cases & Considerations

### 1. FreeBSD Build Time
**Issue:** FreeBSD builds run in VM, significantly slower (~10-15 min)  
**Mitigation:** Workflow allows manual platform selection via `workflow_dispatch` to skip FreeBSD for quick iterations.

### 2. Windows Signing Certificate Missing
**Issue:** If `WINDOWS_CERT_P12_BASE64` or `WINDOWS_CERT_PASSWORD` not set, signing step fails  
**Current behavior:** Step has `continue-on-error: true`, build continues without signing  
**Impact:** Windows binaries released unsigned (users may see SmartScreen warnings)  
**Recommendation:** Set up code signing certificate for production releases.

### 3. VirusTotal API Rate Limits
**Issue:** Free tier limited to 4 requests/min  
**Mitigation:** Script enforces delay between requests (`VT_REQUEST_RATE=4` env var), handles 429 errors with retry.

### 4. VirusTotal 409 Errors (File Already Scanned)
**Issue:** GitHub Action fails on 409, breaking workflow  
**Solution:** Python script fetches existing report instead, continues workflow.

### 5. Android Build Script Missing
**Issue:** If `./mobile/build.gh.sh` doesn't exist, Android build fails  
**Mitigation:** Verify file exists before implementation (pre-implementation checklist).

### 6. RELEASE_TOKEN vs GITHUB_TOKEN
**Issue:** Current workflow uses `RELEASE_TOKEN`, reference uses `GITHUB_TOKEN`  
**Decision:** Switch to `GITHUB_TOKEN` (user approved)  
**Impact:** Must ensure GITHUB_TOKEN has `contents: write` permission in repo settings.

## Success Criteria

1. ✅ All 5 workflow files copied and adapted from reference
2. ✅ `.github/scripts/virustotal_scan.py` added
3. ✅ All "uad-shizuku" references replaced with "dure-sijang"
4. ✅ FreeBSD build enabled with pkg-config synthesis
5. ✅ Windows signing enhanced with pre-validation
6. ✅ Windows binary renaming step added
7. ✅ VirusTotal scanner switched to Python with 409 handling
8. ✅ GITHUB_TOKEN used for release creation
9. ✅ All workflows pass syntax validation
10. ✅ Test run completes successfully (all platforms build, release created)

## Open Questions

None - all design decisions validated with user.

## Next Steps

1. Write implementation plan using `writing-plans` skill
2. Execute plan: copy files, apply transformations, validate syntax
3. Commit changes with descriptive message
4. Test with `test` branch push or manual workflow_dispatch
5. Verify outputs and rollback if issues found

## References

- Reference workflows: `reference/uad-shizuku/.github/workflows/`
- Reference VirusTotal scanner: `reference/uad-shizuku/.github/scripts/virustotal_scan.py`
- Current workflows: `.github/workflows/`
- Project Cargo.toml: `mobile/Cargo.toml` (package name: `dure-sijang`)
- Android build script: `./mobile/build.gh.sh` (to be verified)
