# Phase 2 Evidence Collector 后端设计

---
status: draft
updated: 2026-06-11
---

> 本文档设计 Phase 2 evidence collector 的后端模块布局、`collect_evidence` command 设计、文件读取策略、代码分块策略、错误处理和单元测试设计。
> 不写产品代码。设计约束来自 [`phase-2-evidence-model.md`](phase-2-evidence-model.md) 和 [`phase-2-evidence-requirements.md`](../requirements/phase-2-evidence-requirements.md)。

## 1. 模块布局

### 1.1 新增 Rust 模块

```text
src-tauri/src/
├── commands/
│   ├── mod.rs              # 已有
│   ├── open_workspace.rs   # 已有 (Phase 1)
│   ├── select_stage.rs     # 已有 (Phase 1)
│   └── collect_evidence.rs # Phase 2 新增 — collect_evidence command 入口
├── evidence/               # Phase 2 新增模块目录
│   ├── mod.rs              # 模块声明
│   ├── collector.rs         # 核心收集调度器
│   ├── models.rs            # EvidenceItem / EvidenceCollection / EvidenceStrength 枚举
│   ├── id_generator.rs      # evidence_id 生成器
│   ├── excerpt.rs           # summary 提取与截断逻辑
│   ├── index_builder.rs     # index_by_path / index_by_kind / index_by_symbol 构建
│   └── extractors/          # 按语言类型的提取器
│       ├── mod.rs           # 提取器 trait + 分派逻辑
│       ├── python.rs        # Python 提取器 (def / class)
│       ├── verilog.rs       # Verilog 提取器 (module / endmodule)
│       ├── systemverilog.rs # SystemVerilog 提取器 (module / class / interface)
│       ├── markdown.rs      # Markdown 提取器 (标题章节)
│       └── config.rs        # 配置文件提取器 (TCL/XDC 约束等)
├── models/                  # 已有 (Phase 1 共享模型)
│   └── ...
└── workspace/               # 已有 (Phase 1)
    └── ...
```

### 1.2 模块职责

| 模块 | 职责 | 依赖 |
|------|------|------|
| `commands/collect_evidence.rs` | Tauri command 入口，参数校验，调用 collector，返回 `CommandResult<EvidenceCollection>` | `evidence::collector` |
| `evidence/collector.rs` | 遍历 `StageContext.files[]`，分派到各提取器，组装 `EvidenceCollection` | `evidence::extractors`, `evidence::id_generator`, `evidence::index_builder`, `evidence::models` |
| `evidence/models.rs` | `EvidenceItem`, `EvidenceCollection`, `EvidenceStrength`, `EvidenceWarning`, `EvidenceStats`, `LineRange` 定义 | 无外部依赖 |
| `evidence/id_generator.rs` | 生成 `EV-<stage_id>-<6位序号>` 格式的唯一 ID | 无外部依赖 |
| `evidence/excerpt.rs` | 读取文件片段、截断、非 UTF-8 处理 | 无外部依赖 |
| `evidence/index_builder.rs` | 从 `Vec<EvidenceItem>` 构建 `index_by_path`、`index_by_kind`、`index_by_symbol` | `evidence::models` |
| `evidence/extractors/mod.rs` | 定义 `EvidenceExtractor` trait，按 `language` 分派 | `evidence::models` |
| `evidence/extractors/python.rs` | Python `def`/`class` 关键字提取 | `evidence::models` |
| `evidence/extractors/verilog.rs` | Verilog `module`/`endmodule` 提取 | `evidence::models` |
| `evidence/extractors/systemverilog.rs` | SystemVerilog 提取 | `evidence::models` |
| `evidence/extractors/markdown.rs` | Markdown 标题章节提取 | `evidence::models` |
| `evidence/extractors/config.rs` | TCL/XDC 约束文件提取 | `evidence::models` |

## 2. collect_evidence Command 设计

### 2.1 Command 签名

```rust
#[tauri::command]
fn collect_evidence(
    root_path: String,
    stage_id: String,
) -> Result<CommandResult<EvidenceCollection>, String>
```

### 2.2 TypeScript 调用端

```typescript
// src/lib/tauriCommands.ts 新增
export async function collectEvidence(
  rootPath: string,
  stageId: string
): Promise<EvidenceCollection> {
  const result = await invoke<CommandResult<EvidenceCollection>>('collect_evidence', {
    rootPath,
    stageId,
  });
  return handleResult(result);
}
```

> **注意 Tauri v2 参数命名**：JS invoke 参数使用 camelCase（`rootPath`、`stageId`），与 Phase 1 bug 修复保持一致。

### 2.3 执行流程

```text
collect_evidence(root_path, stage_id)
  │
  ├─ 1. 路径校验（复用 safety_guard）
  │     root_path 不存在 → CommandError { path_not_found }
  │     root_path 不是目录 → CommandError { not_directory }
  │     root_path 是 symlink → CommandError { permission_denied }
  │
  ├─ 2. 获取 StageContext（复用 select_stage 逻辑）
  │     stage_id 不存在 → CommandError { no_stage_found }
  │     stage 为空 → CommandError { stage_empty }
  │
  ├─ 3. 初始化收集器
  │     创建 EvidenceCollector::new(stage_id)
  │     初始化 counters (processed=0, skipped=0, item_count=0)
  │
  ├─ 4. 遍历 StageContext.files[]
  │     for each file in files:
  │       ├─ 4a. 文件预检
  │       │     不存在 → warning { file_unreadable }, skipped++, continue
  │       │     size > 5MB → warning { file_too_large }, skipped++, continue
  │       │     非 UTF-8 → warning { non_utf8_file_skipped }, skipped++, continue
  │       │     二进制文件 → warning { binary_file_skipped }, skipped++, continue
  │       │
  │       ├─ 4b. 读取文件内容（只读）
  │       │     std::fs::read_to_string(source_path)
  │       │
  │       ├─ 4c. 按 language 分派到对应提取器
  │       │     Python → python_extractor.extract(content, source_path, language, source_kind)
  │       │     Verilog → verilog_extractor.extract(...)
  │       │     SystemVerilog → systemverilog_extractor.extract(...)
  │       │     Markdown → markdown_extractor.extract(...)
  │       │     Config → config_extractor.extract(...)
  │       │     其他 → 整文件级 evidence (strength=indirect)
  │       │
  │       ├─ 4d. 收集提取结果
  │       │     为每个提取结果分配 evidence_id
  │       │     生成 summary（调用 excerpt 模块截断）
  │       │     设置 strength
  │       │     加入 evidence_items[]
  │       │
  │       └─ 4e. processed++
  │
  ├─ 5. 构建索引
  │     index_builder::build(evidence_items)
  │       → index_by_path
  │       → index_by_kind
  │       → index_by_symbol
  │
  ├─ 6. 计算统计
  │     EvidenceStats { files_processed, files_skipped, total_items, items_by_kind, items_by_strength }
  │
  └─ 7. 返回 EvidenceCollection
        CommandResult { success=true, data=Some(EvidenceCollection), warnings }
```

## 3. EvidenceExtractor Trait

```rust
/// 证据提取器 trait
trait EvidenceExtractor {
    /// 从文件内容中提取 evidence
    /// content: 文件全文
    /// source_path: 文件绝对路径
    /// language: 文件语言
    /// source_kind: 文件来源类型
    /// 返回: Vec<RawExtraction>（未经 ID 分配和截断的原始提取结果）
    fn extract(
        &self,
        content: &str,
        source_path: &str,
        language: Language,
        source_kind: SourceKind,
    ) -> Vec<RawExtraction>;
}

/// 原始提取结果（ID 和 summary 截断在 collector 层统一处理）
struct RawExtraction {
    /// 符号名称
    symbol: Option<String>,
    /// 行号范围
    line_range: LineRange,
    /// 原始代码片段（可能超过 500 字符，由 excerpt 模块截断）
    raw_excerpt: String,
    /// 证据强度
    strength: EvidenceStrength,
}
```

### 3.1 提取器分派逻辑

```rust
fn dispatch_extractor(language: &Language) -> Box<dyn EvidenceExtractor> {
    match language {
        Language::Python => Box::new(PythonExtractor),
        Language::Verilog => Box::new(VerilogExtractor),
        Language::SystemVerilog => Box::new(SystemVerilogExtractor),
        Language::Markdown => Box::new(MarkdownExtractor),
        Language::Tcl | Language::Xdc => Box::new(ConfigExtractor),
        _ => Box::new(FallbackExtractor), // 整文件级证据
    }
}
```

## 4. 文件读取策略

### 4.1 文件大小限制

| 条件 | 处理 |
|------|------|
| 文件 ≤ 5MB | 正常读取全文 |
| 文件 > 5MB | 跳过，生成 `file_too_large` warning |

### 4.2 编码处理

| 条件 | 处理 |
|------|------|
| UTF-8 | 正常读取 |
| 非 UTF-8 | 跳过，生成 `non_utf8_file_skipped` warning |

### 4.3 二进制检测

使用简单启发式：检查文件扩展名是否属于二进制类型（Phase 1 `file_classifier` 已维护跳过列表），或前 8KB 中是否包含 NULL 字节。

### 4.4 安全约束

**只允许的文件系统操作**：

| 操作 | 用途 |
|------|------|
| `std::fs::metadata(path)` | 检查文件存在性、大小 |
| `std::fs::read_to_string(path)` | 读取文本文件内容 |
| `std::fs::read_dir(dir)` | 列出目录内容（复用 Phase 1 scanner） |

**禁止的操作**（与 Phase 1 一致）：

| 禁止操作 | 说明 |
|----------|------|
| `std::fs::write` | 不写入 |
| `std::fs::create_dir` | 不创建目录 |
| `std::fs::remove_file` | 不删除文件 |
| `std::fs::rename` | 不重命名 |
| `std::fs::copy` | 不复制 |
| `std::process::Command` | 不执行外部命令 |
| `Command::new` | 不执行外部进程 |

## 5. 代码分块策略

### 5.1 Python 提取规则

| 目标 | 匹配模式 | strength |
|------|----------|----------|
| 函数定义 | `^def\s+(\w+)\s*\(` | `direct` |
| 类定义 | `^class\s+(\w+)` | `direct` |
| 函数边界推断 | 基于 `def` 行开始，下一个同级 `def` 或 EOF 结束 | `indirect` |

**函数边界推断算法**：
1. 找到所有 `def` 行，记录行号和缩进级别
2. 对每个 `def`，其 `end` 为下一个同级或更低缩进的 `def` 的前一行，或 EOF
3. 顶层 `def` 的缩进级别为 0

**示例**：

```python
# Line 1:  def foo():        → symbol="foo", line_range={1,5}, strength=direct
# Line 2:      x = 1
# Line 3:      return x
# Line 4:
# Line 5:  def bar(a, b):    → symbol="bar", line_range={5,8}, strength=direct
# Line 6:      c = a + b
# Line 7:      return c
# Line 8:
```

### 5.2 Verilog 提取规则

| 目标 | 匹配模式 | strength |
|------|----------|----------|
| module 定义 | `^\s*module\s+(\w+)` | `direct` |
| module 结束 | `^\s*endmodule` | `direct` |
| input 声明 | `^\s*input\s+` | `indirect` |
| output 声明 | `^\s*output\s+` | `indirect` |
| assign 语句 | `^\s*assign\s+` | `indirect` |

**module 边界算法**：
1. 找到 `module <name>` 行 → `start`
2. 找到对应的 `endmodule` 行 → `end`
3. `line_range = {start, end}`

**示例**：

```verilog
// Line 1:  module top(           → symbol="top", line_range={1,10}, strength=direct
// Line 2:      input clk,
// Line 3:      input rst,
// Line 4:      output [7:0] data_out
// Line 5:  );
// Line 6:  wire [7:0] internal;
// Line 7:  assign data_out = internal;
// Line 8:
// Line 9:  endmodule
```

### 5.3 SystemVerilog 提取规则

| 目标 | 匹配模式 | strength |
|------|----------|----------|
| module 定义 | `^\s*module\s+(\w+)` | `direct` |
| interface 定义 | `^\s*interface\s+(\w+)` | `direct` |
| class 定义 | `^\s*class\s+(\w+)` | `direct` |
| package 定义 | `^\s*package\s+(\w+)` | `direct` |
| endmodule/endinterface/endclass/endpackage | 对应结束关键字 | `direct` |

### 5.4 Markdown 提取规则

| 目标 | 匹配模式 | strength |
|------|----------|----------|
| 一级标题 | `^#\s+(.+)` | `direct` |
| 二级标题 | `^##\s+(.+)` | `direct` |
| 三级标题 | `^###\s+(.+)` | `direct` |
| 章节范围 | 从标题行到下一个同级/上级标题前一行 | `indirect` |

### 5.5 Config/TCL/XDC 提取规则

| 目标 | 匹配模式 | strength |
|------|----------|----------|
| TCL 过程 | `^\s*proc\s+(\w+)` | `direct` |
| 约束命令 | `^\s*(set_property|create_clock|set_input_delay|set_output_delay)` | `indirect` |
| 变量赋值 | `^\s*set\s+(\w+)` | `indirect` |

### 5.6 Fallback 提取器

对 Phase 1 `file_classifier` 中已分类但无专用提取器的语言（如 `Unknown`、`Text` 等）：

| 策略 | 说明 |
|------|------|
| 整文件级 evidence | `line_range = {1, total_lines}`，`symbol = None`，`strength = indirect` |
| summary | 文件前 200 字符 + `"...(共 N 行)"` |

## 6. evidence_id 生成器

```rust
struct EvidenceIdGenerator {
    stage_id: String,
    counter: u32,
}

impl EvidenceIdGenerator {
    fn new(stage_id: &str) -> Self {
        Self {
            stage_id: stage_id.to_string(),
            counter: 0,
        }
    }

    /// 生成下一个唯一 evidence_id
    /// 格式: "EV-<stage_id>-<6位序号>"
    fn next_id(&mut self) -> String {
        self.counter += 1;
        format!("EV-{}-{:06}", self.stage_id, self.counter)
    }
}
```

**约束**：
- 单次 `collect_evidence` 调用内唯一（不需要跨会话持久化）
- counter 从 1 开始，每次 `next_id()` 递增
- 不做 counter 溢出处理（`u32` 最大值 4,294,967,295，远超实际需求）

## 7. excerpt 模块

```rust
/// 摘要提取与截断
struct ExcerptProcessor;

impl ExcerptProcessor {
    const MAX_SUMMARY_LEN: usize = 500;
    const TRUNCATE_KEEP_LEN: usize = 400;
    const TRUNCATE_SUFFIX: &'static str = "...(已截断，共 ";

    /// 从原始代码片段生成 summary
    /// raw: line_range 对应的源码文本
    /// total_lines: line_range 的总行数
    fn process(raw: &str, total_lines: usize) -> String {
        if raw.len() <= Self::MAX_SUMMARY_LEN {
            raw.to_string()
        } else {
            let truncated: String = raw.chars().take(Self::TRUNCATE_KEEP_LEN).collect();
            format!("{}{}{} 行)", truncated, Self::TRUNCATE_SUFFIX, total_lines)
        }
    }

    /// 整文件级摘要
    fn file_summary(content: &str, total_lines: usize) -> String {
        let first_200: String = content.chars().take(200).collect();
        if content.len() <= 200 {
            first_200
        } else {
            format!("{}...(共 {} 行)", first_200, total_lines)
        }
    }
}
```

## 8. index_builder 模块

```rust
struct IndexBuilder;

impl IndexBuilder {
    /// 从 evidence_items 构建三组索引
    fn build(items: &[EvidenceItem]) -> (IndexByPath, IndexByKind, IndexBySymbol) {
        let mut by_path: HashMap<String, Vec<String>> = HashMap::new();
        let mut by_kind: HashMap<String, Vec<String>> = HashMap::new();
        let mut by_symbol: HashMap<String, Vec<String>> = HashMap::new();

        for item in items {
            // index_by_path: 所有 item 必须出现
            by_path
                .entry(item.source_path.clone())
                .or_default()
                .push(item.evidence_id.clone());

            // index_by_kind: 所有 item 必须出现
            let kind_key = format!("{:?}", item.source_kind).to_snake_case();
            by_kind
                .entry(kind_key)
                .or_default()
                .push(item.evidence_id.clone());

            // index_by_symbol: 仅 symbol 非 None 的 item
            if let Some(ref symbol) = item.symbol {
                by_symbol
                    .entry(symbol.clone())
                    .or_default()
                    .push(item.evidence_id.clone());
            }
        }

        (by_path, by_kind, by_symbol)
    }
}
```

## 9. 错误处理设计

### 9.1 致命错误（CommandResult.success=false）

| 场景 | error_code | 触发条件 |
|------|-----------|----------|
| 路径不存在 | `path_not_found` | `std::fs::metadata` 失败 |
| 非目录 | `not_directory` | `std::fs::metadata` 显示非目录 |
| 权限拒绝 | `permission_denied` | symlink 根路径或不可读 |
| 阶段不存在 | `no_stage_found` | stage_id 不在 WorkspaceProfile 中 |
| 阶段为空 | `stage_empty` | stage 目录存在但无文件 |
| 整体收集失败 | `evidence_collection_failed` | 无法读取阶段目录（极端情况） |

### 9.2 非致命警告（CommandResult.warnings）

| 场景 | error_code | 触发条件 |
|------|-----------|----------|
| 文件不可读 | `file_unreadable` | 单个文件 `std::fs::read_to_string` 失败 |
| 文件过大 | `file_too_large` | 文件 > 5MB |
| 摘要截断 | `source_excerpt_truncated` | summary 超过 500 字符被截断 |
| 二进制跳过 | `binary_file_skipped` | 二进制文件 |
| 非 UTF-8 跳过 | `non_utf8_file_skipped` | 编码检测失败 |

### 9.3 Warning 传播规则

- 所有非致命问题进入 `EvidenceCollection.warnings[]`
- 不阻断收集流程：一个文件失败不影响其他文件
- 即使所有文件都跳过，仍返回 `success=true`（`evidence_items=[]`），warnings 包含跳过原因

## 10. 单元测试设计

### 10.1 测试模块分布

| 测试文件 | 覆盖模块 | 测试数量（预估） |
|----------|----------|-----------------|
| `evidence/models.rs` | 序列化/反序列化 | 3 |
| `evidence/id_generator.rs` | ID 生成唯一性、格式 | 4 |
| `evidence/excerpt.rs` | 截断逻辑、边界条件 | 5 |
| `evidence/index_builder.rs` | 索引正确性、空输入 | 4 |
| `evidence/extractors/python.rs` | Python 提取 | 5 |
| `evidence/extractors/verilog.rs` | Verilog 提取 | 4 |
| `evidence/extractors/systemverilog.rs` | SystemVerilog 提取 | 3 |
| `evidence/extractors/markdown.rs` | Markdown 提取 | 3 |
| `evidence/extractors/config.rs` | TCL/XDC 提取 | 3 |
| `evidence/collector.rs` | 集成收集流程 | 6 |
| `commands/collect_evidence.rs` | Command 层 | 5 |
| **合计** | | **~45** |

### 10.2 关键测试用例

#### id_generator

| 用例 | 输入 | 预期 |
|------|------|------|
| 首个 ID | stage_id="L0" | `"EV-L0-000001"` |
| 递增 | 连续调用 3 次 | `"EV-L0-000001"`, `"EV-L0-000002"`, `"EV-L0-000003"` |
| 不同 stage | stage_id="RTL" | `"EV-RTL-000001"` |
| 唯一性 | 生成 1000 个 ID | 全部不重复 |

#### excerpt

| 用例 | 输入 | 预期 |
|------|------|------|
| 短文本 | 100 字符 | 原样返回 |
| 恰好 500 字符 | 500 字符 | 原样返回 |
| 超过 500 字符 | 600 字符 | 前 400 + `"...(已截断，共 N 行)"` |
| 整文件摘要 | 1000 字符内容 | 前 200 + `"...(共 N 行)"` |
| 空内容 | 0 字符 | `""` |

#### Python 提取器

| 用例 | 输入 | 预期 |
|------|------|------|
| 简单函数 | `def foo():\n    pass` | 1 item: symbol="foo", range={1,2}, strength=direct |
| 多函数 | 3 个 `def` | 3 items，各自 range 不重叠 |
| 类定义 | `class Bar:` | 1 item: symbol="Bar", strength=direct |
| 嵌套函数 | 函数内嵌套 def | 外层 range 包含内层 |
| 空文件 | `""` | 0 items |
| 注释 | `# comment` | 0 items |

#### Verilog 提取器

| 用例 | 输入 | 预期 |
|------|------|------|
| 单 module | `module top(...);\n...\nendmodule` | 1 item: symbol="top", range 覆盖完整 module |
| 多 module | 2 个 module | 2 items |
| 无 module | 只有 assign 语句 | 0 items（assign 为 indirect，暂不在 Phase 2 提取为独立 item） |
| 空 module | `module empty();\nendmodule` | 1 item: range={1,2} |

#### collector 集成

| 用例 | 输入 | 预期 |
|------|------|------|
| 正常收集 | 含 3 个 Python 文件的阶段 | EvidenceCollection.evidence_items.len() > 0，索引完整 |
| 空阶段 | 0 文件 | CommandResult.success=false, error=stage_empty |
| 混合语言 | Python + Verilog + Markdown | 各提取器分别处理，索引正确 |
| 大文件跳过 | 1 个 >5MB 文件 | warning file_too_large，skipped=1，evidence_items 不含该文件 |
| 非 UTF-8 | 1 个二进制文件 | warning non_utf8_file_skipped，该文件无 evidence item |
| 阶段不存在 | 不存在的 stage_id | CommandResult.success=false, error=no_stage_found |

## 11. 与 Phase 1 的复用关系

| Phase 1 模块 | Phase 2 复用方式 |
|-------------|----------------|
| `workspace/safety_guard.rs` | 直接调用路径校验函数 |
| `workspace/stage_detector.rs` | 通过 `select_stage` 逻辑间接复用 |
| `workspace/file_classifier.rs` | 复用 `Language`、`SourceKind` 枚举和分类结果 |
| `models/workspace.rs` | 复用 `StageContext`、`StageFile` 数据结构 |
| `commands/select_stage.rs` | collector 内部调用 select_stage 获取 StageContext |

## 12. 性能考虑

| 考虑点 | 设计 |
|--------|------|
| 文件读取 | 每个文件只读一次（`read_to_string`），不重复读取 |
| 内存 | `EvidenceCollection` 保留在内存中，不写磁盘 |
| 大文件 | >5MB 跳过，避免内存压力 |
| 提取速度 | 纯正则/行级匹配，O(n) 线性扫描，无 AST 开销 |
| 并发 | Phase 2 不做并发文件读取（单阶段文件数量有限，串行足够） |

## 13. 不做的事情

- **不做 AST 复杂解析**：只做正则/行级关键字匹配
- **不做跨文件符号解析**：每个文件独立提取
- **不做类型推断**：不推断信号类型、函数返回值
- **不做代码语义理解**：不理解代码逻辑，只提取结构事实
- **不做大模型调用**：Phase 2 不调用任何 LLM API
- **不做并发优化**：串行处理足够
- **不做持久化**：产物仅在内存中

## 14. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-11 | 初始创建 | Claude |
