import type { ReactNode } from 'react';
import { SURFACE, ACCENT, FONT } from './workbenchTheme';

/**
 * StateBlocks: Phase 8 Batch D 统一的空/错误/提示状态组件。
 *
 * 目的：让 evidence 未收集、understanding 未生成、views 未生成、timing 诚实空、
 * session 加载失败、阶段加载失败、ContextPanel 无选择等空错态使用一致视觉语义，
 * 不出现审计裁决话术，不制造虚假成功态。
 *
 * 视觉语义：仅表达"工具状态/不确定性"，不评价目标项目正确性。
 */

interface EmptyStateProps {
  icon?: ReactNode;
  title: string;
  message?: ReactNode;
  actions?: ReactNode;
}

/** 统一空状态：图标 + 标题 + 说明 + 可选动作 */
export function EmptyState({ icon, title, message, actions }: EmptyStateProps) {
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        alignItems: 'center',
        justifyContent: 'center',
        textAlign: 'center',
        padding: 32,
        gap: 8,
        background: SURFACE.bgSubtle,
        borderRadius: 8,
        border: `1px solid ${SURFACE.border}`,
        color: SURFACE.textMuted,
      }}
    >
      {icon && <div style={{ fontSize: 28, lineHeight: 1 }}>{icon}</div>}
      <div style={{ fontSize: FONT.body, color: SURFACE.text, fontWeight: 500 }}>{title}</div>
      {message && (
        <div style={{ fontSize: FONT.caption, color: SURFACE.textMuted, maxWidth: 440, lineHeight: 1.6 }}>
          {message}
        </div>
      )}
      {actions && (
        <div style={{ display: 'flex', gap: 8, marginTop: 8, flexWrap: 'wrap', justifyContent: 'center' }}>
          {actions}
        </div>
      )}
    </div>
  );
}

type CalloutVariant = 'info' | 'warning' | 'error';

interface CalloutProps {
  variant?: CalloutVariant;
  icon?: ReactNode;
  title: string;
  message?: ReactNode;
  errorCode?: string;
  actions?: ReactNode;
}

const CALLOUT_STYLE: Record<
  CalloutVariant,
  { bg: string; border: string; color: string }
> = {
  info: { bg: ACCENT.blueSoft, border: ACCENT.blueSoftBorder, color: ACCENT.blueDark },
  warning: { bg: ACCENT.amberSoft, border: ACCENT.amberBorder, color: ACCENT.amber },
  error: { bg: ACCENT.redSoft, border: ACCENT.redBorder, color: ACCENT.red },
};

/**
 * 统一提示/注意/错误条。
 * - info（蓝）：说明性提示，如命名异常已识别、timing 诚实空图
 * - warning（琥珀）：源码变更、目标路径变化等需注意（非裁决）
 * - error（红/中性）：加载失败等错误态（保留 error_code 与可恢复语义）
 */
export function Callout({ variant = 'info', icon, title, message, errorCode, actions }: CalloutProps) {
  const s = CALLOUT_STYLE[variant];
  return (
    <div
      style={{
        padding: '12px 16px',
        background: s.bg,
        border: `1px solid ${s.border}`,
        borderRadius: 8,
        color: s.color,
      }}
    >
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          gap: 8,
          marginBottom: message ? 4 : 0,
        }}
      >
        <span style={{ display: 'flex', alignItems: 'center', gap: 8, fontWeight: 600, fontSize: FONT.heading }}>
          {icon && <span>{icon}</span>}
          {title}
        </span>
        {errorCode && (
          <code style={{ fontSize: FONT.micro, background: 'rgba(255,255,255,0.6)', padding: '1px 6px', borderRadius: 4 }}>
            {errorCode}
          </code>
        )}
      </div>
      {message && <div style={{ fontSize: FONT.body, opacity: 0.95, lineHeight: 1.5 }}>{message}</div>}
      {actions && (
        <div style={{ display: 'flex', gap: 8, marginTop: 8, flexWrap: 'wrap' }}>{actions}</div>
      )}
    </div>
  );
}
