/// `collect_evidence` Tauri command
///
/// 调用 `resolve_stage_context` 获取 StageContext，
/// 再委托 `EvidenceCollector` 收集证据。
///
/// 返回策略：
/// - resolve 失败 → 透传 CommandResult（success=false）
/// - stage_empty → success=true, data=None, error=Some(StageEmpty)
/// - 正常 → success=true, data=Some(EvidenceCollection)
///
/// collector 内部 warnings 不提升为 command failure。

use crate::evidence::collector::EvidenceCollector;
use crate::evidence::models::EvidenceCollection;
use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};

use super::select_stage::resolve_stage_context;

#[tauri::command]
pub fn collect_evidence(root_path: String, stage_id: String) -> CommandResult<EvidenceCollection> {
    // 1. 复用共享校验 + StageContext 构建
    let ctx_result = resolve_stage_context(&root_path, &stage_id);
    if !ctx_result.success {
        return CommandResult {
            success: false,
            data: None,
            error: ctx_result.error,
            warnings: ctx_result.warnings,
        };
    }

    let context = ctx_result.data.unwrap();

    // 2. 空阶段 → success=true, data=None, 标记 StageEmpty error（recoverable）
    if context.error_code == Some(ErrorCode::StageEmpty) {
        return CommandResult {
            success: true,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::StageEmpty,
                message: format!("阶段 {} 为空，无文件可收集", stage_id),
                recoverable: true,
                details: None,
                source_path: Some(context.source_path),
            }),
            warnings: Vec::new(),
        };
    }

    // 3. 正常收集
    let mut collector = EvidenceCollector::new(&stage_id);
    let collection = collector.collect_from_stage_context(&context);

    CommandResult {
        success: true,
        data: Some(collection),
        error: None,
        warnings: Vec::new(),
    }
}

#[cfg(test)]
mod tests {
    use crate::models::enums::ErrorCode;

    use super::*;

    /// 辅助：创建临时目录并写入文件
    fn touch(root: &std::path::Path, rel: &str, content: &[u8]) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }

    // ─── 正常场景 ──────────────────────────────────────────────────

    #[test]
    fn ev_01_available_stage_collects() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top():\n    pass\n");
        touch(root, "L0/helper.py", b"def helper():\n    return 1\n");

        let result = collect_evidence(root.to_str().unwrap().to_string(), "L0".to_string());
        assert!(result.success, "可用阶段应成功");
        assert!(result.data.is_some(), "应有 data");
        assert!(result.error.is_none(), "不应有 error");

        let col = result.data.unwrap();
        assert_eq!(col.stage_id, "L0");
        assert_eq!(col.evidence_items.len(), 2);
        assert_eq!(col.stats.files_processed, 2);
        assert!(col.stats.total_items >= 2);
        assert!(!col.index_by_path.is_empty());
    }

    #[test]
    fn ev_07_verilog_stage() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "rtl/top.v", b"module top(\n    input clk\n);\nendmodule\n");

        let result = collect_evidence(root.to_str().unwrap().to_string(), "RTL".to_string());
        assert!(result.success, "Verilog 阶段应成功");
        assert!(result.data.is_some());

        let col = result.data.unwrap();
        assert_eq!(col.stage_id, "RTL");
        assert!(!col.evidence_items.is_empty(), "应有证据项");
        assert_eq!(col.evidence_items[0].evidence_id, "EV-RTL-000001");
        assert_eq!(col.stats.items_by_kind.get("rtl"), Some(&1));
    }

    // ─── 空阶段 ────────────────────────────────────────────────────

    #[test]
    fn ev_02_empty_stage_no_data() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir(root.join("L0")).unwrap();
        touch(root, "L1/top.py", b"def top(): pass\n");

        let result = collect_evidence(root.to_str().unwrap().to_string(), "L0".to_string());
        assert!(result.success, "空阶段应返回 success=true");
        assert!(result.data.is_none(), "空阶段不应有 data");
        let err = result.error.as_ref().unwrap();
        assert_eq!(err.error_code, ErrorCode::StageEmpty);
        assert!(err.recoverable, "StageEmpty 应 recoverable=true");
    }

    // ─── 错误场景 ──────────────────────────────────────────────────

    #[test]
    fn ev_03_nonexistent_stage_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top(): pass\n");

        let result = collect_evidence(root.to_str().unwrap().to_string(), "L9".to_string());
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::NotDirectory
        );
    }

    #[test]
    fn ev_04_empty_stage_id_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top(): pass\n");

        let result = collect_evidence(root.to_str().unwrap().to_string(), "".to_string());
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::NotDirectory
        );
    }

    #[test]
    fn ev_05_invalid_root_fails() {
        let result = collect_evidence("/does/not/exist".to_string(), "L0".to_string());
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::PathNotFound
        );
    }

    // ─── warning 不阻断 ───────────────────────────────────────────

    /// E2E: 使用 /tmp 临时项目验证 open → select → collect 完整链路
    #[test]
    fn ev_08_e2e_temp_project_pipeline() {
        let test_root = "/tmp/fpga-flow-mind-phase2-acceptance-Utcnmb";
        if !std::path::Path::new(test_root).exists() {
            eprintln!("跳过 E2E 测试：临时项目不存在");
            return;
        }

        // select_stage L0
        let l0_ctx = super::super::select_stage::resolve_stage_context(test_root, "L0");
        assert!(l0_ctx.success, "resolve L0 应成功");
        assert_eq!(l0_ctx.data.as_ref().unwrap().files.len(), 2);

        // collect_evidence L0 — 应找到 process_signal, normalize, SignalProcessor, interface_handler
        let ev_l0 = collect_evidence(test_root.to_string(), "L0".to_string());
        assert!(ev_l0.success);
        let col = ev_l0.data.unwrap();
        assert!(col.evidence_items.len() >= 3, "L0 应至少 3 项，实际 {}", col.evidence_items.len());
        assert!(col.evidence_items[0].evidence_id.starts_with("EV-L0-"));
        assert!(!col.index_by_path.is_empty());
        assert!(!col.index_by_kind.is_empty());

        // collect_evidence RTL — 应找到 top + alu
        let ev_rtl = collect_evidence(test_root.to_string(), "RTL".to_string());
        assert!(ev_rtl.success);
        let rtl_col = ev_rtl.data.unwrap();
        assert!(rtl_col.evidence_items.len() >= 2, "RTL 应至少 2 项");

        // collect_evidence L1 — 应找到 DataModel
        let ev_l1 = collect_evidence(test_root.to_string(), "L1".to_string());
        assert!(ev_l1.success);
        assert!(ev_l1.data.unwrap().evidence_items.len() >= 1);

        // 空阶段 L3
        let ev_l3 = collect_evidence(test_root.to_string(), "L3".to_string());
        assert!(ev_l3.success, "空阶段应 success=true");
        assert!(ev_l3.data.is_none(), "空阶段无 data");
        assert!(ev_l3.error.is_some(), "空阶段有 error");

        // 只读验证：文件内容不变
        let content = std::fs::read_to_string(format!("{}/L0/top.py", test_root)).unwrap();
        assert!(content.contains("process_signal"));
    }

    #[test]
    fn ev_06_warnings_no_failure() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();

        // 正常 Python 文件
        touch(root, "L0/code.py", b"def process():\n    pass\n");

        // 超长函数体触发 summary 截断 warning
        let long_var = "x".repeat(600);
        let long_content = format!("def long_fn():\n    var = \"{}\"\n    pass\n", long_var);
        touch(root, "L0/long.py", long_content.as_bytes());

        let result = collect_evidence(root.to_str().unwrap().to_string(), "L0".to_string());
        assert!(result.success, "有 warning 时仍应成功");

        let col = result.data.unwrap();
        assert!(
            !col.evidence_items.is_empty(),
            "应有证据项"
        );
        assert!(
            col.warnings
                .iter()
                .any(|w| w.error_code == ErrorCode::SourceExcerptTruncated),
            "应有 source_excerpt_truncated warning"
        );
        assert!(
            col.evidence_items.len() >= 2,
            "应有至少 2 个证据项（process + long_fn），实际 {}",
            col.evidence_items.len()
        );
    }
}
