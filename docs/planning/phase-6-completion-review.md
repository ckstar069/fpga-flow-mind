# Phase 6 完成审查

---
status: active
updated: 2026-06-14
---

> 本文档是 Phase 6（持久化、回放与 MVP 总体验收）的完成审查。记录 P6-T01~P6-T11 完成状态、测试结果、桌面验收结果、安全边界、已知限制，并给出是否允许 MVP completion / Phase 6 exit 的结论。

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
| P6-T09 | 前端 Session 管理与状态恢复 | ✅ | `npm run build` + 代码路径审查 |
| P6-T10 | Batch D 审核收口（自动保存竞态、ui_states 恢复、QA history 真实问题、类型对齐） | ✅ | `npm run build` + `cargo test --lib` |
| P6-T11 | MVP 总体验收与 completion review 收口 | ⚠️ 部分受限 | 文档完成；真实交互式桌面验收受环境限制 |

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

## 3. 桌面验收

### 3.1 验收项目准备

已创建自包含临时样例项目：

```text
/tmp/fpga-flow-mind-phase6-acceptance-20260614-211345
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

初始 checksum 已记录在 `/tmp/fpga-flow-mind-phase6-acceptance-20260614-211345/checksums.md`。

### 3.2 Tauri app 启动尝试

执行：

```bash
npm run tauri dev -- --no-watch
```

结果：Vite dev server 启动（http://localhost:1420/），Tauri Rust 端编译成功。由于当前 CLI 环境无法长时间保持交互式桌面窗口运行，未能在本会话内完成 15 步人工点击验收。app 本身可编译启动，无启动错误。

### 3.3 已完成的非交互式验收

| 验收项 | 方式 | 结果 |
|--------|------|------|
| 样例项目结构完整 | 脚本生成 + 文件检查 | ✅ |
| checksum 只读基线 | `shasum -a 256` | ✅ |
| 前端构建 | `npm run build` | ✅ |
| Rust 全量测试 | `cargo test --lib` | ✅ 411 passed |
| Rust 类型检查 | `cargo check` | ✅ |
| 后端 save/load/list/delete 集成 | 单元测试 | ✅ |
| fingerprint 变更/缺失/不安全路径检测 | 单元测试 | ✅ |
| 版本不兼容拒绝 | 单元测试 | ✅ |
| 目标项目只读（代码层面） | rg + 代码审查 | ✅ |

### 3.4 待用户在真实桌面环境补做的验收清单

以下步骤需在可交互的桌面会话中完成：

1. 打开样例项目 `/tmp/fpga-flow-mind-phase6-acceptance-20260614-211345`，确认 workspace 概览、阶段列表、warnings 正常。
2. 在 L0 执行：收集证据 → 生成理解 → 生成视图。
3. 点击视图节点/边，确认 TracePanel 展示 trace_refs。
4. 点击“查看源码片段”，确认 SourceExcerptPanel 展示源码且不打开外部编辑器。
5. 点击“定位 evidence”，确认 EvidencePanel 高亮。
6. 提问可回答问题，确认 Grounded Q&A 返回带 citation 的回答。
7. 提问证据不足问题，确认返回 unknown/证据不足，不伪造 citation。
8. 手动保存 session，确认保存状态变为“已保存”，最近项目列表出现记录。
9. 切换到 L1，确认旧 L0 trace/Q&A/views 状态清空；对 L1 执行收集/理解/视图，并保存。
10. 关闭并重新打开 app，从最近项目加载 session，确认 root_path、selected_stage_id、stage_context、evidence、understanding、views、trace/excerpt/qa/ui_state 恢复。
11. 修改临时项目中的一个源文件（副本），确认加载时出现 source_changed 可恢复提示。
12. 验证 L2 空阶段无收集/生成误入口或明确空状态。
13. 验证 rtl_final Verilog 阶段可收集、理解、生成视图。
14. 删除最近项目记录，确认只删除 app-owned session，不删除目标项目。
15. 重新计算 checksum，确认目标项目文件前后一致。

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
5. **交互式桌面验收受环境限制**：本 CLI 环境无法完成 15 步人工点击验收，已提供可复现样例项目和清单供用户在真实桌面环境补做。
6. **source_changed 模拟需用户手动触发**：临时项目 checksum 基线已建立，验收时可通过修改临时项目源文件后重新加载来验证。

## 7. 结论

- P6-T01~P6-T10 已完成并通过自动化验证。
- P6-T11 完成审查文档已产出，但真实交互式桌面验收因环境限制未能全部执行。
- 代码层面、测试层面、安全检查层面均满足 Phase 6 退出条件。
- **建议允许 Phase 6 / MVP completion，但需在真实桌面环境中补做 15 步验收清单并确认 checksum 前后一致。**

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-14 | 初始 active：记录 Phase 6 任务状态、测试结果、安全边界、已知限制、completion 结论 | Claude |
