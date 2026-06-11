import { invoke } from '@tauri-apps/api/core';
import type { CommandResult, WorkspaceProfile, StageContext } from '../types/workspace';

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
  if (result.data === undefined || result.data === null) {
    throw new CommandError({
      error_code: 'unknown',
      message: '返回数据缺失',
      recoverable: false,
    });
  }
  return result.data;
}
