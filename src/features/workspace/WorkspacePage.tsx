import { useCallback, useMemo, useState, useRef, useEffect } from 'react';
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
  SessionState,
  LoadSessionStatus,
  QaHistory,
  SessionSummary,
  PersistedUiState,
  QualityReport,
} from '../../types/workspace';
import {
  openWorkspace,
  selectStage,
  collectEvidence,
  generateUnderstanding,
  generateViews,
  generateQualityReport,
  resolveTraceTarget,
  getSourceExcerpt,
  askGroundedQuestion,
  saveSession,
  loadSession,
  listSessions,
  deleteSession,
  getLastSession,
  CommandError,
} from '../../lib/tauriCommands';
import type { CommandError as CommandErrorType } from '../../types/workspace';
import type { UiError } from './workspaceUiTypes';

import ErrorPanel from './components/ErrorPanel';
import WorkspaceSummary from './components/WorkspaceSummary';
import StageList from './components/StageList';
import StageDetail from './components/StageDetail';
import RecentProjectsPanel from './components/RecentProjectsPanel';
import LoadStatusBanner from './components/LoadStatusBanner';
import AppShell from './components/AppShell';
import AppHeader from './components/AppHeader';
import LeftNav from './components/LeftNav';
import StageWorkspace from './components/StageWorkspace';
import type { ContextSelection } from './components/contextPanelTypes';

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

  // Phase 6 Session 状态
  const [sessionId, setSessionId] = useState<string | null>(null);
  const [saveStatus, setSaveStatus] = useState<'unsaved' | 'saving' | 'saved' | 'error'>('unsaved');
  const [saveError, setSaveError] = useState<UiError | null>(null);
  const [lastSavedAt, setLastSavedAt] = useState<string | null>(null);
  const [isLoadingSession, setIsLoadingSession] = useState(false);
  const [loadStatus, setLoadStatus] = useState<LoadSessionStatus | null>(null);
  const [loadError, setLoadError] = useState<UiError | null>(null);

  // Phase 7 Quality Review 状态
  const [qualityReport, setQualityReport] = useState<QualityReport | null>(null);
  const [qualityLoading, setQualityLoading] = useState(false);
  const [qualityError, setQualityError] = useState<UiError | null>(null);

  // Phase 8 Batch C 右侧 ContextPanel 前端局部选中态（不持久化）
  const [contextSelection, setContextSelection] = useState<ContextSelection | null>(null);

  // Phase 6 跨阶段累积映射（用于构造 SessionState）
  const [stageContextsMap, setStageContextsMap] = useState<Record<string, StageContext>>({});
  const [evidenceCollectionsMap, setEvidenceCollectionsMap] = useState<Record<string, EvidenceCollection>>({});
  const [understandingsMap, setUnderstandingsMap] = useState<Record<string, ImplementationUnderstanding>>({});
  const [viewGraphsMap, setViewGraphsMap] = useState<Record<string, ViewGraph[]>>({});
  const [qaHistoriesMap, setQaHistoriesMap] = useState<Record<string, QaHistory>>({});
  const [uiStatesMap, setUiStatesMap] = useState<Record<string, PersistedUiState>>({});

  // Phase 6 最近项目列表
  const [sessions, setSessions] = useState<SessionSummary[]>([]);
  const [sessionsLoading, setSessionsLoading] = useState(false);
  const [loadingSessionId, setLoadingSessionId] = useState<string | null>(null);

  // 用于取消旧 trace/excerpt/qa/quality 请求的守卫
  const traceGuardRef = useRef<number>(0);
  const excerptGuardRef = useRef<number>(0);
  const qaGuardRef = useRef<number>(0);
  const qualityGuardRef = useRef<number>(0);
  const highlightTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const autoSaveTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);
  const dirtyVersionRef = useRef<number>(0);

  // 统一清理自动保存定时器
  const clearAutoSaveTimer = useCallback(() => {
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
      autoSaveTimerRef.current = null;
    }
  }, []);

  // 标记状态已脏，递增 dirty version 并清理旧定时器
  const markUnsaved = useCallback(() => {
    dirtyVersionRef.current += 1;
    setSaveStatus('unsaved');
    clearAutoSaveTimer();
  }, [clearAutoSaveTimer]);

  // 使所有 pending 保存请求失效（打开/加载/删除 session 时调用）
  const invalidatePendingSessionSave = useCallback(() => {
    clearAutoSaveTimer();
    dirtyVersionRef.current += 1;
    setSaveError(null);
  }, [clearAutoSaveTimer]);

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
    setContextSelection(null);
    if (highlightTimerRef.current) {
      clearTimeout(highlightTimerRef.current);
      highlightTimerRef.current = null;
    }
  }, []);

  // ─── 清空 quality 相关状态 ───
  const clearQualityState = useCallback(() => {
    qualityGuardRef.current += 1;
    setQualityReport(null);
    setQualityLoading(false);
    setQualityError(null);
  }, []);

  // ─── 打开项目 ───
  const handleOpen = useCallback(async () => {
    const path = pathInput.trim();
    if (!path) return;
    invalidatePendingSessionSave();
    clearTraceState();
    clearQualityState();
    setSessionId(null);
    setSaveStatus('unsaved');
    setSaveError(null);
    setLastSavedAt(null);
    setLoadStatus(null);
    setLoadError(null);
    setStageContextsMap({});
    setEvidenceCollectionsMap({});
    setUnderstandingsMap({});
    setViewGraphsMap({});
    setQaHistoriesMap({});
    setUiStatesMap({});
    setState({ phase: 'opening' });
    try {
      const profile = await openWorkspace(path);
      setState({ phase: 'loaded', profile });
    } catch (err) {
      setState({ phase: 'error', error: makeUiError(err) });
    }
  }, [pathInput, clearTraceState, clearQualityState, invalidatePendingSessionSave]);

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
      if (!profile || isLoadingSession) return;
      clearTraceState();
      clearQualityState();
      markUnsaved();
      setState({ phase: 'selecting_stage', profile, stageId });
      try {
        const context = await selectStage(profile.root_path, stageId);
        setStageContextsMap((prev) => ({ ...prev, [stageId]: context }));
        setState({ phase: 'stage_loaded', profile, stageId, context });
      } catch (err) {
        setState({ phase: 'stage_error', profile, stageId, error: makeUiError(err) });
      }
    },
    [state, clearTraceState, clearQualityState, isLoadingSession]
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
    if (isLoadingSession) return;
    const { profile, stageId, context } = state as {
      profile: WorkspaceProfile;
      stageId: string;
      context: StageContext;
    };
    // 进入 collecting_evidence 时自动清除旧 understanding / views / trace / QA / quality
    clearTraceState();
    clearQualityState();
    markUnsaved();
    // 清除 maps 中当前阶段的 stale downstream data
    setUnderstandingsMap((prev) => { const { [stageId]: _, ...rest } = prev; return rest; });
    setViewGraphsMap((prev) => { const { [stageId]: _, ...rest } = prev; return rest; });
    setQaHistoriesMap((prev) => { const { [stageId]: _, ...rest } = prev; return rest; });
    setState({ phase: 'collecting_evidence', profile, stageId, context });
    try {
      const evidence = await collectEvidence(profile.root_path, stageId);
      setEvidenceCollectionsMap((prev) => ({ ...prev, [stageId]: evidence }));
      setState({ phase: 'evidence_loaded', profile, stageId, context, evidence });
    } catch (err) {
      setState({ phase: 'evidence_error', profile, stageId, context, error: makeUiError(err) });
    }
  }, [state, clearTraceState, clearQualityState, isLoadingSession]);

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
    if (isLoadingSession) return;
    const { profile, stageId, context } = state as {
      profile: WorkspaceProfile;
      stageId: string;
      context: StageContext;
    };
    const evidence =
      'evidence' in state ? (state as { evidence?: EvidenceCollection }).evidence : undefined;
    clearTraceState();
    clearQualityState();
    markUnsaved();
    // 清除当前阶段的 stale views / QA（基于旧 understanding 的产物应失效）
    setViewGraphsMap((prev) => { const { [stageId]: _, ...rest } = prev; return rest; });
    setQaHistoriesMap((prev) => { const { [stageId]: _, ...rest } = prev; return rest; });
    setState({ phase: 'understanding_loading', profile, stageId, context, evidence });
    try {
      const understanding = await generateUnderstanding(profile.root_path, stageId);
      setUnderstandingsMap((prev) => ({ ...prev, [stageId]: understanding }));
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
  }, [state, clearTraceState, clearQualityState, isLoadingSession]);

  // ─── 生成视图 ───
  const handleGenerateViews = useCallback(async () => {
    if (
      state.phase !== 'understanding_loaded' &&
      state.phase !== 'views_loaded' &&
      state.phase !== 'views_error'
    ) return;
    if (isLoadingSession) return;
    const { profile, stageId, context, evidence, understanding } = state as {
      profile: WorkspaceProfile;
      stageId: string;
      context: StageContext;
      evidence?: EvidenceCollection;
      understanding: ImplementationUnderstanding;
    };
    clearTraceState();
    clearQualityState();
    markUnsaved();
    // 清除当前阶段的 stale QA（基于旧视图的 Q&A 应失效）
    setQaHistoriesMap((prev) => { const { [stageId]: _, ...rest } = prev; return rest; });
    setState({ phase: 'views_loading', profile, stageId, context, evidence, understanding });
    try {
      const views = await generateViews(understanding);
      setViewGraphsMap((prev) => ({ ...prev, [stageId]: views }));
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
  }, [state, clearTraceState, clearQualityState, isLoadingSession]);

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

  // ─── Phase 7: 生成质量评估报告 ───
  const handleGenerateQualityReport = useCallback(async () => {
    const stageId = selectedStageId;
    const profile = currentProfile;
    if (!profile || !stageId) return;
    if (
      state.phase !== 'stage_loaded' &&
      state.phase !== 'evidence_loaded' &&
      state.phase !== 'evidence_error' &&
      state.phase !== 'understanding_loaded' &&
      state.phase !== 'understanding_error' &&
      state.phase !== 'views_loaded' &&
      state.phase !== 'views_error'
    ) return;
    if (isLoadingSession) return;

    const { context } = state as { context: StageContext };
    const recognizedStatus =
      profile.stages.find((s) => s.stage_id === stageId)?.status ?? 'available';
    const evidence =
      'evidence' in state
        ? (state as { evidence?: EvidenceCollection }).evidence
        : undefined;
    const understanding =
      'understanding' in state
        ? (state as { understanding?: ImplementationUnderstanding }).understanding
        : undefined;
    const views =
      'views' in state ? (state as { views?: ViewGraph[] }).views : undefined;

    const guard = (qualityGuardRef.current += 1);
    setContextSelection(null);
    setQualityReport(null);
    setQualityError(null);
    setQualityLoading(true);
    try {
      const report = await generateQualityReport(
        context,
        recognizedStatus,
        evidence,
        understanding,
        views,
        groundedAnswer ?? undefined
      );
      if (guard === qualityGuardRef.current) {
        setQualityReport(report);
        setQualityLoading(false);
      }
    } catch (err) {
      if (guard === qualityGuardRef.current) {
        setQualityError(makeUiError(err));
        setQualityLoading(false);
      }
    }
  }, [state, currentProfile, selectedStageId, isLoadingSession, groundedAnswer]);

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

  // Phase 7 质量报告是否处于忙碌或无可评估产物
  const hasQualityEvaluableArtifact =
    evidenceState?.evidence != null ||
    evidenceState?.understanding != null ||
    evidenceState?.views != null ||
    groundedAnswer != null;

  const canGenerateQualityReport =
    !isLoadingSession &&
    !isLoadingStage &&
    !groundedAnswerLoading &&
    !qualityLoading &&
    evidenceState != null &&
    hasQualityEvaluableArtifact;

  const qualityDisabledReason = (() => {
    if (isLoadingSession) return '正在加载会话';
    if (isLoadingStage) return '阶段加载中，请稍候';
    if (groundedAnswerLoading) return 'Q\u{26}A 生成中，请稍候';
    if (qualityLoading) return '质量报告生成中';
    if (evidenceState == null) return '请先选择一个阶段';
    if (!hasQualityEvaluableArtifact) return '请先收集证据或生成理解/视图';
    return undefined;
  })();

  // ─── Phase 6: 刷新最近项目列表 ───
  const refreshSessions = useCallback(async () => {
    setSessionsLoading(true);
    try {
      const list = await listSessions();
      setSessions(list);
    } catch {
      // 列表加载失败不阻断主流程
    } finally {
      setSessionsLoading(false);
    }
  }, []);

  // ─── Phase 6: 删除最近项目记录 ───
  const handleDeleteSession = useCallback(
    async (targetSessionId: string) => {
      try {
        if (targetSessionId === sessionId) {
          invalidatePendingSessionSave();
          setSessionId(null);
          setSaveStatus('unsaved');
          setLastSavedAt(null);
        }
        await deleteSession(targetSessionId);
        await refreshSessions();
      } catch (err) {
        // 删除失败不阻断主流程；列表仍显示，可再次操作
        setLoadError(makeUiError(err));
      }
    },
    [sessionId, refreshSessions, invalidatePendingSessionSave]
  );

  // ─── Phase 6: 构造 SessionState ───
  const buildSessionState = useCallback((): SessionState | null => {
    if (!currentProfile) return null;
    const stageId = selectedStageId ?? undefined;

    let uiStates = uiStatesMap;

    if (stageId) {
      const currentStageId = stageId;
      const uiStateForCurrentStage: PersistedUiState = {
        stage_id: currentStageId,
        selected_trace_target: selectedTraceTarget ?? undefined,
        resolved_traces: resolvedTraces,
        current_source_excerpt: sourceExcerpt ?? undefined,
        highlighted_evidence_id: highlightedEvidenceId ?? undefined,
      };
      uiStates = { ...uiStatesMap, [currentStageId]: uiStateForCurrentStage };
    }

    return {
      workspace_profile: currentProfile,
      selected_stage_id: stageId,
      stage_contexts: stageContextsMap,
      evidence_collections: evidenceCollectionsMap,
      understandings: understandingsMap,
      view_graphs: viewGraphsMap,
      qa_histories: qaHistoriesMap,
      ui_states: uiStates,
      global_ui_state: {
        last_session_id: sessionId ?? undefined,
        last_root_path: currentProfile.root_path,
      },
    };
  }, [
    currentProfile,
    selectedStageId,
    stageContextsMap,
    evidenceCollectionsMap,
    understandingsMap,
    viewGraphsMap,
    qaHistoriesMap,
    uiStatesMap,
    selectedTraceTarget,
    resolvedTraces,
    sourceExcerpt,
    highlightedEvidenceId,
    sessionId,
  ]);

  // ─── Phase 6: 保存 session ───
  const handleSaveSession = useCallback(async () => {
    const sessionState = buildSessionState();
    if (!sessionState || isLoadingSession || saveStatus === 'saving') return;
    const startedAtVersion = dirtyVersionRef.current;
    setSaveStatus('saving');
    setSaveError(null);
    try {
      const result = await saveSession(sessionState, sessionId ?? undefined);
      if (dirtyVersionRef.current === startedAtVersion) {
        setSessionId(result.session_id);
        setSaveStatus('saved');
        setLastSavedAt(result.saved_at);
      }
      await refreshSessions();
    } catch (err) {
      // 若保存期间 dirty version 已增加，说明状态已过期，避免覆盖更新的错误/未保存状态
      if (dirtyVersionRef.current === startedAtVersion) {
        setSaveStatus('error');
        setSaveError(makeUiError(err));
      }
    }
  }, [buildSessionState, sessionId, isLoadingSession, saveStatus, refreshSessions]);

  // ─── Phase 6: 加载并恢复 session ───
  const handleLoadSession = useCallback(
    async (targetSessionId: string) => {
      if (isLoadingSession || isLoadingStage) return;
      invalidatePendingSessionSave();
      setLoadingSessionId(targetSessionId);
      setIsLoadingSession(true);
      setLoadStatus(null);
      setLoadError(null);
      clearTraceState();
      clearQualityState();
      try {
        const result = await loadSession(targetSessionId);
        const restored = result.session_state;

        setStageContextsMap(restored.stage_contexts);
        setEvidenceCollectionsMap(restored.evidence_collections);
        setUnderstandingsMap(restored.understandings);
        setViewGraphsMap(restored.view_graphs);
        setQaHistoriesMap(restored.qa_histories);
        setUiStatesMap(restored.ui_states);

        setSessionId(targetSessionId);
        setPathInput(restored.workspace_profile.root_path);
        dirtyVersionRef.current = 0;
        setSaveStatus('saved');
        setSaveError(null);
        setLoadStatus(result.status);

        const stageId = restored.selected_stage_id;
        if (stageId && restored.stage_contexts[stageId]) {
          const context = restored.stage_contexts[stageId];
          const evidence = restored.evidence_collections[stageId];
          const understanding = restored.understandings[stageId];
          const views = restored.view_graphs[stageId];

          if (views) {
            setState({
              phase: 'views_loaded',
              profile: restored.workspace_profile,
              stageId,
              context,
              evidence,
              understanding,
              views,
            });
          } else if (understanding) {
            setState({
              phase: 'understanding_loaded',
              profile: restored.workspace_profile,
              stageId,
              context,
              evidence,
              understanding,
            });
          } else if (evidence) {
            setState({
              phase: 'evidence_loaded',
              profile: restored.workspace_profile,
              stageId,
              context,
              evidence,
            });
          } else {
            setState({
              phase: 'stage_loaded',
              profile: restored.workspace_profile,
              stageId,
              context,
            });
          }

          const uiState = restored.ui_states?.[stageId];
          if (uiState) {
            setSelectedTraceTarget(uiState.selected_trace_target ?? null);
            setResolvedTraces(uiState.resolved_traces);
            setSourceExcerpt(uiState.current_source_excerpt ?? null);
            setHighlightedEvidenceId(uiState.highlighted_evidence_id ?? null);
          } else {
            setSelectedTraceTarget(null);
            setResolvedTraces([]);
            setSourceExcerpt(null);
            setHighlightedEvidenceId(null);
          }

          const qaHistory = restored.qa_histories?.[stageId];
          if (qaHistory && qaHistory.entries.length > 0) {
            setGroundedAnswer(qaHistory.entries[qaHistory.entries.length - 1].answer);
          } else {
            setGroundedAnswer(null);
          }
        } else {
          setState({ phase: 'loaded', profile: restored.workspace_profile });
          setSelectedTraceTarget(null);
          setResolvedTraces([]);
          setSourceExcerpt(null);
          setHighlightedEvidenceId(null);
          setGroundedAnswer(null);
        }
      } catch (err) {
        setLoadError(makeUiError(err));
        setSaveStatus('unsaved');
      } finally {
        setIsLoadingSession(false);
        setLoadingSessionId(null);
      }
    },
    [isLoadingSession, isLoadingStage, clearTraceState, invalidatePendingSessionSave]
  );

  // ─── Phase 6: 初始加载最近项目列表与最后一次路径 ───
  useEffect(() => {
    refreshSessions();
    getLastSession()
      .then((session) => {
        if (session) setPathInput(session.root_path);
      })
      .catch(() => {});
  }, [refreshSessions]);

  // ─── Phase 6: 轻量自动保存 ───
  useEffect(() => {
    if (!currentProfile) return;
    if (isLoadingStage || isLoadingSession) return;
    if (saveStatus !== 'unsaved') return;
    if (autoSaveTimerRef.current) {
      clearTimeout(autoSaveTimerRef.current);
    }
    autoSaveTimerRef.current = setTimeout(() => {
      handleSaveSession();
    }, 2000);
    return () => {
      if (autoSaveTimerRef.current) {
        clearTimeout(autoSaveTimerRef.current);
      }
    };
  }, [
    currentProfile,
    selectedStageId,
    state.phase,
    evidenceCollectionsMap,
    understandingsMap,
    viewGraphsMap,
    qaHistoriesMap,
    uiStatesMap,
    selectedTraceTarget,
    resolvedTraces,
    sourceExcerpt,
    highlightedEvidenceId,
    groundedAnswer,
    isLoadingStage,
    isLoadingSession,
    saveStatus,
    handleSaveSession,
  ]);

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

      const stageId = selectedStageId;
      if (stageId) {
        setContextSelection({
          kind: 'trace_target',
          stageId,
          payload: { kind: 'trace_target', target, resolvedTraces: [] },
        });
      }

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
          if (stageId) {
            setContextSelection((prev) =>
              prev?.kind === 'trace_target' && prev.stageId === stageId
                ? { ...prev, payload: { ...prev.payload, resolvedTraces: traces } }
                : prev
            );
          }
          markUnsaved();
          setTraceLoading(false);
        }
      } catch (err) {
        if (guard === traceGuardRef.current) {
          setTraceError(makeUiError(err));
          setTraceLoading(false);
        }
      }
    },
    [state, selectedStageId]
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
    setContextSelection(null);
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
          if (currentStageIdForTrace) {
            setContextSelection({
              kind: 'source_excerpt',
              stageId: currentStageIdForTrace,
              payload: { kind: 'source_excerpt', excerpt },
            });
          }
          markUnsaved();
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
    markUnsaved();
    if (highlightTimerRef.current) {
      clearTimeout(highlightTimerRef.current);
    }
    highlightTimerRef.current = setTimeout(() => {
      setHighlightedEvidenceId((current) =>
        current === evidenceId ? null : current
      );
    }, 3000);
  }, [markUnsaved]);

  // ─── evidence 卡片点击 ───
  const handleEvidenceSelect = useCallback((evidenceId: string) => {
    setHighlightedEvidenceId(evidenceId);
    markUnsaved();
    if (highlightTimerRef.current) {
      clearTimeout(highlightTimerRef.current);
    }
    highlightTimerRef.current = setTimeout(() => {
      setHighlightedEvidenceId((current) =>
        current === evidenceId ? null : current
      );
    }, 3000);
  }, [markUnsaved]);

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
      clearQualityState();
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
          const selectedTargetKind = selectedTraceTarget?.kind;
          const newEntry: import('../../types/workspace').QaHistoryEntry = {
            entry_id: `qa-${Date.now()}`,
            timestamp: new Date().toISOString(),
            question: questionText,
            answer,
            selected_target_kind: selectedTargetKind,
          };
          setQaHistoriesMap((prev) => {
            const existing = prev[stageId];
            const history: import('../../types/workspace').QaHistory = existing
              ? { ...existing, entries: [...existing.entries, newEntry] }
              : { stage_id: stageId, entries: [newEntry], version: '1.0.0' };
            return { ...prev, [stageId]: history };
          });
          setGroundedAnswer(answer);
          markUnsaved();
          setGroundedAnswerLoading(false);
        }
      } catch (err) {
        if (guard === qaGuardRef.current) {
          setGroundedAnswerError(makeUiError(err));
          setGroundedAnswerLoading(false);
        }
      }
    },
    [currentProfile, selectedStageId, state, selectedTraceTarget, resolvedTraces, clearQualityState]
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
      if (selectedStageId) {
        setContextSelection({
          kind: 'qa_citation',
          stageId: selectedStageId,
          payload: { kind: 'qa_citation', citation },
        });
      }
    },
    [handleViewSource, selectedStageId]
  );

  // ─── 渲染 ───
  const header = (
    <AppHeader
      pathInput={pathInput}
      setPathInput={setPathInput}
      onOpen={handleOpen}
      isOpening={state.phase === 'opening'}
      isLoadingSession={isLoadingSession}
      saveStatus={saveStatus}
      saveError={saveError}
      lastSavedAt={lastSavedAt}
      onSave={handleSaveSession}
    />
  );

  const leftNav = (
    <LeftNav
      projectInfo={
        state.phase === 'initial' ? (
          <div style={{ color: '#94a3b8', textAlign: 'center', marginTop: 40 }}>
            <p>请输入项目路径并点击"打开项目"</p>
          </div>
        ) : state.phase === 'opening' ? (
          <div style={{ color: '#94a3b8', textAlign: 'center', marginTop: 40 }}>
            <p>正在扫描 workspace...</p>
          </div>
        ) : state.phase === 'error' ? (
          <ErrorPanel error={state.error} />
        ) : currentProfile ? (
          <WorkspaceSummary profile={currentProfile} />
        ) : null
      }
      stageList={
        currentProfile ? (
          <StageList
            stages={currentProfile.stages}
            selectedStageId={selectedStageId}
            isLoading={isLoadingStage || isLoadingSession}
            onSelect={handleSelectStage}
          />
        ) : null
      }
      recentProjects={
        <RecentProjectsPanel
          sessions={sessions}
          loading={sessionsLoading}
          disabled={isLoadingSession || isLoadingStage}
          loadingSessionId={loadingSessionId}
          onLoad={handleLoadSession}
          onDelete={handleDeleteSession}
          onOpenOtherProject={() => {
            const input = document.getElementById('workspace-path-input') as HTMLInputElement | null;
            input?.focus();
          }}
        />
      }
      loadError={
        loadError ? (
          <div
            style={{
              padding: 12,
              background: 'rgba(239, 68, 68, 0.15)',
              borderRadius: 8,
              border: '1px solid rgba(239, 68, 68, 0.3)',
            }}
          >
            <h4 style={{ margin: '0 0 8px', fontSize: 14, color: '#fca5a5' }}>加载失败</h4>
            <p style={{ margin: '0 0 4px', fontSize: 13, color: '#e2e8f0' }}>{loadError.message}</p>
            {'error_code' in loadError && (
              <code style={{ fontSize: 12, color: '#94a3b8' }}>{loadError.error_code}</code>
            )}
          </div>
        ) : null
      }
    />
  );

  const main = (
    <main
      style={{
        flex: 1,
        display: 'flex',
        flexDirection: 'column',
        overflow: 'hidden',
        background: '#f8fafc',
      }}
    >
      {loadStatus && (
        <LoadStatusBanner
          status={loadStatus}
          onClose={() => setLoadStatus(null)}
          onReanalyze={() => {
            setLoadStatus(null);
            if (selectedStageId) {
              handleCollectEvidence();
            }
          }}
          onDelete={() => {
            if (sessionId) handleDeleteSession(sessionId);
          }}
        />
      )}

      {state.phase === 'initial' && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flex: 1,
            color: '#94a3b8',
          }}
        >
          <p>请从左侧打开一个项目</p>
        </div>
      )}

      {state.phase === 'selecting_stage' && currentProfile && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flex: 1,
            color: '#64748b',
          }}
        >
          <p>正在加载阶段详情：{state.stageId}</p>
        </div>
      )}

      {evidenceState && currentProfile && selectedStageId && (
        <StageWorkspace
          profile={currentProfile}
          stageId={selectedStageId}
          context={evidenceState.context}
          stageStatus={
            currentProfile.stages.find((s) => s.stage_id === selectedStageId)?.status ??
            'available'
          }
          evidence={'evidence' in evidenceState ? evidenceState.evidence : undefined}
          evidenceLoading={'isCollecting' in evidenceState ? evidenceState.isCollecting : undefined}
          understanding={'understanding' in evidenceState ? evidenceState.understanding : undefined}
          understandingLoading={
            'understandingLoading' in evidenceState ? evidenceState.understandingLoading : undefined
          }
          views={'views' in evidenceState ? evidenceState.views : undefined}
          viewsLoading={'viewsLoading' in evidenceState ? evidenceState.viewsLoading : undefined}
          qaHistory={qaHistoriesMap[selectedStageId]}
          qaLoading={groundedAnswerLoading}
          qualityReport={qualityReport}
          qualityLoading={qualityLoading}
          contextSelection={contextSelection}
          onContextSelectionChange={setContextSelection}
          onViewSource={handleViewSource}
          onLocateEvidence={handleLocateEvidence}
          renderContent={({ activeTab, evidenceFilter, qualityFilter, onContextSelectionChange }) => (
            <StageDetail
              activeTab={activeTab}
              stageId={selectedStageId}
              context={evidenceState.context}
              evidence={'evidence' in evidenceState ? evidenceState.evidence : undefined}
              evidenceError={'evidenceError' in evidenceState ? evidenceState.evidenceError : undefined}
              isCollecting={'isCollecting' in evidenceState ? evidenceState.isCollecting : undefined}
              onCollectEvidence={handleCollectEvidence}
              understanding={'understanding' in evidenceState ? evidenceState.understanding : undefined}
              understandingLoading={
                'understandingLoading' in evidenceState ? evidenceState.understandingLoading : undefined
              }
              understandingError={
                'understandingError' in evidenceState ? evidenceState.understandingError : undefined
              }
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
              onContextSelectionChange={onContextSelectionChange}
              qualityReport={qualityReport}
              qualityLoading={qualityLoading}
              qualityError={qualityError}
              canGenerateQualityReport={canGenerateQualityReport}
              qualityDisabledReason={qualityDisabledReason}
              onGenerateQualityReport={handleGenerateQualityReport}
              evidenceFilter={evidenceFilter}
              qualityFilter={qualityFilter}
            />
          )}
        />
      )}

      {state.phase === 'stage_error' && currentProfile && (
        <div
          style={{
            display: 'flex',
            alignItems: 'center',
            justifyContent: 'center',
            flex: 1,
            padding: 24,
          }}
        >
          <div style={{ padding: 24, background: '#fff3e0', borderRadius: 8, maxWidth: 720 }}>
            <h3 style={{ margin: '0 0 8px' }}>阶段加载失败</h3>
            <p style={{ margin: 0 }}>{state.error.message}</p>
          </div>
        </div>
      )}

      {!['selecting_stage', 'stage_loaded', 'stage_error', 'collecting_evidence', 'evidence_loaded', 'evidence_error', 'understanding_loading', 'understanding_loaded', 'understanding_error', 'views_loading', 'views_loaded', 'views_error'].includes(state.phase) &&
        !currentProfile && (
          <div
            style={{
              display: 'flex',
              alignItems: 'center',
              justifyContent: 'center',
              flex: 1,
              color: '#94a3b8',
            }}
          >
            <p>请从左侧打开一个项目</p>
          </div>
        )}
    </main>
  );

  const footer = currentWarnings.length > 0 ? (
    <footer
      style={{
        maxHeight: 200,
        overflowY: 'auto',
        background: '#fff8e1',
        borderTop: '1px solid #e2e8f0',
        padding: '8px 24px',
        flexShrink: 0,
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
              <span style={{ color: '#64748b' }}> ({w.source_path})</span>
            )}
          </li>
        ))}
      </ul>
    </footer>
  ) : null;

  return (
    <AppShell
      header={header}
      leftNav={leftNav}
      main={main}
      footer={footer}
    />
  );
}
