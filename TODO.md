# TODO

## 核心理念

**COGTOME 是 Agent 的执行层约束。** Agent 的核心问题不是"想不清楚"，而是"做的时候出错"——工具调用顺序错、参数传错、错误没处理干净。

COGTOME 给 Agent 一个**写好的、测试过的、可复用的执行剧本**。Agent 只需要说"执行这个 Skill"，Runtime 负责 DAG 执行、状态传播、错误处理。

**Skills 自净化闭环：** 规范化执行 → 暴露 Skill 不足 → Agent 改进 → 重新验证。通过可观测的 DAG 执行，Agent 不仅能减少幻觉，还能学会优化自己的工具。

## 已完成

- [x] WebUI 设计系统优化 (Light theme)
- [x] Structure/Motif 编辑器浮窗属性面板
- [x] 画布背景切换 (dot/grid)
- [x] Unit/Motif 列表块/条状切换
- [x] 完整 Unit/Motif 创建流程 (含模板选择)
- [x] 无界画布编辑器
- [x] Rust GraphMotifEngine JSON 执行
- [x] API 端点 v2 JSON 格式
- [x] 修复节点组件 undefined data 报错
- [x] 修复 React Flow edge handle id 问题

---

## 第一阶段：止血与定位修正（0–2 周）

### 重新定义产品定位
- [x] README 去概念化：删除"微操作系统"修辞，聚焦"Agent 执行层约束"
- [x] 核心理念确立：Skills 自净化闭环
- [ ] 更新 README 和对外文档，统一描述

### 接入 MCP 生态（生死线）
- [ ] 实现 MCP Bridge Unit：通过 stdio JSON-RPC 2.0 转发，将 MCP Server 作为普通 Unit 运行
- [ ] 验证 `cogtome run mcp-server-filesystem --input '{"path": "/tmp"}'` 能稳定跑通

### 砍掉/合并冗余分层
- [ ] 将 Structure 和 Complex 合并为 Skill 层
- [ ] 统一目录结构：`skills/<skill-name>/` 取代 `structures/` + `<complex>/`
- [ ] 更新 CLI：`cogtome run <skill>` 作为唯一入口

---

## 第二阶段：建立最小可用闭环（2–6 周）

### 降低使用门槛
- [ ] Motif 内联脚本节点：支持 `type: script`, `lang: python/bash`
- [ ] Runtime 用临时文件 + fork+exec 执行内联脚本，输出自动映射到上下文
- [ ] `cogtome wrap ./my_script.py --name text-analyzer` 一键迁移 CLI
- [ ] 自动检测脚本参数，生成 unit.json Schema；不支持 `--json` 时生成 wrapper shim

### 真正的隔离
- [ ] Docker Unit Runner（可选）
- [ ] 支持 `runtime: docker`, `image`, `memory_limit`, `timeout` 等字段
- [ ] Agent 调用不可信脚本时有资源限制和沙箱

---

## 第三阶段：差异化与可观测性（6–12 周）

### 深度集成（本地 Agent 联邦）
- [ ] KimiCLI Bridge：利用 Wire/ACP 模式，注册为长连接 Unit，减少冷启动延迟
- [ ] OpenClaw Gateway Bridge：对接 `ws://127.0.0.1:18789`，映射 Agent-to-Agent 通信为 Motif 节点

### 状态持久化与可观测性
- [ ] 执行链路日志：每个 Motif 运行的完整 trace（输入、输出、耗时、错误）
- [ ] Checkpoint 节点：崩溃后从断点续跑
- [ ] Prometheus 格式指标导出（调用频次、Unit 成功率）

### Skills 自净化闭环（核心价值）
- [ ] 执行链路可观测：Agent 能"看到"每个节点的输入、输出、耗时、错误
- [ ] Skill 不足识别：执行失败时自动分析 Skill 定义中的缺陷
- [ ] Agent 自我改进：识别问题 → 修改 Skill → 重新验证
- [ ] Skill 版本化：改进后的 Skill 保存为新版本，保留历史

### Web UI：创作与调试双模式
- [ ] 低代码创作增强：更友好的属性面板、节点模板、连线校验
- [ ] 执行可视化：看 Motif 运行时数据怎么流转（哪个节点卡住、输入输出是什么）
- [ ] 一键测试：对单个 Unit 快速填参数、跑一下、看结果
- [ ] Debug 模式：运行时在图上实时高亮当前节点、展示中间数据

---

## 长期计划

### 现有待完成项
- [ ] 完善 agent 辅助设计开发 (ChatAssistant)
- [ ] 调整 Structure 中逻辑块的设计方法（Skill 合并后重新设计）

### Agent 辅助设计开发
- 增强 ChatAssistant 功能
- 更好的上下文理解和代码生成

### Skill 逻辑块调整
- 优化结构编辑器的节点设计
- 改进 Skill 层面的可视化逻辑

---

## 代码规范与工程纪律

### Rust 运行时
- [ ] **强制 async I/O**：所有 I/O 操作必须使用 `tokio` API，禁止在 async 上下文中使用 `std::fs` 或 `std::process`（启动配置加载除外）
- [ ] **统一错误处理**：边界函数返回 `anyhow::Result`，用 `anyhow::Context` 丰富错误信息，优先 `?` 而非 `unwrap`/`expect`
- [ ] **JSON 处理规范**：动态 Schema 用 `serde_json::Value`，manifest 等强类型场景用 struct，禁止混用导致类型污染
- [ ] **命名规范**：函数/变量 `snake_case`，类型 `PascalCase`；四层概念名（Unit / Motif / Skill）作为专有名词保持 `PascalCase`
- [ ] **ExecContext 不可变原则**：步骤存储使用 `Arc<HashMap>`，快照保持 O(1)，修改时 clone-on-write
- [ ] **死代码注释**：`#[allow(dead_code)]` 必须附带说明该字段的计划用途，禁止无理由保留

### 模块化与复用
- [ ] **DRY 原则**：提取重复逻辑为独立函数 / struct / trait，禁止复制粘贴超过 3 行的重复代码块
- [ ] **多用 struct + trait 组合**：将行为抽象为 trait，数据封装为 struct，用组合替代重复实现
- [ ] **泛型与抽象**：对相似但类型不同的逻辑（如不同 Runner、不同 Validator）优先使用泛型参数或 trait bound，而非各自写一套
- [ ] **引擎层复用**：UnitRunner、MotifEngine、StructureExecutor 的公共逻辑（超时、重试、上下文快照）提取到共享 trait 或辅助模块

### 工程检查
- [ ] **Clippy 零警告**：`cargo clippy -- -D warnings` 通过
- [ ] **格式化检查**：`cargo fmt --check` 加入 CI
- [ ] **单测覆盖率**：新增代码必须附带单元测试（参考 `expression.rs` 和 `discovery.rs` 的测试模式）

---

## 不要做的事

- ❌ 不要造新协议（用 MCP + HTTP API 就够了）
- ❌ 不要写论文式架构文档（先让 `cogtome run` 能稳定跑 100 次不崩）
- ❌ 不要追求"四层完美抽象"（让用户 5 分钟封装一个脚本比 50 分钟理解 Cog/Tome 更重要）
