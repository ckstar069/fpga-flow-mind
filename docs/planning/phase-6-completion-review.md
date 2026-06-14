# Phase 6 完成审查

---
status: active
updated: 2026-06-15
---

> 本文档是 Phase 6（持久化、回放与 MVP 总体验收）的完成审查。P6-T01~P6-T11 已完成，真实桌面验收已通过，checksum 只读验证通过，允许 Phase 6 / MVP completion。

## 1. 任务完成状态

| 任务 | 目标 | 状态 | 验证方式 |
|------|------|------|----------|
| P6-T01 | Phase 6 Rust 数据模型 | ✅ | `cargo test --lib persistence::models` |
| P6-T02 | StorageVersionService | ✅ | `cargo test --lib persistence::storage_version` |
| P6-T03 | WorkspaceFingerprintService | ✅ | `cargo test --lib persistence::fingerprint_service` |
| P6-T04 | ArtifactRepository | ✅ | `cargo test --lib persistence::artifact_repository` |
| P6-T05 | SessionManifestRepository | ✅ | `cargo test --lib persistence::manifest_repository` |
| P6-T06 | SessionStore | ✅ | `cargo test --lib persistence::session_store` |
| P6-T07 | Phase 6 Tauri commands | ✅ | `cargo test --lib commands::` |
| P6-T08 | 前端 TypeScript 类型 + command 调用 | ✅ | `npm run build` |
| P6-T09 | 前端 Session 管理与状态恢复 | ✅ | `npm run build` + 桌面验收 |
| P6-T10 | Batch D 审核收口（自动保存竞态、ui_states 恢复、QA history 真实问题、类型对齐） | ✅ | `npm run build` + `cargo test --lib` |
| P6-T11 | MVP 总体验收与 completion review 收口 | ✅ | 真实桌面验收 15 步 + checksum 对比 |

## 2. 测试结果

### 2.1 前端构建

```bash
npm run build
```

结果：**通过**，TypeScript 编译无错误，Vite 生产构建成功。

### 2.2 Rust 测试

```bash
cd src-tauri && cargo test --lib
cd src-tauri && cargo check
```

结果：

```text
test result: ok. 411 passed; 0 failed
Finished `dev` profile [unoptimized + debuginfo] target(s)
```

## 3. 真实桌面验收

### 3.1 验收项目

临时样例项目路径：

```text
/tmp/fpga-flow-mind-phase6-acceptance-20260614-214944
├── L0/
│   ├── __init__.py
│   ├── top.py
│   └── utils.py
├── L1/
│   ├── controller.py
│   └── interface_l1_to_l0.py
├── L2/               (空目录)
├── rtl_final/
│   └── top.v
└── docs/
    └── README.md
```

验收前 checksum 基线：`/tmp/fpga-flow-mind-phase6-acceptance-20260614-214944/checksums.md`

### 3.2 15 步验收结果

| 步骤 | 验收内容 | 结果 |
|------|----------|------|
| 1 | 打开样例项目，workspace 概览、阶段列表（L0/L1/L2/RTL）、warnings 正常 | ✅ |
| 2 | L0 收集证据 → 生成理解 → 生成视图 | ✅ |
| 3 | 点击视图节点/边，TracePanel 展示 trace_refs | ✅ |
| 4 | 点击“查看源码片段”，SourceExcerptPanel 展示源码且不打开外部编辑器 | ✅ |
| 5 | 点击“定位 evidence”，EvidencePanel 高亮 | ✅ |
| 6 | 可回答问题“L0 counter 的位宽是多少？”返回带 citation 的回答（8 bit） | ✅ |
| 7 | 不可回答问题“这个模块的量子纠缠算法是什么？”返回 unknown/证据不足，无伪造 citation | ✅ |
| 8 | 手动保存 session，顶部状态变为“已保存”，最近项目列表出现记录 | ✅ |
| 9 | 切换到 L1，旧 L0 trace/Q&A/views 清空；对 L1 执行收集/理解/视图并保存 | ✅（用户口头确认） |
| 10 | 关闭重开 app，从最近项目加载 session，状态恢复 | ✅（用户口头确认） |
| 11 | 修改临时项目源文件后重新加载，出现 source_changed 可恢复提示 | ✅（用户口头确认） |
| 12 | L2 空阶段无收集/生成误入口或明确空状态 | ✅（L2 显示“为空”） |
| 13 | RTL 命名异常阶段可收集、理解、生成视图 | ✅ |
| 14 | 删除最近项目记录，只删除 app-owned session，不删除目标项目 | ✅（用户口头确认） |
| 15 | 重新计算 checksum，目标项目文件前后一致 | ✅ |

### 3.3 截图证据

用户已回传关键步骤截图：

- 图 1：L0 阶段已完成证据收集、理解生成、视图生成；TracePanel 展示 `module_Counter` trace；SourceExcerptPanel 展示 `top.py` 源码；最近项目列表已出现该 session；顶部显示“已保存”。
- 图 2：Grounded Q&A 可回答问题“L0 counter 的位宽是多少？”返回“位宽为 8 bit”，带 8 条 citation。
- 图 3：Grounded Q&A 不可回答问题“这个模块的量子纠缠算法是什么？”返回“未知”，提示 `evidence_gap`，无伪造 citation。
- 图 4：RTL 命名异常阶段可完成证据收集、理解生成、视图生成，并展示 trace 详情与源码片段。

### 3.4 checksum 只读验证

验收前后源文件 checksum 对比：

```bash
diff checksums.md checksums-recomputed.md
# Source files checksums MATCH
```

目标项目文件未被修改，只读验证通过。

## 4. Session 保存/加载/删除/恢复验证

### 4.1 后端集成测试

`persistence::session_store` 测试覆盖：

- `save_then_load_roundtrip`：保存后加载状态一致。
- `load_changed_source_returns_recoverable_status`：fingerprint 变化返回 `source_changed`。
- `load_missing_source_returns_recoverable_status`：root_path 不存在返回 `source_missing`。
- `load_unsafe_source_returns_recoverable_status`：root_path 变为 symlink 返回 `source_path_not_allowed`。
- `load_incompatible_version`：不兼容版本明确拒绝。
- `delete_session_removes_storage`：删除仅移除 app storage。
- `delete_session_rejects_symlink_session_dir_and_preserves_target`：拒绝 symlink 攻击并保护目标项目。

### 4.2 前端状态恢复

`WorkspacePage.tsx` 已实现：

- 手动保存 + 2s debounce 轻量自动保存。
- 最近项目列表加载、删除、空状态。
- 加载 session 后恢复 `stage_contexts/evidence_collections/understandings/view_graphs/qa_histories/ui_states`。
- 根据可用产物推断 phase（views_loaded → understanding_loaded → evidence_loaded → stage_loaded）。
- `source_changed` / `source_missing` / `source_path_not_allowed` 横幅提示。
- 阻塞错误（session_not_found / storage_version_incompatible / load_failed）显示错误卡片。
- `invalidatePendingSessionSave()` 防止旧 save 请求覆盖新 workspace/session 状态。

## 5. 安全边界确认

```bash
rg "OpenAI|Anthropic|api_key" src src-tauri/src        # 无匹配
rg "Command::new|std::process::Command" src src-tauri/src # 无匹配
rg "Vivado|synthesis|implementation|bitstream" src src-tauri/src
# 仅命中 src-tauri/src/understanding/models.rs 测试函数名 implementation_understanding_roundtrip（benign）
rg "PASS|HOLD|审计" src/features src/lib src/types docs/planning docs/README.md
# 仅出现在规划文档禁用列表、历史 completion review 结论、UI 错误码文案中，不作为当前用户可见结论输出
```

- 不修改目标项目源码。
- 不运行 Vivado/synthesis/implementation/bitstream。
- 不调用真实 LLM API。
- 持久化只写 app-owned storage。
- 拒绝 path traversal / symlink / root mismatch。
- 不保存完整源码副本。
- 不保存敏感环境变量。

## 6. 已知限制

1. **真实 LLM 未接入**：Grounded Q&A 当前为 MockProvider，基于关键词匹配生成回答；真实 LLM Provider 需在后续阶段显式配置并经过 GroundedQaValidator。
2. **自动保存策略轻量**：2s debounce，无复杂队列/后台 watcher/冲突合并。
3. **active_view_type 未持久化**：当前 UI 无集中 active view type 状态。
4. **提问文本集中保存**：QA history 使用 `handleAskGroundedQuestion` 的真实 `questionText`，但历史列表未在 UI 中展示，仅持久化。
5. **source_changed 模拟为手动触发**：本次验收通过手动修改临时项目源文件验证，未引入文件系统 watcher。

## 7. 结论

- P6-T01~P6-T11 全部完成。
- 真实桌面验收 15 步通过，关键步骤有截图佐证。
- 目标项目 checksum 验收前后一致，只读验证通过。
- 全量测试、构建、类型检查通过。
- 安全边界满足。
- **允许 Phase 6 / MVP completion。**

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-14 | 初始 draft：记录 Phase 6 任务状态、测试结果、安全边界、已知限制。明确真实交互式桌面验收未完成，completion review 不应标记为完成。 | Claude |
| 2026-06-15 | 更新为 active：用户完成 15 步真实桌面验收，checksum 只读验证通过，允许 Phase 6 / MVP completion。 | Claude |
