export const VALIDITY_LABEL: Record<string, string> = {
  likely_valid: '项目结构符合预期',
  uncertain: '项目结构部分匹配，阶段可能不完整',
  unlikely: '项目结构不符合预期模板',
};

export const VALIDITY_COLOR: Record<string, string> = {
  likely_valid: '#2e7d32',
  uncertain: '#f57c00',
  unlikely: '#c62828',
};

export const STATUS_LABEL: Record<string, string> = {
  available: '可用',
  empty: '为空',
  naming_anomaly: '命名异常',
  unreadable: '不可读',
  missing: '缺失',
};

export function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

export function getStageDisabledReason(status: string): string | null {
  switch (status) {
    case 'empty':
      return '该阶段无可分析文件';
    case 'unreadable':
      return '该阶段不可读';
    default:
      return null;
  }
}
