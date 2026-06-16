import type { ReactNode } from 'react';
import type { WorkspaceProfile, StageContext } from '../../../types/workspace';

interface StageWorkspaceProps {
  profile: WorkspaceProfile;
  stageId: string;
  context: StageContext;
  children: ReactNode;
}

const ARTIFACT_TAB_LABELS = [
  '概览',
  '证据',
  '理解',
  '视图',
  '追溯',
  'Q&A',
  '质量',
];

/**
 * StageWorkspace: 阶段工作区骨架
 * Batch A 仅搭建占位容器：顶部标题/概览条、筛选条、Artifact tabs、
 * 主内容区（继续渲染 StageDetail 作为 legacy content）、右侧 ContextPanel 占位。
 *
 * 当前 Artifact tabs 仅为视觉占位，点击不切换内容；真实内容迁移将在 Batch B 实现。
 */
export default function StageWorkspace({
  profile,
  stageId,
  context,
  children,
}: StageWorkspaceProps) {
  return (
    <div
      className="stage-workspace"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100%',
        background: '#f8fafc',
        overflow: 'hidden',
      }}
    >
      {/* WorkspaceTopBar */}
      <div
        className="stage-workspace-topbar"
        style={{
          padding: '12px 20px',
          background: '#fff',
          borderBottom: '1px solid #e2e8f0',
          flexShrink: 0,
        }}
      >
        <div
          style={{
            fontSize: 15,
            fontWeight: 600,
            color: '#1e293b',
          }}
        >
          {profile.workspace_name} / {stageId}
        </div>
        <div
          style={{
            fontSize: 12,
            color: '#64748b',
            marginTop: 4,
            wordBreak: 'break-all',
          }}
        >
          {context.source_path}
        </div>
      </div>

      {/* StageOverviewBar 占位 */}
      <div
        className="stage-overview-bar"
        style={{
          padding: '10px 20px',
          background: '#fff',
          borderBottom: '1px solid #e2e8f0',
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 12, color: '#94a3b8' }}>概览指标占位（Batch B 实现）</span>
      </div>

      {/* StageFilterBar 占位 */}
      <div
        className="stage-filter-bar"
        style={{
          padding: '8px 20px',
          background: '#f8fafc',
          borderBottom: '1px solid #e2e8f0',
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 12, color: '#94a3b8' }}>筛选 / 分组 / 视图切换占位（Batch B 实现）</span>
      </div>

      {/* Artifact tabs 占位：仅视觉展示，不响应点击 */}
      <div
        className="artifact-tabs"
        style={{
          display: 'flex',
          gap: 4,
          padding: '8px 20px 0',
          background: '#f8fafc',
          borderBottom: '1px solid #e2e8f0',
          flexShrink: 0,
          overflowX: 'auto',
        }}
      >
        {ARTIFACT_TAB_LABELS.map((label) => (
          <span
            key={label}
            style={{
              padding: '8px 14px',
              borderRadius: '6px 6px 0 0',
              border: '1px solid transparent',
              borderBottom: '2px solid transparent',
              background: 'transparent',
              color: '#94a3b8',
              fontSize: 13,
              fontWeight: 400,
              whiteSpace: 'nowrap',
              userSelect: 'none',
            }}
          >
            {label}
          </span>
        ))}
      </div>

      {/* Artifact tabs 占位说明 */}
      <div
        className="artifact-tabs-placeholder-notice"
        style={{
          padding: '6px 20px',
          background: '#f1f5f9',
          borderBottom: '1px solid #e2e8f0',
          flexShrink: 0,
        }}
      >
        <span style={{ fontSize: 11, color: '#64748b' }}>
          Artifact tabs 内容迁移将在 Batch B 实现；当前主内容区保留原 StageDetail 兼容视图。
        </span>
      </div>

      {/* 主内容区 + ContextPanel */}
      <div
        className="stage-workspace-body"
        style={{
          display: 'flex',
          flex: 1,
          overflow: 'hidden',
        }}
      >
        <div
          className="stage-content-area"
          style={{
            flex: 1,
            padding: 20,
            overflowY: 'auto',
            minWidth: 0,
          }}
        >
          {children}
        </div>

        {/* ContextPanel 占位 */}
        <div
          className="context-panel"
          style={{
            width: 280,
            minWidth: 240,
            maxWidth: 320,
            background: '#fff',
            borderLeft: '1px solid #e2e8f0',
            padding: 16,
            flexShrink: 0,
            overflowY: 'auto',
          }}
        >
          <div style={{ fontSize: 12, color: '#94a3b8' }}>
            上下文面板占位（Batch C 实现）
          </div>
        </div>
      </div>
    </div>
  );
}
