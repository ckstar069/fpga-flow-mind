import SessionStatusIndicator from './SessionStatusIndicator';
import type { SessionStatusIndicatorProps } from './SessionStatusIndicator';
import type { UiError } from '../workspaceUiTypes';

interface AppHeaderProps {
  pathInput: string;
  setPathInput: (value: string) => void;
  onOpen: () => void;
  isOpening: boolean;
  isLoadingSession: boolean;
  saveStatus: SessionStatusIndicatorProps['status'];
  saveError?: UiError | null;
  lastSavedAt?: string | null;
  onSave: () => void;
}

/**
 * AppHeader: 顶部工具栏
 * 承载产品名、项目路径输入、打开项目按钮、session 保存状态。
 */
export default function AppHeader({
  pathInput,
  setPathInput,
  onOpen,
  isOpening,
  isLoadingSession,
  saveStatus,
  saveError,
  lastSavedAt,
  onSave,
}: AppHeaderProps) {
  return (
    <header
      className="app-header"
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 16,
        padding: '12px 20px',
        background: '#fff',
        borderBottom: '1px solid #e2e8f0',
        flexShrink: 0,
      }}
    >
      <h1
        className="app-header-title"
        style={{
          fontSize: 18,
          fontWeight: 600,
          margin: 0,
          color: '#1e293b',
          whiteSpace: 'nowrap',
        }}
      >
        fpga-flow-mind
      </h1>
      <div
        className="app-header-input-group"
        style={{
          display: 'flex',
          gap: 8,
          flex: 1,
          minWidth: 0,
        }}
      >
        <input
          id="workspace-path-input"
          type="text"
          placeholder="输入项目路径..."
          value={pathInput}
          onChange={(e) => setPathInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && onOpen()}
          disabled={isLoadingSession}
          style={{
            flex: 1,
            padding: '6px 12px',
            border: '1px solid #cbd5e1',
            borderRadius: 6,
            fontSize: 14,
            background: isLoadingSession ? '#f1f5f9' : '#fff',
            color: '#1e293b',
            minWidth: 120,
          }}
        />
        <button
          onClick={onOpen}
          disabled={isOpening || isLoadingSession}
          style={{
            padding: '6px 16px',
            borderRadius: 6,
            border: '1px solid #1976d2',
            cursor: isOpening || isLoadingSession ? 'not-allowed' : 'pointer',
            background: isOpening || isLoadingSession ? '#e2e8f0' : '#1976d2',
            color: isOpening || isLoadingSession ? '#64748b' : '#fff',
            fontSize: 14,
            fontWeight: 500,
            whiteSpace: 'nowrap',
          }}
        >
          {isOpening ? '扫描中...' : '打开项目'}
        </button>
      </div>
      <SessionStatusIndicator
        status={saveStatus}
        error={saveError}
        lastSavedAt={lastSavedAt}
        onSave={onSave}
        onRetry={onSave}
      />
    </header>
  );
}
