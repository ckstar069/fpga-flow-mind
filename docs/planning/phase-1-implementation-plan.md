# Phase 1 Workspace 扫描与阶段识别实施计划

---
status: active
updated: 2026-06-11
---

> 本文档是 Phase 1 的实施计划，将已完成的需求、设计、UI/UX、验证文档转化为后续编码阶段可执行的任务顺序、允许修改范围、验证顺序和退出门槛。
> Phase 1 指 `fpga-flow-mind` **本项目**的开发推进阶段，不是业务项目的 `L0` / `L1` / `RTL` 实现阶段。
> 本计划不创建产品源码或测试代码。

## 1. 阶段目标

Phase 1 编码完成后，产品应能：

- 打开本地 workspace（通过 Tauri 文件选择器）
- 执行路径校验（存在性、目录性、可读性、非 symlink 根路径）
- 安全处理 symlink（根路径拒绝，扫描中跳过）
- 只读扫描目标项目目录，不修改任何文件
- 识别候选阶段目录（标准阶段、变体、命名异常、空、不可读）
- 计算文件类型统计、外部模块引用、validity
- 生成并展示 `WorkspaceProfile`
- 支持 `select_stage` 生成 `StageContext`
- 前端展示 workspace 概览、阶段列表、warnings/errors、阶段概览
- "开始分析"保持禁用或占位，不触发 evidence 收集

Phase 1 **不解决**：
- evidence 收集、索引、存储（Phase 2）
- 大模型调用、语义理解（Phase 3）
- 结构图 / 数据流图 / 时序图（Phase 4）
- grounded Q&A（Phase 5）
- 持久化、回放（Phase 6）
- JSON viewer、Markdown report viewer

## 2. 输入文档清单

| 输入文档 | 定位 | 实施中用途 |
|---------|------|-----------|
| [`docs/requirements/mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) | `active` | 枚举值、错误码、字段语义、validity 规则的唯一权威来源 |
| [`docs/requirements/stories/story-open-workspace.md`](../requirements/stories/story-open-workspace.md) | `draft` | WS-001~007 用户故事与验收标准 |
| [`docs/requirements/stories/story-select-stage.md`](../requirements/stories/story-select-stage.md) | `draft` | ST-001~008 用户故事与验收标准 |
| [`docs/design/workspace-scanning-and-stage-detection.md`](../design/workspace-scanning-and-stage-detection.md) | `active` | Phase 1 技术入口：扫描范围、安全约束、边界说明 |
| [`docs/design/phase-1-architecture.md`](../design/phase-1-architecture.md) | `active` | 模块划分、数据流、前后端职责边界 |
| [`docs/design/phase-1-data-and-api-contract.md`](../design/phase-1-data-and-api-contract.md) | `active` | `CommandResult` 语义、`WorkspaceProfile`/`StageContext` 字段、Tauri command 签名、UI 状态映射 |
| [`docs/design/phase-1-scanner-detail-design.md`](../design/phase-1-scanner-detail-design.md) | `active` | DFS 扫描算法、阶段识别、文件分类、validity 判定、边界条件、error_code 作用域 |
| [`docs/ui-ux/phase-1-workspace-and-stage-flow.md`](../ui-ux/phase-1-workspace-and-stage-flow.md) | `active` | 组件定义、状态展示规则、空状态处理、不展示 JSON 规则 |
| [`docs/testing/phase-1-workspace-scanning-validation.md`](../testing/phase-1-workspace-scanning-validation.md) | `active` | 验证矩阵、验收 checklist、测试夹具设计、自动化测试规划 |

**文档权威优先级**（冲突时按序号从高到低裁决）：

1. [`docs/requirements/mvp-functional-contract.md`](../requirements/mvp-functional-contract.md) — `active` 功能契约，最高优先级。定义枚举值、错误码、字段语义、validity 规则。
2. [`docs/design/phase-1-data-and-api-contract.md`](../design/phase-1-data-and-api-contract.md) — Phase 1 数据/API 契约。定义 `CommandResult` 语义、`WorkspaceProfile`/`StageContext` 字段、Tauri command 签名、UI 状态映射、error/warning 作用域。
3. [`docs/design/phase-1-scanner-detail-design.md`](../design/phase-1-scanner-detail-design.md) — Phase 1 扫描详细设计。定义 DFS 算法、阶段识别、文件分类、validity 判定算法、边界条件。
4. [`docs/design/phase-1-architecture.md`](../design/phase-1-architecture.md) — Phase 1 概要设计。定义模块职责、数据流、前后端边界。
5. [`docs/ui-ux/phase-1-workspace-and-stage-flow.md`](../ui-ux/phase-1-workspace-and-stage-flow.md) — Phase 1 UI/UX 设计。定义 UI 展示和交互规则，**不反向修改数据契约**。
6. [`docs/testing/phase-1-workspace-scanning-validation.md`](../testing/phase-1-workspace-scanning-validation.md) — Phase 1 验证设计。定义验收标准，**不定义新功能或新契约**。
7. [`docs/planning/phase-1-implementation-plan.md`](phase-1-implementation-plan.md) — Phase 1 实施计划。定义任务顺序和退出标准，**不定义新需求或新契约**。

**冲突处理规则**：
- 当高优先级文档与低优先级文档冲突时，以高优先级为准，并同步修正低优先级文档。
- 当同级文档之间冲突时（如两个同级设计文档互相矛盾），**必须暂停编码**，先修正文档并经审核，再继续实施。
- 编码阶段发现契约漏洞时，应暂停编码、更新契约（从高优先级开始）、经审核后再继续。

## 3. 允许修改范围

Phase 1 编码阶段允许创建/修改以下范围（本轮不创建）：

### Rust / Tauri 后端

- `src-tauri/Cargo.toml` — 依赖声明
- `src-tauri/src/lib.rs` — Tauri app 入口、command 注册
- `src-tauri/src/commands/open_workspace.rs` — `open_workspace` Tauri command
- `src-tauri/src/commands/select_stage.rs` — `select_stage` Tauri command
- `src-tauri/src/workspace/scanner.rs` — `workspace_scanner` 模块
- `src-tauri/src/workspace/stage_detector.rs` — `stage_detector` 模块
- `src-tauri/src/workspace/file_classifier.rs` — `file_classifier` 模块
- `src-tauri/src/workspace/external_refs.rs` — `external_ref_detector` 模块
- `src-tauri/src/workspace/validity.rs` — `validity_calculator` 模块
- `src-tauri/src/workspace/safety_guard.rs` — `safety_guard` 模块
- `src-tauri/src/models/workspace_profile.rs` — `WorkspaceProfile` struct
- `src-tauri/src/models/stage_context.rs` — `StageContext` struct
- `src-tauri/src/models/enums.rs` — `WorkspaceValidity`、`StageStatus`、`ErrorCode` 枚举
- `src-tauri/src/models/error.rs` — `CommandError`、`CommandResult` struct

### React / TypeScript 前端

- `src/features/workspace/WorkspacePage.tsx` — 主页面
- `src/features/workspace/StagePanel.tsx` — 阶段列表
- `src/features/workspace/StageOverview.tsx` — 阶段概览
- `src/features/workspace/WarningList.tsx` — 警告列表
- `src/features/workspace/ErrorBanner.tsx` — 错误横幅
- `src/features/workspace/OpenWorkspaceButton.tsx` — 打开项目按钮
- `src/features/workspace/WorkspaceSummary.tsx` — workspace 摘要
- `src/features/workspace/FileTypeStats.tsx` — 文件类型统计
- `src/features/workspace/StatusBar.tsx` — 状态栏
- `src/features/workspace/AnalysisPlaceholderButton.tsx` — 占位按钮
- `src/features/workspace/hooks/` — React hooks（调用 Tauri command）
- `src/types/workspace.ts` — TypeScript interface 与 Rust 契约对应

### 测试

- `src-tauri/src/workspace/scanner_test.rs` — scanner 单元测试
- `src-tauri/src/workspace/stage_detector_test.rs` — stage detector 单元测试
- `src-tauri/src/workspace/file_classifier_test.rs` — file classifier 单元测试
- `src-tauri/tests/open_workspace.rs` — `open_workspace` 集成测试
- `src-tauri/tests/select_stage.rs` — `select_stage` 集成测试
- `src-tauri/tests/fixtures/` — 测试夹具目录（临时构造）
- `src/features/workspace/__tests__/` — 前端组件测试
- `e2e/` — Playwright UI smoke 测试

### 构建与配置

- `package.json` — 前端依赖
- `vite.config.ts` — 前端构建配置
- `tsconfig.json` — TypeScript 配置
- `.claude/settings.json` — 项目级 Claude Code 设置（如需）

### 文档索引

- 各子目录 `README.md` — 必要时更新索引

## 4. 禁止事项

Phase 1 编码阶段明确**不**做：

- **不修改目标业务项目**：不创建 / 修改 / 删除 `fpga_project_*` 目录下的任何文件
- **不写入 fpga_project_* **：所有输出仅进入 app-owned 目录或临时目录
- **不运行 Vivado / synthesis / implementation / bitstream**
- **不运行目标项目脚本**：不自动执行 `.py`、`.sh`、`.tcl` 等
- **不调用大模型做语义理解**：Phase 1 不调用 LLM、不设计 provider 调用
- **不做 evidence 收集**：不提取代码片段、不建立 evidence index
- **不做结构图 / 数据流图 / 时序图**
- **不做 Q&A**：无问答面板、无追问交互
- **不做持久化回放**：不保存历史 workspace、不支持加载旧 session
- **不做 JSON viewer 或 Markdown report viewer**
- **不提前实现 Phase 2~6 功能**：组件可预留扩展空间，但不实现后续功能

## 5. 实施任务拆解

按**先类型契约，再 backend scanner，再 command 边界，再 frontend UI，再测试补齐**的顺序执行。

每个任务完成后应小步 commit，commit message 遵循 `scope: description` 格式。

### P1-T01 初始化 Tauri v2 + React/TypeScript 项目骨架

- **目标**：创建可运行的 Tauri v2 + React + TypeScript 桌面应用骨架
- **预计修改**：项目根目录 `package.json`、`vite.config.ts`、`tsconfig.json`；`src-tauri/Cargo.toml`、`src-tauri/src/main.rs`；前端 `src/main.tsx`、`src/App.tsx`
- **依赖前置**：无
- **验证方式**：`cargo tauri dev` 能启动空白窗口，无报错
- **不做什么**：不实现任何业务逻辑、不添加 Phase 2~6 依赖

### P1-T02 建立 Rust/TypeScript 数据类型与 CommandResult 契约

- **目标**：定义 `CommandResult<T>`、`WorkspaceProfile`、`StageContext`、`WorkspaceValidity`、`StageStatus`、`ErrorCode` 等枚举和 struct，Rust 侧用 `serde` 序列化，TypeScript 侧定义对应 interface
- **预计修改**：`src-tauri/src/models/`（全部 model 文件）；`src/types/workspace.ts`
- **依赖前置**：P1-T01
- **验证方式**：Rust 侧 `cargo check` 通过；TypeScript 侧 `tsc --noEmit` 通过；两端字段名称、类型、可选性一致
- **不做什么**：不实现业务逻辑、不添加 Phase 2~6 的 model 字段

### P1-T03 实现只读路径校验与 symlink root 拒绝

- **目标**：实现 `safety_guard` 模块，覆盖路径存在性、目录性、可读性、非 symlink 根路径校验；根路径为 symlink 时拒绝并映射为 `permission_denied`
- **预计修改**：`src-tauri/src/workspace/safety_guard.rs`；对应单元测试
- **依赖前置**：P1-T02
- **验证方式**：单元测试覆盖：存在目录通过、不存在映射 `path_not_found`、文件映射 `not_directory`、symlink 根映射 `permission_denied`、权限不足映射 `permission_denied`
- **不做什么**：不实现扫描逻辑、不处理扫描中 symlink（见 P1-T04）

### P1-T04 实现 workspace 扫描与文件分类

- **目标**：实现 `workspace_scanner` 模块（DFS，深度 ≤3，单目录 ≤1000，总数 ≤5000，超时 30s）和 `file_classifier` 模块（扩展名映射、测试文件模式、二进制跳过、大文件仅读前 100 行）
- **预计修改**：`src-tauri/src/workspace/scanner.rs`；`src-tauri/src/workspace/file_classifier.rs`；对应单元测试
- **依赖前置**：P1-T03
- **验证方式**：单元测试覆盖：标准项目扫描正确、深度超限跳过、单目录超限 warning、超时返回部分结果、二进制跳过、大文件仅读前 100 行、测试文件正确归类
- **不做什么**：不实现阶段识别（见 P1-T05）、不做外部引用识别（见 P1-T05）

### P1-T05 实现阶段识别、排序、validity、missing/warnings 规则

- **目标**：实现 `stage_detector`（阶段目录识别、命名异常检测、空阶段检测、不可读阶段检测、排序）、`external_ref_detector`（字符串匹配识别 `urban_wireless`）、`validity_calculator`（validity 判定）
- **预计修改**：`src-tauri/src/workspace/stage_detector.rs`；`src-tauri/src/workspace/external_refs.rs`；`src-tauri/src/workspace/validity.rs`；对应单元测试
- **依赖前置**：P1-T04
- **验证方式**：单元测试覆盖：标准阶段识别正确、变体映射正确、命名异常标记正确、空阶段标记正确、不可读阶段标记正确、排序正确、缺失阶段不插入 `stages[]`、validity 判定正确
- **不做什么**：不实现 `select_stage`（见 P1-T06）

### P1-T06 实现 select_stage 与 StageContext

- **目标**：实现 `select_stage` Tauri command，构造阶段路径、验证存在性与可读性、扫描阶段文件（深度 2）、生成 `StageContext`；`stage_empty` 时返回 `success=true` + 空 `files[]` + `error_code = stage_empty`；`stage_unreadable` 时返回 `success=false` + `CommandError`
- **预计修改**：`src-tauri/src/commands/select_stage.rs`；集成测试
- **依赖前置**：P1-T05
- **验证方式**：集成测试覆盖：可用阶段返回正确 `StageContext`、空阶段返回 `stage_empty`、不可读阶段返回 `stage_unreadable`、上游引用推断正确
- **不做什么**：不进入 evidence 收集、不调用大模型

### P1-T07 实现前端状态管理和 Tauri command 调用

- **目标**：实现 React hooks（`useOpenWorkspace`、`useSelectStage`），管理 loading/success/error/empty 状态，调用 Tauri `invoke`，处理 `CommandResult` 响应
- **预计修改**：`src/features/workspace/hooks/`；`src/App.tsx`（或状态管理入口）
- **依赖前置**：P1-T02、P1-T06
- **验证方式**：组件测试覆盖：loading 状态展示、success 状态更新、error 状态展示、空阶段状态处理
- **不做什么**：不实现具体 UI 组件（见 P1-T08~T10）

### P1-T08 实现 workspace 概览 UI

- **目标**：实现 `WorkspacePage`、`WorkspaceSummary`、`FileTypeStats`、`OpenWorkspaceButton`、`StatusBar`
- **预计修改**：`src/features/workspace/WorkspacePage.tsx`；`src/features/workspace/WorkspaceSummary.tsx`；`src/features/workspace/FileTypeStats.tsx`；`src/features/workspace/OpenWorkspaceButton.tsx`；`src/features/workspace/StatusBar.tsx`
- **依赖前置**：P1-T07
- **验证方式**：组件测试 + 手工验证：名称/路径/validity 正确展示、文件统计正确、打开项目按钮工作、validity 颜色语义正确
- **不做什么**：不实现阶段列表和阶段概览（见 P1-T09）

### P1-T09 实现阶段列表与阶段概览 UI

- **目标**：实现 `StagePanel`、`StageListItem`、`StageOverview`；阶段列表状态标识正确、空阶段灰色降级、不可读阶段禁用、命名异常可点击；阶段概览展示文件列表（按 `source_kind` 分组）
- **预计修改**：`src/features/workspace/StagePanel.tsx`；`src/features/workspace/StageOverview.tsx`
- **依赖前置**：P1-T08
- **验证方式**：组件测试 + 手工验证：阶段列表排序正确、状态标识正确、空阶段不可选中、不可读阶段不可选中、命名异常可选中、阶段概览文件列表正确分组
- **不做什么**：不实现 warnings/errors 面板（见 P1-T10）

### P1-T10 实现 warnings/errors 展示与 stage_empty_view

- **目标**：实现 `WarningList`、`ErrorBanner`；`stage_empty_view` 展示空阶段说明、不激活"开始分析"；`stage_unreadable` 作为阶段级错误提示；全局 `path_not_found`/`not_directory`/`permission_denied` 走 `ErrorBanner`
- **预计修改**：`src/features/workspace/WarningList.tsx`；`src/features/workspace/ErrorBanner.tsx`
- **依赖前置**：P1-T09
- **验证方式**：组件测试 + 手工验证：warning 列表正确渲染、error banner 正确展示、空阶段不显示"强制继续分析"按钮、阶段级错误不阻断 workspace
- **不做什么**：不实现 evidence 面板、不实现 JSON viewer

### P1-T11 实现测试夹具与 Rust 单元/集成测试

- **目标**：按 [`phase-1-workspace-scanning-validation.md`](../testing/phase-1-workspace-scanning-validation.md) §3 的样例矩阵构造测试夹具，编写 Rust 单元测试（scanner、stage_detector、file_classifier、validity、safety_guard）和集成测试（`open_workspace`、`select_stage`）
- **预计修改**：`src-tauri/tests/`；各 `*_test.rs`
- **依赖前置**：P1-T06
- **验证方式**：`cargo test` 全部通过；覆盖 §3 样例矩阵中的关键场景
- **不做什么**：不写前端测试（见 P1-T12）、不写 UI smoke 测试（见 P1-T12）

### P1-T12 实现前端组件测试和 UI smoke test

- **目标**：编写前端组件测试（mock Tauri invoke）和 Playwright UI smoke 测试（端到端流程）
- **预计修改**：`src/features/workspace/__tests__/`；`e2e/`
- **依赖前置**：P1-T10、P1-T11
- **验证方式**：`npm test` 前端测试通过；Playwright 测试通过（至少覆盖：打开项目 → 查看概览 → 选择阶段 → 查看阶段概览）
- **不做什么**：不做性能测试、不做跨平台测试

### P1-T13 执行 Phase 1 验收与文档同步

- **目标**：执行 [`phase-1-workspace-scanning-validation.md`](../testing/phase-1-workspace-scanning-validation.md) §9 验收 checklist；修复发现的 bug；同步更新所有设计文档状态；编写 Phase 1 退出报告
- **预计修改**：执行验收、修复 bug、同步实现过程中发现的文档偏差；如 active 文档需要变更，按文档变更规则更新 `updated` 字段并经审核
- **依赖前置**：P1-T12
- **验证方式**：手工 QA 通过（§7 清单）、安全回归通过（§6）、文档一致性通过（§9.4）
- **不做什么**：不进入 Phase 2 编码

## 6. 编码顺序说明

### 为什么先类型契约

- `CommandResult<T>`、`WorkspaceProfile`、`StageContext` 是前后端共享的契约
- 先定义契约可确保前后端对字段名称、类型、可选性达成一致
- 契约确定后，backend 和 frontend 可并行开发

### 为什么先 backend 再 frontend

- backend 提供 `open_workspace` 和 `select_stage` 两个 Tauri command
- frontend 依赖这两个 command 的数据结构和行为
- 先实现 backend 可确保 frontend 有稳定的 mock 数据

### 为什么 command 边界在 scanner 之后

- `open_workspace` 和 `select_stage` 是 backend 算法的封装层
- 先实现底层 scanner/detector，再包装为 Tauri command，可分层测试

### 为什么 UI 最后

- UI 是状态的消费方，不是状态的生产方
- 先确保数据流正确，再做展示，可减少 UI 返工
- UI 先行容易导致契约偏移（UI 需求反向修改数据结构）

### 为什么测试穿插而非最后补

- 每个 backend 模块完成后即写单元测试，可快速发现回归
- integration tests 在 command 完成后写，验证端到端契约
- frontend 测试在组件完成后写，验证状态驱动渲染

### commit 建议

- 每个任务完成后 commit，message 格式：`feat(scope): description`
- 小步提交：一个模块、一个测试、一个 bug fix 各一个 commit
- 不混合无关修改（如同时修改 scanner 和 UI）

## 7. 测试与验证顺序

验证顺序引用 [`phase-1-workspace-scanning-validation.md`](../testing/phase-1-workspace-scanning-validation.md)：

| 顺序 | 测试类型 | 执行时机 | 覆盖范围 | 不覆盖 |
|------|---------|---------|---------|--------|
| 1 | Rust Unit Tests | 每个 backend 模块完成后 | 算法正确性、边界条件 | Tauri command 层、前端渲染 |
| 2 | Rust Integration Tests | `open_workspace` / `select_stage` command 完成后 | Command 输入/输出、边界场景、安全约束 | 前端交互、真实 Tauri 窗口 |
| 3 | Tauri Command Boundary Tests | frontend hooks 完成后 | `invoke` 调用、JSON 解析、错误码映射 | Rust 内部逻辑、React 组件 |
| 4 | Frontend Component Tests | 每个 UI 组件完成后 | 状态驱动渲染、点击行为、空状态 | 真实文件系统、端到端流程 |
| 5 | UI Smoke Tests（Playwright） | Phase 1 功能基本完成后 | 端到端用户流程、关键边界场景 | 性能、跨平台、安全细节 |
| 6 | Manual QA | P1-T13 验收阶段 | §7 手工验证清单（9 组场景） | 自动化重复执行 |
| 7 | Safety Regression | P1-T13 验收阶段 | 目标目录只读、不运行脚本、不运行 Vivado、symlink 安全 | 功能正确性 |
| 8 | Phase 1 Acceptance | P1-T13 验收阶段 | §9 checklist（后端 17 项 + 前端 15 项 + 安全 6 项 + 文档一致性 4 项） | Phase 2~6 功能 |

## 8. Phase 1 退出标准

以下全部满足后，Phase 1 方可退出：

### 功能链路

- [ ] `open_workspace` → `WorkspaceProfile` → 前端展示 workspace 概览 → 完整链路通过
- [ ] `select_stage` → `StageContext` → 前端展示阶段概览 → 完整链路通过
- [ ] 空阶段展示 `stage_empty_view`，不激活"开始分析"
- [ ] 不可读阶段返回 `stage_unreadable`，仅阻断该阶段

### Error Code / Warning 作用域

- [ ] `path_not_found`/`not_directory`/`permission_denied`：通过 `CommandError.error_code` 返回，`success=false`，无 `WorkspaceProfile`
- [ ] `no_stage_found`：进入 `WorkspaceProfile.warnings[]` 和 `WorkspaceProfile.error_codes[]`，`success=true`
- [ ] `stage_empty`：通过 `StageContext.error_code` 表达，**不进入** `WorkspaceProfile`
- [ ] `stage_unreadable`：通过 `CommandError.error_code` 表达，**不进入** `WorkspaceProfile`
- [ ] `file_unreadable`/`file_too_large`/`scan_timeout`：仅进入 `WorkspaceProfile.warnings[]`

### 安全

- [ ] 目标项目目录不被写入（`mtime` 不变、`git status` 无变化）
- [ ] 不运行目标项目脚本
- [ ] 不运行 Vivado / synthesis / implementation / bitstream
- [ ] 根路径 symlink 被拒绝
- [ ] 扫描中 symlink 不跟随到 workspace 外部

### UI 约束

- [ ] 不展示 JSON viewer / Markdown report / 原始 JSON dump
- [ ] 不暗示已完成 evidence 收集或语义分析
- [ ] "开始分析"按钮禁用或占位
- [ ] 所有面向用户文案为简体中文

### 测试

- [ ] Rust unit tests 全部通过
- [ ] Rust integration tests 全部通过
- [ ] Frontend component tests 全部通过
- [ ] UI smoke tests 全部通过
- [ ] Manual QA 通过
- [ ] Safety regression 通过

### 文档

- [ ] 实现与 `mvp-functional-contract.md` 一致
- [ ] 实现与 `phase-1-data-and-api-contract.md` 一致
- [ ] 实现与 `phase-1-scanner-detail-design.md` 一致
- [ ] 实现与 `phase-1-workspace-and-stage-flow.md` 一致
- [ ] 发现的所有文档-实现偏差已记录并同步修正

## 9. 风险与回滚

### R1: Tauri 初始化引入过多模板代码

- **风险**：`cargo create-tauri-app` 生成大量与 Phase 1 无关的模板代码和依赖
- **缓解**：初始化后审查并删除不需要的模板文件；保持依赖最小化；不引入 Phase 2~6 的依赖
- **回滚**：若模板污染严重，可重新初始化并仅保留必要文件

### R2: Rust/TypeScript 类型漂移

- **风险**：backend 修改 struct 字段后，frontend TypeScript interface 未同步
- **缓解**：P1-T02 建立严格契约，后续任何字段变更需同时更新两端；使用 CI 检查 `cargo check` + `tsc --noEmit`
- **回滚**：在契约文档中记录变更，同步更新两端类型定义

### R3: Warning/error_code 作用域混淆

- **风险**：编码时将 `stage_empty`/`stage_unreadable` 误写入 `WorkspaceProfile.error_codes[]`，或将路径校验错误误写入 `WorkspaceProfile`
- **缓解**：P1-T02 契约中明确作用域；P1-T11 集成测试中覆盖作用域验证；代码 review 时重点检查 error_code 归属
- **回滚**：修复归属逻辑，同步修正测试和文档

### R4: 扫描误跟随 symlink 的安全风险

- **风险**：`canonicalize` 隐式跟随 symlink 导致路径穿越；DFS 递归时跟随 symlink 子目录
- **缓解**：P1-T03 实现 symlink 安全检查；P1-T11 安全测试中覆盖 symlink 场景；代码中显式检查 `file_type.is_symlink()` 并跳过
- **回滚**：加固 safety_guard，增加 symlink 检测测试

### R5: UI 先行导致契约偏移

- **风险**：前端展示需求反向修改 `WorkspaceProfile` 字段，导致 backend 重构
- **缓解**：严格执行"先契约、再 backend、再 frontend"顺序；UI 变更需先更新契约文档；禁止 UI 直接修改 backend 数据结构
- **回滚**：回退 UI 修改，按契约重新实现

### R6: 测试夹具不足导致边界遗漏

- **风险**：测试仅覆盖标准项目，遗漏空目录、大文件、超时、权限不足等边界
- **缓解**：按 [`phase-1-workspace-scanning-validation.md`](../testing/phase-1-workspace-scanning-validation.md) §3 矩阵构造夹具；每个边界条件至少一个测试用例
- **回滚**：补充测试夹具，增加测试用例

### R7: 性能问题（大目录扫描卡死）

- **风险**：未正确实现深度/数量/超时约束，导致大目录扫描阻塞
- **缓解**：P1-T04 实现约束检查；P1-T11 测试中覆盖大目录场景；代码 review 检查约束逻辑
- **回滚**：修复约束检查，增加超时测试

## 10. 不进入 Phase 2 的边界

Phase 1 完成后，以下能力**仍未实现**：

- **未收集 evidence**：不抽取代码片段、不建立 evidence index
- **未生成 `ImplementationUnderstanding`**：不做结构化理解产物
- **未生成结构图 / 数据流图 / 时序图**：无图形视图
- **未支持 grounded Q&A**：无问答面板
- **未支持持久化回放**：不保存历史、不支持加载旧 session
- **未调用大模型**：无 LLM 交互
- **未做跨阶段对比**：无阶段间映射

Phase 1 的产出是：
- 可运行的桌面应用骨架
- 稳定的 `WorkspaceProfile` / `StageContext` 数据契约
- 可靠的 workspace 扫描与阶段识别能力
- 清晰的前端状态管理与 UI 展示
- 完整的测试覆盖

Phase 2 的输入是 Phase 1 的 `StageContext`（含文件列表、外部依赖、上游引用），Phase 2 将在此基础上进行 evidence 收集。
