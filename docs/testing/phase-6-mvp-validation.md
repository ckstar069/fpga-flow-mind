# Phase 6 MVP 验证设计

---
status: draft
updated: 2026-06-14
---

> 本文档定义 Phase 6（持久化、回放与 MVP 总体验收）的验证策略、测试矩阵、安全回归清单和桌面验收步骤。
>
> 本文档为 draft，仅供评审与讨论，不得作为 Phase 6 编码唯一依据。本轮修复后仍需审核并转为 active，方可进入 Phase 6 编码。

## 1. 验证目标

Phase 6 编码完成后，以下维度应通过验证：

| 维度 | 验证内容 |
|------|----------|
| 持久化保存 | save_session 能原子写入 manifest 与各 artifact |
| 加载回放 | load_session 能恢复 WorkspacePage 状态 |
| 最近项目列表 | list_sessions 能正确列出、排序、过滤损坏 session |
| 删除 | delete_session 只删除 app-owned storage，不影响目标项目 |
| 变更检测 | fingerprint mismatch / 路径不存在 / symlink 均被正确识别 |
| 版本兼容 | 同版本加载成功，不兼容版本明确拒绝 |
| 前端交互 | 最近项目列表、保存状态、加载失败提示 |
| MVP 总体验收 | 完整端到端流程通过 |
| 安全回归 | 目标项目只读、拒绝越界路径、无真实 LLM 调用、无审计用语 |

## 2. 测试模块分布

### 2.1 Rust 后端测试

| 测试位置 | 覆盖模块 | 预估数量 |
|----------|----------|----------|
| `persistence/models.rs` | StorageVersion / SessionManifest / ArtifactIndex serde | 4 |
| `persistence/version_service.rs` | 版本兼容/不兼容判断 | 6 |
| `persistence/fingerprint_service.rs` | fingerprint 计算、变更检测 | 8 |
| `persistence/manifest_repository.rs` | manifest 读写、损坏处理 | 6 |
| `persistence/artifact_repository.rs` | artifact 原子写入、路径安全 | 8 |
| `persistence/session_store.rs` | save/load/list/delete 集成 | 10 |
| `commands/save_session.rs` | command 层 | 4 |
| `commands/load_session.rs` | command 层 | 6 |
| `commands/list_sessions.rs` | command 层 | 4 |
| `commands/delete_session.rs` | command 层 | 4 |
| **合计** | | **~60** |

### 2.2 前端验证

| 验证方式 | 覆盖内容 |
|----------|----------|
| `npm run build` | TypeScript 编译 + Vite 构建 |
| 代码路径检查 | 最近项目列表、保存状态、加载入口、错误提示 |
| 桌面验收 | 完整 MVP 流程 |

## 3. 后端测试矩阵

### 3.1 StorageVersion

| 用例 | 输入 | 预期 |
|------|------|------|
| same major minor ok | current vs current | 兼容 |
| older minor ok | saved minor < current minor | 兼容 |
| newer minor rejected | saved minor > current minor | 不兼容 |
| different major rejected | saved major != current major | 不兼容 |
| serde roundtrip | StorageVersion | JSON 反序列化一致 |

### 3.2 FingerprintService

| 用例 | 输入 | 预期 |
|------|------|------|
| stable fingerprint | 同一目录两次计算 | 结果相同 |
| changed file detected | 修改一个源码文件后 | fingerprint 变化 |
| symlink rejected | root_path 为 symlink | error |
| nonexistent path | root_path 不存在 | error |
| excludes binary | 包含二进制文件 | 不参与 fingerprint |
| excludes large file | 包含超大文件 | 不参与 fingerprint |

### 3.3 ArtifactRepository

| 用例 | 输入 | 预期 |
|------|------|------|
| atomic write | artifact | 临时文件 + rename |
| read back | artifact path | 反序列化一致 |
| path traversal rejected | `../../evil.json` | error |
| symlink rejected | artifact 文件为 symlink | error |
| parent symlink rejected | 父目录为 symlink | error |

### 3.4 SessionStore 集成

| 用例 | 输入 | 预期 |
|------|------|------|
| save then load | SessionState | 状态一致 |
| list sessions | 保存 3 个 session | 列表按 updated_at 倒序 |
| delete session | session_id | 目录删除，目标项目不变 |
| load nonexistent | 不存在的 session_id | error |
| load corrupted manifest | 损坏的 manifest.json | error |
| load changed source | fingerprint mismatch | `success=true`，`status=source_changed`，`session_state` 可用 |
| load missing source | root_path 不存在 | `success=true`，`status=source_missing`，`session_state` 可用 |
| load unsafe source | root_path 变为 symlink | `success=true`，`status=source_path_not_allowed`，`session_state` 可用 |
| load version incompatible | 旧 major version | error |

## 4. 前端验证矩阵

| 场景 | 预期 |
|------|------|
| 首次启动 | 最近项目为空，显示“打开其他项目” |
| 分析后保存 | 顶部显示“已保存” |
| 重新打开应用 | 最近项目显示该项目 |
| 点击最近项目 | WorkspacePage 恢复状态 |
| 目标路径不存在 | 显示提示，提供重新选择/删除 |
| 目标项目已变更 | 显示提示，提供查看历史/重新分析 |
| 版本不兼容 | 显示提示，仅可删除 |
| manifest 损坏 | 显示提示，仅可删除 |
| 删除记录 | 二次确认，删除后列表更新，目标项目不变 |

## 5. 桌面验收步骤

### 5.1 样例项目

新建 `/tmp/fpga-flow-mind-phase6-acceptance-YYYYMMDD-HHMMSS` 作为自包含 Phase 6 验收项目。

### 5.2 验收步骤

| 步骤 | 操作 | 预期 |
|------|------|------|
| 1 | 打开项目，选择 L0，收集证据，生成理解，生成视图 | 与 Phase 5 一致 |
| 2 | 点击某个模块节点，查看追溯详情 | TracePanel 展示 |
| 3 | 在 grounded 问答区域提问“这个模块的输入位宽是多少” | 返回带引用回答 |
| 4 | 关闭应用 | 无崩溃，状态已保存 |
| 5 | 重新打开应用 | 最近项目显示该项目 |
| 6 | 点击最近项目 | WorkspacePage 恢复，阶段/视图/evidence/问答一致 |
| 7 | 修改目标项目某个源码文件 | — |
| 8 | 重新打开应用并加载该项目 | 提示“项目文件已变更” |
| 9 | 选择“仅查看历史产物” | 加载历史状态，显示变更警告 |
| 10 | 选择“重新分析” | 清空当前阶段产物，回到未分析状态 |
| 11 | 删除最近项目记录 | 列表消失，目标项目文件不变 |
| 12 | 验证目标项目只读 | checksum 前后一致 |

## 6. 安全回归清单

```bash
# 禁止写入/执行 API 检查（持久化模块外）
rg "std::fs::write|std::fs::create_dir|std::fs::remove_file|std::fs::rename|std::fs::copy|std::process::Command|Command::new" src-tauri/src/

# 注意：持久化模块允许使用 write/create_dir/remove_file/rename，但目标必须限于 app_data_dir。
# 以下检查确保没有写入目标项目：
rg "root_path.*write|workspace.*write|target.*write" src-tauri/src/

# 越界检查：无 Vivado/synthesis/implementation/bitstream
rg "Vivado|synthesis|implementation|bitstream" src-tauri/src/ src/ features/

# 真实 LLM API 检查
rg "openai|anthropic|api_key" src-tauri/src/ src/

# 审计用语检查
rg "PASS|HOLD|正确|错误|审计" src/ src-tauri/src/
```

预期：
- 持久化模块内 `write/create_dir/remove_file/rename` 仅用于 `app_data_dir` 下的路径。
- 无 Vivado/synthesis/implementation/bitstream 调用。
- 无真实 LLM API 调用。
- 审计用语仅出现在禁用列表、测试用例、错误码文案中，不作为用户可见结论。

## 7. MVP 完成标准

- P6-T01~P6-T10 全部完成。
- Rust 测试新增 ~60 个且全部通过，总测试数 ≥ 390。
- `npm run build` 通过。
- `cargo check` 通过。
- 桌面验收 12/12 通过。
- 目标项目只读 checksum 验证通过。
- 无真实 LLM 默认调用。
- 无 PASS/HOLD 审计用语出现在用户可见输出。
- Phase 6 completion review 完成并标记 active。

## 8. 变更记录

| 日期 | 变更 | 作者 |
|------|------|------|
| 2026-06-14 | 初始 draft：定义 Phase 6 测试矩阵、后端/前端/桌面验收、安全回归清单、MVP 完成标准 | Claude |
