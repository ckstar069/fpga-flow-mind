export type WorkspaceValidity = 'likely_valid' | 'uncertain' | 'unlikely';

export type StageStatus = 'available' | 'empty' | 'missing' | 'naming_anomaly' | 'unreadable';

export type SourceKind = 'python_stage' | 'rtl' | 'test' | 'doc' | 'config' | 'external_module';

export type Language = 'python' | 'verilog' | 'systemverilog' | 'markdown' | 'text' | 'json' | 'yaml' | 'toml' | 'unknown';

export type ErrorCode =
  | 'path_not_found'
  | 'not_directory'
  | 'permission_denied'
  | 'no_stage_found'
  | 'stage_empty'
  | 'stage_unreadable'
  | 'file_unreadable'
  | 'file_too_large'
  | 'scan_timeout'
  // Phase 2 新增
  | 'evidence_collection_failed'
  | 'source_excerpt_truncated'
  | 'binary_file_skipped'
  | 'non_utf8_file_skipped'
  // Phase 3 新增
  | 'understanding_generation_failed';

export interface WorkspaceProfile {
  workspace_name: string;
  root_path: string;
  stages: StageSummary[];
  file_type_stats: Record<string, number>;
  external_refs: string[];
  validity: WorkspaceValidity;
  validity_reasons: string[];
  warnings: WorkspaceWarning[];
  error_codes: ErrorCode[];
  scan_timestamp: string;
  version: string;
}

export interface StageSummary {
  stage_id: string;
  source_path: string;
  file_count: number;
  status: StageStatus;
}

export interface WorkspaceWarning {
  error_code: ErrorCode;
  message: string;
  source_path?: string;
  related_stage_id?: string;
  recoverable: boolean;
}

export interface StageContext {
  stage_id: string;
  source_path: string;
  files: StageFile[];
  external_deps: string[];
  upstream_refs: UpstreamRef[];
  error_code?: ErrorCode;
}

export interface StageFile {
  source_path: string;
  language: Language;
  source_kind: SourceKind;
  size_bytes?: number;
}

export interface UpstreamRef {
  stage_id: string;
  interface_file_path?: string;
  inferred: boolean;
}

export interface CommandError {
  error_code: ErrorCode;
  message: string;
  recoverable: boolean;
  details?: string;
  source_path?: string;
}

export interface CommandResult<T> {
  success: boolean;
  data?: T;
  error?: CommandError;
  warnings: WorkspaceWarning[];
}

// ─── Phase 2: Evidence Model Types ──────────────────────────────────

/** 证据强度枚举（完整定义，Phase 2 只生成 direct / indirect） */
export type EvidenceStrength = 'direct' | 'indirect' | 'weak' | 'conflicting' | 'missing';

/** 行号范围（1-based，闭区间） */
export interface LineRange {
  start: number;
  end: number;
}

/** 单条证据项 */
export interface EvidenceItem {
  evidence_id: string;
  source_path: string;
  language: Language;
  source_kind: SourceKind;
  line_range: LineRange;
  symbol?: string;
  summary: string;
  strength: EvidenceStrength;
}

/** 证据收集警告（Phase 2 专用） */
export interface EvidenceWarning {
  error_code: ErrorCode;
  message: string;
  source_path?: string;
}

/** 证据收集统计 */
export interface EvidenceStats {
  files_processed: number;
  files_skipped: number;
  total_items: number;
  items_by_kind: Record<string, number>;
  items_by_strength: Record<string, number>;
}

/** 证据集合（单阶段） */
export interface EvidenceCollection {
  stage_id: string;
  evidence_items: EvidenceItem[];
  index_by_path: Record<string, string[]>;
  index_by_kind: Record<string, string[]>;
  index_by_symbol: Record<string, string[]>;
  warnings: EvidenceWarning[];
  stats: EvidenceStats;
  version: string;
}

// ─── Phase 3: Understanding Model Types ─────────────────────────────

/** 声明置信度 — Phase 3 语义判定，与 Phase 2 的 EvidenceStrength 是不同层级 */
export type ClaimConfidence =
  | 'confirmed'
  | 'supported'
  | 'inferred'
  | 'unknown'
  | 'conflicting';

/** 声明类别 */
export type ClaimCategory =
  | 'module_structure'
  | 'signal_definition'
  | 'interface_description'
  | 'data_processing'
  | 'configuration'
  | 'documentation'
  | 'test_coverage'
  | 'other';

/** 阶段摘要（Phase 3） — 与 Phase 1 的 StageSummary 字段完全不同，TS 结构类型自然区分 */
export interface UnderstandingStageSummary {
  short: string;   // 一句话摘要，≤ 80 字
  detailed: string; // 详细摘要，≤ 500 字
}

/** 证据引用 — 通过 evidence_id 回链到 Phase 2 EvidenceCollection */
export interface EvidenceRef {
  evidence_id: string;
  relevance?: string;
}

/** 实现声明 — 描述阶段实现的某个方面 */
export interface ImplementationClaim {
  claim_id: string;
  category: ClaimCategory;
  description: string;
  confidence: ClaimConfidence;
  evidence_refs: EvidenceRef[];
  has_evidence_gap: boolean;
}

/** 无法从现有 evidence 推断的信息项 */
export interface UnknownItem {
  unknown_id: string;
  description: string;
  related_evidence_refs: EvidenceRef[];
  reason: string;
}

/** 期望存在但缺失的证据 */
export interface EvidenceGap {
  gap_id: string;
  expected_evidence: string;
  reason: string;
  related_evidence_refs: EvidenceRef[];
}

/** 模块摘要 */
export interface ModuleSummary {
  name: string;
  description: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

/** 信号摘要 */
export interface SignalSummary {
  name: string;
  description: string;
  direction?: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

/** 接口摘要 */
export interface InterfaceSummary {
  name: string;
  description: string;
  interface_type?: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

/** 处理步骤摘要 */
export interface ProcessingStepSummary {
  name: string;
  description: string;
  order: number;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

/** 生成元信息 */
export interface GenerationMeta {
  provider: string;
  generated_at: string;
  input_evidence_count: number;
  generation_time_ms: number;
  is_degraded: boolean;
}

/** 统计信息 */
export interface UnderstandingStats {
  total_claims: number;
  claims_by_confidence: Record<string, number>;
  claims_by_category: Record<string, number>;
  module_count: number;
  signal_count: number;
  interface_count: number;
  processing_step_count: number;
  unknown_count: number;
  evidence_gap_count: number;
}

/** 单阶段结构化理解产物（Phase 3 中间产物） */
export interface ImplementationUnderstanding {
  stage_id: string;
  version: string;
  summary: UnderstandingStageSummary;
  claims: ImplementationClaim[];
  module_summaries: ModuleSummary[];
  signal_summaries: SignalSummary[];
  interface_summaries: InterfaceSummary[];
  processing_steps: ProcessingStepSummary[];
  unknowns: UnknownItem[];
  evidence_gaps: EvidenceGap[];
  generation_meta: GenerationMeta;
  stats: UnderstandingStats;
}

// ─── Phase 4: View Model Types ──────────────────────────────────────

export type ViewType = 'structure' | 'dataflow' | 'timing';

export type NodeType =
  | 'module' | 'function' | 'interface' | 'signal' | 'processing_step'
  | 'class' | 'constant'
  | 'input_source' | 'output_target' | 'intermediate_data'
  | 'pipeline_stage' | 'clock_domain' | 'reset_domain';

export type EdgeType =
  | 'contains' | 'calls' | 'references' | 'depends_on'
  | 'data_flow'
  | 'sequential_order' | 'pipeline_forward' | 'clock_driven';

export interface ViewTraceRef {
  claim_id?: string;
  evidence_id?: string;
  confidence: ClaimConfidence;
  relevance?: string;
}

export interface ViewLayoutHint {
  column?: number;
  row?: number;
  depth?: number;
  group?: string;
}

export interface ViewMeta {
  stage_id: string;
  view_type: ViewType;
  source_provider: string;
  is_degraded_source: boolean;
  generated_at: string;
  empty_reason?: string;
}

export interface ViewNode {
  node_id: string;
  node_type: NodeType;
  label: string;
  description: string;
  confidence: ClaimConfidence;
  trace_refs: ViewTraceRef[];
  layout?: ViewLayoutHint;
}

export interface ViewEdge {
  edge_id: string;
  edge_type: EdgeType;
  source_node_id: string;
  target_node_id: string;
  label?: string;
  description: string;
  confidence: ClaimConfidence;
  trace_refs: ViewTraceRef[];
}

export interface ViewGraph {
  view_type: ViewType;
  stage_id: string;
  nodes: ViewNode[];
  edges: ViewEdge[];
  meta: ViewMeta;
}
