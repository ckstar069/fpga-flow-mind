import { useCallback, useEffect, useState } from 'react';
import type {
  NetworkMode,
  ProviderConfigInput,
  ProviderKind,
  ProviderStatusResponse,
  ProviderTestConnectionResult,
  ProviderValidationResult,
} from '../../../types/workspace';
import { getProviderStatus, testProviderConnection, validateProviderConfig } from '../../../lib/tauriCommands';
import { ACCENT, FONT, SURFACE } from './workbenchTheme';

export interface ProviderConfigPanelProps {
  isOpen: boolean;
  initialConfig: ProviderConfigInput;
  onClose: () => void;
  onStatusChange: (status: ProviderStatusResponse, config: ProviderConfigInput) => void;
}

const KIND_OPTIONS: { value: ProviderKind; label: string }[] = [
  { value: 'mock', label: 'Mock（本地确定性引擎）' },
  { value: 'fake', label: 'Fake（测试占位）' },
  { value: 'openai', label: 'OpenAI 兼容' },
  { value: 'anthropic', label: 'Anthropic 兼容' },
];

const NETWORK_OPTIONS: { value: NetworkMode; label: string }[] = [
  { value: 'disabled', label: '禁用真实网络（默认）' },
  { value: 'proxy', label: '代理/本地内网' },
  { value: 'allow', label: '允许真实网络' },
];

export default function ProviderConfigPanel({
  isOpen,
  initialConfig,
  onClose,
  onStatusChange,
}: ProviderConfigPanelProps) {
  const [enabled, setEnabled] = useState(false);
  const [kind, setKind] = useState<ProviderKind>('mock');
  const [model, setModel] = useState('mock');
  const [baseUrl, setBaseUrl] = useState('');
  const [apiKeyInput, setApiKeyInput] = useState('');
  const [networkMode, setNetworkMode] = useState<NetworkMode>('disabled');
  const [retryLimit, setRetryLimit] = useState(2);
  const [timeoutMs, setTimeoutMs] = useState(60000);
  const [rateLimitPerMin] = useState(60);

  const [validation, setValidation] = useState<ProviderValidationResult | null>(null);
  const [testResult, setTestResult] = useState<ProviderTestConnectionResult | null>(null);
  const [testing, setTesting] = useState(false);
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (isOpen) {
      setEnabled(initialConfig.enabled);
      setKind(initialConfig.kind);
      setModel(initialConfig.model);
      setBaseUrl(initialConfig.base_url ?? '');
      setNetworkMode(initialConfig.network_mode);
      setRetryLimit(initialConfig.retry_limit);
      setTimeoutMs(initialConfig.timeout_ms);
    } else {
      setApiKeyInput('');
      setTestResult(null);
      setValidation(null);
    }
  }, [initialConfig, isOpen]);

  const buildConfig = useCallback((): ProviderConfigInput & { api_key?: string } => {
    return {
      enabled,
      kind,
      model: model.trim() || 'mock',
      base_url: baseUrl.trim() || undefined,
      timeout_ms: Math.max(1, timeoutMs),
      retry_limit: Math.min(10, Math.max(0, retryLimit)),
      rate_limit_per_min: Math.max(1, rateLimitPerMin),
      network_mode: networkMode,
      api_key: apiKeyInput.trim() || undefined,
    };
  }, [enabled, kind, model, baseUrl, apiKeyInput, networkMode, retryLimit, timeoutMs, rateLimitPerMin]);

  const buildSanitizedConfig = useCallback((): ProviderConfigInput => {
    const config = buildConfig();
    return {
      enabled: config.enabled,
      kind: config.kind,
      model: config.model,
      base_url: config.base_url,
      timeout_ms: config.timeout_ms,
      retry_limit: config.retry_limit,
      rate_limit_per_min: config.rate_limit_per_min,
      network_mode: config.network_mode,
    };
  }, [buildConfig]);

  const handleValidate = useCallback(async () => {
    setLoading(true);
    try {
      const result = await validateProviderConfig(buildConfig());
      setValidation(result);
    } catch (err) {
      setValidation({
        valid: false,
        network_enabled: false,
        issues: [err instanceof Error ? err.message : String(err)],
      });
    } finally {
      setLoading(false);
    }
  }, [buildConfig]);

  const handleTestConnection = useCallback(async () => {
    setTesting(true);
    setTestResult(null);
    try {
      const result = await testProviderConnection(buildConfig());
      setTestResult(result);
    } catch (err) {
      setTestResult({
        success: false,
        code: 'command_error',
        message: err instanceof Error ? err.message : String(err),
      });
    } finally {
      setTesting(false);
    }
  }, [buildConfig]);

  const handleApply = useCallback(async () => {
    setLoading(true);
    try {
      const sanitizedConfig = buildSanitizedConfig();
      const status = await getProviderStatus(sanitizedConfig);
      onStatusChange(status, sanitizedConfig);
      setApiKeyInput('');
      onClose();
    } catch (err) {
      setValidation({
        valid: false,
        network_enabled: false,
        issues: [err instanceof Error ? err.message : String(err)],
      });
    } finally {
      setLoading(false);
    }
  }, [buildSanitizedConfig, onStatusChange, onClose]);

  const handleClearKey = useCallback(() => {
    setApiKeyInput('');
  }, []);

  if (!isOpen) {
    return null;
  }

  return (
    <div
      role="presentation"
      style={{
        position: 'fixed',
        inset: 0,
        zIndex: 1000,
        display: 'flex',
        justifyContent: 'flex-end',
        background: 'transparent',
        border: 'none',
        padding: 0,
      }}
      onClick={onClose}
      onKeyDown={(e) => {
        if (e.key === 'Escape') {
          onClose();
        }
      }}
    >
      <div
        role="dialog"
        aria-modal="true"
        style={{
          width: 380,
          height: '100%',
          background: SURFACE.bg,
          borderLeft: `1px solid ${SURFACE.border}`,
          boxShadow: '-4px 0 16px rgba(0,0,0,0.08)',
          padding: 20,
          display: 'flex',
          flexDirection: 'column',
          gap: 16,
          overflowY: 'auto',
        }}
        onClick={(e) => e.stopPropagation()}
        onKeyDown={(e) => e.stopPropagation()}
        tabIndex={-1}
      >
        <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
          <span style={{ fontSize: FONT.heading, fontWeight: 600, color: SURFACE.text }}>
            LLM Provider 配置
          </span>
          <button
            type="button"
            onClick={onClose}
            style={{
              background: 'transparent',
              border: 'none',
              cursor: 'pointer',
              fontSize: FONT.heading,
              color: SURFACE.textMuted,
            }}
          >
            ×
          </button>
        </div>

        <label style={{ display: 'flex', alignItems: 'center', gap: 8, cursor: 'pointer' }}>
          <input
            type="checkbox"
            checked={enabled}
            onChange={(e) => setEnabled(e.target.checked)}
          />
          <span style={{ fontSize: FONT.body, color: SURFACE.text }}>启用真实 LLM</span>
        </label>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={{ fontSize: FONT.caption, color: SURFACE.textMuted }}>Provider 类别</span>
          <select
            value={kind}
            onChange={(e) => setKind(e.target.value as ProviderKind)}
            style={{
              padding: '6px 8px',
              borderRadius: 4,
              border: `1px solid ${SURFACE.borderStrong}`,
              fontSize: FONT.body,
            }}
          >
            {KIND_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={{ fontSize: FONT.caption, color: SURFACE.textMuted }}>模型</span>
          <input
            type="text"
            value={model}
            onChange={(e) => setModel(e.target.value)}
            placeholder="例如 gpt-4"
            style={{
              padding: '6px 8px',
              borderRadius: 4,
              border: `1px solid ${SURFACE.borderStrong}`,
              fontSize: FONT.body,
            }}
          />
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={{ fontSize: FONT.caption, color: SURFACE.textMuted }}>Base URL（可选）</span>
          <input
            type="text"
            value={baseUrl}
            onChange={(e) => setBaseUrl(e.target.value)}
            placeholder="例如 https://api.openai.com/v1"
            style={{
              padding: '6px 8px',
              borderRadius: 4,
              border: `1px solid ${SURFACE.borderStrong}`,
              fontSize: FONT.body,
            }}
          />
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'space-between' }}>
            <span style={{ fontSize: FONT.caption, color: SURFACE.textMuted }}>API Key</span>
          </div>
          <input
            type="password"
            value={apiKeyInput}
            onChange={(e) => setApiKeyInput(e.target.value)}
            placeholder="输入 API Key（仅本次面板打开期间保留）"
            style={{
              padding: '6px 8px',
              borderRadius: 4,
              border: `1px solid ${SURFACE.borderStrong}`,
              fontSize: FONT.body,
            }}
          />
          <button
            type="button"
            onClick={handleClearKey}
            style={{
              alignSelf: 'flex-start',
              padding: '2px 8px',
              borderRadius: 4,
              border: `1px solid ${SURFACE.borderStrong}`,
              background: SURFACE.bgSubtle,
              cursor: 'pointer',
              fontSize: FONT.caption,
              color: SURFACE.textMuted,
            }}
          >
            清除 Key
          </button>
        </div>

        <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
          <span style={{ fontSize: FONT.caption, color: SURFACE.textMuted }}>网络模式</span>
          <select
            value={networkMode}
            onChange={(e) => setNetworkMode(e.target.value as NetworkMode)}
            style={{
              padding: '6px 8px',
              borderRadius: 4,
              border: `1px solid ${SURFACE.borderStrong}`,
              fontSize: FONT.body,
            }}
          >
            {NETWORK_OPTIONS.map((opt) => (
              <option key={opt.value} value={opt.value}>{opt.label}</option>
            ))}
          </select>
        </div>

        <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 12 }}>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <span style={{ fontSize: FONT.caption, color: SURFACE.textMuted }}>重试次数</span>
            <input
              type="number"
              min={0}
              max={10}
              value={retryLimit}
              onChange={(e) => setRetryLimit(parseInt(e.target.value || '0', 10))}
              style={{
                padding: '6px 8px',
                borderRadius: 4,
                border: `1px solid ${SURFACE.borderStrong}`,
                fontSize: FONT.body,
              }}
            />
          </div>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            <span style={{ fontSize: FONT.caption, color: SURFACE.textMuted }}>超时（ms）</span>
            <input
              type="number"
              min={1}
              value={timeoutMs}
              onChange={(e) => setTimeoutMs(parseInt(e.target.value || '1', 10))}
              style={{
                padding: '6px 8px',
                borderRadius: 4,
                border: `1px solid ${SURFACE.borderStrong}`,
                fontSize: FONT.body,
              }}
            />
          </div>
        </div>

        <div style={{ display: 'flex', gap: 8 }}>
          <button
            type="button"
            onClick={handleValidate}
            disabled={loading}
            style={{
              flex: 1,
              padding: '8px 12px',
              borderRadius: 4,
              border: 'none',
              background: ACCENT.blueSoft,
              color: ACCENT.blue,
              cursor: 'pointer',
              fontSize: FONT.body,
            }}
          >
            {loading ? '校验中...' : '校验配置'}
          </button>
          <button
            type="button"
            onClick={handleTestConnection}
            disabled={testing}
            style={{
              flex: 1,
              padding: '8px 12px',
              borderRadius: 4,
              border: 'none',
              background: ACCENT.tealSoft,
              color: ACCENT.teal,
              cursor: 'pointer',
              fontSize: FONT.body,
            }}
          >
            {testing ? '测试中...' : '测试连接'}
          </button>
        </div>

        {validation && (
          <div
            style={{
              padding: 10,
              borderRadius: 4,
              background: validation.valid ? ACCENT.greenSoft : ACCENT.amberSoft,
              border: `1px solid ${validation.valid ? ACCENT.green : ACCENT.amber}`,
              fontSize: FONT.caption,
              color: validation.valid ? ACCENT.green : ACCENT.amber,
            }}
          >
            {validation.valid
              ? `配置格式有效${validation.network_enabled ? '，真实网络模式为允许（未发起连接）' : ''}`
              : validation.issues.join('；')}
          </div>
        )}

        {testResult && (
          <div
            style={{
              padding: 10,
              borderRadius: 4,
              background: testResult.success ? ACCENT.greenSoft : ACCENT.amberSoft,
              border: `1px solid ${testResult.success ? ACCENT.green : ACCENT.amber}`,
              fontSize: FONT.caption,
              color: testResult.success ? ACCENT.green : ACCENT.amber,
            }}
          >
            [{testResult.code}] {testResult.message}
          </div>
        )}

        <div
          style={{
            padding: 12,
            borderRadius: 4,
            background: ACCENT.blueSoft,
            border: `1px solid ${ACCENT.blueSoftBorder}`,
            fontSize: FONT.caption,
            color: SURFACE.text,
            display: 'flex',
            flexDirection: 'column',
            gap: 8,
          }}
        >
          <strong style={{ fontSize: FONT.body }}>安全说明</strong>
          <span>API Key 仅在本次配置面板打开期间保存在内存中，不会被写入 localStorage、sessionStorage、磁盘或日志。</span>
          <span>关闭“启用真实 LLM”后，所有分析将回退到本地 Mock 模式，不发起任何网络请求。</span>
          <span>连接测试仅在启用真实 LLM、选择允许真实网络并提供 API Key 时发送最小 ping；不会发送项目源码、evidence、Q&A、session 或截图。</span>
        </div>

        <button
          type="button"
          onClick={handleApply}
          disabled={loading}
          style={{
            marginTop: 'auto',
            padding: '10px 16px',
            borderRadius: 4,
            border: 'none',
            background: ACCENT.blue,
            color: '#fff',
            cursor: 'pointer',
            fontSize: FONT.body,
          }}
        >
          {loading ? '应用中...' : '应用'}
        </button>
      </div>
    </div>
  );
}
