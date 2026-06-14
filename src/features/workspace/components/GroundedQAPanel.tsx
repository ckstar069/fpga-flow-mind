import { useState } from 'react';
import type {
  GroundedAnswer,
  GroundedAnswerCitation,
  GroundedAnswerClaim,
} from '../../../types/workspace';
import type { UiError } from '../workspaceUiTypes';

interface GroundedQAPanelProps {
  canAsk: boolean;
  disabledReason?: string;
  answer?: GroundedAnswer | null;
  loading?: boolean;
  error?: UiError | null;
  onAsk: (question: string) => void;
  onCitationClick?: (citation: GroundedAnswerCitation) => void;
}

export default function GroundedQAPanel({
  canAsk,
  disabledReason,
  answer,
  loading,
  error,
  onAsk,
  onCitationClick,
}: GroundedQAPanelProps) {
  const [question, setQuestion] = useState('');

  const handleSubmit = () => {
    const q = question.trim();
    if (!q || !canAsk || loading) return;
    onAsk(q);
  };

  return (
    <div style={{ marginBottom: 24 }}>
      <h3 style={{ fontSize: 15, margin: '0 0 12px' }}> grounded 问答</h3>

      {!canAsk && (
        <div
          style={{
            padding: 14,
            background: '#fff3e0',
            borderRadius: 8,
            fontSize: 13,
            color: '#795548',
            marginBottom: 12,
          }}
        >
          {disabledReason ?? '请先收集证据并生成理解后再提问'}
        </div>
      )}

      <div style={{ display: 'flex', gap: 8, marginBottom: 12 }}>
        <input
          type="text"
          placeholder="输入关于当前阶段的问题..."
          value={question}
          onChange={(e) => setQuestion(e.target.value)}
          onKeyDown={(e) => e.key === 'Enter' && handleSubmit()}
          disabled={!canAsk || loading}
          style={{
            flex: 1,
            padding: '8px 12px',
            border: '1px solid #ccc',
            borderRadius: 6,
            fontSize: 14,
            background: !canAsk ? '#f5f5f5' : '#fff',
          }}
        />
        <button
          onClick={handleSubmit}
          disabled={!canAsk || loading || !question.trim()}
          style={{
            padding: '8px 18px',
            borderRadius: 6,
            border: '1px solid #1976d2',
            background: !canAsk || loading || !question.trim() ? '#e0e0e0' : '#1976d2',
            color: !canAsk || loading || !question.trim() ? '#999' : '#fff',
            cursor: !canAsk || loading || !question.trim() ? 'not-allowed' : 'pointer',
            fontSize: 14,
          }}
        >
          {loading ? '生成中...' : '提问'}
        </button>
      </div>

      {error && (
        <div
          style={{
            padding: 14,
            background: '#ffebee',
            borderRadius: 8,
            marginBottom: 12,
            fontSize: 13,
            color: '#c62828',
          }}
        >
          <div style={{ fontWeight: 600, marginBottom: 4 }}>问答失败</div>
          {'error_code' in error && error.error_code && (
            <div style={{ marginBottom: 4 }}>
              错误码：<code>{error.error_code}</code>
            </div>
          )}
          <div>{error.message}</div>
          {'details' in error && error.details && (
            <div style={{ marginTop: 4, color: '#999' }}>{error.details}</div>
          )}
        </div>
      )}

      {answer && !loading && !error && (
        <div
          style={{
            padding: 16,
            background: '#fff',
            border: '1px solid #e0e0e0',
            borderRadius: 8,
          }}
        >
          <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 12 }}>
            <span style={{ fontSize: 14, fontWeight: 600 }}>回答</span>
            <ConfidenceTag confidence={answer.confidence} />
            {answer.is_degraded && (
              <span
                style={{
                  padding: '2px 6px',
                  borderRadius: 3,
                  fontSize: 11,
                  background: '#fff3e0',
                  color: '#f57c00',
                }}
              >
                mock
              </span>
            )}
          </div>

          <div style={{ fontSize: 14, lineHeight: 1.6, marginBottom: 12, color: '#333' }}>
            {answer.text}
          </div>

          {answer.claims.length > 0 && (
            <div style={{ marginBottom: 12 }}>
              <h4 style={{ fontSize: 13, margin: '0 0 8px', color: '#666' }}>claims</h4>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 8 }}>
                {answer.claims.map((claim, i) => (
                  <ClaimItem key={i} claim={claim} />
                ))}
              </div>
            </div>
          )}

          {answer.citations.length > 0 && (
            <div style={{ marginBottom: 12 }}>
              <h4 style={{ fontSize: 13, margin: '0 0 8px', color: '#666' }}>引用</h4>
              <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
                {answer.citations.map((citation) => (
                  <CitationItem
                    key={citation.index}
                    citation={citation}
                    onClick={() => onCitationClick?.(citation)}
                  />
                ))}
              </div>
            </div>
          )}

          {answer.warnings.length > 0 && (
            <div
              style={{
                padding: 10,
                background: '#fff8e1',
                borderRadius: 6,
                marginTop: 10,
              }}
            >
              <h4 style={{ fontSize: 13, margin: '0 0 6px', color: '#795548' }}>提示</h4>
              <ul style={{ margin: 0, paddingLeft: 18, fontSize: 12, color: '#795548' }}>
                {answer.warnings.map((w, i) => (
                  <li key={i} style={{ marginBottom: 2 }}>
                    <code>{w.code}</code>: {w.message}
                  </li>
                ))}
              </ul>
            </div>
          )}
        </div>
      )}

      {!answer && !loading && !error && canAsk && (
        <div
          style={{
            padding: 20,
            background: '#fafafa',
            borderRadius: 8,
            textAlign: 'center',
            color: '#999',
            fontSize: 13,
          }}
        >
          输入问题后点击“提问”，回答将基于当前 evidence / understanding / views / trace 上下文生成
        </div>
      )}
    </div>
  );
}

function ClaimItem({ claim }: { claim: GroundedAnswerClaim }) {
  return (
    <div
      style={{
        padding: '8px 10px',
        background: claim.confidence === 'unknown' ? '#ffebee' : '#f5f5f5',
        borderRadius: 6,
        fontSize: 13,
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <ConfidenceTag confidence={claim.confidence} />
        {claim.citation_indices.length > 0 && (
          <span style={{ fontSize: 11, color: '#666' }}>
            citation: {claim.citation_indices.join(', ')}
          </span>
        )}
      </div>
      <div style={{ color: '#333' }}>{claim.text}</div>
      {claim.reason && (
        <div style={{ marginTop: 4, fontSize: 12, color: '#c62828' }}>{claim.reason}</div>
      )}
    </div>
  );
}

function CitationItem({
  citation,
  onClick,
}: {
  citation: GroundedAnswerCitation;
  onClick?: () => void;
}) {
  const clickable = !!onClick && (citation.evidence_id || citation.source_location);
  return (
    <div
      onClick={clickable ? onClick : undefined}
      style={{
        padding: '8px 10px',
        background: clickable ? '#e3f2fd' : '#f5f5f5',
        borderRadius: 6,
        fontSize: 12,
        cursor: clickable ? 'pointer' : 'default',
      }}
    >
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 2 }}>
        <span
          style={{
            minWidth: 18,
            height: 18,
            borderRadius: 9,
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
        {citation.evidence_id && (
          <code style={{ color: '#666' }}>{citation.evidence_id}</code>
        )}
        {citation.claim_id && !citation.evidence_id && (
          <code style={{ color: '#666' }}>{citation.claim_id}</code>
        )}
      </div>
      <div style={{ color: '#555', marginLeft: 26 }}>{citation.excerpt_summary}</div>
    </div>
  );
}

const CONFIDENCE_LABEL: Record<GroundedAnswer['confidence'], string> = {
  confirmed: '已确认',
  supported: '有支撑',
  inferred: '推断',
  unknown: '未知',
  conflicting: '矛盾',
};

const CONFIDENCE_COLOR: Record<GroundedAnswer['confidence'], string> = {
  confirmed: '#1565c0',
  supported: '#2e7d32',
  inferred: '#f57c00',
  unknown: '#757575',
  conflicting: '#c62828',
};

const CONFIDENCE_BG: Record<GroundedAnswer['confidence'], string> = {
  confirmed: '#e3f2fd',
  supported: '#e8f5e9',
  inferred: '#fff3e0',
  unknown: '#f5f5f5',
  conflicting: '#ffebee',
};

function ConfidenceTag({ confidence }: { confidence: GroundedAnswer['confidence'] }) {
  return (
    <span
      style={{
        padding: '2px 8px',
        borderRadius: 4,
        fontSize: 11,
        fontWeight: 600,
        background: CONFIDENCE_BG[confidence],
        color: CONFIDENCE_COLOR[confidence],
      }}
    >
      {CONFIDENCE_LABEL[confidence]}
    </span>
  );
}
