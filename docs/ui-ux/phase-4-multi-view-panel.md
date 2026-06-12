# Phase 4 三视图面板前端设计

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 4 前端三视图面板（MultiViewPanel）的设计：结构图 / 数据流图 / 时序流水图的 tab 切换、节点渲染、交互和状态管理。

## 1. 组件定位

`MultiViewPanel` 嵌入 `StageDetail` 组件中，位于 `UnderstandingPanel` 之后，基于 `ViewGraph` 渲染三类视图。

```text
StageDetail
├── 文件列表
├── 证据收集区域（Phase 2）
├── 理解生成区域（Phase 3）
│   └── UnderstandingPanel
├── 三视图区域（Phase 4）  ← 新增
│   ├── Tab bar: 结构图 | 数据流 | 时序流水
│   ├── 当前视图渲染区
│   └── Node/Edge hover tooltip
├── 外部依赖
└── 上游引用
```

## 2. 状态设计

### 2.1 前端状态扩展

在 `WorkspacePage.tsx` 的 `AppState` 中新增 Phase 4 相关状态：

```typescript
| { phase: 'views_loading'; profile; stageId; context; evidence?; understanding; }
| { phase: 'views_loaded'; profile; stageId; context; evidence?; understanding; views: ViewGraph[]; }
| { phase: 'views_error'; profile; stageId; context; evidence?; understanding; viewsError: UiError; }
```

**前置条件**：进入 `views_*` 状态前必须已持有 `understanding`（Phase 3 `understanding_loaded` 状态）。若 `understanding` 不存在，不展示"生成视图"按钮。

### 2.2 MultiViewPanel 显示状态

| 状态 | 含义 | UI 表现 |
|------|------|---------|
| `loading` | 正在生成三视图 | 三个 tab + 加载动画 |
| `loaded` | 三视图生成成功 | 当前 tab 内容渲染 |
| `empty` | IU 数据不足以生成视图 | 节点/边为空的轻量空状态 |
| `error` | 生成失败 | 当前 tab 显示错误面板 |
| `degraded` | 来自 degraded IU | tab 内容标注"降级数据" |

## 3. MultiViewPanel 组件结构

```text
MultiViewPanel
├── Tab bar
│   ├── 结构图（默认选中）
│   ├── 数据流
│   └── 时序流水
├── 当前 View 渲染区
│   ├── 空状态（无数据时）
│   ├── 错误面板（生成失败时）
│   ├── Degraded 提示
│   └── 正常视图（节点 + 边 SVG/CSS 渲染）
└── Hover Tooltip
    ├── Node/Edge 名称
    ├── confidence 标签
    └── trace_refs 列表（claim_id / evidence_id）
```

## 4. Tab Bar 设计

### 4.1 外观

- 三个 tab 水平排列，使用 segmented control 风格
- 选中 tab：蓝色下划线 + 加粗
- 未选中 tab：灰色文字
- 始终显示三个 tab，即使某个视图数据为空

### 4.2 交互

- 点击 tab 切换到对应视图
- 切换不触发后端重新计算
- 当前选中 tab 记忆在组件 state 中
- 阶段切换时重置为默认 tab（结构图）

## 5. 视图渲染方案

### 5.1 渲染策略

- 使用纯 **SVG + CSS** 方案，不引入 React Flow / D3 / Mermaid
- 节点使用 `<rect>` + `<text>` 组合
- 边使用 `<line>` 或 `<path>` + 箭头标记
- 布局使用固定 grid 模式，参考 `ViewLayoutHint` 中的 column/row/depth

### 5.2 选择 SVG 的原因

- 零外部依赖
- 与项目现有 inline style 风格一致
- 足够满足 MVP 阶段三类视图的可读性需求
- 避免大型图形库的版本兼容和学习成本

### 5.3 节点渲染

| NodeType | 形状 | 颜色 | 示例 |
|----------|------|------|------|
| Module | 圆角矩形 | 蓝色 `#e3f2fd` 边框 `#1565c0` | 模块 |
| Function | 矩形 | 绿色 `#e8f5e9` 边框 `#2e7d32` | 函数 |
| Interface | 菱形 | 紫色 `#f3e5f5` 边框 `#7b1fa2` | 接口 |
| Signal | 小圆角矩形 | 灰色 `#f5f5f5` 边框 `#757575` | 信号 |
| InputSource | 左圆角矩形 | 绿色 `#c8e6c9` | 输入源 |
| OutputTarget | 右圆角矩形 | 橙色 `#ffe0b2` | 输出目标 |
| ProcessingStep | 矩形 | 蓝色 `#bbdefb` | 处理步骤 |
| PipelineStage | 矩形 | 蓝色 `#bbdefb` | 流水级 |
| ClockDomain | 六边形 | 黄色 `#fff9c4` | 时钟域 |
| IntermediateData | 小矩形 | 灰色 `#eeeeee` | 中间数据 |

### 5.4 边渲染

- 带箭头直线
- 实线 = confirmed / supported
- 虚线 = inferred
- 灰色细线 = unknown

### 5.5 置信度视觉编码

| confidence | 节点边框 | 边样式 |
|------------|----------|--------|
| confirmed | 实线 2px | 实线 2px |
| supported | 实线 2px | 实线 1.5px |
| inferred | 虚线 2px | 虚线 1.5px |
| unknown | 点线 1px | 点线 1px 灰色 |
| conflicting | 实线 2px 红色 `#c62828` | 实线 1.5px 红色 |

## 6. Tooltip 设计（只读信息层）

- 桌面端 hover 节点/边 200ms 后显示 tooltip
- touch 设备 tap 节点/边显示 popover（不做 hover）
- keyboard focus 也触发 tooltip（可访问性）
- tooltip 内容：
  - 名称（加粗）
  - 类型标签（NodeType 中文）
  - confidence 彩色标签
  - trace_refs 列表：`claim_id` + `evidence_id`（蓝色 chip）
  - 无 trace 时显示"无证据追溯"
- tooltip 位于鼠标右下方
- mouseleave / blur / tap-away 时立即消失
- **禁止**：点击 tooltip 或节点/边触发源码导航、evidence 跳转、EvidencePanel 高亮（Phase 5）

## 7. 空状态与错误状态

### 7.1 空状态

| 场景 | 显示 |
|------|------|
| IU 尚未生成 | 不显示 MultiViewPanel（无"生成视图"按钮） |
| IU 为 degraded | 三视图 tab + "降级数据，视图内容有限" 横幅 |
| ViewMeta.empty_reason 非空 | 对应 tab 显示 `empty_reason` 中的文案 + 轻量空状态图标 |
| nodes=[] 且 edges=[] | 不渲染 SVG 画布，直接展示空状态 |

### 7.2 错误状态

- 单个 view 生成失败 → 对应 tab 显示 ErrorPanel（红底），其他 tab 正常
- 全部 view 生成失败 → 三个 tab 都显示错误，但 tab bar 仍可切换

## 8. 交互规范

- **禁止** 拖拽节点
- **禁止** 缩放/平移（MVP）
- **禁止** 双击编辑
- **允许** hover 查看 tooltip
- **允许** 点击 tab 切换视图
- **禁止** 使用"正确/错误"、"PASS/HOLD"等审计用语

## 9. 文案规范

| 用途 | 文案 |
|------|------|
| Tab 1 | 结构图 |
| Tab 2 | 数据流 |
| Tab 3 | 时序流水 |
| 按钮 | 生成视图 |
| 重新生成 | 重新生成视图 |
| 生成中 | 生成视图中... |
| 降级提示 | 当前为降级数据，视图内容有限 |
| 空状态-通用 | 无足够数据生成视图 |
| 空状态-无处理步骤 | 无处理步骤信息 |
| 无证据追溯 | 无证据追溯 |
| 错误标题 | 视图生成失败 |

## 10. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft）：定义 MultiViewPanel 布局、Tab bar、SVG 渲染方案、节点/边颜色/形状/置信度编码、hover tooltip、空状态、交互规范 | Claude |
