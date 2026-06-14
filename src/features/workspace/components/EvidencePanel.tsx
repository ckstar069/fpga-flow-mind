import { useEffect, useRef } from 'react';
import type { EvidenceCollection, EvidenceItem, EvidenceStrength } from '../../../types/workspace';

// ─── strength 标签映射 ───
const STRENGTH_LABEL: Record<EvidenceStrength, string> = {
  direct: '直接',
  indirect: '间接',
  weak: '弱',
  conflicting: '冲突',
  missing: '缺失',
};

const STRENGTH_COLOR: Record<EvidenceStrength, string> = {
  direct: '#4caf50',
  indirect: '#2196f3',
  weak: '#ff9800',
  conflicting: '#f44336',
  missing: '#9e9e9e',
};

interface EvidencePanelProps {
  evidence: EvidenceCollection;
  highlightedEvidenceId?: string;
  currentSourceEvidenceId?: string;
  onEvidenceSelect?: (evidenceId: string) => void;
}

export default function EvidencePanel({
  evidence,
  highlightedEvidenceId,
  currentSourceEvidenceId,
  onEvidenceSelect,
}: EvidencePanelProps) {
  const { stats, warnings, evidence_items } = evidence;

  return (
    <div style={{ marginBottom: 24 }}>
      <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>
        证据结果 ({evidence_items.length} 项)
      </h3>

      {/* 统计栏 */}
      <div
        style={{
          display: 'flex',
          gap: 16,
          padding: '10px 16px',
          background: '#f5f5f5',
          borderRadius: 6,
          marginBottom: 12,
          fontSize: 13,
        }}
      >
        <span>
          处理文件: <strong>{stats.files_processed}</strong>
        </span>
        <span>
          跳过文件: <strong>{stats.files_skipped}</strong>
        </span>
        <span>
          证据项: <strong>{stats.total_items}</strong>
        </span>
      </div>

      {/* 类型分组 */}
      {Object.keys(stats.items_by_kind).length > 0 && (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 12 }}>
          {Object.entries(stats.items_by_kind).map(([kind, count]) => (
            <span
              key={kind}
              style={{
                padding: '3px 10px',
                background: '#e8eaf6',
                borderRadius: 4,
                fontSize: 12,
              }}
            >
              {kind} ({count})
            </span>
          ))}
        </div>
      )}

      {/* 强度分组 */}
      {Object.keys(stats.items_by_strength).length > 0 && (
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 6, marginBottom: 16 }}>
          {Object.entries(stats.items_by_strength).map(([strength, count]) => {
            const color =
              STRENGTH_COLOR[strength as EvidenceStrength] ?? '#9e9e9e';
            const label =
              STRENGTH_LABEL[strength as EvidenceStrength] ?? strength;
            return (
              <span
                key={strength}
                style={{
                  padding: '3px 10px',
                  background: color,
                  color: '#fff',
                  borderRadius: 4,
                  fontSize: 12,
                }}
              >
                {label} ({count})
              </span>
            );
          })}
        </div>
      )}

      {/* 警告区 */}
      {warnings.length > 0 && (
        <div
          style={{
            padding: 12,
            background: '#fff8e1',
            borderRadius: 6,
            marginBottom: 16,
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 13 }}>
            警告 ({warnings.length})
          </h4>
          <ul style={{ margin: 0, paddingLeft: 20, fontSize: 12 }}>
            {warnings.map((w, i) => (
              <li key={i} style={{ marginBottom: 2 }}>
                <code>{w.error_code}</code>: {w.message}
                {w.source_path && (
                  <span style={{ color: '#999', marginLeft: 6 }}>
                    ({w.source_path})
                  </span>
                )}
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* 空状态 */}
      {evidence_items.length === 0 && (
        <div
          style={{
            padding: 32,
            background: '#fafafa',
            borderRadius: 8,
            textAlign: 'center',
            color: '#999',
          }}
        >
          <p style={{ margin: '0 0 8px', fontSize: 15 }}>未收集到证据</p>
          <p style={{ margin: 0, fontSize: 13 }}>
            该阶段可能无可提取的结构信息。
            {stats.files_processed > 0 && (
              <>收集了 {stats.files_processed} 个文件，跳过了 {stats.files_skipped} 个文件。</>
            )}
          </p>
        </div>
      )}

      {/* 证据列表 */}
      {evidence_items.length > 0 && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {evidence_items.map((item) => (
            <EvidenceItemCard
              key={item.evidence_id}
              item={item}
              highlighted={highlightedEvidenceId === item.evidence_id}
              currentSource={currentSourceEvidenceId === item.evidence_id}
              onSelect={onEvidenceSelect}
            />
          ))}
        </div>
      )}
    </div>
  );
}

// ─── 证据项卡片 ───
function EvidenceItemCard({
  item,
  highlighted,
  currentSource,
  onSelect,
}: {
  item: EvidenceItem;
  highlighted?: boolean;
  currentSource?: boolean;
  onSelect?: (evidenceId: string) => void;
}) {
  const cardRef = useRef<HTMLDivElement>(null);
  const strengthColor =
    STRENGTH_COLOR[item.strength as EvidenceStrength] ?? '#9e9e9e';
  const strengthLabel =
    STRENGTH_LABEL[item.strength as EvidenceStrength] ?? item.strength;

  // 高亮时滚动到该元素
  useEffect(() => {
    if (highlighted && cardRef.current) {
      cardRef.current.scrollIntoView({ behavior: 'smooth', block: 'center' });
    }
  }, [highlighted]);

  return (
    <div
      ref={cardRef}
      onClick={() => onSelect?.(item.evidence_id)}
      style={{
        padding: '10px 14px',
        background: highlighted ? '#fff9c4' : currentSource ? '#e3f2fd' : '#fff',
        borderRadius: 6,
        border: '1px solid #e0e0e0',
        borderLeft: currentSource ? '4px solid #1976d2' : highlighted ? '4px solid #fbc02d' : '1px solid #e0e0e0',
        cursor: onSelect ? 'pointer' : 'default',
        transition: 'background 0.2s',
      }}
    >
      {/* 顶行：ID + strength badge */}
      <div
        style={{
          display: 'flex',
          justifyContent: 'space-between',
          alignItems: 'center',
          marginBottom: 6,
        }}
      >
        <code style={{ fontSize: 12, color: '#666' }}>{item.evidence_id}</code>
        <span
          style={{
            padding: '2px 8px',
            background: strengthColor,
            color: '#fff',
            borderRadius: 3,
            fontSize: 11,
          }}
        >
          {strengthLabel}
        </span>
      </div>

      {/* symbol */}
      {item.symbol && (
        <div style={{ fontSize: 14, fontWeight: 600, marginBottom: 4 }}>
          {item.symbol}
        </div>
      )}

      {/* summary */}
      <div style={{ fontSize: 13, color: '#333', marginBottom: 6 }}>
        {item.summary}
      </div>

      {/* 底行：文件名 + 行号 + 语言/类型 */}
      <div style={{ fontSize: 11, color: '#999' }}>
        <span title={item.source_path} style={{ cursor: 'help' }}>
          {item.source_path.split('/').pop() || item.source_path}
        </span>
        {' · '}
        <span>
          行 {item.line_range.start}–{item.line_range.end}
        </span>
        {' · '}
        <span>
          {item.language} / {item.source_kind}
        </span>
      </div>
    </div>
  );
}
