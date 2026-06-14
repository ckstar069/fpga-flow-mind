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
  | 'understanding_generation_failed'
  // Phase 5 新增
  | 'trace_target_not_found'
  | 'source_path_not_allowed'
  | 'source_file_unreadable'
  | 'line_range_invalid'
  | 'qa_generation_failed'
  | 'qa_validation_failed'
  // Phase 6 新增
  | 'persist_failed'
  | 'load_failed'
  | 'session_not_found'
  | 'storage_version_incompatible'
  | 'session_delete_failed'
  | 'invalid_session_id';

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

// ─── Phase 5: Trace & Grounded Q&A Types ───────────────────────────

/** 用户一次点击选择的目标 */
export type SelectedTraceTarget =
  | { kind: 'view_node'; view_type: ViewType; node_id: string }
  | { kind: 'view_edge'; view_type: ViewType; edge_id: string }
  | { kind: 'claim'; claim_id: string }
  | { kind: 'evidence'; evidence_id: string };

/** 追溯来源类型 */
export type TraceSourceKind = 'view_node' | 'view_edge' | 'claim' | 'evidence';

/** 追溯解析状态 */
export type TraceResolution =
  | 'resolved'
  | 'claim_only'
  | 'evidence_only'
  | 'missing_claim'
  | 'missing_evidence';

/** claim 的轻量展示形态 */
export interface ClaimSnapshot {
  claim_id: string;
  category: ClaimCategory;
  description: string;
  confidence: ClaimConfidence;
  evidence_ref_count: number;
  has_evidence_gap: boolean;
}

/** evidence 的轻量展示形态 */
export interface EvidenceSnapshot {
  evidence_id: string;
  source_path: string;
  language: Language;
  source_kind: SourceKind;
  line_range: LineRange;
  symbol?: string;
  summary: string;
  strength: EvidenceStrength;
}

/** 解析后的追溯引用 */
export interface TraceRefResolved {
  source_kind: TraceSourceKind;
  claim?: ClaimSnapshot;
  evidence?: EvidenceSnapshot;
  confidence: ClaimConfidence;
  relevance?: string;
  resolution: TraceResolution;
}

/** 源码位置 */
export interface SourceLocation {
  source_path: string;
  line_range: LineRange;
  evidence_id?: string;
}

/** 源码行 */
export interface SourceLine {
  line_number: number;
  content: string;
}

/** 源码片段警告 */
export interface ExcerptWarning {
  error_code: string;
  message: string;
}

/** 源码片段 */
export interface SourceExcerpt {
  location: SourceLocation;
  language: Language;
  lines: SourceLine[];
  is_truncated: boolean;
  truncation_reason?: string;
  warnings: ExcerptWarning[];
}

/** Q&A 输入 */
export interface GroundedQuestion {
  question: string;
  stage_id: string;
  selected_target?: SelectedTraceTarget;
  understanding: ImplementationUnderstanding;
  evidence_collection: EvidenceCollection;
}

/** 回答中的单个 claim */
export interface GroundedAnswerClaim {
  text: string;
  confidence: ClaimConfidence;
  citation_indices: number[];
  reason?: string;
}

/** 回答引用 */
export interface GroundedAnswerCitation {
  index: number;
  evidence_id?: string;
  claim_id?: string;
  source_location?: SourceLocation;
  excerpt_summary: string;
}

/** Q&A 警告 */
export interface GroundedQaWarning {
  code: string;
  message: string;
}

/** Q&A 输出 */
export interface GroundedAnswer {
  answer_id: string;
  generated_at: string;
  text: string;
  claims: GroundedAnswerClaim[];
  citations: GroundedAnswerCitation[];
  confidence: ClaimConfidence;
  warnings: GroundedQaWarning[];
  provider: string;
  is_degraded: boolean;
}

// ─── Phase 6: Persistence Types ─────────────────────────────────────

/** 持久化存储格式版本号 */
export interface StorageVersion {
  major: number;
  minor: number;
  patch: number;
}

/** 持久化的目标项目信息 */
export interface PersistedWorkspace {
  workspace_name: string;
  root_path: string;
  canonical_root_path: string;
  fingerprint: string;
  fingerprint_algorithm: string;
  workspace_profile_path: string;
}

/** artifact 相对路径索引 */
export interface ArtifactIndex {
  stage_context_path?: string;
  evidence_collection_path?: string;
  understanding_path?: string;
  view_graphs_path?: string;
  qa_history_path?: string;
  ui_state_path?: string;
}

/** manifest 中的阶段摘要 */
export interface PersistedStageSummary {
  stage_id: string;
  stage_name: string;
  artifacts: ArtifactIndex;
  last_analyzed_at: string;
}

/** 会话清单 */
export interface SessionManifest {
  session_id: string;
  storage_version: StorageVersion;
  created_at: string;
  updated_at: string;
  app_version: string;
  persisted_workspace: PersistedWorkspace;
  stages: PersistedStageSummary[];
  selected_stage_id?: string;
  global_ui_state?: GlobalUiState;
}

/** 单阶段完整产物集合 */
export interface PersistedStageArtifacts {
  stage_id: string;
  stage_context?: StageContext;
  evidence_collection?: EvidenceCollection;
  understanding?: ImplementationUnderstanding;
  view_graphs?: ViewGraph[];
  qa_history?: QaHistory;
  ui_state?: PersistedUiState;
}

/** Q&A 历史 */
export interface QaHistory {
  stage_id: string;
  entries: QaHistoryEntry[];
  version: string;
}

/** 单条 Q&A 记录 */
export interface QaHistoryEntry {
  entry_id: string;
  timestamp: string;
  question: string;
  answer: GroundedAnswer;
  selected_target_kind?: string;
}

/** 单阶段 UI 状态 */
export interface PersistedUiState {
  stage_id: string;
  selected_trace_target?: SelectedTraceTarget;
  resolved_traces: TraceRefResolved[];
  current_source_excerpt?: SourceExcerpt;
  highlighted_evidence_id?: string;
  active_view_type?: ViewType;
}

/** 全局 UI 状态 */
export interface GlobalUiState {
  last_session_id?: string;
  last_root_path?: string;
}

/** load_session 的目标项目状态 */
export type LoadSessionStatus =
  | 'source_unchanged'
  | 'source_changed'
  | 'source_missing'
  | 'source_path_not_allowed';

/** 完整运行时会话状态 */
export interface SessionState {
  workspace_profile: WorkspaceProfile;
  selected_stage_id?: string;
  stage_contexts: Record<string, StageContext>;
  evidence_collections: Record<string, EvidenceCollection>;
  understandings: Record<string, ImplementationUnderstanding>;
  view_graphs: Record<string, ViewGraph[]>;
  qa_histories: Record<string, QaHistory>;
  ui_states: Record<string, PersistedUiState>;
  global_ui_state?: GlobalUiState;
}

/** load_session 命令的业务结果（成功分支） */
export interface LoadSessionResult {
  success: boolean;
  status: LoadSessionStatus;
  session_state: SessionState;
  mismatch_reason?: string;
  warnings: string[];
}

/** save_session 命令的结果 */
export interface SaveSessionResult {
  session_id: string;
  saved_at: string;
  success: boolean;
}

/** 会话列表摘要 */
export interface SessionSummary {
  session_id: string;
  workspace_name: string;
  root_path: string;
  updated_at: string;
  stage_count: number;
}
