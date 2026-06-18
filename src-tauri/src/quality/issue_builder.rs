//! 确定性 `QualityIssue` 构造辅助模块（Phase 7 Batch A）。
//!
//! 将 `make_issue`、`make_guardrail` 等无状态 helper 从 `reporter` 中剥离，
//! 供 Batch B/C/D/E 的 evaluator 模块复用，避免循环依赖。

use std::collections::HashSet;

use crate::evidence::models::{EvidenceItem, LineRange};
use crate::models::enums::{Language, SourceKind};
use crate::quality::models::{
    ArtifactKind, DetectionMethod, IssueStatus, QualityIssue, QualityIssueKind, QualityIssuePolarity,
    QualitySeverity,
};

/// 构造一条 `QualityIssue`，`issue_id` 留空（由外部分配器统一填充），
/// `detected_by` 固定为 `Automated`，`status` 为 `Open`，`polarity` 取自 `kind.default_polarity()`。
#[allow(clippy::too_many_arguments)]
pub fn make_issue(
    sample_id: &str,
    stage_id: &str,
    artifact_kind: ArtifactKind,
    kind: QualityIssueKind,
    severity: QualitySeverity,
    evidence_id: Option<&str>,
    claim_id: Option<&str>,
    node_id: Option<&str>,
    source_path: Option<&str>,
    line_range: Option<LineRange>,
    description: &str,
) -> QualityIssue {
    QualityIssue {
        issue_id: String::new(),
        sample_id: sample_id.to_string(),
        stage_id: stage_id.to_string(),
        artifact_kind,
        kind,
        polarity: kind.default_polarity(),
        severity,
        evidence_id: evidence_id.map(|s| s.to_string()),
        claim_id: claim_id.map(|s| s.to_string()),
        node_id: node_id.map(|s| s.to_string()),
        source_path: source_path.map(|s| s.to_string()),
        line_range,
        description: description.to_string(),
        detected_by: DetectionMethod::Automated,
        status: IssueStatus::Open,
    }
}

/// 构造一条正向 guardrail 记录（`HallucinatedClaimBlocked`）。
///
/// 内部调用 `make_issue` 并将 `polarity` 覆写为 `PositiveGuardrail`，
/// 用于标记“在证据不足时未伪造引用”的诚实行为。
pub fn make_guardrail(
    sample_id: &str,
    stage_id: &str,
    artifact_kind: ArtifactKind,
    kind: QualityIssueKind,
    claim_id: Option<&str>,
    description: &str,
) -> QualityIssue {
    let mut issue = make_issue(
        sample_id,
        stage_id,
        artifact_kind,
        kind,
        QualitySeverity::Low,
        None,
        claim_id,
        None,
        None,
        None,
        description,
    );
    issue.polarity = QualityIssuePolarity::PositiveGuardrail;
    issue.status = IssueStatus::Open;
    issue
}

/// 检查 `(Option<String> evidence_id, Option<String> claim_id)` 是否能在给定的
/// evidence/claim 集合中解析。
///
/// 完全空的引用（既无 `evidence_id` 也无 `claim_id`）视为不可解析。
pub fn trace_ref_ok(
    evidence_id: &Option<String>,
    claim_id: &Option<String>,
    evidence_set: &HashSet<String>,
    claim_set: &HashSet<String>,
) -> bool {
    let ev_present = evidence_id.as_ref().map(|s| !s.is_empty()).unwrap_or(false);
    let cl_present = claim_id.as_ref().map(|s| !s.is_empty()).unwrap_or(false);

    // 空引用（无 evidence_id 且无 claim_id）无法解析回任何产物。
    if !ev_present && !cl_present {
        return false;
    }

    let mut ok = true;
    if ev_present && !evidence_set.contains(evidence_id.as_ref().unwrap()) {
        ok = false;
    }
    if cl_present && !claim_set.contains(claim_id.as_ref().unwrap()) {
        ok = false;
    }
    ok
}

/// 低价值 Python 符号集合：typing/annotation、decorator/meta、常见模块别名、
/// 配置导入、通用数据词（在 import/return-type 上下文中）。
const PYTHON_TYPING_SYMBOLS: &[&str] = &[
    "annotations", "optional", "list", "dict", "tuple", "union", "any", "callable",
    "typevar", "generic", "protocol", "final", "literal", "classvar", "self",
];
const PYTHON_DECORATOR_SYMBOLS: &[&str] = &[
    "dataclass", "abstractmethod", "property", "staticmethod", "classmethod", "overload",
];
const PYTHON_MODULE_ALIASES: &[&str] = &["np", "numpy", "pd", "pandas"];
const PYTHON_CONFIG_SYMBOLS: &[&str] = &["params", "config", "parameters", "settings"];
const PYTHON_GENERIC_DATA_WORDS: &[&str] = &["data", "data_width", "width", "size", "value", "result"];

/// 判断一个 Python 符号是否为低价值噪声（typing/decorator/import 上下文）。
///
/// 注意：仅跳过 `__` dunder，不跳过单下划线业务函数（如 `_stage_correlation`）。
pub fn is_low_value_python_symbol(sym: &str, summary: &str) -> bool {
    if sym.starts_with("__") && sym.ends_with("__") {
        return true;
    }
    let sym_lower = sym.to_lowercase();
    let sum_lower = summary.to_lowercase();
    if PYTHON_TYPING_SYMBOLS.contains(&sym_lower.as_str())
        || PYTHON_DECORATOR_SYMBOLS.contains(&sym_lower.as_str())
        || PYTHON_MODULE_ALIASES.contains(&sym_lower.as_str())
        || PYTHON_CONFIG_SYMBOLS.contains(&sym_lower.as_str())
    {
        return true;
    }
    // 通用数据词仅在 import / return-type / typing 上下文中视为噪声
    let is_import_or_typing_context = sum_lower.starts_with("import")
        || sum_lower.starts_with("from")
        || sum_lower.contains("->")
        || sum_lower.contains("typing");
    if is_import_or_typing_context && PYTHON_GENERIC_DATA_WORDS.contains(&sym_lower.as_str()) {
        return true;
    }
    false
}

/// 检测 summary / symbol 中是否包含噪声标记。
///
/// - TODO / FIXME / XXX / HACK（大小写不敏感）
/// - 低价值 Python import / typing / decorator 符号证据
pub fn is_noisy(summary: &str, symbol: Option<&str>) -> bool {
    let upper = summary.to_uppercase();
    if upper.contains("TODO") || upper.contains("FIXME") || upper.contains("XXX") || upper.contains("HACK") {
        return true;
    }
    if let Some(sym) = symbol {
        let sum_lower = summary.to_lowercase();
        if is_low_value_python_symbol(sym, &sum_lower)
            && (sum_lower.starts_with("import")
                || sum_lower.starts_with("from")
                || sum_lower.contains("->")
                || sum_lower.contains("typing"))
        {
            return true;
        }
    }
    false
}

/// 检查 `EvidenceItem` 的 `source_kind` 与 `language` 标注是否自洽。
///
/// Batch A 启发式：Python 对应 `PythonStage`，Verilog / SystemVerilog 对应 `Rtl`，
/// Markdown 对应 `Doc`；其余语言视为默认通过。
pub fn is_label_sane(item: &EvidenceItem) -> bool {
    match item.language {
        Language::Python => item.source_kind == SourceKind::PythonStage,
        Language::Verilog | Language::SystemVerilog => item.source_kind == SourceKind::Rtl,
        Language::Markdown => item.source_kind == SourceKind::Doc,
        _ => true,
    }
}

/// 将 `sample_id` 净化为仅含 ASCII 字母数字、`-`、`_` 的字符串，用于生成确定性 report ID。
///
/// 非法字符替换为 `-`；若结果为空则返回 `"sample"`。
pub fn sanitize_scope(sample_id: &str) -> String {
    let s: String = sample_id
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '-' })
        .collect();
    if s.is_empty() {
        "sample".to_string()
    } else {
        s
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::evidence::models::{EvidenceItem, LineRange};
    use crate::models::enums::{Language, SourceKind};
    use std::collections::HashSet;

    #[test]
    fn sanitize_scope_replaces_invalid_chars() {
        assert_eq!(sanitize_scope("a/b.c"), "a-b-c");
        assert_eq!(sanitize_scope(""), "sample");
        assert_eq!(sanitize_scope("rtl_final"), "rtl_final");
    }

    #[test]
    fn is_noisy_detects_markers() {
        assert!(is_noisy("TODO: fix this", None));
        assert!(is_noisy("FIXME urgent", None));
        assert!(is_noisy("xxx hack", None));
        assert!(!is_noisy("normal summary", None));
    }

    #[test]
    fn is_noisy_flags_python_import_noise() {
        assert!(is_noisy("from __future__ import annotations", Some("annotations")));
        assert!(is_noisy("import numpy as np", Some("np")));
        assert!(is_noisy("from config.parameters import PARAMS", Some("PARAMS")));
        assert!(is_noisy("def process(self) -> Optional[dict]", Some("Optional")));
        assert!(!is_noisy("def coarse_sync(rx_signal):", Some("coarse_sync")));
    }

    #[test]
    fn is_low_value_python_symbol_filters_noise() {
        assert!(is_low_value_python_symbol("annotations", "from __future__ import annotations"));
        assert!(is_low_value_python_symbol("dataclass", "@dataclass"));
        assert!(is_low_value_python_symbol("np", "import numpy as np"));
        assert!(is_low_value_python_symbol("PARAMS", "from config.parameters import PARAMS"));
        assert!(!is_low_value_python_symbol("coarse_sync", "def coarse_sync(rx_signal):"));
        assert!(!is_low_value_python_symbol("_stage_correlation", "def _stage_correlation(self):"));
    }

    #[test]
    fn trace_ref_ok_logic() {
        let mut ev = HashSet::new();
        ev.insert("EV-1".to_string());
        let mut cl = HashSet::new();
        cl.insert("CL-1".to_string());

        // 空引用不可解析
        assert!(!trace_ref_ok(&None, &None, &ev, &cl));

        // 有效引用
        assert!(trace_ref_ok(&Some("EV-1".to_string()), &None, &ev, &cl));
        assert!(trace_ref_ok(&None, &Some("CL-1".to_string()), &ev, &cl));

        // 无效引用
        assert!(!trace_ref_ok(&Some("EV-2".to_string()), &None, &ev, &cl));
        assert!(!trace_ref_ok(&None, &Some("CL-2".to_string()), &ev, &cl));
    }

    #[test]
    fn make_issue_default_polarity_and_status() {
        let issue = make_issue(
            "S1", "L0", ArtifactKind::Evidence, QualityIssueKind::MissingEvidence,
            QualitySeverity::High, None, None, None, None, None, "test",
        );
        assert_eq!(issue.issue_id, "");
        assert_eq!(issue.detected_by, DetectionMethod::Automated);
        assert_eq!(issue.status, IssueStatus::Open);
        assert_eq!(issue.polarity, QualityIssuePolarity::Problem);
    }

    #[test]
    fn make_guardrail_positive_polarity() {
        let grd = make_guardrail(
            "S1", "L0", ArtifactKind::Understanding, QualityIssueKind::HallucinatedClaimBlocked,
            Some("CL-1"), "guardrail test",
        );
        assert_eq!(grd.polarity, QualityIssuePolarity::PositiveGuardrail);
        assert_eq!(grd.severity, QualitySeverity::Low);
    }

    #[test]
    fn is_label_sane_consistency() {
        let item = |lang, kind| EvidenceItem {
            evidence_id: "E".to_string(),
            source_path: "/p/a.py".to_string(),
            language: lang,
            source_kind: kind,
            line_range: LineRange { start: 1, end: 2 },
            symbol: None,
            summary: "ok".to_string(),
            strength: crate::evidence::models::EvidenceStrength::Direct,
        };
        assert!(is_label_sane(&item(Language::Python, SourceKind::PythonStage)));
        assert!(!is_label_sane(&item(Language::Python, SourceKind::Rtl)));
        assert!(is_label_sane(&item(Language::Verilog, SourceKind::Rtl)));
        assert!(is_label_sane(&item(Language::Markdown, SourceKind::Doc)));
    }
}
