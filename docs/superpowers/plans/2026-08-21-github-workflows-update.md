# GitHub Workflows Update Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update all GitHub Actions workflows from reference/uad-shizuku with FreeBSD support, enhanced Windows signing, Python VirusTotal scanner, and dure-sijang name replacements.

**Architecture:** Complete file replacement of 5 workflow files + 1 new Python script. Apply systematic name transformations (uad-shizuku → dure-sijang, RELEASE_TOKEN → GITHUB_TOKEN). Validate with actionlint before commit.

**Tech Stack:** GitHub Actions, Python 3.11, sed, actionlint

## Global Constraints

- All workflow files use GitHub Actions YAML syntax
- Name transformations: `uad-shizuku` → `dure-sijang`, `uad_shizuku` → `dure_sijang`
- Token replacement: `RELEASE_TOKEN` → `GITHUB_TOKEN` (release.yml only)
- Python script must be executable (`chmod +x`)
- All workflows must pass `actionlint` validation
- Commit message format: Conventional Commits (feat/fix/refactor prefix)

---

### Task 1: Pre-Implementation Verification

**Files:**
- Check: `./mobile/build.gh.sh`

**Interfaces:**
- Consumes: None
- Produces: Confirmation that Android build script exists

- [ ] **Step 1: Verify Android build script exists**

```bash
test -f ./mobile/build.gh.sh && echo "✓ Found" || echo "✗ Missing"
```

Expected: `✓ Found`

If missing, workflow will fail at Android build step. This is a blocker.

---

### Task 2: Create Scripts Directory and Copy VirusTotal Scanner

**Files:**
- Create: `.github/scripts/` (directory)
- Create: `.github/scripts/virustotal_scan.py`

**Interfaces:**
- Consumes: `reference/uad-shizuku/.github/scripts/virustotal_scan.py` (source file)
- Produces: `.github/scripts/virustotal_scan.py` (198 lines, executable Python script)

- [ ] **Step 1: Create scripts directory**

```bash
mkdir -p .github/scripts
```

Expected: Directory created (no output if successful)

- [ ] **Step 2: Copy VirusTotal scanner from reference**

```bash
cp reference/uad-shizuku/.github/scripts/virustotal_scan.py .github/scripts/virustotal_scan.py
```

Expected: File copied (no output if successful)

- [ ] **Step 3: Make script executable**

```bash
chmod +x .github/scripts/virustotal_scan.py
```

Expected: Permissions updated (no output if successful)

- [ ] **Step 4: Verify script contents**

```bash
head -20 .github/scripts/virustotal_scan.py
```

Expected output:
```
#!/usr/bin/env python3
"""
VirusTotal Scanner with 409 Error Handling
...
```

- [ ] **Step 5: Commit**

```bash
git add .github/scripts/virustotal_scan.py
git commit -m "feat(ci): add Python VirusTotal scanner with 409 error handling

- Handles 409 errors (file already scanned) by fetching existing report
- Handles 429 errors (rate limit) with 60s retry
- Configurable rate limit via VT_REQUEST_RATE env var
- Computes SHA256 hash for existing report lookup

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit created with 1 file changed, 198 insertions

---

### Task 3: Copy and Update release.yml

**Files:**
- Replace: `.github/workflows/release.yml`

**Interfaces:**
- Consumes: `reference/uad-shizuku/.github/workflows/release.yml` (source), Task 2 output (virustotal_scan.py)
- Produces: `.github/workflows/release.yml` (563 lines with name transformations applied)

- [ ] **Step 1: Copy release.yml from reference**

```bash
cp reference/uad-shizuku/.github/workflows/release.yml .github/workflows/release.yml
```

Expected: File replaced (no output if successful)

- [ ] **Step 2: Replace uad-shizuku with dure-sijang (hyphenated form)**

```bash
sed -i.bak 's/uad-shizuku/dure-sijang/g' .github/workflows/release.yml && rm .github/workflows/release.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 3: Replace uad_shizuku with dure_sijang (underscore form)**

```bash
sed -i.bak 's/uad_shizuku/dure_sijang/g' .github/workflows/release.yml && rm .github/workflows/release.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 4: Replace RELEASE_TOKEN with GITHUB_TOKEN**

```bash
sed -i.bak 's/RELEASE_TOKEN/GITHUB_TOKEN/g' .github/workflows/release.yml && rm .github/workflows/release.yml.bak
```

Expected: Token reference updated (no output if successful)

- [ ] **Step 5: Verify transformations**

```bash
grep -c "dure-sijang" .github/workflows/release.yml
grep -c "uad-shizuku" .github/workflows/release.yml
grep -c "GITHUB_TOKEN" .github/workflows/release.yml
grep -c "RELEASE_TOKEN" .github/workflows/release.yml
```

Expected output:
```
20    # dure-sijang count (should be > 0)
0     # uad-shizuku count (should be 0)
2     # GITHUB_TOKEN count (should be > 0)
0     # RELEASE_TOKEN count (should be 0)
```

- [ ] **Step 6: Verify key improvements are present**

Check FreeBSD enabled:
```bash
grep -A 3 "FreeBSD x86_64" .github/workflows/release.yml | head -4
```

Expected output:
```yaml
- name: FreeBSD x86_64
  target: x86_64-unknown-freebsd
  build-args: "--release"
  freebsd_version: "14.2"
```

Check Windows certificate validation:
```bash
grep -A 2 "HasPrivateKey" .github/workflows/release.yml | head -3
```

Expected output:
```
echo "Certificate HasPrivateKey: $($cert.HasPrivateKey)"
...
```

Check Python VirusTotal scanner:
```bash
grep "virustotal_scan.py" .github/workflows/release.yml
```

Expected output:
```
python .github/scripts/virustotal_scan.py \
```

- [ ] **Step 7: Commit**

```bash
git add .github/workflows/release.yml
git commit -m "feat(ci): update release.yml with reference improvements

Major changes:
- Enable FreeBSD x86_64 builds with pkg-config synthesis
- Add Windows certificate pre-validation (HasPrivateKey, expiration)
- Add Windows binary renaming step (arch-specific names)
- Replace VirusTotal GitHub Action with Python scanner
- Add explicit permissions: contents: write
- Replace all uad-shizuku references with dure-sijang
- Use GITHUB_TOKEN instead of RELEASE_TOKEN

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit created with 1 file changed

---

### Task 4: Copy and Update release-googleplay.yml

**Files:**
- Replace: `.github/workflows/release-googleplay.yml`

**Interfaces:**
- Consumes: `reference/uad-shizuku/.github/workflows/release-googleplay.yml` (source)
- Produces: `.github/workflows/release-googleplay.yml` (with name transformations applied)

- [ ] **Step 1: Copy release-googleplay.yml from reference**

```bash
cp reference/uad-shizuku/.github/workflows/release-googleplay.yml .github/workflows/release-googleplay.yml
```

Expected: File replaced (no output if successful)

- [ ] **Step 2: Replace uad-shizuku with dure-sijang**

```bash
sed -i.bak 's/uad-shizuku/dure-sijang/g' .github/workflows/release-googleplay.yml && rm .github/workflows/release-googleplay.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 3: Replace uad_shizuku with dure_sijang**

```bash
sed -i.bak 's/uad_shizuku/dure_sijang/g' .github/workflows/release-googleplay.yml && rm .github/workflows/release-googleplay.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 4: Verify transformations**

```bash
grep -c "dure-sijang" .github/workflows/release-googleplay.yml
grep -c "uad-shizuku" .github/workflows/release-googleplay.yml
```

Expected output:
```
>0    # dure-sijang count
0     # uad-shizuku count (should be 0)
```

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release-googleplay.yml
git commit -m "feat(ci): update release-googleplay.yml from reference

- Replace all uad-shizuku references with dure-sijang
- Sync with reference workflow improvements

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit created with 1 file changed

---

### Task 5: Copy and Update release-msstore.yml

**Files:**
- Replace: `.github/workflows/release-msstore.yml`

**Interfaces:**
- Consumes: `reference/uad-shizuku/.github/workflows/release-msstore.yml` (source)
- Produces: `.github/workflows/release-msstore.yml` (with name transformations applied)

- [ ] **Step 1: Copy release-msstore.yml from reference**

```bash
cp reference/uad-shizuku/.github/workflows/release-msstore.yml .github/workflows/release-msstore.yml
```

Expected: File replaced (no output if successful)

- [ ] **Step 2: Replace uad-shizuku with dure-sijang**

```bash
sed -i.bak 's/uad-shizuku/dure-sijang/g' .github/workflows/release-msstore.yml && rm .github/workflows/release-msstore.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 3: Replace uad_shizuku with dure_sijang**

```bash
sed -i.bak 's/uad_shizuku/dure_sijang/g' .github/workflows/release-msstore.yml && rm .github/workflows/release-msstore.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 4: Verify transformations**

```bash
grep -c "dure-sijang" .github/workflows/release-msstore.yml
grep -c "uad-shizuku" .github/workflows/release-msstore.yml
```

Expected output:
```
>0    # dure-sijang count
0     # uad-shizuku count (should be 0)
```

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release-msstore.yml
git commit -m "feat(ci): update release-msstore.yml from reference

- Replace all uad-shizuku references with dure-sijang
- Sync with reference workflow improvements

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit created with 1 file changed

---

### Task 6: Copy and Update release-snap.yml

**Files:**
- Replace: `.github/workflows/release-snap.yml`

**Interfaces:**
- Consumes: `reference/uad-shizuku/.github/workflows/release-snap.yml` (source)
- Produces: `.github/workflows/release-snap.yml` (with name transformations applied)

- [ ] **Step 1: Copy release-snap.yml from reference**

```bash
cp reference/uad-shizuku/.github/workflows/release-snap.yml .github/workflows/release-snap.yml
```

Expected: File replaced (no output if successful)

- [ ] **Step 2: Replace uad-shizuku with dure-sijang**

```bash
sed -i.bak 's/uad-shizuku/dure-sijang/g' .github/workflows/release-snap.yml && rm .github/workflows/release-snap.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 3: Replace uad_shizuku with dure_sijang**

```bash
sed -i.bak 's/uad_shizuku/dure_sijang/g' .github/workflows/release-snap.yml && rm .github/workflows/release-snap.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 4: Verify transformations**

```bash
grep -c "dure-sijang" .github/workflows/release-snap.yml
grep -c "uad-shizuku" .github/workflows/release-snap.yml
```

Expected output:
```
>0    # dure-sijang count
0     # uad-shizuku count (should be 0)
```

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/release-snap.yml
git commit -m "feat(ci): update release-snap.yml from reference

- Replace all uad-shizuku references with dure-sijang
- Sync with reference workflow improvements

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit created with 1 file changed

---

### Task 7: Copy and Update vite.docs.yml

**Files:**
- Replace: `.github/workflows/vite.docs.yml`

**Interfaces:**
- Consumes: `reference/uad-shizuku/.github/workflows/vite.docs.yml` (source)
- Produces: `.github/workflows/vite.docs.yml` (with name transformations applied)

- [ ] **Step 1: Copy vite.docs.yml from reference**

```bash
cp reference/uad-shizuku/.github/workflows/vite.docs.yml .github/workflows/vite.docs.yml
```

Expected: File replaced (no output if successful)

- [ ] **Step 2: Replace uad-shizuku with dure-sijang**

```bash
sed -i.bak 's/uad-shizuku/dure-sijang/g' .github/workflows/vite.docs.yml && rm .github/workflows/vite.docs.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 3: Replace uad_shizuku with dure_sijang**

```bash
sed -i.bak 's/uad_shizuku/dure_sijang/g' .github/workflows/vite.docs.yml && rm .github/workflows/vite.docs.yml.bak
```

Expected: All occurrences replaced (no output if successful)

- [ ] **Step 4: Verify transformations**

```bash
grep -c "dure-sijang" .github/workflows/vite.docs.yml || echo "0"
grep -c "uad-shizuku" .github/workflows/vite.docs.yml
```

Expected output:
```
0 or >0  # dure-sijang count (may be 0 if docs workflow doesn't reference binary name)
0        # uad-shizuku count (should be 0)
```

- [ ] **Step 5: Commit**

```bash
git add .github/workflows/vite.docs.yml
git commit -m "feat(ci): update vite.docs.yml from reference

- Replace any uad-shizuku references with dure-sijang
- Sync with reference workflow improvements

Co-Authored-By: Claude Sonnet 4.5 <noreply@anthropic.com>"
```

Expected: Commit created with 1 file changed

---

### Task 8: Validate Workflow Syntax

**Files:**
- Validate: All `.github/workflows/*.yml` files

**Interfaces:**
- Consumes: All workflow files from Tasks 3-7
- Produces: Validation report (all workflows pass actionlint)

- [ ] **Step 1: Check if actionlint is installed**

```bash
which actionlint || echo "Not installed"
```

Expected: Path to actionlint binary, or "Not installed"

If not installed, skip to Step 3 (manual GitHub validation)

- [ ] **Step 2: Run actionlint on all workflows**

```bash
actionlint .github/workflows/*.yml
```

Expected output: No errors (empty output means success)

If errors found, review and fix before proceeding.

- [ ] **Step 3: Manually validate syntax (alternative if actionlint not available)**

Check YAML syntax with Python:
```bash
python3 -c "import yaml; [yaml.safe_load(open(f)) for f in ['.github/workflows/release.yml', '.github/workflows/release-googleplay.yml', '.github/workflows/release-msstore.yml', '.github/workflows/release-snap.yml', '.github/workflows/vite.docs.yml']]" 2>&1 && echo "✓ All YAML files valid"
```

Expected output: `✓ All YAML files valid`

- [ ] **Step 4: List all workflow files with line counts**

```bash
wc -l .github/workflows/*.yml
```

Expected output: 5 files listed with line counts, total ~600-800 lines

---

### Task 9: Final Verification and Summary

**Files:**
- Verify: All changed files in `.github/`

**Interfaces:**
- Consumes: All outputs from Tasks 2-7
- Produces: Summary of changes, ready for push

- [ ] **Step 1: Check git status**

```bash
git status --short
```

Expected output: Clean working tree (all changes committed)

- [ ] **Step 2: Verify all commits created**

```bash
git log --oneline --graph -7
```

Expected output: 6 commits (1 for virustotal_scan.py, 5 for workflow files)

- [ ] **Step 3: List all files in .github/ directory**

```bash
find .github -type f | sort
```

Expected output:
```
.github/scripts/virustotal_scan.py
.github/workflows/release-googleplay.yml
.github/workflows/release-msstore.yml
.github/workflows/release-snap.yml
.github/workflows/release.yml
.github/workflows/vite.docs.yml
```

- [ ] **Step 4: Verify no uad-shizuku references remain**

```bash
grep -r "uad-shizuku" .github/ || echo "✓ No uad-shizuku references found"
grep -r "uad_shizuku" .github/ || echo "✓ No uad_shizuku references found"
```

Expected output:
```
✓ No uad-shizuku references found
✓ No uad_shizuku references found
```

- [ ] **Step 5: Verify dure-sijang references exist**

```bash
grep -r "dure-sijang" .github/workflows/ | wc -l
```

Expected output: >20 (at least 20 occurrences across all workflows)

- [ ] **Step 6: Summary report**

```bash
echo "=== GitHub Workflows Update Complete ==="
echo ""
echo "Files updated:"
echo "  - .github/scripts/virustotal_scan.py (NEW)"
echo "  - .github/workflows/release.yml (REPLACED)"
echo "  - .github/workflows/release-googleplay.yml (REPLACED)"
echo "  - .github/workflows/release-msstore.yml (REPLACED)"
echo "  - .github/workflows/release-snap.yml (REPLACED)"
echo "  - .github/workflows/vite.docs.yml (REPLACED)"
echo ""
echo "Transformations applied:"
echo "  - uad-shizuku → dure-sijang"
echo "  - uad_shizuku → dure_sijang"
echo "  - RELEASE_TOKEN → GITHUB_TOKEN (release.yml only)"
echo ""
echo "Key improvements:"
echo "  ✓ FreeBSD support enabled"
echo "  ✓ Windows certificate pre-validation"
echo "  ✓ Windows binary renaming step"
echo "  ✓ Python VirusTotal scanner with 409 handling"
echo "  ✓ Explicit permissions in release job"
echo ""
echo "Next steps:"
echo "  1. Push to test branch: git push origin HEAD:test"
echo "  2. Verify build in GitHub Actions"
echo "  3. Check all 13 platform builds complete"
echo "  4. Merge to main if successful"
```

Expected output: Summary printed to console

---

## Self-Review Checklist

**Spec coverage:**
- ✅ Task 2: VirusTotal Python scanner added
- ✅ Task 3: release.yml updated with FreeBSD, Windows signing, Python VT scanner
- ✅ Tasks 4-7: All other workflows updated
- ✅ All tasks: Name transformations applied (uad-shizuku → dure-sijang)
- ✅ Task 3: GITHUB_TOKEN replaces RELEASE_TOKEN
- ✅ Task 8: Syntax validation
- ✅ Task 9: Final verification

**Placeholder scan:**
- ✅ No TBD or TODO markers
- ✅ All commands complete with expected outputs
- ✅ All file paths exact
- ✅ All transformations explicit

**Type consistency:**
- ✅ File names consistent across all tasks
- ✅ Binary name "dure-sijang" used consistently
- ✅ Token name "GITHUB_TOKEN" used consistently

**No gaps found.**
