# Phase 4 视图生成器后端设计

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 4 后端 ViewGraph 生成器的设计：从 `ImplementationUnderstanding` 到三类 `ViewGraph` 的确定性转换逻辑。

## 1. 整体架构

```text
输入: ImplementationUnderstanding (Phase 3 产出)
  │
  ▼
┌─────────────────────────────────┐
│  ViewGraphGenerator             │
│  ┌───────────────────────────┐  │
│  │ build_structure_view()    │  │  → ViewGraph(structure)
│  │ build_dataflow_view()     │  │  → ViewGraph(dataflow)
│  │ build_timing_view()       │  │  → ViewGraph(timing)
│  └───────────────────────────┘  │
└─────────────────────────────────┘
  │
  ▼
三个 ViewGraph → Tauri command → 前端渲染
```

**核心原则**：
- 后端只做确定性数据转换，不做布局计算
- 不调用 LLM API
- 不访问目标项目文件
- 不修改任何文件

## 2. 模块布局

```text
src-tauri/src/
├── views/
│   ├── mod.rs                  ← 模块入口，re-export
│   ├── models.rs               ← ViewGraph 等数据结构（Phase 4 模型）
│   ├── structure_builder.rs    ← IU → ViewGraph(structure)
│   ├── dataflow_builder.rs     ← IU → ViewGraph(dataflow)
│   ├── timing_builder.rs       ← IU → ViewGraph(timing)
│   └── generator.rs            ← ViewGraphGenerator 总调度
├── commands/
│   └── generate_views.rs       ← Tauri command
└── lib.rs                       ← 注册模块 + command
```

## 3. ViewGraphGenerator

```rust
pub struct ViewGraphGenerator;

impl ViewGraphGenerator {
    /// 从 IU 生成全部三类 ViewGraph
    pub fn generate_all(iu: &ImplementationUnderstanding) -> Vec<ViewGraph> {
        vec![
            Self::build_structure_view(iu),
            Self::build_dataflow_view(iu),
            Self::build_timing_view(iu),
        ]
    }

    pub fn build_structure_view(iu: &ImplementationUnderstanding) -> ViewGraph {
        // 从 module_summaries, signal_summaries, interface_summaries,
        // processing_steps, claims 构建结构图
    }

    pub fn build_dataflow_view(iu: &ImplementationUnderstanding) -> ViewGraph {
        // 从 processing_steps, claims, signal_summaries, interface_summaries
        // 构建数据流图
    }

    pub fn build_timing_view(iu: &ImplementationUnderstanding) -> ViewGraph {
        // 从 processing_steps, claims, module_summaries 构建时序图
    }
}
```

## 4. build_structure_view 转换规则

### 4.1 节点生成

| IU 字段 | → NodeType | node_id 格式 | layout hint |
|---------|-----------|-------------|-------------|
| `module_summaries[i]` | Module | `N-S-{:04}` | column=0, row=i, depth=0 |
| `signal_summaries[i]` | Signal | `N-S-{:04}` | column=1, row=i, depth=0 |
| `interface_summaries[i]` | Interface | `N-S-{:04}` | column=2, row=i, depth=0 |
| `processing_steps[i]` | ProcessingStep | `N-S-{:04}` | column=0, row=i+N, depth=1 |

### 4.2 边生成

- Module → Signal：互相关联模块的信号，type=`References`
- Module → Interface：模块使用了接口，type=`References`
- ProcessingStep → Module：处理步骤发生在模块内，type=`Contains`

### 4.3 trace_refs 映射

- 每个 node 的 trace_refs 来自对应 IU 条目的 `evidence_refs`
- 如 IU 条目无 evidence_refs，trace_refs 为空
- 从 claims 匹配 description 含节点名称的 claim，追加 claim_id

## 5. build_dataflow_view 转换规则

### 5.1 节点生成

| IU 字段 | → NodeType | 条件 |
|---------|-----------|------|
| `interface_summaries` 含输入信号 | InputSource | direction 或名称含 "in"/"input" |
| `processing_steps[i]` | ProcessingStep | 按 order 字段排序 |
| `interface_summaries` 含输出信号 | OutputTarget | direction 或名称含 "out"/"output" |
| `signal_summaries` 中间信号 | IntermediateData | 非输入非输出 |

### 5.2 边生成

- InputSource → ProcessingStep[0]：type=`DataFlow`
- ProcessingStep[i] → ProcessingStep[i+1]：type=`DataFlow`
- ProcessingStep[-1] → OutputTarget：type=`DataFlow`
- 无 processing_steps 时：Signal → Signal 直接 data_flow 边

### 5.3 空数据流处理

- `processing_steps` 为空且 `signal_summaries` 为空 → 生成空 ViewGraph（nodes=[], edges=[]）
- 前端显示"数据流信息不足"

## 6. build_timing_view 转换规则

### 6.1 节点生成

| IU 字段 | → NodeType | 条件 |
|---------|-----------|------|
| `processing_steps[i]` | PipelineStage | 按 order 排序 |
| `claims` 含 clock/复位相关 | ClockDomain / ResetDomain | description 含 "clock"/"clk"/"reset"/"rst" |
| `module_summaries` 含硬件模块 | PipelineStage | 作为备选 |

### 6.2 边生成

- PipelineStage[i] → PipelineStage[i+1]：type=`SequentialOrder`
- PipelineStage[i] → PipelineStage[i+1]：type=`PipelineForward`（若描述含 "pipe"/"流水"）
- ClockDomain → PipelineStage（关联的 stage）：type=`ClockDriven`

### 6.3 无 timing 信息处理

- `processing_steps` 为空且无 clock/reset claims → 生成最小 ViewGraph
- 单一节点标注"No timing information available from evidence"
- 前端显示"时序信息不足"

## 7. Tauri Command

```rust
#[tauri::command]
pub fn generate_views(
    root_path: String,
    stage_id: String,
) -> CommandResult<Vec<ViewGraph>> {
    // 1. resolve_stage_context + EvidenceCollector + generate_understanding
    //    复用 Phase 3 链路的 generate_understanding 逻辑
    // 2. ViewGraphGenerator::generate_all(&iu)
    // 3. 返回 CommandResult<Vec<ViewGraph>>
}
```

**与 Phase 3 的关系**：
- 方案 A：`generate_views` 内部先调用 `generate_understanding` 再转换
- 方案 B：前端先 `generateUnderstanding`，再传 IU 给 `generate_views`
- MVP 阶段建议方案 A（简化前端状态机）

## 8. 错误处理

| 场景 | 行为 |
|------|------|
| IU 不存在（尚未生成） | 返回 error，前端不展示三视图 tab |
| IU 为 degraded | 仍生成 ViewGraph（含空状态标注） |
| 单个 view builder panic | catch panic，对应 ViewGraph 含 error meta |
| claims/module_summaries 等为空 | 生成空 nodes/edges 的 ViewGraph，不 panic |

## 9. 与 Phase 3 代码的关系

- 复用 `resolve_stage_context`（Phase 1/2 共享）
- 复用 `EvidenceCollector`（Phase 2）
- 复用 `generate_understanding` command 逻辑（Phase 3）
- `ViewGraphGenerator` 是纯转换函数，不调用外部系统
- `ViewGraph` 不包含文件路径或行号范围 — 相关信息通过 `trace_refs` 间接引用

## 10. 安全约束

- 不使用 `std::fs::write`、`std::fs::create_dir`、`std::fs::remove_file`
- 不使用 `std::process::Command`
- 不调用 Vivado / synthesis / implementation / bitstream
- 不调用真实 LLM API
- 不访问目标项目文件（仅通过已缓存的 IU 数据）

## 11. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft）：定义 ViewGraphGenerator + 三类 builder 转换规则 + Tauri command + 错误处理 | Claude |
