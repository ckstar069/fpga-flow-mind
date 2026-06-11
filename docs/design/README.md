# 设计文档索引

---
status: active
updated: 2026-06-11
---

## 设计目录用途

本目录存放 `fpga-flow-mind` 的技术设计文档。设计文档描述"系统如何构建、数据如何流转、组件如何协作"，不描述产品需求或界面细节。

## 建议文档类型

| 文档类型 | 说明 | 示例 |
|----------|------|------|
| `semantic data contract` | 语义数据契约 | 结构化理解产物的 schema、字段定义、版本约定 |
| `agent workflow` | Agent 工作流设计 | 大模型/Agent 的调查计划、理解流程、grounding 检查 |
| `workspace scanning design` | Workspace 扫描设计 | 如何发现、识别和索引业务项目中的阶段和文件 |
| `evidence model` | 证据模型 | EvidenceItem 定义、证据抽取、索引、关联 |
| `visualization spec` | 可视化规范 | 图数据格式、渲染要求、交互协议 |
| `persistence design` | 持久化设计 | 产物存储格式、加载协议、版本迁移 |
| `provider boundary` | Provider 调用边界 | 大模型 API 调用、错误处理、配额控制、审计 |
| `safety boundary` | 安全边界 | 目标项目只读约束、文件访问白名单、禁止操作 |
| `Tauri/Rust/React architecture` | 技术架构 | 前后端职责划分、command 边界、状态管理 |

## 设计原则

### 1. 大模型/Agent 是主语义理解引擎

- 系统的核心理解能力来自大模型/Agent，不是静态分析
- 静态分析负责提供证据、索引和约束，不独立做出语义结论

### 2. 静态分析是辅助证据基础设施

- 静态分析负责：文件/符号/调用提取、RTL module/port/signal 提取、测试断言提取、行号和切片
- 静态分析不独立回答：主算法路径是什么、哪些逻辑是主路径、Python 概念如何映射到 RTL

### 3. 图来自结构化理解对象，不是自由文本拼接

- 可视化产物应基于结构化的 `ImplementationUnderstanding` 对象生成
- 不应将大模型返回的自由文本直接拼接成图

### 4. 用户可见主要结论必须绑定 evidence id、源码路径、行号范围

- 每个 claim 必须关联到具体的 evidence
- 用户应能追溯到源码的精确位置

### 5. 目标项目只读

- 系统设计必须确保对业务项目的访问是只读的
- 所有写入操作应发生在 app-owned 目录或临时目录

## 明确约束

- **Python 不能作为产品核心实现** — 后端必须用 Rust，前端用 React/TypeScript
- Python 可用于原型验证、工具脚本、辅助分析，但不应成为产品主路线

## 当前文档列表

| 文档 | 状态 | 说明 | 推荐阅读时机 |
|------|------|------|-------------|
| （待补充） | — | — | — |

> 注：当前设计参考主要来源于 `../initial-requirements-draft/MVP_ARCHITECTURE.md`，后续应从中提炼为正式设计文档。
