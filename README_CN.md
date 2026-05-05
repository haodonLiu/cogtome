<img src="docs/cover.jpg" width="400" alt="COGTOME" />

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
3. [快速开始](#快速开始)
4. [打包分发](#打包分发)
5. [项目结构](#项目结构)
6. [CLI 参考](#cli-参考)
7. [对比](#对比)
8. [设计原则](#设计原则)

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

COGTOME 提供经过测试的执行蓝图（Structure），Agent 可以调用。Agent 专注于意图；COGTOME 负责执行的严谨性。

```
Agent 意图  ->  COGTOME Structure  ->  保证执行
             (DAG + 契约)
```

---

## 核心特性

| 特性 | 说明 |
|------|------|
| **进程隔离** | 每个工具在独立 OS 进程中执行，超时、临时目录沙箱 |
| **零重写适配** | Python 脚本、Bash 命令只要支持 JSON stdin/stdout 即可成为 Unit |
| **JSON Schema 契约** | 输入输出自动校验 |
| **DAG 工作流** | Motif 支持 `if` 分支、`foreach` 循环、`fork/join` 并行执行 |
| **MCP 原生** | 原生运行 MCP Server，支持 stdio 模式和 Bridge 模式 |
| **WebUI** | 内置 Web 管理界面，浏览 Structures/Motifs/Units/Traces |
| **Trace 可视化** | 执行链路追踪 + Dashboard，定位性能瓶颈 |

---

## 快速开始

```bash
git clone https://github.com/haodonLiu/cogtome.git
cd cogtome

just build    # 构建
just start    # 启动 WebUI → http://localhost:3334
just run <name>  # 运行 skill
```

---

## 打包分发

```bash
just deb   # Linux .deb
just win   # Windows .exe
```

---

## 项目结构

```
cogtome/
├── src/                    # Rust 源码
│   ├── main.rs             # CLI 入口（clap）
│   ├── api.rs              # HTTP API + WebUI 静态服务（axum）
│   ├── mcp_server.rs       # MCP Server（JSON-RPC 2.0 stdio）
│   ├── engine/             # 执行引擎（GraphMotifEngine, UnitRunner）
│   ├── context/            # 执行上下文（变量解析、表达式求值）
│   ├── discovery.rs        # 目录扫描
│   ├── config.rs           # cogtome.toml 配置加载
│   └── assembly.rs         # Assembly 注册表
├── webui/                  # 前端（React + TypeScript）
├── skills/                 # Skills 目录（运行时加载）
│   ├── units/              # Units（原子执行体）
│   ├── motifs/             # Motifs（JSON DAG 编排）
│   └── structures/         # Structures（业务结构）
├── assemblies/             # MCP Server Assemblies
├── units/                  # 独立 Units（含构建产物）
├── packaging/              # 打包脚本（deb, windows）
├── docs/                   # 文档（用户手册、技术规格、设计笔记）
├── scripts/                # 工具脚本
├── tests/                  # 集成测试
├── Cargo.toml              # Rust 项目配置
├── build.rs                # 构建脚本（嵌入 git hash）
├── justfile                # 构建命令
└── cogtome.toml            # 运行时配置
```

---

## CLI 参考

```bash
cogtome run <name> --input <json>    # 运行 skill / unit / motif
cogtome discover                     # 列出所有 skill
cogtome serve --port 3334            # 启动 WebUI + API
cogtome pack <name>                  # 打包为 .cogtome
cogtome install <file.cogtome>       # 安装
```

完整命令参考（MCP、Trace、validate 等高级功能）→ [docs/CLI_REFERENCE.md](./docs/CLI_REFERENCE.md)

---

## 对比

| 特性 | COGTOME | E2B | MCP | LangChain | Dify/n8n |
|------|---------|-----|-----|-----------|----------|
| **主要目标** | 安全运行现有脚本 | 云端 AI 代码沙箱 | 协议标准 | Python 框架 | 人类工作流 |
| **是否需要重写工具** | 否 | Python SDK | 是 | Python 包装器 | 通常需要 |
| **进程隔离** | 分层后端 | MicroVM | 取决于宿主 | 进程内 | 服务端 |
| **可观测性** | Trace + Dashboard | 基础 | 无 | 回调 | 好 |
| **Agent 原生接口** | CLI + WebUI | Python/JS SDK | 协议 | Python API | GUI/API |
| **最适合** | 本地脚本沙箱 | 远程不受信代码 | 跨平台工具 | Python 应用集成 | 业务自动化 |

---

## 设计原则

1. **不让用户先学隐喻** — 东西就叫它本身的名字：Unit、Motif、Structure。
2. **零重写接入** — 你现有的脚本就是资产。保留它们。
3. **默认隔离** — 每个工具在独立进程中运行。没有例外。
4. **Schema 契约** — 每个边界都有 JSON Schema 校验。
5. **MCP 兼容** — 我们不与 MCP 竞争；我们运行它。
6. **开源优先** — 纯 Rust 实现，无闭源依赖。每一行代码可审计。

---

## 文档

- [用户手册](./docs/USER_MANUAL.md)
- [技术规格](./docs/SPEC.md)
- [贡献指南](./docs/CONTRIBUTING.md)
- [安全说明](./docs/SECURITY.md)

---

## 许可证

MIT
