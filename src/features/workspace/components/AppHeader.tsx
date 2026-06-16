import SessionStatusIndicator from './SessionStatusIndicator';
import type { SessionStatusIndicatorProps } from './SessionStatusIndicator';
import type { UiError } from '../workspaceUiTypes';
import { SURFACE, ACCENT, FONT } from './workbenchTheme';

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
 * AppHeader: 工作台顶部条。
 * 承载产品标识、项目路径输入、打开项目按钮、session 保存状态。
 * Batch D：更像"工作台 header"——品牌标识 + 副标题，蓝色强调操作。
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
  const inputDisabled = isLoadingSession;
  const buttonDisabled = isOpening || isLoadingSession;
  return (
    <header
      className="app-header"
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 16,
        padding: '10px 20px',
        background: SURFACE.bg,
        borderBottom: `1px solid ${SURFACE.border}`,
        flexShrink: 0,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 10, flexShrink: 0 }}>
        <span
          style={{
            width: 26,
            height: 26,
            borderRadius: 6,
            background: `linear-gradient(135deg, ${ACCENT.blue}, ${ACCENT.blueDark})`,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            color: '#fff',
            fontSize: 15,
            fontWeight: 700,
          }}
        >
          f
        </span>
        <div style={{ display: 'flex', flexDirection: 'column', lineHeight: 1.2 }}>
          <h1
            className="app-header-title"
            style={{
              fontSize: 15,
              fontWeight: 600,
              margin: 0,
              color: SURFACE.text,
              whiteSpace: 'nowrap',
            }}
          >
            fpga-flow-mind
          </h1>
          <span style={{ fontSize: 10, color: SURFACE.textDim, whiteSpace: 'nowrap' }}>
            FPGA 阶段理解工作台
          </span>
        </div>
      </div>
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
          placeholder="输入目标项目路径..."
          value={pathInput}
          onChange={(e) => setPathInput(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && onOpen()}
          disabled={inputDisabled}
          style={{
            flex: 1,
            padding: '7px 12px',
            border: `1px solid ${SURFACE.borderStrong}`,
            borderRadius: 6,
            fontSize: FONT.body,
            background: inputDisabled ? SURFACE.bgSubtle : SURFACE.bg,
            color: SURFACE.text,
            minWidth: 120,
            outline: 'none',
            transition: 'border-color 0.12s',
          }}
          onFocus={(e) => (e.currentTarget.style.borderColor = ACCENT.blue)}
          onBlur={(e) => (e.currentTarget.style.borderColor = SURFACE.borderStrong)}
        />
        <button
          onClick={onOpen}
          disabled={buttonDisabled}
          style={{
            padding: '7px 18px',
            borderRadius: 6,
            border: 'none',
            cursor: buttonDisabled ? 'not-allowed' : 'pointer',
            background: buttonDisabled ? SURFACE.borderStrong : ACCENT.blue,
            color: buttonDisabled ? SURFACE.textDim : '#fff',
            fontSize: FONT.body,
            fontWeight: 500,
            whiteSpace: 'nowrap',
            transition: 'background 0.12s',
          }}
          onMouseEnter={(e) => {
            if (!buttonDisabled) e.currentTarget.style.background = ACCENT.blueDark;
          }}
          onMouseLeave={(e) => {
            if (!buttonDisabled) e.currentTarget.style.background = ACCENT.blue;
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
