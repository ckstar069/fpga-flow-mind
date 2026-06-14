# Phase 5 证据回链与 Grounded Q&A 前端设计

---
status: active
updated: 2026-06-14
---

> 本文档定义 Phase 5 前端界面：视图节点/边可选中态、TracePanel、SourceExcerptPanel、EvidencePanel 高亮、GroundedQAPanel 的布局与交互。实施前必须与 `phase-5-trace-model.md` 和 `phase-5-trace-and-qa-design.md` 对齐。
>
> 本文档已收口（status=active），是 Phase 5 编码依据。

## 1. 整体布局

```text
WorkspacePage
├── 左侧：WorkspaceSummary + StageList
└── 右侧：StageDetail
    ├── 文件列表
    ├── 证据收集/生成理解区域
    ├── UnderstandingPanel
    ├── MultiViewPanel（Phase 4）
    │   └── 节点/边可选中 ← 新增
    ├── TracePanel / SourceExcerptPanel ← 新增
    ├── GroundedQAPanel ← 新增
    └── EvidencePanel（Phase 2，支持高亮）← 增强
```

## 2. MultiViewPanel 选中态

### 2.1 节点/边可选中

- 点击节点或边后，该元素进入 `selected` 状态。
- 选中态视觉：
  - 节点：边框加粗 + 外发光阴影（`box-shadow`）。
  - 边：线宽加粗 + 颜色加深。
- 同一视图内同时只能选中一个 node 或一个 edge。
- 切换 tab 时保留选中目标（若目标在新 view 中存在）。
- 切换阶段时清空选中态。

### 2.2 选中后触发

- 选中节点/边后，前端发送 `resolve_trace_target` Tauri command。
- TracePanel 展示解析结果。
- 若节点/边无 `trace_refs`，TracePanel 显示"无证据追溯"。

## 3. TracePanel

### 3.1 位置

- 默认位于 StageDetail 右侧或下方，宽度 320px 或占父容器 1/3。
- 可折叠，折叠后显示"已选择 X 个追溯目标"提示。

### 3.2 内容结构

```text
TracePanel
├── 头部
│   ├── 标题：追溯详情
│   ├── 清空选择按钮
│   └── 关闭按钮
├── 选中目标摘要
│   ├── 类型标签（节点/边/claim/evidence）
│   └── 名称/ID
├── Trace 列表
│   └── TraceRefResolved 卡片
│       ├── ClaimSnapshot（若有）
│       ├── EvidenceSnapshot（若有）
│       ├── confidence 标签
│       ├── relevance 说明
│       └── 操作：查看源码片段 / 在 EvidencePanel 中定位
└── 空状态
    └── "点击视图节点/边或 claim/evidence 查看追溯"
```

### 3.3 Trace 卡片

每张卡片显示：
- claim 描述（若有）
- evidence_id + source_path + line_range
- strength / confidence 标签
- "查看源码片段"按钮 → 打开 SourceExcerptPanel
- "定位 evidence"按钮 → EvidencePanel 高亮

## 4. SourceExcerptPanel

### 4.1 位置

- 可嵌入 TracePanel 下方，或作为独立抽屉/弹窗。
- 同时只展示一个 excerpt。

### 4.2 内容结构

```text
SourceExcerptPanel
├── 头部
│   ├── 文件路径
│   ├── 语言标签
│   ├── 行号范围
│   └── 关闭按钮
├── 源码区域
│   ├── 行号列（1-based）
│   ├── 源码内容列
│   └── 截断提示（若有）
└── Warnings（若有）
    └── 二进制/非 UTF-8/超大等警告
```

### 4.3 视觉规范

- 等宽字体。
- 行号列背景略灰。
- 无语法高亮（MVP）。
- 截断行显示 `...（已截断，共 N 行）`。

## 5. EvidencePanel 高亮状态

### 5.1 高亮触发

- 用户在 TracePanel 中点击"定位 evidence"。
- 用户在 SourceExcerptPanel 头部点击"在 EvidencePanel 中打开"。

### 5.2 高亮视觉

- 对应 evidence item 背景色变为浅黄色/蓝色。
- 滚动到该 item。
- 高亮持续 3 秒或直到用户点击其他 evidence。

### 5.3 当前 source 状态

- 当 SourceExcerptPanel 打开某 evidence 时，EvidencePanel 中对应 item 显示"当前查看"标记。

## 6. GroundedQAPanel

### 6.1 位置

- 位于 TracePanel 下方或 StageDetail 底部。
- 展开高度约 240px，可拖拽调整（MVP 可固定）。

### 6.2 内容结构

```text
GroundedQAPanel
├── 头部
│   ├── 标题：追问
│   └── 清空历史按钮
├── 问答历史
│   └── Q&A 条目
│       ├── 用户问题
│       ├── 系统回答
│       ├── confidence 标签
│       └── citations 列表（可点击）
├── 输入区
│   ├── 问题输入框
│   ├── 关联上下文提示（若已选中节点/边）
│   └── 提交按钮
└── 空状态
    └── "基于当前阶段的证据提问"
```

### 6.3 输入约束

- 问题最大 500 字符。
- 空问题不可提交。
- 提交后输入框 disabled 直到回答返回。

### 6.4 回答展示

- 回答文本中 citation 以 `[1]` `[2]` 上标形式展示。
- 点击 citation 打开对应 SourceExcerptPanel。
- 回答末尾列出所有 citations 的 evidence_id / source_path。
- unknown 回答以灰色背景 + "证据不足"图标展示。

## 7. 视觉语义

### 7.1 confidence 显示

复用 Phase 3/4 的语义：

| confidence | 颜色 | 标签文字 |
|------------|------|---------|
| confirmed | 蓝色 `#1565c0` | 已确认 |
| supported | 琥珀色 `#f57c00` | 有支撑 |
| inferred | 绿色 `#2e7d32` | 推断 |
| unknown | 灰色 `#757575` | 未知 |
| conflicting | 红色 `#c62828` | 矛盾 |

### 7.2 selected / highlight / current-source 状态

| 状态 | 视觉 |
|------|------|
| selected | 节点/边边框加粗 + 外发光 |
| highlight | EvidencePanel item 背景高亮 |
| current-source | EvidencePanel item 左侧蓝色竖条 |

### 7.3 loading / error / empty 状态

| 状态 | 视觉 |
|------|------|
| loading | TracePanel/Q&A 输入区显示 spinner |
| error | 红色错误条，显示错误消息 |
| empty | 灰色占位文案 + 轻量图标 |

## 8. 点击交互

| 点击目标 | 行为 |
|----------|------|
| ViewNode / ViewEdge | 选中 + resolve_trace_target + TracePanel 展示 |
| claim 的 evidence chip | `SelectedTraceTarget::Claim` + TracePanel 展开 |
| evidence chip | `SelectedTraceTarget::Evidence` + TracePanel 展开 |
| Trace 卡片"查看源码片段" | SourceExcerptPanel 展示 |
| Trace 卡片"定位 evidence" | EvidencePanel 高亮 |
| Q&A citation | SourceExcerptPanel 展示 |

## 9. 明确不能做

- **不打开外部编辑器**：所有 source excerpt 在应用内展示。
- **不做 PASS/HOLD 结论**：界面文案不出现"通过/不通过"。
- **不隐藏 unknown**：unknown 节点、claim、回答必须可见。
- **不让 evidence_id 只是不可追溯的字符串**：每个 evidence_id 必须可点击定位或查看源码。
- **不做自由聊天**：Q&A 必须绑定 citations。
- **不修改源码**：无任何编辑按钮。

## 10. 文案规范

| 用途 | 文案 |
|------|------|
| TracePanel 标题 | 追溯详情 |
| SourceExcerptPanel 标题 | 源码片段 |
| GroundedQAPanel 标题 | 追问 |
| 无 trace | 无证据追溯 |
| 证据缺失 | 证据缺失：{reason} |
| unknown 回答 | 根据当前证据无法确定 |
| 提交问题 | 提问 |
| 生成中 | 思考中... |
| 清空历史 | 清空对话 |

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-13 | 初始创建（draft）：定义 MultiViewPanel 选中态、TracePanel、SourceExcerptPanel、EvidencePanel 高亮、GroundedQAPanel 布局与视觉语义 | Claude |
