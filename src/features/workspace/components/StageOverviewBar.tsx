import type {
  StageContext,
  EvidenceCollection,
  ImplementationUnderstanding,
  ViewGraph,
  QaHistory,
  QualityReport,
  StageStatus,
} from '../../../types/workspace';
import { SURFACE, ACCENT, FONT } from './workbenchTheme';

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

// 阶段状态视觉色：不使用红绿裁决色。
// available=蓝（可用焦点）；naming_anomaly=琥珀（需注意）；empty/missing/unreadable=灰（中性弱化）。
const STATUS_COLOR: Record<StageStatus, string> = {
  available: ACCENT.blue,
  empty: SURFACE.textDim,
  missing: SURFACE.textDim,
  naming_anomaly: ACCENT.amber,
  unreadable: SURFACE.textDim,
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

  const metrics: Array<{ label: string; value: number | string; loading?: boolean; accent?: boolean }> = [
    { label: '阶段', value: stageId, accent: true },
    { label: '状态', value: STATUS_LABEL[stageStatus] ?? stageStatus },
    { label: '文件', value: fileCount },
    { label: '证据', value: evidenceCount, loading: evidenceLoading },
    { label: '声明', value: claimCount, loading: understandingLoading },
    { label: '视图', value: viewCount, loading: viewsLoading },
    { label: 'Q&A', value: qaCount, loading: qaLoading },
    { label: '质量记录', value: qualityIssueCount, loading: qualityLoading },
  ];

  return (
    <div
      className="stage-overview-bar"
      style={{
        display: 'flex',
        alignItems: 'stretch',
        gap: 8,
        overflowX: 'auto',
      }}
    >
      {metrics.map((m, i) => {
        const isStatus = m.label === '状态';
        const statusColor = STATUS_COLOR[stageStatus] ?? SURFACE.text;
        return (
          <div
            key={i}
            style={{
              display: 'flex',
              flexDirection: 'column',
              justifyContent: 'center',
              minWidth: 60,
              padding: '6px 12px',
              background: SURFACE.bgSubtle,
              border: `1px solid ${SURFACE.border}`,
              borderRadius: 6,
            }}
          >
            <span style={{ fontSize: FONT.micro, color: SURFACE.textDim, marginBottom: 2 }}>
              {m.label}
            </span>
            <span
              style={{
                fontSize: m.accent ? FONT.heading : FONT.body,
                fontWeight: 600,
                color: isStatus ? statusColor : SURFACE.text,
                whiteSpace: 'nowrap',
              }}
            >
              {m.loading ? '…' : m.value}
            </span>
          </div>
        );
      })}
    </div>
  );
}
