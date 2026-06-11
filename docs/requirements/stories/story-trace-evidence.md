# Story: 追溯证据

---
status: draft
updated: 2026-06-11
---

## 用户目标

作为 FPGA 开发人员，我希望点击图中的节点或结论后，能查看对应的源码证据，以便验证系统的理解是否正确。

## 业务背景

所有语义结论必须有源码证据支撑。用户需要通过追溯 evidence 来验证系统生成的结构、数据流和时序结论是否与真实代码一致。

## 前置条件

- 用户正在查看某类视图（结构图、数据流图或时序图）
- 图中存在可点击的节点或边
- evidence_index 已建立

## 触发入口

用户在图中点击一个节点或边，或点击节点详情面板中的"查看证据"按钮。

## 主流程

1. 用户点击图中的节点或边
2. 系统查找关联的 evidence_refs
3. 系统从 evidence_index 中检索 evidence items
4. 系统展示 evidence 面板
5. 用户查看证据详情

## 功能点清单

### TR-001 点击节点/边触发追溯

- **用户动作**：用户在图中点击一个节点或边
- **系统必须**：
  - 高亮选中的节点/边
  - 查找该节点/边携带的 `evidence_refs`
  - 打开 evidence 面板
- **成功结果**：evidence 面板展示与该节点/边关联的证据
- **失败表现**：无 evidence_refs 时提示"该节点暂无关联证据"
- **evidence 要求**：不需要（此动作为触发追溯）
- **MVP 必须**：是

### TR-002 evidence 面板显示字段

- **系统动作**：在 evidence 面板中展示证据详情
- **系统必须**：面板中展示以下字段：
  - `evidence_id`：唯一标识
  - `source_path`：源码文件绝对路径
  - `line_range`：起始行号 - 结束行号
  - `language`：语言类型（python / verilog / systemverilog）
  - `source_kind`：来源类型（python_stage / rtl / test / doc / config）
  - `symbol`：符号名称（函数名/模块名/信号名）
  - `summary`：代码片段或描述
  - `confidence`：该证据的置信度
- **成功结果**：用户可查看 evidence 的完整信息
- **失败表现**：evidence 字段缺失时标注"信息缺失"
- **evidence 要求**：面板内容直接来自 evidence_index
- **MVP 必须**：是

### TR-003 源码路径和行号范围呈现

- **系统动作**：清晰展示源码位置和范围
- **系统必须**：
  - `source_path` 以可点击链接形式展示（点击可在系统编辑器中打开文件，或展示代码片段）
  - `line_range` 格式为"第 X 行 - 第 Y 行"
  - 如为单行，格式为"第 X 行"
  - 展示对应行号的代码片段（上下各 3 行上下文）
- **成功结果**：用户可精确定位到源码位置
- **失败表现**：行号范围无效时标注"行号信息缺失"
- **evidence 要求**：source_path 和 line_range 必须准确
- **MVP 必须**：是

### TR-004 多证据展示

- **系统动作**：当一个节点/边关联多个 evidence 时展示
- **系统必须**：
  - 以列表形式展示所有关联的 evidence items
  - 每个 evidence item 可展开/折叠查看详情
  - 按 source_kind 分组（如 RTL 证据在前，文档证据在后）
  - 标注每个 evidence 的 confidence
- **成功结果**：用户可浏览所有支撑证据
- **失败表现**：无
- **evidence 要求**：每个 evidence 必须包含完整的 evidence_id、source_path、line_range
- **MVP 必须**：是

### TR-005 evidence 不存在处理

- **系统动作**：当 evidence_refs 中的 id 在 evidence_index 中找不到时处理
- **系统必须**：
  - 提示"部分证据已失效或丢失"
  - 列出失效的 evidence_ids
  - 展示仍可用的 evidence（如有）
  - 提供"重新收集证据"的建议
- **成功结果**：用户了解 evidence 失效情况
- **失败表现**：无
- **evidence 要求**：标注失效的 evidence_id
- **MVP 必须**：是

### TR-006 evidence 失效处理

- **系统动作**：当 evidence 对应的源码文件已被修改或删除时处理
- **系统必须**：
  - 对比 evidence 中的 source_path 与当前文件系统状态
  - 如文件已删除，标注"源文件已删除"
  - 如文件已修改且行号范围可能偏移，标注"源文件已变更，行号可能偏移"
  - 仍展示 evidence 中缓存的代码片段
- **成功结果**：用户了解 evidence 的时效性
- **失败表现**：无
- **evidence 要求**：展示 evidence 的缓存内容和时效性状态
- **MVP 必须**：否（MVP 可选，首次打开时通常不会遇到）

### TR-007 代码片段展示

- **系统动作**：在 evidence 面板中展示代码片段
- **系统必须**：
  - 展示 evidence line_range 对应的代码内容
  - 提供语法高亮（Python / Verilog）
  - 行号与代码对齐显示
  - 如代码片段过长，提供展开/折叠
- **成功结果**：用户可直接阅读源码证据
- **失败表现**：代码片段无法读取时标注"无法加载代码片段"
- **evidence 要求**：代码片段来自 source_path 和 line_range 指定的位置
- **MVP 必须**：是

### TR-008 从 evidence 面板返回视图

- **用户动作**：用户关闭 evidence 面板或点击返回
- **系统必须**：
  - 关闭 evidence 面板
  - 保持当前视图状态（不刷新或重置）
  - 保持节点高亮状态（或取消高亮）
- **成功结果**：用户可继续浏览图或选择其他节点
- **失败表现**：无
- **evidence 要求**：不需要
- **MVP 必须**：是

## 输入

| 输入项 | 来源 | 类型 |
|--------|------|------|
| 用户点击的节点/边 | 图中交互 | UI 事件 |
| evidence_refs | 节点/边携带 | evidence_id 列表 |
| evidence_index.json | story-collect-evidence 输出 | 结构化数据 |

## 输出

| 输出项 | 类型 | 说明 |
|--------|------|------|
| evidence 面板 | UI 面板 | evidence 详情、代码片段 |
| 高亮状态 | UI 状态 | 选中节点/边的高亮 |

## 异常 / 空状态

| 场景 | 处理 |
|------|------|
| 节点无 evidence_refs | 提示"该节点暂无关联证据" |
| evidence_id 在索引中不存在 | 提示"部分证据已失效"，列出失效 id |
| 源文件已删除 | 标注"源文件已删除"，展示缓存片段 |
| 行号范围无效 | 标注"行号信息缺失"，展示整个文件 |
| 代码片段加载失败 | 标注"无法加载代码片段"，展示 evidence 其他字段 |

## 证据与追溯要求

- evidence 面板中的每个字段必须直接来自 evidence_index
- `evidence_id` 必须可验证存在于 evidence_index
- `source_path` 和 `line_range` 必须准确指向源码位置
- 多证据时必须完整展示所有关联的 evidence items

## 不确定性表达要求

- `unknown` 证据以降级样式展示
- `inferred` 证据标注推断依据
- `conflicting` 证据以警示样式展示，列出冲突点
- 失效的 evidence 明确标注时效性问题

## MVP 验收标准

- [ ] 点击节点/边可打开 evidence 面板
- [ ] evidence 面板展示 evidence_id、source_path、line_range、language、symbol、summary
- [ ] 源码路径以可点击形式展示
- [ ] 行号范围格式正确
- [ ] 展示对应代码片段（含语法高亮）
- [ ] 多证据以列表形式展示
- [ ] evidence 失效时有恰当提示
- [ ] 关闭面板后保持视图状态

## 非目标

- 不直接在 evidence 面板中编辑源码（只读）
- 不做代码 diff 对比（留到后续版本）
- 不做跨版本证据追溯
- 不把 evidence 面板做成独立的 JSON viewer

## 关联文档

- [`story-view-structure.md`](story-view-structure.md) — 相关：结构图中的节点点击
- [`story-view-dataflow.md`](story-view-dataflow.md) — 相关：数据流图中的节点点击
- [`story-view-timing.md`](story-view-timing.md) — 相关：时序图中的节点点击
- [`story-ask-node-question.md`](story-ask-node-question.md) — 相关：基于 evidence 追问
- [`../mvp-requirements.md`](../mvp-requirements.md) — MVP 证据回链要求
