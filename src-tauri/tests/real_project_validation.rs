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

// ─── Checksum helper (pure Rust, no Command::new) ──────────────────────

fn compute_src_checksum(root: &Path) -> Vec<(String, String)> {
    let mut results: Vec<(String, String)> = Vec::new();
    let src_dir = root.join("src");
    if !src_dir.exists() {
        return results;
    }
    collect_checksums(&src_dir, &mut results);
    // stable sort by relative path
    results.sort_by(|a, b| a.0.cmp(&b.0));
    results
}

fn collect_checksums(dir: &Path, out: &mut Vec<(String, String)>) {
    use sha2::Digest;

    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
        // skip noise directories
        if path.is_dir() {
            if matches!(name,
                "__pycache__" | ".git" | ".claude" | "node_modules" | "target" |
                "vivado" | "reports" | "build" | "dist" | ".venv" | "venv" |
                "sim_build" | ".tox" | "htmlcov" | ".pytest_cache" | ".mypy_cache" |
                ".ruff_cache" | ".egg-info" | ".idea" | ".vscode"
            ) {
                continue;
            }
            collect_checksums(&path, out);
            continue;
        }
        // only .py / .v / .sv / .md
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        if !matches!(ext, "py" | "v" | "sv" | "md") {
            continue;
        }
        let Ok(content) = std::fs::read(&path) else { continue };
        let hash = format!("{:x}", sha2::Sha256::digest(&content));
        let rel = path.strip_prefix(path.ancestors().nth(2).unwrap_or(&path))
            .unwrap_or(&path)
            .to_string_lossy()
            .to_string();
        out.push((rel, hash));
    }
}

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

// ─── Test 6: Phase 8 - L0/L4 Quality Blockers Fixed ─────────────────────

#[test]
#[ignore = "requires real project at PRIMARY_SAMPLE"]
fn primary_sample_l0_l4_quality_blockers_fixed() {
    use fpga_flow_mind_lib::commands::select_stage::resolve_stage_context;
    use fpga_flow_mind_lib::evidence::collector::EvidenceCollector;
    use fpga_flow_mind_lib::quality::{QualityIssueKind, QualityReportInput, QualityReporter, StageQualityInput};
    use fpga_flow_mind_lib::understanding::generator::{MockProvider, UnderstandingGenerator};
    use fpga_flow_mind_lib::views::generator::ViewGraphGenerator;
    use fpga_flow_mind_lib::views::models::{NodeType, ViewType};

    let root = Path::new(PRIMARY_SAMPLE);
    assert!(root.exists(), "Primary sample must exist at {}", PRIMARY_SAMPLE);

    // 修复前后校验和不变
    let checksum_before = compute_src_checksum(root);

    let l0_ctx = resolve_stage_context(PRIMARY_SAMPLE, "L0")
        .data
        .expect("L0 stage context must resolve");
    let l4_ctx = resolve_stage_context(PRIMARY_SAMPLE, "L4")
        .data
        .expect("L4 stage context must resolve");

    let mut l0_collector = EvidenceCollector::new("L0");
    let l0_collection = l0_collector.collect_from_stage_context(&l0_ctx);
    let mut l4_collector = EvidenceCollector::new("L4");
    let l4_collection = l4_collector.collect_from_stage_context(&l4_ctx);

    let l0_iu = UnderstandingGenerator::new(Box::new(MockProvider))
        .generate(&l0_collection)
        .expect("L0 understanding generation must succeed");
    let l4_iu = UnderstandingGenerator::new(Box::new(MockProvider))
        .generate(&l4_collection)
        .expect("L4 understanding generation must succeed");

    let l0_views = ViewGraphGenerator::generate_all(&l0_iu);
    let l4_views = ViewGraphGenerator::generate_all(&l4_iu);

    // ── L0: 标准粗同步流水线 ──
    let l0_step_names: Vec<&str> = l0_iu.processing_steps.iter().map(|s| s.name.as_str()).collect();
    let l0_expected = ["correlation", "energy", "metric", "smoothing", "peak_detection", "cfo_estimation"];
    for name in &l0_expected {
        assert!(
            l0_step_names.contains(name),
            "L0 processing_steps 应包含标准步骤 {}，实际: {:?}",
            name, l0_step_names
        );
    }

    let l0_dataflow = l0_views.iter().find(|v| v.view_type == ViewType::Dataflow).unwrap();
    let l0_noise_symbols = ["annotations", "dataclass", "Optional", "np", "PARAMS", "config", "data_width"];
    for noise in &l0_noise_symbols {
        assert!(
            !l0_dataflow.nodes.iter().any(|n| n.label == *noise),
            "L0 dataflow 不应出现噪声节点 {}", noise
        );
    }
    for name in &l0_expected {
        assert!(
            l0_dataflow.nodes.iter().any(|n| n.node_type == NodeType::ProcessingStep && n.label == *name),
            "L0 dataflow 应包含步骤节点 {}", name
        );
    }

    // L0 为 Python 算法阶段：dataflow 可生成算法处理链，但 timing 在无硬件时序证据时必须为空
    let l0_timing = l0_views.iter().find(|v| v.view_type == ViewType::Timing).unwrap();
    assert!(
        l0_timing.nodes.is_empty(),
        "L0 timing view 必须为空（无 cycle/clock/posedge 等硬件时序证据），实际节点: {:?}",
        l0_timing.nodes.iter().map(|n| &n.label).collect::<Vec<_>>()
    );
    let l0_timing_reason = l0_timing
        .meta
        .empty_reason
        .as_deref()
        .expect("L0 timing empty_reason 必须非空");
    assert!(
        l0_timing_reason.contains("cycle") || l0_timing_reason.contains("clock") || l0_timing_reason.contains("时序"),
        "L0 timing empty_reason 应说明缺少硬件时序证据: {}",
        l0_timing_reason
    );
    assert!(
        l0_timing_reason.contains("算法/函数顺序"),
        "L0 timing empty_reason 应说明 processing_steps 为算法/函数顺序: {}",
        l0_timing_reason
    );

    // ── L4: 周期精确流水线 + AXI-Stream I/O ──
    let l4_step_names: Vec<&str> = l4_iu.processing_steps.iter().map(|s| s.name.as_str()).collect();
    let l4_expected = ["input", "correlation", "energy", "metric", "detection", "output"];
    for name in &l4_expected {
        assert!(
            l4_step_names.contains(name),
            "L4 processing_steps 应包含周期精确步骤 {}，实际: {:?}",
            name, l4_step_names
        );
    }

    let l4_timing = l4_views.iter().find(|v| v.view_type == ViewType::Timing).unwrap();
    assert!(
        !l4_timing.nodes.is_empty(),
        "L4 timing view 必须非空，实际节点: {:?}",
        l4_timing.nodes.iter().map(|n| &n.label).collect::<Vec<_>>()
    );
    assert!(
        l4_timing.nodes.iter().any(|n| n.node_type == NodeType::PipelineStage),
        "L4 timing view 应包含 PipelineStage 节点"
    );
    // 精确断言 L4 timing PipelineStage 标签覆盖完整周期精确流水线
    let l4_timing_labels: Vec<&str> = l4_timing
        .nodes
        .iter()
        .filter(|n| n.node_type == NodeType::PipelineStage)
        .map(|n| n.label.as_str())
        .collect();
    assert_eq!(
        l4_timing_labels,
        vec!["input", "correlation", "energy", "metric", "detection", "output"],
        "L4 timing PipelineStage 标签必须按顺序覆盖完整周期精确流水线，实际：{:?}",
        l4_timing_labels
    );

    let l4_dataflow = l4_views.iter().find(|v| v.view_type == ViewType::Dataflow).unwrap();
    assert!(
        l4_dataflow.nodes.iter().any(|n| n.node_type == NodeType::InputSource && n.label.starts_with("s_")),
        "L4 dataflow 应包含 s_* 输入节点"
    );
    assert!(
        l4_dataflow.nodes.iter().any(|n| n.node_type == NodeType::OutputTarget && n.label.starts_with("m_")),
        "L4 dataflow 应包含 m_* 输出节点"
    );

    // dataflow / timing 均不得出现 import/typing/decorator 噪声节点
    let noise_symbols = ["annotations", "dataclass", "Optional", "np", "PARAMS", "config", "data_width"];
    for noise in &noise_symbols {
        assert!(
            !l0_dataflow.nodes.iter().any(|n| n.label == *noise),
            "L0 dataflow 不应出现噪声节点 {}", noise
        );
        assert!(
            !l4_dataflow.nodes.iter().any(|n| n.label == *noise),
            "L4 dataflow 不应出现噪声节点 {}", noise
        );
        assert!(
            !l4_timing.nodes.iter().any(|n| n.label == *noise),
            "L4 timing 不应出现噪声节点 {}", noise
        );
    }

    // ── QualityReport: 诚实暴露退化 ──
    let l0_expected_paths: Vec<String> = l0_ctx.files.iter().map(|f| f.source_path.clone()).collect();
    let l4_expected_paths: Vec<String> = l4_ctx.files.iter().map(|f| f.source_path.clone()).collect();

    let l0_stage_input = StageQualityInput {
        stage_id: "L0".to_string(),
        recognized_status: "available".to_string(),
        expected_status: None,
        expected_source_paths: Some(&l0_expected_paths),
        evidence: Some(&l0_collection),
        understanding: Some(&l0_iu),
        views: l0_views.iter().collect(),
        grounded_answer: None,
    };
    let l4_stage_input = StageQualityInput {
        stage_id: "L4".to_string(),
        recognized_status: "available".to_string(),
        expected_status: None,
        expected_source_paths: Some(&l4_expected_paths),
        evidence: Some(&l4_collection),
        understanding: Some(&l4_iu),
        views: l4_views.iter().collect(),
        grounded_answer: None,
    };

    let report = QualityReporter::new().evaluate(&QualityReportInput {
        sample_id: "primary-coarse-sync".to_string(),
        generated_at_marker: "2026-06-18T00:00:00Z".to_string(),
        stages: vec![l0_stage_input, l4_stage_input],
    });

    assert!(
        report.issues.iter().any(|i| i.kind == QualityIssueKind::NoisyEvidence),
        "QualityReport 应诚实记录 NoisyEvidence: {:?}",
        report.issues.iter().map(|i| format!("{:?}", i.kind)).collect::<Vec<_>>()
    );

    // 校验和一致性
    let checksum_after = compute_src_checksum(root);
    assert_eq!(
        checksum_before, checksum_after,
        "修复前后目标项目 src 校验和必须一致"
    );

    println!("Phase 8 quality blockers fixed for primary sample");
    println!("  L0 steps: {:?}", l0_step_names);
    println!("  L4 steps: {:?}", l4_step_names);
    println!("  L4 timing nodes: {}", l4_timing.nodes.len());
    println!("  Quality issues: {}", report.issues.len());
}
