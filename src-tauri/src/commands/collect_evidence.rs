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
        // P1: module + port extraction → ≥ 2 items
        assert!(col.evidence_items.len() >= 2, "应有至少 2 项证据, 实际 {}", col.evidence_items.len());
        assert!(col.evidence_items.iter().any(|i| i.evidence_id == "EV-RTL-000001"), "ID 应从 EV-RTL-000001 开始");
        let rtl_count = col.stats.items_by_kind.get("rtl").copied().unwrap_or(0);
        assert!(rtl_count >= 2, "rtl 类应有至少 2 项, 实际 {}", rtl_count);
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

    /// E2E: 测试内自建临时项目验证 resolve → collect 完整链路
    #[test]
    fn ev_08_e2e_temp_project_pipeline() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();

        // ─── 构建夹具 ────────────────────────────────────────
        // L0/top.py: 2 个 def + 1 个 class
        touch(root, "L0/top.py", br#"""Top-level signal processing module."""
def process_signal(data, sample_rate):
    normalized = normalize(data)
    return normalized

def normalize(data):
    max_val = max(abs(data))
    return [x / max_val for x in data]

class SignalProcessor:
    def __init__(self, config):
        self.config = config
"#);
        // L0/top_interface.py: 1 个 def
        touch(root, "L0/top_interface.py", b"def interface_handler(cmd):\n    return True\n");
        // L1/model.py: 1 个 class
        touch(root, "L1/model.py", b"class DataModel:\n    def __init__(self):\n        self.data = []\n");
        // RTL/top.v: 1 个 module
        touch(root, "RTL/top.v", b"module top(\n    input wire clk\n);\nendmodule\n");
        // RTL/alu.v: 1 个 module
        touch(root, "RTL/alu.v", b"module alu(\n    input wire [7:0] a\n);\nendmodule\n");
        // L3/ 空目录
        std::fs::create_dir_all(root.join("L3")).unwrap();

        // ─── 收集前快照（用于只读验证） ──────────────────────
        let l0_py_before = std::fs::read_to_string(root.join("L0/top.py")).unwrap();

        // ─── resolve + collect L0 ────────────────────────────
        let l0_ctx = super::super::select_stage::resolve_stage_context(root_str, "L0");
        assert!(l0_ctx.success, "resolve L0 应成功");
        assert_eq!(l0_ctx.data.as_ref().unwrap().files.len(), 2);

        let ev_l0 = collect_evidence(root_str.to_string(), "L0".to_string());
        assert!(ev_l0.success, "collect L0 应成功");
        let col = ev_l0.data.unwrap();
        assert!(col.evidence_items.len() >= 3, "L0 应至少 3 项，实际 {}", col.evidence_items.len());
        assert!(col.evidence_items[0].evidence_id.starts_with("EV-L0-"));
        assert!(!col.index_by_path.is_empty());
        assert!(!col.index_by_kind.is_empty());

        // ─── collect RTL — 应找到 top + alu ──────────────────
        let ev_rtl = collect_evidence(root_str.to_string(), "RTL".to_string());
        assert!(ev_rtl.success, "collect RTL 应成功");
        let rtl_col = ev_rtl.data.unwrap();
        assert!(rtl_col.evidence_items.len() >= 2, "RTL 应至少 2 项");

        // ─── collect L1 — 应找到 DataModel ──────────────────
        let ev_l1 = collect_evidence(root_str.to_string(), "L1".to_string());
        assert!(ev_l1.success, "collect L1 应成功");
        assert!(ev_l1.data.unwrap().evidence_items.len() >= 1, "L1 应至少 1 项");

        // ─── 空阶段 L3 ──────────────────────────────────────
        let ev_l3 = collect_evidence(root_str.to_string(), "L3".to_string());
        assert!(ev_l3.success, "空阶段应 success=true");
        assert!(ev_l3.data.is_none(), "空阶段无 data");
        assert!(ev_l3.error.is_some(), "空阶段有 error");

        // ─── 只读验证：文件内容不变 ─────────────────────────
        let l0_py_after = std::fs::read_to_string(root.join("L0/top.py")).unwrap();
        assert_eq!(l0_py_before, l0_py_after, "目标文件不应被修改");
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
