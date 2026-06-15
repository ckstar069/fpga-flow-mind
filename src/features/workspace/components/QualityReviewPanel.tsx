import type {
  QualityReport,
  QualityIssue,
  EvidenceQualityReport,
  UnderstandingQualityReport,
  ViewQualityReport,
  QaQualityReport,
  QualityAcceptanceStatus,
  QualitySeverity,
  QualityIssueKind,
} from '../../../types/workspace';
import type { UiError } from '../workspaceUiTypes';

interface QualityReviewPanelProps {
  report?: QualityReport | null;
  loading?: boolean;
  error?: UiError | null;
  canGenerate: boolean;
  disabledReason?: string;
  onGenerate: () => void;
  onEvidenceSelect?: (evidenceId: string) => void;
  onViewSource?: (location: {
    source_path: string;
    line_range: { start: number; end: number };
    evidence_id?: string;
  }) => void;
}

export default function QualityReviewPanel({
  report,
  loading,
  error,
  canGenerate,
  disabledReason,
  onGenerate,
  onEvidenceSelect,
  onViewSource,
}: QualityReviewPanelProps) {
  const acceptanceLabel = (status: QualityAcceptanceStatus): string => {
    switch (status) {
      case 'meets_gate':
        return '达到当前质量门槛';
      case 'below_gate':
        return '低于当前质量门槛';
      default:
        return '未评估';
    }
  };

  const severityColor = (severity: QualitySeverity): string => {
    switch (severity) {
      case 'high':
        return '#c62828';
      case 'medium':
        return '#f57c00';
      case 'low':
        return '#546e7a';
      default:
        return '#999';
    }
  };

  const kindLabel = (kind: QualityIssueKind): string => {
    const labels: Record<QualityIssueKind, string> = {
      missing_evidence: '缺失证据',
      noisy_evidence: '噪声证据',
      wrong_source_kind: '源类型标注异常',
      stage_identification_mismatch: '阶段识别不一致',
      weak_summary: '摘要偏弱',
      unsupported_claim: '无证据声明',
      hallucinated_claim_blocked: '幻觉声明被拦截',
      empty_or_unhelpful_view: '视图退化',
      qa_unanswered_when_evidence_exists: '有证据未回答',
      qa_answer_without_valid_citation: '引用无效',
      confusing_ui_state: 'UI 状态不清',
    };
    return labels[kind] ?? kind;
  };

  const handleIssueClick = (issue: QualityIssue) => {
    if (issue.evidence_id && onEvidenceSelect) {
      onEvidenceSelect(issue.evidence_id);
    }
    if (issue.source_path && onViewSource) {
      onViewSource({
        source_path: issue.source_path,
        line_range: issue.line_range ?? { start: 1, end: 1 },
        evidence_id: issue.evidence_id,
      });
    }
  };

  return (
    <div style={{ marginBottom: 24 }}>
      <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>质量评估（Phase 7）</h3>

      <button
        onClick={onGenerate}
        disabled={!canGenerate || loading}
        title={disabledReason}
        style={{
          padding: '8px 20px',
          borderRadius: 6,
          border: report ? '1px solid #2e7d32' : '1px solid #1976d2',
          background: loading ? '#e0e0e0' : report ? '#2e7d32' : '#1976d2',
          color: loading ? '#999' : '#fff',
          cursor: !canGenerate || loading ? 'not-allowed' : 'pointer',
          fontSize: 14,
        }}
      >
        {loading ? '生成中...' : report ? '重新生成质量报告' : '生成质量报告'}
      </button>

      {!canGenerate && disabledReason && (
        <span style={{ fontSize: 12, color: '#999', marginLeft: 12 }}>{disabledReason}</span>
      )}

      {error && (
        <div
          style={{
            marginTop: 16,
            padding: 16,
            background: '#ffebee',
            borderRadius: 8,
            border: '1px solid #ef9a9a',
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 14, color: '#c62828' }}>质量报告生成失败</h4>
          <p style={{ margin: 0, fontSize: 13 }}>{error.message}</p>
          {'error_code' in error && error.error_code && (
            <code style={{ fontSize: 12, color: '#666' }}>{error.error_code}</code>
          )}
        </div>
      )}

      {!report && !loading && !error && (
        <div
          style={{
            marginTop: 16,
            padding: 24,
            background: '#fafafa',
            borderRadius: 8,
            textAlign: 'center',
            color: '#999',
          }}
        >
          <p style={{ margin: 0, fontSize: 13 }}>点击上方按钮生成当前阶段的质量评估报告</p>
        </div>
      )}

      {report && (
        <div style={{ marginTop: 16, display: 'flex', flexDirection: 'column', gap: 16 }}>
          {/* 总体状态 */}
          <div
            style={{
              padding: 16,
              background: report.acceptance === 'meets_gate' ? '#e8f5e9' : '#ffebee',
              borderRadius: 8,
              border: `1px solid ${report.acceptance === 'meets_gate' ? '#81c784' : '#ef9a9a'}`,
            }}
          >
            <div style={{ fontSize: 14, fontWeight: 600 }}>{acceptanceLabel(report.acceptance)}</div>
            <div style={{ fontSize: 12, color: '#666', marginTop: 4 }}>
              负向问题：{report.summary.total_issues} · 正向守卫：
              {report.summary.positive_guardrail_event_count}
            </div>
          </div>

          {/* 汇总 */}
          <div
            style={{
              padding: 16,
              background: '#fff',
              borderRadius: 8,
              border: '1px solid #e0e0e0',
            }}
          >
            <h4 style={{ margin: '0 0 12px', fontSize: 14 }}>汇总</h4>
            <div style={{ display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(160px, 1fr))', gap: 12 }}>
              <StatBox label="负向问题" value={String(report.summary.total_issues)} />
              <StatBox
                label="正向守卫"
                value={String(report.summary.positive_guardrail_event_count)}
              />
              <StatBox
                label="维度指标"
                value={String(report.summary.metric_snapshots.length)}
              />
              <StatBox label="质量记录" value={String(report.issues.length)} />
            </div>

            {<ObjectList title="按分类" data={report.summary.issues_by_kind} />}
            {<ObjectList title="按严重程度" data={report.summary.issues_by_severity} />}
            {<ObjectList title="按状态" data={report.summary.issues_by_status} />}
          </div>

          {/* 分维度 */}
          <div
            style={{
              padding: 16,
              background: '#fff',
              borderRadius: 8,
              border: '1px solid #e0e0e0',
            }}
          >
            <h4 style={{ margin: '0 0 12px', fontSize: 14 }}>分维度概览</h4>
            {report.evidence_reports.map((r) => (
              <EvidenceReportView key={`ev-${r.stage_id}`} report={r} />
            ))}
            {report.understanding_reports.map((r) => (
              <UnderstandingReportView key={`un-${r.stage_id}`} report={r} />
            ))}
            {report.view_reports.map((r) => (
              <ViewReportView key={`view-${r.stage_id}-${r.view_type}`} report={r} />
            ))}
            {report.qa_reports.map((r) => (
              <QaReportView key={`qa-${r.stage_id}`} report={r} />
            ))}
          </div>

          {/* Issues 列表 */}
          <div
            style={{
              padding: 16,
              background: '#fff',
              borderRadius: 8,
              border: '1px solid #e0e0e0',
            }}
          >
            <h4 style={{ margin: '0 0 12px', fontSize: 14 }}>质量记录 ({report.issues.length})</h4>
            {report.issues.length === 0 ? (
              <p style={{ margin: 0, fontSize: 13, color: '#666' }}>未发现质量问题或守卫记录。</p>
            ) : (
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {report.issues.map((issue) => (
                  <button
                    key={issue.issue_id}
                    onClick={() => handleIssueClick(issue)}
                    disabled={!issue.evidence_id && !issue.source_path}
                    style={{
                      textAlign: 'left',
                      padding: 12,
                      borderRadius: 6,
                      border: '1px solid #e0e0e0',
                      background: issue.polarity === 'positive_guardrail' ? '#f1f8e9' : '#fff',
                      cursor:
                        issue.evidence_id || issue.source_path ? 'pointer' : 'default',
                      display: 'flex',
                      flexDirection: 'column',
                      gap: 6,
                    }}
                  >
                    <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
                      <span
                        style={{
                          fontSize: 12,
                          padding: '2px 6px',
                          borderRadius: 4,
                          background: severityColor(issue.severity),
                          color: '#fff',
                        }}
                      >
                        {issue.severity}
                      </span>
                      <span style={{ fontSize: 12, color: '#666' }}>{issue.artifact_kind}</span>
                      <span style={{ fontSize: 13, fontWeight: 500 }}>{kindLabel(issue.kind)}</span>
                      {issue.polarity === 'positive_guardrail' && (
                        <span style={{ fontSize: 11, color: '#558b2f' }}>正向守卫</span>
                      )}
                    </div>
                    <div style={{ fontSize: 13, color: '#333', wordBreak: 'break-word' }}>
                      {issue.description}
                    </div>
                    <div style={{ fontSize: 12, color: '#999' }}>
                      stage={issue.stage_id}
                      {issue.evidence_id && ` · evidence=${issue.evidence_id}`}
                      {issue.claim_id && ` · claim=${issue.claim_id}`}
                      {issue.node_id && ` · node=${issue.node_id}`}
                      {issue.source_path && ` · ${issue.source_path}`}
                    </div>
                  </button>
                ))}
              </div>
            )}
          </div>
        </div>
      )}
    </div>
  );
}

function StatBox({ label, value }: { label: string; value: string }) {
  return (
    <div
      style={{
        padding: 12,
        background: '#f5f5f5',
        borderRadius: 6,
        textAlign: 'center',
      }}
    >
      <div style={{ fontSize: 18, fontWeight: 600 }}>{value}</div>
      <div style={{ fontSize: 11, color: '#666', marginTop: 4 }}>{label}</div>
    </div>
  );
}

function ObjectList({ title, data }: { title: string; data: Record<string, number> }) {
  const entries = Object.entries(data);
  if (entries.length === 0) return null;
  return (
    <div style={{ marginTop: 12 }}>
      <div style={{ fontSize: 12, color: '#666', marginBottom: 6 }}>{title}</div>
      <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
        {entries.map(([key, count]) => (
          <span
            key={key}
            style={{
              fontSize: 12,
              padding: '4px 8px',
              background: '#f5f5f5',
              borderRadius: 4,
            }}
          >
            {key}: {count}
          </span>
        ))}
      </div>
    </div>
  );
}

function EvidenceReportView({ report }: { report: EvidenceQualityReport }) {
  return (
    <div style={{ marginBottom: 10 }}>
      <div style={{ fontSize: 13, fontWeight: 500 }}>Evidence · {report.stage_id}</div>
      <div style={{ fontSize: 12, color: '#666' }}>
        文件覆盖率：{formatRatio(report.file_coverage_ratio)} · 行号准确率：
        {formatRatio(report.line_range_accuracy)} · 标注合理率：
        {formatRatio(report.label_sanity_ratio)}
      </div>
      {report.uncovered_files.length > 0 && (
        <div style={{ fontSize: 12, color: '#999', marginTop: 4 }}>
          未覆盖文件：{report.uncovered_files.map((f) => f.source_path).join(', ')}
        </div>
      )}
    </div>
  );
}

function UnderstandingReportView({ report }: { report: UnderstandingQualityReport }) {
  return (
    <div style={{ marginBottom: 10 }}>
      <div style={{ fontSize: 13, fontWeight: 500 }}>Understanding · {report.stage_id}</div>
      <div style={{ fontSize: 12, color: '#666' }}>
        claim 存在性：{formatRatio(report.claim_existence_check_ratio)} · 不确定性表达：
        {formatRatio(report.uncertainty_expression_ratio)} · 置信度校准：
        {formatRatio(report.confidence_calibration_ratio)}
      </div>
    </div>
  );
}

function ViewReportView({ report }: { report: ViewQualityReport }) {
  return (
    <div style={{ marginBottom: 10 }}>
      <div style={{ fontSize: 13, fontWeight: 500 }}>View · {report.view_type} · {report.stage_id}</div>
      <div style={{ fontSize: 12, color: '#666' }}>
        追溯可解析率：{formatRatio(report.trace_resolvable_ratio)} · 孤立节点：
        {report.isolated_node_count} · 错连嫌疑：{report.suspected_misconnection_count}
      </div>
    </div>
  );
}

function QaReportView({ report }: { report: QaQualityReport }) {
  return (
    <div style={{ marginBottom: 10 }}>
      <div style={{ fontSize: 13, fontWeight: 500 }}>Q&A · {report.stage_id}</div>
      <div style={{ fontSize: 12, color: '#666' }}>
        引用有效：{formatRatio(report.citation_validity_ratio)} · 有证据命中：
        {formatRatio(report.answerable_hit_ratio)} · 诚实 unknown：
        {formatRatio(report.unknown_honesty_ratio)}
      </div>
    </div>
  );
}

function formatRatio(value: number): string {
  return `${(value * 100).toFixed(1)}%`;
}
