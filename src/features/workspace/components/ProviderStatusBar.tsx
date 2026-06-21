import type { KeyboardEvent } from 'react';
import type { ProviderStatus, ProviderStatusResponse } from '../../../types/workspace';
import { ACCENT, FONT } from './workbenchTheme';

export interface ProviderStatusBarProps {
  status: ProviderStatusResponse | null;
  loading?: boolean;
  error?: string | null;
  onConfigureClick: () => void;
  onRetry?: () => void;
}

const STATUS_CONFIG: Record<
  ProviderStatus | 'loading' | 'error' | 'uninitialized',
  { label: string; dotColor: string; textColor: string }
> = {
  uninitialized: {
    label: '未检测',
    dotColor: ACCENT.slate,
    textColor: ACCENT.slate,
  },
  loading: {
    label: '检测中...',
    dotColor: ACCENT.amber,
    textColor: ACCENT.amber,
  },
  error: {
    label: '状态获取失败',
    dotColor: ACCENT.amber,
    textColor: ACCENT.amber,
  },
  mock: {
    label: 'Mock · 本地模式',
    dotColor: ACCENT.slate,
    textColor: ACCENT.slate,
  },
  real: {
    label: '真实 LLM',
    dotColor: ACCENT.blue,
    textColor: ACCENT.blue,
  },
  degraded: {
    label: '降级',
    dotColor: ACCENT.amber,
    textColor: ACCENT.amber,
  },
  unknown: {
    label: '未知',
    dotColor: ACCENT.slate,
    textColor: ACCENT.slate,
  },
};

function degradedReasonText(reason?: string): string {
  switch (reason) {
    case 'network_disabled':
      return '网络已禁用';
    case 'not_configured':
      return '未配置';
    case 'provider_error':
      return 'Provider 异常';
    case 'cancelled':
      return '已取消';
    case 'grounding_failed':
      return 'grounding 校验失败';
    default:
      return '';
  }
}

export default function ProviderStatusBar({
  status,
  loading,
  error,
  onConfigureClick,
  onRetry,
}: ProviderStatusBarProps) {
  let key: keyof typeof STATUS_CONFIG = 'uninitialized';
  if (loading) {
    key = 'loading';
  } else if (error) {
    key = 'error';
  } else if (status) {
    key = status.status;
  }

  const config = STATUS_CONFIG[key];
  const reasonText = status?.degraded_reason ? degradedReasonText(status.degraded_reason) : '';
  const label = reasonText ? `${config.label} · ${reasonText}` : config.label;
  const handleKeyDown = (event: KeyboardEvent<HTMLDivElement>) => {
    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault();
      onConfigureClick();
    }
  };

  return (
    <div
      role="button"
      tabIndex={0}
      onClick={onConfigureClick}
      onKeyDown={handleKeyDown}
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 8,
        fontSize: FONT.caption,
        color: config.textColor,
        background: 'transparent',
        border: 'none',
        cursor: 'pointer',
        padding: '4px 8px',
        borderRadius: 4,
      }}
      title="点击配置 LLM Provider"
    >
      {loading ? (
        <span
          style={{
            width: 8,
            height: 8,
            borderRadius: '50%',
            border: `2px solid ${config.dotColor}`,
            borderTopColor: 'transparent',
            animation: 'provider-spin 1s linear infinite',
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
      <span>{label}</span>
      {key === 'error' && onRetry && (
        <button
          type="button"
          onClick={(e) => {
            e.stopPropagation();
            onRetry();
          }}
          style={{
            marginLeft: 8,
            padding: '2px 6px',
            borderRadius: 3,
            background: ACCENT.slateSoft,
            color: ACCENT.slate,
            fontSize: 11,
            cursor: 'pointer',
            border: 'none',
          }}
        >
          重试
        </button>
      )}
      <style>{`
        @keyframes provider-spin {
          to { transform: rotate(360deg); }
        }
      `}</style>
    </div>
  );
}
