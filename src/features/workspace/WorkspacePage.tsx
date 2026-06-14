import { useCallback, useMemo, useState, useRef } from 'react';
import type {
  WorkspaceProfile,
  StageContext,
  WorkspaceWarning,
  EvidenceCollection,
  ImplementationUnderstanding,
  ViewGraph,
  SelectedTraceTarget,
  TraceRefResolved,
  SourceExcerpt,
  GroundedAnswer,
  GroundedAnswerCitation,
} from '../../types/workspace';
import {
  openWorkspace,
  selectStage,
  collectEvidence,
  generateUnderstanding,
  generateViews,
  resolveTraceTarget,
  getSourceExcerpt,
  askGroundedQuestion,
  CommandError,
} from '../../lib/tauriCommands';
import type { CommandError as CommandErrorType } from '../../types/workspace';
import type { UiError } from './workspaceUiTypes';

import ErrorPanel from './components/ErrorPanel';
import WorkspaceSummary from './components/WorkspaceSummary';
import StageList from './components/StageList';
import StageDetail from './components/StageDetail';

// ─── 状态机 ───
type AppState =
  | { phase: 'initial' }
  | { phase: 'opening' }
  | { phase: 'loaded'; profile: WorkspaceProfile }
  | { phase: 'error'; error: UiError }
  | { phase: 'selecting_stage'; profile: WorkspaceProfile; stageId: string }
  | { phase: 'stage_loaded'; profile: WorkspaceProfile; stageId: string; context: StageContext }
  | { phase: 'stage_error'; profile: WorkspaceProfile; stageId: string; error: UiError }
  | { phase: 'collecting_evidence'; profile: WorkspaceProfile; stageId: string; context: StageContext }
  | { phase: 'evidence_loaded'; profile: WorkspaceProfile; stageId: string; context: StageContext; evidence: EvidenceCollection }
  | { phase: 'evidence_error'; profile: WorkspaceProfile; stageId: string; context: StageContext; error: UiError }
  | { phase: 'understanding_loading'; profile: WorkspaceProfile; stageId: string; context: StageContext; evidence?: EvidenceCollection }
  | { phase: 'understanding_loaded'; profile: WorkspaceProfile; stageId: string; context: StageContext; evidence?: EvidenceCollection; understanding: ImplementationUnderstanding }
  | { phase: 'understanding_error'; profile: WorkspaceProfile; stageId: string; context: StageContext; evidence?: EvidenceCollection; understandingError: UiError }
  | { phase: 'views_loading'; profile: WorkspaceProfile; stageId: string; context: StageContext; evidence?: EvidenceCollection; understanding: ImplementationUnderstanding }
  | { phase: 'views_loaded'; profile: WorkspaceProfile; stageId: string; context: StageContext; evidence?: EvidenceCollection; understanding: ImplementationUnderstanding; views: ViewGraph[] }
  | { phase: 'views_error'; profile: WorkspaceProfile; stageId: string; context: StageContext; evidence?: EvidenceCollection; understanding: ImplementationUnderstanding; viewsError: UiError };

function makeUiError(err: unknown): UiError {
  return err instanceof CommandError
    ? (err as unknown as CommandErrorType)
    : { error_code: 'frontend_error', message: String(err), recoverable: false };
}

export default function WorkspacePage() {
  const [state, setState] = useState<AppState>({ phase: 'initial' });
  const [pathInput, setPathInput] = useState('');

  // Phase 5 trace 状态
  const [selectedTraceTarget, setSelectedTraceTarget] = useState<SelectedTraceTarget | null>(null);
  const [resolvedTraces, setResolvedTraces] = useState<TraceRefResolved[]>([]);
  const [traceLoading, setTraceLoading] = useState(false);
  const [traceError, setTraceError] = useState<UiError | null>(null);
  const [sourceExcerpt, setSourceExcerpt] = useState<SourceExcerpt | null>(null);
  const [excerptError, setExcerptError] = useState<UiError | null>(null);
  const [highlightedEvidenceId, setHighlightedEvidenceId] = useState<string | null>(null);
  const [currentSourceEvidenceId, setCurrentSourceEvidenceId] = useState<string | null>(null);

  // Phase 5 Grounded Q&A 状态
  const [groundedAnswer, setGroundedAnswer] = useState<GroundedAnswer | null>(null);
  const [groundedAnswerLoading, setGroundedAnswerLoading] = useState(false);
  const [groundedAnswerError, setGroundedAnswerError] = useState<UiError | null>(null);

  // 用于取消旧 trace/excerpt/qa 请求的守卫
  const traceGuardRef = useRef<number>(0);
  const excerptGuardRef = useRef<number>(0);
  const qaGuardRef = useRef<number>(0);
  const highlightTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  // ─── 清空 trace 相关状态 ───
  const clearTraceState = useCallback(() => {
    // 递增守卫，使所有旧 trace/excerpt/qa 请求失效
    traceGuardRef.current += 1;
    excerptGuardRef.current += 1;
    qaGuardRef.current += 1;
    setSelectedTraceTarget(null);
    setResolvedTraces([]);
    setTraceLoading(false);
    setTraceError(null);
    setSourceExcerpt(null);
    setExcerptError(null);
    setHighlightedEvidenceId(null);
    setCurrentSourceEvidenceId(null);
    setGroundedAnswer(null);
    setGroundedAnswerLoading(false);
    setGroundedAnswerError(null);
    if (highlightTimerRef.current) {
      clearTimeout(highlightTimerRef.current);
      highlightTimerRef.current = null;
    }
  }, []);

  // ─── 打开项目 ───
  const handleOpen = useCallback(async () => {
    const path = pathInput.trim();
    if (!path) return;
    clearTraceState();
    setState({ phase: 'opening' });
    try {
      const profile = await openWorkspace(path);
      setState({ phase: 'loaded', profile });
    } catch (err) {
      setState({ phase: 'error', error: makeUiError(err) });
    }
  }, [pathInput, clearTraceState]);

  // ─── 选择阶段 ───
  const handleSelectStage = useCallback(
    async (stageId: string) => {
      const profile =
        state.phase === 'loaded' ||
        state.phase === 'stage_loaded' ||
        state.phase === 'stage_error' ||
        state.phase === 'evidence_loaded' ||
        state.phase === 'evidence_error' ||
        state.phase === 'understanding_loaded' ||
        state.phase === 'understanding_error' ||
        state.phase === 'views_loaded' ||
        state.phase === 'views_error'
          ? (state as { profile: WorkspaceProfile }).profile
          : null;
      if (!profile) return;
      clearTraceState();
      setState({ phase: 'selecting_stage', profile, stageId });
      try {
        const context = await selectStage(profile.root_path, stageId);
        setState({ phase: 'stage_loaded', profile, stageId, context });
      } catch (err) {
        setState({ phase: 'stage_error', profile, stageId, error: makeUiError(err) });
      }
    },
    [state, clearTraceState]
  );

  // ─── 收集证据 ───
  // 允许从以下状态重新收集：stage_loaded / evidence_* / understanding_loaded / understanding_error / views_loaded / views_error
  // 禁止在 loading 状态（understanding_loading / views_loading）下启动收集
  const handleCollectEvidence = useCallback(async () => {
    if (
      state.phase !== 'stage_loaded' &&
      state.phase !== 'evidence_loaded' &&
      state.phase !== 'evidence_error' &&
      state.phase !== 'understanding_loaded' &&
      state.phase !== 'understanding_error' &&
      state.phase !== 'views_loaded' &&
      state.phase !== 'views_error'
    ) return;
    const { profile, stageId, context } = state as {
      profile: WorkspaceProfile;
      stageId: string;
      context: StageContext;
    };
    // 进入 collecting_evidence 时自动清除旧 understanding / views / trace
    clearTraceState();
    setState({ phase: 'collecting_evidence', profile, stageId, context });
    try {
      const evidence = await collectEvidence(profile.root_path, stageId);
      setState({ phase: 'evidence_loaded', profile, stageId, context, evidence });
    } catch (err) {
      setState({ phase: 'evidence_error', profile, stageId, context, error: makeUiError(err) });
    }
  }, [state, clearTraceState]);

  // ─── 生成理解 ───
  const handleGenerateUnderstanding = useCallback(async () => {
    if (
      state.phase !== 'stage_loaded' &&
      state.phase !== 'evidence_loaded' &&
      state.phase !== 'evidence_error' &&
      state.phase !== 'understanding_loaded' &&
      state.phase !== 'understanding_error' &&
      state.phase !== 'views_loaded' &&
      state.phase !== 'views_error'
    ) return;
    const { profile, stageId, context } = state as {
      profile: WorkspaceProfile;
      stageId: string;
      context: StageContext;
    };
    const evidence =
      'evidence' in state ? (state as { evidence?: EvidenceCollection }).evidence : undefined;
    clearTraceState();
    setState({ phase: 'understanding_loading', profile, stageId, context, evidence });
    try {
      const understanding = await generateUnderstanding(profile.root_path, stageId);
      setState({ phase: 'understanding_loaded', profile, stageId, context, evidence, understanding });
    } catch (err) {
      setState({
        phase: 'understanding_error',
        profile,
        stageId,
        context,
        evidence,
        understandingError: makeUiError(err),
      });
    }
  }, [state, clearTraceState]);

  // ─── 生成视图 ───
  const handleGenerateViews = useCallback(async () => {
    if (
      state.phase !== 'understanding_loaded' &&
      state.phase !== 'views_loaded' &&
      state.phase !== 'views_error'
    ) return;
    const { profile, stageId, context, evidence, understanding } = state as {
      profile: WorkspaceProfile;
      stageId: string;
      context: StageContext;
      evidence?: EvidenceCollection;
      understanding: ImplementationUnderstanding;
    };
    clearTraceState();
    setState({ phase: 'views_loading', profile, stageId, context, evidence, understanding });
    try {
      const views = await generateViews(understanding);
      setState({ phase: 'views_loaded', profile, stageId, context, evidence, understanding, views });
    } catch (err) {
      setState({
        phase: 'views_error',
        profile,
        stageId,
        context,
        evidence,
        understanding,
        viewsError: makeUiError(err),
      });
    }
  }, [state, clearTraceState]);

  // ─── 当前 profile 提取 ───
  const currentProfile = useMemo<WorkspaceProfile | null>(() => {
    if (state.phase === 'loaded') return state.profile;
    if (state.phase === 'selecting_stage') return state.profile;
    if (state.phase === 'stage_loaded') return state.profile;
    if (state.phase === 'stage_error') return state.profile;
    if (state.phase === 'collecting_evidence') return state.profile;
    if (state.phase === 'evidence_loaded') return state.profile;
    if (state.phase === 'evidence_error') return state.profile;
    if (state.phase === 'understanding_loading') return state.profile;
    if (state.phase === 'understanding_loaded') return state.profile;
    if (state.phase === 'views_loading') return state.profile;
    if (state.phase === 'understanding_error') return state.profile;
    if (state.phase === 'views_loaded') return state.profile;
    if (state.phase === 'views_error') return state.profile;
    return null;
  }, [state]);

  const currentWarnings = useMemo<WorkspaceWarning[]>(() => {
    return currentProfile?.warnings ?? [];
  }, [currentProfile]);

  const selectedStageId = useMemo<string | null>(() => {
    if (state.phase === 'selecting_stage') return state.stageId;
    if (state.phase === 'stage_loaded') return state.stageId;
    if (state.phase === 'stage_error') return state.stageId;
    if (state.phase === 'collecting_evidence') return state.stageId;
    if (state.phase === 'evidence_loaded') return state.stageId;
    if (state.phase === 'evidence_error') return state.stageId;
    if (state.phase === 'understanding_loading') return state.stageId;
    if (state.phase === 'understanding_loaded') return state.stageId;
    if (state.phase === 'understanding_error') return state.stageId;
    if (state.phase === 'views_loading') return state.stageId;
    if (state.phase === 'views_loaded') return state.stageId;
    if (state.phase === 'views_error') return state.stageId;
    return null;
  }, [state]);

  const isLoadingStage =
    state.phase === 'selecting_stage' ||
    state.phase === 'collecting_evidence' ||
    state.phase === 'understanding_loading' ||
    state.phase === 'views_loading';

  // ─── 右栏 evidence + understanding 状态提取 ───
  const evidenceState = useMemo(() => {
    if (state.phase === 'collecting_evidence') {
      return { context: state.context, isCollecting: true };
    }
    if (state.phase === 'evidence_loaded') {
      return { context: state.context, evidence: state.evidence };
    }
    if (state.phase === 'evidence_error') {
      return { context: state.context, evidenceError: state.error };
    }
    if (state.phase === 'stage_loaded') {
      return { context: state.context };
    }
    if (state.phase === 'understanding_loading') {
      return {
        context: state.context,
        evidence: state.evidence,
        understandingLoading: true,
      };
    }
    if (state.phase === 'understanding_loaded') {
      return {
        context: state.context,
        evidence: state.evidence,
        understanding: state.understanding,
      };
    }
    if (state.phase === 'understanding_error') {
      return {
        context: state.context,
        evidence: state.evidence,
        understandingError: state.understandingError,
      };
    }
    if (state.phase === 'views_loading') {
      return {
        context: state.context,
        evidence: state.evidence,
        understanding: state.understanding,
        viewsLoading: true,
      };
    }
    if (state.phase === 'views_loaded') {
      return {
        context: state.context,
        evidence: state.evidence,
        understanding: state.understanding,
        views: state.views,
      };
    }
    if (state.phase === 'views_error') {
      return {
        context: state.context,
        evidence: state.evidence,
        understanding: state.understanding,
        viewsError: state.viewsError,
      };
    }
    return null;
  }, [state]);

  // ─── trace/excerpt 请求守卫 ───
  const currentStageIdForTrace = selectedStageId;

  // ─── 处理视图节点选择 ───
  const handleSelectTraceTarget = useCallback(
    async (target: SelectedTraceTarget) => {
      traceGuardRef.current += 1;
      excerptGuardRef.current += 1;
      setSelectedTraceTarget(target);
      setTraceError(null);
      setSourceExcerpt(null);
      setExcerptError(null);
      setHighlightedEvidenceId(null);
      setCurrentSourceEvidenceId(null);

      if (
        state.phase !== 'views_loaded' ||
        !state.views ||
        !state.understanding ||
        !state.evidence
      ) {
        setResolvedTraces([]);
        return;
      }

      const guard = traceGuardRef.current;
      setTraceLoading(true);
      try {
        const traces = await resolveTraceTarget(
          target,
          state.understanding,
          state.evidence,
          state.views
        );
        // 仅在请求仍属于当前状态时更新结果
        if (guard === traceGuardRef.current) {
          setResolvedTraces(traces);
          setTraceLoading(false);
        }
      } catch (err) {
        if (guard === traceGuardRef.current) {
          setTraceError(makeUiError(err));
          setTraceLoading(false);
        }
      }
    },
    [state]
  );

  // ─── 清空选择 ───
  const handleClearTraceTarget = useCallback(() => {
    traceGuardRef.current += 1;
    excerptGuardRef.current += 1;
    setSelectedTraceTarget(null);
    setResolvedTraces([]);
    setTraceError(null);
    setSourceExcerpt(null);
    setExcerptError(null);
    setHighlightedEvidenceId(null);
    setCurrentSourceEvidenceId(null);
  }, []);

  // ─── 查看源码片段 ───
  const handleViewSource = useCallback(
    async (location: {
      source_path: string;
      line_range: { start: number; end: number };
      evidence_id?: string;
    }) => {
      if (currentStageIdForTrace == null) return;
      const profile = currentProfile;
      if (!profile) return;

      excerptGuardRef.current += 1;
      setExcerptError(null);
      setSourceExcerpt(null);
      if (location.evidence_id) {
        setCurrentSourceEvidenceId(location.evidence_id);
      }

      const guard = excerptGuardRef.current;
      try {
        const excerpt = await getSourceExcerpt(location, profile.root_path);
        if (guard === excerptGuardRef.current) {
          setSourceExcerpt(excerpt);
        }
      } catch (err) {
        if (guard === excerptGuardRef.current) {
          setExcerptError(makeUiError(err));
        }
      }
    },
    [currentProfile, currentStageIdForTrace]
  );

  // ─── 关闭源码片段 ───
  const handleCloseSourceExcerpt = useCallback(() => {
    excerptGuardRef.current += 1;
    setSourceExcerpt(null);
    setExcerptError(null);
    setCurrentSourceEvidenceId(null);
  }, []);

  // ─── 定位 evidence 高亮 ───
  const handleLocateEvidence = useCallback((evidenceId: string) => {
    setHighlightedEvidenceId(evidenceId);
    setCurrentSourceEvidenceId(evidenceId);
    if (highlightTimerRef.current) {
      clearTimeout(highlightTimerRef.current);
    }
    highlightTimerRef.current = setTimeout(() => {
      setHighlightedEvidenceId((current) =>
        current === evidenceId ? null : current
      );
    }, 3000);
  }, []);

  // ─── evidence 卡片点击 ───
  const handleEvidenceSelect = useCallback((evidenceId: string) => {
    setHighlightedEvidenceId(evidenceId);
    if (highlightTimerRef.current) {
      clearTimeout(highlightTimerRef.current);
    }
    highlightTimerRef.current = setTimeout(() => {
      setHighlightedEvidenceId((current) =>
        current === evidenceId ? null : current
      );
    }, 3000);
  }, []);

  // ─── Grounded Q&A 提问 ───
  const handleAskGroundedQuestion = useCallback(
    async (questionText: string) => {
      const stageId = selectedStageId;
      const profile = currentProfile;
      const evidence =
        'evidence' in state
          ? (state as { evidence?: EvidenceCollection }).evidence
          : undefined;
      const understanding =
        'understanding' in state
          ? (state as { understanding?: ImplementationUnderstanding }).understanding
          : undefined;

      if (!profile || !stageId || !evidence || !understanding) return;

      qaGuardRef.current += 1;
      setGroundedAnswer(null);
      setGroundedAnswerError(null);
      setGroundedAnswerLoading(true);

      const guard = qaGuardRef.current;
      try {
        const views = 'views' in state ? (state as { views?: ViewGraph[] }).views : undefined;
        const answer = await askGroundedQuestion(
          {
            question: questionText,
            stage_id: stageId,
            selected_target: selectedTraceTarget ?? undefined,
            understanding,
            evidence_collection: evidence,
          },
          views,
          resolvedTraces
        );
        if (guard === qaGuardRef.current) {
          setGroundedAnswer(answer);
          setGroundedAnswerLoading(false);
        }
      } catch (err) {
        if (guard === qaGuardRef.current) {
          setGroundedAnswerError(makeUiError(err));
          setGroundedAnswerLoading(false);
        }
      }
    },
    [currentProfile, selectedStageId, state, selectedTraceTarget, resolvedTraces]
  );

  // ─── Grounded Q&A citation 点击 ───
  const handleGroundedCitationClick = useCallback(
    (citation: GroundedAnswerCitation) => {
      if (citation.evidence_id) {
        setHighlightedEvidenceId(citation.evidence_id);
        setCurrentSourceEvidenceId(citation.evidence_id);
        if (highlightTimerRef.current) {
          clearTimeout(highlightTimerRef.current);
        }
        highlightTimerRef.current = setTimeout(() => {
          setHighlightedEvidenceId((current) =>
            current === citation.evidence_id ? null : current
          );
        }, 3000);
      }
      if (citation.source_location) {
        handleViewSource({
          source_path: citation.source_location.source_path,
          line_range: citation.source_location.line_range,
          evidence_id: citation.evidence_id,
        });
      }
    },
    [handleViewSource]
  );

  // ─── 渲染 ───
  return (
    <div
      style={{
        display: 'flex',
        flexDirection: 'column',
        height: '100vh',
        fontFamily: 'system-ui, -apple-system, sans-serif',
        background: '#f5f5f5',
      }}
    >
      {/* 顶部工具栏 */}
      <header
        style={{
          display: 'flex',
          alignItems: 'center',
          gap: 16,
          padding: '12px 24px',
          background: '#fff',
          borderBottom: '1px solid #ddd',
        }}
      >
        <h1 style={{ fontSize: 18, margin: 0 }}>fpga-flow-mind</h1>
        <div style={{ display: 'flex', gap: 8, flex: 1 }}>
          <input
            type="text"
            placeholder="输入项目路径..."
            value={pathInput}
            onChange={(e) => setPathInput(e.target.value)}
            onKeyDown={(e) => e.key === 'Enter' && handleOpen()}
            style={{
              flex: 1,
              padding: '6px 12px',
              border: '1px solid #ccc',
              borderRadius: 4,
              fontSize: 14,
            }}
          />
          <button
            onClick={handleOpen}
            disabled={state.phase === 'opening'}
            style={{
              padding: '6px 16px',
              borderRadius: 4,
              border: '1px solid #ccc',
              cursor: 'pointer',
            }}
          >
            {state.phase === 'opening' ? '扫描中...' : '打开项目'}
          </button>
        </div>
      </header>

      {/* 主内容 */}
      <div style={{ display: 'flex', flex: 1, overflow: 'hidden' }}>
        {/* 左栏 */}
        <aside
          style={{
            width: 320,
            minWidth: 280,
            background: '#fff',
            borderRight: '1px solid #ddd',
            overflowY: 'auto',
            padding: 16,
          }}
        >
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
              <StageList
                stages={currentProfile.stages}
                selectedStageId={selectedStageId}
                isLoading={isLoadingStage}
                onSelect={handleSelectStage}
              />
            </>
          )}
        </aside>

        {/* 右栏 */}
        <main style={{ flex: 1, padding: 24, overflowY: 'auto' }}>
          {state.phase === 'initial' && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                height: '100%',
                color: '#999',
              }}
            >
              <p>请从左侧打开一个项目</p>
            </div>
          )}

          {state.phase === 'selecting_stage' && (
            <div
              style={{
                display: 'flex',
                alignItems: 'center',
                justifyContent: 'center',
                height: '100%',
                color: '#666',
              }}
            >
              <p>
                正在加载阶段详情：
                {state.stageId}
              </p>
            </div>
          )}

          {evidenceState && (
            <StageDetail
              context={evidenceState.context}
              evidence={'evidence' in evidenceState ? evidenceState.evidence : undefined}
              evidenceError={'evidenceError' in evidenceState ? evidenceState.evidenceError : undefined}
              isCollecting={'isCollecting' in evidenceState ? evidenceState.isCollecting : undefined}
              onCollectEvidence={handleCollectEvidence}
              understanding={'understanding' in evidenceState ? evidenceState.understanding : undefined}
              understandingLoading={'understandingLoading' in evidenceState ? evidenceState.understandingLoading : undefined}
              understandingError={'understandingError' in evidenceState ? evidenceState.understandingError : undefined}
              onGenerateUnderstanding={handleGenerateUnderstanding}
              views={'views' in evidenceState ? evidenceState.views : undefined}
              viewsLoading={'viewsLoading' in evidenceState ? evidenceState.viewsLoading : undefined}
              viewsError={'viewsError' in evidenceState ? evidenceState.viewsError : undefined}
              onGenerateViews={handleGenerateViews}
              selectedTraceTarget={selectedTraceTarget}
              resolvedTraces={resolvedTraces}
              traceLoading={traceLoading}
              traceError={traceError}
              sourceExcerpt={sourceExcerpt}
              excerptError={excerptError}
              highlightedEvidenceId={highlightedEvidenceId}
              currentSourceEvidenceId={currentSourceEvidenceId}
              groundedAnswer={groundedAnswer}
              groundedAnswerLoading={groundedAnswerLoading}
              groundedAnswerError={groundedAnswerError}
              onSelectTraceTarget={handleSelectTraceTarget}
              onClearTraceTarget={handleClearTraceTarget}
              onViewSource={handleViewSource}
              onCloseSourceExcerpt={handleCloseSourceExcerpt}
              onLocateEvidence={handleLocateEvidence}
              onEvidenceSelect={handleEvidenceSelect}
              onAskGroundedQuestion={handleAskGroundedQuestion}
              onGroundedCitationClick={handleGroundedCitationClick}
            />
          )}

          {state.phase === 'stage_error' && (
            <div style={{ padding: 24, background: '#fff3e0', borderRadius: 8 }}>
              <h3 style={{ margin: '0 0 8px' }}>阶段加载失败</h3>
              <p style={{ margin: 0 }}>{state.error.message}</p>
            </div>
          )}

          {!['selecting_stage', 'stage_loaded', 'stage_error', 'collecting_evidence', 'evidence_loaded', 'evidence_error', 'understanding_loading', 'understanding_loaded', 'understanding_error', 'views_loading', 'views_loaded', 'views_error'].includes(state.phase) &&
            !currentProfile && (
              <div
                style={{
                  display: 'flex',
                  alignItems: 'center',
                  justifyContent: 'center',
                  height: '100%',
                  color: '#999',
                }}
              >
                <p>请从左侧选择一个阶段查看详情</p>
              </div>
            )}
        </main>
      </div>

      {/* 底部 warnings */}
      {currentWarnings.length > 0 && (
        <footer
          style={{
            maxHeight: 200,
            overflowY: 'auto',
            background: '#fff8e1',
            borderTop: '1px solid #ddd',
            padding: '8px 24px',
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 14 }}>
            警告 ({currentWarnings.length})
          </h4>
          <ul style={{ margin: 0, paddingLeft: 20, fontSize: 13 }}>
            {currentWarnings.map((w, i) => (
              <li key={i} style={{ marginBottom: 4 }}>
                <code>{w.error_code}</code>: {w.message}
                {w.source_path && (
                  <span style={{ color: '#666' }}> ({w.source_path})</span>
                )}
              </li>
            ))}
          </ul>
        </footer>
      )}
    </div>
  );
}
