import { useMemo } from 'react';
import type { EvidenceCollection, QualityReport } from '../../../types/workspace';

export interface EvidenceFilter {
  source_kind?: string;
  strength?: string;
  textQuery?: string;
}

export interface QualityFilter {
  severity?: string;
  kind?: string;
  status?: string;
}

interface StageFilterBarProps {
  activeTab: string;
  evidence?: EvidenceCollection;
  qualityReport?: QualityReport | null;
  evidenceFilter: EvidenceFilter;
  onEvidenceFilterChange: (filter: EvidenceFilter) => void;
  qualityFilter: QualityFilter;
  onQualityFilterChange: (filter: QualityFilter) => void;
}

export default function StageFilterBar({
  activeTab,
  evidence,
  qualityReport,
  evidenceFilter,
  onEvidenceFilterChange,
  qualityFilter,
  onQualityFilterChange,
}: StageFilterBarProps) {
  const evidenceKinds = useMemo(() => {
    if (!evidence) return [];
    const kinds = new Set(evidence.evidence_items.map((i) => i.source_kind));
    return Array.from(kinds).sort();
  }, [evidence]);

  const evidenceStrengths = useMemo(() => {
    if (!evidence) return [];
    const strengths = new Set(evidence.evidence_items.map((i) => i.strength));
    return Array.from(strengths).sort();
  }, [evidence]);

  const qualityKinds = useMemo(() => {
    if (!qualityReport) return [];
    const kinds = new Set(qualityReport.issues.map((i) => i.kind));
    return Array.from(kinds).sort();
  }, [qualityReport]);

  const qualitySeverities = useMemo(() => {
    if (!qualityReport) return [];
    const severities = new Set(qualityReport.issues.map((i) => i.severity));
    return Array.from(severities).sort();
  }, [qualityReport]);

  const qualityStatuses = useMemo(() => {
    if (!qualityReport) return [];
    const statuses = new Set(qualityReport.issues.map((i) => i.status));
    return Array.from(statuses).sort();
  }, [qualityReport]);

  if (activeTab === 'evidence') {
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <select
          value={evidenceFilter.source_kind ?? ''}
          onChange={(e) =>
            onEvidenceFilterChange({ ...evidenceFilter, source_kind: e.target.value || undefined })
          }
          style={selectStyle}
        >
          <option value="">全部 source kind</option>
          {evidenceKinds.map((k) => (
            <option key={k} value={k}>{k}</option>
          ))}
        </select>
        <select
          value={evidenceFilter.strength ?? ''}
          onChange={(e) =>
            onEvidenceFilterChange({ ...evidenceFilter, strength: e.target.value || undefined })
          }
          style={selectStyle}
        >
          <option value="">全部强度</option>
          {evidenceStrengths.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
        <input
          type="text"
          placeholder="搜索 evidence 摘要 / symbol"
          value={evidenceFilter.textQuery ?? ''}
          onChange={(e) =>
            onEvidenceFilterChange({ ...evidenceFilter, textQuery: e.target.value || undefined })
          }
          style={{ ...selectStyle, minWidth: 180 }}
        />
      </div>
    );
  }

  if (activeTab === 'quality') {
    return (
      <div style={{ display: 'flex', alignItems: 'center', gap: 10 }}>
        <select
          value={qualityFilter.severity ?? ''}
          onChange={(e) =>
            onQualityFilterChange({ ...qualityFilter, severity: e.target.value || undefined })
          }
          style={selectStyle}
        >
          <option value="">全部严重度</option>
          {qualitySeverities.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
        <select
          value={qualityFilter.kind ?? ''}
          onChange={(e) =>
            onQualityFilterChange({ ...qualityFilter, kind: e.target.value || undefined })
          }
          style={selectStyle}
        >
          <option value="">全部类别</option>
          {qualityKinds.map((k) => (
            <option key={k} value={k}>{k}</option>
          ))}
        </select>
        <select
          value={qualityFilter.status ?? ''}
          onChange={(e) =>
            onQualityFilterChange({ ...qualityFilter, status: e.target.value || undefined })
          }
          style={selectStyle}
        >
          <option value="">全部状态</option>
          {qualityStatuses.map((s) => (
            <option key={s} value={s}>{s}</option>
          ))}
        </select>
      </div>
    );
  }

  return (
    <div style={{ fontSize: 12, color: '#94a3b8' }}>
      {activeTab === 'overview'
        ? '选择 Artifact tab 以使用对应筛选'
        : '当前分区暂无可筛选对象'}
    </div>
  );
}

const selectStyle: React.CSSProperties = {
  padding: '4px 8px',
  fontSize: 12,
  borderRadius: 4,
  border: '1px solid #cbd5e1',
  background: '#fff',
  color: '#1e293b',
  outline: 'none',
};
