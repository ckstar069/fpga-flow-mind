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
  | 'non_utf8_file_skipped';

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
