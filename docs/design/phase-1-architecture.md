# Phase 1 概要设计

---
status: draft
updated: 2026-06-11
---

> 本文档是 Phase 1 概要设计，不是实施计划，不是代码。
> Phase 1 指 `fpga-flow-mind` **本项目**的开发推进阶段，不是业务项目的 `L0` / `L1` / `RTL` 实现阶段。

## 1. 设计目标

Phase 1 只解决：

- 打开业务项目目录（Tauri 文件选择器）
- 只读扫描 workspace
- 识别是否可能是 `ai_project_template` 创建的项目
- 识别候选阶段目录
- 生成 `workspace_profile.json`
- 用户选择单阶段后生成 `stage_context.json`

Phase 1 **不解决**：evidence 提取（Phase 2）、大模型调用（Phase 3）、语义理解（Phase 3）、视图生成（Phase 4）、追问（Phase 5）、完整持久化回放（Phase 6）、跨阶段对比、Python→RTL 映射。

## 2. 技术栈与约束

| 层级 | 技术选择 | 说明 |
|------|---------|------|
| 桌面壳 | Tauri v2 | 本地文件访问、前后桥接、打包 |
| 后端 | Rust | workspace 扫描、阶段识别、文件分类、安全边界 |
| 前端 | React + TypeScript | 展示、交互、状态管理 |

**明确约束**：

- 不使用 Python 作为产品核心实现
- 不使用 Electron / PySide6
- 不做 Web GUI 主路线
- Phase 1 不调用大模型，不设计 provider 调用
- 所有产物在 Phase 1 默认可只在内存中保留，不强制持久化

## 3. 模块划分

### Frontend（React + TypeScript）

| 模块 | 职责 |
|------|------|
| `WorkspacePage` | 承载 workspace 概览面板，展示名称、路径、阶段列表、文件统计 |
| `StagePanel` | 阶段列表展示，支持单选、状态标注、排序 |
| `StageOverview` | 选中阶段后的概览展示，含文件列表分组和"开始分析"按钮（Phase 1 禁用或占位） |
| `StatusBar` | 底部状态栏，展示 validity、扫描进度、强制继续入口 |
| `WarningList` | 警告/错误列表面板，展示 warnings 和 error_codes |
| `FileTypeStats` | 文件类型统计展示组件 |

### Tauri command boundary

| Command | 职责 |
|---------|------|
| `open_workspace` | 接收路径字符串，调用 Rust 扫描，返回 `CommandResult<WorkspaceProfile>` |
| `select_stage` | 接收 `root_path` + `stage_id`，调用 Rust 生成 `CommandResult<StageContext>` |

### Rust backend

| 模块 | 职责 |
|------|------|
| `workspace_scanner` | 递归扫描目录、收集文件元数据、生成文件类型统计 |
| `stage_detector` | 识别阶段目录、判断命名异常、检测缺失阶段、计算 file_count |
| `file_classifier` | 按扩展名和文件名模式分类 language / source_kind |
| `external_ref_detector` | 字符串匹配识别 `urban_wireless` 等外部模块引用 |
| `validity_calculator` | 基于扫描结果计算 `workspace_validity` |
| `safety_guard` | 路径校验（存在/目录/可读）、扫描边界控制（深度/数量/超时）、只读检查 |
| `dto/model layer` | 定义 Rust struct 与 serde 序列化，输出 JSON 契约对象 |

## 4. 数据流

```text
┌─────────────────┐     选择目录      ┌──────────────────┐
│   React UI      │ ────────────────> │ Tauri open_workspace
│  (WorkspacePage)│                   │     command      │
└─────────────────┘                   └────────┬─────────┘
                                               │
                                               ▼
                              ┌────────────────────────────────┐
                              │   Rust workspace_scanner       │
                              │   + stage_detector             │
                              │   + file_classifier            │
                              │   + safety_guard               │
                              └────────┬───────────────────────┘
                                       │
                                       │ WorkspaceProfile (JSON)
                                       ▼
┌─────────────────┐     展示概览      ┌──────────────────┐
│   React UI      │ <───────────────  │   解析 JSON      │
│  (WorkspacePage)│                   │   更新状态       │
│  + StagePanel   │                   └──────────────────┘
│  + WarningList  │
└────────┬────────┘
         │ 点击阶段
         ▼
┌─────────────────┐     stage_id      ┌──────────────────┐
│  StagePanel     │ ────────────────> │ Tauri select_stage
│  (单选)         │                   │     command      │
└─────────────────┘                   └────────┬─────────┘
                                               │
                                               ▼
                              ┌────────────────────────────────┐
                              │   Rust 验证阶段存在+可读         │
                              │   + 扫描阶段文件（深度2）        │
                              │   + 生成 StageContext          │
                              └────────┬───────────────────────┘
                                       │
                                       │ StageContext (JSON)
                                       ▼
┌─────────────────┐     展示阶段      ┌──────────────────┐
│   React UI      │ <───────────────  │   解析 JSON      │
│  (StageOverview)│                   │   更新状态       │
└─────────────────┘                   └──────────────────┘
```

## 5. 前后端职责边界

| 职责 | 归属 | 说明 |
|------|------|------|
| 路径校验（存在/目录/可读） | **Rust only** | 前端不直接访问文件系统 |
| 递归文件扫描 | **Rust only** | 受深度/数量/超时约束 |
| 阶段目录识别 | **Rust only** | 基于目录名模式匹配 |
| 文件类型分类 | **Rust only** | 扩展名和文件名模式 |
| 外部模块引用识别 | **Rust only** | 正则/字符串匹配 |
| validity 计算 | **Rust only** | 基于扫描结果的规则引擎 |
| 安全边界执行 | **Rust only** | 只读检查、不跟随 symlink |
| 阶段选择验证 | **Rust only** | 验证阶段目录仍存在且可读 |
| 展示、渲染、样式 | **React only** | 状态驱动 UI |
| 选中状态管理 | **React only** | 单选高亮、禁用状态 |
| warning/error 呈现 | **React only** | 列表、图标、颜色语义 |
| 筛选/排序交互 | **React only** | 前端状态操作 |
| 强制继续按钮 | **React only** | 用户意图收集 |

**核心原则**：React 不直接访问目标项目文件系统，所有文件系统操作必须通过 Tauri command → Rust 完成。

## 6. 安全设计

- **目标项目只读**：Rust backend 仅使用 `read_dir`、`metadata`、`read_file`，禁止 `write`、`create`、`remove`、`rename`
- **不写入 `fpga_project_*`**：所有产物写入 app-owned 目录或仅保留在内存中
- **不运行 Vivado / synthesis / implementation / bitstream**
- **不执行目标项目脚本**：不默认运行 `.py`、`.sh`、`.tcl` 等脚本
- **不跟随 symlink**：根路径若为符号链接则拒绝（按 `permission_denied` 处理）；扫描时遇到符号链接直接跳过
- **扫描边界**：递归深度 ≤ 3、单目录文件数 ≤ 1000、总文件数 ≤ 5000、超时 30 秒
- **权限检查**：扫描前验证路径可读性，遇 `permission_denied` 立即返回错误

## 7. Phase 1 目录结构建议

> 以下仅为编码阶段参考，本轮不创建文件。

```text
fpga-flow-mind/
├── src/                          # React + TypeScript 前端
│   ├── features/
│   │   └── workspace/
│   │       ├── WorkspacePage.tsx
│   │       ├── StagePanel.tsx
│   │       ├── StageOverview.tsx
│   │       ├── WarningList.tsx
│   │       └── hooks/
│   ├── types/
│   │   └── workspace.ts          # TypeScript interface
│   └── components/
│       └── StatusBar.tsx
├── src-tauri/
│   ├── src/
│   │   ├── commands/
│   │   │   ├── open_workspace.rs
│   │   │   └── select_stage.rs
│   │   ├── workspace/
│   │   │   ├── scanner.rs        # workspace_scanner
│   │   │   ├── stage_detector.rs # stage_detector
│   │   │   ├── file_classifier.rs# file_classifier
│   │   │   ├── external_refs.rs  # external_ref_detector
│   │   │   ├── validity.rs       # validity_calculator
│   │   │   └── safety_guard.rs   # safety_guard
│   │   ├── models/
│   │   │   ├── workspace_profile.rs
│   │   │   ├── stage_context.rs
│   │   │   ├── enums.rs          # WorkspaceValidity, StageStatus, etc.
│   │   │   └── error.rs          # CommandError, CommandResult
│   │   └── lib.rs
│   └── Cargo.toml
```

## 8. 与其他设计文档关系

| 文档 | 定位 | 关系 |
|------|------|------|
| [`workspace-scanning-and-stage-detection.md`](workspace-scanning-and-stage-detection.md) | Phase 1 技术设计**入口与边界说明** | 阅读起点，概述 Phase 1 做什么、不做什么 |
| **本文档** | Phase 1 **概要设计** | 在入口文档基础上展开模块划分、数据流、职责边界、安全设计 |
| [`phase-1-data-and-api-contract.md`](phase-1-data-and-api-contract.md) | Phase 1 **数据结构与前后端接口** | 定义 Rust struct、TypeScript interface、Tauri command 签名、错误格式 |
| [`phase-1-scanner-detail-design.md`](phase-1-scanner-detail-design.md) | Phase 1 **扫描与阶段识别详细设计** | 细化扫描算法、阶段识别算法、validity 判定算法、边界条件处理 |

阅读顺序建议：
```text
workspace-scanning-and-stage-detection.md（入口）
  -> phase-1-architecture.md（概要）
    -> phase-1-data-and-api-contract.md（数据/API）
      -> phase-1-scanner-detail-design.md（详细算法）
```
