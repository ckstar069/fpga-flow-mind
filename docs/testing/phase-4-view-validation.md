# Phase 4 视图验证设计

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 4 三类视图的验证策略，覆盖后端 ViewGraph 生成、前端渲染、安全回归和桌面验收。

## 1. 验证目标

Phase 4 编码完成后，以下维度应通过验证：

| 维度 | 验证内容 |
|------|----------|
| 后端 ViewGraph 生成 | IU → ViewGraph 转换正确性 |
| Node/Edge trace | trace_refs 正确关联 claim_id / evidence_id |
| 空状态 | 无数据时生成空 ViewGraph，不 panic |
| 前端渲染 | MultiViewPanel 三 tab 切换 + 节点/边 SVG |
| Hover tooltip | trace_refs 信息正确展示 |
| 安全回归 | 目标项目只读、无 LLM API、无 Phase 5 功能 |

## 2. 测试模块分布

### 2.1 Rust 后端测试

| 测试位置 | 覆盖模块 | 预估数量 |
|----------|----------|----------|
| `views/models.rs` | ViewGraph/ViewNode/ViewEdge serde 序列化 | 4 |
| `views/structure_builder.rs` | IU → ViewGraph(structure) 正确性 | 6 |
| `views/dataflow_builder.rs` | IU → ViewGraph(dataflow) 正确性 | 6 |
| `views/timing_builder.rs` | IU → ViewGraph(timing) 正确性 | 6 |
| `views/generator.rs` | ViewGraphGenerator 总调度 + 空 IU + degraded IU | 6 |
| `commands/generate_views.rs` | Tauri command 层（E2E + 只读 + 错误路径） | 8 |
| **合计** | | **~36** |

### 2.2 前端验证

| 验证方式 | 覆盖内容 |
|----------|----------|
| `npm run build` | TypeScript 编译 + Vite 构建 |
| 代码路径检查 | MultiViewPanel 状态 + tab 切换 + 空状态 |
| 桌面验收 | 完整用户流程 |

## 3. 后端测试矩阵

### 3.1 StructureBuilder

| 用例 | 输入 | 预期 |
|------|------|------|
| 正常 IU（含 module + signal + interface） | 3 modules + 2 signals + 1 interface | 6 nodes + 关联边，trace_refs 非空 |
| 仅 modules | 2 modules，无 signal/interface | 2 nodes，0 edges |
| 空 IU | 全部字段为空 | ViewGraph nodes=[], edges=[]，不 panic |
| Degraded IU | is_degraded=true | ViewMeta.is_degraded_source=true，nodes/edges 最小 |
| IU 含 claims | 3 claims 匹配 module 描述 | trace_refs 包含对应 claim_id |

### 3.2 DataflowBuilder

| 用例 | 输入 | 预期 |
|------|------|------|
| 正常 IU（含 processing_steps + input/output signals） | 3 steps + 1 input + 1 output | 5 nodes + 顺序边 DataFlow |
| 无 processing_steps | 仅有 signals | 空 ViewGraph，不 panic |
| 单 processing_step | 1 step | 1 node，0 edges |
| steps 带 order 字段 | out-of-order order | 按 order 排序生成节点 |

### 3.3 TimingBuilder

| 用例 | 输入 | 预期 |
|------|------|------|
| 正常 IU（含 processing_steps + clock claims） | 3 steps + 1 clock claim | PipelineStage nodes + ClockDomain node |
| 无 processing_steps + 无 clock claims | 空 | 最小 ViewGraph + "No timing info" 标注 |
| 仅 clock/reset claims | 2 clock claims | ClockDomain + ResetDomain nodes |

### 3.4 Generator 集成

| 用例 | 输入 | 预期 |
|------|------|------|
| generate_all 正常 IU | full IU | 返回 3 个 ViewGraph |
| generate_all degraded IU | degraded IU | 3 个 ViewGraph，均 is_degraded_source=true |
| generate_all 空 IU | empty IU | 3 个空 ViewGraph，不 panic |
| 单个 builder 独立性 | IU 含 modules 但无 steps | structure 正常，dataflow/timing 空 |

## 4. 前端验证矩阵

| 场景 | 预期 |
|------|------|
| 三 tab 渲染 | 结构图/数据流/时序流水 tab 可见可点击 |
| 默认选中结构图 tab | 初始显示结构图 |
| 切换 tab | 内容切换，无闪烁 |
| 节点渲染 | 各 NodeType 形状/颜色正确 |
| 边渲染 | 箭头 + 实线/虚线正确 |
| 置信度视觉 | confirmed/supported/inferred/unknown/conflicting 线型正确 |
| Hover tooltip | 显示名称 + confidence + trace_refs |
| 空状态 | 无数据 tab 显示空状态，非空白 |
| Degraded 横幅 | 降级数据标注可见 |
| 错误面板 | 单个 view 错误不影响其他 tab |
| 证据链可见 | evidence_id / claim_id chip 可见 |

## 5. 安全回归

```bash
# 禁止 API 检查
rg "std::fs::write|std::fs::create_dir|std::fs::remove_file|std::fs::rename|std::fs::copy|std::process::Command|Command::new" src-tauri/src/views/

# 越界检查
rg "GraphView|ReactFlow|D3|Mermaid|Q&A|LLM" src src-tauri/src/views/

# LLM API 检查
rg "openai|anthropic|api_key" src src-tauri/src/views/

# Phase 5 回链未提前实现
rg "open.*source|jump.*to.*file|click.*evidence.*highlight" src/
```

预期：无匹配。

## 6. 桌面验收

使用 Phase 3 样例项目 `/tmp/fpga-flow-mind-phase3-acceptance-20260612-144026`：

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 打开项目，选择 L0，生成理解 | UnderstandingPanel 正常 |
| 2 | 点击"生成视图" | loading → 三视图 tab 出现 |
| 3 | 查看结构图 tab | modules + signals + processing steps 节点可见 |
| 4 | 查看数据流 tab | 输入/处理/输出节点 + 数据流边 |
| 5 | 查看时序流水 tab | 处理顺序链 |
| 6 | Hover 节点 | tooltip 显示 claim_id + evidence_id |
| 7 | 切换 L2（空阶段） | 生成理解 → 生成视图 → 空状态正确 |
| 8 | 验证目标项目只读 | checksum 一致 |

## 7. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft）：定义 Phase 4 测试策略、后端/前端测试矩阵、安全回归、桌面验收 | Claude |
