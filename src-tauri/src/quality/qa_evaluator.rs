//! Q&A 质量评估器（Phase 7 Batch B）。
//!
//! 对单个阶段的 `GroundedAnswer` 进行形式化质量评估，输出
//! `QaQualityReport` 与 QA 相关 `QualityIssue`。
//!
//! 使用既有类型：
//! - `GroundedAnswer`、`GroundedAnswerCitation`（`crate::trace::models`）
//! - `QaQualityReport`、`QaEvaluationQuestionSet`、`QaExpectedAnswerability`、
//!   `QualityIssue`、`QualityIssueKind`、`QualitySeverity`（`crate::quality::models`）
//! - `make_issue`（`crate::quality::issue_builder`）

use std::collections::HashSet;

use crate::quality::issue_builder::make_issue;
use crate::quality::models::{
    ArtifactKind, QaEvaluationQuestion, QaEvaluationQuestionSet, QaExpectedAnswerability,
    QaQualityReport, QualityIssue, QualityIssueKind, QualitySeverity,
};
use crate::trace::models::GroundedAnswer;

/// Q&A 评估输入。
#[derive(Debug, Clone)]
pub struct QaEvaluatorInput<'a> {
    pub sample_id: &'a str,
    pub stage_id: &'a str,
    pub answer: &'a GroundedAnswer,
    pub evidence_id_set: &'a HashSet<String>,
    pub claim_id_set: &'a HashSet<String>,
    pub question_set: Option<&'a QaEvaluationQuestionSet>,
}

/// Q&A 质量评估器（无状态）。
pub struct QaEvaluator;

impl QaEvaluator {
    /// 评估 Q&A 质量，返回报告与发现的质量记录。
    ///
    /// 行为详见模块文档与实现注释。
    pub fn evaluate(
        input: &QaEvaluatorInput<'_>,
    ) -> (QaQualityReport, Vec<QualityIssue>) {
        let mut issues = Vec::new();

        // ── 1. Citation validity scan（always performed）────────────────
        let mut valid_citations = 0u32;
        let mut total_citations_with_id = 0u32;

        for citation in &input.answer.citations {
            let ev_id = citation.evidence_id.as_ref().map(|s| s.as_str());
            let cl_id = citation.claim_id.as_ref().map(|s| s.as_str());

            let has_ev = ev_id.map(|s| !s.is_empty()).unwrap_or(false);
            let has_cl = cl_id.map(|s| !s.is_empty()).unwrap_or(false);

            if !(has_ev || has_cl) {
                // 完全没有 ID 的 citation 不计入 denominator
                continue;
            }

            total_citations_with_id += 1;

            let mut ev_valid = true;
            let mut cl_valid = true;

            if has_ev && !input.evidence_id_set.contains(ev_id.unwrap()) {
                ev_valid = false;
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::Qa,
                    QualityIssueKind::QaAnswerWithoutValidCitation,
                    QualitySeverity::Medium,
                    ev_id,
                    cl_id,
                    None,
                    None,
                    None,
                    &format!("Q&A 回答引用了不存在的 evidence '{}'", ev_id.unwrap()),
                ));
            }

            if has_cl && !input.claim_id_set.contains(cl_id.unwrap()) {
                cl_valid = false;
                issues.push(make_issue(
                    input.sample_id,
                    input.stage_id,
                    ArtifactKind::Qa,
                    QualityIssueKind::QaAnswerWithoutValidCitation,
                    QualitySeverity::Medium,
                    ev_id,
                    cl_id,
                    None,
                    None,
                    None,
                    &format!("Q&A 回答引用了不存在的 claim '{}'", cl_id.unwrap()),
                ));
            }

            let ev_ok = !has_ev || ev_valid;
            let cl_ok = !has_cl || cl_valid;
            if ev_ok && cl_ok {
                valid_citations += 1;
            }
        }

        let citation_validity_ratio = if total_citations_with_id > 0 {
            valid_citations as f32 / total_citations_with_id as f32
        } else {
            1.0
        };

        // ── 2. Question-set-aware evaluation ────────────────────────────
        let answerable_hit_ratio: f32;
        let unknown_honesty_ratio: f32;

        if let Some(qs) = input.question_set {
            // Find the first matching question for this stage
            let matched_question: Option<&QaEvaluationQuestion> = qs
                .questions
                .iter()
                .find(|q| q.stage_id == input.stage_id);

            if let Some(question) = matched_question {
                match question.expected_answerability {
                    QaExpectedAnswerability::Answerable => {
                        // Answerable question: should be answered (not degraded/unknown, with valid citations)
                        let is_unanswered = input.answer.is_degraded
                            || input.answer.confidence
                                == crate::trace::models::ClaimConfidence::Unknown
                            || valid_citations == 0;

                        if is_unanswered {
                            issues.push(make_issue(
                                input.sample_id,
                                input.stage_id,
                                ArtifactKind::Qa,
                                QualityIssueKind::QaUnansweredWhenEvidenceExists,
                                QualitySeverity::High,
                                None,
                                None,
                                None,
                                None,
                                None,
                                "有证据支持的问题未被回答（ degraded / unknown / 无有效 citation）",
                            ));
                            answerable_hit_ratio = 0.0;
                        } else {
                            answerable_hit_ratio = 1.0;
                        }

                        // For answerable questions, unknown_honesty_ratio is not applicable
                        // (the question expects an answer, not honesty about gaps)
                        unknown_honesty_ratio = 1.0;
                    }
                    QaExpectedAnswerability::NotAnswerable => {
                        // NotAnswerable question: should return unknown/degraded with no fabricated citations
                        let is_unknown_or_degraded = input.answer.confidence
                            == crate::trace::models::ClaimConfidence::Unknown
                            || input.answer.is_degraded;

                        // Check for fabricated/invalid citations
                        let has_fabricated_citations = input.answer.citations.iter().any(|c| {
                            let ev_present = c
                                .evidence_id
                                .as_ref()
                                .map(|s| !s.is_empty())
                                .unwrap_or(false);
                            let cl_present = c
                                .claim_id
                                .as_ref()
                                .map(|s| !s.is_empty())
                                .unwrap_or(false);
                            // A citation is "fabricated" if it has IDs but none are valid
                            (ev_present
                                && !input.evidence_id_set.contains(
                                    c.evidence_id.as_ref().unwrap(),
                                ))
                                || (cl_present
                                    && !input.claim_id_set.contains(
                                        c.claim_id.as_ref().unwrap(),
                                    ))
                        });

                        if is_unknown_or_degraded && !has_fabricated_citations {
                            // Honest expression of uncertainty — no negative issue
                            unknown_honesty_ratio = 1.0;
                        } else if !is_unknown_or_degraded {
                            // Gave a confident answer when it shouldn't be answerable
                            // This is a form of dishonesty / hallucination
                            issues.push(make_issue(
                                input.sample_id,
                                input.stage_id,
                                ArtifactKind::Qa,
                                QualityIssueKind::QaUnansweredWhenEvidenceExists,
                                QualitySeverity::High,
                                None,
                                None,
                                None,
                                None,
                                None,
                                "对无证据支持的问题给出了确定回答（应返回 unknown / degraded）",
                            ));
                            unknown_honesty_ratio = 0.0;
                        } else {
                            // is_unknown_or_degraded but has fabricated citations
                            // The invalid citations are already emitted in step 1 as
                            // QaAnswerWithoutValidCitation issues.
                            // We still mark honesty as 0.0 because fabricated citations
                            // indicate dishonest behavior even when claiming uncertainty.
                            unknown_honesty_ratio = 0.0;
                        }

                        // For not-answerable questions, answerable_hit_ratio is not applicable
                        answerable_hit_ratio = 1.0;
                    }
                }
            } else {
                // No matching question — fallback to Batch A behavior
                // Batch A fallback: no QaEvaluationQuestionSet provided; ratios are confidence proxies,
                // not real hit/honesty assessments.
                answerable_hit_ratio = if !input.answer.is_degraded
                    && input.answer.confidence
                        != crate::trace::models::ClaimConfidence::Unknown
                {
                    1.0
                } else {
                    0.0
                };
                unknown_honesty_ratio = if input.answer.confidence
                    == crate::trace::models::ClaimConfidence::Unknown
                {
                    1.0
                } else {
                    0.0
                };
            }
        } else {
            // Batch A fallback: no QaEvaluationQuestionSet provided; ratios are confidence proxies,
            // not real hit/honesty assessments.
            answerable_hit_ratio = if !input.answer.is_degraded
                && input.answer.confidence
                    != crate::trace::models::ClaimConfidence::Unknown
            {
                1.0
            } else {
                0.0
            };
            unknown_honesty_ratio = if input.answer.confidence
                == crate::trace::models::ClaimConfidence::Unknown
            {
                1.0
            } else {
                0.0
            };
        }

        let report = QaQualityReport {
            sample_id: input.sample_id.to_string(),
            stage_id: input.stage_id.to_string(),
            citation_validity_ratio,
            answerable_hit_ratio,
            unknown_honesty_ratio,
            issue_refs: Vec::new(), // reporter fills
        };

        (report, issues)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::quality::models::{
        QaEvaluationQuestion, QaExpectedAnswerability,
        QualityIssueKind, QualityIssuePolarity, QualitySeverity,
    };
    use crate::trace::models::{
        ClaimConfidence, GroundedAnswer, GroundedAnswerCitation, GroundedAnswerClaim,
    };
    use std::collections::HashSet;

    fn make_answer(
        confidence: ClaimConfidence,
        is_degraded: bool,
        citations: Vec<GroundedAnswerCitation>,
    ) -> GroundedAnswer {
        GroundedAnswer {
            answer_id: "A-001".to_string(),
            generated_at: "2026-06-14T10:00:00Z".to_string(),
            text: "test answer".to_string(),
            claims: vec![GroundedAnswerClaim {
                text: "claim text".to_string(),
                confidence: ClaimConfidence::Confirmed,
                citation_indices: vec![],
                reason: None,
            }],
            citations,
            confidence,
            warnings: vec![],
            provider: "mock".to_string(),
            is_degraded,
        }
    }

    fn make_citation(ev_id: Option<&str>, cl_id: Option<&str>) -> GroundedAnswerCitation {
        GroundedAnswerCitation {
            index: 0,
            evidence_id: ev_id.map(|s| s.to_string()),
            claim_id: cl_id.map(|s| s.to_string()),
            source_location: None,
            excerpt_summary: "test".to_string(),
        }
    }

    #[test]
    fn invalid_evidence_citation_emits_issue() {
        let mut ev_set = HashSet::new();
        ev_set.insert("EV-1".to_string());

        let answer = make_answer(
            ClaimConfidence::Confirmed,
            false,
            vec![make_citation(Some("EV-BAD"), None)],
        );
        let input = QaEvaluatorInput {
            sample_id: "sample-001",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &ev_set,
            claim_id_set: &HashSet::new(),
            question_set: None,
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.citation_validity_ratio, 0.0);
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.kind, QualityIssueKind::QaAnswerWithoutValidCitation);
        assert_eq!(issue.evidence_id, Some("EV-BAD".to_string()));
        assert_eq!(issue.polarity, QualityIssuePolarity::Problem);
        assert_eq!(issue.severity, QualitySeverity::Medium);
    }

    #[test]
    fn invalid_claim_citation_emits_issue() {
        let mut cl_set = HashSet::new();
        cl_set.insert("CL-1".to_string());

        let answer = make_answer(
            ClaimConfidence::Confirmed,
            false,
            vec![make_citation(None, Some("CL-BAD"))],
        );
        let input = QaEvaluatorInput {
            sample_id: "sample-002",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &HashSet::new(),
            claim_id_set: &cl_set,
            question_set: None,
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.citation_validity_ratio, 0.0);
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.kind, QualityIssueKind::QaAnswerWithoutValidCitation);
        assert_eq!(issue.claim_id, Some("CL-BAD".to_string()));
    }

    #[test]
    fn answerable_question_unknown_emits_unanswered_issue() {
        let mut ev_set = HashSet::new();
        ev_set.insert("EV-1".to_string());

        let question_set = QaEvaluationQuestionSet {
            set_id: "QS-1".to_string(),
            sample_id: "sample-003".to_string(),
            questions: vec![QaEvaluationQuestion {
                question: "What is the width?".to_string(),
                stage_id: "L0".to_string(),
                expected_answerability: QaExpectedAnswerability::Answerable,
                expected_evidence_ids: vec!["EV-1".to_string()],
                expected_claim_ids: vec![],
                note: None,
            }],
        };

        // Unknown confidence with no valid citations = unanswered
        let answer = make_answer(ClaimConfidence::Unknown, true, vec![]);
        let input = QaEvaluatorInput {
            sample_id: "sample-003",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &ev_set,
            claim_id_set: &HashSet::new(),
            question_set: Some(&question_set),
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.answerable_hit_ratio, 0.0);
        assert_eq!(report.unknown_honesty_ratio, 1.0);
        assert_eq!(issues.len(), 1);
        let issue = &issues[0];
        assert_eq!(issue.kind, QualityIssueKind::QaUnansweredWhenEvidenceExists);
        assert_eq!(issue.severity, QualitySeverity::High);
    }

    #[test]
    fn not_answerable_unknown_without_citation_no_problem() {
        let question_set = QaEvaluationQuestionSet {
            set_id: "QS-1".to_string(),
            sample_id: "sample-004".to_string(),
            questions: vec![QaEvaluationQuestion {
                question: "What is the secret?".to_string(),
                stage_id: "L0".to_string(),
                expected_answerability: QaExpectedAnswerability::NotAnswerable,
                expected_evidence_ids: vec![],
                expected_claim_ids: vec![],
                note: None,
            }],
        };

        // Unknown with no citations = honest expression of uncertainty
        let answer = make_answer(ClaimConfidence::Unknown, true, vec![]);
        let input = QaEvaluatorInput {
            sample_id: "sample-004",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &HashSet::new(),
            claim_id_set: &HashSet::new(),
            question_set: Some(&question_set),
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.unknown_honesty_ratio, 1.0);
        assert_eq!(report.answerable_hit_ratio, 1.0);
        assert!(issues.is_empty(), "NotAnswerable with unknown and no citations should not emit issues");
    }

    #[test]
    fn answerable_hit_ratio_with_question_set() {
        let mut ev_set = HashSet::new();
        ev_set.insert("EV-1".to_string());

        let question_set = QaEvaluationQuestionSet {
            set_id: "QS-1".to_string(),
            sample_id: "sample-005".to_string(),
            questions: vec![QaEvaluationQuestion {
                question: "What is the width?".to_string(),
                stage_id: "L0".to_string(),
                expected_answerability: QaExpectedAnswerability::Answerable,
                expected_evidence_ids: vec!["EV-1".to_string()],
                expected_claim_ids: vec![],
                note: None,
            }],
        };

        // Properly answered: Confirmed confidence, not degraded, with valid citation
        let answer = make_answer(
            ClaimConfidence::Confirmed,
            false,
            vec![make_citation(Some("EV-1"), None)],
        );
        let input = QaEvaluatorInput {
            sample_id: "sample-005",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &ev_set,
            claim_id_set: &HashSet::new(),
            question_set: Some(&question_set),
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.citation_validity_ratio, 1.0);
        assert_eq!(report.answerable_hit_ratio, 1.0);
        assert_eq!(report.unknown_honesty_ratio, 1.0);
        assert!(issues.is_empty());
    }

    #[test]
    fn citation_with_valid_evidence_but_invalid_claim_is_not_valid() {
        let mut ev_set = HashSet::new();
        ev_set.insert("EV-1".to_string());

        let answer = make_answer(
            ClaimConfidence::Confirmed,
            false,
            vec![make_citation(Some("EV-1"), Some("CL-BAD"))],
        );
        let input = QaEvaluatorInput {
            sample_id: "sample-010",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &ev_set,
            claim_id_set: &HashSet::new(),
            question_set: None,
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.citation_validity_ratio, 0.0);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].claim_id, Some("CL-BAD".to_string()));
    }

    #[test]
    fn citation_with_valid_claim_but_invalid_evidence_is_not_valid() {
        let mut cl_set = HashSet::new();
        cl_set.insert("CL-1".to_string());

        let answer = make_answer(
            ClaimConfidence::Confirmed,
            false,
            vec![make_citation(Some("EV-BAD"), Some("CL-1"))],
        );
        let input = QaEvaluatorInput {
            sample_id: "sample-011",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &HashSet::new(),
            claim_id_set: &cl_set,
            question_set: None,
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.citation_validity_ratio, 0.0);
        assert_eq!(issues.len(), 1);
        assert_eq!(issues[0].evidence_id, Some("EV-BAD".to_string()));
    }

    #[test]
    fn citation_with_both_valid_counts_once() {
        let mut ev_set = HashSet::new();
        ev_set.insert("EV-1".to_string());
        let mut cl_set = HashSet::new();
        cl_set.insert("CL-1".to_string());

        let answer = make_answer(
            ClaimConfidence::Confirmed,
            false,
            vec![make_citation(Some("EV-1"), Some("CL-1"))],
        );
        let input = QaEvaluatorInput {
            sample_id: "sample-012",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &ev_set,
            claim_id_set: &cl_set,
            question_set: None,
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.citation_validity_ratio, 1.0);
        assert!(issues.is_empty());
    }

    #[test]
    fn citation_without_ids_not_counted_in_denominator() {
        let mut ev_set = HashSet::new();
        ev_set.insert("EV-1".to_string());

        // One citation without IDs, one valid citation
        let answer = make_answer(
            ClaimConfidence::Confirmed,
            false,
            vec![
                make_citation(None, None),
                make_citation(Some("EV-1"), None),
            ],
        );
        let input = QaEvaluatorInput {
            sample_id: "sample-013",
            stage_id: "L0",
            answer: &answer,
            evidence_id_set: &ev_set,
            claim_id_set: &HashSet::new(),
            question_set: None,
        };
        let (report, issues) = QaEvaluator::evaluate(&input);
        assert_eq!(report.citation_validity_ratio, 1.0);
        assert!(issues.is_empty());
    }
}
