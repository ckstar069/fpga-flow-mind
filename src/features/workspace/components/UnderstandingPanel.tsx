import type {
  ImplementationUnderstanding,
  ImplementationClaim,
  ModuleSummary,
  SignalSummary,
  InterfaceSummary,
  ProcessingStepSummary,
  EvidenceRef,
  ProviderStatusResponse,
} from '../../../types/workspace';

import { ACCENT, FONT } from './workbenchTheme';

const CONFIDENCE_COLOR: Record<string, string> = {
  confirmed: '#2e7d32',
  supported: '#1565c0',
  inferred: '#e65100',
  unknown: '#757575',
  conflicting: '#c62828',
};

const CONFIDENCE_BG: Record<string, string> = {
  confirmed: '#e8f5e9',
  supported: '#e3f2fd',
  inferred: '#fff3e0',
  unknown: '#f5f5f5',
  conflicting: '#ffebee',
};

const CONFIDENCE_LABEL: Record<string, string> = {
  confirmed: '已确认',
  supported: '有支撑',
  inferred: '推断',
  unknown: '未知',
  conflicting: '矛盾',
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

// ─── 子组件 ───────────────────────────────────────────────────────────

function EvidenceRefChips({ refs }: { refs: EvidenceRef[] }) {
  if (refs.length === 0) return <span style={{ color: '#999', fontSize: 12 }}>无引用</span>;
  return (
    <span style={{ display: 'inline-flex', flexWrap: 'wrap', gap: 4 }}>
      {refs.map((r, i) => (
        <code
          key={i}
          style={{
            padding: '1px 6px',
            background: '#e3f2fd',
            borderRadius: 3,
            fontSize: 11,
            fontFamily: 'monospace',
          }}
          title={r.relevance}
        >
          {r.evidence_id}
        </code>
      ))}
    </span>
  );
}

function ConfidenceBadge({ confidence }: { confidence: string }) {
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

function EmptyState({ label }: { label: string }) {
  return (
    <div
      style={{
        padding: 12,
        background: '#fafafa',
        borderRadius: 6,
        textAlign: 'center',
        color: '#999',
        fontSize: 13,
      }}
    >
      暂无{label}
    </div>
  );
}

// ─── 主组件 ───────────────────────────────────────────────────────────

interface UnderstandingPanelProps {
  understanding: ImplementationUnderstanding;
  providerStatus?: ProviderStatusResponse | null;
}

function ProviderBadge({ status }: { status?: ProviderStatusResponse | null }) {
  if (!status) return null;
  const color =
    status.status === 'real'
      ? ACCENT.blue
      : status.status === 'degraded'
        ? ACCENT.amber
        : ACCENT.slate;
  const bg =
    status.status === 'real'
      ? ACCENT.blueSoft
      : status.status === 'degraded'
        ? ACCENT.amberSoft
        : ACCENT.slateSoft;
  const label =
    status.status === 'mock'
      ? 'Mock'
      : status.status === 'real'
        ? '真实 LLM'
        : status.status === 'degraded'
          ? '降级'
          : '未知';
  return (
    <span
      style={{
        padding: '4px 10px',
        background: bg,
        color,
        borderRadius: 4,
        fontSize: FONT.caption,
        fontWeight: 600,
        border: `1px solid ${color}`,
      }}
      title={`${status.kind} / ${status.model}`}
    >
      {label}
    </span>
  );
}

export default function UnderstandingPanel({ understanding, providerStatus }: UnderstandingPanelProps) {
  const { summary, claims, module_summaries, signal_summaries, interface_summaries, processing_steps, unknowns, evidence_gaps, generation_meta, stats } = understanding;

  return (
    <div style={{ display: 'flex', flexDirection: 'column', gap: 20 }}>
      {/* ─── 顶部摘要 ─── */}
      <Section title="阶段理解摘要">
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 12, marginBottom: 12 }}>
          <MetaChip label="阶段" value={understanding.stage_id} />
          <MetaChip label="版本" value={understanding.version} />
          <MetaChip label="Provider" value={generation_meta.provider} />
          <ProviderBadge status={providerStatus} />
          {generation_meta.is_degraded && (
            <span
              style={{
                padding: '4px 10px',
                background: '#fff3e0',
                color: '#e65100',
                borderRadius: 4,
                fontSize: 12,
                fontWeight: 600,
                border: '1px solid #ffcc80',
              }}
            >
              降级生成 · Provider 未配置
            </span>
          )}
        </div>
        <p style={{ margin: '0 0 8px', fontSize: 14, fontWeight: 500 }}>
          {summary.short}
        </p>
        <p style={{ margin: 0, fontSize: 13, color: '#555', lineHeight: 1.6 }}>
          {summary.detailed}
        </p>
      </Section>

      {/* ─── 降级模式提示 ─── */}
      {generation_meta.is_degraded && (
        <div
          style={{
            padding: '12px 16px',
            background: '#fff8e1',
            borderRadius: 8,
            border: '1px solid #ffe082',
            fontSize: 13,
            color: '#795548',
            lineHeight: 1.6,
          }}
        >
          <strong>当前为降级生成模式：</strong>
          语义分析 Provider 未配置，生成结果仅包含证据的结构化汇总，未进行语义推断。
          如需完整理解，请确认 Provider 配置后重新生成。
        </div>
      )}

      {/* ─── Claims 列表 ─── */}
      <Section title={`声明 (${claims.length})`}>
        {claims.length === 0 ? (
          <EmptyState label="声明" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {claims.map((c) => (
              <ClaimCard key={c.claim_id} claim={c} />
            ))}
          </div>
        )}
      </Section>

      {/* ─── 模块/信号/接口/处理步骤摘要 ─── */}
      <Section title="模块与信号">
        <SummaryGroup
          label="模块"
          items={module_summaries}
          render={(m: ModuleSummary) => (
            <SummaryRow
              key={m.name}
              name={m.name}
              description={m.description}
              confidence={m.confidence}
              evidenceRefs={m.evidence_refs}
            />
          )}
        />
        <SummaryGroup
          label="信号"
          items={signal_summaries}
          render={(s: SignalSummary) => (
            <SummaryRow
              key={s.name}
              name={s.name + (s.direction ? ` (${s.direction})` : '')}
              description={s.description}
              confidence={s.confidence}
              evidenceRefs={s.evidence_refs}
            />
          )}
        />
        <SummaryGroup
          label="接口"
          items={interface_summaries}
          render={(i: InterfaceSummary) => (
            <SummaryRow
              key={i.name}
              name={i.name + (i.interface_type ? ` [${i.interface_type}]` : '')}
              description={i.description}
              confidence={i.confidence}
              evidenceRefs={i.evidence_refs}
            />
          )}
        />
        <SummaryGroup
          label="处理步骤"
          items={processing_steps}
          render={(p: ProcessingStepSummary) => (
            <SummaryRow
              key={`${p.name}-${p.order}`}
              name={`${p.order}. ${p.name}`}
              description={p.description}
              confidence={p.confidence}
              evidenceRefs={p.evidence_refs}
            />
          )}
        />
      </Section>

      {/* ─── Unknowns ─── */}
      <Section title={`未推断项 (${unknowns.length})`}>
        {unknowns.length === 0 ? (
          <EmptyState label="未推断项" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {unknowns.map((u) => (
              <UnknownGapCard
                key={u.unknown_id}
                id={u.unknown_id}
                description={u.description}
                reason={u.reason}
                refs={u.related_evidence_refs}
                type="unknown"
              />
            ))}
          </div>
        )}
      </Section>

      {/* ─── Evidence Gaps ─── */}
      <Section title={`证据缺失 (${evidence_gaps.length})`}>
        {evidence_gaps.length === 0 ? (
          <EmptyState label="证据缺失" />
        ) : (
          <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
            {evidence_gaps.map((g) => (
              <UnknownGapCard
                key={g.gap_id}
                id={g.gap_id}
                description={g.expected_evidence}
                reason={g.reason}
                refs={g.related_evidence_refs}
                type="gap"
              />
            ))}
          </div>
        )}
      </Section>

      {/* ─── Stats ─── */}
      <Section title="统计">
        <div style={{ display: 'flex', flexWrap: 'wrap', gap: 16, fontSize: 13 }}>
          <StatKV label="总声明数" value={stats.total_claims} />
          <StatKV label="模块数" value={stats.module_count} />
          <StatKV label="信号数" value={stats.signal_count} />
          <StatKV label="接口数" value={stats.interface_count} />
          <StatKV label="处理步骤数" value={stats.processing_step_count} />
          <StatKV label="未知项" value={stats.unknown_count} />
          <StatKV label="证据缺失" value={stats.evidence_gap_count} />
        </div>
        {(Object.keys(stats.claims_by_confidence).length > 0) && (
          <div style={{ marginTop: 12 }}>
            <span style={{ fontSize: 12, color: '#666' }}>置信度分布：</span>
            {Object.entries(stats.claims_by_confidence).map(([conf, count]) => (
              <span
                key={conf}
                style={{
                  marginLeft: 8,
                  padding: '2px 8px',
                  borderRadius: 4,
                  fontSize: 12,
                  background: CONFIDENCE_BG[conf] ?? '#f5f5f5',
                  color: CONFIDENCE_COLOR[conf] ?? '#757575',
                }}
              >
                {CONFIDENCE_LABEL[conf] ?? conf}: {count}
              </span>
            ))}
          </div>
        )}
        {(Object.keys(stats.claims_by_category).length > 0) && (
          <div style={{ marginTop: 8 }}>
            <span style={{ fontSize: 12, color: '#666' }}>分类分布：</span>
            {Object.entries(stats.claims_by_category).map(([cat, count]) => (
              <span
                key={cat}
                style={{
                  marginLeft: 8,
                  padding: '2px 8px',
                  borderRadius: 4,
                  fontSize: 12,
                  background: '#e8eaf6',
                  color: '#283593',
                }}
              >
                {CATEGORY_LABEL[cat] ?? cat}: {count}
              </span>
            ))}
          </div>
        )}
      </Section>
    </div>
  );
}

// ─── 布局辅助组件 ─────────────────────────────────────────────────────

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <div>
      <h3
        style={{
          fontSize: 15,
          fontWeight: 600,
          margin: '0 0 12px',
          paddingBottom: 6,
          borderBottom: '1px solid #e0e0e0',
          color: '#333',
        }}
      >
        {title}
      </h3>
      {children}
    </div>
  );
}

function MetaChip({ label, value }: { label: string; value: string }) {
  return (
    <span style={{ fontSize: 12, color: '#555' }}>
      <span style={{ color: '#999' }}>{label}：</span>
      <code
        style={{
          padding: '1px 6px',
          background: '#f5f5f5',
          borderRadius: 3,
          fontFamily: 'monospace',
          fontSize: 12,
        }}
      >
        {value}
      </code>
    </span>
  );
}

function StatKV({ label, value }: { label: string; value: number }) {
  return (
    <div
      style={{
        padding: '8px 14px',
        background: '#f5f5f5',
        borderRadius: 6,
        textAlign: 'center',
        minWidth: 80,
      }}
    >
      <div style={{ fontSize: 18, fontWeight: 700, color: '#333' }}>{value}</div>
      <div style={{ fontSize: 11, color: '#888', marginTop: 2 }}>{label}</div>
    </div>
  );
}

function ClaimCard({ claim }: { claim: ImplementationClaim }) {
  return (
    <div
      style={{
        padding: '12px 14px',
        background: '#fff',
        borderRadius: 6,
        border: '1px solid #e0e0e0',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 6, flexWrap: 'wrap' }}>
        <code style={{ fontSize: 11, color: '#888', fontFamily: 'monospace' }}>
          {claim.claim_id}
        </code>
        <span
          style={{
            padding: '2px 8px',
            borderRadius: 4,
            fontSize: 11,
            background: '#e8eaf6',
            color: '#283593',
          }}
        >
          {CATEGORY_LABEL[claim.category] ?? claim.category}
        </span>
        <ConfidenceBadge confidence={claim.confidence} />
        {claim.has_evidence_gap && (
          <span
            style={{
              padding: '2px 8px',
              borderRadius: 4,
              fontSize: 11,
              background: '#fff3e0',
              color: '#e65100',
            }}
          >
            证据不足
          </span>
        )}
      </div>
      <p style={{ margin: '0 0 6px', fontSize: 13, lineHeight: 1.5, wordBreak: 'break-word' }}>
        {claim.description}
      </p>
      <div style={{ fontSize: 12, color: '#888' }}>
        证据引用：<EvidenceRefChips refs={claim.evidence_refs} />
      </div>
    </div>
  );
}

function SummaryRow({
  name,
  description,
  confidence,
  evidenceRefs,
}: {
  name: string;
  description: string;
  confidence: string;
  evidenceRefs: EvidenceRef[];
}) {
  return (
    <div
      style={{
        padding: '8px 12px',
        background: '#fff',
        borderRadius: 6,
        border: '1px solid #e0e0e0',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4, flexWrap: 'wrap' }}>
        <span style={{ fontSize: 13, fontWeight: 600 }}>{name}</span>
        <ConfidenceBadge confidence={confidence} />
      </div>
      <p style={{ margin: '0 0 4px', fontSize: 12, color: '#555', lineHeight: 1.4, wordBreak: 'break-word' }}>
        {description}
      </p>
      <EvidenceRefChips refs={evidenceRefs} />
    </div>
  );
}

function SummaryGroup<T>({
  label,
  items,
  render,
}: {
  label: string;
  items: T[];
  render: (item: T) => React.ReactNode;
}) {
  return (
    <div style={{ marginTop: 12 }}>
      <h4 style={{ fontSize: 13, fontWeight: 600, margin: '0 0 8px', color: '#555' }}>
        {label} ({items.length})
      </h4>
      {items.length === 0 ? (
        <EmptyState label={label} />
      ) : (
        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>{items.map(render)}</div>
      )}
    </div>
  );
}

function UnknownGapCard({
  id,
  description,
  reason,
  refs,
  type,
}: {
  id: string;
  description: string;
  reason: string;
  refs: EvidenceRef[];
  type: 'unknown' | 'gap';
}) {
  const bg = type === 'unknown' ? '#fafafa' : '#fff8e1';
  const borderColor = type === 'unknown' ? '#e0e0e0' : '#ffe082';
  return (
    <div
      style={{
        padding: '10px 14px',
        background: bg,
        borderRadius: 6,
        border: `1px solid ${borderColor}`,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4, flexWrap: 'wrap' }}>
        <code style={{ fontSize: 11, color: '#888', fontFamily: 'monospace' }}>{id}</code>
        <span
          style={{
            padding: '2px 6px',
            borderRadius: 3,
            fontSize: 11,
            background: type === 'unknown' ? '#eee' : '#fff3e0',
            color: type === 'unknown' ? '#666' : '#e65100',
          }}
        >
          {type === 'unknown' ? '未推断' : '证据缺失'}
        </span>
      </div>
      <p style={{ margin: '0 0 4px', fontSize: 12, lineHeight: 1.4, wordBreak: 'break-word' }}>
        {description}
      </p>
      <p style={{ margin: '0 0 4px', fontSize: 12, color: '#888', fontStyle: 'italic' }}>
        原因：{reason}
      </p>
      <EvidenceRefChips refs={refs} />
    </div>
  );
}
