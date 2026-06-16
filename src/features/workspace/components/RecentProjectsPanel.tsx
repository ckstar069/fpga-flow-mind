import { useState } from 'react';
import type { SessionSummary } from '../../../types/workspace';
import ConfirmDialog from './ConfirmDialog';
import { NAV, ACCENT, FONT } from './workbenchTheme';

export interface RecentProjectsPanelProps {
  sessions: SessionSummary[];
  loading: boolean;
  disabled: boolean;
  loadingSessionId: string | null;
  onLoad: (sessionId: string) => void;
  onDelete: (sessionId: string) => void;
  onOpenOtherProject: () => void;
}

function formatDate(iso: string): string {
  try {
    const d = new Date(iso);
    return d.toLocaleString('zh-CN', {
      month: 'short',
      day: 'numeric',
      hour: '2-digit',
      minute: '2-digit',
    });
  } catch {
    return iso;
  }
}

export default function RecentProjectsPanel({
  sessions,
  loading,
  disabled,
  loadingSessionId,
  onLoad,
  onDelete,
  onOpenOtherProject,
}: RecentProjectsPanelProps) {
  const [confirmSessionId, setConfirmSessionId] = useState<string | null>(null);

  const confirmSession = sessions.find((s) => s.session_id === confirmSessionId);

  return (
    <div>
      <h3
        style={{
          fontSize: FONT.micro,
          margin: '0 0 8px',
          color: NAV.textDim,
          fontWeight: 600,
          letterSpacing: 0.5,
          textTransform: 'uppercase',
        }}
      >
        最近项目
      </h3>

      {loading && sessions.length === 0 && (
        <div
          style={{
            padding: 16,
            background: NAV.surface,
            borderRadius: 8,
            textAlign: 'center',
            color: NAV.textMuted,
            fontSize: FONT.caption,
            border: `1px solid ${NAV.border}`,
          }}
        >
          加载中...
        </div>
      )}

      {!loading && sessions.length === 0 && (
        <div
          style={{
            padding: 16,
            background: NAV.surface,
            borderRadius: 8,
            textAlign: 'center',
            color: NAV.textMuted,
            fontSize: FONT.caption,
            border: `1px solid ${NAV.border}`,
          }}
        >
          <p style={{ margin: '0 0 6px' }}>暂无最近项目</p>
          <p style={{ margin: 0, color: NAV.textDim, fontSize: FONT.micro }}>
            点击下方"打开其他项目"开始分析。
          </p>
        </div>
      )}

      {sessions.length > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
          {sessions.map((session) => {
            const isLoading = loadingSessionId === session.session_id;
            return (
              <div
                key={session.session_id}
                style={{
                  display: 'flex',
                  alignItems: 'stretch',
                  gap: 6,
                }}
              >
                <button
                  onClick={() => !disabled && !isLoading && onLoad(session.session_id)}
                  disabled={disabled || isLoading}
                  style={{
                    flex: 1,
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 3,
                    padding: '8px 10px',
                    background: disabled ? NAV.bgSubtle : NAV.surface,
                    border: `1px solid ${NAV.border}`,
                    borderRadius: 6,
                    cursor: disabled || isLoading ? 'not-allowed' : 'pointer',
                    textAlign: 'left',
                    opacity: disabled || isLoading ? 0.7 : 1,
                    color: NAV.text,
                    minWidth: 0,
                  }}
                  onMouseEnter={(e) => {
                    if (!disabled && !isLoading) e.currentTarget.style.background = NAV.bgHover;
                  }}
                  onMouseLeave={(e) => {
                    if (!disabled && !isLoading) e.currentTarget.style.background = NAV.surface;
                  }}
                  title={session.root_path}
                >
                  <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                    {isLoading && (
                      <span
                        style={{
                          width: 10,
                          height: 10,
                          borderRadius: '50%',
                          border: `2px solid ${ACCENT.blue}`,
                          borderTopColor: 'transparent',
                          animation: 'session-spin 1s linear infinite',
                          flexShrink: 0,
                        }}
                      />
                    )}
                    <span
                      style={{
                        fontWeight: 600,
                        fontSize: FONT.body,
                        color: NAV.text,
                        overflow: 'hidden',
                        textOverflow: 'ellipsis',
                        whiteSpace: 'nowrap',
                      }}
                    >
                      {session.workspace_name || session.session_id}
                    </span>
                  </div>
                  <span
                    style={{
                      fontSize: FONT.micro,
                      color: NAV.textDim,
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {session.root_path}
                  </span>
                  <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
                    <span style={{ fontSize: FONT.micro, color: NAV.textDim }}>
                      {formatDate(session.updated_at)}
                    </span>
                    <span style={{ fontSize: FONT.micro, color: NAV.textDim }}>
                      {session.stage_count} 阶段
                    </span>
                  </div>
                </button>
                <button
                  onClick={() => setConfirmSessionId(session.session_id)}
                  disabled={disabled || isLoading}
                  title="删除此记录"
                  style={{
                    padding: '8px 10px',
                    borderRadius: 6,
                    border: '1px solid rgba(252, 165, 165, 0.3)',
                    background: 'rgba(198, 40, 40, 0.12)',
                    cursor: disabled || isLoading ? 'not-allowed' : 'pointer',
                    fontSize: FONT.micro,
                    color: '#fca5a5',
                    flexShrink: 0,
                  }}
                >
                  删除
                </button>
              </div>
            );
          })}
        </div>
      )}

      <button
        onClick={onOpenOtherProject}
        disabled={disabled}
        style={{
          marginTop: 10,
          padding: '7px 12px',
          borderRadius: 6,
          border: `1px solid ${NAV.border}`,
          background: 'transparent',
          cursor: disabled ? 'not-allowed' : 'pointer',
          fontSize: FONT.caption,
          color: disabled ? NAV.textDim : NAV.text,
          width: '100%',
        }}
      >
        打开其他项目...
      </button>

      {confirmSession && (
        <ConfirmDialog
          title="确定删除此记录？"
          confirmLabel="删除"
          danger
          onConfirm={() => {
            onDelete(confirmSession.session_id);
            setConfirmSessionId(null);
          }}
          onCancel={() => setConfirmSessionId(null)}
        >
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            <div>
              <span style={{ color: '#666' }}>项目名称：</span>{' '}
              {confirmSession.workspace_name || confirmSession.session_id}
            </div>
            <div>
              <span style={{ color: '#666' }}>目标路径：</span> {confirmSession.root_path}
            </div>
            <p style={{ margin: '8px 0 0', color: '#999', fontSize: 13 }}>
              此操作仅删除应用内的持久化记录，不会删除目标项目文件。
            </p>
          </div>
        </ConfirmDialog>
      )}

      <style>{`
        @keyframes session-spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
