# Phase 2 Evidence View 前端设计

---
status: draft
updated: 2026-06-11
---

> 本文档设计 Phase 2 evidence view 的前端组件布局、状态管理、交互细节和前后端边界。
> 不写产品代码。设计约束来自 [`phase-2-evidence-requirements.md`](../requirements/phase-2-evidence-requirements.md) 和 [`phase-2-evidence-model.md`](../design/phase-2-evidence-model.md)。

## 1. 设计目标

在 Phase 1 的 `StageDetail` 组件基础上，新增"收集证据"入口和 evidence 面板，使用户能够：

- 触发证据收集
- 查看证据收集结果（列表、统计、筛选）
- 了解收集过程中的警告和不确定项
- 在进入 Phase 3 前审查证据质量

## 2. 前端新增组件

### 2.1 组件层级

```text
WorkspacePage (已有)
├── WorkspaceSummary (已有)
├── StageList (已有)
└── 右栏区域
    ├── StageDetail (已有，需改造)
    │   ├── 文件列表 (已有)
    │   ├── 外部依赖 (已有)
    │   ├── 上游引用 (已有)
    │   └── 新增：CollectEvidenceButton（收集证据按钮）
    └── EvidencePanel (Phase 2 新增)
        ├── EvidenceStatsBar（统计概要）
        ├── EvidenceFilterBar（筛选栏）
        ├── EvidenceItemList（证据项列表）
        └── EvidenceWarningList（警告列表，可折叠）
```

### 2.2 新增组件清单

| 组件 | 文件路径 | 职责 |
|------|----------|------|
| `CollectEvidenceButton` | `src/features/workspace/components/CollectEvidenceButton.tsx` | 收集证据按钮，状态管理 |
| `EvidencePanel` | `src/features/evidence/EvidencePanel.tsx` | 证据面板容器 |
| `EvidenceStatsBar` | `src/features/evidence/EvidenceStatsBar.tsx` | 统计概要条 |
| `EvidenceFilterBar` | `src/features/evidence/EvidenceFilterBar.tsx` | 筛选栏（按文件/类型/符号） |
| `EvidenceItemList` | `src/features/evidence/EvidenceItemList.tsx` | 证据项列表 |
| `EvidenceItemCard` | `src/features/evidence/EvidenceItemCard.tsx` | 单条证据项卡片 |
| `EvidenceWarningList` | `src/features/evidence/EvidenceWarningList.tsx` | 警告列表（可折叠） |

## 3. CollectEvidenceButton 设计

### 3.1 按钮位置

在 `StageDetail` 组件的文件列表上方，新增"收集证据"按钮。

### 3.2 按钮状态

| 状态 | 条件 | 外观 | 行为 |
|------|------|------|------|
| `disabled` | `StageContext.files.length === 0` | 灰色，不可点击 | — |
| `idle` | `files.length > 0` 且未开始收集 | 蓝色主题按钮，文案"收集证据" | 点击触发 `collectEvidence()` |
| `loading` | 正在收集 | 按钮变为 loading 状态，文案"正在收集证据..." | 禁止重复点击 |
| `done` | 收集完成 | 绿色，文案"已收集 N 条证据" | 点击可重新收集 |
| `error` | 收集失败 | 红色，文案"收集失败" + 错误信息 | 点击可重试 |

### 3.3 按钮可见性

- 只在选中了一个有效阶段时显示
- 空阶段（`stage_empty`）不显示
- 阶段文件列表为空时不显示（disabled 状态）

## 4. EvidencePanel 设计

### 4.1 面板位置

收集完成后，evidence 面板替换或追加到 `StageDetail` 的右栏区域。采用 **Tab 切换** 方式：

| Tab | 内容 |
|-----|------|
| "阶段详情" | 原有 StageDetail 内容（文件列表、外部依赖、上游引用） |
| "证据" | EvidencePanel 内容（新增） |

Tab 默认选中"阶段详情"，收集完成后自动切换到"证据" tab。

### 4.2 面板布局

```text
┌─────────────────────────────────────────────────┐
│ [阶段详情] [证据(N)]                              │ ← Tab 切换
├─────────────────────────────────────────────────┤
│  📊 总计: 12 条 | Python: 5 | Verilog: 4 | ...   │ ← EvidenceStatsBar
│  Direct: 8 | Indirect: 3 | Unknown: 1            │
├─────────────────────────────────────────────────┤
│  筛选: [全部▾] [按文件▾] [按类型▾] [按符号▾]      │ ← EvidenceFilterBar
├─────────────────────────────────────────────────┤
│  ┌─────────────────────────────────────────────┐ │
│  │ EV-L0-000001 | 🟢 direct                    │ │ ← EvidenceItemCard
│  │ top.py | L10-35                              │ │
│  │ def process_signal(data):                    │ │
│  │   ...                                        │ │
│  └─────────────────────────────────────────────┘ │
│  ┌─────────────────────────────────────────────┐ │
│  │ EV-L0-000002 | 🟢 direct                    │ │
│  │ top.py | L37-52                              │ │
│  │ class SignalProcessor:                       │ │
│  │   ...                                        │ │
│  └─────────────────────────────────────────────┘ │
│  ...                                             │
├─────────────────────────────────────────────────┤
│  ⚠ Warnings (2) ▾                               │ ← EvidenceWarningList
│  • src/large_file.v: 文件过大已跳过 (5.2MB)       │
│  • src/binary.dat: 非 UTF-8 文件已跳过            │
└─────────────────────────────────────────────────┘
```

## 5. EvidenceStatsBar 设计

### 5.1 展示内容

| 字段 | 来源 | 格式 |
|------|------|------|
| 证据总数 | `EvidenceCollection.stats.total_items` | "总计: N 条" |
| 按类型分组 | `EvidenceCollection.stats.items_by_kind` | "Python: 5, Verilog: 4, ..." |
| 按 confidence 分组 | `EvidenceCollection.stats.items_by_confidence` | "Direct: 8, Indirect: 3, Unknown: 1" |
| 跳过文件数 | `EvidenceCollection.stats.files_skipped` | "跳过: N 个文件"（如果 > 0） |

### 5.2 空状态

当 `evidence_items` 为空时：

```text
┌─────────────────────────────────────────────────┐
│                                                  │
│     未收集到证据                                  │
│                                                  │
│     该阶段可能无可提取的结构信息。                  │
│     收集了 3 个文件，跳过了 2 个文件。             │
│                                                  │
│     ⚠ Warnings (2) ▾                             │
│     • ...                                        │
│                                                  │
└─────────────────────────────────────────────────┘
```

## 6. EvidenceFilterBar 设计

### 6.1 筛选维度

| 维度 | 数据来源 | 筛选逻辑 |
|------|----------|----------|
| 全部 | — | 显示所有 evidence item |
| 按文件 | `index_by_path` 的 key 列表 | 下拉选择文件路径，展示该文件的 evidence |
| 按类型 | `index_by_kind` 的 key 列表 | 下拉选择 source_kind |
| 按符号 | `index_by_symbol` 的 key 列表 | 下拉选择 symbol |

### 6.2 筛选实现

```typescript
type FilterMode = 'all' | 'path' | 'kind' | 'symbol';

interface FilterState {
  mode: FilterMode;
  value?: string; // 选中的 key
}

// 筛选逻辑：从 index 中获取 evidence_id 列表，再过滤 evidence_items
function filterItems(
  items: EvidenceItem[],
  indexes: EvidenceCollection,
  filter: FilterState
): EvidenceItem[] {
  if (filter.mode === 'all') return items;

  const index = filter.mode === 'path' ? indexes.index_by_path
    : filter.mode === 'kind' ? indexes.index_by_kind
    : indexes.index_by_symbol;

  if (!filter.value || !index[filter.value]) return items;

  const ids = new Set(index[filter.value]);
  return items.filter(item => ids.has(item.evidence_id));
}
```

## 7. EvidenceItemCard 设计

### 7.1 展示字段

| 字段 | 来源 | 展示格式 |
|------|------|----------|
| evidence_id | `EvidenceItem.evidence_id` | 等宽字体，如 `EV-L0-000001` |
| confidence | `EvidenceItem.confidence` | 颜色标签：🟢 direct、🔵 indirect、⚪ unknown |
| 文件路径 | `EvidenceItem.source_path` | 截断展示（只显示文件名，hover 显示完整路径） |
| 行号范围 | `EvidenceItem.line_range` | `L{start}-{end}` 格式 |
| symbol | `EvidenceItem.symbol` | 有则显示，无则不显示 |
| summary | `EvidenceItem.summary` | 等宽字体，灰色背景，截断展示（最多 3 行） |
| language | `EvidenceItem.language` | 小标签 |
| source_kind | `EvidenceItem.source_kind` | 小标签 |

### 7.2 Confidence 颜色映射

```typescript
const CONFIDENCE_STYLE: Record<EvidenceStrength, { label: string; color: string }> = {
  direct:     { label: '直接证据', color: '#22c55e' },  // green
  indirect:   { label: '间接证据', color: '#3b82f6' },  // blue
  unknown:    { label: '不确定',   color: '#9ca3af' },  // gray
  weak:       { label: '弱证据',   color: '#f59e0b' },  // amber (Phase 3+)
  conflicting:{ label: '矛盾',     color: '#ef4444' },  // red (Phase 3+)
  missing:    { label: '缺失',     color: '#6b7280' },  // gray-dark (Phase 3+)
};
```

### 7.3 卡片交互

- **点击展开/收起** summary 完整内容
- **路径 hover** 显示完整绝对路径 tooltip
- **Phase 2 不做**：点击跳转到源码文件（Phase 5）

## 8. EvidenceWarningList 设计

### 8.1 展示逻辑

| 条件 | 展示 |
|------|------|
| `warnings.length === 0` | 不显示警告区域 |
| `warnings.length > 0` | 显示可折叠的警告列表，默认折叠 |
| 每条 warning | 展示 `error_code` + `message` + `source_path`（如果有） |

### 8.2 Warning 样式

- 黄色背景区域
- 可折叠（点击展开/收起）
- 折叠时只显示 "⚠ Warnings (N)" 计数

## 9. 前端状态管理

### 9.1 新增状态

在 `WorkspacePage` 的状态机中新增 evidence 相关状态：

```typescript
type AppState =
  // Phase 1 已有状态
  | { step: 'idle' }
  | { step: 'scanning' }
  | { step: 'scan_failed'; error: CommandError }
  | { step: 'workspace_loaded'; profile: WorkspaceProfile }
  | { step: 'selecting_stage'; profile: WorkspaceProfile }
  | { step: 'stage_selected'; profile: WorkspaceProfile; stage: StageContext }
  // Phase 2 新增状态
  | { step: 'collecting_evidence'; profile: WorkspaceProfile; stage: StageContext }
  | { step: 'evidence_collected'; profile: WorkspaceProfile; stage: StageContext; evidence: EvidenceCollection }
  | { step: 'evidence_failed'; profile: WorkspaceProfile; stage: StageContext; error: CommandError };
```

### 9.2 状态转换

```text
stage_selected
  → (用户点击"收集证据") → collecting_evidence
    → (成功) → evidence_collected
    → (失败) → evidence_failed
      → (用户点击"重试") → collecting_evidence
      → (用户选择其他阶段) → selecting_stage

evidence_collected
  → (用户点击"重新收集") → collecting_evidence
  → (用户选择其他阶段) → selecting_stage
```

### 9.3 TypeScript 类型新增

```typescript
// src/types/workspace.ts 新增

interface EvidenceItem {
  evidence_id: string;
  source_path: string;
  language: string;
  source_kind: string;
  line_range: { start: number; end: number };
  symbol?: string;
  summary: string;
  confidence: EvidenceStrength;
}

type EvidenceStrength = 'direct' | 'indirect' | 'unknown'
  | 'weak' | 'conflicting' | 'missing';

interface EvidenceCollection {
  stage_id: string;
  evidence_items: EvidenceItem[];
  index_by_path: Record<string, string[]>;
  index_by_kind: Record<string, string[]>;
  index_by_symbol: Record<string, string[]>;
  warnings: EvidenceWarning[];
  stats: EvidenceStats;
  version: string;
}

interface EvidenceWarning {
  error_code: string;
  message: string;
  source_path?: string;
}

interface EvidenceStats {
  files_processed: number;
  files_skipped: number;
  total_items: number;
  items_by_kind: Record<string, number>;
  items_by_confidence: Record<string, number>;
}
```

## 10. 前后端边界

| 职责 | 前端 | 后端 |
|------|------|------|
| 触发收集 | ✅ 按钮点击 → 调用 `collectEvidence(rootPath, stageId)` | ✅ command 接收参数，执行收集 |
| 文件读取 | ❌ | ✅ 后端负责所有文件读取 |
| 提取逻辑 | ❌ | ✅ 后端负责所有提取逻辑 |
| evidence_id 生成 | ❌ | ✅ 后端生成 |
| 索引构建 | ❌ | ✅ 后端构建 |
| 错误处理 | ✅ 展示错误信息 | ✅ 返回 CommandResult |
| 筛选/排序 | ✅ 前端本地筛选（使用 index） | ❌ 后端一次性返回完整数据 |
| 分页 | ❌ Phase 2 不做分页 | ❌ |
| 持久化 | ❌ Phase 2 不持久化 | ❌ |

## 11. 不做的事情

- **不做代码高亮渲染**：summary 用等宽字体展示，不做语法高亮
- **不做 evidence item 点击跳转到源码**：Phase 5 解决
- **不做实时协作**：单人桌面应用
- **不做 evidence 编辑**：只读展示
- **不做分页/虚拟滚动**：Phase 2 阶段文件数量有限，直接渲染
- **不做持久化**：页面刷新后需重新收集

## 12. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-11 | 初始创建 | Claude |
