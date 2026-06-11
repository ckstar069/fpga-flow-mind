import { useCallback, useMemo, useState } from 'react';
import type {
  WorkspaceProfile,
  StageContext,
  WorkspaceWarning,
  CommandError as CommandErrorType,
  StageSummary,
} from '../../types/workspace';
import { openWorkspace, selectStage, CommandError } from '../../lib/tauriCommands';

// ─── 状态机 ───
type AppState =
  | { phase: 'initial' }
  | { phase: 'opening' }
  | { phase: 'loaded'; profile: WorkspaceProfile }
  | { phase: 'error'; error: CommandErrorType }
  | { phase: 'selecting_stage' }
  | { phase: 'stage_loaded'; profile: WorkspaceProfile; stageId: string; context: StageContext }
  | { phase: 'stage_error'; profile: WorkspaceProfile; stageId: string; error: CommandErrorType };

const VALIDITY_LABEL: Record<string, string> = {
  likely_valid: '项目结构符合预期',
  uncertain: '项目结构部分匹配，阶段可能不完整',
  unlikely: '项目结构不符合预期模板',
};

const VALIDITY_COLOR: Record<string, string> = {
  likely_valid: '#2e7d32',
  uncertain: '#f57c00',
  unlikely: '#c62828',
};

const STATUS_LABEL: Record<string, string> = {
  available: '可用',
  empty: '为空',
  naming_anomaly: '命名异常',
  unreadable: '不可读',
  missing: '缺失',
};

export default function WorkspacePage() {
  const [state, setState] = useState<AppState>({ phase: 'initial' });
  const [pathInput, setPathInput] = useState('');

  // ─── 打开项目 ───
  const handleOpen = useCallback(async () => {
    const path = pathInput.trim();
    if (!path) return;
    setState({ phase: 'opening' });
    try {
      const profile = await openWorkspace(path);
      setState({ phase: 'loaded', profile });
    } catch (err) {
      const error: CommandErrorType = err instanceof CommandError
        ? (err as unknown as CommandErrorType)
        : { error_code: 'scan_timeout' as ErrorCodeType, message: String(err), recoverable: false };
      setState({ phase: 'error', error });
    }
  }, [pathInput]);

  // ─── 选择阶段 ───
  const handleSelectStage = useCallback(async (stageId: string) => {
    const profile = state.phase === 'loaded' || state.phase === 'stage_loaded' || state.phase === 'stage_error'
      ? (state as { profile: WorkspaceProfile }).profile
      : null;
    if (!profile) return;
    setState({ phase: 'selecting_stage' });
    try {
      const context = await selectStage(profile.root_path, stageId);
      setState({ phase: 'stage_loaded', profile, stageId, context });
    } catch (err) {
      const error: CommandErrorType = err instanceof CommandError
        ? (err as unknown as CommandErrorType)
        : { error_code: 'scan_timeout' as ErrorCodeType, message: String(err), recoverable: false };
      setState({ phase: 'stage_error', profile, stageId, error });
    }
  }, [state]);

  // ─── 当前 profile 提取 ───
  const currentProfile = useMemo<WorkspaceProfile | null>(() => {
    if (state.phase === 'loaded') return state.profile;
    if (state.phase === 'selecting_stage') return null;
    if (state.phase === 'stage_loaded') return state.profile;
    if (state.phase === 'stage_error') return state.profile;
    return null;
  }, [state]);

  const currentWarnings = useMemo<WorkspaceWarning[]>(() => {
    return currentProfile?.warnings ?? [];
  }, [currentProfile]);

  // ─── 渲染 ───
  return (
    <div style={{ display: 'flex', flexDirection: 'column', height: '100vh', fontFamily: 'system-ui, -apple-system, sans-serif', background: '#f5f5f5' }}>
      {/* 顶部工具栏 */}
      <header style={{ display: 'flex', alignItems: 'center', gap: 16, padding: '12px 24px', background: '#fff', borderBottom: '1px solid #ddd' }}>
        <h1 style={{ fontSize: 18, margin: 0 }}>fpga-flow-mind</h1>
        <div style={{ display: 'flex', gap: 8, flex: 1 }}>
          <input
            type="text"
            placeholder="输入项目路径..."
            value={pathInput}
            onChange={(e) => setPathInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleOpen()}
            style={{ flex: 1, padding: '6px 12px', border: '1px solid #ccc', borderRadius: 4, fontSize: 14 }}
          />
          <button onClick={handleOpen} disabled={state.phase === 'opening'} style={{ padding: '6px 16px', borderRadius: 4, border: '1px solid #ccc', cursor: 'pointer' }}>
            {state.phase === 'opening' ? '扫描中...' : '打开项目'}
          </button>
        </div>
      </header>

      {/* 主内容 */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* 左栏 */}
        <aside style={{ width: 320, minWidth: 280, background: '#fff', borderRight: '1px solid #ddd', overflowY: 'auto', padding: 16 }}>
          {state.phase === 'initial' && (
            <div style={{ color: '#666', textAlign: 'center', marginTop: 40 }}>
              <p>请输入项目路径并点击"打开项目"</p>
            </div>
          )}

          {state.phase === 'opening' && (
            <div style={{ color: '#666', textAlign: 'center', marginTop: 40 }}>
              <p>正在扫描 workspace...</p>
            </div>
          )}

          {state.phase === 'error' && <ErrorPanel error={state.error} />}

          {currentProfile && (
            <>
              <WorkspaceSummary profile={currentProfile} />
              <StageList stages={currentProfile.stages} onSelect={handleSelectStage} />
            </>
          )}
        </aside>

        {/* 右栏 */}
        <main style={{ flex: 1, padding: 24, overflowY: 'auto' }}>
          {state.phase === 'initial' && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#999' }}>
              <p>请从左侧打开一个项目</p>
            </div>
          )}

          {state.phase === 'selecting_stage' && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#666' }}>
              <p>正在加载阶段详情...</p>
            </div>
          )}

          {state.phase === 'stage_loaded' && <StageDetail context={state.context} />}

          {state.phase === 'stage_error' && (
            <div style={{ padding: 24, background: '#fff3e0', borderRadius: 8 }}>
              <h3 style={{ margin: '0 0 8px' }}>阶段加载失败</h3>
              <p style={{ margin: 0 }}>{state.error.message}</p>
            </div>
          )}

          {!['selecting_stage', 'stage_loaded', 'stage_error'].includes(state.phase) && !currentProfile && (
            <div style={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: '#999' }}>
              <p>请从左侧选择一个阶段查看详情</p>
            </div>
          )}
        </main>
      </div>

      {/* 底部 warnings */}
      {currentWarnings.length > 0 && (
        <footer style={{ maxHeight: 200, overflowY: 'auto', background: '#fff8e1', borderTop: '1px solid #ddd', padding: '8px 24px' }}>
          <h4 style={{ margin: '0 0 8px', fontSize: 14 }}>警告 ({currentWarnings.length})</h4>
          <ul style={{ margin: 0, paddingLeft: 20, fontSize: 13 }}>
            {currentWarnings.map((w: WorkspaceWarning, i: number) => (
              <li key={i} style={{ marginBottom: 4 }}>
                <code>{w.error_code}</code>: {w.message}
                {w.source_path && <span style={{ color: '#666' }}> ({w.source_path})</span>}
              </li>
            ))}
          </ul>
        </footer>
      )}
    </div>
  );
}

// ─── 子组件 ───

function ErrorPanel({ error }: { error: CommandErrorType }) {
  return (
    <div style={{ padding: 16, background: '#ffebee', borderRadius: 8, marginBottom: 16 }}>
      <h3 style={{ margin: '0 0 8px', color: '#c62828' }}>错误</h3>
      <p style={{ margin: '0 0 4px' }}>{error.message}</p>
      <code style={{ fontSize: 12, color: '#666' }}>{error.error_code}</code>
    </div>
  );
}

function WorkspaceSummary({ profile }: { profile: WorkspaceProfile }) {
  const stats = Object.entries(profile.file_type_stats);
  return (
    <div style={{ marginBottom: 24 }}>
      <h2 style={{ fontSize: 16, margin: '0 0 8px' }}>{profile.workspace_name}</h2>
      <p style={{ fontSize: 12, color: '#666', margin: '0 0 8px', wordBreak: 'break-all' }}>{profile.root_path}</p>
      <div style={{ display: 'flex', alignItems: 'center', gap: 8, marginBottom: 8 }}>
        <span style={{ fontSize: 13, fontWeight: 600, color: VALIDITY_COLOR[profile.validity] ?? '#333' }}>
          {VALIDITY_LABEL[profile.validity] ?? profile.validity}
        </span>
      </div>
      {profile.external_refs.length > 0 && (
        <p style={{ fontSize: 12, color: '#666', margin: '0 0 8px' }}>
          外部引用: {profile.external_refs.join(', ')}
        </p>
      )}
      {stats.length > 0 && (
        <div style={{ fontSize: 12, color: '#666' }}>
          文件统计: {stats.map(([ext, count]) => `${ext}: ${count}`).join(', ')}
        </div>
      )}
      {profile.error_codes.length > 0 && (
        <div style={{ marginTop: 8, fontSize: 12 }}>
          {profile.error_codes.map((code: ErrorCodeType, i: number) => (
            <span key={i} style={{ display: 'inline-block', padding: '2px 6px', background: '#ffebee', color: '#c62828', borderRadius: 4, marginRight: 4 }}>
              {code}
            </span>
          ))}
        </div>
      )}
    </div>
  );
}

type ErrorCodeType = 'path_not_found' | 'not_directory' | 'permission_denied' | 'no_stage_found' | 'stage_empty' | 'stage_unreadable' | 'file_unreadable' | 'file_too_large' | 'scan_timeout';

function StageList({ stages, onSelect }: { stages: StageSummary[]; onSelect: (id: string) => void }) {
  if (stages.length === 0) {
    return (
      <div style={{ padding: 16, background: '#fafafa', borderRadius: 8, textAlign: 'center' }}>
        <p style={{ margin: 0, color: '#999', fontSize: 14 }}>未识别到阶段目录</p>
      </div>
    );
  }

  return (
    <div>
      <h3 style={{ fontSize: 14, margin: '0 0 12px' }}>阶段列表</h3>
      <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
        {stages.map((stage: StageSummary) => {
          const clickable = stage.status === 'available' || stage.status === 'naming_anomaly';
          return (
            <button
              key={stage.stage_id}
              onClick={() => clickable && onSelect(stage.stage_id)}
              disabled={!clickable}
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'space-between',
                padding: '8px 12px',
                borderRadius: 6,
                border: '1px solid #e0e0e0',
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
              <span style={{ fontSize: 12, color: '#999' }}>{stage.file_count} 文件</span>
            </button>
          );
        })}
      </div>
    </div>
  );
}

function StageDetail({ context }: { context: StageContext }) {
  return (
    <div>
      <h2 style={{ margin: '0 0 16px', fontSize: 20 }}>
        {context.stage_id}
        {context.error_code && (
          <span style={{ fontSize: 13, marginLeft: 12, padding: '2px 8px', background: '#ffebee', color: '#c62828', borderRadius: 4 }}>
            {context.error_code}
          </span>
        )}
      </h2>
      <p style={{ fontSize: 13, color: '#666', margin: '0 0 16px', wordBreak: 'break-all' }}>{context.source_path}</p>

      {context.error_code === 'stage_empty' && (
        <div style={{ padding: 24, background: '#fafafa', borderRadius: 8, textAlign: 'center', marginBottom: 16 }}>
          <p style={{ margin: 0, color: '#999' }}>该阶段无文件</p>
        </div>
      )}

      {context.files.length > 0 && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>文件列表 ({context.files.length})</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 4 }}>
            {context.files.map((f: StageFile, i: number) => (
              <div key={i} style={{ padding: '8px 12px', background: '#fff', borderRadius: 6, border: '1px solid #e0e0e0' }}>
                <div style={{ fontSize: 13, fontWeight: 500, wordBreak: 'break-all' }}>{f.source_path}</div>
                <div style={{ fontSize: 12, color: '#999', marginTop: 4 }}>
                  {f.language} / {f.source_kind}
                  {f.size_bytes !== undefined && ` · ${formatBytes(f.size_bytes)}`}
                </div>
              </div>
            ))}
          </div>
        </div>
      )}

      {context.external_deps.length > 0 && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>外部依赖</h3>
          <div style={{ display: 'flex', flexWrap: 'wrap', gap: 8 }}>
            {context.external_deps.map((dep: string, i: number) => (
              <span key={i} style={{ padding: '4px 10px', background: '#e3f2fd', borderRadius: 4, fontSize: 13 }}>
                {dep}
              </span>
            ))}
          </div>
        </div>
      )}

      {context.upstream_refs.length > 0 && (
        <div style={{ marginBottom: 24 }}>
          <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>上游引用</h3>
          <div style={{ display: 'flex', flexDirection: 'column', gap: 6 }}>
            {context.upstream_refs.map((ref: UpstreamRefType, i: number) => (
              <div key={i} style={{ padding: '8px 12px', background: '#fff', borderRadius: 6, border: '1px solid #e0e0e0' }}>
                <span style={{ fontWeight: 600 }}>{ref.stage_id}</span>
                {ref.interface_file_path && (
                  <span style={{ fontSize: 12, color: '#666', marginLeft: 8 }}>{ref.interface_file_path}</span>
                )}
                <span style={{ fontSize: 11, color: '#999', marginLeft: 8 }}>(推断)</span>
              </div>
            ))}
          </div>
        </div>
      )}

      <button disabled style={{ padding: '8px 24px', borderRadius: 6, border: '1px solid #ccc', background: '#f5f5f5', color: '#999', cursor: 'not-allowed' }}>
        开始分析（Phase 2 后可用）
      </button>
    </div>
  );
}

interface UpstreamRefType {
  stage_id: string;
  interface_file_path?: string;
  inferred: boolean;
}

interface StageFile {
  source_path: string;
  language: string;
  source_kind: string;
  size_bytes?: number;
}

function formatBytes(bytes: number): string {
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}
