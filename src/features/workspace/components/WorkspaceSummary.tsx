import type { WorkspaceProfile } from '../../../types/workspace';
import { VALIDITY_LABEL } from '../workspaceUiUtils';
import { NAV, ACCENT, FONT } from './workbenchTheme';

// validity → 适配深色侧栏的可读色（不使用裁决红绿，仅表达结构匹配程度提示）
function validityColor(validity: string): string {
  switch (validity) {
    case 'likely_valid':
      return '#66bb6a';
    case 'uncertain':
      return ACCENT.amber;
    default:
      return '#fca5a5';
  }
}

export default function WorkspaceSummary({ profile }: { profile: WorkspaceProfile }) {
  const stats = Object.entries(profile.file_type_stats);
  return (
    <div>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 4 }}>
        <span
          style={{
            width: 8,
            height: 8,
            borderRadius: 2,
            background: ACCENT.blue,
            flexShrink: 0,
          }}
        />
        <h2
          style={{
            fontSize: FONT.title,
            fontWeight: 600,
            margin: 0,
            color: NAV.text,
            overflow: 'hidden',
            textOverflow: 'ellipsis',
            whiteSpace: 'nowrap',
          }}
          title={profile.workspace_name}
        >
          {profile.workspace_name}
        </h2>
      </div>
      <p
        style={{
          fontSize: FONT.micro,
          color: NAV.textDim,
          margin: '0 0 8px 16px',
          wordBreak: 'break-all',
          lineHeight: 1.4,
        }}
      >
        {profile.root_path}
      </p>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8, paddingLeft: 16 }}>
        <span
          style={{
            fontSize: FONT.caption,
            fontWeight: 600,
            color: validityColor(profile.validity),
          }}
        >
          {VALIDITY_LABEL[profile.validity] ?? profile.validity}
        </span>
      </div>
      {profile.external_refs.length > 0 && (
        <p style={{ fontSize: FONT.micro, color: NAV.textDim, margin: '0 0 6px 16px', lineHeight: 1.5 }}>
          外部引用：{profile.external_refs.join(', ')}
        </p>
      )}
      {stats.length > 0 && (
        <div style={{ fontSize: FONT.micro, color: NAV.textDim, paddingLeft: 16, lineHeight: 1.6 }}>
          {stats.map(([ext, count]) => `${ext}: ${count}`).join(' · ')}
        </div>
      )}
      {profile.error_codes.length > 0 && (
        <div style={{ marginTop: 8, paddingLeft: 16, display: 'flex', flexWrap: 'wrap', gap: 4 }}>
          {profile.error_codes.map((code, i) => (
            <span
              key={i}
              style={{
                display: 'inline-block',
                padding: '1px 6px',
                background: 'rgba(198, 40, 40, 0.2)',
                color: '#fca5a5',
                borderRadius: 4,
                fontSize: FONT.micro,
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
