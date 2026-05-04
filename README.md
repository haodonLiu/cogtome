# COGTOME

> **让 Agent 拥有可自编排、自进化、可观测的执行层。**

COGTOME 是一个 Rust 实现的高可靠 Agent 运行时框架——将任何脚本转化为可组合、可隔离、可自进化的执行单元，为 AI Agent 提供经过验证的执行 playbook。

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 核心定位

**Agent 知道该做什么，但常常做不好怎么执行。**

- 工具调用顺序错误
- 参数类型传错
- 错误处理不完整
- 多步骤状态中途丢失
- 执行过程对 Agent 不透明

COGTOME 为 Agent 提供一个**经过验证的执行蓝图（Structure）**——Agent 专注意图，COGTOME 保证执行严格按 DAG 路径进行，处理好错误，维护状态，并留下完整执行痕迹供自我分析。

```
Agent 意图  ->  COGTOME Structure  ->  DAG 执行 + 隔离 + 可观测
```

---

## 核心特性

### 自编排（Self-Orchestrating）

三层执行模型，Agent 只感知 Structure，底层执行由 DAG 引擎驱动：

| 层级 | 可见性 | 职责 |
|------|--------|------|
| **Structure** | Agent 可见 | 名称、描述、输入输出 Schema（skills/ 或 assemblies/） |
| **Motif** | 内部 | JSON DAG 编排（if/foreach/fork/join） |
| **Unit** | 内部 | 原子进程（任意语言，JSON stdin/stdout） |

### 可观测（Observable）

每一步执行都输出结构化事件流：

```
{"event":"step_start","step_id":1,"unit":"http-get","timestamp":1700000000}
{"event":"variable_resolved","name":"url","source":"input","resolved":"https://..."}
{"event":"step_end","step_id":1,"duration_ms":234,"status":"ok","output":{...}}
{"event":"execution_failed","step_id":2,"error":"timeout","retry_attempt":1}
```

事件流写入 stderr，支持 HTTP SSE 推送，配合 trace 可视化 dashboard 使用。

### MCP 原生运行（MCP-Native）

COGTOME 不是另一个协议桥——**原生运行 MCP Servers 作为 COGTOME Units**，利用现有 MCP 生态：

```bash
# 把任何 MCP Server 直接作为 Unit 使用
cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
```

---

## 架构

```
Agent (意图层)
    |
    v
+-----------------------------+
|       Structure             |
|  (skills/ 或 assemblies/)   |
|  声明输入输出 Schema         |
+-------------+---------------+
              | DAG 执行引擎
              v
+-----------------------------+
|         Motif               |
|  if/foreach/fork/join/retry |
|  DAG 节点类型，控制执行路径  |
+-------------+---------------+
              | fork + exec
              v
+-----------------------------+
|         Unit                |
|  任意语言，JSON stdin/stdout |
|  每步独立进程，资源上限      |
+-----------------------------+
```

---

## 快速开始

```bash
# 编译
git clone https://github.com/haodonLiu/cogtome.git
cd cogtome && cargo build --release

# 发现所有 Structures
./target/release/cogtome discover

# 运行一个 Structure
./target/release/cogtome run text-processing --input '{"text":"hello world"}'

# 启动 API Server
./target/release/cogtome serve --port 8080

# 启动 MCP Server（stdio 模式）
./target/release/cogtome mcp-server --assemblies ./assemblies
```

### 环境变量

```bash
export COGTOME_SKILLS_DIR=./skills    # Structures 目录
export COGTOME_TIMEOUT=60             # Unit 超时（秒）
```

---

## 项目结构

```
cogtome/
|-- src/
|   |-- main.rs              # CLI 入口（clap）
|   |-- api.rs               # HTTP API Server（axum + tokio）
|   |-- mcp_server.rs        # MCP Server（JSON-RPC 2.0）
|   |-- assembly.rs          # Assembly 注册表
|   |-- discovery.rs         # 目录扫描
|   |-- validation.rs        # JSON Schema 输入校验
|   |-- engine/              # 执行引擎
|   |   |-- mod.rs           # GraphMotifEngine
|   |   |-- unit_runner.rs    # 进程启动 + 协议解析
|   |   |-- graph.rs         # DAG 验证
|   |   |-- mcp_bridge.rs    # MCP Bridge
|   |   |-- protocol.rs      # stdout/stderr 协议约定
|   |   |-- events.rs        # 执行事件流
|   |-- context/              # 执行上下文
|   |   |-- variables.rs      # 变量作用域隔离
|   |   |-- expression.rs     # 聚合表达式
|-- skills/                   # Skills 目录（运行时加载）
|   |-- <name>/
|   |   |-- SKILL.md         # Structure 定义
|   |   |-- motifs/          # Motifs
|   |   |-- units/           # Units
|-- assemblies/               # MCP Server Assemblies
|   |-- <name>/
|   |   |-- manifest.json
|   |   |-- workflow.json
|-- development/             # 技术规格、编写指南
|-- cogtome.toml            # 运行时配置
```

---

## Phase 状态

| Phase | 内容 | 状态 |
|-------|------|------|
| **Phase 1** | 核心运行时（Unit/Motif/Structure 三层模型、CLI、HTTP API、MCP Bridge） | Done |
| **Phase 2** | 可用性（输入校验、错误策略、并发隔离、执行事件流） | Done |
| **Phase 3** | 自进化（trace-logger + 分析） | Planned |

---

## 设计原则

1. **零重写采纳** — 你现有的脚本就是资产，保留它们。
2. **进程隔离默认** — 每个工具跑在独立进程，无例外。
3. **Schema 契约** — 每个边界做 JSON Schema 校验，坏输入快速失败。
4. **MCP 借力生态** — 不重新发明轮子，原生运行 MCP Servers。
5. **纯 Rust** — 无闭源依赖，每行代码可审计。

---

## 与其他方案对比

| 特性 | COGTOME | E2B | MCP | LangChain | Dify/n8n |
|------|---------|-----|-----|-----------|----------|
| **定位** | Agent 执行层 | 云端沙箱 | 协议标准 | Python 框架 | 人工工作流 |
| **脚本需要重写** | 否 | 需要 Python/JS | 需要 | 需要 Python | 需要 |
| **进程隔离** | 分层后端 | MicroVM | 依赖宿主机 | 进程内 | 服务端 |
| **可观测性** | 执行事件流 | 基础 | 无 | 回调 | 好 |
| **MCP 原生** | 是 | 否 | 是 | 部分 | 否 |

---

## 文档

- [用户手册](./docs/USER_MANUAL.md)
- [技术规格](./development/TECHNICAL_SPEC.md)
- [编写指南](./development/SKILL_AUTHORING_GUIDE.md)

---

## License

MIT