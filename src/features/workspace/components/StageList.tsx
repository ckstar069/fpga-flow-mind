import type { StageSummary } from '../../../types/workspace';
import { STATUS_LABEL, getStageDisabledReason } from '../workspaceUiUtils';

export default function StageList({
  stages,
  selectedStageId,
  isLoading,
  onSelect,
}: {
  stages: StageSummary[];
  selectedStageId: string | null;
  isLoading: boolean;
  onSelect: (id: string) => void;
}) {
  if (stages.length === 0) {
    return (
      <div
        style={{
          padding: 16,
          background: '#fafafa',
          borderRadius: 8,
          textAlign: 'center',
        }}
      >
        <p style={{ margin: 0, color: '#999', fontSize: 14 }}>未识别到阶段目录</p>
      </div>
    );
  }

  return (
    <div>
      <h3 style={{ fontSize: 14, margin: '0 0 12px' }}>阶段列表</h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
        {stages.map((stage) => {
          const clickable =
            !isLoading &&
            (stage.status === 'available' || stage.status === 'naming_anomaly');
          const isSelected = selectedStageId === stage.stage_id;
          const disabledReason = getStageDisabledReason(stage.status);

          return (
            <button
              key={stage.stage_id}
              onClick={() => clickable && onSelect(stage.stage_id)}
              disabled={!clickable}
              title={disabledReason ?? undefined}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '8px 12px',
                borderRadius: 6,
                border: isSelected ? '2px solid #1976d2' : '1px solid #e0e0e0',
                background: clickable ? '#fff' : '#f5f5f5',
                cursor: clickable ? 'pointer' : 'not-allowed',
                textAlign: 'left',
                width: '100%',
                opacity: clickable ? 1 : 0.7,
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8 }}>
                <span style={{ fontWeight: 600, fontSize: 14 }}>{stage.stage_id}</span>
                {stage.status !== 'available' && (
                  <span
                    style={{
                      fontSize: 11,
                      padding: '2px 6px',
                      borderRadius: 4,
                      background:
                        stage.status === 'naming_anomaly'
                          ? '#fff3e0'
                          : stage.status === 'empty'
                            ? '#f5f5f5'
                            : '#ffebee',
                      color:
                        stage.status === 'naming_anomaly'
                          ? '#f57c00'
                          : stage.status === 'empty'
                            ? '#999'
                            : '#c62828',
                    }}
                  >
                    {STATUS_LABEL[stage.status] ?? stage.status}
                  </span>
                )}
              </div>
              <span style={{ fontSize: 12, color: '#999' }}>
                {stage.file_count} 文件
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
