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
3. [Skills 自我改进闭环](#skills-自我改进闭环)
4. [架构](#架构)
5. [快速开始](#快速开始)
6. [项目结构](#项目结构)
7. [CLI 参考](#cli-参考)
8. [Web UI](#web-ui)
9. [路线图](#路线图)
10. [设计原则](#设计原则)

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

### Skills 自我改进闭环

核心洞察：**可观测的执行过程暴露 Skill 的不足**。

```
执行 Skill
    ↓
观察 DAG 追踪（哪个节点失败、输入输出、超时）
    ↓
识别 Skill 缺陷（缺少错误处理、参数边界问题）
    ↓
Agent 改进 Skill 定义
    ↓
重新执行验证
```

这创造了一个反馈循环：Agent 不仅使用工具，还学会构建更好的工具。

---

## 差异化能力

| 能力 | COGTOME 的处理方式 |
|-----------|------------------------|
| **现有脚本兼容** | 任何能读写 JSON stdin/stdout 的可执行文件。零重写。 |
| **执行保证** | DAG 执行 + 状态传播 + 错误处理。 |
| **可观测性** | 完整执行历史：输入、输出、耗时、失败。 |
| **自我改进** | 执行失败暴露 Skill 弱点，Agent 可修复。 |
| **MCP 生态** | 桥接层将 MCP Server 作为一等 Unit 运行（[见路线图](#路线图)）。 |

---

## 核心特性

**🔒 进程隔离** — 每次工具执行都是独立的 OS 进程，具备超时控制、临时目录沙箱和可选的环境变量白名单。有缺陷的 Unit 不会拖垮运行时或其他 Unit。

**🛠 零重写工具适配** — 你的 Python 脚本、Bash 单行命令或编译后的二进制文件，只要从 stdin 读取 JSON、向 stdout 输出 JSON，就能成为 Unit。无需 SDK，无需协议适配器。

**📐 JSON Schema 契约** — 用 JSON Schema 定义输入输出。运行时在执行前校验输入，在返回后校验输出。

**🧩 声明式工作流** — 用 YAML 将 Unit 串联成 Motif：顺序执行、`if` 分支、`foreach` 循环、并行执行和结果聚合。

**🎨 低代码 Skill 创建** — Web UI 提供拖拽式图形编辑器，可视化编排 Motif 和组装 Skill。非开发者也能不写 YAML 就构建可复用 Skill。

**🎯 语义化 CLI** — Agent 通过人类可理解的命令（`read file`、`fetch webpage`）交互，而非原始 shell 命令（`cat /path`、`curl url`）。

**🌉 MCP 桥接（计划中）** — 无需重写即可在 COGTOME 内运行现有 MCP Server，解决生态冷启动问题。

---

## 架构

COGTOME 采用三层执行模型：

```
Agent (自然语言意图)
        │
        ▼
┌─────────────────────┐
│       Skill         │  ← Agent 可见层。有名称、描述、输入输出 Schema。
│     (业务单元)       │     内部是一个 Motif 或直接引用 Unit。
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif         │  ← 编排逻辑。YAML 声明式流程。
│      (工作流)        │     步骤引用 Unit。支持 foreach、if、retry、on_error。
└─────────┬───────────┘
          │ IPC (fork+exec, stdin/stdout JSON)
          ▼
┌─────────────────────┐
│        Unit         │  ← 原子执行。独立进程。
│     (可执行文件)      │     任意语言。从 stdin 读取 JSON，向 stdout 输出 JSON。
└─────────────────────┘
```

### 层级概览

| 层级 | 作用 | Agent 可见？ |
|-------|---------|---------------|
| **Skill** | 对外暴露的能力，含描述和 Schema | ✅ 是 |
| **Motif** | 将 Unit 编排为可复用工作流 | ❌ 否 |
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
# 发现所有 Skill
./target/release/cogtome discover

# 运行 Skill
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# 直接运行 Motif
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'

# 直接运行 Unit
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 3. 封装自己的脚本（计划中）

```bash
# 一键封装（即将推出）
cogtome wrap ./my_script.py --name my-analyzer
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
│   ├── context/            # 执行上下文
│   │   ├── mod.rs
│   │   ├── expression.rs   # 表达式求值
│   │   └── variables.rs    # 变量解析
│   └── engine/             # 执行引擎
│       ├── mod.rs          # MotifEngine + StructureExecutor
│       ├── motif_manifest.rs # 类型定义
│       ├── unit_runner.rs  # Unit 执行器 (fork+exec)
│       └── foreach.rs      # Foreach 执行器
├── webui/                  # Web UI (React + React Flow + TypeScript)
│   ├── src/
│   │   ├── components/     # React 组件
│   │   ├── store/          # Zustand 状态
│   │   └── api/            # API 客户端
│   └── dist/               # 构建产物
├── skills/                 # Skills 目录（运行时加载）
│   ├── units/<name>/bin/   # 原子可执行文件
│   ├── motifs/<name>.yaml  # YAML 工作流 Motif
│   ├── structures/<name>/  # 业务结构（将合并入 Skill）
│   └── <complex>/          # Complex 定义（将合并入 Skill）
│       └── SKILL.md
├── Cargo.toml
└── cogtome.toml            # 运行时配置
```

---

## CLI 参考

### 执行命令

```bash
# 发现
cogtome discover                              # 扫描所有 Skills

# 运行（Skill → Motif → Unit）
cogtome run <skill> --input <json>            # 运行 Skill
cogtome motif run <name> --input <json>       # 运行 Motif
cogtome unit run <name> --input <json>        # 运行 Unit

# HTTP API 服务器
cogtome serve --port 8080                     # 启动 REST API

# 打包与安装
cogtome pack <skill>                          # 打包为 .cogtome
cogtome install <file.cogtome>                # 安装包

# 工具
cogtome validate                              # 校验所有 skills
cogtome reload                                # 热重载 skills
cogtome help                                  # 显示所有命令
```

---

## Web UI

COGTOME 包含一个 **Skill 可视化工作室**，同时支持创作和调试 Motif。

### Skill 创作

- **图形编辑器**：拖拽式编排 Motif，支持 9 种节点类型（start、unit、if、match、foreach、fork、join、return、motif）
- **Graph ↔ YAML 同步**：可视化编辑，自动序列化为 YAML
- **自动布局**：基于网格的自动节点定位

### 执行调试器

- **执行链路追踪**：查看每个步骤的数据流（哪个节点卡住、输入输出是什么）
- **Unit 测试面板**：快速用自定义参数运行单个 Unit
- **实时图形视图**：在执行期间或执行后可视化 Motif DAG

### 启动 Web UI

```bash
# 一键启动（构建 Rust + API + WebUI）
./start-webui.sh

# 或手动
cargo build --release
cogtome serve --port 3334 &
cd webui && npm install && npm run dev
```

访问 **http://localhost:3333**

---

## 对比

| 特性 | COGTOME | MCP | LangChain | Dify/n8n |
|---------|---------|-----|-----------|----------|
| **主要目标** | 安全运行现有脚本 | 协议标准 | Python 框架 | 人类工作流 |
| **是否需要重写工具** | ❌ 否 | ✅ 是（MCP Server） | ⚠️ Python 包装器 | ⚠️ 通常需要 |
| **进程隔离** | ✅ 是 | 取决于宿主 | ❌ 进程内 | ✅ 服务端 |
| **Agent 原生接口** | ✅ CLI | 协议 | Python API | GUI/API |
| **最适合** | 本地脚本沙箱 | 跨平台工具 | Python 应用集成 | 业务自动化 |

---

## 路线图

### 第一阶段：稳定（当前）

- [x] CLI 框架：discover、run、unit/motif/skill run
- [x] Unit 执行：fork+exec、stdin/stdout JSON、超时、临时沙箱
- [x] YAML Motif 解析与执行
- [x] Skill 发现（SKILL.md front-matter 解析）
- [x] `foreach` 循环与聚合
- [x] `if` 条件执行
- [x] 重试与退避策略
- [x] 错误策略（fail、continue、fallback）
- [x] HTTP API 服务器
- [x] 打包/安装（tar.gz）

### 第二阶段：MCP 兼容与易用性（0–6 周）

- [ ] **MCP Bridge Unit** — 通过 stdio JSON-RPC 将 MCP Server 作为 COGTOME Unit 运行
- [ ] **Skill 层合并** — 将 Structure + Complex 合并为单一的 Skill 概念
- [ ] **内联脚本节点** — 在 Motif 中直接运行 Python/Bash 片段，无需独立 Unit
- [ ] **`cogtome wrap`** — 从现有脚本一键迁移
- [ ] **Docker Unit Runner** — 为不可信工具提供可选的容器化执行

### 第三阶段：可观测性与集成（6–12 周）

- [ ] 执行链路日志（每次运行的完整输入/输出/历史）
- [ ] 长时运行 Motif 的 checkpoint/断点续跑
- [ ] Prometheus 指标导出
- [ ] KimiCLI 桥接（Wire/ACP 长连接模式）
- [ ] OpenClaw 网关桥接（WebSocket）

### 第四阶段：生态

- [ ] 文件系统自动重载（notify crate）
- [ ] Skill 注册中心 / 市场
- [ ] Web UI 执行调试器（trace 视图）

---

## 设计原则

1. **不让用户先学隐喻** — 东西就叫它本身的名字：Unit、Workflow、Skill。
2. **零重写接入** — 你现有的脚本就是资产。保留它们。
3. **默认隔离** — 每个工具在独立进程中运行。没有例外。
4. **Schema 契约** — 每个边界都有 JSON Schema 校验。
5. **MCP 兼容** — 我们不与 MCP 竞争；我们运行它。
6. **可视化 + 文本化** — 同时支持图形编辑器和 YAML 编写。可调试性和创作体验同等重要。

---

## 相关链接

- [技术规格](./development/TECHNICAL_SPEC.md)
- [实现指南](./development/IMPLEMENTATION_GUIDE.md)
- [Skill 编写指南](./development/SKILL_AUTHORING_GUIDE.md)

---

## 许可证

MIT
