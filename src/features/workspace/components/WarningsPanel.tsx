import { useMemo, useState } from 'react';
import type { WorkspaceWarning } from '../../../types/workspace';
import { SURFACE, ACCENT, FONT } from './workbenchTheme';

interface WarningsPanelProps {
  warnings: WorkspaceWarning[];
}

/**
 * WarningsPanel: Phase 8 Batch D warnings 降噪组件。
 *
 * 替代旧版底部长条 warnings 区。产品化要求：
 * - 折叠/分类/计数/可展开，默认不长时间占据底部大面积
 * - 按 error_code 聚合，展示 count + 分类徽标
 * - 展开查看单条明细（含 source_path）
 * - 只读，不自动隐藏关键异常，不删除原始 warning
 * - 不写成"正确/错误裁决"，仅表达"工具在扫描/读取时的提示与限制"
 *
 * warnings 为空时不渲染（不占空间）。
 */
export default function WarningsPanel({ warnings }: WarningsPanelProps) {
  const [expanded, setExpanded] = useState(false);

  const groups = useMemo(() => {
    const map = new Map<string, WorkspaceWarning[]>();
    for (const w of warnings) {
      const key = w.error_code || 'unknown';
      const list = map.get(key);
      if (list) list.push(w);
      else map.set(key, [w]);
    }
    return Array.from(map.entries()).sort(([a], [b]) => a.localeCompare(b));
  }, [warnings]);

  if (warnings.length === 0) return null;

  const maxBadges = 4;
  const visibleGroups = groups.slice(0, maxBadges);
  const hiddenGroupCount = groups.length - visibleGroups.length;

  return (
    <footer
      className="warnings-panel"
      style={{
        flexShrink: 0,
        borderTop: `1px solid ${SURFACE.border}`,
        background: ACCENT.amberSoft,
      }}
    >
      {/* 折叠条（始终可见，紧凑） */}
      <button
        onClick={() => setExpanded((v) => !v)}
        style={{
          width: '100%',
          display: 'flex',
          alignItems: 'center',
          gap: 10,
          padding: '6px 20px',
          background: 'transparent',
          border: 'none',
          cursor: 'pointer',
          textAlign: 'left',
          color: ACCENT.amber,
          fontSize: FONT.caption,
        }}
      >
        <span style={{ fontWeight: 700 }}>⚠</span>
        <span style={{ fontWeight: 600 }}>
          扫描/读取提示 {warnings.length} 条
        </span>
        <span style={{ display: 'flex', gap: 6, flexWrap: 'wrap', alignItems: 'center' }}>
          {visibleGroups.map(([code, list]) => (
            <span
              key={code}
              style={{
                padding: '1px 7px',
                background: 'rgba(245, 124, 0, 0.12)',
                border: `1px solid ${ACCENT.amberBorder}`,
                borderRadius: 10,
                fontSize: FONT.micro,
                color: ACCENT.amber,
                whiteSpace: 'nowrap',
              }}
            >
              <code>{code}</code> ×{list.length}
            </span>
          ))}
          {hiddenGroupCount > 0 && (
            <span style={{ fontSize: FONT.micro, color: SURFACE.textDim }}>
              +{hiddenGroupCount} 类
            </span>
          )}
        </span>
        <span style={{ marginLeft: 'auto', fontSize: FONT.micro, color: SURFACE.textDim }}>
          {expanded ? '收起 ▾' : '展开 ▸'}
        </span>
      </button>

      {/* 展开态：按分类聚合的明细 */}
      {expanded && (
        <div
          style={{
            maxHeight: 240,
            overflowY: 'auto',
            padding: '4px 20px 10px',
            display: 'flex',
            flexDirection: 'column',
            gap: 8,
          }}
        >
          {groups.map(([code, list]) => (
            <div
              key={code}
              style={{
                background: SURFACE.bg,
                border: `1px solid ${SURFACE.border}`,
                borderRadius: 6,
                padding: 8,
              }}
            >
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  gap: 8,
                  marginBottom: list.length > 1 ? 4 : 0,
                }}
              >
                <code style={{ fontSize: FONT.micro, color: ACCENT.amber, fontWeight: 600 }}>
                  {code}
                </code>
                <span style={{ fontSize: FONT.micro, color: SURFACE.textDim }}>
                  ({list.length})
                </span>
              </div>
              <ul style={{ margin: 0, paddingLeft: 16, display: 'flex', flexDirection: 'column', gap: 2 }}>
                {list.map((w, i) => (
                  <li key={i} style={{ fontSize: FONT.caption, color: SURFACE.textMuted, lineHeight: 1.5 }}>
                    {w.message}
                    {w.source_path && (
                      <span style={{ color: SURFACE.textDim, marginLeft: 6 }}>
                        (<code style={{ fontSize: FONT.micro }}>{w.source_path}</code>)
                      </span>
                    )}
                  </li>
                ))}
              </ul>
            </div>
          ))}
        </div>
      )}
    </footer>
  );
}
