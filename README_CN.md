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
4. [快速开始](#快速开始)
5. [项目结构](#项目结构)
6. [CLI 参考](#cli-参考)
7. [Web UI](#web-ui)
8. [对比](#对比)
9. [设计原则](#设计原则)

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
| **零重写适配** | Python 脚本、Bash 命令只要支持 JSON stdin/stdout 即可成为 Unit |
| **JSON Schema 契约** | 输入输出自动校验 |
| **DAG 工作流** | Motif 支持 `if` 分支、`foreach` 循环、并行执行 |
| **MCP Bridge** | 将 MCP Server 作为 COGTOME Unit 运行 |
| **可视化编辑器** | Web UI 拖拽式图形编辑器 |

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
# 发现所有 Complex
./target/release/cogtome discover

# 运行 Complex
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# 直接运行 Motif
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'

# 直接运行 Unit
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. MCP Bridge

```bash
# 将 MCP Server 作为 COGTOME Unit 运行
./target/release/cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
```

### 4. 环境变量

```bash
# Skills 目录（默认：./skills）
export COGTOME_SKILLS_DIR=./skills

# Unit 执行超时（默认：30秒）
export COGTOME_TIMEOUT=60
```

---

## 项目结构

```
cogtome/
├── src/                    # Runtime 源码 (Rust)
│   ├── main.rs             # CLI 入口 (clap)
│   ├── api.rs              # HTTP API 服务器 (axum)
│   ├── discovery.rs        # 目录扫描
│   ├── config.rs           # 配置文件加载
│   ├── engine/             # 执行引擎
│   │   ├── mod.rs          # GraphMotifEngine + StructureExecutor
│   │   ├── graph.rs        # 图验证
│   │   ├── unit_runner.rs  # Unit 执行器 (fork+exec)
│   │   └── mcp_bridge.rs  # MCP Bridge
│   └── context/            # 执行上下文
│       ├── expression.rs   # 表达式求值
│       └── variables.rs    # 变量解析
├── webui/                  # Web UI (React + React Flow)
├── skills/                 # Skills 目录（运行时加载）
│   ├── units/<name>/bin/   # 原子可执行文件
│   ├── motifs/<name>.json  # JSON Motif DAG
│   └── <complex>/SKILL.md  # Complex 定义
└── cogtome.toml           # 运行时配置
```

---

## CLI 参考

```bash
# 发现
cogtome discover                              # 扫描所有 Complex

# 执行
cogtome run <complex> --input <json>         # 运行 Complex
cogtome motif run <name> --input <json>      # 运行 Motif
cogtome structure run <name> --input <json>  # 运行 Structure
cogtome unit run <name> --input <json>       # 运行 Unit

# HTTP API 服务器
cogtome serve --port 8080                     # 启动 REST API

# MCP
cogtome mcp-bridge --server <cmd> --tool <name>  # 运行 MCP Server 为 Unit
cogtome mcp-server --assemblies <dir>        # 启动 MCP Server (stdio 模式)

# 打包与安装
cogtome pack <skill>                          # 打包为 .cogtome
cogtome install <file.cogtome>               # 安装包

# 工具
cogtome reload                                # 热重载
cogtome validate <path>                       # 验证 manifest
cogtome stats                                 # Assembly 调用热力图
```

---

## Web UI

COGTOME 包含一个 **可视化工作室**，用于创作和调试 Motif。

### 启动 Web UI

```bash
# 一键启动
./start-webui.sh

# 或手动
cargo build --release
./target/release/cogtome serve --port 3334 &
cd webui && npm install && npm run dev
```

访问 **http://localhost:3333**

### 功能

- **图形编辑器**：拖拽式编排，支持 9 种节点类型
- **自动布局**：基于网格的节点定位
- **执行追踪**：查看每个步骤的数据流

---

## 对比

| 特性 | COGTOME | MCP | LangChain | Dify/n8n |
|---------|---------|-----|-----------|----------|
| **主要目标** | 安全运行现有脚本 | 协议标准 | Python 框架 | 人类工作流 |
| **是否需要重写工具** | ❌ 否 | ✅ 是 | ⚠️ Python 包装器 | ⚠️ 通常需要 |
| **进程隔离** | ✅ 是 | 取决于宿主 | ❌ 进程内 | ✅ 服务端 |
| **Agent 原生接口** | ✅ CLI | 协议 | Python API | GUI/API |
| **最适合** | 本地脚本沙箱 | 跨平台工具 | Python 应用集成 | 业务自动化 |

---

## 设计原则

1. **不让用户先学隐喻** — 东西就叫它本身的名字：Unit、Motif、Skill。
2. **零重写接入** — 你现有的脚本就是资产。保留它们。
3. **默认隔离** — 每个工具在独立进程中运行。没有例外。
4. **Schema 契约** — 每个边界都有 JSON Schema 校验。
5. **MCP 兼容** — 我们不与 MCP 竞争；我们运行它。
6. **可视化 + 文本化** — 同时支持图形编辑器和 JSON 编写。

---

## 相关链接

- [用户手册](./docs/USER_MANUAL.md)
- [技术规格](./development/TECHNICAL_SPEC.md)
- [Skill 编写指南](./development/SKILL_AUTHORING_GUIDE.md)

---

## 许可证

MIT
