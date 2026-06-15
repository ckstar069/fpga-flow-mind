# MVP Release Notes

---
status: active
updated: 2026-06-15
---

> 本文记录 `fpga-flow-mind` 第一个可用闭环（MVP / Phase 0–6 completion）的发布信息。对应 Git tag `v0.1.0-mvp`。

## 1. Release 概要

| 项 | 值 |
|----|----|
| Release 名称 | MVP / Phase 0–6 completion |
| Release 日期 | 2026-06-15 |
| 对应提交 | `67e0f93`（main） |
| Git tag | `v0.1.0-mvp` |
| 结论 | **允许 Phase 6 / MVP completion** |

## 2. 已完成能力

| 能力 | 说明 |
|------|------|
| Workspace 扫描与阶段识别 | 打开本地 FPGA 项目，识别 L0/L1/L2/RTL 等阶段目录结构与文件类型 |
| Evidence 收集与索引 | 按阶段抽取 Python/Verilog 等源码证据，建立 path/kind/symbol 索引与强度标注 |
| ImplementationUnderstanding 生成 | 生成结构化阶段理解产物（模块/信号/接口/处理步骤/未知项/证据缺口） |
| 三类视图 | 结构图 / 数据流图 / 时序流水图，节点与边带 trace_refs |
| Trace 回链与源码片段 | 点击视图节点/边追溯到 claim/evidence，展示源码片段，不打开外部编辑器 |
| Grounded Q&A | 基于当前阶段 evidence/understanding 回答问题，带 citation；证据不足返回 unknown 且不伪造 citation（当前为 MockProvider） |
| Session 持久化 | session 保存 / 加载 / 最近项目列表 / 删除 / 2s 轻量自动保存 |
| 只读保护 | 目标项目 checksum 前后一致，持久化只写 app-owned storage |

## 3. 验证结果

| 验证项 | 命令 | 结果 |
|--------|------|------|
| 前端构建 | `npm run build` | ✅ 通过 |
| Rust 测试 | `cd src-tauri && cargo test --lib` | ✅ 411 passed |
| Rust 类型检查 | `cd src-tauri && cargo check` | ✅ 通过，0 warnings |
| Phase 6 真实桌面验收 | 15 步清单 | ✅ 15/15 |
| 目标项目只读 | checksum 前后对比 | ✅ 一致 |

完整验收记录见 [`phase-6-completion-review.md`](phase-6-completion-review.md)。

## 4. 安全边界

- 目标项目只读：不修改目标 FPGA 项目源码，checksum 前后一致。
- 不运行 Vivado / synthesis / implementation / bitstream。
- 不调用真实 LLM API（当前为 MockProvider）。
- 持久化只写 app-owned storage（app_data_dir），拒绝 path traversal / symlink / root mismatch。
- 不保存完整源码副本，不保存敏感环境变量。
- 不输出 PASS/HOLD/正确/错误等审计结论。

## 5. 已知限制

1. **Grounded Q&A 当前为 MockProvider**：基于关键词匹配生成回答；真实 LLM Provider 需在后续阶段单独设计、显式配置、经过 GroundedQaValidator，并独立验收。
2. **自动保存为 2s debounce**：无复杂队列 / 后台 watcher / 冲突合并。
3. **active_view_type 未持久化**：当前 UI 无集中 active view type 状态。
4. **QA history 已持久化但 UI 未展示完整历史列表**：当前阶段最新回答会显示，完整历史仅持久化。
5. **source_changed 为手动触发**：通过修改源文件后重新加载验证，未引入文件系统 watcher。

## 6. 本地运行

```bash
npm install
npm run tauri dev
```

构建与测试：

```bash
npm run build
cd src-tauri && cargo test --lib
cd src-tauri && cargo check
```

## 7. 下一步

- Phase 7 及后续阶段需在新的需求/设计/计划文档 active 后方可启动。
- 真实 LLM Provider 接入、自动保存策略增强、QA history 完整展示等为候选后续工作，不属于本次 MVP release。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-15 | 初始 MVP release notes：记录已完成能力、验证结果、安全边界、已知限制，对应 tag `v0.1.0-mvp`。 | Claude |
