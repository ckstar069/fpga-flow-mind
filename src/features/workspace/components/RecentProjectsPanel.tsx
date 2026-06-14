import { useState } from 'react';
import type { SessionSummary } from '../../../types/workspace';
import ConfirmDialog from './ConfirmDialog';

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
    <div style={{ marginTop: 24 }}>
      <h3 style={{ fontSize: 14, margin: '0 0 12px' }}>最近项目</h3>

      {loading && sessions.length === 0 && (
        <div
          style={{
            padding: 16,
            background: '#fafafa',
            borderRadius: 8,
            textAlign: 'center',
            color: '#999',
            fontSize: 13,
          }}
        >
          加载中...
        </div>
      )}

      {!loading && sessions.length === 0 && (
        <div
          style={{
            padding: 16,
            background: '#fafafa',
            borderRadius: 8,
            textAlign: 'center',
            color: '#666',
            fontSize: 13,
          }}
        >
          <p style={{ margin: '0 0 8px' }}>暂无最近项目</p>
          <p style={{ margin: 0, color: '#999', fontSize: 12 }}>
            点击“打开其他项目”开始分析。
          </p>
        </div>
      )}

      {sessions.length > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {sessions.map((session) => {
            const isLoading = loadingSessionId === session.session_id;
            return (
              <div
                key={session.session_id}
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                }}
              >
                <button
                  onClick={() => !disabled && !isLoading && onLoad(session.session_id)}
                  disabled={disabled || isLoading}
                  style={{
                    flex: 1,
                    display: 'flex',
                    flexDirection: 'column',
                    gap: 4,
                    padding: '10px 12px',
                    background: disabled ? '#f5f5f5' : '#fff',
                    border: '1px solid #e0e0e0',
                    borderRadius: 6,
                    cursor: disabled || isLoading ? 'not-allowed' : 'pointer',
                    textAlign: 'left',
                    opacity: disabled || isLoading ? 0.7 : 1,
                  }}
                  title={session.root_path}
                >
                  <div
                    style={{
                      display: 'flex',
                      alignItems: 'center',
                      gap: 8,
                    }}
                  >
                    {isLoading && (
                      <span
                        style={{
                          width: 10,
                          height: 10,
                          borderRadius: '50%',
                          border: '2px solid #1976d2',
                          borderTopColor: 'transparent',
                          animation: 'session-spin 1s linear infinite',
                        }}
                      />
                    )}
                    <span
                      style={{
                        fontWeight: 600,
                        fontSize: 14,
                        color: '#333',
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
                      fontSize: 12,
                      color: '#666',
                      overflow: 'hidden',
                      textOverflow: 'ellipsis',
                      whiteSpace: 'nowrap',
                    }}
                  >
                    {session.root_path}
                  </span>
                  <div
                    style={{
                      display: 'flex',
                      justifyContent: 'space-between',
                      alignItems: 'center',
                    }}
                  >
                    <span style={{ fontSize: 11, color: '#999' }}>
                      {formatDate(session.updated_at)}
                    </span>
                    <span style={{ fontSize: 11, color: '#999' }}>
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
                    border: '1px solid #e0e0e0',
                    background: '#fff',
                    cursor: disabled || isLoading ? 'not-allowed' : 'pointer',
                    fontSize: 12,
                    color: '#c62828',
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
          marginTop: 12,
          padding: '8px 12px',
          borderRadius: 6,
          border: '1px solid #ccc',
          background: '#fff',
          cursor: disabled ? 'not-allowed' : 'pointer',
          fontSize: 13,
          color: disabled ? '#999' : '#333',
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
              <span style={{ color: '#666' }}>目标路径：</span>{' '}
              {confirmSession.root_path}
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
