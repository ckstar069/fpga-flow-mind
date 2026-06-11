# Phase 2 Evidence Collection 验证计划

---
status: active
updated: 2026-06-11
---

> 本文档定义 Phase 2 evidence collection 的验证策略、测试矩阵和验收标准。
> 不写产品代码。验证范围覆盖 Rust 后端单元测试、集成测试、前端组件测试和安全回归测试。

## 1. 验证目标

确认 Phase 2 证据收集功能满足以下条件：

1. 从有效 `StageContext` 中正确提取 evidence item
2. 每条 evidence item 的字段完整且符合约束
3. 索引覆盖所有 evidence item
4. 错误和不确定项正确表达
5. 目标项目文件系统无副作用
6. 前端正确展示 evidence 面板
7. Phase 1 功能无回归

## 2. 测试层次

| 层次 | 工具 | 数量（预估） | 覆盖范围 |
|------|------|-------------|----------|
| Rust 单元测试 | `cargo test` | ~40 | models、id_generator、excerpt、index_builder、各语言提取器 |
| Rust 集成测试 | `cargo test` | ~10 | collect_evidence command、collector 集成、端到端收集 |
| 前端组件测试 | Vitest + RTL | ~8 | EvidencePanel、EvidenceItemCard、CollectEvidenceButton |
| 手工验收 | `cargo tauri dev` | 1 次 | 完整 UI 链路验证 |

## 3. 测试夹具设计

### 3.1 夹具结构

复用 Phase 1 的测试夹具生成模式，为 Phase 2 创建专用夹具：

```text
/tmp/fpga-flow-mind-phase2-test-<random>/
├── L0/
│   ├── top.py                # Python — 含 def + class
│   ├── top_interface.py      # Python — 含 def
│   └── constants.py          # Python — 仅赋值，无 def/class
├── L1/
│   ├── model.py              # Python — 含 class + 嵌套 def
│   └── helpers.py            # Python — 含多个 def
├── RTL/
│   ├── top.v                 # Verilog — 含完整 module
│   ├── alu.v                 # Verilog — 含 module + assign
│   └── empty_module.v        # Verilog — 空 module
├── L3/                        # empty
├── constraints/
│   └── timing.xdc            # XDC — 含 create_clock、set_property
├── scripts/
│   └── build.tcl             # TCL — 含 proc
├── docs/
│   ├── readme.md             # Markdown — 含多级标题
│   └── design.md             # Markdown — 含 # ## ### 结构
├── binary/
│   └── firmware.bin          # 二进制文件
├── large/
│   └── huge_file.v           # >5MB 的 Verilog 文件
├── encoding/
│   └── latin1.v              # 非 UTF-8 编码文件
└── README.md                 # 根目录 Markdown
```

### 3.2 夹具文件内容

#### `L0/top.py`

```python
"""Top-level signal processing module."""
import numpy as np

def process_signal(data, sample_rate):
    """Process incoming signal data."""
    normalized = normalize(data)
    filtered = apply_filter(normalized, sample_rate)
    return filtered

def normalize(data):
    """Normalize signal data."""
    max_val = max(abs(data))
    return [x / max_val for x in data]

class SignalProcessor:
    """Main signal processor class."""
    def __init__(self, config):
        self.config = config
        self.buffer = []

    def process(self, input_data):
        result = process_signal(input_data, self.config['sample_rate'])
        self.buffer.append(result)
        return result
```

**预期 evidence item**（Python 提取器）：

| evidence_id | symbol | line_range | strength |
|-------------|--------|------------|----------|
| EV-L0-000001 | `process_signal` | {4, 8} | direct |
| EV-L0-000002 | `normalize` | {10, 12} | direct |
| EV-L0-000003 | `SignalProcessor` | {14, 22} | direct |

#### `RTL/top.v`

```verilog
module top(
    input wire clk,
    input wire rst,
    input wire [7:0] data_in,
    output wire [7:0] data_out,
    output wire valid
);

wire [7:0] processed;
assign processed = data_in ^ 8'hFF;
assign data_out = processed;
assign valid = ~rst;

endmodule
```

**预期 evidence item**（Verilog 提取器）：

| evidence_id | symbol | line_range | strength |
|-------------|--------|------------|----------|
| EV-RTL-000001 | `top` | {1, 14} | direct |

#### `docs/readme.md`

```markdown
# Project Documentation

## Overview

This is the main project documentation.

## Architecture

### L0 - Signal Processing

Signal processing description.

### RTL - Hardware

Hardware description.
```

**预期 evidence item**（Markdown 提取器）：

| evidence_id | symbol | line_range | strength |
|-------------|--------|------------|----------|
| EV-docs-000001 | `Project Documentation` | {1, 14} | indirect (章节范围推断) |

## 4. Rust 单元测试矩阵

### 4.1 EvidenceId 生成器

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| ID-01 | 格式正确 | stage_id="L0" | `"EV-L0-000001"` |
| ID-02 | 连续递增 | 3 次调用 | `000001`, `000002`, `000003` |
| ID-03 | 不同 stage | stage_id="RTL" | `"EV-RTL-000001"` |
| ID-04 | 唯一性 | 1000 次调用 | 所有 ID 不同 |

### 4.2 Excerpt 处理

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| EX-01 | 短文本 | 100 字符 | 原样返回 |
| EX-02 | 恰好 500 | 500 字符 | 原样返回 |
| EX-03 | 超出截断 | 600 字符, 20 行 | 前 400 + `"...(已截断，共 20 行)"` |
| EX-04 | 整文件摘要 | 1000 字符, 50 行 | 前 200 + `"...(共 50 行)"` |
| EX-05 | 空内容 | 0 字符 | `""` |

### 4.3 Index Builder

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| IX-01 | 索引覆盖 | 3 items | 每个 id 出现在 by_path 和 by_kind 中 |
| IX-02 | symbol 索引 | 2 items 有 symbol, 1 无 | by_symbol 只有 2 条 |
| IX-03 | 空输入 | 0 items | 三个索引均为空 |
| IX-04 | 同文件多 item | 3 items 同 source_path | by_path[file] = [id1, id2, id3] |

### 4.4 Python 提取器

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| PY-01 | 函数定义 | `def foo():\n    pass` | 1 item: symbol=foo, range={1,2} |
| PY-02 | 类定义 | `class Bar:\n    pass` | 1 item: symbol=Bar |
| PY-03 | 多函数 | 3 个 `def` | 3 items |
| PY-04 | 嵌套函数 | 函数内 `def` | 外层 range 包含内层 |
| PY-05 | 空文件 | `""` | 0 items |

### 4.5 Verilog 提取器

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| VR-01 | 单 module | `module top(...);\n...\nendmodule` | 1 item |
| VR-02 | 多 module | 2 个 module | 2 items |
| VR-03 | 空 module | `module empty();\nendmodule` | 1 item: range={1,2} |
| VR-04 | 无 module | 只有 `assign` | 0 items |

### 4.6 SystemVerilog 提取器

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| SV-01 | module | `module top;\nendmodule` | 1 item |
| SV-02 | interface | `interface bus;\nendinterface` | 1 item |
| SV-03 | class | `class Packet;\nendclass` | 1 item |

### 4.7 Markdown 提取器

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| MD-01 | 多级标题 | `#\n##\n###` | 3 items |
| MD-02 | 空文件 | `""` | 0 items |
| MD-03 | 纯文本 | 无标题行 | 0 items |

### 4.8 Config/TCL 提取器

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| CF-01 | TCL proc | `proc build {} { ... }` | 1 item |
| CF-02 | XDC 约束 | `create_clock -period 10` | 1 item (indirect) |
| CF-03 | 空文件 | `""` | 0 items |

## 5. Rust 集成测试矩阵

### 5.1 collect_evidence Command

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| CE-01 | 正常收集 | L0 阶段（3 个 Python 文件） | success=true, items > 0, 索引完整 |
| CE-02 | 空阶段 | L3 目录 | success=false, error=stage_empty |
| CE-03 | 阶段不存在 | stage_id="XYZ" | success=false, error=no_stage_found |
| CE-04 | 路径不存在 | 不存在的 root_path | success=false, error=path_not_found |
| CE-05 | 混合语言 | 含 Python + Verilog + Markdown 阶段 | 各提取器处理，索引正确 |
| CE-06 | 大文件跳过 | 含 >5MB 文件 | success=true, warning file_too_large, items 不含该文件 |
| CE-07 | 非 UTF-8 | 含二进制文件 | success=true, warning non_utf8_file_skipped |
| CE-08 | warnings 传播 | 2 个文件不可读 | warnings 有 2 条，其他文件正常提取 |
| CE-09 | 空结果 | 只有不可读文件 | success=true, items=[], warnings 非空 |
| CE-10 | evidence_id 唯一 | 任意阶段 | 所有 evidence_id 互不相同 |

## 6. 前端组件测试矩阵

### 6.1 CollectEvidenceButton

| ID | 用例 | 初始状态 | 操作 | 预期 |
|----|------|----------|------|------|
| FB-01 | 空阶段 | files=[] | 渲染 | 按钮不显示或 disabled |
| FB-02 | 点击收集 | idle 状态 | 点击按钮 | 调用 collectEvidence(), 进入 loading |
| FB-03 | 收集成功 | loading | 后端返回成功 | 切换到 done, 显示证据数量 |
| FB-04 | 收集失败 | loading | 后端返回错误 | 切换到 error, 显示错误信息 |

### 6.2 EvidencePanel

| ID | 用例 | 输入 | 预期 |
|----|------|------|------|
| FP-01 | 正常展示 | 10 条 evidence | 列表显示 10 个卡片 |
| FP-02 | 空结果 | 0 条 evidence | 显示"未收集到证据" |
| FP-03 | 统计展示 | EvidenceStats | 正确显示总数、按类型、按 strength 分组 |
| FP-04 | Warning 展示 | 2 条 warning | 折叠状态下显示计数，展开后显示详情 |

## 7. 安全回归测试

### 7.1 文件系统只读验证

| ID | 检查项 | 验证方式 |
|----|--------|----------|
| SR-01 | 无写入 API | `rg "std::fs::write\|std::fs::create_dir\|std::fs::remove_file\|std::fs::rename\|std::fs::copy" src-tauri/src/evidence/` → 无匹配 |
| SR-02 | 无进程执行 | `rg "std::process\|Command::new" src-tauri/src/evidence/` → 无匹配 |
| SR-03 | 无 Vivado 调用 | `rg "vivado\|synthesis\|implementation\|bitstream" src-tauri/src/evidence/` → 无匹配 |
| SR-04 | 目标目录无变化 | 收集前后 `git status` 无变化 |
| SR-05 | Symlink 安全校验 | 根路径 symlink 仍被拒绝 |

### 7.2 Phase 1 功能回归

| ID | 检查项 | 验证方式 |
|----|--------|----------|
| RG-01 | Phase 1 测试仍通过 | `cd src-tauri && cargo test` 全部 passed（含原有 65 个 + 新增） |
| RG-02 | Phase 1 UI 正常 | `cargo tauri dev` 中 open_workspace + select_stage 正常 |
| RG-03 | Phase 1 错误码不变 | 原有 9 个 error_code 语义不变 |

## 8. 手工验收

### 8.1 验收环境

使用 Phase 1 验收样例目录 + Phase 2 新增夹具文件。

### 8.2 验收步骤

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 启动 `cargo tauri dev` | 应用正常启动 |
| 2 | 输入路径，点击"打开项目" | Workspace 概览正常 |
| 3 | 选择 L0 阶段 | 阶段详情展示，出现"收集证据"按钮 |
| 4 | 点击"收集证据" | 按钮变为 loading，收集完成后切换到证据 tab |
| 5 | 查看证据面板 | 统计概要、evidence item 列表、strength 标签 |
| 6 | 切换筛选维度 | 按文件/类型/符号筛选正常 |
| 7 | 展开 warnings | 警告列表正确展示 |
| 8 | 选择 L3 (空阶段) | 不出现"收集证据"按钮 |
| 9 | 选择 RTL | 收集 Verilog evidence，module 正确提取 |
| 10 | 检查目标目录 | 目标目录无变化 |

## 9. Phase 1 样例复用

Phase 2 测试夹具在 Phase 1 样例基础上扩展，复用策略：

| Phase 1 样例 | Phase 2 扩展 |
|-------------|-------------|
| `L0/top.py` (15B) | 替换为含 def/class 的完整 Python 文件 |
| `L0/top_interface.py` (10B) | 替换为含 def 的 Python 文件 |
| `L1/model.py` (14B) | 替换为含 class 的 Python 文件 |
| `rtl_final/top.v` (22B) | 替换为含完整 module 的 Verilog 文件 |
| `docs/readme.md` | 替换为含多级标题的 Markdown |
| 新增 | `constraints/`, `scripts/`, `binary/`, `large/`, `encoding/` 目录 |

## 10. 验收标准总结

| 类别 | 标准 | 验证方式 |
|------|------|----------|
| Rust 单元测试 | 所有测试通过（~40 个） | `cargo test` |
| Rust 集成测试 | 所有测试通过（~10 个） | `cargo test` |
| 前端构建 | `npm run build` 通过 | CI |
| 安全约束 | 无写入/执行 API 调用 | `rg` 检查 |
| Phase 1 回归 | 原有 65 个测试仍通过 | `cargo test` |
| 手工验收 | 10 步验收全部通过 | `cargo tauri dev` |
| 代码越界 | 无 Phase 3+ 关键字（LLM/Q&A/graph） | `rg` 检查 |

## 11. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-11 | 初始创建 | Claude |
