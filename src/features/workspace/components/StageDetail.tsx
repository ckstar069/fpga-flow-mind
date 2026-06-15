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
import EvidencePanel from './EvidencePanel';
import UnderstandingPanel from './UnderstandingPanel';
import MultiViewPanel from './MultiViewPanel';
import TracePanel from './TracePanel';
import SourceExcerptPanel from './SourceExcerptPanel';
import GroundedQAPanel from './GroundedQAPanel';
import QualityReviewPanel from './QualityReviewPanel';

interface StageDetailProps {
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
  qualityReport?: QualityReport | null;
  qualityLoading?: boolean;
  qualityError?: UiError | null;
  canGenerateQualityReport?: boolean;
  qualityDisabledReason?: string;
  onGenerateQualityReport?: () => void;
}

export default function StageDetail({
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
  qualityReport,
  qualityLoading,
  qualityError,
  canGenerateQualityReport,
  qualityDisabledReason,
  onGenerateQualityReport,
}: StageDetailProps) {
  const canCollect =
    !context.error_code && context.files.length > 0 && !!onCollectEvidence;
  const canAskGrounded = !!evidence && !!understanding && evidence.evidence_items.length > 0;

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
            padding: 24,
            background: '#fafafa',
            borderRadius: 8,
            textAlign: 'center',
            marginBottom: 16,
          }}
        >
          <p style={{ margin: 0, color: '#999' }}>该阶段无文件</p>
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

      {/* 证据收集区域 */}
      {canCollect && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>证据收集</h3>
          <button
            onClick={onCollectEvidence}
            disabled={isCollecting || understandingLoading || viewsLoading}
            style={{
              padding: '8px 20px',
              borderRadius: 6,
              border: evidence
                ? '1px solid #4caf50'
                : '1px solid #1976d2',
              background: (isCollecting || understandingLoading || viewsLoading)
                ? '#e0e0e0'
                : evidence
                  ? '#4caf50'
                  : '#1976d2',
              color: (isCollecting || understandingLoading || viewsLoading) ? '#999' : '#fff',
              cursor: (isCollecting || understandingLoading || viewsLoading) ? 'not-allowed' : 'pointer',
              fontSize: 14,
            }}
          >
            {isCollecting
              ? '收集中...'
              : understandingLoading
                ? '生成中，请稍候'
                : viewsLoading
                  ? '视图生成中，请稍候'
                  : evidence
                    ? `重新收集 (${evidence.evidence_items.length} 项)`
                    : '收集证据'}
          </button>
        </div>
      )}

      {evidenceError && (
        <div
          style={{
            padding: 16,
            background: '#fff3e0',
            borderRadius: 8,
            marginBottom: 16,
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 14 }}>证据收集失败</h4>
          <div style={{ fontSize: 13 }}>
            {'error_code' in evidenceError && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>错误码：</span>
                <code>{evidenceError.error_code}</code>
              </div>
            )}
            <div style={{ marginBottom: 4 }}>
              <span style={{ color: '#666' }}>信息：</span>
              {evidenceError.message}
            </div>
            {'source_path' in evidenceError && evidenceError.source_path && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>路径：</span>
                <code style={{ fontSize: 12 }}>{evidenceError.source_path}</code>
              </div>
            )}
            {'details' in evidenceError && evidenceError.details && (
              <div>
                <span style={{ color: '#666' }}>详情：</span>
                {evidenceError.details}
              </div>
            )}
          </div>
        </div>
      )}

      {/* 理解生成按钮 */}
      {onGenerateUnderstanding && !context.error_code && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>理解生成</h3>
          {context.error_code === 'stage_empty' ? (
            <p style={{ fontSize: 13, color: '#999', margin: 0 }}>
              空阶段无法生成理解
            </p>
          ) : (
            <>
              <button
                onClick={onGenerateUnderstanding}
                disabled={understandingLoading || viewsLoading}
                style={{
                  padding: '8px 20px',
                  borderRadius: 6,
                  border: understanding
                    ? '1px solid #2e7d32'
                    : '1px solid #7b1fa2',
                  background: (understandingLoading || viewsLoading)
                    ? '#e0e0e0'
                    : understanding
                      ? '#2e7d32'
                      : '#7b1fa2',
                  color: (understandingLoading || viewsLoading) ? '#999' : '#fff',
                  cursor: (understandingLoading || viewsLoading) ? 'not-allowed' : 'pointer',
                  fontSize: 14,
                }}
              >
                {understandingLoading
                  ? '生成中...'
                  : viewsLoading
                    ? '视图生成中，请稍候'
                    : understanding
                      ? '重新生成'
                      : '生成理解'}
              </button>
              {!understanding && !understandingLoading && !viewsLoading && (
                <span style={{ fontSize: 12, color: '#999', marginLeft: 12 }}>
                  基于已收集的证据生成结构化理解
                </span>
              )}
            </>
          )}
        </div>
      )}

      {/* 理解生成错误 */}
      {understandingError && (
        <div
          style={{
            padding: 16,
            background: '#fce4ec',
            borderRadius: 8,
            marginBottom: 16,
            border: '1px solid #ef9a9a',
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 14, color: '#c62828' }}>理解生成失败</h4>
          <div style={{ fontSize: 13 }}>
            {'error_code' in understandingError && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>错误码：</span>
                <code>{understandingError.error_code}</code>
              </div>
            )}
            <div style={{ marginBottom: 4 }}>
              <span style={{ color: '#666' }}>信息：</span>
              {understandingError.message}
            </div>
            {'source_path' in understandingError && understandingError.source_path && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>路径：</span>
                <code style={{ fontSize: 12 }}>{understandingError.source_path}</code>
              </div>
            )}
            {'details' in understandingError && understandingError.details && (
              <div>
                <span style={{ color: '#666' }}>详情：</span>
                {understandingError.details}
              </div>
            )}
            {'recoverable' in understandingError && (
              <div style={{ marginTop: 4 }}>
                <span style={{ fontSize: 12, color: understandingError.recoverable ? '#f57c00' : '#c62828' }}>
                  {understandingError.recoverable ? '可重试' : '不可恢复'}
                </span>
              </div>
            )}
          </div>
        </div>
      )}

      {/* 理解生成 loading */}
      {understandingLoading && (
        <div
          style={{
            padding: 24,
            background: '#f3e5f5',
            borderRadius: 8,
            textAlign: 'center',
            marginBottom: 16,
            border: '1px solid #ce93d8',
          }}
        >
          <p style={{ margin: 0, color: '#7b1fa2', fontSize: 14 }}>正在生成理解...</p>
          <p style={{ margin: '8px 0 0', color: '#999', fontSize: 12 }}>正在调用后端处理，请稍候</p>
        </div>
      )}

      {/* 理解面板 */}
      {understanding && (
        <div style={{ marginBottom: 24 }}>
          <UnderstandingPanel understanding={understanding} />
        </div>
      )}

      {/* 视图生成区域 */}
      {onGenerateViews && !context.error_code && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>视图生成</h3>
          {!understanding ? (
            <p style={{ fontSize: 13, color: '#999', margin: 0 }}>
              请先生成理解
            </p>
          ) : (
            <>
              <button
                onClick={onGenerateViews}
                disabled={viewsLoading || understandingLoading}
                style={{
                  padding: '8px 20px',
                  borderRadius: 6,
                  border: views
                    ? '1px solid #2e7d32'
                    : '1px solid #1976d2',
                  background: (viewsLoading || understandingLoading)
                    ? '#e0e0e0'
                    : views
                      ? '#2e7d32'
                      : '#1976d2',
                  color: (viewsLoading || understandingLoading) ? '#999' : '#fff',
                  cursor: (viewsLoading || understandingLoading) ? 'not-allowed' : 'pointer',
                  fontSize: 14,
                }}
              >
                {viewsLoading
                  ? '生成视图中...'
                  : views
                    ? '重新生成视图'
                    : '生成视图'}
              </button>
              {!views && !viewsLoading && (
                <span style={{ fontSize: 12, color: '#999', marginLeft: 12 }}>
                  基于理解结果生成结构图、数据流和时序流水视图
                </span>
              )}
            </>
          )}
        </div>
      )}

      {/* 视图错误 */}
      {viewsError && !viewsLoading && (
        <div
          style={{
            padding: 16,
            background: '#fce4ec',
            borderRadius: 8,
            marginBottom: 16,
            border: '1px solid #ef9a9a',
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 14, color: '#c62828' }}>视图生成失败</h4>
          <div style={{ fontSize: 13 }}>
            {'error_code' in viewsError && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>错误码：</span>
                <code>{viewsError.error_code}</code>
              </div>
            )}
            <div style={{ marginBottom: 4 }}>
              <span style={{ color: '#666' }}>信息：</span>
              {viewsError.message}
            </div>
            {'source_path' in viewsError && viewsError.source_path && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>路径：</span>
                <code style={{ fontSize: 12 }}>{viewsError.source_path}</code>
              </div>
            )}
            {'details' in viewsError && viewsError.details && (
              <div>
                <span style={{ color: '#666' }}>详情：</span>
                {viewsError.details}
              </div>
            )}
          </div>
        </div>
      )}

      {/* 视图面板 — 当 views / viewsLoading / viewsError 任一存在时渲染 */}
      {(views || viewsLoading || (!viewsLoading && viewsError)) && (
        <MultiViewPanel
          views={views ?? []}
          loading={viewsLoading}
          error={!viewsLoading ? viewsError : undefined}
          selectedTarget={selectedTraceTarget}
          onSelectTarget={onSelectTraceTarget}
        />
      )}

      {/* TracePanel */}
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

      {/* SourceExcerptPanel */}
      {(sourceExcerpt || excerptError) && (
        <SourceExcerptPanel
          excerpt={sourceExcerpt}
          onClose={onCloseSourceExcerpt ?? (() => {})}
          error={excerptError}
        />
      )}

      {/* Grounded Q&A */}
      {onAskGroundedQuestion && (
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
      )}

      {/* Phase 7 质量评估 */}
      {onGenerateQualityReport && (
        <QualityReviewPanel
          report={qualityReport}
          loading={qualityLoading}
          error={qualityError}
          canGenerate={canGenerateQualityReport ?? true}
          disabledReason={qualityDisabledReason}
          onGenerate={onGenerateQualityReport}
          onEvidenceSelect={onEvidenceSelect}
          onViewSource={onViewSource}
        />
      )}

      {evidence && (
        <EvidencePanel
          evidence={evidence}
          highlightedEvidenceId={highlightedEvidenceId ?? undefined}
          currentSourceEvidenceId={currentSourceEvidenceId ?? undefined}
          onEvidenceSelect={onEvidenceSelect}
        />
      )}

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
                <span style={{ fontSize: 11, color: '#999', marginLeft: 8 }}>
                  (推断)
                </span>
              </div>
            ))}
          </div>
        </div>
      )}
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
