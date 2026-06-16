import type {
  EvidenceStrength,
  ClaimConfidence,
  TraceResolution,
  QualitySeverity,
  QualityIssueKind,
  QualityIssuePolarity,
  IssueStatus,
  NodeType,
  EdgeType,
  ViewType,
} from '../../../types/workspace';
import type { ContextSelectionKind } from './contextPanelTypes';

/** ContextPanel 对象类型中文标签 */
export const CONTEXT_KIND_LABEL: Record<ContextSelectionKind, string> = {
  evidence: '证据',
  trace_target: '追溯目标',
  source_excerpt: '源码片段',
  quality_issue: '质量记录',
  view_node: '视图节点',
  view_edge: '视图边',
  qa_citation: '问答引用',
};

/** 轻量 emoji 图标（不引入图标库） */
export const CONTEXT_KIND_ICON: Record<ContextSelectionKind, string> = {
  evidence: '📄',
  trace_target: '🔍',
  source_excerpt: '📝',
  quality_issue: '⚠️',
  view_node: '🔷',
  view_edge: '➡️',
  qa_citation: '💬',
};

/** evidence strength 中文标签 */
export const EVIDENCE_STRENGTH_LABEL: Record<EvidenceStrength, string> = {
  direct: '直接',
  indirect: '间接',
  weak: '弱',
  conflicting: '冲突',
  missing: '缺失',
};

/** evidence strength 颜色 */
export const EVIDENCE_STRENGTH_COLOR: Record<EvidenceStrength, string> = {
  direct: '#4caf50',
  indirect: '#2196f3',
  weak: '#ff9800',
  conflicting: '#f44336',
  missing: '#9e9e9e',
};

/** confidence 中文标签 */
export const CONFIDENCE_LABEL: Record<ClaimConfidence, string> = {
  confirmed: '已确认',
  supported: '有支撑',
  inferred: '推断',
  unknown: '未知',
  conflicting: '矛盾',
};

/** confidence 背景色 */
export const CONFIDENCE_BG: Record<ClaimConfidence, string> = {
  confirmed: '#e3f2fd',
  supported: '#e8f5e9',
  inferred: '#fff3e0',
  unknown: '#f5f5f5',
  conflicting: '#ffebee',
};

/** confidence 文字色 */
export const CONFIDENCE_COLOR: Record<ClaimConfidence, string> = {
  confirmed: '#1565c0',
  supported: '#2e7d32',
  inferred: '#f57c00',
  unknown: '#757575',
  conflicting: '#c62828',
};

/** trace resolution 中文标签 */
export const RESOLUTION_LABEL: Record<TraceResolution, string> = {
  resolved: '已解析',
  claim_only: '仅声明',
  evidence_only: '仅证据',
  missing_claim: '声明缺失',
  missing_evidence: '证据缺失',
};

/** trace resolution 颜色 */
export const RESOLUTION_COLOR: Record<TraceResolution, string> = {
  resolved: '#2e7d32',
  claim_only: '#f57c00',
  evidence_only: '#1565c0',
  missing_claim: '#c62828',
  missing_evidence: '#c62828',
};

/** quality severity 颜色 */
export const SEVERITY_COLOR: Record<QualitySeverity, string> = {
  high: '#c62828',
  medium: '#f57c00',
  low: '#546e7a',
};

/** quality issue kind 中文标签 */
export const QUALITY_KIND_LABEL: Record<QualityIssueKind, string> = {
  missing_evidence: '缺失证据',
  noisy_evidence: '噪声证据',
  wrong_source_kind: '源类型标注异常',
  stage_identification_mismatch: '阶段识别不一致',
  weak_summary: '摘要偏弱',
  unsupported_claim: '无证据声明',
  hallucinated_claim_blocked: '幻觉声明被拦截',
  empty_or_unhelpful_view: '视图退化（空/无帮助）',
  expected_empty_timing: '时序空图（预期行为）',
  isolated_or_unconnected_view: '孤立节点视图',
  traceability_gap: '追溯缺口',
  low_semantic_diversity: '语义多样性不足',
  qa_unanswered_when_evidence_exists: '有证据未回答',
  qa_answer_without_valid_citation: '引用无效',
  confusing_ui_state: 'UI 状态不清',
};

/** quality issue polarity 中文标签 */
export const POLARITY_LABEL: Record<QualityIssuePolarity, string> = {
  problem: '问题',
  positive_guardrail: '正向守卫',
};

/** issue status 中文标签 */
export const ISSUE_STATUS_LABEL: Record<IssueStatus, string> = {
  open: '打开',
  fixed: '已修复',
  accepted_as_known_limitation: '已接受为已知限制',
};

/** node type 中文标签 */
export const NODE_TYPE_LABEL: Record<NodeType, string> = {
  module: '模块',
  function: '函数',
  interface: '接口',
  signal: '信号',
  processing_step: '处理步骤',
  class: '类',
  constant: '常量',
  input_source: '输入源',
  output_target: '输出目标',
  intermediate_data: '中间数据',
  pipeline_stage: '流水线阶段',
  clock_domain: '时钟域',
  reset_domain: '复位域',
};

/** edge type 中文标签 */
export const EDGE_TYPE_LABEL: Record<EdgeType, string> = {
  contains: '包含',
  calls: '调用',
  references: '引用',
  depends_on: '依赖',
  data_flow: '数据流',
  sequential_order: '顺序',
  pipeline_forward: '流水线前向',
  clock_driven: '时钟驱动',
};

/** view type 中文标签 */
export const VIEW_TYPE_LABEL: Record<ViewType, string> = {
  structure: '结构图',
  dataflow: '数据流',
  timing: '时序流水',
};

/** source kind 中文标签 */
export const SOURCE_KIND_LABEL: Record<string, string> = {
  python_stage: 'Python 阶段',
  rtl: 'RTL',
  test: '测试',
  doc: '文档',
  config: '配置',
  external_module: '外部模块',
};

/** language 中文标签 */
export const LANGUAGE_LABEL: Record<string, string> = {
  python: 'Python',
  verilog: 'Verilog',
  systemverilog: 'SystemVerilog',
  markdown: 'Markdown',
  text: '纯文本',
  json: 'JSON',
  yaml: 'YAML',
  toml: 'TOML',
  unknown: '未知',
};

/** SelectedTraceTarget 类型中文标签 */
export function traceTargetTypeLabel(kind: string): string {
  switch (kind) {
    case 'view_node':
      return '视图节点';
    case 'view_edge':
      return '视图边';
    case 'claim':
      return '声明';
    case 'evidence':
      return '证据';
    default:
      return kind;
  }
}
