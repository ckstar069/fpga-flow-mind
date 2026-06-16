/**
 * workbenchTheme: Phase 8 Batch D 工作台设计 token
 *
 * 仅导出常量（颜色/层级），不引入任何运行时依赖。
 * 视觉语义服从产品定位：理解与可视化工具，非审计 dashboard。
 * 强调色单一蓝色系 + 中性青/绿/琥珀/灰，不使用紫色默认风格、不使用红绿裁决色。
 *
 * 设计分层：
 * - NAV.*    深色固定左侧导航（深 slate）
 * - SURFACE.* 浅色主工作区
 * - ACCENT.* 强调与语义色（蓝=操作/选中/可追溯；青=理解；绿=已有/成功；琥珀=注意；灰=未知/弱化）
 */

// ─── 深色左侧导航 ───────────────────────────────────────────────────────
export const NAV = {
  bg: '#1e293b',
  bgHover: '#334155',
  bgActive: '#15315c', // 选中项蓝调底
  bgSubtle: '#24354f',
  border: '#334155',
  divider: '#2c3e54',
  text: '#e2e8f0',
  textMuted: '#94a3b8',
  textDim: '#64748b',
  surface: '#273449', // nav 内卡片底
} as const;

// ─── 浅色主工作区 ───────────────────────────────────────────────────────
export const SURFACE = {
  appBg: '#f1f5f9',
  bg: '#ffffff',
  bgSubtle: '#f8fafc',
  bgHover: '#f1f5f9',
  border: '#e2e8f0',
  borderStrong: '#cbd5e1',
  text: '#1e293b',
  textMuted: '#64748b',
  textDim: '#94a3b8',
} as const;

// ─── 强调与语义色 ───────────────────────────────────────────────────────
export const ACCENT = {
  blue: '#1976d2',
  blueDark: '#1565c0',
  blueSoft: '#e3f2fd',
  blueSoftBorder: '#90caf9',
  teal: '#00838f', // 理解（替代紫色）
  tealSoft: '#e0f2f1',
  green: '#2e7d32',
  greenSoft: '#e8f5e9',
  amber: '#f57c00',
  amberSoft: '#fff8e1',
  amberBorder: '#ffe082',
  red: '#c62828',
  redSoft: '#ffebee',
  redBorder: '#ef9a9a',
  slate: '#546e7a',
  slateSoft: '#eceff1',
} as const;

// ─── 字号阶梯 ───────────────────────────────────────────────────────────
export const FONT = {
  title: 16,
  heading: 14,
  body: 13,
  caption: 12,
  micro: 11,
} as const;
