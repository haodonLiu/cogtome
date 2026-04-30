<img src="cover.jpg" width="400" alt="COGTOME" />

> [English](README.md) | 中文版本

# COGTOME

> **Agent 的执行层约束 — 减少幻觉，提升可靠性。**

> COGTOME 为 AI Agent 提供经过测试、可复用的执行剧本。Agent 决定 *做什么*；COGTOME 确保执行遵循正确的 DAG、处理错误、保持状态。

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 目录

1. [什么是 COGTOME](#什么是-cogtome)
2. [核心特性](#核心特性)
3. [架构](#架构)
4. [沙箱隔离策略](#沙箱隔离策略)
5. [快速开始](#快速开始)
6. [项目结构](#项目结构)
7. [CLI 参考](#cli-参考)
8. [对比](#对比)
9. [设计原则](#设计原则)
10. [阶段状态](#阶段状态)

---

## 什么是 COGTOME

COGTOME 是一个**运行时，以声明式工作流作为 Agent 工具执行** — 进程隔离、DAG 编排、状态传播、可观测的执行追踪。

### 解决的问题

Agent 知道 *应该做什么*，但经常在 *怎么做* 上出错：
- 工具调用顺序错误
- 参数传递不正确
- 错误处理不完整
- 多步操作状态丢失

### 解决方案

COGTOME 提供经过测试的执行蓝图（Skill），Agent 可以调用。Agent 专注于意图；COGTOME 负责执行的严谨性。

```
Agent 意图  →  COGTOME Skill  →  保证执行
             (DAG + 契约)
```

---

## 核心特性

| 特性 | 说明 |
|------|------|
| **进程隔离** | 每个工具在独立 OS 进程中执行，超时、临时目录沙箱 |
| **分层沙箱隔离** | SandboxBackend trait，4 种后端：bubblewrap、e2b、quickjs、none |
| **零重写适配** | Python 脚本、Bash 命令只要支持 JSON stdin/stdout 即可成为 Unit |
| **JSON Schema 契约** | 输入输出自动校验 |
| **DAG 工作流** | Motif 支持 `if` 分支、`foreach` 循环、并行执行 |
| **MCP Bridge** | 将 MCP Server 作为 COGTOME Unit 运行 |

---

## 架构

COGTOME 采用三层执行模型：

```
Agent (自然语言意图)
        │
        ▼
┌─────────────────────┐
│       Skill         │  ← Agent 可见层
│                     │     名称、描述、输入输出 Schema
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif         │  ← 编排逻辑 (JSON DAG)
│                     │     节点：start, unit, if, match, foreach, fork, join, return
└─────────┬───────────┘
          │ IPC (fork+exec, stdin/stdout JSON)
          ▼
┌─────────────────────┐
│        Unit         │  ← 原子执行 (独立进程)
│                     │     任意语言，JSON stdin/stdout
└─────────────────────┘
```

### 层级概览

| 层级 | 作用 | Agent 可见？ |
|-------|---------|---------------|
| **Skill** | 对外暴露的能力，含描述和 Schema | ✅ 是 |
| **Motif** | JSON DAG 编排逻辑 | ❌ 否 |
| **Unit** | 原子可执行文件 | ❌ 否 |

### 核心纪律

1. **Unit 之间绝不相互调用** — 运行时通过 `COGTOME_UNIT_MODE=1` 阻止递归调用。
2. **所有跨层调用通过运行时 IPC** — 禁止直接耦合。
3. **每个边界都有 Schema 校验** — 坏输入尽早失败。

### SandboxBackend Trait

COGTOME 定义了 `SandboxBackend` trait 用于可插拔隔离。每个 Unit 可以在 `unit.json` 中声明隔离级别：

```json
{
  "name": "my-unit",
  "isolation": "bubblewrap",
  "entry": "bin/my-unit"
}
```

| 后端 | 隔离级别 | 适用场景 |
|------|---------|---------|
| `bubblewrap` | 本地命名空间沙箱 | 大多数 Unit 的默认选择 |
| `e2b` | 远程强隔离 | 不受信代码、网络敏感场景 |
| `quickjs` | 超轻量 JS 沙箱 | 简单 JS 脚本 |
| `none` | 无沙箱（降级） | 受信本地工具 |

---

## 沙箱隔离策略

COGTOME 将"执行什么"与"在哪里执行"的关注点分离。隔离层委托给专用沙箱运行时，而不是重新实现 cgroup/seccomp 逻辑。

### 分层逻辑

1. **Unit 声明隔离** — 在 `unit.json` 中设置（或从 `cogtome.toml` 继承默认值）。
2. **运行时解析后端** — 根据 `isolation` 字段选择对应的 SandboxBackend。
3. **沙箱包裹执行** — fork+exec 在选定的后端内发生。
4. **降级链** — 如果后端不可用，COGTOME 降级到 `none` 并发出警告。

```toml
# cogtome.toml
[units.defaults]
isolation = "bubblewrap"

[units.isolation.my-untrusted-unit]
backend = "e2b"
e2b_api_key = "${E2B_API_KEY}"
```

### 威胁模型覆盖

| 威胁 | bubblewrap | e2b | quickjs | none |
|------|-----------|-----|---------|------|
| 文件系统逃逸 | ✅ | ✅ | ✅ | ❌ |
| 网络访问 | ✅ | ✅ | ✅ | ❌ |
| 进程树逃逸 | ✅ | ✅ | ✅ | ❌ |
| 内核漏洞利用 | ❌ | ✅ | ❌ | ❌ |

---

## 快速开始

### 1. 构建

```bash
git clone https://github.com/haodonLiu/cogtome.git
cd cogtome
cargo build --release
```

### 2. 运行示例

```bash
./target/release/cogtome discover
./target/release/cogtome run text-processing --input '{"text":"hello"}'
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. MCP Bridge

```bash
./target/release/cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
```

### 4. 环境变量

```bash
export COGTOME_SKILLS_DIR=./skills   # Skills 目录（默认：./skills）
export COGTOME_TIMEOUT=60            # Unit 执行超时（默认：30秒）
```

---

## 项目结构

```
cogtome/
├── src/                    # Runtime 源码 (Rust)
│   ├── main.rs             # CLI 入口 (clap)
│   ├── api.rs              # HTTP API 服务器 (axum)
│   ├── assembly.rs         # Assembly 注册表
│   ├── mcp_server.rs       # MCP Server (JSON-RPC 2.0)
│   ├── discovery.rs        # 目录扫描
│   ├── config.rs           # 配置文件加载
│   ├── engine/             # 执行引擎
│   │   ├── mod.rs          # GraphMotifEngine + StructureExecutor
│   │   ├── graph.rs        # 图验证
│   │   ├── unit_runner.rs  # Unit 执行器 (fork+exec)
│   │   └── mcp_bridge.rs   # MCP Bridge
│   └── context/            # 执行上下文
│       ├── expression.rs   # 表达式求值
│       └── variables.rs    # 变量解析
├── skills/                 # Skills 目录（运行时加载）
│   ├── units/<name>/bin/   # 原子可执行文件
│   ├── motifs/<name>.json  # JSON Motif DAG
│   └── <complex>/SKILL.md  # Complex 定义
├── assemblies/             # MCP Server assemblies
│   └── <name>/
│       ├── manifest.json
│       └── workflow.json   # MotifManifestV2 DAG
└── cogtome.toml           # 运行时配置
```

---

## CLI 参考

```bash
cogtome discover                              # 扫描所有 Complex
cogtome run <complex> --input <json>         # 运行 Complex
cogtome motif run <name> --input <json>      # 运行 Motif
cogtome structure run <name> --input <json>  # 运行 Structure
cogtome unit run <name> --input <json>       # 运行 Unit
cogtome serve --port 8080                     # 启动 REST API
cogtome mcp-bridge --server <cmd> --tool <name>  # 运行 MCP Server 为 Unit
cogtome mcp-server --assemblies <dir>        # 启动 MCP Server (stdio 模式)
cogtome pack <skill>                          # 打包为 .cogtome
cogtome install <file.cogtome>               # 安装包
cogtome reload                                # 热重载
cogtome validate <path>                       # 验证 manifest
cogtome stats                                 # Assembly 调用热力图
```

---

## 对比

| 特性 | COGTOME | E2B | MCP | LangChain | Dify/n8n |
|------|---------|-----|-----|-----------|----------|
| **主要目标** | 安全运行现有脚本 | 云端 AI 代码沙箱 | 协议标准 | Python 框架 | 人类工作流 |
| **是否需要重写工具** | ❌ 否 | ⚠️ Python SDK | ✅ 是 | ⚠️ Python 包装器 | ⚠️ 通常需要 |
| **进程隔离** | ✅ 分层后端 | ✅ MicroVM | 取决于宿主 | ❌ 进程内 | ✅ 服务端 |
| **沙箱选项** | 4 种后端 | 单一 (Firecracker) | 无 | 无 | 无 |
| **Agent 原生接口** | ✅ CLI | ✅ Python/JS SDK | 协议 | Python API | GUI/API |
| **最适合** | 本地脚本沙箱 | 远程不受信代码 | 跨平台工具 | Python 应用集成 | 业务自动化 |

---

## 设计原则

1. **不让用户先学隐喻** — 东西就叫它本身的名字：Unit、Motif、Skill。
2. **零重写接入** — 你现有的脚本就是资产。保留它们。
3. **默认隔离** — 每个工具在独立进程中运行。没有例外。
4. **Schema 契约** — 每个边界都有 JSON Schema 校验。
5. **MCP 兼容** — 我们不与 MCP 竞争；我们运行它。
6. **开源优先** — 纯 Rust 实现，无闭源依赖。每一行代码可审计。
7. **隔离外包** — 不重复造 seccomp 轮子。通过 trait 委托给 bubblewrap、e2b、quickjs。

---

## 阶段状态

### Phase 1: 核心运行时 ✅

- [x] 四层执行模型 (Complex → Structure → Motif → Unit)
- [x] CLI 框架 (discover, run, unit/motif/structure)
- [x] Unit 执行 (fork+exec, stdin/stdout JSON, timeout, temp sandbox)
- [x] JSON Motif 解析与执行 (DAG graph)
- [x] Complex 发现 (SKILL.md front-matter parsing)
- [x] `foreach` 循环、`if` 条件执行、错误策略
- [x] HTTP API 服务器、打包/安装、MCP Bridge、MCP Server
- [x] Assembly 注册表、热力图、Graceful shutdown

### Phase 2: 易用性 🔧

- [ ] 完整集成测试覆盖 (test_suite/)
- [ ] `cogtome run` 稳定跑通 100 次
- [ ] Motif 内联脚本节点、`cogtome wrap` 一键迁移工具
- [ ] 分层沙箱后端 (bubblewrap, e2b, quickjs)

### Phase 3: 可观测性 📊

- [ ] 执行链路日志、Checkpoint 节点、Prometheus 指标

### Phase 4: 集成 🔗

- [ ] KimiCLI bridge、OpenClaw gateway、文件系统自动重载、Skill 注册中心

---

## 相关链接

- [用户手册](./docs/USER_MANUAL.md)
- [技术规格](./development/TECHNICAL_SPEC.md)
- [Skill 编写指南](./development/SKILL_AUTHORING_GUIDE.md)

---

## 许可证

MIT
