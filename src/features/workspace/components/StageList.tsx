import type { StageSummary } from '../../../types/workspace';
import { STATUS_LABEL, getStageDisabledReason } from '../workspaceUiUtils';
import { NAV, ACCENT, FONT } from './workbenchTheme';

function badgePalette(status: string): { bg: string; color: string } {
  switch (status) {
    case 'naming_anomaly':
      return { bg: 'rgba(245, 124, 0, 0.18)', color: ACCENT.amber };
    case 'empty':
      return { bg: 'rgba(148, 163, 184, 0.18)', color: NAV.textMuted };
    default:
      // missing / unreadable：中性警示色（非裁决红绿）
      return { bg: 'rgba(198, 40, 40, 0.2)', color: '#fca5a5' };
  }
}

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
          background: NAV.surface,
          borderRadius: 8,
          textAlign: 'center',
          border: `1px solid ${NAV.border}`,
        }}
      >
        <p style={{ margin: 0, color: NAV.textMuted, fontSize: FONT.caption }}>
          未识别到阶段目录
        </p>
      </div>
    );
  }

  return (
    <div>
      <h3
        style={{
          fontSize: FONT.micro,
          margin: '0 0 8px',
          color: NAV.textDim,
          fontWeight: 600,
          letterSpacing: 0.5,
          textTransform: 'uppercase',
        }}
      >
        阶段列表
      </h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
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
                gap: 8,
                padding: '8px 12px',
                borderRadius: 6,
                border: '1px solid transparent',
                borderLeft: isSelected ? `3px solid ${ACCENT.blue}` : '3px solid transparent',
                background: isSelected
                  ? NAV.bgActive
                  : clickable
                    ? 'transparent'
                    : NAV.bgSubtle,
                cursor: clickable ? 'pointer' : 'not-allowed',
                textAlign: 'left',
                width: '100%',
                color: clickable ? NAV.text : NAV.textDim,
                opacity: clickable ? 1 : 0.7,
                transition: 'background 0.12s',
              }}
              onMouseEnter={(e) => {
                if (clickable && !isSelected) e.currentTarget.style.background = NAV.bgHover;
              }}
              onMouseLeave={(e) => {
                if (clickable && !isSelected) e.currentTarget.style.background = 'transparent';
              }}
            >
              <div style={{ display: 'flex', alignItems: 'center', gap: 8, minWidth: 0 }}>
                <span style={{ fontWeight: 600, fontSize: FONT.body }}>{stage.stage_id}</span>
                {stage.status !== 'available' && (() => {
                  const p = badgePalette(stage.status);
                  return (
                    <span
                      style={{
                        fontSize: FONT.micro,
                        padding: '1px 6px',
                        borderRadius: 4,
                        background: p.bg,
                        color: p.color,
                        whiteSpace: 'nowrap',
                      }}
                    >
                      {STATUS_LABEL[stage.status] ?? stage.status}
                    </span>
                  );
                })()}
              </div>
              <span style={{ fontSize: FONT.micro, color: NAV.textDim, whiteSpace: 'nowrap' }}>
                {stage.file_count} 文件
              </span>
            </button>
          );
        })}
      </div>
    </div>
  );
}
