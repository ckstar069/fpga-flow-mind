use std::collections::HashSet;

/// 验证错误 — 阻断性，导致 is_valid = false
#[derive(Debug, Clone, PartialEq)]
pub enum ValidationError {
    /// JSON schema 验证失败（字段缺失、类型错误、枚举非法等）
    SchemaViolation { path: String, message: String },
    /// evidence_id 不存在于输入 EvidenceCollection 中（hallucination）
    UnknownEvidenceId {
        evidence_id: String,
        location: String,
    },
    /// claim 无 evidence_refs 且未标注 has_evidence_gap
    ClaimWithoutEvidence { claim_id: String },
    /// unknown 项绑定了不存在的 evidence_id
    UnknownWithFakeEvidence {
        unknown_id: String,
        evidence_id: String,
    },
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

/// SchemaValidator — 对 generator 输出进行两层验证
///
/// 验证流程：
/// 1. 结构验证：字段完整性、类型正确性
/// 2. evidence_id existence check（hallucination guard）
/// 3. 业务规则检查
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

    /// 第 1 层：结构验证
    fn check_structure(output: &serde_json::Value, errors: &mut Vec<ValidationError>) {
        // 必须是 object
        if !output.is_object() {
            errors.push(ValidationError::SchemaViolation {
                path: "/".to_string(),
                message: "输出必须是 JSON 对象".to_string(),
            });
            return;
        }

        // stage_id 非空
        match output.get("stage_id").and_then(|v| v.as_str()) {
            Some(s) if !s.is_empty() => {}
            Some(_) | None => {
                errors.push(ValidationError::SchemaViolation {
                    path: "/stage_id".to_string(),
                    message: "stage_id 必须是非空字符串".to_string(),
                });
            }
        }

        // summary 存在且包含 short 和 detailed
        if let Some(summary) = output.get("summary") {
            if !summary.is_object() {
                errors.push(ValidationError::SchemaViolation {
                    path: "/summary".to_string(),
                    message: "summary 必须是对象".to_string(),
                });
            } else {
                if summary.get("short").and_then(|v| v.as_str()).is_none() {
                    errors.push(ValidationError::SchemaViolation {
                        path: "/summary/short".to_string(),
                        message: "summary.short 必须是字符串".to_string(),
                    });
                }
                if summary.get("detailed").and_then(|v| v.as_str()).is_none() {
                    errors.push(ValidationError::SchemaViolation {
                        path: "/summary/detailed".to_string(),
                        message: "summary.detailed 必须是字符串".to_string(),
                    });
                }
            }
        } else {
            errors.push(ValidationError::SchemaViolation {
                path: "/summary".to_string(),
                message: "缺少 summary 字段".to_string(),
            });
        }

        // claims 是数组
        match output.get("claims") {
            Some(v) if v.is_array() => {
                // 检查每个 claim 的必填字段
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
    }

    /// 检查单个 claim 的结构
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

        // claim_id
        if claim.get("claim_id").and_then(|v| v.as_str()).is_none() {
            errors.push(ValidationError::SchemaViolation {
                path: format!("{}/claim_id", path_prefix),
                message: "claim_id 必须是字符串".to_string(),
            });
        }

        // category
        if claim.get("category").and_then(|v| v.as_str()).is_none() {
            errors.push(ValidationError::SchemaViolation {
                path: format!("{}/category", path_prefix),
                message: "category 必须是字符串".to_string(),
            });
        }

        // description
        if claim.get("description").and_then(|v| v.as_str()).is_none() {
            errors.push(ValidationError::SchemaViolation {
                path: format!("{}/description", path_prefix),
                message: "description 必须是字符串".to_string(),
            });
        }

        // confidence
        if claim.get("confidence").and_then(|v| v.as_str()).is_none() {
            errors.push(ValidationError::SchemaViolation {
                path: format!("{}/confidence", path_prefix),
                message: "confidence 必须是字符串".to_string(),
            });
        }

        // evidence_refs 是数组
        match claim.get("evidence_refs") {
            Some(v) if v.is_array() => {}
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

        // has_evidence_gap 是布尔
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

    /// 第 2 层：evidence_id existence check（hallucination guard）
    fn check_evidence_ids(
        output: &serde_json::Value,
        known_evidence_ids: &HashSet<String>,
        errors: &mut Vec<ValidationError>,
    ) {
        // 检查 claims[].evidence_refs[].evidence_id
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

        // 检查 module_summaries[].evidence_refs
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

        // 检查 signal_summaries[].evidence_refs
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

        // 检查 interface_summaries[].evidence_refs
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

        // 检查 processing_steps[].evidence_refs
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

        // 检查 unknowns[].related_evidence_refs
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

        // 检查 evidence_gaps[].related_evidence_refs
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
                    if !known_ids.contains(eid) {
                        errors.push(ValidationError::UnknownEvidenceId {
                            evidence_id: eid.to_string(),
                            location: location.to_string(),
                        });
                    }
                }
            }
        }
    }

    /// 第 3 层：业务规则检查
    fn check_business_rules(
        output: &serde_json::Value,
        errors: &mut Vec<ValidationError>,
        warnings: &mut Vec<ValidationWarning>,
    ) {
        // claim 无 evidence_refs 且 has_evidence_gap=false → 错误
        if let Some(claims) = output.get("claims").and_then(|v| v.as_array()) {
            for claim in claims {
                let claim_id = claim
                    .get("claim_id")
                    .and_then(|v| v.as_str())
                    .unwrap_or("unknown");
                let refs_empty = claim
                    .get("evidence_refs")
                    .and_then(|v| v.as_array())
                    .map(|a| a.is_empty())
                    .unwrap_or(true);
                let has_gap = claim
                    .get("has_evidence_gap")
                    .and_then(|v| v.as_bool())
                    .unwrap_or(false);

                if refs_empty && !has_gap {
                    errors.push(ValidationError::ClaimWithoutEvidence {
                        claim_id: claim_id.to_string(),
                    });
                }
            }
        }

        // unknown 的 related_evidence_refs 含不存在的 ID → 错误
        // （已在 check_evidence_ids 中处理为 UnknownEvidenceId）
        // 这里专门检查 unknown 中是否有伪造 ID 并用更具体的错误类型
        // 注意：evidence_id existence check 已经在 check_evidence_ids 中完成
        // 此处不需要重复

        // unknown 数量 > claims 数量 → 警告
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

        // gaps 数量 > 10 → 警告
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

    #[test]
    fn val_01_valid_output_passes() {
        let output = make_valid_output();
        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(result.is_valid, "valid output should pass: {:?}", result.errors);
        assert!(result.errors.is_empty());
        assert!(result.warnings.is_empty());
    }

    #[test]
    fn val_02_missing_stage_id_fails() {
        let mut output = make_valid_output();
        // 移除 stage_id
        output.as_object_mut().unwrap().remove("stage_id");

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result.errors.iter().any(|e| matches!(
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

        assert!(result.is_valid, "all IDs exist, should pass: {:?}", result.errors);
    }

    #[test]
    fn val_04_unknown_evidence_id_fails() {
        let mut output = make_valid_output();
        // 引用不存在的 evidence_id
        output["claims"][0]["evidence_refs"] = serde_json::json!([
            {"evidence_id": "EV-L0-009999"}
        ]);

        let known_ids = make_known_ids(&["EV-L0-000001"]);
        let result = SchemaValidator::validate(&output, &known_ids);

        assert!(!result.is_valid);
        assert!(
            result.errors.iter().any(|e| matches!(
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
            result.errors.iter().any(|e| matches!(
                e,
                ValidationError::ClaimWithoutEvidence { claim_id }
                if claim_id == "CL-L0-000001"
            )),
            "expected ClaimWithoutEvidence, got: {:?}",
            result.errors
        );
    }

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
                    "description": "有 gap 的 claim",
                    "confidence": "inferred",
                    "evidence_refs": [],
                    "has_evidence_gap": true
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

        assert!(
            result.is_valid,
            "claim with has_evidence_gap=true should pass: {:?}",
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
            result.errors.iter().any(|e| matches!(
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

        // is_valid=true — warnings 不影响
        assert!(
            result.is_valid,
            "too many unknowns should still be valid: {:?}",
            result.errors
        );
        // 有 warning
        assert!(
            result.warnings.iter().any(|w| matches!(
                w,
                ValidationWarning::TooManyUnknowns { count: 2, claim_count: 1 }
            )),
            "expected TooManyUnknowns warning, got: {:?}",
            result.warnings
        );
    }
}
