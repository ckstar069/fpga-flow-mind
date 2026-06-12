import type { StageContext, StageFile, UpstreamRef, EvidenceCollection } from '../../../types/workspace';
import type { UiError } from '../workspaceUiTypes';
import { formatBytes } from '../workspaceUiUtils';
import EvidencePanel from './EvidencePanel';

interface StageDetailProps {
  context: StageContext;
  evidence?: EvidenceCollection;
  evidenceError?: UiError;
  isCollecting?: boolean;
  onCollectEvidence?: () => void;
}

export default function StageDetail({
  context,
  evidence,
  evidenceError,
  isCollecting,
  onCollectEvidence,
}: StageDetailProps) {
  const canCollect =
    !context.error_code && context.files.length > 0 && !!onCollectEvidence;

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
            disabled={isCollecting}
            style={{
              padding: '8px 20px',
              borderRadius: 6,
              border: '1px solid #1976d2',
              background: isCollecting ? '#e0e0e0' : '#1976d2',
              color: isCollecting ? '#999' : '#fff',
              cursor: isCollecting ? 'not-allowed' : 'pointer',
              fontSize: 14,
            }}
          >
            {isCollecting ? '收集中...' : '收集证据'}
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
          <p style={{ margin: 0, fontSize: 13 }}>{evidenceError.message}</p>
        </div>
      )}

      {evidence && <EvidencePanel evidence={evidence} />}

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
