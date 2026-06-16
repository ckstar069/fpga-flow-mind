import type { ReactNode } from 'react';

interface AppShellProps {
  header: ReactNode;
  leftNav: ReactNode;
  main: ReactNode;
  footer?: ReactNode;
}

/**
 * AppShell: 工作台外层骨架
 * 纵向：Header + 横向（LeftNav + MainWorkspace）+ Footer
 * 不承载业务逻辑，只做布局容器。
 */
export default function AppShell({ header, leftNav, main, footer }: AppShellProps) {
  return (
    <div
      className="app-shell"
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        overflow: 'hidden',
      }}
    >
      {header}
      <div
        className="app-shell-body"
        style={{
          display: 'flex',
          flex: 1,
          overflow: 'hidden',
        }}
      >
        {leftNav}
        {main}
      </div>
      {footer}
    </div>
  );
}
