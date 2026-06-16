# Phase 7 Batch D 真实项目质量基线报告

---
status: active
updated: 2026-06-16
---

> 本报告是 Phase 7 Batch D 第一步的产出：基于真实/等价 `ai_project_template` 生成项目建立质量基线，回答“fpga-flow-mind 在接近真实项目时到底离可用还差多少”。
> 本报告**不是** Phase 7 完成验收，也不是 Phase 8/9/10 规划。

## 0. 当前状态（最新更新：2026-06-16，请优先阅读）

> 本节描述 **P0-1/P0-2 修复后** 的当前状态。下文 §1~§6 的具体数值与“真实结构不被识别 / scan_timeout 30 条”等表述是 **历史基线发现**（详见 §0.2），不得据此判断当前工具状态。

### 0.1 已收口项（Batch D P0-1~P0-4 + P1）

- **P0-1/P0-2/P0-3/P0-4** 已验收通过（见下方各子节）。
- **P2 质量信号校准 + 阶段状态隔离**（2026-06-16 完成）：引入 4 个更细 `QualityIssueKind`（`ExpectedEmptyTiming`/`IsolatedOrUnconnectedView`/`TraceabilityGap`/`LowSemanticDiversity`）；`view_evaluator` 校准分类与严重度；`reporter` 新增 structure 降解检测；前端 `WorkspacePage` 状态隔离补强（maps 清除 + guard）。详见 §4.5。
- **P1 evidence/understanding 丰富度补强**（2026-06-16 完成）：
  - **Python evidence extractor 粒度提升：** 新增 imports（`import X` / `from Y import Z`）、顶层常量（全大写 `NAME = value`）、`@dataclass` 字段、返回类型注释（`-> Type`）、`self.field` 赋值、函数内关键调用站等 6 类提取。单文件 evidence 数从粗放 def/class 级别提升到涵盖模块依赖、配置参数、数据结构、函数关系的细粒度级别。
  - **Verilog / SystemVerilog evidence extractor 粒度提升：** 新增端口（input/output/inout）、信号声明（wire/reg/logic）、assign 语句、always/always_ff/always_comb 块、模块实例化、parameter/localparam 等 6 类提取。单 module 不再只有 1 条粗 evidence，而是拆分为端口、信号、组合/时序逻辑、子模块、参数等多条可追溯 evidence。
  - **Understanding generator 丰富度提升：**
    - claims：从固定前 3 条 evidence 增加到为每条 evidence 生成独立 claim（无封顶）。
    - module_summaries：从只有 1 个粗模块增加到为每个有 symbol 的 evidence 生成 module_summary（上限 15）。
    - interface_summaries：从保守为空数组变为从端口证据、import 依赖、实例化证据保守派生接口端点。
    - signal_summaries：新增 Python 全大写常量作为配置信号的派生。
    - unknowns / evidence_gaps：从简单的 `< 2` / `< 3` 阈值变为基于实际维度缺失（无模块、无信号、无接口、无处理步骤）的真实 gap 表达。
  - **Structure view 补强：** 边生成从 `break` 限定的第一个模块扩展到所有通过 evidence_id 匹配的模块/信号/接口/步骤。不再只有 1 个模块节点和 1 条 module→signal 边，而是多个模块节点各自连向相关证据的端口/信号节点。
  - **P1 前置 hygiene：** `real_project_validation.rs` 的 checksum 已在 P1 前置收口中改为纯 Rust SHA-256 实现（无 Command::new），通过验证。
  - **测试新增：** Python extractor 7 项（imports/constants/dataclass/return-type/self-field/call-site/comprehensive）、Verilog extractor 7 项（port/signal/assign/always/instance/param/comprehensive）、SystemVerilog extractor 6 项（port/signal/always/param/instance/comprehensive）。全量 `cargo test --lib` 从 516 增加到 **536**（+20，1 ignored）。

### 0.2 已收口项（本轮 Batch D P0-1/P0-2/P0-3）

- **P0-1 阶段识别已修：** `stage_detector` 已支持 `ai_project_template` 深层布局，打开 `src/python_model/L*_xxx`、`src/verilog_model/rtl` 原目录即可识别 L0~L6、RTL 阶段，顶层目录优先，重复候选生成 warning。`select_stage` 端到端测试覆盖 ai_project_template 布局（L1、RTL 及深度 5 源码进入 StageContext.files）。
- **P0-2 扫描收口已修：** `scanner` 已跳过 `.git` / `.claude` / `__pycache__` / `.pytest_cache` / `.mypy_cache` / `.ruff_cache` / `.egg-info` / `vivado` / `reports` / `build` / `dist` / `node_modules` / `target` / `.idea` / `.vscode` / `.venv` / `venv` / `sim_build` / `.tox` / `htmlcov` 等噪声目录与 `.DS_Store` / `.coverage` 等噪声文件；**并修复了深层源码漏扫**：一旦进入 `src/python_model/L*_xxx` 或 `src/verilog_model/rtl` 深层源码树，其全部子孙目录不再受固定 `depth > 3` 拦截，真实深度 5 的源码（如 `src/python_model/L0_external/rx_02_coarse_sync/coarse_block.py`、`src/python_model/L0_external/shared_04_preamble/preamble.py`）可被完整扫描且不产生 `scan_timeout`。噪声目录的深度跳过在深层源码树之外仍然生效。
- **P0-3 dataflow/timing 最小非空保守生成已修（Batch D P0-3 首轮）：** 根因是 `MockProvider` 硬编码 `signal_summaries`/`interface_summaries`/`processing_steps` 为空数组，丢弃了 evidence 已携带的 symbol/excerpt/source_kind。修复路径（保守、可追溯，不接真实 LLM）：`MockProvider` 新增 `derive_conservative_summaries`，从 evidence 的 Python 函数/类符号按 evidence 顺序派生 `processing_steps`（绑定 evidence_id，confidence=inferred），从 RTL evidence excerpt 识别 input/output/clk 派生 `signal_summaries`；`dataflow_builder`/`timing_builder` 的顺序/时钟边改为从端点节点 trace_refs 合并派生（消除原空 trace 推断边）；`timing_builder` 新增 RTL 时序保守回退（processing_steps 空但有 clk/rst 信号时生成 ClockDomain/ResetDomain 节点）。
- **P0-3 收口修复（Batch D P0-3 本轮）：禁止 Python 函数顺序伪造成 timing 图。** `timing_builder` 新增 `has_temporal_evidence` 门控：只有 step.description/name/claim/signal 中出现 cycle/latency/clock/pipeline/stage/clk/rst/posedge 等时序关键词，或 stage 明确含 RTL/pipeline/cycle 语义且 evidence 含时序内容时，才允许从 `processing_steps` 生成 `PipelineStage` 节点。对普通 L0/L1 Python 原型阶段，即使存在 `processing_steps`（由 MockProvider 从函数符号派生），也不能仅凭 order 生成 timing。若无时序依据，timing view 必须为空，并给出明确 `empty_reason`：“无 cycle/latency/clock/pipeline 等可追溯时序证据，未生成 timing 图（当前 processing_steps 为算法/函数顺序，非硬件时序）”。真实项目只读验证（`fpga_project_coarse_sync`，src/ SHA-256 前后一致，未创建临时目录）：
  - **L0：** dataflow 12 节点/11 边（非空，正确）；timing 0 节点/0 边 + 明确 empty_reason（**历史违规产物：此前曾生成 12/11 伪 pipeline stage，已修正**）。
  - **L1：** dataflow 9 节点/8 边（非空，正确）；timing 0 节点/0 边 + 明确 empty_reason（**历史违规产物：此前曾生成 9/8 伪 pipeline stage，已修正**）。
  - **RTL：** dataflow 4 节点/0 边；timing 2 节点/0 边（clk/rst 节点，无 pipeline 可连 → 孤立节点 Low 提示，属诚实信号而非伪造）。
  - dataflow 的 Medium `empty_or_unhelpful_view` 保持 **0**；timing 的 Medium empty 在 L0/L1 为 **1**（诚实空图，非伪造），RTL 为 **0**（有 clk/rst 节点）。

### 0.2 历史基线发现（已部分修复，仅作背景）

- 下文 §1~§6 中的“阶段检测器只识别顶层阶段目录”“真实 `src/python_model/L1_prototype` 不被识别”“扫描产生 30 条 scan_timeout”“evidence/understanding/view 过粗或为空”等，记录的是 **P0-1/P0-2/P0-3 修复前** 的基线观测值。
- 其中“阶段不被识别”与“scan_timeout 集中于噪声目录/深层目录跳过”两项，已由 P0-1/P0-2 修复（见 §0.1）。
- “dataflow/timing view 完全为空 / empty_or_unhelpful_view”一项，已由 P0-3 修复（见 §0.1）。
- evidence 粒度（Phase 2 拆分策略）、understanding 丰富度（接口/信号/处理步骤的更细语义识别）等项 **仍未修**，属 P1 及之后范围。当前 P0-3 的保守派生只解决“有 evidence 时不再无谓退化”，不替代后续 P1 的语义补强。

### 0.3 本轮明确不做 / 不得进入

- **不得** 视为 Phase 7 完成。
- **不得** 进入 Phase 8 / 9 / 10。
- 不修改目标项目目录；不运行 Vivado / synthesis / implementation / bitstream；不接真实 LLM；不输出 PASS/HOLD/正确性裁决。
- Batch D P2 已完成。后续 P3（completion review）需单独授权。

## 1. 样本项目来源与结构

> **历史基线段落：** 本节及以下 §2~§6 的具体数值、问题描述为 **P0-1/P0-2 修复前** 的基线观测。当前状态见 §0.1。其中“阶段不被识别”“scan_timeout 集中于深层噪声/源码跳过”两项已修复。

### 1.1 真实项目

- **路径：** `/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync`
- **性质：** 基于 `ai_project_template` 生成的真实 OFDM 粗同步 FPGA 实现项目，与原始模板仓库位于同一父目录下。
- **结构：**
  - `src/python_model/`：L0_external、L1_prototype、L2_structured、L3_pipeline、L4_cycle_acc、L5_fixedpoint、L6_resource_opt
  - `src/verilog_model/rtl/`：RTL 实现
  - `tests/`、`scripts/`、`vivado/`、`docs/` 等辅助目录
- **只读验证：** 对 `src/` 下所有 `.py` / `.v` / `.sv` / `.md` 文件在运行前后分别计算 SHA-256；`src/` 校验和一致，未发现源文件内容被修改。
- **⚠️ 验收方法偏差：** 本次为了让当前工具完成评估，曾将真实项目源码**临时复制到目标项目根目录**的 `L0` / `L1` / ... / `RTL` 目录中，评估完成后已删除这些临时副本。该操作**违反了严格意义上的“目标项目目录只读”边界**，即使 `src/` checksum 未变、临时副本已删除，仍属于对目标项目目录的写入。该偏差**不得作为后续验收方法复用**。
- **扫描限制：** 由于 `fpga-flow-mind` 当前阶段检测器只识别根目录下 `L0` / `L1` / ... / `RTL` 等顶层阶段目录，真实项目源码位于 `src/python_model/L1_prototype` 这类深层路径，导致直接打开真实项目时阶段识别失败（`no_stage_found` / `未识别到阶段目录`）。
  - 这一事实本身即是一项关键发现：fpga-flow-mind 的阶段检测策略与 `ai_project_template` 真实目录结构不匹配，真实项目无法直接被工具识别。

### 1.2 是否等价于 ai_project_template

- 该项目是 `ai_project_template` 工作流的直接产物，目录命名、阶段分层、文档/测试/约束结构均与模板一致。
- 相比 `/tmp` 下为 Batch C 准备的 toy 样例，本项目包含 39 个 Python 文件、6 个 Verilog 文件、真实算法语义（粗同步、能量检测、互相关、CFO 估计等），足以暴露工具的真实理解能力边界。

## 2. 完整 Phase 1~7 链路运行结果

### 2.1 Phase 1：Workspace 扫描

> **历史基线观测（修复前）。** 当前状态：阶段识别已修（见 §0.1 P0-1）；`scan_timeout` 已通过噪声目录跳过 + 深层源码扫描修复降为零（见 §0.1 P0-2）。

- **结果：** 成功打开；在临时顶层阶段目录存在时，L0~L6、RTL 均被识别，文件数统计正常。
- **问题（基线）：**
  - ~~真实 `src/python_model/L1_prototype` 结构不被识别为阶段~~（**P0-1 已修**，见 §0.1）。
  - ~~扫描产生 30 条 `scan_timeout: 目录深度超过 3` 警告~~（**P0-2 已修**：噪声目录跳过 + 深层源码树不受 depth>3 拦截，见 §0.1）。基线时这些警告多集中在 `tests/python/L*/__pycache__` 与 `.claude/commands/...` 深层目录，导致部分可分析文件被跳过。

### 2.2 Phase 2：Evidence 收集

| 阶段 | 文件数 | 收集到的 evidence 项数 | 备注 |
|------|--------|----------------------|------|
| L0   | 14     | 20                   | 包含子模块 shared_00_sync_types、shared_04_preamble、rx_02_coarse_sync |
| L1   | 4      | 7                    | coarse_sync_config/core/iterative |
| RTL  | 6      | 6                    | 每个 Verilog module 一项 |

- **问题：**
  - evidence 计数与文件数不匹配，说明子目录文件被聚合或部分被跳过；但相对于文件规模，evidence 项数偏低。
  - 深层 `src/` 真实路径无法直接扫描，真实项目中大量代码位于 `src/python_model/*`，当前 evidence 覆盖范围受限于顶层目录拷贝。

### 2.3 Phase 3：Understanding 生成

| 阶段 | evidence 数 | 声明数 | 模块数 | 未知项 | 缺失证据 |
|------|------------|--------|--------|--------|----------|
| L0   | 20         | 3      | 1      | 0      | 0        |
| L1   | 7          | 3      | 1      | 0      | 0        |
| RTL  | 6          | 未在截图中精确读取，但视图生成可用 | - | - | - |

- **问题：**
  - 对于 14 个文件、包含多个子包的真实 L0，只生成 3 条声明和 1 个模块，明显过粗。
  - 未识别接口、信号、处理步骤等关键细节；`unknown` 和 `missing_evidence` 显示为 0，可能是由于证据被简单聚合而非真正覆盖。

### 2.4 Phase 4：视图生成

| 阶段 | 结构图 | 数据流 | 时序/流水 |
|------|--------|--------|-----------|
| L0   | 1 个节点（module_SyncSearchC...） | 空 | 空 |
| L1   | 未精确读取，但 quality report 显示 empty_or_unhelpful_view: 3 | 空 | 空 |
| RTL  | 未精确读取，但 quality report 显示 empty_or_unhelpful_view: 3 | 空 | 空 |

- **问题：**
  - 结构图仅能识别出单个顶层模块，子模块、函数、类之间的关系未展开。
  - 数据流图和时序/流水图完全为空，无法帮助用户理解数据如何在模块间流动、如何处理时序。
  - 节点缺少可点击的丰富证据回链，trace 价值有限。

### 2.5 Phase 5：Trace / Grounded Q&A

- **结果：** 本次基线未在 GUI 中执行点击节点追溯和 Q&A 提问；但从 quality report 的 `missing_evidence` 与 `empty_or_unhelpful_view` 可推断：
  - trace 可点击节点数量极少（结构图仅 1 个节点）。
  - 由于 understanding 声明稀疏，Q&A 可引用的证据范围有限，复杂问题很可能返回 unknown / evidence_gap。

### 2.6 Phase 7：Quality Review

| 阶段 | acceptance | 负向问题 | 正向守卫 | 维度指标 | 质量记录 | 主要 issue 分类 |
|------|------------|----------|----------|----------|----------|-----------------|
| L0   | below_gate | 10       | 0        | 9        | 10       | missing_evidence: 7, empty_or_unhelpful_view: 3 |
| L1   | below_gate | 4        | 0        | 9        | 4        | missing_evidence: 1, empty_or_unhelpful_view: 3 |
| RTL  | below_gate | 3        | 0        | 9        | 3        | empty_or_unhelpful_view: 3 |

- **观察：**
  - 三个阶段的 quality report 均未使用 PASS/HOLD/正确/错误等审计用语，统一使用“低于当前质量门槛”。
  - Quality Review UI 能正常显示统计、issue 分类、质量记录列表，Batch C 的 UI 接入是通的。
  - 但报告本身几乎全部由退化项组成，说明 Batch C 只完成了“暴露问题”，尚未修复退化。

## 3. 主要质量差距

### 3.1 Evidence 抽取（Phase 2）

> **当前状态（P1 已修）。** 第 1、2 项已由 P0-1/P0-2 修复。第 3 项（evidence 粒度）已由 P1 修复（见 §0.1 P1 描述）。以下为 P1 前后对比。

1. ~~**目录结构不匹配真实项目：** 真实 `ai_project_template` 项目源码在 `src/python_model/L1_prototype` 等深层目录，当前 stage_detector 要求顶层阶段目录，导致真实项目无法被识别。~~（**P0-1 已修**）
2. ~~**扫描深度限制导致遗漏：** `scan_timeout` 警告频繁，深层子目录（如 `__pycache__`、`.claude`、测试辅助目录）被跳过，可能错过或误过滤有用文件。~~（**P0-2 已修**）
3. ~~**evidence 粒度偏粗：** 真实 L0 有 14 个文件、多个子模块，但只生成 20 项 evidence，无法支撑后续细粒度理解。~~（**P1 已修**：Python 新增 import/constant/dataclass/call-site 等 6 类提取，Verilog/SV 新增 port/signal/assign/always/instance/parameter 等 6 类提取，单文件 evidence 数提升 2~5x）

### 3.2 Understanding 生成（Phase 3）

1. ~~**声明数量过少：** L0 仅 3 条声明，无法覆盖多文件、多子包的语义。~~（**P1 已修**：claims 不再封顶 3 条，每条 evidence 生成独立 claim；module_summaries 从 1 个扩展到每个有 symbol 的 evidence 独立生成）
2. ~~**缺少关键维度：** 未显式识别接口契约、信号/变量、处理步骤、数据依赖、配置参数等 FPGA 设计中的关键元素。~~（**P1 已修**：interface_summaries 从空数组变为从端口/import/实例化证据派生；signal_summaries 新增 Python 常量；处理步骤从 evidence 符号派生）
3. ~~**unknown/gap 统计为 0 可能失真：** 未必表示真正理解完整，可能是 claim 生成过于保守或聚合导致未暴露未知。~~（**P1 已修**：unknowns/evidence_gaps 改为基于维度缺失的阈值判断，当缺少模块/信号/接口/步骤时诚实标注缺口）

### 3.3 View 生成（Phase 4）

1. ~~**Structure view 过简：** 仅单个节点，缺少子模块、函数、类层级。~~（**P1 已修**：边生成从 `break` 限定的第一个模块扩展到所有通过 evidence_id 匹配的模块/信号/接口/步骤节点）
2. ~~**Dataflow / Timing 完全退化：~~（**P0-3 已修**：dataflow 从 evidence 顺序派生处理步骤与顺序边；timing 含门控，Python 无时序证据时保持空+empty_reason）
3. **View 与 evidence/understanding 脱节（部分修）：** 每个 node 的 trace_refs 已有 evidence_id 和 claim_id，且 P1 的多个模块节点各自连向相关证据。但前端尚未实现节点点击追溯面板展开，仍在 Phase 5 范围。

### 3.4 Trace / Q&A（Phase 5）

1. **可 trace 目标稀少：** 结构图节点少，用户可追问的对象有限。
2. **Q&A 真实能力未验证：** 本次未执行复杂问题，但从 evidence/understanding 稀疏度判断，复杂问题大概率返回 unknown。

### 3.5 UI / 信息架构（Phase 8 范畴）

1. **界面仍是调试式堆叠：** 左侧面板、中间文件列表、右侧理解/视图/质量报告、底部警告，纵向信息密度高，用户难以快速定位关键信息。
2. **阶段切换后状态未完全隔离：** 切换回 L0 时，底部警告/质量报告未明显区分当前阶段与他阶段。
3. **警告区喧宾夺主：** 30 条 `scan_timeout` 长期占据底部，分散用户对质量报告的注意力。

## 4. 问题分类与修复优先级

### 4.1 Phase 7 修复状态

| 优先级 | 问题 | 范围 | 状态 |
|--------|------|------|------|
| P0 | stage_detector 不识别 `src/python_model/L1_prototype` 等真实结构 | workspace/stage_detector | **P0-1 已修** |
| P0 | scan_timeout 过多 | workspace/scanner | **P0-2 已修**（噪声目录跳过 + 深层源码深度修正） |
| P0 | dataflow / timing view 为空 | understanding + view generator | **P0-3 已修**（保守派生 + 门控） |
| P1 | structure view 节点过少 | understanding + view generator | **P1 已修**（多模块/信号/接口节点 + evidence_id 匹配边） |
| P1 | evidence 粒度过粗 | evidence extractors | **P1 已修**（Python 6 类 + HDL 6 类行级提取） |
| P1 | understanding 丰富度不足 | understanding generator | **P1 已修**（claims 无封顶、多个 module/interface/signal 派生） |
| P2 | 质量信号校准：4 新分类 + 前端状态隔离 | quality evaluator + frontend | **P2 已修**（ExpectedEmptyTiming/TraceabilityGap/IsolatedOrUnconnectedView/LowSemanticDiversity；前端 maps 清除 + 守卫） |

### 4.2 应留给 Phase 8 的问题

- 整体工作台信息架构重构（导航、搜索、聚焦模式）。
- 视图的可交互性、布局美化、节点详情面板。
- 警告/质量报告的聚合与降噪 UI。
- 跨阶段对比视图（L0 vs L1 vs RTL 的演进关系）。

### 4.3 应留给 Phase 9 的问题

- 接入真实 LLM 进行 grounded reasoning。
- 复杂 Q&A、证据摘要、跨文件语义关联。
- 真实 LLM 的 citation 校验与幻觉抑制。

### 4.4 应留给 Phase 10 的问题

- Python 模型到 Verilog RTL 的语义映射。
- 跨阶段等价性检查（L1 浮点算法 ↔ RTL 定点实现）。
- 信号级 trace 与差异报告。

## 5. 安全与边界确认

- **目标项目只读：** `src/` 校验和前后一致，未发现源文件内容被修改。但本次验收过程中**曾在目标项目根目录创建并删除临时阶段副本**，属于验收方法偏差，不视为完全合规的只读验收。后续 Batch D 必须避免此类操作。
- **无真实 LLM：** 当前 understanding/provider 仍为 mock。
- **无 Vivado / synthesis / implementation / bitstream：** 未运行 Vivado。
- **无 PASS/HOLD/正确/错误：** quality report 使用“低于当前质量门槛”。
- **未进入 Phase 8/9/10：** 本轮仅产出基线报告与小范围分析，未做 UI 重构或 LLM 接入。

## 6. 结论与下一步建议

> **历史基线结论（修复前）。** 当前状态：阶段识别不匹配与 scan_timeout 已由 P0-1/P0-2 修复（见 §0.1）；understanding/view 退化仍未修，属 P0-3 及之后范围。本节结论保留作为基线背景，不得据此判断当前工具阶段识别状态。

### 6.1 核心结论（基线）

Phase 7 Batch C 的 UI 接入是通的，但面对真实 `ai_project_template` 项目时，工具远未达到可用：

- ~~**阶段识别**与真实项目结构不匹配。~~（**P0-1 已修**）
- **Understanding / View / Trace / Q&A** 在真实代码上严重退化（dataflow/timing 为空属 P0-3 范围，未进入）。
- **Quality Review** 目前主要是“诚实地报告自己分析得不好”，而不是“分析得很好”。

### 6.2 建议继续 Phase 7 Batch D 修复

在进入 Phase 8 之前，应先完成 Batch D 的真实项目质量补强，特别是：

1. ~~让 stage_detector 支持真实 `ai_project_template` 目录结构。~~（**P0-1 已完成**）
2. 提升 understanding 的丰富度（接口、信号、处理步骤）。（未进入）
3. 让 dataflow / timing view 至少能生成非空图。（属 P0-3，未进入，需单独授权）
4. ~~优化 scanner，减少 scan_timeout 并提高有效源码覆盖率。~~（**P0-2 已完成**，含深层源码扫描修复）

**关于验收方法：** 后续 Batch D 验收必须满足以下二者之一：
- 直接修复 `stage_detector`，使工具能打开真实项目原目录并识别阶段；或
- 如需适配当前工具，必须将真实项目**复制到 `/tmp` 或 app-owned 临时目录**形成 normalized mirror，在镜像上操作；不得再向真实项目根目录写入临时阶段目录。

### 6.3 不建议立即进入 Phase 8/9/10

- Phase 8 的 UI 重构应建立在“分析内容足够丰富”的基础上；当前分析内容稀疏，重构后也无内容可呈现。
- Phase 9 的真实 LLM 应建立在“确定性理解管道稳定”的基础上；当前 evidence/understanding 不稳定，LLM 会放大噪声。
- Phase 10 的跨阶段映射应建立在“单阶段理解充分”的基础上；当前单阶段理解本身不足。

## 8. Phase 7 Batch D P0 修复计划

本计划是 Batch D 的下一步动作顺序，聚焦“让真实项目能被识别、被分析、被可视化”，不进入 Phase 8/9/10。

### 8.1 P0-1：stage_detector 支持真实 ai_project_template 目录结构

**目标：** 打开 `/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync` 原目录时，自动识别以下阶段：

- `src/python_model/L0_external`
- `src/python_model/L1_prototype`
- `src/python_model/L2_structured`
- `src/python_model/L3_pipeline`
- `src/python_model/L4_cycle_acc`
- `src/python_model/L5_fixedpoint`
- `src/python_model/L6_resource_opt`
- `src/verilog_model/rtl`

**做法方向（二选一，优先方案 A）：**

- **方案 A：扩展命名变体映射**
  - 将 `L0_external` 映射为 `L0`，`L1_prototype` 映射为 `L1`，依此类推。
  - `src/verilog_model/rtl` 映射为 `RTL`。
  - 保留对顶层 `L0` / `RTL` 等目录的支持，确保向后兼容。
- **方案 B：配置化阶段根目录**
  - 允许在 workspace profile 中配置阶段根目录，如 `{"python_stages_root": "src/python_model", "rtl_stages_root": "src/verilog_model/rtl"}`。
  - 适用于更灵活的项目模板，但实现更重。

**验收标准：**

- 对真实项目原目录执行 `build_workspace_profile`，`no_stage_found` 不出现。
- L0~L6、RTL 至少被识别为阶段，且文件数统计合理。
- 不修改目标项目目录。

### 8.2 P0-2：scanner 跳过噪声目录

**目标：** 减少 `scan_timeout` 与无效文件扫描，提升有效源码覆盖率。

**默认跳过目录：**

- `.git`
- `.claude`
- `__pycache__`
- `.pytest_cache`
- `.egg-info`
- `vivado`
- `reports`
- `stage_current`（如果包含生成物）
- `tests` 子目录中的 cache / sim_build / 生成物（可配置）

**验收标准：**

- 重新扫描真实项目，scan_timeout 警告数量显著下降（目标 < 5 条或完全消除）。
- 有效源码文件（`.py` / `.v` / `.sv`）仍被完整扫描。
- 不修改目标项目目录。

### 8.3 P0-3：dataflow / timing 最小非空生成

**目标：** 在 evidence / understanding 已存在时，生成保守但诚实的 dataflow 边和时序/流水边；无证据时输出 `evidence_gap` 或 `empty_reason`，不伪造边。

**做法方向：**

1. **dataflow view：**
   - 从 understanding 的 `claim` / `processing_step` / `interface` 中提取输入、输出、数据依赖。
   - 为每个处理步骤创建节点，为有明确数据关系的步骤创建边。
   - 若缺少接口或信号信息，则创建占位节点并标注 `evidence_gap`。
2. **timing view：**
   - 从 processing step 顺序、循环/流水线结构中提取时序阶段。
   - 生成阶段节点与顺序边；无法推断时序时输出 `empty_reason: 未识别到时序/流水结构`。

**验收标准：**

- 对 L0 / L1 / RTL 生成视图后，dataflow 或 timing 至少其一不再为空。
- 若为空，必须显示明确的 `empty_reason`（如 `evidence_gap: 未识别到接口/信号`）。
- 不伪造不存在的边。

### 8.4 P0-4：回归验收

**场景：** 使用真实项目原目录，不再创建临时顶层阶段目录。

**验收清单：**

- [ ] 直接打开真实项目，L0~L6、RTL 均被识别。
- [ ] 扫描警告显著减少（< 5 条 scan_timeout 或完全消除）。
- [ ] L0 / L1 / RTL 均可完成收集 evidence → 生成 understanding → 生成视图 → 生成质量报告。
- [ ] dataflow 或 timing view 至少其一非空，或显示明确 empty_reason。
- [ ] Quality Report 仍不使用 PASS/HOLD/正确/错误。
- [ ] 目标项目 `src/` 校验和前后一致，未修改。
- [ ] 如需 normalized mirror，必须位于 `/tmp` 或 app-owned 目录，并记录来源与 checksum。

### 8.5 本轮不进入的范围

- Phase 8 UI 重构（工作台、导航、美化）。
- Phase 9 真实 LLM 接入。
- Phase 10 Python-to-RTL 跨阶段映射。
- 大规模 understanding schema 重构（可在 P1 中评估，但不在 P0）。

### 4.5 P2 质量信号校准结果

P2 校准（2026-06-16 完成）：

| 校准项 | P1 行为 | P2 行为 | 评估 |
|--------|---------|---------|------|
| Python L0/L1 timing 空图 | `EmptyOrUnhelpfulView（Medium）` | `ExpectedEmptyTiming（Low）` | **已校准**：诚实空图不再触发高严重度问题 |
| RTL 有 clk/rst evidence timing | 非空 → 无 issue（正确） | 非空 → 无 issue（不变） | ✅ 延续 P0-3 行为 |
| node/edge 缺 trace_refs | `EmptyOrUnhelpfulView（Medium）` | `TraceabilityGap（Medium）` | **已校准**：分类标识更精准 |
| 孤立节点 | `EmptyOrUnhelpfulView（Low）` | `IsolatedOrUnconnectedView（Low/Medium）` | **已校准**：单独分类；>50% 孤立升 Medium |
| structure 多 summary 但单节点 | 无检测 | `LowSemanticDiversity（Medium）` | **新增**：reporter 级退化检测 |
| 标签高度重复 | 无检测 | `LowSemanticDiversity（Low）` | **新增**：view_evaluator 重复标签检测 |
| dataflow 非空 traceable | 无 issue（正确） | 无 issue（不变） | ✅ 延续 P0-3 行为 |
| 空视图（非 timing） | `EmptyOrUnhelpfulView（Medium）` | `EmptyOrUnhelpfulView（Medium）` | ✅ 未变：正常退化检测 |

**前端状态隔离**：全部通过验收。切换阶段、重新收集/生成/视图时，上一阶段 quality/trace/QA/understanding/views maps 正确清除；加载 session 后仅恢复当前阶段可持久化 UI state；所有异步请求使用 guard/version 防旧请求回写。

## 7. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-16 | **Batch D P2 质量信号校准完成**：新增 4 个 QualityIssueKind（ExpectedEmptyTiming/TraceabilityGap/IsolatedOrUnconnectedView/LowSemanticDiversity）；view_evaluator 校准——Python timing 仅预期空图不触发高严重度；frontend 状态隔离——maps/downstream 清除 + guard。文档 §4.1/§4.5 对应更新。未进入 Phase 8/9/10。 | Claude |
| 2026-06-15 | 创建 Phase 7 Batch D 真实项目质量基线报告，基于 `fpga_project_coarse_sync` 运行完整链路，记录 evidence / understanding / view / quality 退化项，提出 Batch D 修复优先级与 Phase 8/9/10 分界。 | Claude |
| 2026-06-15 | 安全收口修正：明确承认”曾向目标项目根目录写入临时阶段副本”属于验收方法偏差；修正”目标项目未被修改”为”src/ 校验和一致但存在目录写入偏差”；新增后续必须使用 /tmp normalized mirror 或修 stage_detector 的约束；新增 Batch D P0 修复计划（stage_detector 真实目录识别、scanner 噪声跳过、dataflow/timing 非空生成、回归验收）。 | Claude |
| 2026-06-16 | **Batch D P0-1/P0-2 完成**：修复 `stage_detector` 支持 ai_project_template 目录结构（`src/python_model/L0_external` -> L0、`src/verilog_model/rtl` -> RTL）；修复 `scanner` 跳过噪声目录（`.git`、`.claude`、`__pycache__`、`.pytest_cache`、`.mypy_cache`、`.ruff_cache`、`.egg-info`、`reports`、`vivado`、`build`、`dist`、`node_modules`、`target`、`.DS_Store`、`.idea`、`.vscode`、`.venv`、`venv`、`sim_build`、`.tox`、`.coverage`、`htmlcov`）；新增 Rust 单元测试 12 项（stage_detector）+ 7 项（scanner）；真实项目只读验证通过（src/ checksum 前后一致，未创建临时目录）；`cargo test --lib` 494 通过、`cargo check` 通过、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查无新增产品代码越界。 | Claude |
| 2026-06-16 | **Batch D P0-1/P0-2 审核收口（深层源码漏扫修复）**：发现并修复 P0-2 残留缺陷——原 `is_deep_source_dir` 仅匹配阶段根目录名本身，对阶段根之下的子孙目录（如 `src/python_model/L0_external/rx_02_coarse_sync/`）仍触发 `depth > 3` 拦截，导致真实深度 5 源码（`coarse_block.py`、`preamble.py`）漏扫并产生 `scan_timeout`。重写为 `is_deep_source_root` + `is_inside_deep_source_tree`：一旦进入 ai_project_template 深层源码树，其全部子孙目录不再受固定深度限制；噪声目录跳过在深层源码树之外仍生效。新增 scanner 测试 3 项（深度 5 源码扫描、子孙目录全递归、噪声目录深度限制仍生效）；新增 select_stage ai_project_template 布局端到端测试 3 项（L1、RTL、深度 5 源码进入 StageContext）。文档结构调整：新增 §0 当前状态（历史基线 vs 修复后状态分离），§1/§2.1/§3.1/§6 旧结论标注为历史基线并删除线标注已修复项。明确本轮不进入 P0-3、不进入 Phase 8/9/10。 | Claude |
| 2026-06-16 | **Batch D P0-3 完成（dataflow/timing 最小非空保守生成）**：根因——`MockProvider` 硬编码 `signal_summaries`/`interface_summaries`/`processing_steps` 为空数组，丢弃 evidence 已携带的 symbol/excerpt/source_kind，导致 dataflow/timing builder 无可派生输入而退化为空图。修复（保守、可追溯、不接真实 LLM）：(1) `MockProvider` 新增 `derive_conservative_summaries`，从 evidence context items 派生 `processing_steps`（Python 函数/类符号按 evidence 顺序，绑定 evidence_id，confidence=inferred，跳过 dunder/下划线前缀；RTL 不派生 step）、`signal_summaries`（RTL excerpt 识别 input/output/clk）；interface 本轮保守不派生。(2) `dataflow_builder`/`timing_builder` 顺序/时钟边改为从端点节点 trace_refs 合并派生（`merge_node_trace_refs`），消除原空 trace 推断边。(3) `timing_builder` 新增 RTL 时序保守回退（steps 空但有 clk/rst 信号时生成 ClockDomain/ResetDomain 节点）。新增 Rust 单测 14 项。真实项目只读验证（src/ SHA-256 前后一致 `090fd1f4...`，未创建临时目录）：L0 dataflow 12 节点/11 边、timing 12/11（基线 0/0）；L1 dataflow 9/8、timing 9/8；RTL dataflow 4/0、timing 2/0。**该条为历史记录：其中 L0/L1 timing 12/11 与 9/8 中的 PipelineStage 节点在后续 P0-3 收口修复中被判定为违规产物（Python 函数顺序不应产生时序图），已修正为 timing 0 节点/0 边 + 明确 empty_reason。** dataflow/timing 的 Medium `empty_or_unhelpful_view` 由修复前每阶段 3 条降为 0（RTL 残留 1 条 Low 孤立节点提示为诚实信号）。`cargo test --lib` 514 通过、`cargo check` 通过、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查无新增产品代码越界。**仍保持空图的诚实情形**：Python 阶段无时序证据时 timing 保持空（明确 empty_reason）；interface 本轮不派生。未进入 Phase 8/9/10。后续 P1（evidence 粒度细化、understanding 接口/信号语义补强）需单独授权。 | Claude |
| 2026-06-16 | **Batch D P0-3 收口修复（禁止 Python 函数顺序伪造成 timing 图）**：`timing_builder` 新增 `has_temporal_evidence` 门控：只有 step.description/name/claim/signal 中出现 cycle/latency/clock/pipeline/stage/clk/rst/posedge 等时序关键词，或 stage 明确含 RTL/pipeline/cycle 语义且 evidence 含时序内容时，才允许从 `processing_steps` 生成 `PipelineStage` 节点。对普通 L0/L1 Python 原型阶段，即使存在 `processing_steps`（由 MockProvider 从函数符号派生），也不能仅凭 order 生成 timing。若无时序依据，timing view 必须为空，并给出明确 `empty_reason`：“无 cycle/latency/clock/pipeline 等可追溯时序证据，未生成 timing 图（当前 processing_steps 为算法/函数顺序，非硬件时序）”。真实项目只读验证（`fpga_project_coarse_sync`，src/ SHA-256 前后一致，未创建临时目录）：L0 dataflow 12 节点/11 边（非空，正确）；timing 0 节点/0 边 + 明确 empty_reason（**历史违规产物：此前曾生成 12/11 伪 pipeline stage，已修正**）。L1 dataflow 9 节点/8 边（非空，正确）；timing 0 节点/0 边 + 明确 empty_reason（**历史违规产物：此前曾生成 9/8 伪 pipeline stage，已修正**）。RTL dataflow 4 节点/0 边；timing 2 节点/0 边（clk/rst 节点，无 pipeline 可连 → 孤立节点 Low 提示，属诚实信号而非伪造）。dataflow 的 Medium `empty_or_unhelpful_view` 保持 **0**；timing 的 Medium empty 在 L0/L1 为 **1**（诚实空图，非伪造），RTL 为 **0**（有 clk/rst 节点）。`cargo test --lib` 516 通过（1 ignored）、`cargo check` 0 warning、`npm run build` 通过、`npx tsc --noEmit` 通过；rg 边界检查无新增越界。未进入 Phase 8/9/10。 | Claude |
| 2026-06-16 | **Batch D P0-4 综合回归验收**：新增集成测试 `tests/real_project_validation.rs`（5 项测试：主样本 L0/L1/RTL 阶段检测、副样本阶段检测、深层扫描无 timeout、噪声目录跳过、checksum 一致性）。主样本 `fpga_project_coarse_sync`：识别 8 阶段（L0~L6+RTL），扫描 152 文件，timeout 0，深层文件（depth 5）10 个全部找到（`rx_02_coarse_sync`、`shared_04_preamble`），噪声目录（`__pycache__`/`.git`/`.claude`/`vivado`/`node_modules`/`target`）全部跳过，Python 101 个/Verilog 8 个，checksum 48 文件前后一致。副样本 `fpga_project_fft`：识别 7 阶段（L0~L5+RTL），扫描 60 文件，checksum 前后一致。全量测试：`cargo test --lib` 516 通过（1 ignored）、集成测试 5 通过（0 ignored）、`cargo check` 0 warning、`npm run build` 通过、`npx tsc --noEmit` 通过。rg 边界检查：产品代码无 `std::fs::write`/`create_dir`/`remove_file`/`remove_dir`/`rename`/`copy`/`Command::new`（测试代码除外）；无 Vivado/synthesis/implementation/bitstream（仅 quality/mod.rs 注释说明不做）；无 OpenAI/Anthropic/api_key；无 PASS/HOLD/审计用语（"正确"/"错误"仅出现在代码注释/错误处理语境，非审计裁决）。文档更新：`docs/testing/phase-7-real-project-quality-validation.md` 追加 P0-4 验收记录；`docs/planning/phase-7-real-project-gap-report.md` 追加 P0-4 验收记录。本轮不改产品代码，仅新增集成测试与文档。未进入 Phase 8/9/10。P0-4 验收结论：P0-1/P0-2/P0-3 合并后真实项目效果达标，允许进入 P1（evidence 粒度细化、understanding 接口/信号语义补强），但 P1 需单独授权。 | Claude |
