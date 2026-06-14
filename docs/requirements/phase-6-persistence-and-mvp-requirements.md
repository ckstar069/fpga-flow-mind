# Phase 6 持久化、回放与 MVP 验收需求

---
status: active
updated: 2026-06-14
---

> 本文档定义 Phase 6（持久化、回放与 MVP 总体验收）的产品需求。Phase 6 的目标是让 fpga-flow-mind 在 Phase 1~5 产生的所有系统内产物可持久化、可重新加载、可回放，并完成 MVP 级总体验收闭环。
>
> 本文档 status 为 active，是 Phase 6 编码的实施依据之一。

## 1. 用户目标

- 用户关闭并重新打开应用后，能自动或手动恢复上次的 workspace 分析状态，无需重新扫描、重新收集证据、重新生成理解。
- 用户能在多个 workspace 之间切换，并查看最近打开/分析过的项目列表。
- 用户在目标项目未变更时，加载产物后可直接继续查看视图、追溯证据、进行 grounded 问答。
- 用户在目标项目已变更时，能明确收到提示，并自主选择重新分析或仅查看历史产物。
- MVP 最终能通过端到端验收，确认主链路、安全边界、目标项目只读约束全部满足。

## 2. 业务背景

Phase 1~5 已完成 workspace 扫描、evidence 收集、结构化理解、三类视图、证据追溯与 grounded Q&A。这些产物目前仅存于运行时内存中，应用关闭后丢失。Phase 6 需要把产物持久化到 app-owned storage，使用户可以跨会话继续工作，同时为 MVP 最终验收提供可重复的测试基线。

## 3. 功能点

### P6-001 自动保存当前 session

| 维度 | 说明 |
|------|------|
| **输入** | 当前 workspace root_path、WorkspaceProfile、各阶段 StageContext、EvidenceCollection、ImplementationUnderstanding、ViewGraph[]、选中的阶段、Trace 状态、Q&A 历史 |
| **输出** | 持久化到 app-owned storage 的 session 目录与 manifest 文件 |
| **前端责任** | 在关键状态变更后调用保存命令；展示保存中/保存失败状态 |
| **后端责任** | 原子写入 session manifest 与各 artifact 文件；返回 session_id 和保存结果 |
| **验收标准** | 关闭应用前能成功保存；重新打开后能找到最近一次 session |
| **非目标** | 不支持手动保存按钮作为唯一入口；不保存完整源码副本 |

### P6-002 列出最近 session

| 维度 | 说明 |
|------|------|
| **输入** | app-owned storage 中的 session 目录列表 |
| **输出** | 最近 session 列表，含项目名称、workspace root_path、最后更新时间、阶段数、是否可加载 |
| **前端责任** | 在 WorkspacePage 或入口区域展示最近项目列表；支持点击加载 |
| **后端责任** | 扫描 storage 目录，读取各 session manifest，过滤损坏/不可读项 |
| **验收标准** | 列表展示最近至少 10 个 session；损坏 session 显示降级状态但不阻塞 |
| **非目标** | 不做复杂搜索、排序、标签、收藏 |

### P6-003 重新加载上次 session

| 维度 | 说明 |
|------|------|
| **输入** | 用户选择最近 session 或应用启动时自动检测 |
| **输出** | 恢复 WorkspaceProfile、StageContext、EvidenceCollection、ImplementationUnderstanding、ViewGraph[]、选中阶段、Trace/Q&A 状态 |
| **前端责任** | 加载成功后恢复页面状态；加载失败时展示错误与可操作建议；目标项目变更/缺失/不安全时展示可恢复加载选项 |
| **后端责任** | 读取 manifest 与 artifact 文件；校验 schema 版本；校验 workspace fingerprint；**对 fingerprint mismatch、目标路径不存在、目标路径不安全返回可恢复加载状态（`success=true`，`status` 字段），仍携带 `session_state`** |
| **验收标准** | 加载后用户可直接查看上次分析的视图、evidence、问答历史，无需重新生成；**目标项目变更时仍允许“仅查看历史产物”** |
| **非目标** | 不自动重新扫描目标项目；不自动重新调用 MockProvider |

### P6-004 目标项目变更检测

| 维度 | 说明 |
|------|------|
| **输入** | session 中记录的 workspace_fingerprint / checksum 与当前目标项目实际状态 |
| **输出** | 加载结果状态：`source_unchanged`（未变更，正常恢复）、`source_changed`（项目已变更，可查看历史产物或重新分析）、`source_missing`（目标路径不存在，可重新选择或删除）、`source_path_not_allowed`（目标路径不安全，可删除）|
| **前端责任** | 以非阻塞方式展示变更提示；根据 `status` 提供“重新分析”“仅查看历史产物”“重新选择路径”“删除记录”选项 |
| **后端责任** | 计算目标项目关键文件的 checksum；与 manifest 中记录比对；返回明确的 `status` 和可选的 `mismatch_reason` |
| **验收标准** | 目标项目变更时用户收到明确提示；未变更时静默加载 |
| **非目标** | 不做实时文件监控；不自动重新分析 |

### P6-005 产物版本与 schema 兼容性

| 维度 | 说明 |
|------|------|
| **输入** | 持久化文件中的 `storage_version` / `artifact_version` |
| **输出** | 加载成功、或版本不兼容错误 |
| **前端责任** | 展示版本不兼容错误及建议 |
| **后端责任** | 校验版本号；对已知小版本提供最小迁移；对不兼容版本拒绝加载 |
| **验收标准** | 同版本加载成功；不兼容版本明确拒绝并提示 |
| **非目标** | 不做复杂版本迁移框架；不支持跨大版本自动迁移 |

### P6-006 产物清理与用户可控删除

| 维度 | 说明 |
|------|------|
| **输入** | 用户选择删除某个 session |
| **输出** | 删除 app-owned storage 中对应 session 目录 |
| **前端责任** | 提供删除入口与二次确认；展示删除结果 |
| **后端责任** | 安全删除 session 目录；拒绝删除非 app-owned 路径 |
| **验收标准** | 删除后 session 不再出现在最近列表；目标项目不受影响 |
| **非目标** | 不做自动清理策略；不做云端同步删除 |

### P6-007 会话级 trace/Q&A 状态恢复

| 维度 | 说明 |
|------|------|
| **输入** | session 中可选保存的 selected_trace_target、resolved_traces、source_excerpt、grounded_answer 历史 |
| **输出** | 加载后恢复用户上次的追溯与问答上下文 |
| **前端责任** | 恢复 UI 状态：选中节点、TracePanel、SourceExcerptPanel、问答区域 |
| **后端责任** | 将这些 UI 状态作为 artifact 保存与加载；校验其引用的 evidence/claim 仍存在于当前产物中 |
| **验收标准** | 加载后用户可继续查看上次的 trace 和问答 |
| **非目标** | 不保存 UI 滚动位置、窗口大小等纯视图状态 |

### P6-008 MVP 总体验收闭环

| 维度 | 说明 |
|------|------|
| **输入** | Phase 1~6 全部能力 + 样例验收项目 |
| **输出** | MVP 验收报告：功能、测试、安全、只读验证 |
| **前端责任** | 支持验收所需全部 UI 操作 |
| **后端责任** | 支持验收所需全部命令与持久化能力 |
| **验收标准** | 端到端场景通过；Rust 测试 ≥ 350 passed；前端构建通过；安全 rg 检查通过；目标项目只读 checksum 通过 |
| **非目标** | 不做 MVP 之后的扩展；不做正式发布流程 |

## 4. 异常 / 空状态

| 场景 | 处理 |
|------|------|
| 首次启动无 session | 展示“打开项目”入口，不报错 |
| session manifest 损坏 | 标记为损坏，允许删除，不允许加载 |
| workspace root 路径不存在 | 返回可恢复加载状态 `status=source_missing`，提示目标项目已移动或删除；提供“重新选择路径”“仅查看历史产物”“删除 session” |
| workspace root 变为 symlink | 返回可恢复加载状态 `status=source_path_not_allowed`，提示安全风险；提供“删除 session” |
| 目标项目 checksum mismatch | 返回可恢复加载状态 `status=source_changed`，提示项目已变更；提供“仅查看历史产物”或“重新分析” |
| schema 版本不兼容 | 阻塞错误：标记为不兼容，允许删除，不允许加载 |
| 存储空间不足 | 保存失败；前端展示错误，不丢失内存中状态 |

## 5. 证据与追溯要求

- 持久化产物中的 evidence_id、claim_id、source_path、line_range 必须与原始产物一致。
- 加载后 evidence 回链、source excerpt、Q&A citation 必须仍可追溯到目标项目源码。
- 不允许在持久化过程中伪造或修改 evidence 绑定。

## 6. MVP 验收标准

- P6-001~P6-008 全部实现并通过单元/集成测试。
- `save_session`、`load_session`、`list_sessions`、`delete_session` 命令稳定可用。
- 真实 Tauri 桌面验收覆盖：打开项目 → 分析 → 关闭 → 重新打开 → 恢复状态 → 目标项目变更检测 → 重新分析。
- 目标项目只读：验收前后 checksum 一致。

## 7. 非目标

- 云同步、多用户协作、数据库服务。
- 真实 LLM 接入。
- 自动运行 Vivado / synthesis / implementation / bitstream。
- 写回目标项目。
- 复杂全文搜索、项目审计 PASS/HOLD。
- Phase 7+ 能力。

## 8. 关联设计文档

- [`../design/phase-6-persistence-model.md`](../design/phase-6-persistence-model.md) — 数据模型
- [`../design/phase-6-persistence-and-replay-design.md`](../design/phase-6-persistence-and-replay-design.md) — 后端设计
- [`../ui-ux/phase-6-session-and-mvp-view.md`](../ui-ux/phase-6-session-and-mvp-view.md) — UI/UX 设计
- [`../testing/phase-6-mvp-validation.md`](../testing/phase-6-mvp-validation.md) — 测试与验收
- [`../planning/phase-6-implementation-plan.md`](../planning/phase-6-implementation-plan.md) — 实施计划

## 9. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-14 | 初始 draft：定义 P6-001~P6-008、验收标准、非目标、异常处理 | Claude |
