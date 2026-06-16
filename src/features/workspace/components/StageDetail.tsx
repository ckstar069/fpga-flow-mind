import type {
  StageContext,
  StageFile,
  UpstreamRef,
  EvidenceCollection,
  ImplementationUnderstanding,
  ViewGraph,
  SelectedTraceTarget,
  TraceRefResolved,
  SourceExcerpt,
  GroundedAnswer,
  GroundedAnswerCitation,
  QualityReport,
} from '../../../types/workspace';
import type { UiError } from '../workspaceUiTypes';
import { formatBytes } from '../workspaceUiUtils';
import type { ContextSelection } from './contextPanelTypes';
import { SURFACE, ACCENT, FONT } from './workbenchTheme';
import EvidencePanel from './EvidencePanel';
import UnderstandingPanel from './UnderstandingPanel';
import MultiViewPanel from './MultiViewPanel';
import TracePanel from './TracePanel';
import SourceExcerptPanel from './SourceExcerptPanel';
import GroundedQAPanel from './GroundedQAPanel';
import QualityReviewPanel from './QualityReviewPanel';
import type { EvidenceFilter, QualityFilter } from './StageFilterBar';

type ArtifactTab =
  | 'overview'
  | 'evidence'
  | 'understanding'
  | 'views'
  | 'trace'
  | 'qa'
  | 'quality';

interface StageDetailProps {
  activeTab: ArtifactTab;
  stageId: string;
  context: StageContext;
  evidence?: EvidenceCollection;
  evidenceError?: UiError;
  isCollecting?: boolean;
  onCollectEvidence?: () => void;
  understanding?: ImplementationUnderstanding;
  understandingLoading?: boolean;
  understandingError?: UiError;
  onGenerateUnderstanding?: () => void;
  views?: ViewGraph[];
  viewsLoading?: boolean;
  viewsError?: UiError;
  onGenerateViews?: () => void;
  rootPath?: string;
  selectedTraceTarget?: SelectedTraceTarget | null;
  resolvedTraces?: TraceRefResolved[];
  traceLoading?: boolean;
  traceError?: UiError | null;
  sourceExcerpt?: SourceExcerpt | null;
  excerptError?: UiError | null;
  highlightedEvidenceId?: string | null;
  currentSourceEvidenceId?: string | null;
  groundedAnswer?: GroundedAnswer | null;
  groundedAnswerLoading?: boolean;
  groundedAnswerError?: UiError | null;
  onSelectTraceTarget?: (target: SelectedTraceTarget) => void;
  onClearTraceTarget?: () => void;
  onViewSource?: (location: {
    source_path: string;
    line_range: { start: number; end: number };
    evidence_id?: string;
  }) => void;
  onCloseSourceExcerpt?: () => void;
  onLocateEvidence?: (evidenceId: string) => void;
  onEvidenceSelect?: (evidenceId: string) => void;
  onAskGroundedQuestion?: (question: string) => void;
  onGroundedCitationClick?: (citation: GroundedAnswerCitation) => void;
  onContextSelectionChange?: (selection: ContextSelection | null) => void;
  qualityReport?: QualityReport | null;
  qualityLoading?: boolean;
  qualityError?: UiError | null;
  canGenerateQualityReport?: boolean;
  qualityDisabledReason?: string;
  onGenerateQualityReport?: () => void;
  evidenceFilter?: EvidenceFilter;
  qualityFilter?: QualityFilter;
}

export default function StageDetail({
  activeTab,
  stageId,
  context,
  evidence,
  evidenceError,
  isCollecting,
  onCollectEvidence,
  understanding,
  understandingLoading,
  understandingError,
  onGenerateUnderstanding,
  views,
  viewsLoading,
  viewsError,
  onGenerateViews,
  selectedTraceTarget,
  resolvedTraces,
  traceLoading,
  traceError,
  sourceExcerpt,
  excerptError,
  highlightedEvidenceId,
  currentSourceEvidenceId,
  groundedAnswer,
  groundedAnswerLoading,
  groundedAnswerError,
  onSelectTraceTarget,
  onClearTraceTarget,
  onViewSource,
  onCloseSourceExcerpt,
  onLocateEvidence,
  onEvidenceSelect,
  onAskGroundedQuestion,
  onGroundedCitationClick,
  onContextSelectionChange,
  qualityReport,
  qualityLoading,
  qualityError,
  canGenerateQualityReport,
  qualityDisabledReason,
  onGenerateQualityReport,
  evidenceFilter,
  qualityFilter,
}: StageDetailProps) {
  const canCollect =
    !context.error_code && context.files.length > 0 && !!onCollectEvidence;
  const canAskGrounded = !!evidence && !!understanding && evidence.evidence_items.length > 0;

  switch (activeTab) {
    case 'overview':
      return (
        <OverviewTab
          context={context}
          canCollect={canCollect}
          isCollecting={isCollecting}
          evidence={evidence}
          onCollectEvidence={onCollectEvidence}
          onGenerateUnderstanding={onGenerateUnderstanding}
          understanding={understanding}
          understandingLoading={understandingLoading}
          viewsLoading={viewsLoading}
          onGenerateViews={onGenerateViews}
          views={views}
        />
      );
    case 'evidence':
      return (
        <EvidenceTab
          evidence={evidence}
          evidenceError={evidenceError}
          isCollecting={isCollecting}
          canCollect={canCollect}
          onCollectEvidence={onCollectEvidence}
          highlightedEvidenceId={highlightedEvidenceId}
          currentSourceEvidenceId={currentSourceEvidenceId}
          onEvidenceSelect={onEvidenceSelect}
          onContextSelectionChange={onContextSelectionChange}
          stageId={stageId}
          evidenceFilter={evidenceFilter}
        />
      );
    case 'understanding':
      return (
        <UnderstandingTab
          context={context}
          understanding={understanding}
          understandingLoading={understandingLoading}
          understandingError={understandingError}
          onGenerateUnderstanding={onGenerateUnderstanding}
          viewsLoading={viewsLoading}
        />
      );
    case 'views':
      return (
        <ViewsTab
          views={views}
          viewsLoading={viewsLoading}
          viewsError={viewsError}
          onGenerateViews={onGenerateViews}
          understanding={understanding}
          understandingLoading={understandingLoading}
          selectedTraceTarget={selectedTraceTarget}
          onSelectTraceTarget={onSelectTraceTarget}
          stageId={stageId}
          onContextSelectionChange={onContextSelectionChange}
        />
      );
    case 'trace':
      return (
        <TraceTab
          selectedTraceTarget={selectedTraceTarget}
          resolvedTraces={resolvedTraces}
          traceLoading={traceLoading}
          traceError={traceError}
          sourceExcerpt={sourceExcerpt}
          excerptError={excerptError}
          views={views}
          onClearTraceTarget={onClearTraceTarget}
          onViewSource={onViewSource}
          onLocateEvidence={onLocateEvidence}
          onCloseSourceExcerpt={onCloseSourceExcerpt}
        />
      );
    case 'qa':
      return (
        <QaTab
          canAskGrounded={canAskGrounded}
          evidence={evidence}
          understanding={understanding}
          groundedAnswer={groundedAnswer}
          groundedAnswerLoading={groundedAnswerLoading}
          groundedAnswerError={groundedAnswerError}
          onAskGroundedQuestion={onAskGroundedQuestion}
          onGroundedCitationClick={onGroundedCitationClick}
        />
      );
    case 'quality':
      return (
        <QualityTab
          qualityReport={qualityReport}
          qualityLoading={qualityLoading}
          qualityError={qualityError}
          canGenerateQualityReport={canGenerateQualityReport}
          qualityDisabledReason={qualityDisabledReason}
          onGenerateQualityReport={onGenerateQualityReport}
          onEvidenceSelect={onEvidenceSelect}
          onContextSelectionChange={onContextSelectionChange}
          qualityFilter={qualityFilter}
        />
      );
    default:
      return null;
  }
}

// ─── Overview Tab ───────────────────────────────────────────────────────

function OverviewTab({
  context,
  canCollect,
  isCollecting,
  evidence,
  onCollectEvidence,
  onGenerateUnderstanding,
  understanding,
  understandingLoading,
  viewsLoading,
  onGenerateViews,
  views,
}: {
  context: StageContext;
  canCollect: boolean;
  isCollecting?: boolean;
  evidence?: EvidenceCollection;
  onCollectEvidence?: () => void;
  onGenerateUnderstanding?: () => void;
  understanding?: ImplementationUnderstanding;
  understandingLoading?: boolean;
  viewsLoading?: boolean;
  onGenerateViews?: () => void;
  views?: ViewGraph[];
}) {
  return (
    <div>
      <h2 style={{ margin: '0 0 16px', fontSize: 20 }}>
        {context.stage_id}
        {context.error_code && (
          <span
            style={{
              fontSize: 13,
              marginLeft: 12,
              padding: '2px 8px',
              background: '#ffebee',
              color: '#c62828',
              borderRadius: 4,
            }}
          >
            {context.error_code}
          </span>
        )}
      </h2>
      <p
        style={{
          fontSize: 13,
          color: '#666',
          margin: '0 0 16px',
          wordBreak: 'break-all',
        }}
      >
        {context.source_path}
      </p>

      {context.error_code === 'stage_empty' && (
        <div
          style={{
            padding: 16,
            background: SURFACE.bgSubtle,
            border: `1px solid ${SURFACE.border}`,
            borderRadius: 8,
            textAlign: 'center',
            marginBottom: 16,
            color: SURFACE.textMuted,
            fontSize: FONT.caption,
          }}
        >
          该阶段为空：未发现可分析文件。
        </div>
      )}

      {context.files.length > 0 && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>
            文件列表 ({context.files.length})
          </h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {context.files.map((f: StageFile, i: number) => (
              <div
                key={i}
                style={{
                  padding: '8px 12px',
                  background: '#fff',
                  borderRadius: 6,
                  border: '1px solid #e0e0e0',
                }}
              >
                <div
                  style={{
                    fontSize: 13,
                    fontWeight: 500,
                    wordBreak: 'break-all',
                  }}
                >
                  {f.source_path}
                </div>
                <div style={{ fontSize: 12, color: '#999', marginTop: 4 }}>
                  {f.language} / {f.source_kind}
                  {f.size_bytes !== undefined && ` · ${formatBytes(f.size_bytes)}`}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, marginBottom: 24 }}>
        {canCollect && (
          <ActionButton
            onClick={onCollectEvidence}
            disabled={isCollecting || understandingLoading || viewsLoading}
            variant={evidence ? 'success' : 'primary'}
            label={
              isCollecting
                ? '收集中...'
                : understandingLoading
                  ? '生成中，请稍候'
                  : viewsLoading
                    ? '视图生成中，请稍候'
                    : evidence
                      ? `重新收集 (${evidence.evidence_items.length} 项)`
                      : '收集证据'
            }
          />
        )}

        {onGenerateUnderstanding && !context.error_code && (
          <ActionButton
            onClick={onGenerateUnderstanding}
            disabled={understandingLoading || viewsLoading}
            variant={understanding ? 'success' : 'secondary'}
            label={
              understandingLoading
                ? '生成中...'
                : viewsLoading
                  ? '视图生成中，请稍候'
                  : understanding
                    ? '重新生成理解'
                    : '生成理解'
            }
          />
        )}

        {onGenerateViews && !context.error_code && understanding && (
          <ActionButton
            onClick={onGenerateViews}
            disabled={viewsLoading || understandingLoading}
            variant={views ? 'success' : 'primary'}
            label={
              viewsLoading
                ? '生成视图中...'
                : views
                  ? '重新生成视图'
                  : '生成视图'
            }
          />
        )}
      </div>

      {context.external_deps.length > 0 && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>外部依赖</h3>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
            {context.external_deps.map((dep: string, i: number) => (
              <span
                key={i}
                style={{
                  padding: '4px 10px',
                  background: '#e3f2fd',
                  borderRadius: 4,
                  fontSize: 13,
                }}
              >
                {dep}
              </span>
            ))}
          </div>
        </div>
      )}

      {context.upstream_refs.length > 0 && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>上游引用</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            {context.upstream_refs.map((ref: UpstreamRef, i: number) => (
              <div
                key={i}
                style={{
                  padding: '8px 12px',
                  background: '#fff',
                  borderRadius: 6,
                  border: '1px solid #e0e0e0',
                }}
              >
                <span style={{ fontWeight: 600 }}>{ref.stage_id}</span>
                {ref.interface_file_path && (
                  <span style={{ fontSize: 12, color: '#666', marginLeft: 8 }}>
                    {ref.interface_file_path}
                  </span>
                )}
                <span style={{ fontSize: 11, color: '#999', marginLeft: 8 }}>(推断)</span>
              </div>
            ))}
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Evidence Tab ───────────────────────────────────────────────────────

function EvidenceTab({
  evidence,
  evidenceError,
  isCollecting,
  canCollect,
  onCollectEvidence,
  highlightedEvidenceId,
  currentSourceEvidenceId,
  onEvidenceSelect,
  onContextSelectionChange,
  stageId,
  evidenceFilter,
}: {
  evidence?: EvidenceCollection;
  evidenceError?: UiError;
  isCollecting?: boolean;
  canCollect: boolean;
  onCollectEvidence?: () => void;
  highlightedEvidenceId?: string | null;
  currentSourceEvidenceId?: string | null;
  onEvidenceSelect?: (evidenceId: string) => void;
  onContextSelectionChange?: (selection: ContextSelection | null) => void;
  stageId: string;
  evidenceFilter?: EvidenceFilter;
}) {
  return (
    <div>
      {canCollect && (
        <div style={{ marginBottom: 16 }}>
          <ActionButton
            onClick={onCollectEvidence}
            disabled={isCollecting}
            variant={evidence ? 'success' : 'primary'}
            label={isCollecting ? '收集中...' : evidence ? '重新收集' : '收集证据'}
          />
        </div>
      )}

      {evidenceError && <ErrorBlock title="证据收集失败" error={evidenceError} />}

      {evidence && (
        <EvidencePanel
          evidence={evidence}
          stageId={stageId}
          highlightedEvidenceId={highlightedEvidenceId ?? undefined}
          currentSourceEvidenceId={currentSourceEvidenceId ?? undefined}
          onEvidenceSelect={onEvidenceSelect}
          onContextSelection={onContextSelectionChange}
          evidenceFilter={evidenceFilter}
        />
      )}

      {!evidence && !evidenceError && !isCollecting && (
        <EmptyState message="尚未收集证据，请点击上方按钮开始收集。" />
      )}
    </div>
  );
}

// ─── Understanding Tab ──────────────────────────────────────────────────

function UnderstandingTab({
  context,
  understanding,
  understandingLoading,
  understandingError,
  onGenerateUnderstanding,
  viewsLoading,
}: {
  context: StageContext;
  understanding?: ImplementationUnderstanding;
  understandingLoading?: boolean;
  understandingError?: UiError;
  onGenerateUnderstanding?: () => void;
  viewsLoading?: boolean;
}) {
  return (
    <div>
      {onGenerateUnderstanding && !context.error_code && (
        <div style={{ marginBottom: 16 }}>
          {context.error_code === 'stage_empty' ? (
            <p style={{ fontSize: 13, color: '#999', margin: 0 }}>空阶段无法生成理解</p>
          ) : (
            <ActionButton
              onClick={onGenerateUnderstanding}
              disabled={understandingLoading || viewsLoading}
              variant={understanding ? 'success' : 'secondary'}
              label={
                understandingLoading
                  ? '生成中...'
                  : viewsLoading
                    ? '视图生成中，请稍候'
                    : understanding
                      ? '重新生成理解'
                      : '生成理解'
              }
            />
          )}
        </div>
      )}

      {understandingError && <ErrorBlock title="理解生成失败" error={understandingError} />}

      {understandingLoading && (
        <LoadingBlock
          title="正在生成理解..."
          subtitle="正在调用后端处理，请稍候"
          color={ACCENT.blue}
        />
      )}

      {understanding && (
        <UnderstandingPanel understanding={understanding} />
      )}

      {!understanding && !understandingLoading && !understandingError && (
        <EmptyState message="尚未生成理解，请点击上方按钮开始生成。" />
      )}
    </div>
  );
}

// ─── Views Tab ──────────────────────────────────────────────────────────

function ViewsTab({
  views,
  viewsLoading,
  viewsError,
  onGenerateViews,
  understanding,
  understandingLoading,
  selectedTraceTarget,
  onSelectTraceTarget,
  stageId,
  onContextSelectionChange,
}: {
  views?: ViewGraph[];
  viewsLoading?: boolean;
  viewsError?: UiError;
  onGenerateViews?: () => void;
  understanding?: ImplementationUnderstanding;
  understandingLoading?: boolean;
  selectedTraceTarget?: SelectedTraceTarget | null;
  onSelectTraceTarget?: (target: SelectedTraceTarget) => void;
  stageId: string;
  onContextSelectionChange?: (selection: ContextSelection | null) => void;
}) {
  return (
    <div>
      {onGenerateViews && (
        <div style={{ marginBottom: 16 }}>
          {!understanding ? (
            <p style={{ fontSize: 13, color: '#999', margin: 0 }}>请先生成理解</p>
          ) : (
            <ActionButton
              onClick={onGenerateViews}
              disabled={viewsLoading || understandingLoading}
              variant={views ? 'success' : 'primary'}
              label={
                viewsLoading
                  ? '生成视图中...'
                  : views
                    ? '重新生成视图'
                    : '生成视图'
              }
            />
          )}
        </div>
      )}

      {viewsError && !viewsLoading && <ErrorBlock title="视图生成失败" error={viewsError} />}

      {(views || viewsLoading || (!viewsLoading && viewsError)) && (
        <MultiViewPanel
          views={views ?? []}
          loading={viewsLoading}
          error={!viewsLoading ? viewsError : undefined}
          stageId={stageId}
          selectedTarget={selectedTraceTarget}
          onSelectTarget={onSelectTraceTarget}
          onContextSelection={onContextSelectionChange}
        />
      )}

      {!views && !viewsLoading && !viewsError && (
        <EmptyState message="尚未生成视图，请先生成理解再生成视图。" />
      )}
    </div>
  );
}

// ─── Trace Tab ──────────────────────────────────────────────────────────

function TraceTab({
  selectedTraceTarget,
  resolvedTraces,
  traceLoading,
  traceError,
  sourceExcerpt,
  excerptError,
  views,
  onClearTraceTarget,
  onViewSource,
  onLocateEvidence,
  onCloseSourceExcerpt,
}: {
  selectedTraceTarget?: SelectedTraceTarget | null;
  resolvedTraces?: TraceRefResolved[];
  traceLoading?: boolean;
  traceError?: UiError | null;
  sourceExcerpt?: SourceExcerpt | null;
  excerptError?: UiError | null;
  views?: ViewGraph[];
  onClearTraceTarget?: () => void;
  onViewSource?: (location: {
    source_path: string;
    line_range: { start: number; end: number };
    evidence_id?: string;
  }) => void;
  onLocateEvidence?: (evidenceId: string) => void;
  onCloseSourceExcerpt?: () => void;
}) {
  return (
    <div>
      {!selectedTraceTarget && !sourceExcerpt && !excerptError && (
        <EmptyState message="请在视图分区选择一个节点或边以开始追溯。" />
      )}

      {selectedTraceTarget && (
        <TracePanel
          selectedTargetLabel={getSelectedTargetLabel(selectedTraceTarget, views)}
          selectedTargetType={getSelectedTargetTypeLabel(selectedTraceTarget)}
          resolvedTraces={resolvedTraces ?? []}
          loading={traceLoading}
          error={traceError}
          onClear={onClearTraceTarget ?? (() => {})}
          onViewSource={onViewSource ?? (() => {})}
          onLocateEvidence={onLocateEvidence ?? (() => {})}
        />
      )}

      {(sourceExcerpt || excerptError) && (
        <SourceExcerptPanel
          excerpt={sourceExcerpt}
          onClose={onCloseSourceExcerpt ?? (() => {})}
          error={excerptError}
        />
      )}
    </div>
  );
}

// ─── Q&A Tab ────────────────────────────────────────────────────────────

function QaTab({
  canAskGrounded,
  evidence,
  understanding,
  groundedAnswer,
  groundedAnswerLoading,
  groundedAnswerError,
  onAskGroundedQuestion,
  onGroundedCitationClick,
}: {
  canAskGrounded: boolean;
  evidence?: EvidenceCollection;
  understanding?: ImplementationUnderstanding;
  groundedAnswer?: GroundedAnswer | null;
  groundedAnswerLoading?: boolean;
  groundedAnswerError?: UiError | null;
  onAskGroundedQuestion?: (question: string) => void;
  onGroundedCitationClick?: (citation: GroundedAnswerCitation) => void;
}) {
  if (!onAskGroundedQuestion) return null;
  return (
    <div>
      <GroundedQAPanel
        canAsk={canAskGrounded}
        disabledReason={
          !evidence
            ? '请先收集证据'
            : !understanding
              ? '请先生成理解'
              : evidence.evidence_items.length === 0
                ? '当前阶段无 evidence，无法提问'
                : undefined
        }
        answer={groundedAnswer}
        loading={groundedAnswerLoading}
        error={groundedAnswerError}
        onAsk={onAskGroundedQuestion}
        onCitationClick={onGroundedCitationClick}
      />
    </div>
  );
}

// ─── Quality Tab ────────────────────────────────────────────────────────

function QualityTab({
  qualityReport,
  qualityLoading,
  qualityError,
  canGenerateQualityReport,
  qualityDisabledReason,
  onGenerateQualityReport,
  onEvidenceSelect,
  onContextSelectionChange,
  qualityFilter,
}: {
  qualityReport?: QualityReport | null;
  qualityLoading?: boolean;
  qualityError?: UiError | null;
  canGenerateQualityReport?: boolean;
  qualityDisabledReason?: string;
  onGenerateQualityReport?: () => void;
  onEvidenceSelect?: (evidenceId: string) => void;
  onContextSelectionChange?: (selection: ContextSelection | null) => void;
  qualityFilter?: QualityFilter;
}) {
  const filteredReport = useFilteredQualityReport(qualityReport, qualityFilter);

  if (!onGenerateQualityReport) return null;
  return (
    <div>
      <QualityReviewPanel
        report={filteredReport}
        loading={qualityLoading}
        error={qualityError}
        canGenerate={canGenerateQualityReport ?? true}
        disabledReason={qualityDisabledReason}
        onGenerate={onGenerateQualityReport}
        onEvidenceSelect={onEvidenceSelect}
        onContextSelection={onContextSelectionChange}
      />
    </div>
  );
}

function useFilteredQualityReport(
  report: QualityReport | null | undefined,
  filter: QualityFilter | undefined
): QualityReport | null | undefined {
  if (!report || !filter) return report;
  const severity = filter.severity;
  const kind = filter.kind;
  const status = filter.status;
  if (!severity && !kind && !status) return report;

  const filteredIssues = report.issues.filter((issue) => {
    if (severity && issue.severity !== severity) return false;
    if (kind && issue.kind !== kind) return false;
    if (status && issue.status !== status) return false;
    return true;
  });

  return {
    ...report,
    issues: filteredIssues,
    summary: {
      ...report.summary,
      total_issues: filteredIssues.length,
      issues_by_severity: countBy(filteredIssues, (i) => i.severity),
      issues_by_status: countBy(filteredIssues, (i) => i.status),
      issues_by_kind: countBy(filteredIssues, (i) => i.kind),
    },
  };
}

function countBy<T>(items: T[], keyFn: (item: T) => string): Record<string, number> {
  const result: Record<string, number> = {};
  for (const item of items) {
    const key = keyFn(item);
    result[key] = (result[key] ?? 0) + 1;
  }
  return result;
}

// ─── 通用辅助组件 ───────────────────────────────────────────────────────

function ActionButton({
  onClick,
  disabled,
  variant,
  label,
}: {
  onClick?: () => void;
  disabled?: boolean;
  variant: 'primary' | 'secondary' | 'success';
  label: string;
}) {
  const colors = {
    primary: { border: ACCENT.blue, bg: ACCENT.blue },
    secondary: { border: ACCENT.teal, bg: ACCENT.teal },
    success: { border: ACCENT.green, bg: ACCENT.green },
  };
  const c = colors[variant];
  return (
    <button
      onClick={onClick}
      disabled={disabled}
      style={{
        padding: '8px 20px',
        borderRadius: 6,
        border: `1px solid ${c.border}`,
        background: disabled ? SURFACE.border : c.bg,
        color: disabled ? SURFACE.textDim : '#fff',
        cursor: disabled ? 'not-allowed' : 'pointer',
        fontSize: FONT.body,
      }}
    >
      {label}
    </button>
  );
}

function ErrorBlock({ title, error }: { title: string; error: UiError }) {
  return (
    <div
      style={{
        padding: 16,
        background: ACCENT.redSoft,
        border: `1px solid ${ACCENT.redBorder}`,
        borderRadius: 8,
        marginBottom: 16,
      }}
    >
      <h4 style={{ margin: '0 0 8px', fontSize: FONT.heading, color: ACCENT.red }}>{title}</h4>
      <div style={{ fontSize: FONT.body, color: SURFACE.text }}>
        {'error_code' in error && (
          <div style={{ marginBottom: 4 }}>
            <span style={{ color: SURFACE.textMuted }}>错误码：</span>
            <code>{error.error_code}</code>
          </div>
        )}
        <div style={{ marginBottom: 4 }}>
          <span style={{ color: SURFACE.textMuted }}>信息：</span>
          {error.message}
        </div>
        {'source_path' in error && error.source_path && (
          <div style={{ marginBottom: 4 }}>
            <span style={{ color: SURFACE.textMuted }}>路径：</span>
            <code style={{ fontSize: FONT.caption }}>{error.source_path}</code>
          </div>
        )}
        {'details' in error && error.details && (
          <div>
            <span style={{ color: SURFACE.textMuted }}>详情：</span>
            {error.details}
          </div>
        )}
        {'recoverable' in error && (
          <div style={{ marginTop: 4 }}>
            <span
              style={{
                fontSize: FONT.caption,
                color: error.recoverable ? ACCENT.amber : ACCENT.red,
              }}
            >
              {error.recoverable ? '可重试' : '不可恢复'}
            </span>
          </div>
        )}
      </div>
    </div>
  );
}

function LoadingBlock({
  title,
  subtitle,
  color,
}: {
  title: string;
  subtitle: string;
  color: string;
}) {
  return (
    <div
      style={{
        padding: 24,
        background: `${color}15`,
        borderRadius: 8,
        textAlign: 'center',
        marginBottom: 16,
        border: `1px solid ${color}40`,
      }}
    >
      <p style={{ margin: 0, color, fontSize: 14 }}>{title}</p>
      <p style={{ margin: '8px 0 0', color: '#999', fontSize: 12 }}>{subtitle}</p>
    </div>
  );
}

function EmptyState({ message }: { message: string }) {
  return (
    <div
      style={{
        padding: 32,
        background: SURFACE.bgSubtle,
        border: `1px solid ${SURFACE.border}`,
        borderRadius: 8,
        textAlign: 'center',
        color: SURFACE.textMuted,
      }}
    >
      <p style={{ margin: 0, fontSize: FONT.body }}>{message}</p>
    </div>
  );
}

// ─── 选择目标标签辅助函数 ───────────────────────────────────────────────

function getSelectedTargetTypeLabel(target: SelectedTraceTarget): string {
  switch (target.kind) {
    case 'view_node':
      return '视图节点';
    case 'view_edge':
      return '视图边';
    case 'claim':
      return '声明';
    case 'evidence':
      return '证据';
    default:
      return '未知';
  }
}

function getSelectedTargetLabel(
  target: SelectedTraceTarget,
  views?: ViewGraph[]
): string {
  if (target.kind === 'view_node') {
    const node = views
      ?.find((g) => g.view_type === target.view_type)
      ?.nodes.find((n) => n.node_id === target.node_id);
    return node?.label ?? target.node_id;
  }
  if (target.kind === 'view_edge') {
    const edge = views
      ?.find((g) => g.view_type === target.view_type)
      ?.edges.find((e) => e.edge_id === target.edge_id);
    return edge?.label ?? edge?.edge_id ?? target.edge_id;
  }
  if (target.kind === 'claim') {
    return target.claim_id;
  }
  return target.evidence_id;
}
