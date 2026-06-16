import { useEffect, useState, type ReactNode } from 'react';
import type {
  WorkspaceProfile,
  StageContext,
  StageStatus,
  EvidenceCollection,
  ImplementationUnderstanding,
  ViewGraph,
  QaHistory,
  QualityReport,
} from '../../../types/workspace';
import StageOverviewBar from './StageOverviewBar';
import StageFilterBar, { type EvidenceFilter, type QualityFilter } from './StageFilterBar';
import ContextPanel from './ContextPanel';
import type { ContextSelection } from './contextPanelTypes';
import { SURFACE, ACCENT, FONT } from './workbenchTheme';

const INITIAL_EVIDENCE_FILTER: EvidenceFilter = {};
const INITIAL_QUALITY_FILTER: QualityFilter = {};

type ArtifactTab =
  | 'overview'
  | 'evidence'
  | 'understanding'
  | 'views'
  | 'trace'
  | 'qa'
  | 'quality';

const ARTIFACT_TAB_LABELS: { key: ArtifactTab; label: string }[] = [
  { key: 'overview', label: '概览' },
  { key: 'evidence', label: '证据' },
  { key: 'understanding', label: '理解' },
  { key: 'views', label: '视图' },
  { key: 'trace', label: '追溯' },
  { key: 'qa', label: 'Q&A' },
  { key: 'quality', label: '质量' },
];

interface StageWorkspaceProps {
  profile: WorkspaceProfile;
  stageId: string;
  context: StageContext;
  stageStatus: StageStatus;
  evidence?: EvidenceCollection;
  evidenceLoading?: boolean;
  understanding?: ImplementationUnderstanding;
  understandingLoading?: boolean;
  views?: ViewGraph[];
  viewsLoading?: boolean;
  qaHistory?: QaHistory;
  qaLoading?: boolean;
  qualityReport?: QualityReport | null;
  qualityLoading?: boolean;
  contextSelection?: ContextSelection | null;
  onContextSelectionChange?: (selection: ContextSelection | null) => void;
  onViewSource?: (location: {
    source_path: string;
    line_range: { start: number; end: number };
    evidence_id?: string;
  }) => void;
  onLocateEvidence?: (evidenceId: string) => void;
  renderContent: (state: {
    activeTab: ArtifactTab;
    evidenceFilter: EvidenceFilter;
    qualityFilter: QualityFilter;
    onContextSelectionChange: (selection: ContextSelection | null) => void;
  }) => ReactNode;
}

/**
 * StageWorkspace: 阶段工作区骨架。
 * Batch D：统一设计 token，顶部阶段 header + 概览 + 筛选 + Artifact tabs + 内容 + ContextPanel。
 */
export default function StageWorkspace({
  profile,
  stageId,
  context,
  stageStatus,
  evidence,
  evidenceLoading,
  understanding,
  understandingLoading,
  views,
  viewsLoading,
  qaHistory,
  qaLoading,
  qualityReport,
  qualityLoading,
  contextSelection,
  onContextSelectionChange,
  onViewSource,
  onLocateEvidence,
  renderContent,
}: StageWorkspaceProps) {
  const [activeTab, setActiveTab] = useState<ArtifactTab>('overview');
  const [evidenceFilter, setEvidenceFilter] = useState<EvidenceFilter>(INITIAL_EVIDENCE_FILTER);
  const [qualityFilter, setQualityFilter] = useState<QualityFilter>(INITIAL_QUALITY_FILTER);

  // 切换阶段后自动回到概览 tab 并清空筛选
  useEffect(() => {
    setActiveTab('overview');
    setEvidenceFilter(INITIAL_EVIDENCE_FILTER);
    setQualityFilter(INITIAL_QUALITY_FILTER);
  }, [stageId]);

  // 阶段作用域校验：非当前阶段的选中上下文视为过期
  const validSelection = contextSelection?.stageId === stageId ? contextSelection : null;

  return (
    <div
      className="stage-workspace"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        background: SURFACE.bgSubtle,
        overflow: 'hidden',
      }}
    >
      {/* 阶段工作台 header */}
      <div
        className="stage-workspace-topbar"
        style={{
          display: 'flex',
          alignItems: 'baseline',
          gap: 12,
          padding: '10px 20px',
          background: SURFACE.bg,
          borderBottom: `1px solid ${SURFACE.border}`,
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: FONT.caption, color: SURFACE.textDim }}>{profile.workspace_name}</span>
        <span style={{ fontSize: FONT.caption, color: SURFACE.textDim }}>/</span>
        <span style={{ fontSize: FONT.title, fontWeight: 600, color: SURFACE.text }}>{stageId}</span>
        <span
          style={{
            fontSize: FONT.micro,
            color: SURFACE.textMuted,
            marginLeft: 'auto',
            wordBreak: 'break-all',
            maxWidth: '60%',
            textAlign: 'right',
          }}
          title={context.source_path}
        >
          {context.source_path}
        </span>
      </div>

      {/* StageOverviewBar */}
      <div
        className="stage-overview-bar"
        style={{
          padding: '10px 20px',
          background: SURFACE.bg,
          borderBottom: `1px solid ${SURFACE.border}`,
          flexShrink: 0,
        }}
      >
        <StageOverviewBar
          stageId={stageId}
          context={context}
          stageStatus={stageStatus}
          evidence={evidence}
          evidenceLoading={evidenceLoading}
          understanding={understanding}
          understandingLoading={understandingLoading}
          views={views}
          viewsLoading={viewsLoading}
          qaHistory={qaHistory}
          qaLoading={qaLoading}
          qualityReport={qualityReport}
          qualityLoading={qualityLoading}
        />
      </div>

      {/* StageFilterBar */}
      <div
        className="stage-filter-bar"
        style={{
          padding: '8px 20px',
          background: SURFACE.bgSubtle,
          borderBottom: `1px solid ${SURFACE.border}`,
          flexShrink: 0,
        }}
      >
        <StageFilterBar
          activeTab={activeTab}
          evidence={evidence}
          qualityReport={qualityReport}
          evidenceFilter={evidenceFilter}
          onEvidenceFilterChange={setEvidenceFilter}
          qualityFilter={qualityFilter}
          onQualityFilterChange={setQualityFilter}
        />
      </div>

      {/* Artifact tabs */}
      <div
        className="artifact-tabs"
        style={{
          display: 'flex',
          gap: 2,
          padding: '0 20px',
          background: SURFACE.bg,
          borderBottom: `1px solid ${SURFACE.border}`,
          flexShrink: 0,
          overflowX: 'auto',
        }}
      >
        {ARTIFACT_TAB_LABELS.map(({ key, label }) => {
          const isActive = activeTab === key;
          return (
            <button
              key={key}
              onClick={() => setActiveTab(key)}
              style={{
                padding: '10px 14px',
                border: 'none',
                borderBottom: isActive ? `2px solid ${ACCENT.blue}` : '2px solid transparent',
                background: 'transparent',
                color: isActive ? ACCENT.blue : SURFACE.textMuted,
                fontSize: FONT.body,
                fontWeight: isActive ? 600 : 400,
                whiteSpace: 'nowrap',
                cursor: 'pointer',
                marginBottom: -1,
              }}
            >
              {label}
            </button>
          );
        })}
      </div>

      {/* 主内容区 + ContextPanel */}
      <div
        className="stage-workspace-body"
        style={{
          display: 'flex',
          flex: 1,
          overflow: 'hidden',
        }}
      >
        <div
          className="stage-content-area"
          style={{
            flex: 1,
            padding: 20,
            overflowY: 'auto',
            minWidth: 0,
          }}
        >
          {renderContent({
            activeTab,
            evidenceFilter,
            qualityFilter,
            onContextSelectionChange: onContextSelectionChange ?? (() => {}),
          })}
        </div>

        <ContextPanel
          selection={validSelection}
          onViewSource={onViewSource}
          onLocateEvidence={onLocateEvidence}
        />
      </div>
    </div>
  );
}
