import type {
  StageContext,
  EvidenceCollection,
  ImplementationUnderstanding,
  ViewGraph,
  QaHistory,
  QualityReport,
  StageStatus,
} from '../../../types/workspace';

interface StageOverviewBarProps {
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
}

const STATUS_LABEL: Record<StageStatus, string> = {
  available: '可用',
  empty: '空阶段',
  missing: '缺失',
  naming_anomaly: '命名异常',
  unreadable: '不可读',
};

const STATUS_COLOR: Record<StageStatus, string> = {
  available: '#4caf50',
  empty: '#9e9e9e',
  missing: '#f44336',
  naming_anomaly: '#ff9800',
  unreadable: '#c62828',
};

export default function StageOverviewBar({
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
}: StageOverviewBarProps) {
  const evidenceCount = evidence?.evidence_items.length ?? 0;
  const claimCount = understanding?.claims.length ?? 0;
  const viewCount = views?.length ?? 0;
  const qaCount = qaHistory?.entries.length ?? 0;
  const qualityIssueCount = qualityReport?.issues.length ?? 0;
  const fileCount = context.files.length;

  const metrics: Array<{ label: string; value: number | string; loading?: boolean }> = [
    { label: '阶段', value: stageId },
    { label: '状态', value: STATUS_LABEL[stageStatus] ?? stageStatus },
    { label: '文件', value: fileCount },
    { label: '证据', value: evidenceCount, loading: evidenceLoading },
    { label: '声明', value: claimCount, loading: understandingLoading },
    { label: '视图', value: viewCount, loading: viewsLoading },
    { label: 'Q&A', value: qaCount, loading: qaLoading },
    { label: '质量问题', value: qualityIssueCount, loading: qualityLoading },
  ];

  return (
    <div
      className="stage-overview-bar"
      style={{
        display: 'flex',
        alignItems: 'center',
        gap: 12,
        overflowX: 'auto',
      }}
    >
      {metrics.map((m, i) => (
        <div
          key={i}
          style={{
            display: 'flex',
            flexDirection: 'column',
            minWidth: 64,
            padding: '6px 10px',
            background: '#fff',
            border: '1px solid #e2e8f0',
            borderRadius: 6,
          }}
        >
          <span style={{ fontSize: 11, color: '#94a3b8', marginBottom: 2 }}>{m.label}</span>
          <span
            style={{
              fontSize: 14,
              fontWeight: 600,
              color:
                m.label === '状态'
                  ? STATUS_COLOR[stageStatus] ?? '#1e293b'
                  : '#1e293b',
              whiteSpace: 'nowrap',
            }}
          >
            {m.loading ? '...' : m.value}
          </span>
        </div>
      ))}
    </div>
  );
}
