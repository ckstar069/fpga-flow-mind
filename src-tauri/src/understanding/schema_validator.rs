use std::collections::HashSet;

// ─── 合法枚举值 ────────────────────────────────────────────────────────

/// 合法 ClaimConfidence 枚举值（snake_case）
const VALID_CONFIDENCES: &[&str] = &[
    "confirmed",
    "supported",
    "inferred",
    "unknown",
    "conflicting",
];

/// 合法 ClaimCategory 枚举值（snake_case）
const VALID_CATEGORIES: &[&str] = &[
    "module_structure",
    "signal_definition",
    "interface_description",
    "data_processing",
    "configuration",
    "documentation",
    "test_coverage",
    "other",
];

/// 需要 ≥1 evidence_ref 的 confidence（非 unknown）
const CONFIDENCE_REQUIRES_REFS: &[&str] = &[
    "confirmed",
    "supported",
    "inferred",
    "conflicting",
];

// ─── Error/Warning 类型 ───────────────────────────────────────────────

/// 验证错误 — 阻断性，导致 is_valid = false
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// JSON 结构验证失败（字段缺失、类型错误、枚举非法等）
    SchemaViolation { path: String, message: String },
    /// evidence_id 不存在于输入 EvidenceCollection 中（hallucination）
    UnknownEvidenceId {
        evidence_id: String,
        location: String,
    },
    /// claim 缺少必要的 evidence_refs
    ClaimWithoutEvidence { claim_id: String },
    /// unknown 项绑定了不存在的 evidence_id
    UnknownWithFakeEvidence {
        unknown_id: String,
        evidence_id: String,
    },
    /// 重复的 claim_id
    DuplicateClaimId { claim_id: String },
}

/// 验证警告 — 非阻断性，不影响 is_valid
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationWarning {
    /// unknown 项数量过多（超过 claim 数量）
    TooManyUnknowns { count: usize, claim_count: usize },
    /// evidence gap 数量过多
    TooManyGaps { count: usize },
}

/// 验证结果
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<ValidationWarning>,
}

/// SchemaValidator — 对 generator 输出进行三层验证
///
/// 验证流程：
/// 1. 结构验证：12 个必填字段、类型、枚举值、evidence_refs 元素结构
/// 2. evidence_id existence check（hallucination guard）
/// 3. 业务规则检查：confidence 与 refs 关系、重复 claim_id、阈值警告
pub struct SchemaValidator;

impl SchemaValidator {
    /// 验证 generator 输出的 JSON
    ///
    /// - `output`: generator 输出的 JSON（ImplementationUnderstanding 的 JSON 表示）
    /// - `known_evidence_ids`: 输入 EvidenceCollection 中所有 evidence_id 集合
    pub fn validate(
        output: &serde_json::Value,
        known_evidence_ids: &HashSet<String>,
    ) -> ValidationResult {
        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // 第 1 层：结构验证
        Self::check_structure(output, &mut errors);

        // 仅当结构合法时才继续后续检查
        if errors.is_empty() {
            // 第 2 层：evidence_id existence check（hallucination guard）
            Self::check_evidence_ids(output, known_evidence_ids, &mut errors);

            // 第 3 层：业务规则检查
            Self::check_business_rules(output, &mut errors, &mut warnings);
        }

        let is_valid = errors.is_empty();
        ValidationResult {
            is_valid,
            errors,
            warnings,
        }
    }

    // ─── 第 1 层：结构验证 ─────────────────────────────────────────

    fn check_structure(output: &serde_json::Value, errors: &mut Vec<ValidationError>) {
        // 必须是 object
        if !output.is_object() {
            errors.push(ValidationError::SchemaViolation {
                path: "/".to_string(),
                message: "输出必须是 JSON 对象".to_string(),
            });
            return;
        }

        // 1. stage_id: 非空字符串
        match output.get("stage_id").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => {}
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/stage_id".to_string(),
                    message: "stage_id 不能为空字符串".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/stage_id".to_string(),
                    message: "缺少 stage_id 字段".to_string(),
                });
            }
        }

        // 2. version: 字符串
        if output.get("version").and_then(|v| v.as_str()).is_none() {
            errors.push(ValidationError::SchemaViolation {
                path: "/version".to_string(),
                message: "缺少 version 字段".to_string(),
            });
        }

        // 3. summary: 对象 + short/detailed 非空字符串
        match output.get("summary") {
            Some(s) if s.is_object() => {
                match s.get("short").and_then(|v| v.as_str()) {
                    Some(v) if !v.is_empty() => {}
                    Some(_) => {
                        errors.push(ValidationError::SchemaViolation {
                            path: "/summary/short".to_string(),
                            message: "summary.short 不能为空字符串".to_string(),
                        });
                    }
                    None => {
                        errors.push(ValidationError::SchemaViolation {
                            path: "/summary/short".to_string(),
                            message: "缺少 summary.short 字段".to_string(),
                        });
                    }
                }
                match s.get("detailed").and_then(|v| v.as_str()) {
                    Some(v) if !v.is_empty() => {}
                    Some(_) => {
                        errors.push(ValidationError::SchemaViolation {
                            path: "/summary/detailed".to_string(),
                            message: "summary.detailed 不能为空字符串".to_string(),
                        });
                    }
                    None => {
                        errors.push(ValidationError::SchemaViolation {
                            path: "/summary/detailed".to_string(),
                            message: "缺少 summary.detailed 字段".to_string(),
                        });
                    }
                }
            }
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/summary".to_string(),
                    message: "summary 必须是对象".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/summary".to_string(),
                    message: "缺少 summary 字段".to_string(),
                });
            }
        }

        // 4. claims: 数组 + 逐项验证
        match output.get("claims") {
            Some(v) if v.is_array() => {
                if let Some(claims) = v.as_array() {
                    for (i, claim) in claims.iter().enumerate() {
                        let path_prefix = format!("/claims/{}", i);
                        Self::check_claim_structure(claim, &path_prefix, errors);
                    }
                }
            }
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/claims".to_string(),
                    message: "claims 必须是数组".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/claims".to_string(),
                    message: "缺少 claims 字段".to_string(),
                });
            }
        }

        // 5-8. 摘要数组 + evidence_refs 元素结构检查
        for field in &[
            "module_summaries",
            "signal_summaries",
            "interface_summaries",
            "processing_steps",
        ] {
            match output.get(*field) {
                Some(v) if v.is_array() => {
                    if let Some(arr) = v.as_array() {
                        for (i, item) in arr.iter().enumerate() {
                            let refs_path = format!("/{}/{}/evidence_refs", field, i);
                            Self::check_evidence_ref_elements(
                                item.get("evidence_refs"),
                                &refs_path,
                                errors,
                            );
                        }
                    }
                }
                Some(_) => {
                    errors.push(ValidationError::SchemaViolation {
                        path: format!("/{}", field),
                        message: format!("{} 必须是数组", field),
                    });
                }
                None => {
                    errors.push(ValidationError::SchemaViolation {
                        path: format!("/{}", field),
                        message: format!("缺少 {} 字段", field),
                    });
                }
            }
        }

        // 9. unknowns: 数组 + related_evidence_refs 元素结构检查
        match output.get("unknowns") {
            Some(v) if v.is_array() => {
                if let Some(arr) = v.as_array() {
                    for (i, item) in arr.iter().enumerate() {
                        let refs_path = format!("/unknowns/{}/related_evidence_refs", i);
                        Self::check_evidence_ref_elements(
                            item.get("related_evidence_refs"),
                            &refs_path,
                            errors,
                        );
                    }
                }
            }
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/unknowns".to_string(),
                    message: "unknowns 必须是数组".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/unknowns".to_string(),
                    message: "缺少 unknowns 字段".to_string(),
                });
            }
        }

        // 10. evidence_gaps: 数组 + related_evidence_refs 元素结构检查
        match output.get("evidence_gaps") {
            Some(v) if v.is_array() => {
                if let Some(arr) = v.as_array() {
                    for (i, item) in arr.iter().enumerate() {
                        let refs_path = format!("/evidence_gaps/{}/related_evidence_refs", i);
                        Self::check_evidence_ref_elements(
                            item.get("related_evidence_refs"),
                            &refs_path,
                            errors,
                        );
                    }
                }
            }
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/evidence_gaps".to_string(),
                    message: "evidence_gaps 必须是数组".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/evidence_gaps".to_string(),
                    message: "缺少 evidence_gaps 字段".to_string(),
                });
            }
        }

        // 11. generation_meta: 对象
        match output.get("generation_meta") {
            Some(v) if v.is_object() => {}
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/generation_meta".to_string(),
                    message: "generation_meta 必须是对象".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/generation_meta".to_string(),
                    message: "缺少 generation_meta 字段".to_string(),
                });
            }
        }

        // 12. stats: 对象
        match output.get("stats") {
            Some(v) if v.is_object() => {}
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/stats".to_string(),
                    message: "stats 必须是对象".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/stats".to_string(),
                    message: "缺少 stats 字段".to_string(),
                });
            }
        }
    }

    /// 检查单个 claim 的结构（含枚举值验证和 evidence_refs 元素结构）
    fn check_claim_structure(
        claim: &serde_json::Value,
        path_prefix: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if !claim.is_object() {
            errors.push(ValidationError::SchemaViolation {
                path: path_prefix.to_string(),
                message: "claim 必须是对象".to_string(),
            });
            return;
        }

        // claim_id: 字符串
        if claim.get("claim_id").and_then(|v| v.as_str()).is_none() {
            errors.push(ValidationError::SchemaViolation {
                path: format!("{}/claim_id", path_prefix),
                message: "claim_id 必须是字符串".to_string(),
            });
        }

        // category: 合法枚举值
        match claim.get("category").and_then(|v| v.as_str()) {
            Some(s) if VALID_CATEGORIES.contains(&s) => {}
            Some(s) => {
                errors.push(ValidationError::SchemaViolation {
                    path: format!("{}/category", path_prefix),
                    message: format!("非法 category 值: {}", s),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: format!("{}/category", path_prefix),
                    message: "缺少 category 字段".to_string(),
                });
            }
        }

        // description: 字符串
        if claim.get("description").and_then(|v| v.as_str()).is_none() {
            errors.push(ValidationError::SchemaViolation {
                path: format!("{}/description", path_prefix),
                message: "description 必须是字符串".to_string(),
            });
        }

        // confidence: 合法枚举值
        match claim.get("confidence").and_then(|v| v.as_str()) {
            Some(s) if VALID_CONFIDENCES.contains(&s) => {}
            Some(s) => {
                errors.push(ValidationError::SchemaViolation {
                    path: format!("{}/confidence", path_prefix),
                    message: format!("非法 confidence 值: {}", s),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: format!("{}/confidence", path_prefix),
                    message: "缺少 confidence 字段".to_string(),
                });
            }
        }

        // evidence_refs: 数组 + 元素结构检查
        match claim.get("evidence_refs") {
            Some(v) if v.is_array() => {
                let refs_path = format!("{}/evidence_refs", path_prefix);
                Self::check_evidence_ref_elements(Some(v), &refs_path, errors);
            }
            Some(_) => {
                errors.push(ValidationError::SchemaViolation {
                    path: format!("{}/evidence_refs", path_prefix),
                    message: "evidence_refs 必须是数组".to_string(),
                });
            }
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: format!("{}/evidence_refs", path_prefix),
                    message: "缺少 evidence_refs 字段".to_string(),
                });
            }
        }

        // has_evidence_gap: 布尔值
        match claim.get("has_evidence_gap").and_then(|v| v.as_bool()) {
            Some(_) => {}
            None => {
                errors.push(ValidationError::SchemaViolation {
                    path: format!("{}/has_evidence_gap", path_prefix),
                    message: "has_evidence_gap 必须是布尔值".to_string(),
                });
            }
        }
    }

    /// 检查 evidence_refs 数组中每个元素的结构
    ///
    /// 每个元素必须是对象且包含非空 evidence_id 字符串。
    fn check_evidence_ref_elements(
        refs_value: Option<&serde_json::Value>,
        path_prefix: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if let Some(refs) = refs_value.and_then(|v| v.as_array()) {
            for (j, r) in refs.iter().enumerate() {
                let ref_path = format!("{}/{}", path_prefix, j);
                if !r.is_object() {
                    errors.push(ValidationError::SchemaViolation {
                        path: ref_path,
                        message: "evidence_ref 必须是对象".to_string(),
                    });
                    continue;
                }
                match r.get("evidence_id").and_then(|v| v.as_str()) {
                    Some(s) if !s.is_empty() => { /* OK */ }
                    Some(_) => {
                        errors.push(ValidationError::SchemaViolation {
                            path: format!("{}/evidence_id", ref_path),
                            message: "evidence_id 不能为空字符串".to_string(),
                        });
                    }
                    None => {
                        errors.push(ValidationError::SchemaViolation {
                            path: format!("{}/evidence_id", ref_path),
                            message: "缺少 evidence_id 字段".to_string(),
                        });
                    }
                }
            }
        }
    }

    // ─── 第 2 层：evidence_id existence check ──────────────────────

    fn check_evidence_ids(
        output: &serde_json::Value,
        known_evidence_ids: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        // claims[].evidence_refs[].evidence_id
        if let Some(claims) = output.get("claims").and_then(|v| v.as_array()) {
            for claim in claims {
                if let Some(claim_id) = claim.get("claim_id").and_then(|v| v.as_str()) {
                    Self::check_refs_against_known(
                        claim.get("evidence_refs"),
                        known_evidence_ids,
                        &format!("claim:{}", claim_id),
                        errors,
                    );
                }
            }
        }

        // module_summaries[].evidence_refs
        if let Some(arr) = output.get("module_summaries").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                Self::check_refs_against_known(
                    item.get("evidence_refs"),
                    known_evidence_ids,
                    &format!("module_summaries:{}", i),
                    errors,
                );
            }
        }

        // signal_summaries[].evidence_refs
        if let Some(arr) = output.get("signal_summaries").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                Self::check_refs_against_known(
                    item.get("evidence_refs"),
                    known_evidence_ids,
                    &format!("signal_summaries:{}", i),
                    errors,
                );
            }
        }

        // interface_summaries[].evidence_refs
        if let Some(arr) = output.get("interface_summaries").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                Self::check_refs_against_known(
                    item.get("evidence_refs"),
                    known_evidence_ids,
                    &format!("interface_summaries:{}", i),
                    errors,
                );
            }
        }

        // processing_steps[].evidence_refs
        if let Some(arr) = output.get("processing_steps").and_then(|v| v.as_array()) {
            for (i, item) in arr.iter().enumerate() {
                Self::check_refs_against_known(
                    item.get("evidence_refs"),
                    known_evidence_ids,
                    &format!("processing_steps:{}", i),
                    errors,
                );
            }
        }

        // unknowns[].related_evidence_refs
        if let Some(unknowns) = output.get("unknowns").and_then(|v| v.as_array()) {
            for unknown in unknowns {
                if let Some(unknown_id) = unknown.get("unknown_id").and_then(|v| v.as_str()) {
                    Self::check_refs_against_known(
                        unknown.get("related_evidence_refs"),
                        known_evidence_ids,
                        &format!("unknown:{}", unknown_id),
                        errors,
                    );
                }
            }
        }

        // evidence_gaps[].related_evidence_refs
        if let Some(gaps) = output.get("evidence_gaps").and_then(|v| v.as_array()) {
            for gap in gaps {
                if let Some(gap_id) = gap.get("gap_id").and_then(|v| v.as_str()) {
                    Self::check_refs_against_known(
                        gap.get("related_evidence_refs"),
                        known_evidence_ids,
                        &format!("evidence_gap:{}", gap_id),
                        errors,
                    );
                }
            }
        }
    }

    /// 检查一组 evidence_refs 中的 evidence_id 是否都在 known_ids 中
    fn check_refs_against_known(
        refs_value: Option<&serde_json::Value>,
        known_ids: &HashSet<String>,
        location: &str,
        errors: &mut Vec<ValidationError>,
    ) {
        if let Some(refs) = refs_value.and_then(|v| v.as_array()) {
            for r in refs {
                if let Some(eid) = r.get("evidence_id").and_then(|v| v.as_str()) {
                    if !eid.is_empty() && !known_ids.contains(eid) {
                        errors.push(ValidationError::UnknownEvidenceId {
                            evidence_id: eid.to_string(),
                            location: location.to_string(),
                        });
                    }
                }
            }
        }
    }

    // ─── 第 3 层：业务规则检查 ─────────────────────────────────────

    fn check_business_rules(
        output: &serde_json::Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // 获取 top-level evidence_gaps 是否非空
        let top_gaps_non_empty = output
            .get("evidence_gaps")
            .and_then(|v| v.as_array())
            .map(|a| !a.is_empty())
            .unwrap_or(false);

        if let Some(claims) = output.get("claims").and_then(|v| v.as_array()) {
            // 1a. 重复 claim_id 检测
            let mut seen_claim_ids = HashSet::new();
            for claim in claims {
                if let Some(id) = claim.get("claim_id").and_then(|v| v.as_str()) {
                    if !seen_claim_ids.insert(id.to_string()) {
                        errors.push(ValidationError::DuplicateClaimId {
                            claim_id: id.to_string(),
                        });
                    }
                }
            }

            // 1b. confidence-specific evidence requirements
            for claim in claims {
                let claim_id = claim
                    .get("claim_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let confidence = claim
                    .get("confidence")
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let refs_empty = claim
                    .get("evidence_refs")
                    .and_then(|v| v.as_array())
                    .map(|a| a.is_empty())
                    .unwrap_or(true);
                let has_gap = claim
                    .get("has_evidence_gap")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if refs_empty {
                    if CONFIDENCE_REQUIRES_REFS.contains(&confidence) {
                        // confirmed/supported/inferred/conflicting: 必须有 refs
                        errors.push(ValidationError::ClaimWithoutEvidence {
                            claim_id: claim_id.clone(),
                        });
                    } else if confidence == "unknown" {
                        // unknown: 仅当 has_gap=true 且 top-level gaps 非空时允许
                        if !has_gap || !top_gaps_non_empty {
                            errors.push(ValidationError::ClaimWithoutEvidence {
                                claim_id: claim_id.clone(),
                            });
                        }
                    }
                }
            }
        }

        // 2. unknown 数量 > claims 数量 → 警告
        let claim_count = output
            .get("claims")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);
        let unknown_count = output
            .get("unknowns")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        if unknown_count > claim_count && claim_count > 0 {
            warnings.push(ValidationWarning::TooManyUnknowns {
                count: unknown_count,
                claim_count,
            });
        }

        // 3. gaps 数量 > 10 → 警告
        let gap_count = output
            .get("evidence_gaps")
            .and_then(|v| v.as_array())
            .map(|a| a.len())
            .unwrap_or(0);

        if gap_count > 10 {
            warnings.push(ValidationWarning::TooManyGaps { count: gap_count });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_known_ids(ids: &[&str]) -> HashSet<String> {
        ids.iter().map(|s| s.to_string()).collect()
    }

    /// 构建合法的完整 ImplementationUnderstanding JSON
    fn make_valid_output() -> serde_json::Value {
        serde_json::json!({
            "stage_id": "L0",
            "version": "3.0.0",
            "summary": {
                "short": "L0 参考模型",
                "detailed": "实现了 OFDM 参考模型"
            },
            "claims": [
                {
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "定义了调制器",
                    "confidence": "confirmed",
                    "evidence_refs": [
                        {"evidence_id": "EV-L0-000001"}
                    ],
                    "has_evidence_gap": false
                }
            ],
            "module_summaries": [],
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": [],
            "evidence_gaps": [],
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": 1,
                "generation_time_ms": 100,
                "is_degraded": false
            },
            "stats": {
                "total_claims": 1,
                "claims_by_confidence": {"confirmed": 1},
                "claims_by_category": {"module_structure": 1},
                "module_count": 0,
                "signal_count": 0,
                "interface_count": 0,
                "processing_step_count": 0,
                "unknown_count": 0,
                "evidence_gap_count": 0
            }
        })
    }

    // ─── val_01 ~ val_08: 基础测试 ──────────────────────────────────

    #[test]
    fn val_01_valid_output_passes() {
        let output = make_valid_output();
        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(
            result.is_valid,
            "valid output should pass: {:?}",
            result.errors
        );
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn val_02_missing_stage_id_fails() {
        let mut output = make_valid_output();
        output.as_object_mut().unwrap().remove("stage_id");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, .. } if path == "/stage_id"
                )),
            "expected SchemaViolation for /stage_id, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_03_all_evidence_ids_exist() {
        let output = serde_json::json!({
            "stage_id": "L0",
            "version": "3.0.0",
            "summary": {"short": "test", "detailed": "test"},
            "claims": [
                {
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "claim 1",
                    "confidence": "confirmed",
                    "evidence_refs": [
                        {"evidence_id": "EV-L0-000001"},
                        {"evidence_id": "EV-L0-000002"}
                    ],
                    "has_evidence_gap": false
                },
                {
                    "claim_id": "CL-L0-000002",
                    "category": "signal_definition",
                    "description": "claim 2",
                    "confidence": "supported",
                    "evidence_refs": [
                        {"evidence_id": "EV-L0-000003"}
                    ],
                    "has_evidence_gap": false
                }
            ],
            "module_summaries": [],
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": [],
            "evidence_gaps": [],
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": 3,
                "generation_time_ms": 100,
                "is_degraded": false
            },
            "stats": {
                "total_claims": 2,
                "claims_by_confidence": {},
                "claims_by_category": {},
                "module_count": 0,
                "signal_count": 0,
                "interface_count": 0,
                "processing_step_count": 0,
                "unknown_count": 0,
                "evidence_gap_count": 0
            }
        });

        let known_ids = make_known_ids(&["EV-L0-000001", "EV-L0-000002", "EV-L0-000003"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(
            result.is_valid,
            "all IDs exist, should pass: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_04_unknown_evidence_id_fails() {
        let mut output = make_valid_output();
        output["claims"][0]["evidence_refs"] =
            serde_json::json!([{"evidence_id": "EV-L0-009999"}]);

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::UnknownEvidenceId { evidence_id, .. }
                    if evidence_id == "EV-L0-009999"
                )),
            "expected UnknownEvidenceId for EV-L0-009999, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_05_claim_without_evidence_fails() {
        let output = serde_json::json!({
            "stage_id": "L0",
            "version": "3.0.0",
            "summary": {"short": "test", "detailed": "test"},
            "claims": [
                {
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "无证据的 claim",
                    "confidence": "inferred",
                    "evidence_refs": [],
                    "has_evidence_gap": false
                }
            ],
            "module_summaries": [],
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": [],
            "evidence_gaps": [],
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": 0,
                "generation_time_ms": 100,
                "is_degraded": false
            },
            "stats": {
                "total_claims": 1,
                "claims_by_confidence": {},
                "claims_by_category": {},
                "module_count": 0,
                "signal_count": 0,
                "interface_count": 0,
                "processing_step_count": 0,
                "unknown_count": 0,
                "evidence_gap_count": 0
            }
        });

        let known_ids = make_known_ids(&[]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::ClaimWithoutEvidence { claim_id }
                    if claim_id == "CL-L0-000001"
                )),
            "expected ClaimWithoutEvidence, got: {:?}",
            result.errors
        );
    }

    /// val_06: unknown + has_gap=true + top-level evidence_gaps 非空 → 通过
    #[test]
    fn val_06_claim_with_gap_passes() {
        let output = serde_json::json!({
            "stage_id": "L0",
            "version": "3.0.0",
            "summary": {"short": "test", "detailed": "test"},
            "claims": [
                {
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "有 gap 的 unknown claim",
                    "confidence": "unknown",
                    "evidence_refs": [],
                    "has_evidence_gap": true
                }
            ],
            "module_summaries": [],
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": [],
            "evidence_gaps": [
                {
                    "gap_id": "GAP-L0-000001",
                    "expected_evidence": "需要更多证据",
                    "reason": "证据不足",
                    "related_evidence_refs": []
                }
            ],
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": 0,
                "generation_time_ms": 100,
                "is_degraded": false
            },
            "stats": {
                "total_claims": 1,
                "claims_by_confidence": {},
                "claims_by_category": {},
                "module_count": 0,
                "signal_count": 0,
                "interface_count": 0,
                "processing_step_count": 0,
                "unknown_count": 0,
                "evidence_gap_count": 1
            }
        });

        let known_ids = make_known_ids(&[]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(
            result.is_valid,
            "unknown claim with gap and top-level gaps should pass: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_07_unknown_with_fake_id_fails() {
        let output = serde_json::json!({
            "stage_id": "L0",
            "version": "3.0.0",
            "summary": {"short": "test", "detailed": "test"},
            "claims": [
                {
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "claim 1",
                    "confidence": "confirmed",
                    "evidence_refs": [{"evidence_id": "EV-L0-000001"}],
                    "has_evidence_gap": false
                }
            ],
            "module_summaries": [],
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": [
                {
                    "unknown_id": "UNK-L0-000001",
                    "description": "未知的项",
                    "related_evidence_refs": [
                        {"evidence_id": "EV-L0-009999"}
                    ],
                    "reason": "证据不足"
                }
            ],
            "evidence_gaps": [],
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": 1,
                "generation_time_ms": 100,
                "is_degraded": false
            },
            "stats": {
                "total_claims": 1,
                "claims_by_confidence": {},
                "claims_by_category": {},
                "module_count": 0,
                "signal_count": 0,
                "interface_count": 0,
                "processing_step_count": 0,
                "unknown_count": 1,
                "evidence_gap_count": 0
            }
        });

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::UnknownEvidenceId { evidence_id, location, .. }
                    if evidence_id == "EV-L0-009999" && location.contains("UNK-L0-000001")
                )),
            "expected UnknownEvidenceId for fake ID in unknown, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_08_too_many_unknowns_warning() {
        let output = serde_json::json!({
            "stage_id": "L0",
            "version": "3.0.0",
            "summary": {"short": "test", "detailed": "test"},
            "claims": [
                {
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "claim 1",
                    "confidence": "confirmed",
                    "evidence_refs": [{"evidence_id": "EV-L0-000001"}],
                    "has_evidence_gap": false
                }
            ],
            "module_summaries": [],
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": [
                {
                    "unknown_id": "UNK-L0-000001",
                    "description": "unknown 1",
                    "related_evidence_refs": [],
                    "reason": "无证据"
                },
                {
                    "unknown_id": "UNK-L0-000002",
                    "description": "unknown 2",
                    "related_evidence_refs": [],
                    "reason": "无证据"
                }
            ],
            "evidence_gaps": [],
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": 1,
                "generation_time_ms": 100,
                "is_degraded": false
            },
            "stats": {
                "total_claims": 1,
                "claims_by_confidence": {},
                "claims_by_category": {},
                "module_count": 0,
                "signal_count": 0,
                "interface_count": 0,
                "processing_step_count": 0,
                "unknown_count": 2,
                "evidence_gap_count": 0
            }
        });

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(
            result.is_valid,
            "too many unknowns should still be valid: {:?}",
            result.errors
        );
        assert!(
            result.warnings.iter().any(|w| matches!(
                w,
                ValidationWarning::TooManyUnknowns { count: 2, claim_count: 1 }
            )),
            "expected TooManyUnknowns warning, got: {:?}",
            result.warnings
        );
    }

    // ─── val_09 ~ val_15: 结构验证加强 ─────────────────────────────

    #[test]
    fn val_09_missing_version_fails() {
        let mut output = make_valid_output();
        output.as_object_mut().unwrap().remove("version");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, .. } if path == "/version"
                )),
            "expected SchemaViolation for /version, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_10_missing_generation_meta_fails() {
        let mut output = make_valid_output();
        output.as_object_mut().unwrap().remove("generation_meta");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, .. } if path == "/generation_meta"
                )),
            "expected SchemaViolation for /generation_meta, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_11_missing_stats_fails() {
        let mut output = make_valid_output();
        output.as_object_mut().unwrap().remove("stats");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, .. } if path == "/stats"
                )),
            "expected SchemaViolation for /stats, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_12_invalid_confidence_fails() {
        let mut output = make_valid_output();
        output["claims"][0]["confidence"] = serde_json::json!("definitely");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, message, .. }
                    if path == "/claims/0/confidence" && message.contains("definitely")
                )),
            "expected SchemaViolation for invalid confidence, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_13_invalid_category_fails() {
        let mut output = make_valid_output();
        output["claims"][0]["category"] = serde_json::json!("nonexistent");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, message, .. }
                    if path == "/claims/0/category" && message.contains("nonexistent")
                )),
            "expected SchemaViolation for invalid category, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_14_empty_summary_short_fails() {
        let mut output = make_valid_output();
        output["summary"]["short"] = serde_json::json!("");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, .. } if path == "/summary/short"
                )),
            "expected SchemaViolation for empty summary.short, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_15_empty_summary_detailed_fails() {
        let mut output = make_valid_output();
        output["summary"]["detailed"] = serde_json::json!("");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, .. }
                    if path == "/summary/detailed"
                )),
            "expected SchemaViolation for empty summary.detailed, got: {:?}",
            result.errors
        );
    }

    // ─── val_16: 重复 claim_id ──────────────────────────────────────

    #[test]
    fn val_16_duplicate_claim_id_fails() {
        let output = serde_json::json!({
            "stage_id": "L0",
            "version": "3.0.0",
            "summary": {"short": "test", "detailed": "test"},
            "claims": [
                {
                    "claim_id": "CL-L0-000001",
                    "category": "module_structure",
                    "description": "claim 1",
                    "confidence": "confirmed",
                    "evidence_refs": [{"evidence_id": "EV-L0-000001"}],
                    "has_evidence_gap": false
                },
                {
                    "claim_id": "CL-L0-000001",
                    "category": "signal_definition",
                    "description": "duplicate claim",
                    "confidence": "supported",
                    "evidence_refs": [{"evidence_id": "EV-L0-000001"}],
                    "has_evidence_gap": false
                }
            ],
            "module_summaries": [],
            "signal_summaries": [],
            "interface_summaries": [],
            "processing_steps": [],
            "unknowns": [],
            "evidence_gaps": [],
            "generation_meta": {
                "provider": "mock",
                "generated_at": "2026-06-12T10:00:00Z",
                "input_evidence_count": 1,
                "generation_time_ms": 100,
                "is_degraded": false
            },
            "stats": {
                "total_claims": 2,
                "claims_by_confidence": {},
                "claims_by_category": {},
                "module_count": 0,
                "signal_count": 0,
                "interface_count": 0,
                "processing_step_count": 0,
                "unknown_count": 0,
                "evidence_gap_count": 0
            }
        });

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::DuplicateClaimId { claim_id }
                    if claim_id == "CL-L0-000001"
                )),
            "expected DuplicateClaimId, got: {:?}",
            result.errors
        );
    }

    // ─── val_17 ~ val_22: confidence 与 evidence_refs 关系 ──────────

    /// 辅助：验证非 unknown confidence + 空 refs → ClaimWithoutEvidence
    fn check_confidence_requires_refs(confidence: &str) {
        let mut output = make_valid_output();
        output["claims"][0]["confidence"] = serde_json::json!(confidence);
        output["claims"][0]["evidence_refs"] = serde_json::json!([]);
        output["claims"][0]["has_evidence_gap"] = serde_json::json!(true);

        let known_ids = make_known_ids(&[]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(
            !result.is_valid,
            "{} with empty refs should fail even with has_evidence_gap=true",
            confidence
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::ClaimWithoutEvidence { claim_id }
                    if claim_id == "CL-L0-000001"
                )),
            "expected ClaimWithoutEvidence for {} with empty refs, got: {:?}",
            confidence,
            result.errors
        );
    }

    #[test]
    fn val_17_confirmed_requires_refs() {
        check_confidence_requires_refs("confirmed");
    }

    #[test]
    fn val_18_supported_requires_refs() {
        check_confidence_requires_refs("supported");
    }

    #[test]
    fn val_19_inferred_requires_refs() {
        check_confidence_requires_refs("inferred");
    }

    #[test]
    fn val_20_conflicting_requires_refs() {
        check_confidence_requires_refs("conflicting");
    }

    #[test]
    fn val_21_unknown_without_gap_fails() {
        let mut output = make_valid_output();
        output["claims"][0]["confidence"] = serde_json::json!("unknown");
        output["claims"][0]["evidence_refs"] = serde_json::json!([]);
        output["claims"][0]["has_evidence_gap"] = serde_json::json!(false);

        let known_ids = make_known_ids(&[]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::ClaimWithoutEvidence { .. }
                )),
            "unknown without gap should fail, got: {:?}",
            result.errors
        );
    }

    /// val_22: unknown + has_gap=true 但 top-level evidence_gaps 为空 → 失败
    #[test]
    fn val_22_unknown_with_gap_but_empty_top_gaps_fails() {
        let mut output = make_valid_output();
        output["claims"][0]["confidence"] = serde_json::json!("unknown");
        output["claims"][0]["evidence_refs"] = serde_json::json!([]);
        output["claims"][0]["has_evidence_gap"] = serde_json::json!(true);
        // evidence_gaps 保持为空 []

        let known_ids = make_known_ids(&[]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(
            !result.is_valid,
            "unknown with gap=true but empty top-level gaps should fail: {:?}",
            result.errors
        );
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::ClaimWithoutEvidence { .. }
                )),
            "expected ClaimWithoutEvidence, got: {:?}",
            result.errors
        );
    }

    // ─── val_23 ~ val_24: evidence_ref 元素结构 ────────────────────

    #[test]
    fn val_23_evidence_ref_missing_evidence_id_fails() {
        let mut output = make_valid_output();
        output["claims"][0]["evidence_refs"] = serde_json::json!([{}]);

        let known_ids = make_known_ids(&[]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, .. }
                    if path.contains("evidence_id")
                )),
            "expected SchemaViolation for missing evidence_id, got: {:?}",
            result.errors
        );
    }

    #[test]
    fn val_24_evidence_ref_empty_evidence_id_fails() {
        let mut output = make_valid_output();
        output["claims"][0]["evidence_refs"] =
            serde_json::json!([{"evidence_id": ""}]);

        let known_ids = make_known_ids(&[]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result
                .errors
                .iter()
                .any(|e| matches!(
                    e,
                    ValidationError::SchemaViolation { path, message, .. }
                    if path.contains("evidence_id") && message.contains("不能为空")
                )),
            "expected SchemaViolation for empty evidence_id, got: {:?}",
            result.errors
        );
    }
}
