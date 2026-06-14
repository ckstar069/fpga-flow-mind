export interface ConfirmDialogProps {
  title: string;
  children: React.ReactNode;
  confirmLabel: string;
  cancelLabel?: string;
  danger?: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}

export default function ConfirmDialog({
  title,
  children,
  confirmLabel,
  cancelLabel = '取消',
  danger = false,
  onConfirm,
  onCancel,
}: ConfirmDialogProps) {
  return (
    <div
      onClick={onCancel}
      style={{
        position: 'fixed',
        inset: 0,
        background: 'rgba(0, 0, 0, 0.4)',
        display: 'flex',
        alignItems: 'center',
        justifyContent: 'center',
        zIndex: 1000,
      }}
    >
      <div
        onClick={(e) => e.stopPropagation()}
        style={{
          background: '#fff',
          borderRadius: 8,
          padding: 24,
          minWidth: 360,
          maxWidth: 480,
          boxShadow: '0 4px 20px rgba(0,0,0,0.2)',
        }}
      >
        <h3 style={{ margin: '0 0 16px', fontSize: 16, color: '#333' }}>{title}</h3>
        <div style={{ marginBottom: 24, fontSize: 14, color: '#555', lineHeight: 1.5 }}>{children}</div>
        <div style={{ display: 'flex', justifyContent: 'flex-end', gap: 12 }}>
          <button
            onClick={onCancel}
            style={{
              padding: '6px 16px',
              borderRadius: 4,
              border: '1px solid #ccc',
              background: '#fff',
              cursor: 'pointer',
              fontSize: 13,
              color: '#333',
            }}
          >
            {cancelLabel}
          </button>
          <button
            onClick={onConfirm}
            style={{
              padding: '6px 16px',
              borderRadius: 4,
              border: '1px solid transparent',
              background: danger ? '#c62828' : '#1976d2',
              cursor: 'pointer',
              fontSize: 13,
              color: '#fff',
            }}
          >
            {confirmLabel}
          </button>
        </div>
      </div>
    </div>
  );
}
