//! Real Project Validation Integration Tests for Phase 7 Batch D P0-4
//!
//! These tests validate against actual ai_project_template projects on disk.
//! They are marked with #[ignore] to avoid running in CI (requires specific paths).
//! Run with: cargo test --test real_project_validation -- --ignored

use std::collections::HashSet;
use std::path::Path;

use fpga_flow_mind_lib::workspace::scanner::scan_workspace_files;
use fpga_flow_mind_lib::workspace::stage_detector::detect_stages;
use fpga_flow_mind_lib::models::enums::ErrorCode;

const PRIMARY_SAMPLE: &str = "/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync";
const SECONDARY_SAMPLE: &str = "/Users/ckstar/Repo/znxt_ofdm/fpga_project_fft";

// ─── Test 1: Primary Sample - Stage Detection ─────────────────────────

#[test]
#[ignore = "requires real project at PRIMARY_SAMPLE"]
fn primary_sample_detects_l0_l1_rtl() {
    let root = Path::new(PRIMARY_SAMPLE);
    assert!(root.exists(), "Primary sample must exist at {}", PRIMARY_SAMPLE);

    let scan = scan_workspace_files(root);
    let result = detect_stages(root, &scan.files);

    let stage_ids: HashSet<String> = result.stages.iter().map(|s| s.stage_id.clone()).collect();

    println!("Primary sample stages: {:?}", stage_ids);
    println!("Missing stages: {:?}", result.missing);

    assert!(stage_ids.contains("L0"), "Must detect L0");
    assert!(stage_ids.contains("L1"), "Must detect L1");
    assert!(stage_ids.contains("RTL"), "Must detect RTL");

    // L0 should have files from deep source tree
    let l0 = result.stages.iter().find(|s| s.stage_id == "L0").unwrap();
    assert!(l0.file_count > 0, "L0 must have >0 files, got {}", l0.file_count);
    assert!(l0.source_path.contains("L0_external"), "L0 source_path should point to L0_external");

    // RTL should have files
    let rtl = result.stages.iter().find(|s| s.stage_id == "RTL").unwrap();
    assert!(rtl.file_count > 0, "RTL must have >0 files, got {}", rtl.file_count);
    assert!(rtl.source_path.contains("verilog_model"), "RTL source_path should contain verilog_model");
}

// ─── Test 2: Secondary Sample - Stage Detection ─────────────────────────

#[test]
#[ignore = "requires real project at SECONDARY_SAMPLE"]
fn secondary_sample_detects_stages() {
    let root = Path::new(SECONDARY_SAMPLE);
    assert!(root.exists(), "Secondary sample must exist at {}", SECONDARY_SAMPLE);

    let scan = scan_workspace_files(root);
    let result = detect_stages(root, &scan.files);

    let stage_ids: HashSet<String> = result.stages.iter().map(|s| s.stage_id.clone()).collect();

    println!("Secondary sample stages: {:?}", stage_ids);

    assert!(stage_ids.contains("L0"), "Must detect L0");
    assert!(stage_ids.contains("RTL"), "Must detect RTL");

    // L0 should have files
    let l0 = result.stages.iter().find(|s| s.stage_id == "L0").unwrap();
    assert!(l0.file_count > 0, "L0 must have >0 files");
}

// ─── Test 3: Deep Source Scanning - No Timeout ─────────────────────────

#[test]
#[ignore = "requires real project at PRIMARY_SAMPLE"]
fn primary_sample_deep_scan_no_timeout() {
    let root = Path::new(PRIMARY_SAMPLE);
    let scan = scan_workspace_files(root);

    let timeout_warnings: Vec<_> = scan.warnings.iter()
        .filter(|w| w.error_code == ErrorCode::ScanTimeout)
        .collect();

    println!("Total files scanned: {}", scan.files.len());
    println!("Timeout warnings: {}", timeout_warnings.len());

    // Should have minimal or no timeout warnings (P0-2 fix)
    assert!(timeout_warnings.len() <= 5,
            "Too many scan_timeout warnings: {}", timeout_warnings.len());

    // Must find deep source files (depth 5 in ai_project_template)
    let deep_files: Vec<_> = scan.files.iter()
        .filter(|f| f.rel_path.contains("rx_02_coarse_sync") || f.rel_path.contains("shared_04_preamble"))
        .collect();

    println!("Deep source files found: {}", deep_files.len());
    for f in &deep_files {
        println!("  {}", f.rel_path);
    }

    assert!(!deep_files.is_empty(), "Must find deep source files (rx_02_coarse_sync, shared_04_preamble)");
}

// ─── Test 4: Noise Directory Skipping ───────────────────────────────────

#[test]
#[ignore = "requires real project at PRIMARY_SAMPLE"]
fn primary_sample_skips_noise_dirs() {
    let root = Path::new(PRIMARY_SAMPLE);
    let scan = scan_workspace_files(root);

    let noise_paths = ["__pycache__", ".git", ".claude", "vivado", "node_modules", "target"];

    for noise in &noise_paths {
        let found = scan.files.iter().any(|f| {
            let parts: Vec<&str> = f.rel_path.split('/').collect();
            parts.iter().any(|p| p == noise || p.starts_with(&format!("{}.", noise)))
        });
        assert!(!found, "Must not contain files from noise directory: {}", noise);
    }

    // But should still have valid source files
    let py_files: Vec<_> = scan.files.iter().filter(|f| f.rel_path.ends_with(".py")).collect();
    let v_files: Vec<_> = scan.files.iter().filter(|f| f.rel_path.ends_with(".v") || f.rel_path.ends_with(".sv")).collect();

    println!("Python files: {}, Verilog files: {}", py_files.len(), v_files.len());
    assert!(!py_files.is_empty(), "Must have Python files");
    assert!(!v_files.is_empty(), "Must have Verilog files");
}

// ─── Test 5: Checksum Verification ──────────────────────────────────────

#[test]
#[ignore = "requires real project at PRIMARY_SAMPLE"]
fn primary_sample_checksum_consistency() {
    use std::process::Command;

    let root = Path::new(PRIMARY_SAMPLE);

    // Compute checksum before
    let before = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && find src -type f \\( -name '*.py' -o -name '*.v' -o -name '*.sv' -o -name '*.md' \\) -exec shasum -a 256 {{}} \\; | sort",
            root.display()
        ))
        .output()
        .expect("Failed to compute checksum");

    let before_str = String::from_utf8(before.stdout).unwrap();

    // Re-scan (read-only operation)
    let _scan = scan_workspace_files(root);
    let _result = detect_stages(root, &_scan.files);

    // Compute checksum after
    let after = Command::new("sh")
        .arg("-c")
        .arg(format!(
            "cd {} && find src -type f \\( -name '*.py' -o -name '*.v' -o -name '*.sv' -o -name '*.md' \\) -exec shasum -a 256 {{}} \\; | sort",
            root.display()
        ))
        .output()
        .expect("Failed to compute checksum");

    let after_str = String::from_utf8(after.stdout).unwrap();

    assert_eq!(before_str, after_str, "Checksums must match before and after read-only operations");

    println!("Checksum verification passed: {} files", before_str.lines().count());
}
