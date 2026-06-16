import type { ReactNode } from 'react';
import { NAV } from './workbenchTheme';

interface LeftNavProps {
  projectInfo?: ReactNode;
  stageList?: ReactNode;
  recentProjects?: ReactNode;
  loadError?: ReactNode;
}

/**
 * LeftNav: 深色固定左侧导航。
 * 承载项目信息、阶段列表、最近项目、加载错误。
 * Batch D：使用统一设计 token，各 section 之间加细分隔线，提升层级清晰度。
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
        background: NAV.bg,
        color: NAV.text,
        borderRight: `1px solid ${NAV.border}`,
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
          gap: 16,
        }}
      >
        {projectInfo}
        {projectInfo && (
          <div style={{ height: 1, background: NAV.divider, margin: '0 -16px' }} />
        )}
        {stageList}
        {stageList && (
          <div style={{ height: 1, background: NAV.divider, margin: '0 -16px' }} />
        )}
        {recentProjects}
        {loadError}
      </div>
    </aside>
  );
}
