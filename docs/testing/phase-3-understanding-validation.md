# Phase 3 理解生成验证设计

---
status: draft
updated: 2026-06-12
---

> 本文档定义 Phase 3（单阶段结构化理解产物）的验证策略，覆盖数据模型、context builder、schema validator、evidence_id 检查、claim 约束、unknown/gap 处理、mock provider、前端渲染、安全回归和手工验收。
>
> **Phase 3 不编码**。本文档是 draft，编码前需审核收口。

## 1. 验证目标

Phase 3 编码完成后，以下维度应通过验证：

| 维度 | 验证内容 |
|------|----------|
| 数据模型 | Rust/TypeScript serde 正确性 |
| Context builder | EvidenceCollection → LLM 输入转换正确 |
| Schema validator | 输出 JSON 验证 + evidence_id existence check |
| Claim 约束 | 每 claim 有 evidence_refs 或 evidence_gap |
| Unknown/gap | 无伪造 evidence_id，正确标记 |
| Mock provider | 完整 pipeline 可运行 |
| 前端渲染 | UnderstandingPanel 正确展示 |
| 安全回归 | 目标项目只读、无写入/执行 API |

## 2. 测试模块分布

### 2.1 Rust 后端测试

| 测试位置 | 覆盖模块 | 预估数量 |
|----------|----------|----------|
| `understanding/models.rs` | 数据结构 serde（序列化/反序列化） | 4 |
| `understanding/context_builder.rs` | ContextBuilder 输出正确性 | 5 |
| `understanding/schema_validator.rs` | Schema 验证 + evidence_id check | 8 |
| `understanding/generator.rs` | Generator pipeline（mock provider） | 4 |
| `commands/generate_understanding.rs` | Tauri command 层 | 5 |
| **合计** | | **~26** |

### 2.2 前端测试

| 测试位置 | 覆盖内容 | 预估数量 |
|----------|----------|----------|
| `src/features/workspace/components/UnderstandingPanel.test.tsx` | 组件渲染、状态展示 | 5 |
| **合计** | | **~5** |

> 前端组件测试为可选项，Phase 3 优先保证后端测试。

### 2.3 手工验收

| 验收项 | 说明 |
|--------|------|
| Tauri 桌面端 | 生成理解 → 面板展示 → evidence 回链 → unknown/gap 可见 |
| 安全回归 | 目标项目无变化 |

## 3. 数据模型测试

### 3.1 ImplementationUnderstanding serde

| 用例 | 输入 | 预期 |
|------|------|------|
| 完整序列化 | 含全部字段的 ImplementationUnderstanding | JSON 输出字段完整 |
| 完整反序列化 | 合法 JSON 字符串 | 正确解析为 ImplementationUnderstanding |
| Round-trip | 序列化 → 反序列化 | 与原始对象相等 |
| 最小有效对象 | 仅必填字段 | 成功解析 |

### 3.2 ClaimConfidence 枚举

| 用例 | 输入 | 预期 |
|------|------|------|
| 四种值 | confirmed / inferred / unknown / conflicting | 序列化为 snake_case 字符串 |
| 反序列化 | "confirmed" / "inferred" / "unknown" / "conflicting" | 正确解析 |
| 非法值 | "definitely" | 反序列化失败 |

### 3.3 ClaimCategory 枚举

| 用例 | 输入 | 预期 |
|------|------|------|
| 八种值 | module_structure / signal_definition / ... | 序列化为 snake_case |
| 非法值 | "unknown_category" | 反序列化失败 |

### 3.4 EvidenceRef

| 用例 | 输入 | 预期 |
|------|------|------|
| 有 relevance | { evidence_id: "EV-L0-000001", relevance: "定义了模块" } | 成功 |
| 无 relevance | { evidence_id: "EV-L0-000001" } | relevance = None |

## 4. ContextBuilder 测试

### 4.1 输出正确性

| 用例 | 输入 | 预期 |
|------|------|------|
| 正常 EvidenceCollection | 含 5 个 evidence_items | prompt 包含所有 5 个 item 摘要 |
| 空 EvidenceCollection | evidence_items = [] | prompt 表示无证据 |
| known_evidence_ids 一致 | 5 个 items | known_evidence_ids 包含全部 5 个 evidence_id |
| output_schema 合法 | 任意 | schema 是合法 JSON 且覆盖 ImplementationUnderstanding 结构 |

### 4.2 prompt 内容检查

| 用例 | 检查点 |
|------|--------|
| system prompt 包含约束 | prompt 中包含 "evidence_id"、"confidence"、"unknown" |
| user prompt 包含 evidence | prompt 中包含 evidence_items 的 symbol / summary |
| 不包含 source_path | prompt 中不直接包含文件路径（通过 evidence_id 引用） |

## 5. SchemaValidator 测试

### 5.1 JSON Schema 验证

| 用例 | 输入 | 预期 |
|------|------|------|
| 合法完整输出 | 符合 schema 的 JSON | is_valid = true |
| 缺少必填字段 | 缺少 summary | is_valid = false，SchemaViolation |
| 字段类型错误 | claims 为字符串而非数组 | is_valid = false |
| 枚举值非法 | confidence = "definitely" | is_valid = false |

### 5.2 evidence_id existence check

| 用例 | 输入 | 预期 |
|------|------|------|
| 全部 ID 存在 | 3 个 evidence_refs，均在 known_ids 中 | is_valid = true |
| 一个 ID 不存在 | 1 个 evidence_id 不在 known_ids 中 | UnknownEvidenceId error |
| 空 evidence_refs | claim 无 evidence_refs 且 has_evidence_gap=false | ClaimWithoutEvidence error |
| unknown 无伪造 ID | unknown 的 related_evidence_refs 全在 known_ids 中 | is_valid = true |
| unknown 有伪造 ID | unknown 引用不存在的 ID | UnknownWithFakeEvidence error |

### 5.3 业务规则检查

| 用例 | 输入 | 预期 |
|------|------|------|
| claim 数量 > 0 | 有 claims | is_valid = true |
| 0 claims 但有 unknowns | claims 为空 | is_valid = true（允许纯 unknown 结果） |
| unknown 过多 | unknown > claims | warning: TooManyUnknowns |
| gap 过多 | gap > 10 | warning: TooManyGaps |

## 6. Generator Pipeline 测试（Mock Provider）

### 6.1 完整 pipeline

| 用例 | 输入 | 预期 |
|------|------|------|
| Mock 正常返回 | 预设合法 ImplementationUnderstanding JSON | 成功返回 |
| Mock 返回非法 JSON | 预设缺少字段的 JSON | GeneratorError::ValidationFailed |
| Mock 返回含假 ID | 预设引用不存在 evidence_id | GeneratorError::ValidationFailed |
| Mock provider 错误 | ProviderError::NotConfigured | GeneratorError::ProviderError |

### 6.2 degraded mode

| 用例 | 输入 | 预期 |
|------|------|------|
| 无 LLM provider | ManualProvider | 返回降级结果，is_degraded = true |

## 7. Tauri Command 测试

| 用例 | 输入 | 预期 |
|------|------|------|
| 正常生成 | 有效 EvidenceCollection | success=true，ImplementationUnderstanding 结构完整 |
| 空 evidence | evidence_items=[] | success=true，返回仅含 stats 的空理解 |
| 阶段不存在 | 无效 stage_id | success=false |
| 无效路径 | root_path 不存在 | success=false，PathNotFound |
| 空 stage_id | stage_id="" | success=false |
| 生成超时 | mock provider 模拟超时 | success=false，timeout error |

## 8. 前端渲染测试

| 用例 | 输入 | 预期 |
|------|------|------|
| 正常渲染 | 含 5 claims + 2 modules + 1 unknown | 各区域正确展示 |
| confidence 标签颜色 | confirmed / inferred / unknown / conflicting | 对应颜色正确 |
| evidence 回链 | 点击 evidence_id chip | 触发高亮事件 |
| unknown-heavy 警告 | unknown_count > claim_count | 显示警告 |
| 空理解 | 0 claims | 显示空状态 |
| 生成失败 | error 状态 | 显示错误面板 |

## 9. 安全回归测试

### 9.1 禁止 API 检查

```bash
rg "std::fs::write|std::fs::create_dir|std::fs::remove_file|std::fs::rename|std::fs::copy|std::process::Command|Command::new" src-tauri/src/understanding/
```

预期：无匹配。

### 9.2 目标项目不变

生成理解前后目标项目目录 checksum 一致。

### 9.3 越界检查

```bash
rg "GraphView|Dataflow|Q&A|QA|LLM" src src-tauri/src/understanding/
```

预期：无匹配。

## 10. 手工验收

### 10.1 验收步骤

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 打开 Tauri 桌面应用 | 正常启动 |
| 2 | 打开项目，选择阶段，收集证据 | 证据面板展示正常 |
| 3 | 点击"生成理解"按钮 | 按钮变为"生成理解中..." |
| 4 | 等待生成完成 | 面板展示 UnderstandingPanel |
| 5 | 查看阶段摘要 | 显示中文摘要文本 |
| 6 | 查看 claim 列表 | 每个 claim 有 category + confidence + evidence_refs |
| 7 | 点击 evidence_id chip | EvidencePanel 高亮对应 evidence item |
| 8 | 查看 unknown 区域 | 显示"无法推断的信息" |
| 9 | 查看 evidence gap 区域 | 显示"证据缺失" |
| 10 | 检查目标项目目录 | 无新增/修改/删除文件 |

### 10.2 不允许出现的 UI 行为

- ❌ 显示"正确/错误"判断
- ❌ 显示 PASS/HOLD 审计结论
- ❌ 隐藏或淡化 unknown / evidence gap
- ❌ 显示原始 JSON
- ❌ 显示图视图 / Q&A 面板

## 11. 验收标准总结

| # | 标准 | 验证方式 |
|---|------|----------|
| 1 | Rust 全量测试通过 | `cargo test` |
| 2 | 前端构建通过 | `npm run build` |
| 3 | 所有 evidence_refs 中的 evidence_id 可查到 | 单元测试 + schema validator |
| 4 | 无伪造 evidence_id | 单元测试 |
| 5 | claim 无 refs 时有 evidence_gap 标注 | 单元测试 |
| 6 | confidence 使用正确枚举值 | serde 测试 + 前端渲染测试 |
| 7 | 目标项目只读 | rg 检查 + checksum 比对 |
| 8 | 无越界功能（图/Q&A/持久化） | rg 检查 |
| 9 | Tauri 桌面端手工验收通过 | 10 步验收 |
| 10 | Phase 1/Phase 2 功能无回归 | 全量测试 + UI 验证 |

## 12. 文档变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-12 | 初始创建（draft） | Claude |
