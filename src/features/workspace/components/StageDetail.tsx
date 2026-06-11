import type { StageContext, StageFile, UpstreamRef } from '../../../types/workspace';
import { formatBytes } from '../workspaceUiUtils';

export default function StageDetail({ context }: { context: StageContext }) {
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
