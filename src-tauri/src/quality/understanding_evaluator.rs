//! Phase 7 Batch B: 形式化理解质量评估器（Understanding Quality Evaluator）。
//!
//! 对 `ImplementationUnderstanding` 执行确定性质量评估：
//! - claim 的 evidence_ref 存在性检查
//! - confidence=Unknown 与 has_evidence_gap 的一致性（诚实表达）
//! - StageSummary 弱摘要检测
//!
//! 仅输出 `UnderstandingQualityReport` + `QualityIssue` 列表，issue_id 留空由 reporter 统一填充。

use std::collections::HashSet;

use crate::quality::issue_builder::{make_guardrail, make_issue};
use crate::quality::models::{
    ArtifactKind, QualityIssue, QualityIssueKind, QualitySeverity, SummaryQuality,
    UnderstandingQualityReport,
};
use crate::understanding::models::ImplementationUnderstanding;

/// 评估器输入 — 引用外部数据，零拷贝。
#[derive(Debug, Clone)]
pub struct UnderstandingEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub understanding: &'a ImplementationUnderstanding,
    pub evidence_id_set: &'a HashSet<String>,
}

/// 形式化理解质量评估器（无状态）。
pub struct UnderstandingEvaluator;

impl UnderstandingEvaluator {
    /// 对单个阶段的 `ImplementationUnderstanding` 执行质量评估。
    ///
    /// 返回 `(UnderstandingQualityReport, Vec<QualityIssue>)`：
    /// - `issue_id` 全部留空，由 reporter 统一分配。
    pub fn evaluate(
        input: &UnderstandingEvaluatorInput<'_>,
    ) -> (UnderstandingQualityReport, Vec<QualityIssue>) {
        let mut issues: Vec<QualityIssue> = Vec::new();

        let mut claims_with_refs: u32 = 0;
        let mut claims_all_refs_ok: u32 = 0;

        let mut unknown_count: u32 = 0;
        let mut honest_unknown: u32 = 0;

        // ─── 逐 claim 评估 ───────────────────────────────────────────────
        for claim in &input.understanding.claims {
            // 1. evidence_refs 为空
            if claim.evidence_refs.is_empty() {
                if !claim.has_evidence_gap {
                    // 无证据且无 gap → 未支撑的声明（问题）
                    issues.push(make_issue(
                        input.sample_id,
                        input.stage_id,
                        ArtifactKind::Understanding,
                        QualityIssueKind::UnsupportedClaim,
                        QualitySeverity::High,
                        None,
                        Some(&claim.claim_id),
                        None,
                        None,
                        None,
                        &format!(
                            "claim {} 无 evidence_refs 且未标注 evidence_gap",
                            claim.claim_id
                        ),
                    ));
                } else {
                    // 无证据但标注了 gap → 守卫生效（正向记录）
                    issues.push(make_guardrail(
                        input.sample_id,
                        input.stage_id,
                        ArtifactKind::Understanding,
                        QualityIssueKind::HallucinatedClaimBlocked,
                        Some(&claim.claim_id),
                        &format!(
                            "claim {} 无证据但被 hallucination guard 拦截（诚实标注 gap）",
                            claim.claim_id
                        ),
                    ));
                }
                continue;
            }

            // 2. evidence_refs 非空 → 逐 ref 检查 existence
            claims_with_refs += 1;
            let mut refs_ok = true;
            for ev_ref in &claim.evidence_refs {
                if !input.evidence_id_set.contains(&ev_ref.evidence_id) {
                    refs_ok = false;
                    issues.push(make_issue(
                        input.sample_id,
                        input.stage_id,
                        ArtifactKind::Understanding,
                        QualityIssueKind::UnsupportedClaim,
                        QualitySeverity::High,
                        Some(&ev_ref.evidence_id),
                        Some(&claim.claim_id),
                        None,
                        None,
                        None,
                        &format!(
                            "claim {} 引用不存在的 evidence {}",
                            claim.claim_id, ev_ref.evidence_id
                        ),
                    ));
                }
            }
            if refs_ok {
                claims_all_refs_ok += 1;
            }

            // 3. confidence == Unknown 的诚实性统计
            if claim.confidence == crate::understanding::models::ClaimConfidence::Unknown {
                unknown_count += 1;
                if claim.has_evidence_gap {
                    honest_unknown += 1;
                }
            }
        }

        // ─── claim_existence_check_ratio ─────────────────────────────────
        let claim_existence_check_ratio = if claims_with_refs > 0 {
            claims_all_refs_ok as f32 / claims_with_refs as f32
        } else if input.understanding.claims.is_empty() {
            1.0
        } else {
            0.0
        };

        // ─── uncertainty_expression_ratio ────────────────────────────────
        let uncertainty_expression_ratio = if unknown_count > 0 {
            honest_unknown as f32 / unknown_count as f32
        } else {
            1.0
        };

        // ─── confidence_calibration_ratio ──────────────────────────────
        // Batch B 简化：与 claim_existence_check_ratio 相同
        let confidence_calibration_ratio = claim_existence_check_ratio;

        // ─── summary 弱摘要检测 ────────────────────────────────────────────
        let short_trimmed = input.understanding.summary.short.trim();
        let detailed_trimmed = input.understanding.summary.detailed.trim();
        let is_weak_summary = short_trimmed.is_empty() || detailed_trimmed.len() < 10;

        if is_weak_summary {
            issues.push(make_issue(
                input.sample_id,
                input.stage_id,
                ArtifactKind::Stage,
                QualityIssueKind::WeakSummary,
                QualitySeverity::Medium,
                None,
                None,
                None,
                None,
                None,
                &format!(
                    "stage {} summary 过弱：short='{}' detailed_len={}",
                    input.stage_id,
                    short_trimmed,
                    detailed_trimmed.len()
                ),
            ));
        }

        let summary_quality = SummaryQuality {
            total_summaries: 1,
            weak_summary_count: if is_weak_summary { 1 } else { 0 },
        };

        let report = UnderstandingQualityReport {
            sample_id: input.sample_id.to_string(),
            stage_id: input.stage_id.to_string(),
            claim_existence_check_ratio,
            uncertainty_expression_ratio,
            confidence_calibration_ratio,
            summary_quality,
            issue_refs: Vec::new(), // reporter 填充
        };

        (report, issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::models::QualityIssuePolarity;
    use crate::understanding::models::{
        ClaimCategory, ClaimConfidence, EvidenceRef, GenerationMeta, ImplementationClaim,
        StageSummary, UnderstandingStats,
    };
    use std::collections::HashMap;

    /// 构造最小可运行的 ImplementationUnderstanding（仅用于测试）。
    fn make_minimal_understanding(
        short: &str,
        detailed: &str,
        claims: Vec<ImplementationClaim>,
    ) -> ImplementationUnderstanding {
        ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: short.to_string(),
                detailed: detailed.to_string(),
            },
            claims,
            module_summaries: vec![],
            signal_summaries: vec![],
            interface_summaries: vec![],
            processing_steps: vec![],
            unknowns: vec![],
            evidence_gaps: vec![],
            generation_meta: GenerationMeta {
                provider: "mock".to_string(),
                generated_at: "2026-06-15T00:00:00Z".to_string(),
                input_evidence_count: 0,
                generation_time_ms: 0,
                is_degraded: false,
            },
            stats: UnderstandingStats {
                total_claims: 0,
                claims_by_confidence: HashMap::new(),
                claims_by_category: HashMap::new(),
                module_count: 0,
                signal_count: 0,
                interface_count: 0,
                processing_step_count: 0,
                unknown_count: 0,
                evidence_gap_count: 0,
            },
        }
    }

    fn make_claim(
        claim_id: &str,
        confidence: ClaimConfidence,
        evidence_refs: Vec<EvidenceRef>,
        has_gap: bool,
    ) -> ImplementationClaim {
        ImplementationClaim {
            claim_id: claim_id.to_string(),
            category: ClaimCategory::ModuleStructure,
            description: "test claim".to_string(),
            confidence,
            evidence_refs,
            has_evidence_gap: has_gap,
        }
    }

    #[test]
    fn unsupported_claim_without_gap_emits_issue() {
        let understanding = make_minimal_understanding(
            "short",
            "detailed enough",
            vec![make_claim("CL-1", ClaimConfidence::Confirmed, vec![], false)],
        );
        let evidence_set: HashSet<String> = HashSet::new();
        let input = UnderstandingEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            understanding: &understanding,
            evidence_id_set: &evidence_set,
        };
        let (report, issues) = UnderstandingEvaluator::evaluate(&input);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, QualityIssueKind::UnsupportedClaim);
        assert_eq!(issues[0].polarity, QualityIssuePolarity::Problem);
        assert_eq!(issues[0].severity, QualitySeverity::High);
        assert_eq!(issues[0].claim_id.as_deref(), Some("CL-1"));
        assert_eq!(report.claim_existence_check_ratio, 0.0);
    }

    #[test]
    fn unknown_claim_with_gap_emits_positive_guardrail() {
        let understanding = make_minimal_understanding(
            "short",
            "detailed enough",
            vec![make_claim(
                "CL-1",
                ClaimConfidence::Unknown,
                vec![],
                true, // has_evidence_gap = true
            )],
        );
        let evidence_set: HashSet<String> = HashSet::new();
        let input = UnderstandingEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            understanding: &understanding,
            evidence_id_set: &evidence_set,
        };
        let (report, issues) = UnderstandingEvaluator::evaluate(&input);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, QualityIssueKind::HallucinatedClaimBlocked);
        assert_eq!(issues[0].polarity, QualityIssuePolarity::PositiveGuardrail);
        assert_eq!(issues[0].claim_id.as_deref(), Some("CL-1"));
        // 无 claims_with_refs，且 claims 非空 → 0.0
        assert_eq!(report.claim_existence_check_ratio, 0.0);
    }

    #[test]
    fn unknown_with_fake_evidence_ref_emits_problem() {
        let understanding = make_minimal_understanding(
            "short",
            "detailed enough",
            vec![make_claim(
                "CL-1",
                ClaimConfidence::Unknown,
                vec![EvidenceRef {
                    evidence_id: "EV-FAKE".to_string(),
                    relevance: None,
                }],
                false,
            )],
        );
        let evidence_set: HashSet<String> = HashSet::new();
        let input = UnderstandingEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            understanding: &understanding,
            evidence_id_set: &evidence_set,
        };
        let (report, issues) = UnderstandingEvaluator::evaluate(&input);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, QualityIssueKind::UnsupportedClaim);
        assert_eq!(issues[0].polarity, QualityIssuePolarity::Problem);
        assert_eq!(issues[0].evidence_id.as_deref(), Some("EV-FAKE"));
        assert_eq!(issues[0].claim_id.as_deref(), Some("CL-1"));
        assert_eq!(report.claim_existence_check_ratio, 0.0);
    }

    #[test]
    fn weak_summary_emits_issue() {
        let understanding = make_minimal_understanding(
            "",           // short empty
            "too short",  // detailed < 10 chars
            vec![],
        );
        let evidence_set: HashSet<String> = HashSet::new();
        let input = UnderstandingEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            understanding: &understanding,
            evidence_id_set: &evidence_set,
        };
        let (report, issues) = UnderstandingEvaluator::evaluate(&input);

        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].kind, QualityIssueKind::WeakSummary);
        assert_eq!(issues[0].artifact_kind, ArtifactKind::Stage);
        assert_eq!(report.summary_quality.weak_summary_count, 1);
        assert_eq!(report.summary_quality.total_summaries, 1);
    }

    #[test]
    fn claim_existence_ratio_uses_existing_evidence_ids() {
        let understanding = make_minimal_understanding(
            "short",
            "detailed enough",
            vec![
                make_claim(
                    "CL-1",
                    ClaimConfidence::Confirmed,
                    vec![EvidenceRef {
                        evidence_id: "EV-REAL".to_string(),
                        relevance: None,
                    }],
                    false,
                ),
                make_claim(
                    "CL-2",
                    ClaimConfidence::Confirmed,
                    vec![
                        EvidenceRef {
                            evidence_id: "EV-REAL".to_string(),
                            relevance: None,
                        },
                        EvidenceRef {
                            evidence_id: "EV-FAKE".to_string(),
                            relevance: None,
                        },
                    ],
                    false,
                ),
            ],
        );
        let mut evidence_set = HashSet::new();
        evidence_set.insert("EV-REAL".to_string());

        let input = UnderstandingEvaluatorInput {
            sample_id: "S1",
            stage_id: "L0",
            understanding: &understanding,
            evidence_id_set: &evidence_set,
        };
        let (report, issues) = UnderstandingEvaluator::evaluate(&input);

        // CL-1: 1 ref, all ok
        // CL-2: 2 refs, 1 fake → not ok
        // claims_with_refs = 2, claims_all_refs_ok = 1 → ratio = 0.5
        assert_eq!(report.claim_existence_check_ratio, 0.5);
        assert_eq!(issues.len(), 1); // only CL-2's fake ref
        assert_eq!(issues[0].claim_id.as_deref(), Some("CL-2"));
        assert_eq!(issues[0].evidence_id.as_deref(), Some("EV-FAKE"));
    }
}
