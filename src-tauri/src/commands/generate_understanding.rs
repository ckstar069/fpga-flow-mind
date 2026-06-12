/// `generate_understanding` Tauri command
///
/// 完整链路：resolve_stage_context → EvidenceCollector → UnderstandingGenerator → ImplementationUnderstanding
///
/// 返回策略：
/// - resolve 失败 → 透传 CommandResult（success=false）
/// - stage_empty → success=true, degraded understanding（is_degraded=true）
/// - 空 evidence → success=true, MockProvider 生成含 unknowns/gaps 的理解
/// - provider 未配置 → success=true, degraded understanding
/// - 验证失败 → success=false, UnderstandingGenerationFailed error
/// - 正常 → success=true, data=Some(ImplementationUnderstanding)

use crate::evidence::collector::EvidenceCollector;
use crate::models::enums::ErrorCode;
use crate::models::error::{CommandError, CommandResult};
use crate::understanding::generator::{
    GeneratorError, MockProvider, UnderstandingGenerator,
};
use crate::understanding::models::ImplementationUnderstanding;

use super::select_stage::resolve_stage_context;

#[tauri::command]
pub fn generate_understanding(
    root_path: String,
    stage_id: String,
) -> CommandResult<ImplementationUnderstanding> {
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

    // 2. 空阶段 → 仍然生成，但用空 evidence collection
    // （MockProvider 会生成含 unknowns/gaps 的理解）

    // 3. 收集 evidence（复用 Phase 2 逻辑）
    let mut collector = EvidenceCollector::new(&stage_id);
    let collection = collector.collect_from_stage_context(&context);

    // 4. 创建 MockProvider + Generator，生成理解
    let generator = UnderstandingGenerator::new(Box::new(MockProvider));
    let result = generator.generate(&collection);

    match result {
        Ok(understanding) => CommandResult {
            success: true,
            data: Some(understanding),
            error: None,
            warnings: Vec::new(),
        },
        Err(GeneratorError::ProviderError(e)) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::UnderstandingGenerationFailed,
                message: format!("Provider 错误: {:?}", e),
                recoverable: true,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        },
        Err(GeneratorError::ValidationFailed(errors)) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::UnderstandingGenerationFailed,
                message: format!(
                    "Schema 验证失败: {}",
                    errors
                        .iter()
                        .map(|e| format!("{:?}", e))
                        .collect::<Vec<_>>()
                        .join("; ")
                ),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        },
        Err(GeneratorError::DeserializationError(e)) => CommandResult {
            success: false,
            data: None,
            error: Some(CommandError {
                error_code: ErrorCode::UnderstandingGenerationFailed,
                message: format!("反序列化失败: {}", e),
                recoverable: false,
                details: None,
                source_path: None,
            }),
            warnings: Vec::new(),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::enums::ErrorCode;

    /// 辅助：创建临时目录并写入文件
    fn touch(root: &std::path::Path, rel: &str, content: &[u8]) {
        let path = root.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, content).unwrap();
    }

    // ─── und_01: 正常场景 — Python 项目端到端 ────────────────────────

    #[test]
    fn und_01_available_stage_generates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top():\n    pass\n");
        touch(root, "L0/helper.py", b"def helper():\n    return 1\n");

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
        );
        assert!(result.success, "可用阶段应成功");
        assert!(result.data.is_some(), "应有 data");
        assert!(result.error.is_none(), "不应有 error: {:?}", result.error);

        let understanding = result.data.unwrap();
        assert_eq!(understanding.stage_id, "L0");
        assert_eq!(understanding.version, "3.0.0");
        assert!(!understanding.claims.is_empty(), "应有至少 1 条 claim");
        assert!(!understanding.generation_meta.is_degraded);
    }

    // ─── und_02: Verilog 阶段端到端 ──────────────────────────────────

    #[test]
    fn und_02_verilog_stage_generates() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(
            root,
            "rtl/top.v",
            b"module top(\n    input clk\n);\nendmodule\n",
        );

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "RTL".to_string(),
        );
        assert!(result.success, "Verilog 阶段应成功");
        assert!(result.data.is_some());

        let understanding = result.data.unwrap();
        assert_eq!(understanding.stage_id, "RTL");
        assert!(!understanding.claims.is_empty());
    }

    // ─── und_03: 空阶段 → MockProvider 生成含 unknowns/gaps ─────────

    #[test]
    fn und_03_empty_stage_generates_with_gaps() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        std::fs::create_dir(root.join("L0")).unwrap();

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
        );

        // 空阶段 → 仍然生成 understanding（MockProvider 对空 collection 生成 unknowns/gaps）
        assert!(result.success, "空阶段应返回 success=true");
        assert!(
            result.data.is_some(),
            "应有 data（含 unknowns/gaps）"
        );

        let understanding = result.data.unwrap();
        assert_eq!(understanding.stage_id, "L0");
        assert!(understanding.claims.is_empty(), "空阶段不应有 claims");
        assert!(
            !understanding.unknowns.is_empty() || !understanding.evidence_gaps.is_empty(),
            "空阶段应有 unknowns 或 evidence_gaps"
        );
    }

    // ─── und_04: 无效路径失败 ────────────────────────────────────────

    #[test]
    fn und_04_invalid_root_fails() {
        let result = generate_understanding("/does/not/exist".to_string(), "L0".to_string());
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::PathNotFound
        );
    }

    // ─── und_05: 不存在的阶段失败 ────────────────────────────────────

    #[test]
    fn und_05_nonexistent_stage_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top(): pass\n");

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L9".to_string(),
        );
        assert!(!result.success);
        assert_eq!(
            result.error.as_ref().unwrap().error_code,
            ErrorCode::NotDirectory
        );
    }

    // ─── und_06: 目标项目只读验证 ────────────────────────────────────

    #[test]
    fn und_06_target_project_readonly() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def process():\n    pass\n");

        let before = std::fs::read_to_string(root.join("L0/top.py")).unwrap();

        let _result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "L0".to_string(),
        );

        let after = std::fs::read_to_string(root.join("L0/top.py")).unwrap();
        assert_eq!(before, after, "generate_understanding 不应修改目标文件");
    }

    // ─── und_07: 空 stage_id 失败 ────────────────────────────────────

    #[test]
    fn und_07_empty_stage_id_fails() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        touch(root, "L0/top.py", b"def top(): pass\n");

        let result = generate_understanding(
            root.to_str().unwrap().to_string(),
            "".to_string(),
        );
        assert!(!result.success);
    }

    // ─── und_08: E2E 多阶段完整 pipeline ────────────────────────────

    #[test]
    fn und_08_e2e_multi_stage_pipeline() {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path();
        let root_str = root.to_str().unwrap();

        // 构建夹具
        touch(
            root,
            "L0/top.py",
            br#""""Top-level signal processing module."""
def process_signal(data, sample_rate):
    normalized = normalize(data)
    return normalized

def normalize(data):
    max_val = max(abs(data))
    return [x / max_val for x in data]
"#,
        );
        touch(
            root,
            "RTL/top.v",
            b"module top(\n    input wire clk\n);\nendmodule\n",
        );

        // collect 前快照
        let l0_before = std::fs::read_to_string(root.join("L0/top.py")).unwrap();
        let rtl_before = std::fs::read_to_string(root.join("RTL/top.v")).unwrap();

        // L0 生成理解
        let l0_result =
            generate_understanding(root_str.to_string(), "L0".to_string());
        assert!(l0_result.success, "L0 应成功");
        let l0_understanding = l0_result.data.unwrap();
        assert_eq!(l0_understanding.stage_id, "L0");
        assert!(!l0_understanding.claims.is_empty());
        assert!(
            l0_understanding.stats.total_claims > 0,
            "L0 应有统计 claims"
        );

        // RTL 生成理解
        let rtl_result =
            generate_understanding(root_str.to_string(), "RTL".to_string());
        assert!(rtl_result.success, "RTL 应成功");
        let rtl_understanding = rtl_result.data.unwrap();
        assert_eq!(rtl_understanding.stage_id, "RTL");
        assert!(!rtl_understanding.claims.is_empty());

        // 只读验证
        assert_eq!(
            l0_before,
            std::fs::read_to_string(root.join("L0/top.py")).unwrap(),
            "L0 文件不应被修改"
        );
        assert_eq!(
            rtl_before,
            std::fs::read_to_string(root.join("RTL/top.v")).unwrap(),
            "RTL 文件不应被修改"
        );
    }
}
