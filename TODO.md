# TODO

## 核心理念

**COGTOME 是 Agent 的执行层约束。** Agent 的核心问题不是"想不清楚"，而是"做的时候出错"——工具调用顺序错、参数传错、错误没处理干净。

COGTOME 给 Agent 一个**写好的、测试过的、可复用的执行剧本**。Agent 只需要说"执行这个 Skill"，Runtime 负责 DAG 执行、状态传播、错误处理。

---

## 已完成

- [x] 四层执行模型 (Complex → Structure → Motif → Unit)
- [x] CLI 框架 (discover, run, unit/motif/structure)
- [x] Unit 执行 (fork+exec, stdin/stdout JSON, timeout, temp sandbox)
- [x] JSON Motif 解析与执行 (DAG graph)
- [x] Complex 发现 (SKILL.md front-matter parsing)
- [x] `foreach` 循环
- [x] `if` 条件执行
- [x] 错误策略 (fail, continue, fallback)
- [x] HTTP API 服务器
- [x] 打包/安装 (tar.gz)
- [x] Web UI 可视化 DAG 编辑器
- [x] MCP Bridge Unit
- [x] MCP Server (stdio 模式)
- [x] Assembly 注册表与热力图
- [x] Graceful shutdown

---

## Phase 1 稳定

### 测试与验证
- [ ] 完整集成测试覆盖 (test_suite/)
- [ ] `cogtome run` 稳定跑通 100 次
- [ ] 文档完善 (USER_MANUAL.md)

---

## Phase 2 易用性

### 降低使用门槛
- [ ] Motif 内联脚本节点：`type: script`, `lang: python/bash`
- [ ] `cogtome wrap ./my_script.py --name text-analyzer` 一键迁移
- [ ] 自动检测脚本参数，生成 unit.json Schema

### 真正的隔离
- [ ] Docker Unit Runner
- [ ] 支持 `runtime: docker`, `image`, `memory_limit`, `timeout`

---

## Phase 3 可观测性

- [ ] 执行链路日志 (完整 trace)
- [ ] Checkpoint 节点 (断点续跑)
- [ ] Prometheus 指标导出
- [ ] `/metrics` 端点增强

---

## Phase 4 集成

- [ ] KimiCLI bridge
- [ ] OpenClaw gateway bridge
- [ ] 文件系统自动重载 (notify crate)
- [ ] Skill 注册中心

---

## 代码规范

### Rust 运行时
- [ ] **强制 async I/O**：所有 I/O 必须用 `tokio` API
- [ ] **统一错误处理**：`anyhow::Result` + `anyhow::Context`
- [ ] **JSON 处理规范**：`serde_json::Value` 动态场景，struct 强类型场景
- [ ] **ExecContext 不可变原则**：使用 `Arc<HashMap>` 保持 O(1) 快照

### 工程检查
- [ ] **Clippy 零警告**：`cargo clippy -- -D warnings`
- [ ] **单测覆盖率**：新增代码必须附带单元测试

---

## 不要做的事

- ❌ 不要造新协议（用 MCP + HTTP API 就够了）
- ❌ 不要写论文式架构文档
- ❌ 不要追求"完美抽象"（让用户 5 分钟封装一个脚本更重要）
