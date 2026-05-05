# COGTOME

> **让 Agent 拥有可自编排、自进化、可观测的执行层。**

COGTOME 是一个 Rust 实现的高可靠 Agent 运行时框架——将任何脚本转化为可组合、可隔离、可自进化的执行单元，为 AI Agent 提供经过验证的执行 playbook。

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

> [English](README.md) | [中文版本](README_CN.md)

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

每一步执行都输出结构化事件流，配合 Trace Dashboard 可视化：

```bash
cogtome trace-dashboard   # 启动 Trace 可视化面板
```

### MCP 原生运行（MCP-Native）

COGTOME 原生运行 MCP Servers 作为执行单元：

```bash
# 作为 MCP Server（stdio 模式）
cogtome mcp-server --assemblies ./assemblies --units ./units

# 将 MCP Server 工具作为 Unit 使用
cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
```

### WebUI

内置 Web 管理界面，浏览 Structures、Motifs、Units 和 Trace 历史：

```bash
cogtome serve --port 3334
# 浏览器访问 http://localhost:3334
```

WebUI 已内嵌到二进制文件中，无需额外分发静态资源。

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
| **可观测性** | Trace + Dashboard | 基础 | 无 | 回调 | 好 |
| **MCP 原生** | 是 | 否 | 是 | 部分 | 否 |

---

## 文档

- [用户手册](./docs/USER_MANUAL.md)
- [技术规格](./docs/SPEC.md)
- [贡献指南](./docs/CONTRIBUTING.md)
- [安全说明](./docs/SECURITY.md)

---

## License

MIT
