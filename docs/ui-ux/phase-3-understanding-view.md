# Phase 3 理解面板前端设计

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 3 前端 `UnderstandingPanel` 组件的设计，用于展示 `ImplementationUnderstanding` 的结构化理解结果。
>
> **Phase 3 不编码**。本文档是 draft，编码前需审核收口。

## 1. 组件定位

`UnderstandingPanel` 嵌入 `StageDetail` 组件中，位于证据收集区域之后，展示阶段的结构化理解结果。

```text
StageDetail
├── 文件列表
├── 证据收集区域（Phase 2）
│   ├── 收集/重新收集按钮
│   ├── 错误面板
│   └── EvidencePanel
├── 理解生成区域（Phase 3）  ← 新增
│   ├── 生成/重新生成按钮
│   ├── 错误面板
│   └── UnderstandingPanel   ← 新增
├── 外部依赖
└── 上游引用
```

## 2. 状态设计

### 2.1 前端状态扩展

在 `WorkspacePage.tsx` 的 `AppState` 中新增 Phase 3 相关状态：

```typescript
// 新增的 phase 状态
| { phase: 'generating_understanding'; profile; stageId; context; evidence }
| { phase: 'understanding_loaded'; profile; stageId; context; evidence; understanding: ImplementationUnderstanding }
| { phase: 'understanding_error'; profile; stageId; context; evidence; error: UiError }
```

### 2.2 UnderstandingPanel 显示状态

| 状态 | 含义 | UI 表现 |
|------|------|---------|
| `generating` | 正在生成理解 | 加载动画 + "生成理解中..." |
| `success` | 生成成功 | 完整面板内容 |
| `failure` | 生成失败 | 错误面板 + 重试按钮 |
| `degraded` | 降级模式（无 LLM） | 基本展示 + "降级模式"提示 |
| `unknown_heavy` | unknown 项过多 | 警告提示 + 完整面板 |

## 3. UnderstandingPanel 组件结构

```text
UnderstandingPanel
├── 状态栏（生成状态、provider 信息、耗时）
├── 阶段摘要区
│   └── summary 文本
├── 统计概览
│   ├── claim 总数
│   ├── confidence 分布（confirmed/inferred/unknown/conflicting）
│   └── 摘要数量（模块/信号/接口/处理步骤）
├── Claim 列表
│   └── ClaimCard × N
│       ├── category 标签
│       ├── confidence 标签（颜色映射）
│       ├── description 文本
│       ├── evidence_refs 列表（可点击）
│       └── evidence_gap 标记（如有）
├── 模块摘要区
│   └── ModuleSummaryCard × N
│       ├── 模块名称
│       ├── 描述
│       ├── confidence 标签
│       └── evidence_refs 列表
├── 信号摘要区
│   └── SignalSummaryCard × N
├── 接口摘要区
│   └── InterfaceSummaryCard × N
├── 处理步骤区
│   └── ProcessingStepCard × N（按 order 排序）
├── Unknown 区域
│   └── UnknownItemCard × N
│       ├── 描述
│       ├── 原因说明
│       └── related_evidence_refs（如有）
└── Evidence Gap 区域
    └── EvidenceGapCard × N
        ├── 期望 evidence 描述
        ├── 原因说明
        └── related_evidence_refs（如有）
```

## 4. Confidence 颜色映射

| Confidence | 背景色 | 文字色 | 标签文字 |
|-----------|--------|--------|----------|
| `confirmed` | `#e8f5e9`（浅绿） | `#2e7d32`（深绿） | 已确认 |
| `inferred` | `#e3f2fd`（浅蓝） | `#1565c0`（深蓝） | 推断 |
| `unknown` | `#f5f5f5`（浅灰） | `#757575`（灰色） | 未知 |
| `conflicting` | `#ffebee`（浅红） | `#c62828`（深红） | 矛盾 |

## 5. Category 标签映射

| Category | 标签文字 | 标签色 |
|----------|----------|--------|
| `module_structure` | 模块结构 | `#e8eaf6` |
| `signal_definition` | 信号定义 | `#e0f2f1` |
| `interface_description` | 接口描述 | `#fff3e0` |
| `data_processing` | 数据处理 | `#fce4ec` |
| `configuration` | 配置约束 | `#f3e5f5` |
| `documentation` | 文档注释 | `#e8f5e9` |
| `test_coverage` | 测试覆盖 | `#e1f5fe` |
| `other` | 其他 | `#f5f5f5` |

## 6. Evidence 回链交互

### 6.1 evidence_id 展示

- 每个 claim/摘要/unknown/gap 中的 evidence_refs 以 chip 形式展示
- chip 显示 evidence_id（如 `EV-L0-000001`）
- chip 可点击

### 6.2 点击行为

点击 evidence_id chip 时：
1. 在 EvidencePanel 中高亮对应的 evidence item
2. 如果 EvidencePanel 未展示，自动滚动到 EvidencePanel 并高亮
3. 高亮持续 2 秒后渐隐

### 6.3 回链约束

- 只展示 evidence_id，不重复展示 source_path / line_range
- source_path / line_range 信息通过回链到 EvidenceItem 获取
- 如果 evidence_id 对应的 evidence item 不在当前 EvidenceCollection 中，显示为灰色（不应发生，但做防御性处理）

## 7. 生成按钮设计

### 7.1 显示条件

- 仅在 `evidence_loaded` 状态且有证据项时显示
- evidence 为空时不显示

### 7.2 按钮状态

| 状态 | 文案 | 颜色 | 可点击 |
|------|------|------|--------|
| 未生成 | "生成理解" | 蓝色 `#1976d2` | ✅ |
| 生成中 | "生成理解中..." | 灰色 `#e0e0e0` | ❌ |
| 已生成 | "重新生成" | 绿色 `#4caf50` | ✅ |
| 生成失败 | "重试生成" | 橙色 `#f57c00` | ✅ |
| 降级模式 | "降级模式 — 生成基本理解" | 灰色 `#9e9e9e` | ✅ |

## 8. Unknown 和 Evidence Gap 展示

### 8.1 Unknown 区域

- 标题："无法推断的信息"
- 每个 UnknownItem 展示：
  - 描述（中文）
  - 原因说明
  - 相关 evidence（如有）
- 不隐藏或淡化 unknown 项
- 当 unknown 数量超过 claim 数量时，显示警告："当前理解结果中未知项较多，建议补充更多证据后重新生成"

### 8.2 Evidence Gap 区域

- 标题："证据缺失"
- 每个 EvidenceGap 展示：
  - 期望的 evidence 描述
  - 原因说明
  - 相关已有 evidence（如有）

### 8.3 视觉区分

Unknown 和 Evidence Gap 区域使用灰色背景（`#fafafa`），与正常的 claim 区域视觉区分。

## 9. 禁止用语

以下用语在 UI 中**严禁使用**：

| 禁止 | 原因 |
|------|------|
| "正确" / "错误" | 产品是理解工具，不是审计器 |
| "PASS" / "HOLD" | 审计结论用语 |
| "审计结论" | 产品不做审计判断 |
| "验证通过" / "验证失败" | 误导用户以为代码正确性已验证 |
| "代码质量" / "设计缺陷" | 超出理解工具范围 |
| "建议修改" / "需要修复" | 不提供修改建议 |
| "100% 理解" / "完全正确" | 不支持的不当确定性 |

**允许用语**：

| 允许 | 用途 |
|------|------|
| "已确认" | confidence = confirmed |
| "推断" | confidence = inferred |
| "未知" | confidence = unknown |
| "矛盾" | confidence = conflicting |
| "无法推断" | unknown 项描述 |
| "证据缺失" | evidence gap 描述 |

## 10. 与 StageDetail 的集成

### 10.1 Props 扩展

```typescript
interface StageDetailProps {
  context: StageContext;
  evidence?: EvidenceCollection;
  evidenceError?: UiError;
  isCollecting?: boolean;
  onCollectEvidence?: () => void;
  // Phase 3 新增
  understanding?: ImplementationUnderstanding;
  understandingError?: UiError;
  isGenerating?: boolean;
  onGenerateUnderstanding?: () => void;
}
```

### 10.2 渲染逻辑

1. 证据收集完成后（`evidence_loaded` 且 `evidence.evidence_items.length > 0`），显示"生成理解"按钮
2. 点击按钮 → `onGenerateUnderstanding` → 前端调用 `generateUnderstanding` command
3. 生成中 → 显示 loading
4. 生成完成 → 渲染 `UnderstandingPanel`
5. 生成失败 → 显示错误面板

## 11. TypeScript 类型

### 11.1 新增类型（在 `workspace.ts` 中）

```typescript
interface ImplementationUnderstanding {
  stage_id: string;
  version: string;
  summary: string;
  claims: ImplementationClaim[];
  module_summaries: ModuleSummary[];
  signal_summaries: SignalSummary[];
  interface_summaries: InterfaceSummary[];
  processing_steps: ProcessingStepSummary[];
  unknowns: UnknownItem[];
  evidence_gaps: EvidenceGap[];
  generation_meta: GenerationMeta;
  stats: UnderstandingStats;
}

type ClaimConfidence = 'confirmed' | 'inferred' | 'unknown' | 'conflicting';
type ClaimCategory = 'module_structure' | 'signal_definition' | 'interface_description' | 'data_processing' | 'configuration' | 'documentation' | 'test_coverage' | 'other';

interface ImplementationClaim {
  claim_id: string;
  category: ClaimCategory;
  description: string;
  confidence: ClaimConfidence;
  evidence_refs: EvidenceRef[];
  has_evidence_gap: boolean;
}

interface EvidenceRef {
  evidence_id: string;
  relevance?: string;
}

interface ModuleSummary {
  name: string;
  description: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

interface SignalSummary {
  name: string;
  description: string;
  direction?: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

interface InterfaceSummary {
  name: string;
  description: string;
  interface_type?: string;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

interface ProcessingStepSummary {
  name: string;
  description: string;
  order: number;
  evidence_refs: EvidenceRef[];
  confidence: ClaimConfidence;
}

interface UnknownItem {
  unknown_id: string;
  description: string;
  related_evidence_refs: EvidenceRef[];
  reason: string;
}

interface EvidenceGap {
  gap_id: string;
  expected_evidence: string;
  reason: string;
  related_evidence_refs: EvidenceRef[];
}

interface GenerationMeta {
  provider: string;
  generated_at: string;
  input_evidence_count: number;
  generation_time_ms: number;
  is_degraded: boolean;
}

interface UnderstandingStats {
  total_claims: number;
  claims_by_confidence: Record<string, number>;
  claims_by_category: Record<string, number>;
  module_count: number;
  signal_count: number;
  interface_count: number;
  processing_step_count: number;
  unknown_count: number;
  evidence_gap_count: number;
}
```

### 11.2 Tauri Command 调用

```typescript
// src/lib/tauriCommands.ts 新增
export async function generateUnderstanding(
  rootPath: string,
  stageId: string,
): Promise<ImplementationUnderstanding> {
  const result = await invoke<CommandResult<ImplementationUnderstanding>>(
    'generate_understanding',
    { rootPath, stageId },
  );
  return handleResult(result);
}
```

## 12. 无图视图 / 无 Q&A / 无报告导出

Phase 3 前端**不做**：

- ❌ 结构图 / 数据流图 / 时序图
- ❌ Q&A 对话界面
- ❌ Markdown/PDF 报告导出
- ❌ 原始 JSON 展示
- ❌ 跨阶段对比视图
- ❌ 实时编辑理解结果
- ❌ 持久化 / 本地存储

## 13. 中文 UI

所有用户可见文案使用中文：

- 按钮：生成理解 / 重新生成 / 生成理解中... / 重试生成
- 区域标题：阶段摘要 / 实现声明 / 模块摘要 / 信号摘要 / 接口摘要 / 处理步骤 / 无法推断的信息 / 证据缺失
- 标签：已确认 / 推断 / 未知 / 矛盾
- 统计：声明总数 / 确认 / 推断 / 未知 / 矛盾
- 错误：理解生成失败 / 降级模式

## 14. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft） | Claude |
