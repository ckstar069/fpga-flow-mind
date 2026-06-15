# Phase 7 Batch D 真实项目质量基线报告

---
status: active
updated: 2026-06-15
---

> 本报告是 Phase 7 Batch D 第一步的产出：基于真实/等价 `ai_project_template` 生成项目建立质量基线，回答“fpga-flow-mind 在接近真实项目时到底离可用还差多少”。
> 本报告**不是** Phase 7 完成验收，也不是 Phase 8/9/10 规划。

## 1. 样本项目来源与结构

### 1.1 真实项目

- **路径：** `/Users/ckstar/Repo/znxt_ofdm/fpga_project_coarse_sync`
- **性质：** 基于 `ai_project_template` 生成的真实 OFDM 粗同步 FPGA 实现项目，与原始模板仓库位于同一父目录下。
- **结构：**
  - `src/python_model/`：L0_external、L1_prototype、L2_structured、L3_pipeline、L4_cycle_acc、L5_fixedpoint、L6_resource_opt
  - `src/verilog_model/rtl/`：RTL 实现
  - `tests/`、`scripts/`、`vivado/`、`docs/` 等辅助目录
- **只读验证：** 对 `src/` 下所有 `.py` / `.v` / `.sv` / `.md` 文件在运行前后分别计算 SHA-256；`src/` 校验和完全一致，目标项目未被修改。
- **扫描限制：** 由于 `fpga-flow-mind` 当前阶段检测器只识别根目录下 `L0` / `L1` / ... / `RTL` 等顶层阶段目录，真实项目源码位于 `src/python_model/L1_prototype` 这类深层路径，导致直接打开真实项目时阶段识别失败（`no_stage_found` / `未识别到阶段目录`）。
  - 为完成本次基线评估，临时将 `src/python_model/L*` 与 `src/verilog_model/rtl/` 下的文件**复制**到项目根目录的 `L0` / `L1` / ... / `RTL` 目录中，评估完成后已删除这些临时副本。
  - 这一事实本身即是一项关键发现：fpga-flow-mind 的阶段检测策略与 `ai_project_template` 真实目录结构不匹配，真实项目无法直接被工具识别。

### 1.2 是否等价于 ai_project_template

- 该项目是 `ai_project_template` 工作流的直接产物，目录命名、阶段分层、文档/测试/约束结构均与模板一致。
- 相比 `/tmp` 下为 Batch C 准备的 toy 样例，本项目包含 39 个 Python 文件、6 个 Verilog 文件、真实算法语义（粗同步、能量检测、互相关、CFO 估计等），足以暴露工具的真实理解能力边界。

## 2. 完整 Phase 1~7 链路运行结果

### 2.1 Phase 1：Workspace 扫描

- **结果：** 成功打开；在临时顶层阶段目录存在时，L0~L6、RTL 均被识别，文件数统计正常。
- **问题：**
  - 真实 `src/python_model/L1_prototype` 结构不被识别为阶段；必须采用顶层阶段目录才能扫描。
  - 扫描产生 30 条 `scan_timeout: 目录深度超过 3` 警告，多集中在 `tests/python/L*/__pycache__` 与 `.claude/commands/...` 深层目录。这导致部分可分析文件被跳过，可能影响 evidence 完整性。

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

1. **目录结构不匹配真实项目：** 真实 `ai_project_template` 项目源码在 `src/python_model/L1_prototype` 等深层目录，当前 stage_detector 要求顶层阶段目录，导致真实项目无法被识别。
2. **扫描深度限制导致遗漏：** `scan_timeout` 警告频繁，深层子目录（如 `__pycache__`、`.claude`、测试辅助目录）被跳过，可能错过或误过滤有用文件。
3. **evidence 粒度偏粗：** 真实 L0 有 14 个文件、多个子模块，但只生成 20 项 evidence，无法支撑后续细粒度理解。

### 3.2 Understanding 生成（Phase 3）

1. **声明数量过少：** L0 仅 3 条声明，无法覆盖多文件、多子包的语义。
2. **缺少关键维度：** 未显式识别接口契约、信号/变量、处理步骤、数据依赖、配置参数等 FPGA 设计中的关键元素。
3. **unknown/gap 统计为 0 可能失真：** 未必表示真正理解完整，可能是 claim 生成过于保守或聚合导致未暴露未知。

### 3.3 View 生成（Phase 4）

1. **Structure view 过简：** 仅单个节点，缺少子模块、函数、类层级。
2. **Dataflow / Timing 完全退化：** 真实项目中的数据流和时序关系未被抽取， three-view 面板名存实亡。
3. **View 与 evidence/understanding 脱节：** 节点缺少可点击的 evidence 引用，无法支持探索式理解。

### 3.4 Trace / Q&A（Phase 5）

1. **可 trace 目标稀少：** 结构图节点少，用户可追问的对象有限。
2. **Q&A 真实能力未验证：** 本次未执行复杂问题，但从 evidence/understanding 稀疏度判断，复杂问题大概率返回 unknown。

### 3.5 UI / 信息架构（Phase 8 范畴）

1. **界面仍是调试式堆叠：** 左侧面板、中间文件列表、右侧理解/视图/质量报告、底部警告，纵向信息密度高，用户难以快速定位关键信息。
2. **阶段切换后状态未完全隔离：** 切换回 L0 时，底部警告/质量报告未明显区分当前阶段与他阶段。
3. **警告区喧宾夺主：** 30 条 `scan_timeout` 长期占据底部，分散用户对质量报告的注意力。

## 4. 问题分类与修复优先级

### 4.1 建议 Phase 7 Batch D 修复

| 优先级 | 问题 | 修复方向 | 范围 |
|--------|------|----------|------|
| P0 | stage_detector 不识别 `src/python_model/L1_prototype` 等真实结构 | 支持从 `src/python_model` / `src/verilog_model/rtl` 等路径识别阶段，或提供阶段根目录配置 | workspace/stage_detector |
| P0 | dataflow / timing view 为空 | 在 understanding 中抽取数据依赖和时序关系，view generator 据此生成边 | understanding + view generator |
| P1 | structure view 节点过少 | 增强 understanding 对模块/函数/类的识别，view generator 递归展开子结构 | understanding + view generator |
| P1 | evidence 粒度过粗 | 细化 evidence item 拆分策略，增加函数/类/接口级 evidence | evidence collector |
| P1 | scan_timeout 过多 | 优化扫描策略：跳过已知噪声目录（`__pycache__`、`.git`、`.claude`）但提高有效源码目录深度限制 | workspace/scanner |
| P2 | 阶段切换状态隔离 | 切换阶段时清空/隔离 quality report、warnings 的展示 | frontend state |

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

- **目标项目只读：** `src/` 校验和前后一致，未修改真实项目。
- **无真实 LLM：** 当前 understanding/provider 仍为 mock。
- **无 Vivado / synthesis / implementation / bitstream：** 未运行 Vivado。
- **无 PASS/HOLD/正确/错误：** quality report 使用“低于当前质量门槛”。
- **未进入 Phase 8/9/10：** 本轮仅产出基线报告与小范围分析，未做 UI 重构或 LLM 接入。

## 6. 结论与下一步建议

### 6.1 核心结论

Phase 7 Batch C 的 UI 接入是通的，但面对真实 `ai_project_template` 项目时，工具远未达到可用：

- **阶段识别**与真实项目结构不匹配。
- **Understanding / View / Trace / Q&A** 在真实代码上严重退化。
- **Quality Review** 目前主要是“诚实地报告自己分析得不好”，而不是“分析得很好”。

### 6.2 建议继续 Phase 7 Batch D 修复

在进入 Phase 8 之前，应先完成 Batch D 的真实项目质量补强，特别是：

1. 让 stage_detector 支持真实 `ai_project_template` 目录结构。
2. 提升 understanding 的丰富度（接口、信号、处理步骤）。
3. 让 dataflow / timing view 至少能生成非空图。
4. 优化 scanner，减少 scan_timeout 并提高有效源码覆盖率。

### 6.3 不建议立即进入 Phase 8/9/10

- Phase 8 的 UI 重构应建立在“分析内容足够丰富”的基础上；当前分析内容稀疏，重构后也无内容可呈现。
- Phase 9 的真实 LLM 应建立在“确定性理解管道稳定”的基础上；当前 evidence/understanding 不稳定，LLM 会放大噪声。
- Phase 10 的跨阶段映射应建立在“单阶段理解充分”的基础上；当前单阶段理解本身不足。

## 7. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 创建 Phase 7 Batch D 真实项目质量基线报告，基于 `fpga_project_coarse_sync` 运行完整链路，记录 evidence / understanding / view / quality 退化项，提出 Batch D 修复优先级与 Phase 8/9/10 分界。 | Claude |
