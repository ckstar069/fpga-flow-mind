import { useState, useMemo } from 'react';
import type {
  ViewGraph,
  ViewEdge,
  ViewType,
  NodeType,
  EdgeType,
  SelectedTraceTarget,
} from '../../../types/workspace';
import type { UiError } from '../workspaceUiTypes';
import type { ContextSelection } from './contextPanelTypes';
import { SURFACE, ACCENT, FONT } from './workbenchTheme';

// ─── 中文标签映射 ───────────────────────────────────────────────────────

const VIEW_TYPE_LABEL: Record<ViewType, string> = {
  structure: '结构图',
  dataflow: '数据流',
  timing: '时序流水',
};

const NODE_TYPE_LABEL: Record<NodeType, string> = {
  module: '模块',
  function: '函数',
  interface: '接口',
  signal: '信号',
  processing_step: '处理步骤',
  class: '类',
  constant: '常量',
  input_source: '输入源',
  output_target: '输出目标',
  intermediate_data: '中间数据',
  pipeline_stage: '流水级',
  clock_domain: '时钟域',
  reset_domain: '复位域',
};

const EDGE_TYPE_LABEL: Record<EdgeType, string> = {
  contains: '包含',
  calls: '调用',
  references: '引用',
  depends_on: '依赖',
  data_flow: '数据流',
  sequential_order: '顺序',
  pipeline_forward: '流水前推',
  clock_driven: '时钟驱动',
};

const CONFIDENCE_LABEL: Record<string, string> = {
  confirmed: '已确认',
  supported: '有支撑',
  inferred: '推断',
  unknown: '未知',
  conflicting: '矛盾',
};

// ─── 节点颜色方案 ───────────────────────────────────────────────────────

interface NodeStyle {
  fill: string;
  stroke: string;
  textColor: string;
}

const NODE_STYLE: Record<string, NodeStyle> = {
  module: { fill: '#e3f2fd', stroke: '#1565c0', textColor: '#0d47a1' },
  function: { fill: '#e8f5e9', stroke: '#2e7d32', textColor: '#1b5e20' },
  interface: { fill: '#e0f7fa', stroke: '#00838f', textColor: '#006064' },
  signal: { fill: '#f5f5f5', stroke: '#757575', textColor: '#424242' },
  processing_step: { fill: '#bbdefb', stroke: '#1976d2', textColor: '#0d47a1' },
  class: { fill: '#e1f5fe', stroke: '#0288d1', textColor: '#01579b' },
  constant: { fill: '#eceff1', stroke: '#455a64', textColor: '#263238' },
  input_source: { fill: '#c8e6c9', stroke: '#388e3c', textColor: '#1b5e20' },
  output_target: { fill: '#ffe0b2', stroke: '#e65100', textColor: '#bf360c' },
  intermediate_data: { fill: '#eeeeee', stroke: '#9e9e9e', textColor: '#424242' },
  pipeline_stage: { fill: '#bbdefb', stroke: '#1976d2', textColor: '#0d47a1' },
  clock_domain: { fill: '#fff9c4', stroke: '#f9a825', textColor: '#795548' },
  reset_domain: { fill: '#ffebee', stroke: '#c62828', textColor: '#b71c1c' },
};

// ─── 置信度 → 边框样式 ──────────────────────────────────────────────────

function confidenceDashArray(confidence: string): string {
  switch (confidence) {
    case 'confirmed':
    case 'supported':
      return 'none';
    case 'inferred':
      return '6,4';
    case 'unknown':
      return '2,4';
    case 'conflicting':
      return 'none';
    default:
      return 'none';
  }
}

function confidenceStrokeWidth(confidence: string): number {
  switch (confidence) {
    case 'confirmed':
      return 2;
    case 'supported':
      return 2;
    case 'inferred':
      return 2;
    case 'unknown':
      return 1;
    case 'conflicting':
      return 2;
    default:
      return 1.5;
  }
}

function confidenceEdgeStroke(confidence: string): string {
  switch (confidence) {
    case 'confirmed':
      return '#333';
    case 'supported':
      return '#555';
    case 'inferred':
      return '#888';
    case 'unknown':
      return '#bbb';
    case 'conflicting':
      return '#c62828';
    default:
      return '#555';
  }
}

// ─── 组件 Props ─────────────────────────────────────────────────────────

interface MultiViewPanelProps {
  views: ViewGraph[];
  loading?: boolean;
  error?: UiError | string;
  stageId?: string;
  selectedTarget?: SelectedTraceTarget | null;
  onSelectTarget?: (target: SelectedTraceTarget) => void;
  onContextSelection?: (selection: ContextSelection) => void;
}

// ─── 主组件 ─────────────────────────────────────────────────────────────

export default function MultiViewPanel({
  views,
  loading,
  error,
  stageId,
  selectedTarget,
  onSelectTarget,
  onContextSelection,
}: MultiViewPanelProps) {
  const [selectedTab, setSelectedTab] = useState<ViewType>('structure');

  const currentGraph = useMemo(
    () => views.find((g) => g.view_type === selectedTab) ?? null,
    [views, selectedTab]
  );

  const isDegraded = views.some((g) => g.meta.is_degraded_source);

  // 切换 tab 时，如果当前 selectedTarget 不属于新 tab，保留但 ViewGraphRenderer 不显示高亮
  // 这里选择保留 selectedTarget，由上层在切换阶段/视图生成时清空

  // ─── Loading 状态 ───
  if (loading) {
    return (
      <div style={{ marginBottom: 24 }}>
        <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>视图生成</h3>
        <div
          style={{
            padding: 24,
            background: ACCENT.blueSoft,
            borderRadius: 8,
            textAlign: 'center',
            border: `1px solid ${ACCENT.blueSoftBorder}`,
          }}
        >
          <p style={{ margin: 0, color: ACCENT.blueDark, fontSize: FONT.body, fontWeight: 600 }}>
            正在生成视图...
          </p>
          <p style={{ margin: '8px 0 0', color: SURFACE.textMuted, fontSize: FONT.caption }}>
            正在从理解结果中生成结构图、数据流和时序流水视图
          </p>
        </div>
      </div>
    );
  }

  // ─── Error 状态 ───
  if (error) {
    const errMsg = typeof error === 'string' ? error : error.message;
    const errCode = typeof error === 'string' ? undefined : error.error_code;
    return (
      <div style={{ marginBottom: 24 }}>
        <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>视图生成</h3>
        <div
          style={{
            padding: 16,
            background: '#fce4ec',
            borderRadius: 8,
            border: '1px solid #ef9a9a',
          }}
        >
          <h4 style={{ margin: '0 0 8px', fontSize: 14, color: '#c62828' }}>视图生成失败</h4>
          <div style={{ fontSize: 13 }}>
            {errCode && (
              <div style={{ marginBottom: 4 }}>
                <span style={{ color: '#666' }}>错误码：</span>
                <code>{errCode}</code>
              </div>
            )}
            <div>
              <span style={{ color: '#666' }}>信息：</span>
              {errMsg}
            </div>
          </div>
        </div>
      </div>
    );
  }

  // ─── 正常渲染 ───
  return (
    <div style={{ marginBottom: 24 }}>
      <h3 style={{ fontSize: 15, margin: '0 0 12px' }}>视图生成</h3>

      {/* Degraded 横幅 */}
      {isDegraded && (
        <div
          style={{
            padding: '10px 14px',
            background: '#fff8e1',
            borderRadius: 6,
            border: '1px solid #ffe082',
            marginBottom: 12,
            fontSize: 13,
            color: '#795548',
          }}
        >
          当前为降级数据，视图内容有限
        </div>
      )}

      {/* Tab bar */}
      <div
        style={{
          display: 'flex',
          gap: 0,
          marginBottom: 16,
          borderBottom: '2px solid #e0e0e0',
        }}
      >
        {(['structure', 'dataflow', 'timing'] as ViewType[]).map((vt) => {
          const isSelected = selectedTab === vt;
          const graph = views.find((g) => g.view_type === vt);
          const isEmpty = graph && graph.nodes.length === 0 && graph.edges.length === 0;
          return (
            <button
              key={vt}
              onClick={() => setSelectedTab(vt)}
              style={{
                padding: '8px 20px',
                border: 'none',
                borderBottom: isSelected ? '2px solid #1976d2' : '2px solid transparent',
                background: 'transparent',
                color: isSelected ? '#1976d2' : '#666',
                fontWeight: isSelected ? 600 : 400,
                fontSize: 14,
                cursor: 'pointer',
                marginBottom: -2,
                transition: 'color 0.15s, border-color 0.15s',
              }}
            >
              {VIEW_TYPE_LABEL[vt]}
              {isEmpty && (
                <span style={{ fontSize: 11, color: '#999', marginLeft: 4 }}>(空)</span>
              )}
            </button>
          );
        })}
      </div>

      {/* 当前视图渲染 */}
      {currentGraph ? (
        <ViewGraphRenderer
          graph={currentGraph}
          stageId={stageId}
          selectedTarget={selectedTarget}
          onSelectTarget={onSelectTarget}
          onContextSelection={onContextSelection}
        />
      ) : (
        <div
          style={{
            padding: 32,
            background: '#fafafa',
            borderRadius: 8,
            textAlign: 'center',
            color: '#999',
          }}
        >
          <p style={{ margin: 0 }}>视图数据缺失</p>
        </div>
      )}
    </div>
  );
}

// ─── 视图渲染器 ─────────────────────────────────────────────────────────

function ViewGraphRenderer({
  graph,
  stageId,
  selectedTarget,
  onSelectTarget,
  onContextSelection,
}: {
  graph: ViewGraph;
  stageId?: string;
  selectedTarget?: SelectedTraceTarget | null;
  onSelectTarget?: (target: SelectedTraceTarget) => void;
  onContextSelection?: (selection: ContextSelection) => void;
}) {
  const { nodes, edges, meta } = graph;

  // 当前选中是否在当前 graph 内
  const selectedNodeId =
    selectedTarget?.kind === 'view_node' && selectedTarget.view_type === graph.view_type
      ? selectedTarget.node_id
      : null;
  const selectedEdgeId =
    selectedTarget?.kind === 'view_edge' && selectedTarget.view_type === graph.view_type
      ? selectedTarget.edge_id
      : null;

  const handleNodeClick = (nodeId: string) => {
    const target: SelectedTraceTarget = {
      kind: 'view_node',
      view_type: graph.view_type,
      node_id: nodeId,
    };
    onSelectTarget?.(target);
    if (stageId) {
      const node = nodes.find((n) => n.node_id === nodeId);
      if (node) {
        onContextSelection?.({
          kind: 'view_node',
          stageId,
          payload: { kind: 'view_node', viewType: graph.view_type, node },
        });
      }
    }
  };

  const handleEdgeClick = (edge: ViewEdge) => {
    const target: SelectedTraceTarget = {
      kind: 'view_edge',
      view_type: graph.view_type,
      edge_id: edge.edge_id,
    };
    onSelectTarget?.(target);
    if (stageId) {
      onContextSelection?.({
        kind: 'view_edge',
        stageId,
        payload: { kind: 'view_edge', viewType: graph.view_type, edge },
      });
    }
  };

  // 空状态
  if (nodes.length === 0 && edges.length === 0) {
    return (
      <div
        style={{
          padding: 32,
          background: '#fafafa',
          borderRadius: 8,
          textAlign: 'center',
          color: '#999',
        }}
      >
        <p style={{ margin: '0 0 8px', fontSize: 15 }}>
          {meta.empty_reason ?? '无足够数据生成视图'}
        </p>
        <p style={{ margin: 0, fontSize: 12, color: '#bbb' }}>
          该视图类型在当前阶段无可用数据
        </p>
      </div>
    );
  }

  // 构建 node_id → index 映射，用于边端点查找
  const nodeIndexMap = new Map<string, number>();
  nodes.forEach((n, i) => nodeIndexMap.set(n.node_id, i));

  // 计算节点位置：优先使用 layout hint，否则按索引 grid 排布
  const GRID_COLS = 4;
  const NODE_W = 180;
  const NODE_H = 56;
  const H_GAP = 40;
  const V_GAP = 30;
  const PADDING = 40;

  const positions = nodes.map((n, i) => {
    if (n.layout && n.layout.column != null && n.layout.row != null) {
      return {
        x: PADDING + n.layout.column * (NODE_W + H_GAP),
        y: PADDING + n.layout.row * (NODE_H + V_GAP),
      };
    }
    // 无 layout hint：按索引 grid 排布
    const col = i % GRID_COLS;
    const row = Math.floor(i / GRID_COLS);
    return {
      x: PADDING + col * (NODE_W + H_GAP),
      y: PADDING + row * (NODE_H + V_GAP),
    };
  });

  // 计算 SVG 尺寸
  const maxX = positions.reduce((m, p) => Math.max(m, p.x + NODE_W), 0) + PADDING;
  const maxY = positions.reduce((m, p) => Math.max(m, p.y + NODE_H), 0) + PADDING;
  const svgW = Math.max(400, maxX);
  const svgH = Math.max(200, maxY);

  // 过滤有效边（endpoint 存在）
  const validEdges: { edge: ViewEdge; fromIdx: number; toIdx: number }[] = [];
  const skippedEdges: ViewEdge[] = [];
  for (const e of edges) {
    const fromIdx = nodeIndexMap.get(e.source_node_id);
    const toIdx = nodeIndexMap.get(e.target_node_id);
    if (fromIdx != null && toIdx != null) {
      validEdges.push({ edge: e, fromIdx, toIdx });
    } else {
      skippedEdges.push(e);
    }
  }

  if (skippedEdges.length > 0) {
    console.warn(
      `[MultiViewPanel] ${skippedEdges.length} 条边的端点不存在，已跳过:`,
      skippedEdges.map((e) => e.edge_id)
    );
  }

  return (
    <div
      style={{
        border: '1px solid #e0e0e0',
        borderRadius: 8,
        background: '#fafafa',
        overflow: 'auto',
        maxHeight: 600,
      }}
    >
      <svg width={svgW} height={svgH} style={{ display: 'block' }}>
        {/* 定义箭头标记 */}
        <defs>
          <marker
            id="arrowhead"
            viewBox="0 0 10 7"
            refX="10"
            refY="3.5"
            markerWidth="8"
            markerHeight="6"
            orient="auto-start-reverse"
          >
            <polygon points="0 0, 10 3.5, 0 7" fill="#666" />
          </marker>
        </defs>

        {/* 边 */}
        {validEdges.map(({ edge, fromIdx, toIdx }) => {
          const from = positions[fromIdx];
          const to = positions[toIdx];
          const stroke = confidenceEdgeStroke(edge.confidence);
          const dash = confidenceDashArray(edge.confidence);
          const sw = confidenceStrokeWidth(edge.confidence);
          const isSelected = selectedEdgeId === edge.edge_id;

          // 边从节点右边缘中心 → 目标节点左边缘中心
          const x1 = from.x + NODE_W;
          const y1 = from.y + NODE_H / 2;
          const x2 = to.x;
          const y2 = to.y + NODE_H / 2;

          return (
            <g key={edge.edge_id} onClick={() => handleEdgeClick(edge)} style={{ cursor: 'pointer' }}>
              <line
                x1={x1}
                y1={y1}
                x2={x2}
                y2={y2}
                stroke={isSelected ? '#1565c0' : stroke}
                strokeWidth={isSelected ? sw + 2 : sw}
                strokeDasharray={dash === 'none' ? undefined : dash}
                markerEnd="url(#arrowhead)"
                style={{
                  filter: isSelected ? 'drop-shadow(0 0 3px #1976d2)' : undefined,
                }}
              />
              {/* 边标签（中点上方） */}
              <text
                x={(x1 + x2) / 2}
                y={(y1 + y2) / 2 - 6}
                textAnchor="middle"
                fontSize={10}
                fill="#888"
                style={{ pointerEvents: 'none' }}
              >
                {EDGE_TYPE_LABEL[edge.edge_type] ?? edge.edge_type}
              </text>
              {/* 透明宽点击区域用于 hover/click */}
              <line
                x1={x1}
                y1={y1}
                x2={x2}
                y2={y2}
                stroke="transparent"
                strokeWidth={12}
                style={{ cursor: 'pointer' }}
              >
                <title>
                  {edge.label ? `${edge.label}\n` : ''}
                  {edge.description}
                  {'\n'}类型: {EDGE_TYPE_LABEL[edge.edge_type] ?? edge.edge_type}
                  {'\n'}置信度: {CONFIDENCE_LABEL[edge.confidence] ?? edge.confidence}
                  {edge.trace_refs.length > 0
                    ? '\n\n证据追溯:\n' +
                      edge.trace_refs
                        .map(
                          (tr) =>
                            `  ${tr.claim_id ? `claim: ${tr.claim_id}` : ''}${tr.evidence_id ? ` evidence: ${tr.evidence_id}` : ''}${tr.relevance ? ` (${tr.relevance})` : ''}`
                        )
                        .join('\n')
                    : '\n\n无证据追溯'}
                </title>
              </line>
            </g>
          );
        })}

        {/* 节点 */}
        {nodes.map((n, i) => {
          const pos = positions[i];
          const style = NODE_STYLE[n.node_type] ?? NODE_STYLE.signal;
          const dash = confidenceDashArray(n.confidence);
          const sw = confidenceStrokeWidth(n.confidence);
          const isConflicting = n.confidence === 'conflicting';
          const strokeColor = isConflicting ? '#c62828' : style.stroke;

          // 截断长 label
          const maxLabelLen = 18;
          const displayLabel =
            n.label.length > maxLabelLen ? n.label.slice(0, maxLabelLen) + '…' : n.label;

          return (
            <g key={n.node_id} onClick={() => handleNodeClick(n.node_id)} style={{ cursor: 'pointer' }}>
              {/* 节点矩形 */}
              <rect
                x={pos.x}
                y={pos.y}
                width={NODE_W}
                height={NODE_H}
                rx={6}
                ry={6}
                fill={style.fill}
                stroke={selectedNodeId === n.node_id ? '#1565c0' : strokeColor}
                strokeWidth={selectedNodeId === n.node_id ? sw + 2 : sw}
                strokeDasharray={dash === 'none' ? undefined : dash}
                style={{
                  cursor: 'pointer',
                  filter: selectedNodeId === n.node_id ? 'drop-shadow(0 0 4px #1976d2)' : undefined,
                }}
              />
              {/* 标签 */}
              <text
                x={pos.x + NODE_W / 2}
                y={pos.y + 20}
                textAnchor="middle"
                fontSize={12}
                fontWeight={600}
                fill={style.textColor}
                style={{ pointerEvents: 'none' }}
              >
                {displayLabel}
              </text>
              {/* 类型 + 置信度 */}
              <text
                x={pos.x + NODE_W / 2}
                y={pos.y + 40}
                textAnchor="middle"
                fontSize={10}
                fill="#888"
                style={{ pointerEvents: 'none' }}
              >
                {NODE_TYPE_LABEL[n.node_type] ?? n.node_type}
                {' · '}
                {CONFIDENCE_LABEL[n.confidence] ?? n.confidence}
              </text>
              {/* title tooltip（原生 hover） */}
              <title>
                {n.label}
                {'\n'}类型: {NODE_TYPE_LABEL[n.node_type] ?? n.node_type}
                {'\n'}置信度: {CONFIDENCE_LABEL[n.confidence] ?? n.confidence}
                {'\n\n'}描述: {n.description}
                {n.trace_refs.length > 0
                  ? '\n\n证据追溯:\n' +
                    n.trace_refs
                      .map(
                        (tr) =>
                          `  ${tr.claim_id ? `claim: ${tr.claim_id}` : ''}${tr.evidence_id ? ` evidence: ${tr.evidence_id}` : ''}${tr.relevance ? ` (${tr.relevance})` : ''}`
                      )
                      .join('\n')
                  : '\n\n无证据追溯'}
              </title>
            </g>
          );
        })}
      </svg>

      {/* 跳过边警告（轻量） */}
      {skippedEdges.length > 0 && (
        <div
          style={{
            padding: '6px 12px',
            background: '#fff8e1',
            borderTop: '1px solid #ffe082',
            fontSize: 11,
            color: '#795548',
          }}
        >
          ⚠ {skippedEdges.length} 条边的端点节点不存在，已跳过渲染
        </div>
      )}
    </div>
  );
}
