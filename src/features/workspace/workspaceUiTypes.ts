// 本地 UI 错误类型（不写入后端契约）
// 仅用于前端 UI 层展示异常，不进入 Rust CommandResult.error 或 WorkspaceProfile.error_codes
import type { CommandError } from '../../types/workspace';

export type UiError =
  | CommandError
  | {
      error_code: 'frontend_error';
      message: string;
      recoverable: false;
      details?: string;
      source_path?: string;
    };
