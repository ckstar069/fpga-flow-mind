import { useEffect, useRef } from 'react';
import type { SourceExcerpt, SourceLine, ExcerptWarning, Language } from '../../../types/workspace';

// ─── 语言标签映射 ───────────────────────────────────────────────────────

const LANGUAGE_LABEL: Record<Language, string> = {
  python: 'Python',
  verilog: 'Verilog',
  systemverilog: 'SystemVerilog',
  markdown: 'Markdown',
  text: '纯文本',
  json: 'JSON',
  yaml: 'YAML',
  toml: 'TOML',
  unknown: '未知',
};

// ─── Props ──────────────────────────────────────────────────────────────

interface SourceExcerptPanelProps {
  excerpt?: SourceExcerpt | null;
  onClose: () => void;
  error?: {
    message: string;
    error_code?: string;
    source_path?: string;
    details?: string;
  } | null;
}

// ─── 主组件 ─────────────────────────────────────────────────────────────

export default function SourceExcerptPanel({
  excerpt,
  onClose,
  error,
}: SourceExcerptPanelProps) {
  const containerRef = useRef<HTMLDivElement>(null);

  // 打开时滚动到片段顶部
  useEffect(() => {
    if (containerRef.current) {
      containerRef.current.scrollTop = 0;
    }
  }, [excerpt]);

  return (
    <div
      ref={containerRef}
      style={{
        marginBottom: 24,
        border: '1px solid #e0e0e0',
        borderRadius: 8,
        background: '#fff',
      }}
    >
      {/* 头部 */}
      <div
        style={{
          display: 'flex',
          alignItems: 'center',
          justifyContent: 'space-between',
          padding: '12px 14px',
          borderBottom: '1px solid #e0e0e0',
          background: '#fafafa',
          borderRadius: '8px 8px 0 0',
        }}
      >
        <div style={{ display: 'flex', alignItems: 'center', gap: 12, flexWrap: 'wrap' }}>
          <span style={{ fontSize: 15, fontWeight: 600 }}>源码片段</span>
          {excerpt && (
            <span
              style={{
                padding: '2px 8px',
                borderRadius: 4,
                fontSize: 12,
                background: '#e3f2fd',
                color: '#1565c0',
              }}
            >
              {LANGUAGE_LABEL[excerpt.language] ?? excerpt.language}
            </span>
          )}
        </div>
        <button
          onClick={onClose}
          style={{
            padding: '4px 10px',
            borderRadius: 4,
            border: '1px solid #ccc',
            background: '#fff',
            cursor: 'pointer',
            fontSize: 12,
          }}
        >
          关闭
        </button>
      </div>

      {/* 路径与行号范围 */}
      {excerpt && (
        <div
          style={{
            padding: '10px 14px',
            borderBottom: '1px solid #e0e0e0',
            fontSize: 12,
            color: '#666',
            wordBreak: 'break-all',
          }}
        >
          <div>
            <span style={{ color: '#999' }}>路径：</span>
            <code style={{ fontSize: 12 }}>{excerpt.location.source_path}</code>
          </div>
          <div style={{ marginTop: 4 }}>
            <span style={{ color: '#999' }}>行号：</span>
            {excerpt.location.line_range.start}–{excerpt.location.line_range.end}
            {excerpt.location.evidence_id && (
              <span style={{ marginLeft: 12, color: '#999' }}>
                evidence: <code>{excerpt.location.evidence_id}</code>
              </span>
            )}
          </div>
        </div>
      )}

      {/* 错误 */}
      {error && (
        <div
          style={{
            padding: 14,
            background: '#ffebee',
            borderBottom: '1px solid #ef9a9a',
            fontSize: 13,
            color: '#c62828',
          }}
        >
          <div style={{ fontWeight: 600, marginBottom: 4 }}>读取源码失败</div>
          {error.error_code && (
            <div style={{ marginBottom: 4 }}>
              错误码：<code>{error.error_code}</code>
            </div>
          )}
          <div style={{ marginBottom: 4 }}>{error.message}</div>
          {error.source_path && (
            <div style={{ marginBottom: 4 }}>
              路径：<code>{error.source_path}</code>
            </div>
          )}
          {error.details && <div>{error.details}</div>}
        </div>
      )}

      {/* 源码区域 */}
      {!error && excerpt && (
        <div
          style={{
            maxHeight: 360,
            overflow: 'auto',
            fontFamily: 'ui-monospace, SFMono-Regular, Menlo, Consolas, monospace',
            fontSize: 13,
            lineHeight: 1.5,
            background: '#fafafa',
          }}
        >
          {excerpt.lines.map((line) => (
            <SourceLineRow key={line.line_number} line={line} />
          ))}
          {excerpt.is_truncated && excerpt.truncation_reason && (
            <div
              style={{
                padding: '8px 14px',
                color: '#999',
                fontStyle: 'italic',
                fontSize: 12,
                borderTop: '1px dashed #e0e0e0',
              }}
            >
              ...（{excerpt.truncation_reason}）
            </div>
          )}
        </div>
      )}

      {/* Warnings */}
      {excerpt && excerpt.warnings.length > 0 && (
        <div
          style={{
            padding: '10px 14px',
            background: '#fff8e1',
            borderTop: '1px solid #ffe082',
            borderRadius: '0 0 8px 8px',
          }}
        >
          <h4 style={{ margin: '0 0 6px', fontSize: 13 }}>警告</h4>
          <ul style={{ margin: 0, paddingLeft: 18, fontSize: 12 }}>
            {excerpt.warnings.map((w, i) => (
              <WarningItem key={i} warning={w} />
            ))}
          </ul>
        </div>
      )}
    </div>
  );
}

function SourceLineRow({ line }: { line: SourceLine }) {
  return (
    <div style={{ display: 'flex' }}>
      <div
        style={{
          minWidth: 48,
          padding: '2px 10px',
          textAlign: 'right',
          color: '#999',
          background: '#f0f0f0',
          userSelect: 'none',
          borderRight: '1px solid #e0e0e0',
        }}
      >
        {line.line_number}
      </div>
      <pre
        style={{
          margin: 0,
          padding: '2px 12px',
          flex: 1,
          whiteSpace: 'pre',
          color: '#333',
        }}
      >
        {line.content}
      </pre>
    </div>
  );
}

function WarningItem({ warning }: { warning: ExcerptWarning }) {
  return (
    <li style={{ marginBottom: 2 }}>
      <code>{warning.error_code}</code>: {warning.message}
    </li>
  );
}
