import type { UiError } from '../workspaceUiTypes';

export interface SessionStatusIndicatorProps {
  status: 'unsaved' | 'saving' | 'saved' | 'error';
  error?: UiError | null;
  lastSavedAt?: string | null;
  onSave?: () => void;
  onRetry?: () => void;
}

const STATUS_CONFIG = {
  unsaved: {
    label: '未保存',
    dotColor: '#9e9e9e',
    textColor: '#666',
  },
  saving: {
    label: '保存中...',
    dotColor: '#1976d2',
    textColor: '#1976d2',
  },
  saved: {
    label: '已保存',
    dotColor: '#4caf50',
    textColor: '#2e7d32',
  },
  error: {
    label: '保存失败',
    dotColor: '#c62828',
    textColor: '#c62828',
  },
};

export default function SessionStatusIndicator({
  status,
  error,
  lastSavedAt,
  onSave,
  onRetry,
}: SessionStatusIndicatorProps) {
  const config = STATUS_CONFIG[status];

  const tooltip = lastSavedAt
    ? `最后保存于 ${new Date(lastSavedAt).toLocaleString('zh-CN')}${error ? `\n${error.error_code}: ${error.message}` : ''}`
    : error
      ? `${error.error_code}: ${error.message}`
      : undefined;

  return (
    <div
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        fontSize: 13,
        color: config.textColor,
      }}
      title={tooltip}
    >
      {status === 'saving' ? (
        <span
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            border: `2px solid ${config.dotColor}`,
            borderTopColor: 'transparent',
            animation: 'session-spin 1s linear infinite',
          }}
        />
      ) : (
        <span
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            background: config.dotColor,
          }}
        />
      )}
      <span>{config.label}</span>
      {status === 'unsaved' && onSave && (
        <button
          onClick={onSave}
          style={{
            padding: '2px 8px',
            borderRadius: 4,
            border: '1px solid #ccc',
            background: '#fff',
            cursor: 'pointer',
            fontSize: 12,
            color: '#333',
          }}
        >
          保存
        </button>
      )}
      {status === 'error' && onRetry && (
        <button
          onClick={onRetry}
          style={{
            padding: '2px 8px',
            borderRadius: 4,
            border: '1px solid #c62828',
            background: '#fff',
            cursor: 'pointer',
            fontSize: 12,
            color: '#c62828',
          }}
        >
          重试
        </button>
      )}
      <style>{`
        @keyframes session-spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
