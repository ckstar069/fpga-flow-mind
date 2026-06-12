use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// ─── 枚举类型 ────────────────────────────────────────────────────────

/// 声明置信度 — 与 EvidenceStrength 是不同层级
///
/// - EvidenceStrength：Phase 2 静态提取，描述单条证据的可靠性
/// - ClaimConfidence：Phase 3 语义理解，描述基于多条证据得出的结论的置信度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimConfidence {
    /// 有充分证据直接支持（≥ 2 条 direct evidence，无矛盾）
    Confirmed,
    /// 有证据支撑，需辅助推断或上下文解释
    Supported,
    /// 有 indirect evidence 或仅单条 direct evidence 支持
    Inferred,
    /// evidence 不足或无法从 evidence 推断
    Unknown,
    /// evidence 之间存在矛盾
    Conflicting,
}

/// 声明类别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimCategory {
    /// 模块结构（module、class）
    ModuleStructure,
    /// 信号定义（wire、reg、port）
    SignalDefinition,
    /// 接口描述（输入/输出、接口协议）
    InterfaceDescription,
    /// 数据处理流程（算法、变换、流水线）
    DataProcessing,
    /// 配置与约束（时钟约束、综合参数）
    Configuration,
    /// 文档与注释（从文档中提取的实现描述）
    Documentation,
    /// 测试覆盖（测试用例、断言）
    TestCoverage,
    /// 其他
    Other,
}

// ─── 基础结构 ────────────────────────────────────────────────────────

/// 阶段摘要 — 分 short（一句话）和 detailed（详细描述）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StageSummary {
    /// 一句话摘要，建议 ≤ 80 字
    pub short: String,
    /// 详细摘要，建议 ≤ 500 字
    pub detailed: String,
}

/// 证据引用 — 通过 evidence_id 回链到 Phase 2 EvidenceCollection
///
/// 关键约束：
/// - evidence_id 必须在输入 EvidenceCollection.evidence_items 中真实存在
/// - source_path 和 line_range 不在 claim 中重复，通过 evidence_id 回链到 EvidenceItem 获取
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceRef {
    /// 引用的 evidence_id（必须存在于 EvidenceCollection 中）
    pub evidence_id: String,
    /// 引用相关性描述（可选）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub relevance: Option<String>,
}

// ─── 核心结构 ────────────────────────────────────────────────────────

/// 实现声明 — 描述阶段实现的某个方面
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationClaim {
    /// 声明唯一 ID，格式 "CL-<stage_id>-<6位序号>"
    pub claim_id: String,
    /// 声明类别
    pub category: ClaimCategory,
    /// 声明描述（中文）
    pub description: String,
    /// 置信度
    pub confidence: ClaimConfidence,
    /// 证据引用列表（至少一条，或标注 evidence_gap）
    pub evidence_refs: Vec<EvidenceRef>,
    /// 是否有 evidence gap
    pub has_evidence_gap: bool,
}

/// 无法从现有 evidence 推断的信息项
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnknownItem {
    /// 唯一 ID，格式 "UNK-<stage_id>-<6位序号>"
    pub unknown_id: String,
    /// 描述无法推断的内容
    pub description: String,
    /// 相关 evidence（可选，可能有部分证据但不足以推断）
    pub related_evidence_refs: Vec<EvidenceRef>,
    /// 原因说明
    pub reason: String,
}

/// 期望存在但缺失的证据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvidenceGap {
    /// 唯一 ID，格式 "GAP-<stage_id>-<6位序号>"
    pub gap_id: String,
    /// 期望什么 evidence
    pub expected_evidence: String,
    /// 为什么期望这个 evidence
    pub reason: String,
    /// 相关的已有 evidence（可选）
    pub related_evidence_refs: Vec<EvidenceRef>,
}

// ─── 摘要对象 ────────────────────────────────────────────────────────

/// 模块摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSummary {
    /// 模块名称
    pub name: String,
    /// 模块描述
    pub description: String,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}

/// 信号摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SignalSummary {
    /// 信号名称
    pub name: String,
    /// 信号描述
    pub description: String,
    /// 信号方向（input / output / internal）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<String>,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}

/// 接口摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterfaceSummary {
    /// 接口名称
    pub name: String,
    /// 接口描述
    pub description: String,
    /// 接口类型（port / bus / protocol / api）
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface_type: Option<String>,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}

/// 处理步骤摘要
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessingStepSummary {
    /// 步骤名称
    pub name: String,
    /// 步骤描述
    pub description: String,
    /// 步骤序号（用于排序）
    pub order: u32,
    /// 证据引用
    pub evidence_refs: Vec<EvidenceRef>,
    /// 置信度
    pub confidence: ClaimConfidence,
}

// ─── 元信息与统计 ────────────────────────────────────────────────────

/// 生成元信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMeta {
    /// 生成使用的 provider 类型
    pub provider: String,
    /// 生成时间戳（ISO 8601）
    pub generated_at: String,
    /// 输入的 EvidenceCollection 中 evidence_items 总数
    pub input_evidence_count: u32,
    /// 生成耗时（毫秒）
    pub generation_time_ms: u64,
    /// 是否为 degraded mode（无 LLM 时的降级模式）
    pub is_degraded: bool,
}

/// 统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnderstandingStats {
    /// claim 总数
    pub total_claims: u32,
    /// 按 confidence 分组计数
    pub claims_by_confidence: HashMap<String, u32>,
    /// 按 category 分组计数
    pub claims_by_category: HashMap<String, u32>,
    /// 模块摘要数
    pub module_count: u32,
    /// 信号摘要数
    pub signal_count: u32,
    /// 接口摘要数
    pub interface_count: u32,
    /// 处理步骤数
    pub processing_step_count: u32,
    /// unknown 项数
    pub unknown_count: u32,
    /// evidence gap 项数
    pub evidence_gap_count: u32,
}

// ─── 顶层结构 ────────────────────────────────────────────────────────

/// 单阶段结构化理解产物
///
/// Phase 3 中间产物，不含 structure_view / dataflow_view / timing_view。
/// Phase 4 从此对象的 claims/summaries 生成视图。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImplementationUnderstanding {
    /// 阶段 ID（来自 StageContext）
    pub stage_id: String,
    /// 版本号，格式 "3.0.0"
    pub version: String,
    /// 阶段摘要（short + detailed）
    pub summary: StageSummary,
    /// 实现声明列表
    pub claims: Vec<ImplementationClaim>,
    /// 模块摘要
    pub module_summaries: Vec<ModuleSummary>,
    /// 信号摘要
    pub signal_summaries: Vec<SignalSummary>,
    /// 接口摘要
    pub interface_summaries: Vec<InterfaceSummary>,
    /// 处理步骤摘要
    pub processing_steps: Vec<ProcessingStepSummary>,
    /// 无法从 evidence 推断的项
    pub unknowns: Vec<UnknownItem>,
    /// 证据缺失项
    pub evidence_gaps: Vec<EvidenceGap>,
    /// 生成元信息
    pub generation_meta: GenerationMeta,
    /// 统计信息
    pub stats: UnderstandingStats,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_confidence_five_values_serde() {
        let values = [
            ClaimConfidence::Confirmed,
            ClaimConfidence::Supported,
            ClaimConfidence::Inferred,
            ClaimConfidence::Unknown,
            ClaimConfidence::Conflicting,
        ];
        // 序列化为 snake_case
        assert_eq!(
            serde_json::to_string(&ClaimConfidence::Confirmed).unwrap(),
            "\"confirmed\""
        );
        assert_eq!(
            serde_json::to_string(&ClaimConfidence::Supported).unwrap(),
            "\"supported\""
        );
        assert_eq!(
            serde_json::to_string(&ClaimConfidence::Inferred).unwrap(),
            "\"inferred\""
        );
        assert_eq!(
            serde_json::to_string(&ClaimConfidence::Unknown).unwrap(),
            "\"unknown\""
        );
        assert_eq!(
            serde_json::to_string(&ClaimConfidence::Conflicting).unwrap(),
            "\"conflicting\""
        );

        // round-trip
        for v in &values {
            let json = serde_json::to_string(v).unwrap();
            let back: ClaimConfidence = serde_json::from_str(&json).unwrap();
            assert_eq!(*v, back, "roundtrip failed for {}", json);
        }
    }

    #[test]
    fn claim_confidence_rejects_invalid() {
        let result = serde_json::from_str::<ClaimConfidence>("\"definitely\"");
        assert!(result.is_err(), "must reject invalid confidence value 'definitely'");
    }

    #[test]
    fn implementation_understanding_roundtrip() {
        let iu = ImplementationUnderstanding {
            stage_id: "L0".to_string(),
            version: "3.0.0".to_string(),
            summary: StageSummary {
                short: "L0 参考模型阶段".to_string(),
                detailed: "实现了 OFDM 系统的 Python 参考模型".to_string(),
            },
            claims: vec![ImplementationClaim {
                claim_id: "CL-L0-000001".to_string(),
                category: ClaimCategory::ModuleStructure,
                description: "定义了 OFDM 调制器模块".to_string(),
                confidence: ClaimConfidence::Confirmed,
                evidence_refs: vec![EvidenceRef {
                    evidence_id: "EV-L0-000001".to_string(),
                    relevance: Some("模块定义".to_string()),
                }],
                has_evidence_gap: false,
            }],
            module_summaries: vec![ModuleSummary {
                name: "ofdm_modulator".to_string(),
                description: "OFDM 调制器".to_string(),
                evidence_refs: vec![],
                confidence: ClaimConfidence::Supported,
            }],
            signal_summaries: vec![],
            interface_summaries: vec![],
            processing_steps: vec![],
            unknowns: vec![UnknownItem {
                unknown_id: "UNK-L0-000001".to_string(),
                description: "FFT 点数未确定".to_string(),
                related_evidence_refs: vec![],
                reason: "evidence 中未找到 FFT 点数配置".to_string(),
            }],
            evidence_gaps: vec![EvidenceGap {
                gap_id: "GAP-L0-000001".to_string(),
                expected_evidence: "FFT 点数配置参数".to_string(),
                reason: "需要确认 FFT 点数是 64 还是 128".to_string(),
                related_evidence_refs: vec![],
            }],
            generation_meta: GenerationMeta {
                provider: "mock".to_string(),
                generated_at: "2026-06-12T10:00:00Z".to_string(),
                input_evidence_count: 5,
                generation_time_ms: 100,
                is_degraded: false,
            },
            stats: UnderstandingStats {
                total_claims: 1,
                claims_by_confidence: {
                    let mut m = HashMap::new();
                    m.insert("confirmed".to_string(), 1);
                    m
                },
                claims_by_category: {
                    let mut m = HashMap::new();
                    m.insert("module_structure".to_string(), 1);
                    m
                },
                module_count: 1,
                signal_count: 0,
                interface_count: 0,
                processing_step_count: 0,
                unknown_count: 1,
                evidence_gap_count: 1,
            },
        };
        let json = serde_json::to_string(&iu).unwrap();
        let back: ImplementationUnderstanding = serde_json::from_str(&json).unwrap();
        assert_eq!(iu.stage_id, back.stage_id);
        assert_eq!(iu.version, back.version);
        assert_eq!(iu.summary.short, back.summary.short);
        assert_eq!(iu.summary.detailed, back.summary.detailed);
        assert_eq!(iu.claims.len(), back.claims.len());
        assert_eq!(iu.claims[0].claim_id, back.claims[0].claim_id);
        assert_eq!(iu.claims[0].confidence, back.claims[0].confidence);
        assert_eq!(iu.stats.total_claims, back.stats.total_claims);
        assert_eq!(iu.generation_meta.is_degraded, back.generation_meta.is_degraded);
    }

    #[test]
    fn evidence_ref_relevance_optional() {
        // 有 relevance
        let with_rel = EvidenceRef {
            evidence_id: "EV-L0-000001".to_string(),
            relevance: Some("模块定义".to_string()),
        };
        let json = serde_json::to_string(&with_rel).unwrap();
        assert!(json.contains("\"relevance\""), "relevance should be present: {}", json);

        // 无 relevance — skip_serializing_if
        let without_rel = EvidenceRef {
            evidence_id: "EV-L0-000001".to_string(),
            relevance: None,
        };
        let json = serde_json::to_string(&without_rel).unwrap();
        assert!(
            !json.contains("\"relevance\""),
            "relevance=None should be skipped: {}",
            json
        );

        // 反序列化：缺少 relevance 字段 → None
        let parsed: EvidenceRef = serde_json::from_str(
            r#"{"evidence_id":"EV-L0-000001"}"#,
        )
        .unwrap();
        assert_eq!(parsed.evidence_id, "EV-L0-000001");
        assert!(parsed.relevance.is_none());
    }
}
