# Phase 3 理解生成验证设计

---
status: active
updated: 2026-06-12
---

> 本文档定义 Phase 3（单阶段结构化理解产物）的验证策略，覆盖数据模型、context builder、schema validator、evidence_id 检查、claim 约束、unknown/gap 处理、mock provider、前端渲染、安全回归和手工验收。
>
> **本文档已审核收口，作为 Phase 3 编码依据。**

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
| `understanding/context_builder.rs` | ContextBuilder 输出正确性 + schema 覆盖 | 8 |
| `understanding/schema_validator.rs` | Schema 验证 + evidence_id check + confidence/claim_id/description | 28 |
| `understanding/generator.rs` | Generator pipeline + MockProvider + ManualProvider + degraded + validation | 7 |
| `commands/generate_understanding.rs` | Tauri command E2E（含 readonly、multi-stage） | 8 |
| **合计** | | **~55** |

### 2.2 前端验证

| 验证方式 | 覆盖内容 | 结果 |
|----------|----------|------|
| `npm run build` | TypeScript 编译 + Vite 构建 | ✅ Batch C 通过 |
| 代码路径检查 | WorkspacePage 状态机 + StageDetail 集成 + UnderstandingPanel | ✅ Batch C |
| 手工桌面验收 | 完整用户流程 | 见 §10 |

> 前端组件单元测试为可选项，Phase 3 优先保证后端测试。Batch C 以构建 + 代码路径验证为主。

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
| 五种值 | confirmed / supported / inferred / unknown / conflicting | 序列化为 snake_case 字符串 |
| 反序列化 | "confirmed" / "supported" / "inferred" / "unknown" / "conflicting" | 正确解析 |
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
| unknown 有伪造 ID | unknown 引用不存在的 ID | UnknownEvidenceId error（统一使用 UnknownEvidenceId 覆盖所有位置） |

### 5.3 业务规则检查

| 用例 | 输入 | 预期 |
|------|------|------|
| claim 数量 > 0 | 有 claims | is_valid = true |
| 0 claims 但有 unknowns | claims 为空 | is_valid = true（允许纯 unknown 结果） |
| unknown 过多 | unknown > claims | warning: TooManyUnknowns |
| gap 过多 | gap > 10 | warning: TooManyGaps |

## 6. Generator Pipeline 测试（Mock Provider）

### 6.1 Generator pipeline 测试

| 用例 ID | 输入 | 预期 |
|---------|------|------|
| gen_01 | MockProvider + 含 evidence_items 的 EvidenceCollection | 成功返回 ImplementationUnderstanding，claims 非空 |
| gen_02 | MockProvider 输出 | 所有 evidence_refs 中的 evidence_id 均在 known_evidence_ids 中（无伪造 ID） |
| gen_03 | ManualProvider | 返回 degraded 结果，is_degraded=true，provider="manual"，claims 为空 |
| gen_04 | MockProvider + 空 EvidenceCollection | 成功返回，无 panic |
| gen_05 | ManualProvider + 空 EvidenceCollection | 返回 degraded 结果，无 panic |
| gen_06 | BadProvider（返回无效 JSON） | GeneratorError::ValidationFailed |
| gen_07 | FakeIdProvider（返回含伪造 evidence_id 的 JSON） | GeneratorError::ValidationFailed |

### 6.2 degraded mode 语义

| 字段 | 值 | 说明 |
|------|-----|------|
| version | "3.0.0" | 固定版本 |
| claims | [] | 无语义推断 |
| summary | "语义生成 Provider 未配置，无法生成阶段理解" | 明确说明原因 |
| unknowns | [{reason: "语义生成 Provider 未配置"}] | 标注无法推断的原因 |
| gaps | [{reason: "当前为 degraded mode"}] | 标注缺失原因 |
| is_degraded | true | degraded 标志 |
| provider | "manual" | 标识 provider 类型 |
| evidence_refs | 仅引用 known_evidence_ids | 不伪造任何 ID |

## 7. Tauri Command 测试

| 用例 ID | 输入 | 预期 |
|---------|------|------|
| und_01 | Python 项目有效阶段 | success=true，claims 非空，evidence_refs 有效 |
| und_02 | Verilog 项目有效阶段 | success=true，module/signal claims 正确 |
| und_03 | 空 stage（无源文件） | success=true，MockProvider 返回有效理解 |
| und_04 | root_path 不存在 | success=false，PathNotFound |
| und_05 | stage_id 不存在 | success=false，StageNotFound |
| und_06 | readonly 验证 | 目标项目目录无变化 |
| und_07 | stage_id="" | success=false |
| und_08 | 多阶段 pipeline（先 select → collect → generate） | success=true，全流程串联 |

## 8. 前端渲染验证

| 用例 | 输入 | 预期 | 状态 |
|------|------|------|------|
| 正常渲染 | 含 5 claims + 2 modules + 1 unknown | 各区域正确展示 | Batch C 实现 |
| confidence 标签颜色 | confirmed / supported / inferred / unknown / conflicting | 对应颜色正确 | Batch C 实现 |
| evidence_id 可见 | 所有 evidence_refs | evidence_id 蓝色 chip 可见 | Batch C 实现 |
| degraded 提示 | is_degraded=true | 显示"降级生成 · Provider 未配置" | Batch C 实现 |
| 空理解 | 0 claims | 显示轻量空状态 | Batch C 实现 |
| 生成失败 | error 状态 | 显示错误面板（error_code/message/recoverable） | Batch C 实现 |
| 生成中 | understandingLoading=true | 按钮 disabled + "生成中..." | Batch C 实现 |
| 阶段切换清空 | 切换到其他 stage | 旧 understanding 清空 | 状态机自动清理 |

> evidence 回链交互（点击 evidence_id → 高亮 EvidencePanel）为 Phase 3 后续优化或 Phase 4 实现。

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
| 2026-06-12 | Phase 3 Batch C：前端验证更新（构建 + 代码路径检查 + UnderstandingPanel 渲染矩阵 8 项）；移除可选的组件单元测试预估 | Claude |
| 2026-06-12 | Phase 3 Batch B：更新 generator 测试矩阵（7 用例）、command 测试矩阵（8 用例）、degraded mode 语义表、测试合计 49→55 | Claude |
| 2026-06-12 | 审核收口：删除 UnknownWithFakeEvidence（统一用 UnknownEvidenceId）；测试数量更新（schema_validator 8→28，context_builder 5→8，合计 26→49）；新增 version/claim_id/description 非空 + claim_id 格式验证用例 | Claude |
| 2026-06-12 | 收口修复：ClaimConfidence 测试矩阵补齐 supported（4→5 种值）；前端渲染测试同步；status draft → active | Claude |
| 2026-06-12 | 初始创建（draft） | Claude |
