import type { WorkspaceProfile } from '../../../types/workspace';
import { VALIDITY_LABEL, VALIDITY_COLOR } from '../workspaceUiUtils';

export default function WorkspaceSummary({ profile }: { profile: WorkspaceProfile }) {
  const stats = Object.entries(profile.file_type_stats);
  return (
    <div style={{ marginBottom: 24 }}>
      <h2 style={{ fontSize: 16, margin: '0 0 8px' }}>{profile.workspace_name}</h2>
      <p
        style={{
          fontSize: 12,
          color: '#666',
          margin: '0 0 8px',
          wordBreak: 'break-all',
        }}
      >
        {profile.root_path}
      </p>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <span
          style={{
            fontSize: 13,
            fontWeight: 600,
            color: VALIDITY_COLOR[profile.validity] ?? '#333',
          }}
        >
          {VALIDITY_LABEL[profile.validity] ?? profile.validity}
        </span>
      </div>
      {profile.external_refs.length > 0 && (
        <p style={{ fontSize: 12, color: '#666', margin: '0 0 8px' }}>
          外部引用: {profile.external_refs.join(', ')}
        </p>
      )}
      {stats.length > 0 && (
        <div style={{ fontSize: 12, color: '#666' }}>
          文件统计: {stats.map(([ext, count]) => `${ext}: ${count}`).join(', ')}
        </div>
      )}
      {profile.error_codes.length > 0 && (
        <div style={{ marginTop: 8, fontSize: 12 }}>
          {profile.error_codes.map((code, i) => (
            <span
              key={i}
              style={{
                display: 'inline-block',
                padding: '2px 6px',
                background: '#ffebee',
                color: '#c62828',
                borderRadius: 4,
                marginRight: 4,
              }}
            >
              {code}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}
