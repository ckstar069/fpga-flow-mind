import type {
  EvidenceItem,
  SelectedTraceTarget,
  TraceRefResolved,
  SourceExcerpt,
  QualityIssue,
  ViewNode,
  ViewEdge,
  ViewType,
  GroundedAnswerCitation,
} from '../../../types/workspace';

/** ContextPanel 可展示的对象类型 */
export type ContextSelectionKind =
  | 'evidence'
  | 'trace_target'
  | 'source_excerpt'
  | 'quality_issue'
  | 'view_node'
  | 'view_edge'
  | 'qa_citation';

/**
 * 当前选中的上下文对象。
 * 注意：这是 Batch C 前端局部状态，不进入 PersistedUiState / Rust 持久化契约。
 * stageId 用于阶段作用域校验，切换阶段时必须清空。
 */
export interface ContextSelection {
  kind: ContextSelectionKind;
  stageId: string;
  payload:
    | { kind: 'evidence'; item: EvidenceItem }
    | { kind: 'trace_target'; target: SelectedTraceTarget; resolvedTraces: TraceRefResolved[] }
    | { kind: 'source_excerpt'; excerpt: SourceExcerpt }
    | { kind: 'quality_issue'; issue: QualityIssue }
    | { kind: 'view_node'; viewType: ViewType; node: ViewNode }
    | { kind: 'view_edge'; viewType: ViewType; edge: ViewEdge }
    | { kind: 'qa_citation'; citation: GroundedAnswerCitation };
}

export type ContextSelectionPayload<T extends ContextSelectionKind> = Extract<
  ContextSelection['payload'],
  { kind: T }
>;
