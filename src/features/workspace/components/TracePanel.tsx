import type {
  TraceRefResolved,
  ClaimSnapshot,
  EvidenceSnapshot,
  TraceResolution,
  ClaimConfidence,
  EvidenceStrength,
} from '../../../types/workspace';

// ─── 中文标签映射 ───────────────────────────────────────────────────────

const SOURCE_KIND_LABEL: Record<string, string> = {
  python_stage: 'Python 阶段',
  rtl: 'RTL',
  test: '测试',
  doc: '文档',
  config: '配置',
  external_module: '外部模块',
};

const CONFIDENCE_LABEL: Record<string, string> = {
  confirmed: '已确认',
  supported: '有支撑',
  inferred: '推断',
  unknown: '未知',
  conflicting: '矛盾',
};

const CONFIDENCE_BG: Record<string, string> = {
  confirmed: '#e3f2fd',
  supported: '#e8f5e9',
  inferred: '#fff3e0',
  unknown: '#f5f5f5',
  conflicting: '#ffebee',
};

const CONFIDENCE_COLOR: Record<string, string> = {
  confirmed: '#1565c0',
  supported: '#2e7d32',
  inferred: '#f57c00',
  unknown: '#757575',
  conflicting: '#c62828',
};

const RESOLUTION_LABEL: Record<TraceResolution, string> = {
  resolved: '已解析',
  claim_only: '仅声明',
  evidence_only: '仅证据',
  missing_claim: '声明缺失',
  missing_evidence: '证据缺失',
};

const RESOLUTION_COLOR: Record<TraceResolution, string> = {
  resolved: '#2e7d32',
  claim_only: '#f57c00',
  evidence_only: '#1565c0',
  missing_claim: '#c62828',
  missing_evidence: '#c62828',
};

const STRENGTH_LABEL: Record<EvidenceStrength, string> = {
  direct: '直接',
  indirect: '间接',
  weak: '弱',
  conflicting: '冲突',
  missing: '缺失',
};

const CATEGORY_LABEL: Record<string, string> = {
  module_structure: '模块结构',
  signal_definition: '信号定义',
  interface_description: '接口描述',
  data_processing: '数据处理',
  configuration: '配置',
  documentation: '文档',
  test_coverage: '测试覆盖',
  other: '其他',
};

// ─── Props ──────────────────────────────────────────────────────────────

interface TracePanelProps {
  selectedTargetLabel: string;
  selectedTargetType: string;
  resolvedTraces: TraceRefResolved[];
  loading?: boolean;
  error?: { message: string; error_code?: string; source_path?: string; details?: string } | null;
  onClear: () => void;
  onViewSource: (location: {
    source_path: string;
    line_range: { start: number; end: number };
    evidence_id?: string;
  }) => void;
  onLocateEvidence: (evidenceId: string) => void;
}

// ─── 主组件 ─────────────────────────────────────────────────────────────

export default function TracePanel({
  selectedTargetLabel,
  selectedTargetType,
  resolvedTraces,
  loading,
  error,
  onClear,
  onViewSource,
  onLocateEvidence,
}: TracePanelProps) {
  return (
    <div style={{ marginBottom: 24 }}>
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          marginBottom: 12,
        }}
      >
        <h3 style={{ fontSize: 15, margin: 0 }}>追溯详情</h3>
        <button
          onClick={onClear}
          disabled={loading}
          style={{
            padding: '4px 12px',
            borderRadius: 4,
            border: '1px solid #ccc',
            background: '#fff',
            cursor: loading ? 'not-allowed' : 'pointer',
            fontSize: 12,
            color: loading ? '#999' : '#333',
          }}
        >
          清空选择
        </button>
      </div>

      {/* 选中目标摘要 */}
      <div
        style={{
          padding: '10px 14px',
          background: '#f5f5f5',
          borderRadius: 6,
          marginBottom: 12,
          fontSize: 13,
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 8, flexWrap: 'wrap' }}>
          <span style={{ color: '#666' }}>已选择：</span>
          <TypeTag label={selectedTargetType} />
          <span style={{ fontWeight: 600, wordBreak: 'break-all' }}>{selectedTargetLabel}</span>
        </div>
      </div>

      {/* Loading */}
      {loading && (
        <div
          style={{
            padding: 24,
            background: '#e3f2fd',
            borderRadius: 8,
            textAlign: 'center',
            border: '1px solid #90caf9',
          }}
        >
          <p style={{ margin: 0, color: '#1565c0', fontSize: 14, fontWeight: 600 }}>
            正在解析追溯...
          </p>
        </div>
      )}

      {/* Error */}
      {!loading && error && (
        <div
          style={{
            padding: 16,
            background: '#fce4ec',
            borderRadius: 8,
            border: '1px solid #ef9a9a',
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 14, color: '#c62828' }}>追溯解析失败</h4>
          <div style={{ fontSize: 13 }}>
            {error.error_code && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>错误码：</span>
                <code>{error.error_code}</code>
              </div>
            )}
            <div style={{ marginBottom: 4 }}>
              <span style={{ color: '#666' }}>信息：</span>
              {error.message}
            </div>
            {error.source_path && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>路径：</span>
                <code style={{ fontSize: 12 }}>{error.source_path}</code>
              </div>
            )}
            {error.details && (
              <div>
                <span style={{ color: '#666' }}>详情：</span>
                {error.details}
              </div>
            )}
          </div>
        </div>
      )}

      {/* Trace 列表 */}
      {!loading && !error && (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 10 }}>
          {resolvedTraces.length === 0 ? (
            <div
              style={{
                padding: 24,
                background: '#fafafa',
                borderRadius: 8,
                textAlign: 'center',
                color: '#999',
              }}
            >
              <p style={{ margin: '0 0 4px', fontSize: 14 }}>无证据追溯</p>
              <p style={{ margin: 0, fontSize: 12 }}>当前选择未关联任何 claim 或 evidence</p>
            </div>
          ) : (
            resolvedTraces.map((trace, index) => (
              <TraceCard
                key={`${trace.source_kind}-${index}`}
                trace={trace}
                onViewSource={onViewSource}
                onLocateEvidence={onLocateEvidence}
              />
            ))
          )}
        </div>
      )}
    </div>
  );
}

// ─── Trace 卡片 ─────────────────────────────────────────────────────────

function TraceCard({
  trace,
  onViewSource,
  onLocateEvidence,
}: {
  trace: TraceRefResolved;
  onViewSource: TracePanelProps['onViewSource'];
  onLocateEvidence: (evidenceId: string) => void;
}) {
  return (
    <div
      style={{
        padding: '12px 14px',
        background: '#fff',
        borderRadius: 8,
        border: '1px solid #e0e0e0',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, flexWrap: 'wrap' }}>
        <ResolutionTag resolution={trace.resolution} />
        <ConfidenceBadge confidence={trace.confidence} />
        {trace.relevance && (
          <span style={{ fontSize: 12, color: '#666' }}>{trace.relevance}</span>
        )}
      </div>

      {trace.claim && <ClaimBlock claim={trace.claim} />}
      {trace.evidence && (
        <EvidenceBlock
          evidence={trace.evidence}
          onViewSource={onViewSource}
          onLocateEvidence={onLocateEvidence}
        />
      )}

      {!trace.claim && !trace.evidence && (
        <div style={{ fontSize: 13, color: '#999' }}>
          该 trace 未解析到有效 claim 或 evidence。
        </div>
      )}
    </div>
  );
}

function ClaimBlock({ claim }: { claim: ClaimSnapshot }) {
  return (
    <div
      style={{
        padding: '8px 12px',
        background: '#f5f5f5',
        borderRadius: 6,
        marginBottom: 8,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4, flexWrap: 'wrap' }}>
        <code style={{ fontSize: 11, color: '#888' }}>{claim.claim_id}</code>
        <span
          style={{
            padding: '2px 6px',
            borderRadius: 3,
            fontSize: 11,
            background: '#e8eaf6',
            color: '#283593',
          }}
        >
          {CATEGORY_LABEL[claim.category] ?? claim.category}
        </span>
      </div>
      <p style={{ margin: '0 0 4px', fontSize: 13, lineHeight: 1.5, wordBreak: 'break-word' }}>
        {claim.description}
      </p>
      <div style={{ fontSize: 12, color: '#888' }}>
        证据引用数：{claim.evidence_ref_count}
        {claim.has_evidence_gap && (
          <span style={{ marginLeft: 8, color: '#e65100' }}>存在证据缺失</span>
        )}
      </div>
    </div>
  );
}

function EvidenceBlock({
  evidence,
  onViewSource,
  onLocateEvidence,
}: {
  evidence: EvidenceSnapshot;
  onViewSource: TracePanelProps['onViewSource'];
  onLocateEvidence: (evidenceId: string) => void;
}) {
  return (
    <div
      style={{
        padding: '8px 12px',
        background: '#f5f5f5',
        borderRadius: 6,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4, flexWrap: 'wrap' }}>
        <code style={{ fontSize: 11, color: '#888' }}>{evidence.evidence_id}</code>
        <StrengthBadge strength={evidence.strength} />
      </div>
      <p style={{ margin: '0 0 4px', fontSize: 13, lineHeight: 1.5, wordBreak: 'break-word' }}>
        {evidence.summary}
      </p>
      <div style={{ fontSize: 12, color: '#888', marginBottom: 8 }}>
        <span title={evidence.source_path} style={{ cursor: 'help' }}>
          {evidence.source_path.split('/').pop() || evidence.source_path}
        </span>
        {' · '}
        <span>
          行 {evidence.line_range.start}–{evidence.line_range.end}
        </span>
        {' · '}
        <span>{SOURCE_KIND_LABEL[evidence.source_kind] ?? evidence.source_kind}</span>
        {' · '}
        <span>{evidence.language}</span>
      </div>
      <div style={{ display: 'flex', gap: 8, flexWrap: 'wrap' }}>
        <button
          onClick={() =>
            onViewSource({
              source_path: evidence.source_path,
              line_range: evidence.line_range,
              evidence_id: evidence.evidence_id,
            })
          }
          style={{
            padding: '4px 10px',
            borderRadius: 4,
            border: '1px solid #1976d2',
            background: '#fff',
            color: '#1976d2',
            cursor: 'pointer',
            fontSize: 12,
          }}
        >
          查看源码片段
        </button>
        <button
          onClick={() => onLocateEvidence(evidence.evidence_id)}
          style={{
            padding: '4px 10px',
            borderRadius: 4,
            border: '1px solid #f57c00',
            background: '#fff',
            color: '#f57c00',
            cursor: 'pointer',
            fontSize: 12,
          }}
        >
          定位 evidence
        </button>
      </div>
    </div>
  );
}

// ─── 标签组件 ───────────────────────────────────────────────────────────

function TypeTag({ label }: { label: string }) {
  return (
    <span
      style={{
        padding: '2px 8px',
        borderRadius: 4,
        fontSize: 11,
        background: '#e8eaf6',
        color: '#283593',
        fontWeight: 600,
      }}
    >
      {label}
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

function ConfidenceBadge({ confidence }: { confidence: ClaimConfidence }) {
  return (
    <span
      style={{
        display: 'inline-block',
        padding: '2px 8px',
        borderRadius: 4,
        fontSize: 12,
        fontWeight: 600,
        background: CONFIDENCE_BG[confidence] ?? '#f5f5f5',
        color: CONFIDENCE_COLOR[confidence] ?? '#757575',
      }}
    >
      {CONFIDENCE_LABEL[confidence] ?? confidence}
    </span>
  );
}

function StrengthBadge({ strength }: { strength: EvidenceStrength }) {
  const color: Record<EvidenceStrength, string> = {
    direct: '#4caf50',
    indirect: '#2196f3',
    weak: '#ff9800',
    conflicting: '#f44336',
    missing: '#9e9e9e',
  };
  return (
    <span
      style={{
        padding: '2px 8px',
        borderRadius: 3,
        fontSize: 11,
        background: color[strength] ?? '#9e9e9e',
        color: '#fff',
      }}
    >
      {STRENGTH_LABEL[strength] ?? strength}
    </span>
  );
}
