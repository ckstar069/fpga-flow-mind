# Phase 5 证据回链与 Grounded Q&A 验证设计

---
status: active
updated: 2026-06-14
---

> 本文档定义 Phase 5 的验证策略、测试矩阵、安全回归清单和桌面验收步骤。实施前必须与 `phase-5-trace-and-qa-requirements.md` 和 `phase-5-trace-and-qa-design.md` 对齐。
>
> 本文档已收口（status=active），是 Phase 5 编码依据。

## 1. 验证目标

Phase 5 编码完成后，以下维度应通过验证：

| 维度 | 验证内容 |
|------|----------|
| TraceResolver | 各类选择目标正确解析为 `TraceRefResolved[]` |
| SourceExcerptResolver | 安全读取目标项目文件并返回 `SourceExcerpt` |
| Grounded Q&A | MockProvider 生成带 citations 的回答，Validator 正确拦截非法输出 |
| 前端交互 | 节点/边选中、TracePanel、SourceExcerptPanel、EvidencePanel 高亮、GroundedQAPanel |
| 安全回归 | 目标项目只读、拒绝越界路径、无真实 LLM 默认调用、无审计用语 |
| 桌面验收 | 完整用户流程通过 |

## 2. 测试模块分布

### 2.1 Rust 后端测试

| 测试位置 | 覆盖模块 | 预估数量 |
|----------|----------|----------|
| `trace/models.rs` | SelectedTraceTarget / TraceRefResolved serde | 4 |
| `trace/resolver.rs` | TraceResolver | 10 |
| `trace/source_resolver.rs` | SourceExcerptResolver | 12 |
| `trace/qa/mock_provider.rs` | MockProvider | 5 |
| `trace/qa/validator.rs` | GroundedQaValidator | 8 |
| `commands/resolve_trace_target.rs` | command 层 | 4 |
| `commands/get_source_excerpt.rs` | command 层 | 6 |
| `commands/ask_grounded_question.rs` | command 层 | 4 |
| **合计** | | **~53** |

### 2.2 前端验证

| 验证方式 | 覆盖内容 |
|----------|----------|
| `npm run build` | TypeScript 编译 + Vite 构建 |
| 代码路径检查 | selection state、TracePanel、SourceExcerptPanel、GroundedQAPanel |
| 桌面验收 | 完整用户流程 |

## 3. 后端测试矩阵

### 3.1 TraceResolver

| 用例 | 输入 | 预期 |
|------|------|------|
| view_node resolves | `ViewNode` 含 2 条 trace_refs | 返回 2 条 `TraceRefResolved`，`resolution = Resolved` |
| view_edge resolves | `ViewEdge` 含 1 条 trace_ref | 返回 1 条 `TraceRefResolved` |
| claim resolves evidence_refs | `Claim` 含 3 条 evidence_refs | 返回 3 条 evidence snapshot |
| evidence resolves directly | `Evidence` evidence_id | 返回 1 条 `EvidenceOnly` resolved trace |
| missing evidence_id | trace_ref 引用不存在的 evidence_id | `resolution = MissingEvidence` |
| missing claim_id | trace_ref 引用不存在的 claim_id | `resolution = MissingClaim` |
| claim with evidence_gap | `has_evidence_gap = true` | `resolution = ClaimOnly`，显示 gap 说明 |
| empty trace_refs | node.trace_refs = [] | 返回空列表，UI 显示"无证据追溯" |
| view_type not found | target 指向不存在的 view_type | command 返回 `trace_target_not_found` |

### 3.2 SourceExcerptResolver

| 用例 | 输入 | 预期 |
|------|------|------|
| valid evidence_id resolves | evidence_id 对应的 source_path + line_range | `SourceExcerpt.lines` 非空，行号正确 |
| direct source_location | source_path + line_range | 同上 |
| source_path outside root | source_path = `/etc/passwd` | 返回 `source_path_not_allowed` |
| symlink | source_path 为 symlink | 返回 `source_path_not_allowed` |
| root_path symlink | `root_path` 本身为 symlink | 拒绝并返回 `source_path_not_allowed` |
| parent dir symlink | `source_path` 的某一级父目录为 symlink | 返回 `source_path_not_allowed` |
| string-prefix trick | `root_path = /tmp/root`, `source_path = /tmp/root2/evil.v` | 返回 `source_path_not_allowed` |
| canonical outside root | `source_path` 经 canonicalize 后跳出 `root_path`（如通过 `..` 拼接） | 返回 `source_path_not_allowed` |
| binary file | source_path 指向二进制 | 返回 `source_file_unreadable` |
| non-UTF8 file | source_path 指向非 UTF-8 | 返回 `source_file_unreadable` |
| too-large file | 文件 > 5MB | 返回 `source_file_unreadable` |
| line_range out of bounds | end > 文件总行数 | 返回 `line_range_invalid` |
| truncation | line_range 跨度 > 100 行 | `is_truncated = true`，只返回前 100 行 |

### 3.3 GroundedQaProvider / Validator

| 用例 | 输入 | 预期 |
|------|------|------|
| answer cites evidence | question 匹配关键词 | `GroundedAnswer.citations` 非空，claim 非 unknown |
| answer returns unknown | question 无法匹配 | `confidence = unknown`，`citations = []`，`reason` 非空，`warnings` 非空 |
| unknown answer without citation passes | 构造 `confidence = unknown` 且 `citations = []`，但 `reason` 和 `warnings` 非空 | validator 通过 |
| validator rejects no citation for non-unknown claim | 构造 supported/confirmed/inferred/conflicting claim 但无 citation | `qa_validation_failed` |
| unknown answer with fake citation fails | 构造 `confidence = unknown` 但伪造 citation | `qa_validation_failed` |
| MockProvider unanswerable returns unknown without fabricated citation | 提问无法匹配任何关键词 | 返回 `confidence = unknown`，`citations = []`，不伪造 citation |
| validator rejects audit words | answer 含"PASS" | `qa_validation_failed` |
| validator rejects fake evidence_id | citation 引用不存在 evidence | `qa_validation_failed` |
| degraded flag | MockProvider 返回 | `is_degraded = true` |

## 4. 前端验证矩阵

| 场景 | 预期 |
|------|------|
| 点击节点 | 节点进入 selected 态，TracePanel 展示 |
| 点击边 | 边进入 selected 态，TracePanel 展示 |
| 切换 tab | 选中态保留（若存在）或清空 |
| TracePanel 展开/折叠 | 可折叠，折叠后显示提示 |
| 查看源码片段 | SourceExcerptPanel 展示源码行 |
| 定位 evidence | EvidencePanel 高亮对应 item |
| 提交问题 | GroundedQAPanel 显示回答 + citations |
| unknown 回答 | 回答显示"证据不足" |
| citation 点击 | 打开对应 SourceExcerptPanel |
| 切换阶段 | 所有 trace/qa 状态清空 |

## 5. 安全回归

```bash
# 禁止写入/执行 API 检查
rg "std::fs::write|std::fs::create_dir|std::fs::remove_file|std::fs::rename|std::fs::copy|std::process::Command|Command::new" src-tauri/src/trace/

# 越界检查：无 Vivado/synthesis/implementation/bitstream
rg "Vivado|synthesis|implementation|bitstream" src-tauri/src/trace/ src/ features/

# 真实 LLM API 检查
rg "openai|anthropic|api_key" src-tauri/src/trace/ src/

# 审计用语检查
rg "PASS|HOLD|正确|错误|审计" src/ src-tauri/src/trace/
```

预期：无匹配（审计用语仅在"禁止"语境的文档中允许出现）。

## 6. 桌面验收

### 6.1 样例项目结构建议

复用 Phase 4 验收项目 `/tmp/fpga-flow-mind-phase4-acceptance-20260612-194151`，或新建 `/tmp/fpga-flow-mind-phase5-acceptance-YYYYMMDD-HHMMSS`，结构同 Phase 4（含 L0/L1/rtl_final/docs 等）。

### 6.2 验收步骤

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 打开项目，选择 L0，收集证据，生成理解，生成视图 | 与 Phase 4 一致 |
| 2 | 点击结构图中某个 module 节点 | 节点高亮，TracePanel 展示 trace_refs |
| 3 | 在 TracePanel 中点击"查看源码片段" | SourceExcerptPanel 展示对应源码行 |
| 4 | 在 TracePanel 中点击"定位 evidence" | EvidencePanel 高亮对应 evidence item |
| 5 | 点击某条 claim 的 evidence chip | TracePanel 切换为 claim 视角 |
| 6 | 在 GroundedQAPanel 输入"这个模块的输入位宽是多少" | 返回带 citation 的回答 |
| 7 | 点击回答中的 citation | SourceExcerptPanel 展示引用源码 |
| 8 | 输入无法回答的问题（如"项目的商业目标是什么"） | 返回 `confidence = unknown` |
| 9 | 切换阶段 | trace/qa 状态清空 |
| 10 | 验证目标项目只读 | checksum 前后一致 |

## 7. Phase 5 完成标准

- P5-T01~P5-T11 全部完成。
- Rust 测试新增 ~53 个且全部通过，总测试数 ≥ 313。
- `npm run build` 通过。
- `cargo check` 通过。
- 桌面验收 10/10 通过。
- checksum 只读验证通过。
- 无真实 LLM 默认调用。
- 无 PASS/HOLD 审计用语出现在用户可见输出。
- completion review 完成并标记 active。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-13 | 初始创建（draft）：定义 Phase 5 测试矩阵、安全回归、桌面验收、完成标准 | Claude |
