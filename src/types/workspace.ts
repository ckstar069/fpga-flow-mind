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
  | 'scan_timeout';

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
