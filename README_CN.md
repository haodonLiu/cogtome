<img src="cover.jpg" width="400" alt="COGTOME" />

> [English](README.md) | 中文版本

# COGTOME

> **齿轮转动典籍，机械执行技艺。**
>
> COGTOME 是面向 Agent 的微型操作系统与执行运行时。
> Agent 铸造齿轮（Unit），组装传动组（Motif），封装传动机构（Structure），收录领域典籍（Complex）。
> Runtime 负责发现、编译、调度、执行与回收。

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

---

## 目录

1. [什么是 COGTOME](#什么是-cogtome)
2. [核心亮点](#核心亮点)
3. [核心架构：四层模型](#核心架构四层模型)
4. [快速开始](#快速开始)
5. [项目结构](#项目结构)
6. [CLI 参考](#cli-参考)
7. [Web UI](#web-ui)
8. [路线图](#路线图)
9. [设计原则](#设计原则)

---

## 什么是 COGTOME

COGTOME **不是**框架，**不是**库 — 它是一个**独立的进程级运行时**：Agent 的微型操作系统。

| 操作系统概念 | COGTOME 对应 |
|-----------|------------|
| 内核 | COGTOME Runtime (Rust) |
| 用户进程 | Agent (LLM / 程序) |
| 系统调用 | Unit (原子执行) |
| 用户态函数 | Motif (编排逻辑) |
| 应用程序 | Structure (业务封装) |
| 应用商店 | Complex (领域门面) |
| Shell | `cogtome` CLI |
| GUI | Web UI (React Flow) |

### 核心问题

Agent 需要调用外部工具，但直接 `subprocess` 会导致：
- 进程管理混乱（泄漏、僵尸进程）
- 无类型安全（输入输出无契约）
- 无版本与发现机制
- 无执行链路追踪

COGTOME 解决以上问题：Agent 负责**编写业务逻辑**，Runtime 负责**基础设施**。

### 品牌隐喻

| 技术术语 | 隐喻 | 含义 |
|---------|------|------|
| Unit | 齿 (Cog) | 不可再分的原子执行体 |
| Motif | 齿轮组 | 齿的编排与组合 |
| Structure | 传动机构 | 完成业务目标的结构 |
| Complex | 典籍 (Tome) | 收录传动机构的领域之书 |

---

## 核心亮点

**🎯 Agent 原生 CLI 系统** — COGTOME **为 Agent 设计，由 Agent 使用**。Agent 通过纯 CLI 进行语义交互（"读取文件"、"抓取网页"），而非原始 shell 命令（"cat /path"、"curl url"）。无需人类介入。

**🧩 分层抽象** — 四层模型（Unit → Motif → Structure → Complex）清晰分离原子执行与业务逻辑。Agent 专注"做什么"，Runtime 负责"怎么做"。

**🎨 低代码 Skill 创建** — Web UI 提供拖拽式 React Flow 编辑器，可视化组合 Motifs 和 Structures。人类无需写代码即可构建 Skills，Agent 通过 CLI 使用。

**🔌 协议无关** — 不同于 MCP Server 需要为每个工具适配协议，COGTOME Units 是语言无关的可执行文件。任何支持 JSON stdin/stdout 的程序均可即插即用。

**🏗️ Runtime 零业务逻辑** — COGTOME 二进制本身不内置任何工具。所有能力均来自 Skills——真正的关注点分离。

---

## 对比

| 特性 | COGTOME | MCP Servers | LangChain | Dify/n8n |
|--------|---------|-------------|-----------|-----------|
| **主要用户** | Agent | Agent | 开发者 | 人类 |
| **交互方式** | 纯 CLI | 协议 | Python API | GUI |
| **Skill 创建** | CLI + Web UI | 需要写代码 | 需要写代码 | 可视化 |
| **为 Agent** | ✅ 原生 | ⚠️ 需适配 | ❌ 库 | ❌ 人类 |
| **运行时模型** | 进程隔离 | 协议 | 进程内 | 服务器 |
| **契约** | JSON Schema | JSON-RPC | Python 类型 | 表单式 |

---

## 核心架构：四层模型

```
Agent (自然语言意图)
        │
        ▼
┌─────────────────────┐
│      Complex        │  ← Agent 唯一可见的层
│   (领域典籍 Tome)    │     持有 description，参与自动发现
│                     │
│  select_structure() │
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│     Structure        │  ← 业务黑盒
│  (传动机构 Drive)    │     manifest.yaml 定义契约
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif          │  ← 编排逻辑
│  (齿轮组 Assembly)   │     YAML 声明式 或 Graph
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│        Unit          │  ← 原子执行
│      (齿 Cog)        │     stdin/stdout JSON, fork+exec
└─────────────────────┘
```

### 层级总览

| 层级 | 名称 | Agent 可见？ | 本质 |
|------|------|-------------|------|
| **L4** | **Complex** | ✅ 唯一可见 | 领域门面，有 description |
| **L3** | **Structure** | ❌ 不可见 | 业务结构 |
| **L2** | **Motif** | ❌ 不可见 | 编排 Unit |
| **L1** | **Unit** | ❌ 不可见 | 原子执行体 |

### 核心纪律

1. **Unit 之间绝不相互调用**（Runtime 通过 `COGTOME_UNIT_MODE=1` 阻止）
2. **Motif 之间不直接相互调用**（通过 Structure 组合）
3. **Structure 不直接调用 Unit**（必须通过 Motif）
4. **Complex 是唯一有 `description` 的层**
5. **所有跨层调用通过 Runtime IPC**

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

# 运行 Complex（完整领域 Skill）
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# 运行 Structure
./target/release/cogtome structure run text-pipeline --input '{"text":"hello"}'

# 运行 Motif（编排逻辑）
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'

# 直接运行 Unit
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. 环境变量

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
│   ├── discovery.rs          # 目录扫描
│   ├── context/              # 执行上下文
│   │   ├── mod.rs
│   │   ├── expression.rs    # 表达式求值
│   │   └── variables.rs     # 变量解析
│   └── engine/               # 执行引擎
│       ├── mod.rs            # 主引擎
│       ├── graph.rs          # Graph 数据结构
│       ├── motif_manifest.rs # Motif/Structure YAML 解析
│       ├── unit_runner.rs    # Unit 执行器
│       └── foreach.rs        # Foreach 执行器
├── webui/                   # Web UI (React + React Flow)
│   ├── src/
│   │   ├── components/      # React 组件
│   │   │   ├── editors/     # YAML 编辑器
│   │   │   └── graph/       # Graph 节点组件
│   │   ├── store/           # Zustand 状态
│   │   └── api/             # API 客户端
│   └── dist/                # 构建产物
├── skills/                  # Skills 目录
│   ├── units/               # 原子执行体
│   ├── motifs/              # 编排逻辑 (YAML)
│   ├── structures/           # 业务结构
│   └── <complex>/           # 领域 Complex
│       └── SKILL.md         # Complex 定义
├── test_suite/              # 测试用例
├── development/              # 技术文档
└── Cargo.toml
```

---

## CLI 参考

### 执行命令

```bash
# 发现
cogtome discover                              # 扫描所有 Complex
cogtome discover --verbose                   # 显示详细信息

# 运行（Complex → Structure → Motif → Unit）
cogtome run <complex> --input <json>       # 运行 Complex（自动选择第一个 structure）
cogtome structure run <name> --input <json> # 运行 Structure
cogtome motif run <name> --input <json>     # 运行 Motif
cogtome unit run <name> --input <json>      # 直接运行 Unit

# Unit 管理
cogtome unit list                            # 列出所有 Unit
cogtome motif list                           # 列出所有 Motif
cogtome structure list                       # 列出所有 Structure

# HTTP API 服务器
cogtome serve --port 8080                   # 启动 REST API 服务器

# 打包与安装
cogtome pack <skill>                        # 打包 skill 为 .cogtome
cogtome install <file.cogtome>            # 安装 skill 包

# 工具
cogtome validate                             # 校验所有 skills
cogtome reload                              # 热重载 skills
cogtome help                                 # 显示所有命令
```

---

## Web UI

COGTOME 包含一个基于 **React Flow** 的**可视化图形编辑器**，用于编辑 Motifs 和 Structures。

### 截图

| 编辑器 | 说明 |
|--------|------|
| **Structure Editor** | 可视化图形编辑器，组装 Motifs |
| **Motif Editor** | 图形画布，编排执行流程 |
| **Unit Editor** | Unit 测试面板 |

### 启动 Web UI

```bash
# 开发模式
cd webui && npm install && npm run dev

# 或使用启动脚本
./start-webui.sh

# 访问 http://localhost:5173
```

### 功能特性

- **Graph ↔ YAML 同步**：可视化编辑，自动序列化 YAML
- **自动布局**：Dagre 算法自动定位节点
- **键盘快捷键**：Ctrl+S 保存，Ctrl+Z 撤销，Delete 删除
- **AI 助手**：聊天辅助编辑（组件中）
- **明暗主题**：UI 中切换

---

## 内置 Skills

| Complex | Structures | 说明 |
|---------|-----------|------|
| `core-tools` | `shell-executor`, `file-read`, `file-write` | OpenClaw 工具封装 |
| `web-fetch` | `fetch` | HTTP 内容获取 |
| `browser-fetch` | `simple-fetch` | JS 渲染页面获取 (LightPanda) |
| `text-processing` | `text-pipeline` | 文本转换 |

### 运行内置 Skills

```bash
# Shell 执行
cogtome run core-tools --input '{"command": "ls -la"}'

# 文件操作
cogtome structure run --input '{"path": "/tmp/test.txt", "content": "hello"}' file-write

# 网页抓取
cogtome run browser-fetch --input '{"url": "https://example.com"}'
```

---

## 路线图

### Phase 1: 基础 ✅（当前）

- [x] CLI 框架
- [x] Unit 执行（fork+exec, stdin/stdout JSON）
- [x] YAML Motif 解析（串行流程）
- [x] Structure → Motif → Unit 链路
- [x] Complex 发现（SKILL.md 解析）
- [x] 默认超时（30秒）
- [x] Skills 路径配置（`COGTOME_SKILLS_DIR`）
- [x] HTTP API 服务器（`cogtome serve`）
- [x] Web UI + React Flow

### Phase 2: 核心编排 🔄

- [x] `foreach` 循环 + `aggregate` 聚合（解析）
- [ ] 表达式引擎增强
- [ ] `if` 条件执行（可视化）
- [ ] `max_iterations` 安全限制
- [ ] 错误分层（`runtime` / `motif` / `unit`）
- [ ] 快照语义（只读外部状态）

### Phase 3: 并发 🔮

- [ ] 并行 `foreach`（`parallel: true`）
- [ ] Unit 并发声明（`max_global`, `resource_key`）
- [ ] Runtime 资源限制器

### Phase 4: 生态 🔮

- [x] Python Motif（Unix Socket IPC）
- [ ] Discovery API（`GET /complexes`）
- [ ] `auto-complex` 快捷注册
- [x] `cogtome pack/install`

---

## 设计原则

1. **Runtime 零业务逻辑** — COGTOME 二进制不内置任何 Unit
2. **Agent 创作自由** — Unit 可用任意语言，Motifs 用 YAML/Python/Shell
3. **强契约** — 每层 JSON Schema 校验
4. **进程隔离** — Unit 之间绝不相互调用
5. **可观测性** — 完整执行链路日志
6. **可视化 + 文本化** — 同时支持图形编辑器和 YAML 编写

---

## 相关链接

- [技术规格](./development/TECHNICAL_SPEC.md) — 详细架构
- [操作系统隐喻](./development/OS_METAPHORS.md) — 概念基础
- [OpenClaw 集成](./development/OPENCLAW_INTEGRATION.md) — 集成协议

---

## 许可证

MIT
