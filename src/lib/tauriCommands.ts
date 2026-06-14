import { invoke } from '@tauri-apps/api/core';
import type { CommandResult, WorkspaceProfile, StageContext, EvidenceCollection, ImplementationUnderstanding, ViewGraph, SelectedTraceTarget, TraceRefResolved, SourceLocation, SourceExcerpt, GroundedQuestion, GroundedAnswer, SessionState, SaveSessionResult, LoadSessionResult, SessionSummary } from '../types/workspace';

export class CommandError extends Error {
  error_code: string;
  recoverable: boolean;
  details?: string;
  source_path?: string;

  constructor(error: {
    error_code: string;
    message: string;
    recoverable: boolean;
    details?: string;
    source_path?: string;
  }) {
    super(error.message);
    this.error_code = error.error_code;
    this.recoverable = error.recoverable;
    this.details = error.details;
    this.source_path = error.source_path;
  }
}

export async function openWorkspace(path: string): Promise<WorkspaceProfile> {
  const result = await invoke<CommandResult<WorkspaceProfile>>('open_workspace', { path });
  return handleResult(result);
}

export async function selectStage(rootPath: string, stageId: string): Promise<StageContext> {
  const result = await invoke<CommandResult<StageContext>>('select_stage', {
    rootPath: rootPath,
    stageId: stageId,
  });
  return handleResult(result);
}

export async function collectEvidence(rootPath: string, stageId: string): Promise<EvidenceCollection> {
  const result = await invoke<CommandResult<EvidenceCollection>>('collect_evidence', {
    rootPath: rootPath,
    stageId: stageId,
  });
  return handleResult(result);
}

export async function generateUnderstanding(rootPath: string, stageId: string): Promise<ImplementationUnderstanding> {
  const result = await invoke<CommandResult<ImplementationUnderstanding>>('generate_understanding', {
    rootPath: rootPath,
    stageId: stageId,
  });
  return handleResult(result);
}

export async function generateViews(understanding: ImplementationUnderstanding): Promise<ViewGraph[]> {
  const result = await invoke<CommandResult<ViewGraph[]>>('generate_views', {
    understanding: understanding,
  });
  return handleResult(result);
}

export async function resolveTraceTarget(
  target: SelectedTraceTarget,
  understanding: ImplementationUnderstanding,
  evidenceCollection: EvidenceCollection,
  views: ViewGraph[]
): Promise<TraceRefResolved[]> {
  const result = await invoke<CommandResult<TraceRefResolved[]>>('resolve_trace_target', {
    target,
    understanding,
    evidenceCollection,
    views,
  });
  return handleResult(result);
}

export async function getSourceExcerpt(location: SourceLocation, rootPath: string): Promise<SourceExcerpt> {
  const result = await invoke<CommandResult<SourceExcerpt>>('get_source_excerpt', {
    location,
    rootPath,
  });
  return handleResult(result);
}

export async function askGroundedQuestion(
  question: GroundedQuestion,
  views?: ViewGraph[],
  resolvedTraces?: TraceRefResolved[]
): Promise<GroundedAnswer> {
  const result = await invoke<CommandResult<GroundedAnswer>>('ask_grounded_question', {
    question,
    views: views ?? null,
    resolvedTraces: resolvedTraces ?? null,
  });
  return handleResult(result);
}

// ─── Phase 6: Session Persistence Commands ──────────────────────────

export async function saveSession(sessionState: SessionState, sessionId?: string): Promise<SaveSessionResult> {
  const result = await invoke<CommandResult<SaveSessionResult>>('save_session', {
    sessionId: sessionId ?? null,
    sessionState,
  });
  return handleResult(result);
}

export async function loadSession(sessionId: string): Promise<LoadSessionResult> {
  const result = await invoke<CommandResult<LoadSessionResult>>('load_session', {
    sessionId,
  });
  return handleResult(result);
}

export async function listSessions(limit?: number): Promise<SessionSummary[]> {
  const result = await invoke<CommandResult<SessionSummary[]>>('list_sessions', {
    limit: limit ?? null,
  });
  return handleResult(result);
}

export async function deleteSession(sessionId: string): Promise<void> {
  const result = await invoke<CommandResult<void>>('delete_session', {
    sessionId,
  });
  return handleResult(result);
}

export async function getLastSession(): Promise<SessionSummary | null> {
  const result = await invoke<CommandResult<SessionSummary | null>>('get_last_session', {});
  return handleResult(result);
}

function handleResult<T>(result: CommandResult<T>): T {
  if (!result.success) {
    if (result.error) {
      throw new CommandError(result.error);
    }
    throw new CommandError({
      error_code: 'unknown',
      message: '未知错误',
      recoverable: false,
    });
  }
  // success=true 但 data 为空且有 error（如 stage_empty）→ 抛出
  if (result.data === undefined || result.data === null) {
    if (result.error) {
      throw new CommandError(result.error);
    }
    throw new CommandError({
      error_code: 'unknown',
      message: '返回数据缺失',
      recoverable: false,
    });
  }
  return result.data;
}
