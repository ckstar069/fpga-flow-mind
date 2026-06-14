import type { LoadSessionStatus } from '../../../types/workspace';

export interface LoadStatusBannerProps {
  status: LoadSessionStatus;
  onClose: () => void;
  onReanalyze?: () => void;
  onDelete?: () => void;
}

const CONFIG: Record<
  LoadSessionStatus,
  {
    background: string;
    border: string;
    color: string;
    title: string;
    message: string;
  }
> = {
  source_unchanged: {
    background: '#e8f5e9',
    border: '#a5d6a7',
    color: '#2e7d32',
    title: '已恢复上次状态',
    message: '项目未变更，历史产物可用。',
  },
  source_changed: {
    background: '#fff8e1',
    border: '#ffe082',
    color: '#f57c00',
    title: '项目文件已变更',
    message: '自上次分析以来目标项目文件已发生变化。历史产物可能不准确。',
  },
  source_missing: {
    background: '#fff8e1',
    border: '#ffe082',
    color: '#f57c00',
    title: '目标路径已不存在',
    message: '目标项目路径已不存在或无法访问，仅恢复历史产物。',
  },
  source_path_not_allowed: {
    background: '#ffebee',
    border: '#ef9a9a',
    color: '#c62828',
    title: '目标路径不安全',
    message: '目标路径存在安全风险，仅恢复历史产物。',
  },
};

export default function LoadStatusBanner({
  status,
  onClose,
  onReanalyze,
  onDelete,
}: LoadStatusBannerProps) {
  const cfg = CONFIG[status];

  return (
    <div
      style={{
        padding: '12px 16px',
        marginBottom: 16,
        background: cfg.background,
        border: `1px solid ${cfg.border}`,
        borderRadius: 8,
        color: cfg.color,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: 4,
        }}
      >
        <span style={{ fontWeight: 600, fontSize: 14 }}>{cfg.title}</span>
        <button
          onClick={onClose}
          style={{
            padding: '2px 8px',
            borderRadius: 4,
            border: `1px solid ${cfg.border}`,
            background: '#fff',
            cursor: 'pointer',
            fontSize: 12,
            color: '#666',
          }}
        >
          关闭
        </button>
      </div>
      <p style={{ margin: '0 0 8px', fontSize: 13 }}>{cfg.message}</p>
      <div style={{ display: 'flex', gap: 8 }}>
        {status === 'source_changed' && onReanalyze && (
          <button
            onClick={onReanalyze}
            style={{
              padding: '4px 12px',
              borderRadius: 4,
              border: '1px solid #f57c00',
              background: '#fff',
              cursor: 'pointer',
              fontSize: 12,
              color: '#f57c00',
            }}
          >
            重新分析
          </button>
        )}
        {(status === 'source_missing' || status === 'source_path_not_allowed') &&
          onDelete && (
            <button
              onClick={onDelete}
              style={{
                padding: '4px 12px',
                borderRadius: 4,
                border: '1px solid #c62828',
                background: '#fff',
                cursor: 'pointer',
                fontSize: 12,
                color: '#c62828',
              }}
            >
              删除此记录
            </button>
          )}
      </div>
    </div>
  );
}
