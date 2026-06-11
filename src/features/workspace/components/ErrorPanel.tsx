import type { UiError } from '../workspaceUiTypes';

export default function ErrorPanel({ error }: { error: UiError }) {
  return (
    <div
      style={{
        padding: 16,
        background: '#ffebee',
        borderRadius: 8,
        marginBottom: 16,
      }}
    >
      <h3 style={{ margin: '0 0 8px', color: '#c62828' }}>错误</h3>
      <p style={{ margin: '0 0 4px' }}>{error.message}</p>
      <code style={{ fontSize: 12, color: '#666' }}>{error.error_code}</code>
    </div>
  );
}
