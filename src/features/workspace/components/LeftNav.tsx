import type { ReactNode } from 'react';

interface LeftNavProps {
  projectInfo?: ReactNode;
  stageList?: ReactNode;
  recentProjects?: ReactNode;
  loadError?: ReactNode;
}

/**
 * LeftNav: 深色固定左侧导航
 * 承载项目信息、阶段列表、最近项目、加载错误。
 * Batch A 仅做视觉容器，不改变子组件行为。
 */
export default function LeftNav({
  projectInfo,
  stageList,
  recentProjects,
  loadError,
}: LeftNavProps) {
  return (
    <aside
      className="left-nav"
      style={{
        width: 280,
        minWidth: 240,
        maxWidth: 360,
        background: '#1e293b',
        color: '#e2e8f0',
        borderRight: '1px solid #334155',
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        flexShrink: 0,
      }}
    >
      <div
        className="left-nav-scroll"
        style={{
          flex: 1,
          overflowY: 'auto',
          padding: 16,
          display: 'flex',
          flexDirection: 'column',
          gap: 20,
        }}
      >
        {projectInfo}
        {stageList}
        {recentProjects}
        {loadError}
      </div>
    </aside>
  );
}
