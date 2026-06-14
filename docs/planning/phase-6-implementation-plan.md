# Phase 6 编码实施计划

---
status: draft
updated: 2026-06-14
---

> 本文档定义 Phase 6（持久化、回放与 MVP 验收）的编码实施计划，包含任务拆解、依赖关系、Batch 划分、进入/退出条件、验收标准和安全边界。
>
> 本文档为 draft，仅供评审与讨论，不得作为 Phase 6 编码唯一依据。本轮修复后仍需审核并转为 active，方可进入 Phase 6 编码。

## 1. 进入条件

| 条件 | 状态 |
|------|------|
| Phase 5 completion review status 为 active | ✅ |
| Phase 5 真实 Tauri 桌面验收通过 | ✅ 9/9 |
| Phase 6 需求文档已创建 | ✅ `phase-6-persistence-and-mvp-requirements.md`（draft） |
| Phase 6 数据模型设计已创建 | ✅ `phase-6-persistence-model.md`（draft） |
| Phase 6 后端设计已创建 | ✅ `phase-6-persistence-and-replay-design.md`（draft） |
| Phase 6 UI/UX 文档已创建 | ✅ `phase-6-session-and-mvp-view.md`（draft） |
| Phase 6 测试文档已创建 | ✅ `phase-6-mvp-validation.md`（draft） |
| Phase 6 实施计划已创建 | ✅ 本文档（draft） |
| **以上文档修复并审核后全部转为 active** | ⏳ 当前未满足，不允许进入 Phase 6 编码 |

## 2. 任务拆分

### P6-T01 定义 Phase 6 Rust 数据模型

| 维度 | 说明 |
|------|------|
| **目标** | 在 `persistence/models.rs` 中定义 StorageVersion、SessionManifest、PersistedWorkspace、ArtifactIndex、PersistedStageArtifacts、QaHistory、PersistedUiState、GlobalUiState 等 |
| **输入文档** | `phase-6-persistence-model.md` |
| **预计修改文件** | `src-tauri/src/persistence/mod.rs`（新增）、`src-tauri/src/persistence/models.rs`（新增） |
| **验收命令** | `cargo test --lib persistence::models` |
| **不做什么** | 不实现 repository、command、UI |

### P6-T02 实现 StorageVersionService

| 维度 | 说明 |
|------|------|
| **目标** | 版本号比较、兼容/不兼容判断、serde round-trip |
| **输入文档** | `phase-6-persistence-model.md` §4、`phase-6-persistence-and-replay-design.md` §6 |
| **预计修改文件** | `src-tauri/src/persistence/version_service.rs`（新增） |
| **验收命令** | `cargo test --lib persistence::version_service` |
| **不做什么** | 不做复杂版本迁移框架 |

### P6-T03 实现 WorkspaceFingerprintService

| 维度 | 说明 |
|------|------|
| **目标** | 计算目标项目 fingerprint，检测变更，拒绝 symlink / 不存在路径 |
| **输入文档** | `phase-6-persistence-and-replay-design.md` §8 |
| **预计修改文件** | `src-tauri/src/persistence/fingerprint_service.rs`（新增） |
| **验收命令** | `cargo test --lib persistence::fingerprint_service` |
| **不做什么** | 不做实时文件监控 |

### P6-T04 实现 ArtifactRepository

| 维度 | 说明 |
|------|------|
| **目标** | artifact JSON 文件的原子写入与读取，路径安全校验 |
| **输入文档** | `phase-6-persistence-and-replay-design.md` §4、§5 |
| **预计修改文件** | `src-tauri/src/persistence/artifact_repository.rs`（新增） |
| **验收命令** | `cargo test --lib persistence::artifact_repository` |
| **不做什么** | 不写入目标项目；不保存完整源码副本 |

### P6-T05 实现 SessionManifestRepository

| 维度 | 说明 |
|------|------|
| **目标** | manifest.json 读写、损坏处理、路径安全 |
| **输入文档** | `phase-6-persistence-and-replay-design.md` §4 |
| **预计修改文件** | `src-tauri/src/persistence/manifest_repository.rs`（新增） |
| **验收命令** | `cargo test --lib persistence::manifest_repository` |
| **不做什么** | 不做自动修复损坏 manifest |

### P6-T06 实现 SessionStore

| 维度 | 说明 |
|------|------|
| **目标** | 整合 save_session / load_session / list_sessions / delete_session 高层逻辑；`load_session` 对 fingerprint mismatch / 路径不存在 / 路径不安全返回 `success=true` 的可恢复加载状态（`status` + `session_state`） |
| **输入文档** | `phase-6-persistence-and-replay-design.md` §2、§3 |
| **预计修改文件** | `src-tauri/src/persistence/session_store.rs`（新增） |
| **验收命令** | `cargo test --lib persistence::session_store` |
| **不做什么** | 不直接暴露为 Tauri command |

### P6-T07 实现 Phase 6 Tauri commands

| 维度 | 说明 |
|------|------|
| **目标** | `save_session`、`load_session`（返回 `LoadSessionResult` 含 `status` + `session_state`）、`list_sessions`、`delete_session`、`get_last_session` |
| **输入文档** | `phase-6-persistence-and-replay-design.md` §3 |
| **预计修改文件** | `src-tauri/src/commands/save_session.rs`、`load_session.rs`、`list_sessions.rs`、`delete_session.rs`、`get_last_session.rs` |
| **验收命令** | `cargo test --lib commands` |
| **不做什么** | command 不直接写入磁盘，委托 SessionStore |

### P6-T08 前端 TypeScript 类型 + command 调用

| 维度 | 说明 |
|------|------|
| **目标** | 扩展 `src/types/workspace.ts` + `src/lib/tauriCommands.ts` |
| **输入文档** | `phase-6-persistence-model.md` |
| **验收命令** | `npm run build` |
| **不做什么** | 不实现 UI 组件 |

### P6-T09 前端 Session 管理与状态恢复

| 维度 | 说明 |
|------|------|
| **目标** | 最近项目列表、保存状态、加载入口、加载失败/版本不兼容/目标路径不存在/变更提示 |
| **输入文档** | `phase-6-session-and-mvp-view.md` |
| **预计修改文件** | `src/features/workspace/WorkspacePage.tsx`、`components/RecentProjectsPanel.tsx`、顶部标题栏 |
| **验收命令** | `npm run build` + 桌面验收 |
| **不做什么** | 不做 landing page、不做复杂 dashboard |

### P6-T10 MVP 总体验收与文档同步

| 维度 | 说明 |
|------|------|
| **目标** | 全量验证、文档状态更新、完成审查 |
| **输入文档** | `phase-6-mvp-validation.md` |
| **预计修改文件** | `docs/planning/phase-6-completion-review.md`（新增）、各 index 更新 |
| **验收命令** | 全量测试 + rg 检查 + 桌面验收 + checksum |
| **不做什么** | 不进入 Phase 7+ 编码 |

## 3. 依赖关系

```text
P6-T01 (models)
  │
  ├── P6-T02 (StorageVersionService)
  ├── P6-T03 (WorkspaceFingerprintService)
  │
  ├── P6-T04 (ArtifactRepository)
  ├── P6-T05 (SessionManifestRepository)
  │     │
  │     ▼
  ├── P6-T06 (SessionStore)
  │     │
  │     ▼
  ├── P6-T07 (Tauri commands)
  │     │
  │     ▼
  ├── P6-T08 (TS types/commands)
  │     │
  │     ▼
  ├── P6-T09 (Session UI)
  │     │
  │     ▼
  ├── P6-T10 (验收与 completion review)
```

## 4. Batch 划分

### 4.1 Batch A：数据模型 + 版本 + fingerprint

| 任务 | 内容 |
|------|------|
| P6-T01 | Phase 6 Rust 数据模型 |
| P6-T02 | StorageVersionService |
| P6-T03 | WorkspaceFingerprintService |

**预估测试**：18 个（model 4 + version 6 + fingerprint 8）。

### 4.2 Batch B：Repository + SessionStore

| 任务 | 内容 |
|------|------|
| P6-T04 | ArtifactRepository |
| P6-T05 | SessionManifestRepository |
| P6-T06 | SessionStore |

**预估测试**：24 个（artifact 8 + manifest 6 + session_store 10）。

### 4.3 Batch C：Tauri commands + TS 类型

| 任务 | 内容 |
|------|------|
| P6-T07 | Phase 6 Tauri commands |
| P6-T08 | 前端 TypeScript 类型 + command 调用 |

**预估测试**：18 个（command 4+6+4+4+?）。

### 4.4 Batch D：前端 Session UI

| 任务 | 内容 |
|------|------|
| P6-T09 | 前端 Session 管理与状态恢复 |

**验证**：`npm run build` + 桌面验收（步骤 1~6）。

### 4.5 Batch E：MVP 总体验收 + completion review

| 任务 | 内容 |
|------|------|
| P6-T10 | 全量验证、文档同步、完成审查 |

**验证**：全量测试 + rg + 桌面验收（步骤 1~12）+ checksum。

## 5. 退出条件

| 条件 | 验证方式 |
|------|----------|
| Rust 全量测试通过 | `cargo test --lib` |
| 前端构建通过 | `npm run build` |
| save_session 原子写入 | 单元测试 |
| load_session 恢复状态 | 单元测试 + 桌面验收；对 fingerprint mismatch / 路径缺失 / 路径不安全返回 `success=true` 的可恢复 `status`，仍携带 `session_state` |
| list_sessions 正确排序过滤 | 单元测试 |
| delete_session 不影响目标项目 | 单元测试 + checksum |
| fingerprint 检测变更 | 单元测试 + 桌面验收 |
| 版本不兼容明确拒绝 | 单元测试 |
| 桌面验收 12/12 通过 | 桌面验收 |
| 目标项目只读 | rg + checksum |
| 无真实 LLM 默认调用 | rg |
| 无 PASS/HOLD 审计用语 | rg |
| Phase 6 completion review 完成 | 文档 |

## 6. 安全边界

- 不修改 `fpga_project_*`。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM API。
- 持久化只写 app-owned storage，不写目标 workspace。
- 拒绝 path traversal / symlink / root mismatch。
- 不保存完整源码副本；如缓存 excerpt，必须限定范围并提供清理策略。
- 不保存敏感环境变量。
- 不输出 PASS/HOLD/正确/错误等审计结论。

## 7. 进入 Phase 7 的条件（预留）

- Phase 6 completion review status 为 active。
- Phase 6 真实 Tauri 桌面验收通过。
- 全量测试通过。
- 安全约束满足。
- Phase 7 需求/设计/计划文档 active 后（如有）方可进入。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-14 | 初始 draft：定义 P6-T01~P6-T10、5 个 Batch、退出条件、安全边界、进入 Phase 7 条件 | Claude |
