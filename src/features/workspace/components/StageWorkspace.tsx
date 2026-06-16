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
 * StageWorkspace: 阶段工作区骨架
 * Batch B 实现真实的 Artifact tabs 切换、顶部概览条与中部筛选条。
 * 右侧 ContextPanel 仍为占位容器（Batch C 实现真实联动）。
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
        background: '#f8fafc',
        overflow: 'hidden',
      }}
    >
      {/* WorkspaceTopBar */}
      <div
        className="stage-workspace-topbar"
        style={{
          padding: '12px 20px',
          background: '#fff',
          borderBottom: '1px solid #e2e8f0',
          flexShrink: 0,
        }}
      >
        <div
          style={{
            fontSize: 15,
            fontWeight: 600,
            color: '#1e293b',
          }}
        >
          {profile.workspace_name} / {stageId}
        </div>
        <div
          style={{
            fontSize: 12,
            color: '#64748b',
            marginTop: 4,
            wordBreak: 'break-all',
          }}
        >
          {context.source_path}
        </div>
      </div>

      {/* StageOverviewBar */}
      <div
        className="stage-overview-bar"
        style={{
          padding: '10px 20px',
          background: '#fff',
          borderBottom: '1px solid #e2e8f0',
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
          background: '#f8fafc',
          borderBottom: '1px solid #e2e8f0',
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
          gap: 4,
          padding: '8px 20px 0',
          background: '#f8fafc',
          borderBottom: '1px solid #e2e8f0',
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
                padding: '8px 14px',
                borderRadius: '6px 6px 0 0',
                border: isActive ? '1px solid #e2e8f0' : '1px solid transparent',
                borderBottom: isActive ? '2px solid #1976d2' : '2px solid transparent',
                background: isActive ? '#fff' : 'transparent',
                color: isActive ? '#1976d2' : '#64748b',
                fontSize: 13,
                fontWeight: isActive ? 600 : 400,
                whiteSpace: 'nowrap',
                cursor: 'pointer',
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
