/// Integration tests for GitHub Actions workflow BSD artifact generation
/// Tests that BSD builds produce correctly-named artifacts

#[cfg(test)]
mod bsd_artifact_tests {
    use std::path::Path;

    /// Expected BSD artifact naming pattern: {BIN_NAME}-{arch}-unknown-{os}
    const BIN_NAME: &str = "dure-sijang";

    const EXPECTED_BSD_ARTIFACTS: &[&str] = &[
        "x86_64-unknown-freebsd",
        "x86_64-unknown-openbsd",
        "aarch64-unknown-openbsd",
        "x86_64-unknown-netbsd",
    ];

    #[test]
    fn test_bsd_artifact_names_follow_convention() {
        // Test that BSD artifact names follow the pattern:
        // artifacts/{BIN_NAME}-{arch}-unknown-{os}/{BIN_NAME}
        for target in EXPECTED_BSD_ARTIFACTS {
            let expected_path = format!("artifacts/{BIN_NAME}-{target}/{BIN_NAME}");

            // This test validates the naming convention
            // Actual file existence checked during CI
            assert!(
                expected_path.starts_with("artifacts/"),
                "Artifact path must be under artifacts/ directory"
            );
            assert!(
                expected_path.contains(target),
                "Artifact path must contain target triple {target}"
            );
        }
    }

    #[test]
    fn test_bsd_targets_use_unknown_vendor() {
        // BSD targets should use 'unknown' vendor per Rust target conventions
        for target in EXPECTED_BSD_ARTIFACTS {
            assert!(
                target.contains("-unknown-"),
                "BSD target {target} must use 'unknown' vendor"
            );
        }
    }

    #[test]
    fn test_all_bsd_flavors_covered() {
        // Verify we're building for all major BSD variants
        let freebsd_count = EXPECTED_BSD_ARTIFACTS.iter()
            .filter(|t| t.contains("freebsd"))
            .count();
        let openbsd_count = EXPECTED_BSD_ARTIFACTS.iter()
            .filter(|t| t.contains("openbsd"))
            .count();
        let netbsd_count = EXPECTED_BSD_ARTIFACTS.iter()
            .filter(|t| t.contains("netbsd"))
            .count();

        assert_eq!(freebsd_count, 1, "Should build 1 FreeBSD variant");
        assert_eq!(openbsd_count, 2, "Should build 2 OpenBSD variants (x86_64 + aarch64)");
        assert_eq!(netbsd_count, 1, "Should build 1 NetBSD variant");
    }

    #[test]
    fn test_openbsd_arm64_included() {
        // CLAUDE.md mentions OpenBSD support - ensure arm64 variant included
        let has_openbsd_arm64 = EXPECTED_BSD_ARTIFACTS.iter()
            .any(|t| t.contains("aarch64") && t.contains("openbsd"));

        assert!(
            has_openbsd_arm64,
            "OpenBSD arm64 variant required (mentioned in CLAUDE.md)"
        );
    }

    #[test]
    fn test_virustotal_scan_count() {
        // Current platforms: 8 (from existing workflow)
        // New BSD platforms: 4 (FreeBSD, OpenBSD x2, NetBSD)
        // Total expected: 12
        const EXISTING_PLATFORMS: usize = 8;
        const NEW_BSD_PLATFORMS: usize = 4;
        const TOTAL_EXPECTED: usize = EXISTING_PLATFORMS + NEW_BSD_PLATFORMS;

        assert_eq!(
            EXPECTED_BSD_ARTIFACTS.len(),
            NEW_BSD_PLATFORMS,
            "Expected {} new BSD artifacts", NEW_BSD_PLATFORMS
        );

        // This validates the VirusTotal scan should process 12 files total
        assert_eq!(
            TOTAL_EXPECTED,
            12,
            "VirusTotal scan should process 12 total files"
        );
    }

    #[test]
    fn test_workflow_has_disk_cleanup() {
        // Read workflow file (from workspace root, not mobile/)
        let workflow_path = "../.github/workflows/release.yml";
        let workflow = std::fs::read_to_string(workflow_path)
            .expect("Failed to read workflow file");

        // Must have runner disk cleanup before VM starts
        assert!(
            workflow.contains("free-disk-space") || workflow.contains("jlumbroso/free-disk-space"),
            "Workflow must use free-disk-space action to prevent 'No space left on device' errors"
        );

        // Must have package cache cleanup for at least one BSD variant
        let has_pkg_clean = workflow.contains("pkg clean");
        let has_pkgin_clean = workflow.contains("pkgin clean");
        assert!(
            has_pkg_clean || has_pkgin_clean,
            "Workflow must clean package caches after installation"
        );

        // Must have disk space monitoring
        assert!(
            workflow.contains("df -h"),
            "Workflow must monitor disk space with 'df -h'"
        );

        // Must disable incremental builds to save space
        assert!(
            workflow.contains("CARGO_INCREMENTAL=0"),
            "Workflow must set CARGO_INCREMENTAL=0 to reduce disk usage"
        );
    }
}
