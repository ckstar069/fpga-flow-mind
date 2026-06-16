import type {
  EvidenceItem,
  TraceRefResolved,
  SourceExcerpt,
  QualityIssue,
  ViewNode,
  ViewEdge,
  GroundedAnswerCitation,
  SelectedTraceTarget,
  ClaimConfidence,
  TraceResolution,
} from '../../../types/workspace';
import type { ContextSelection, ContextSelectionKind, ContextSelectionPayload } from './contextPanelTypes';
import {
  CONTEXT_KIND_LABEL,
  CONTEXT_KIND_ICON,
  EVIDENCE_STRENGTH_LABEL,
  EVIDENCE_STRENGTH_COLOR,
  CONFIDENCE_LABEL,
  CONFIDENCE_BG,
  CONFIDENCE_COLOR,
  RESOLUTION_LABEL,
  RESOLUTION_COLOR,
  SEVERITY_COLOR,
  QUALITY_KIND_LABEL,
  POLARITY_LABEL,
  ISSUE_STATUS_LABEL,
  NODE_TYPE_LABEL,
  EDGE_TYPE_LABEL,
  VIEW_TYPE_LABEL,
  SOURCE_KIND_LABEL,
  LANGUAGE_LABEL,
  traceTargetTypeLabel,
} from './contextPanelUtils';

interface ContextPanelProps {
  selection: ContextSelection | null;
  onViewSource?: (location: {
    source_path: string;
    line_range: { start: number; end: number };
    evidence_id?: string;
  }) => void;
  onLocateEvidence?: (evidenceId: string) => void;
}

/**
 * ContextPanel：右侧只读上下文面板。
 * Batch C 实现真实联动，消费前端已有状态与命令结果。
 */
export default function ContextPanel({
  selection,
  onViewSource,
  onLocateEvidence,
}: ContextPanelProps) {
  if (!selection) {
    return (
      <div style={containerStyle}>
        <div style={emptyStyle}>
          <div style={{ fontSize: 32, marginBottom: 12 }}>🔎</div>
          <div style={{ fontSize: 14, color: '#64748b', fontWeight: 500 }}>
            选择 evidence / 节点 / 质量记录后在此查看上下文
          </div>
          <div style={{ fontSize: 12, color: '#94a3b8', marginTop: 8 }}>
            点击工作区中的对象以查看其详细信息
          </div>
        </div>
      </div>
    );
  }

  return (
    <div style={containerStyle}>
      <Header selection={selection} />
      <div style={{ padding: 12 }}>
        <ContextBody
          selection={selection}
          onViewSource={onViewSource}
          onLocateEvidence={onLocateEvidence}
        />
      </div>
    </div>
  );
}

function Header({ selection }: { selection: ContextSelection }) {
  const label = CONTEXT_KIND_LABEL[selection.kind];
  const icon = CONTEXT_KIND_ICON[selection.kind];
  return (
    <div
      style={{
        padding: '12px 14px',
        borderBottom: '1px solid #e2e8f0',
        background: '#f8fafc',
        display: 'flex',
        alignItems: 'center',
        gap: 8,
      }}
    >
      <span style={{ fontSize: 16 }}>{icon}</span>
      <span style={{ fontSize: 13, fontWeight: 600, color: '#1e293b' }}>
        {label}
      </span>
      <span style={{ fontSize: 11, color: '#94a3b8', marginLeft: 'auto' }}>
        stage={selection.stageId}
      </span>
    </div>
  );
}

function getPayload<T extends ContextSelectionKind>(
  selection: ContextSelection,
  _kind: T
): ContextSelectionPayload<T> {
  return selection.payload as ContextSelectionPayload<T>;
}

function ContextBody({
  selection,
  onViewSource,
  onLocateEvidence,
}: ContextPanelProps & { selection: ContextSelection }) {
  switch (selection.kind) {
    case 'evidence': {
      const { item } = getPayload(selection, 'evidence');
      return (
        <EvidenceBody
          item={item}
          onViewSource={onViewSource}
          onLocateEvidence={onLocateEvidence}
        />
      );
    }
    case 'trace_target': {
      const { target, resolvedTraces } = getPayload(selection, 'trace_target');
      return (
        <TraceTargetBody
          target={target}
          resolvedTraces={resolvedTraces}
          onViewSource={onViewSource}
          onLocateEvidence={onLocateEvidence}
        />
      );
    }
    case 'source_excerpt': {
      const { excerpt } = getPayload(selection, 'source_excerpt');
      return <SourceExcerptBody excerpt={excerpt} />;
    }
    case 'quality_issue': {
      const { issue } = getPayload(selection, 'quality_issue');
      return (
        <QualityIssueBody
          issue={issue}
          onViewSource={onViewSource}
          onLocateEvidence={onLocateEvidence}
        />
      );
    }
    case 'view_node': {
      const { node, viewType } = getPayload(selection, 'view_node');
      return <ViewNodeBody node={node} viewType={viewType} />;
    }
    case 'view_edge': {
      const { edge, viewType } = getPayload(selection, 'view_edge');
      return <ViewEdgeBody edge={edge} viewType={viewType} />;
    }
    case 'qa_citation': {
      const { citation } = getPayload(selection, 'qa_citation');
      return (
        <QaCitationBody
          citation={citation}
          onViewSource={onViewSource}
        />
      );
    }
    default:
      return null;
  }
}

// ─── Evidence ───────────────────────────────────────────────────────────

function EvidenceBody({
  item,
  onViewSource,
  onLocateEvidence,
}: {
  item: EvidenceItem;
  onViewSource?: ContextPanelProps['onViewSource'];
  onLocateEvidence?: ContextPanelProps['onLocateEvidence'];
}) {
  const strengthColor = EVIDENCE_STRENGTH_COLOR[item.strength] ?? '#9e9e9e';
  const strengthLabel = EVIDENCE_STRENGTH_LABEL[item.strength] ?? item.strength;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
        <code style={{ fontSize: 11, color: '#64748b' }}>{item.evidence_id}</code>
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
        <span
          style={{
            padding: '2px 8px',
            background: '#e8eaf6',
            color: '#283593',
            borderRadius: 3,
            fontSize: 11,
          }}
        >
          {SOURCE_KIND_LABEL[item.source_kind] ?? item.source_kind}
        </span>
      </div>

      {item.symbol && (
        <div style={{ fontSize: 14, fontWeight: 600, color: '#1e293b' }}>{item.symbol}</div>
      )}

      <div style={{ fontSize: 13, color: '#334155', lineHeight: 1.5 }}>{item.summary}</div>

      <div style={metaBlockStyle}>
        <MetaRow label="路径" value={item.source_path} />
        <MetaRow
          label="行号"
          value={`${item.line_range.start}–${item.line_range.end}`}
        />
        <MetaRow label="语言" value={LANGUAGE_LABEL[item.language] ?? item.language} />
      </div>

      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        <ActionButton
          label="查看源码片段"
          onClick={() =>
            onViewSource?.({
              source_path: item.source_path,
              line_range: item.line_range,
              evidence_id: item.evidence_id,
            })
          }
        />
        <ActionButton
          label="定位 evidence"
          variant="secondary"
          onClick={() => onLocateEvidence?.(item.evidence_id)}
        />
      </div>
    </div>
  );
}

// ─── Trace Target ───────────────────────────────────────────────────────

function TraceTargetBody({
  target,
  resolvedTraces,
  onViewSource,
  onLocateEvidence,
}: {
  target: SelectedTraceTarget;
  resolvedTraces: TraceRefResolved[];
  onViewSource?: ContextPanelProps['onViewSource'];
  onLocateEvidence?: ContextPanelProps['onLocateEvidence'];
}) {
  const targetLabel = traceTargetTypeLabel(target.kind);
  const targetId =
    target.kind === 'view_node' || target.kind === 'view_edge'
      ? `${VIEW_TYPE_LABEL[target.view_type] ?? target.view_type} · ${
          target.kind === 'view_node' ? target.node_id : target.edge_id
        }`
      : target.kind === 'claim'
      ? target.claim_id
      : target.evidence_id;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 12 }}>
      <div style={{ ...metaBlockStyle, background: '#f1f5f9' }}>
        <MetaRow label="类型" value={targetLabel} />
        <MetaRow label="标识" value={targetId} />
        <MetaRow label="解析结果" value={`${resolvedTraces.length} 条`} />
      </div>

      {resolvedTraces.length === 0 ? (
        <div style={{ fontSize: 12, color: '#94a3b8' }}>正在解析或无可解析的追溯…</div>
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
          {resolvedTraces.map((trace, idx) => (
            <TraceCard
              key={`${trace.source_kind}-${idx}`}
              trace={trace}
              onViewSource={onViewSource}
              onLocateEvidence={onLocateEvidence}
            />
          ))}
        </div>
      )}
    </div>
  );
}

function TraceCard({
  trace,
  onViewSource,
  onLocateEvidence,
}: {
  trace: TraceRefResolved;
  onViewSource?: ContextPanelProps['onViewSource'];
  onLocateEvidence?: ContextPanelProps['onLocateEvidence'];
}) {
  return (
    <div
      style={{
        padding: 10,
        background: '#fff',
        border: '1px solid #e2e8f0',
        borderRadius: 6,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap', marginBottom: 6 }}>
        <ResolutionTag resolution={trace.resolution} />
        <ConfidenceTag confidence={trace.confidence} />
      </div>

      {trace.claim && (
        <div style={{ fontSize: 12, color: '#475569', marginBottom: 6 }}>
          <code style={{ color: '#64748b' }}>{trace.claim.claim_id}</code>
          <div>{trace.claim.description}</div>
        </div>
      )}

      {trace.evidence && (
        <div style={{ fontSize: 12, color: '#475569' }}>
          <code style={{ color: '#64748b' }}>{trace.evidence.evidence_id}</code>
          <div>{trace.evidence.summary}</div>
          <div style={{ color: '#94a3b8', marginTop: 4 }}>
            {trace.evidence.source_path.split('/').pop() ?? trace.evidence.source_path} · 行{' '}
            {trace.evidence.line_range.start}–{trace.evidence.line_range.end}
          </div>
          <div style={{ display: 'flex', gap: 6, marginTop: 6 }}>
            <SmallAction
              label="源码"
              onClick={() =>
                onViewSource?.({
                  source_path: trace.evidence!.source_path,
                  line_range: trace.evidence!.line_range,
                  evidence_id: trace.evidence!.evidence_id,
                })
              }
            />
            <SmallAction
              label="定位"
              onClick={() => onLocateEvidence?.(trace.evidence!.evidence_id)}
            />
          </div>
        </div>
      )}
    </div>
  );
}

// ─── Source Excerpt ─────────────────────────────────────────────────────

function SourceExcerptBody({ excerpt }: { excerpt: SourceExcerpt }) {
  const previewLines = excerpt.lines.slice(0, 5);
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span
          style={{
            padding: '2px 8px',
            background: '#e3f2fd',
            color: '#1565c0',
            borderRadius: 4,
            fontSize: 11,
          }}
        >
          {LANGUAGE_LABEL[excerpt.language] ?? excerpt.language}
        </span>
      </div>

      <div style={metaBlockStyle}>
        <MetaRow label="路径" value={excerpt.location.source_path} />
        <MetaRow
          label="行号"
          value={`${excerpt.location.line_range.start}–${excerpt.location.line_range.end}`}
        />
        {excerpt.location.evidence_id && (
          <MetaRow label="关联 evidence" value={excerpt.location.evidence_id} />
        )}
      </div>

      <div
        style={{
          background: '#f8fafc',
          border: '1px solid #e2e8f0',
          borderRadius: 6,
          fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
          fontSize: 12,
          lineHeight: 1.5,
          maxHeight: 200,
          overflow: 'auto',
        }}
      >
        {previewLines.map((line) => (
          <div key={line.line_number} style={{ display: 'flex' }}>
            <div
              style={{
                minWidth: 36,
                padding: '2px 8px',
                textAlign: 'right',
                color: '#94a3b8',
                background: '#f1f5f9',
                userSelect: 'none',
                borderRight: '1px solid #e2e8f0',
              }}
            >
              {line.line_number}
            </div>
            <pre
              style={{
                margin: 0,
                padding: '2px 10px',
                flex: 1,
                whiteSpace: 'pre',
                color: '#334155',
              }}
            >
              {line.content}
            </pre>
          </div>
        ))}
        {excerpt.lines.length > 5 && (
          <div style={{ padding: '6px 10px', color: '#94a3b8', fontSize: 11 }}>
            …共 {excerpt.lines.length} 行
          </div>
        )}
      </div>

      {excerpt.is_truncated && excerpt.truncation_reason && (
        <div style={{ fontSize: 11, color: '#f57c00' }}>
          已截断：{excerpt.truncation_reason}
        </div>
      )}
    </div>
  );
}

// ─── Quality Issue ──────────────────────────────────────────────────────

function QualityIssueBody({
  issue,
  onViewSource,
  onLocateEvidence,
}: {
  issue: QualityIssue;
  onViewSource?: ContextPanelProps['onViewSource'];
  onLocateEvidence?: ContextPanelProps['onLocateEvidence'];
}) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
        <span
          style={{
            padding: '2px 8px',
            background: SEVERITY_COLOR[issue.severity] ?? '#9e9e9e',
            color: '#fff',
            borderRadius: 3,
            fontSize: 11,
            fontWeight: 600,
          }}
        >
          {issue.severity}
        </span>
        <span
          style={{
            padding: '2px 8px',
            background: '#e8eaf6',
            color: '#283593',
            borderRadius: 3,
            fontSize: 11,
          }}
        >
          {QUALITY_KIND_LABEL[issue.kind] ?? issue.kind}
        </span>
        <span
          style={{
            padding: '2px 8px',
            background: issue.polarity === 'positive_guardrail' ? '#e8f5e9' : '#fff3e0',
            color: issue.polarity === 'positive_guardrail' ? '#2e7d32' : '#e65100',
            borderRadius: 3,
            fontSize: 11,
          }}
        >
          {POLARITY_LABEL[issue.polarity]}
        </span>
        <span style={{ fontSize: 11, color: '#94a3b8' }}>
          {ISSUE_STATUS_LABEL[issue.status] ?? issue.status}
        </span>
      </div>

      <div style={{ fontSize: 13, color: '#334155', lineHeight: 1.5 }}>{issue.description}</div>

      <div style={metaBlockStyle}>
        <MetaRow label="产物类型" value={issue.artifact_kind} />
        {issue.evidence_id && <MetaRow label="关联 evidence" value={issue.evidence_id} />}
        {issue.claim_id && <MetaRow label="关联 claim" value={issue.claim_id} />}
        {issue.node_id && <MetaRow label="关联 node" value={issue.node_id} />}
        {issue.source_path && <MetaRow label="路径" value={issue.source_path} />}
      </div>

      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        {issue.evidence_id && (
          <ActionButton
            label="定位 evidence"
            variant="secondary"
            onClick={() => onLocateEvidence?.(issue.evidence_id!)}
          />
        )}
        {issue.source_path && (
          <ActionButton
            label="查看源码片段"
            onClick={() =>
              onViewSource?.({
                source_path: issue.source_path!,
                line_range: issue.line_range ?? { start: 1, end: 1 },
                evidence_id: issue.evidence_id,
              })
            }
          />
        )}
      </div>
    </div>
  );
}

// ─── View Node / Edge ───────────────────────────────────────────────────

function ViewNodeBody({ node, viewType }: { node: ViewNode; viewType: import('../../../types/workspace').ViewType }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
        <span
          style={{
            padding: '2px 8px',
            background: '#e8eaf6',
            color: '#283593',
            borderRadius: 3,
            fontSize: 11,
          }}
        >
          {NODE_TYPE_LABEL[node.node_type] ?? node.node_type}
        </span>
        <span style={{ fontSize: 11, color: '#94a3b8' }}>
          {VIEW_TYPE_LABEL[viewType] ?? viewType}
        </span>
      </div>

      <div style={{ fontSize: 14, fontWeight: 600, color: '#1e293b' }}>{node.label}</div>
      <div style={{ fontSize: 13, color: '#475569' }}>{node.description}</div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <ConfidenceTag confidence={node.confidence} />
        <span style={{ fontSize: 12, color: '#64748b' }}>
          追溯引用 {node.trace_refs.length} 条
        </span>
      </div>
    </div>
  );
}

function ViewEdgeBody({ edge, viewType }: { edge: ViewEdge; viewType: import('../../../types/workspace').ViewType }) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 6, flexWrap: 'wrap' }}>
        <span
          style={{
            padding: '2px 8px',
            background: '#e8eaf6',
            color: '#283593',
            borderRadius: 3,
            fontSize: 11,
          }}
        >
          {EDGE_TYPE_LABEL[edge.edge_type] ?? edge.edge_type}
        </span>
        <span style={{ fontSize: 11, color: '#94a3b8' }}>
          {VIEW_TYPE_LABEL[viewType] ?? viewType}
        </span>
      </div>

      <div style={{ fontSize: 14, fontWeight: 600, color: '#1e293b' }}>
        {edge.label ?? edge.edge_id}
      </div>
      <div style={{ fontSize: 12, color: '#64748b' }}>
        {edge.source_node_id} → {edge.target_node_id}
      </div>
      <div style={{ fontSize: 13, color: '#475569' }}>{edge.description}</div>

      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <ConfidenceTag confidence={edge.confidence} />
        <span style={{ fontSize: 12, color: '#64748b' }}>
          追溯引用 {edge.trace_refs.length} 条
        </span>
      </div>
    </div>
  );
}

// ─── Q&A Citation ───────────────────────────────────────────────────────

function QaCitationBody({
  citation,
  onViewSource,
}: {
  citation: GroundedAnswerCitation;
  onViewSource?: ContextPanelProps['onViewSource'];
}) {
  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
        <span
          style={{
            minWidth: 20,
            height: 20,
            borderRadius: 10,
            background: '#1976d2',
            color: '#fff',
            fontSize: 11,
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
          }}
        >
          {citation.index}
        </span>
        {citation.evidence_id && <code style={{ fontSize: 11, color: '#64748b' }}>{citation.evidence_id}</code>}
        {citation.claim_id && !citation.evidence_id && (
          <code style={{ fontSize: 11, color: '#64748b' }}>{citation.claim_id}</code>
        )}
      </div>

      <div style={{ fontSize: 13, color: '#334155', lineHeight: 1.5 }}>
        {citation.excerpt_summary}
      </div>

      {citation.source_location && (
        <div style={metaBlockStyle}>
          <MetaRow label="路径" value={citation.source_location.source_path} />
          <MetaRow
            label="行号"
            value={`${citation.source_location.line_range.start}–${citation.source_location.line_range.end}`}
          />
        </div>
      )}

      {citation.source_location && (
        <ActionButton
          label="查看源码片段"
          onClick={() =>
            onViewSource?.({
              source_path: citation.source_location!.source_path,
              line_range: citation.source_location!.line_range,
              evidence_id: citation.evidence_id,
            })
          }
        />
      )}
    </div>
  );
}

// ─── 通用辅助组件 ───────────────────────────────────────────────────────

function MetaRow({ label, value }: { label: string; value: string }) {
  return (
    <div style={{ fontSize: 12, color: '#475569', wordBreak: 'break-all' }}>
      <span style={{ color: '#94a3b8' }}>{label}：</span>
      {label === '路径' || label === '关联 evidence' || label === '关联 claim' ? (
        <code style={{ fontSize: 11 }}>{value}</code>
      ) : (
        value
      )}
    </div>
  );
}

function ConfidenceTag({ confidence }: { confidence: ClaimConfidence }) {
  return (
    <span
      style={{
        padding: '2px 8px',
        borderRadius: 4,
        fontSize: 11,
        fontWeight: 600,
        background: CONFIDENCE_BG[confidence] ?? '#f5f5f5',
        color: CONFIDENCE_COLOR[confidence] ?? '#757575',
      }}
    >
      {CONFIDENCE_LABEL[confidence] ?? confidence}
    </span>
  );
}

function ResolutionTag({ resolution }: { resolution: TraceResolution }) {
  return (
    <span
      style={{
        padding: '2px 8px',
        borderRadius: 4,
        fontSize: 11,
        fontWeight: 600,
        background: '#fff',
        color: RESOLUTION_COLOR[resolution] ?? '#757575',
        border: `1px solid ${RESOLUTION_COLOR[resolution] ?? '#bdbdbd'}`,
      }}
    >
      {RESOLUTION_LABEL[resolution] ?? resolution}
    </span>
  );
}

function ActionButton({
  label,
  onClick,
  variant = 'primary',
}: {
  label: string;
  onClick?: () => void;
  variant?: 'primary' | 'secondary';
}) {
  const isPrimary = variant === 'primary';
  return (
    <button
      onClick={onClick}
      style={{
        padding: '5px 12px',
        borderRadius: 4,
        border: `1px solid ${isPrimary ? '#1976d2' : '#f57c00'}`,
        background: '#fff',
        color: isPrimary ? '#1976d2' : '#f57c00',
        cursor: 'pointer',
        fontSize: 12,
      }}
    >
      {label}
    </button>
  );
}

function SmallAction({ label, onClick }: { label: string; onClick?: () => void }) {
  return (
    <button
      onClick={onClick}
      style={{
        padding: '3px 8px',
        borderRadius: 3,
        border: '1px solid #1976d2',
        background: '#fff',
        color: '#1976d2',
        cursor: 'pointer',
        fontSize: 11,
      }}
    >
      {label}
    </button>
  );
}

// ─── 样式 ─────────────────────────────────────────────────────────────────

const containerStyle: React.CSSProperties = {
  width: 280,
  minWidth: 240,
  maxWidth: 320,
  background: '#fff',
  borderLeft: '1px solid #e2e8f0',
  flexShrink: 0,
  overflowY: 'auto',
  display: 'flex',
  flexDirection: 'column',
};

const emptyStyle: React.CSSProperties = {
  flex: 1,
  display: 'flex',
  flexDirection: 'column',
  alignItems: 'center',
  justifyContent: 'center',
  padding: 24,
  textAlign: 'center',
};

const metaBlockStyle: React.CSSProperties = {
  padding: 10,
  background: '#f8fafc',
  border: '1px solid #e2e8f0',
  borderRadius: 6,
  display: 'flex',
  flexDirection: 'column',
  gap: 4,
};
