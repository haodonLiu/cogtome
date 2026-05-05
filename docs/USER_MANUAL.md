# COGTOME 使用手册

> **COGTOME** — AI Agent 的执行层运行时，为 Agent 提供经过测试、可复用的执行剧本。

---

## 目录

1. [产品简介](#1-产品简介)
2. [安装与构建](#2-安装与构建)
3. [核心概念](#3-核心概念)
4. [快速开始](#4-快速开始)
5. [CLI 命令详解](#5-cli-命令详解)
6. [Unit 编写指南](#6-unit-编写指南)
7. [Motif 编写指南](#7-motif-编写指南)
8. [Structure 编写指南](#8-structure-skillsmd-编写指南)
9. [配置文件](#9-配置文件)
10. [HTTP API 参考](#10-http-api-参考)
11. [技能打包与安装](#11-技能打包与安装)
12. [常见问题](#12-常见问题)
13. [开发流程](#附录-c开发流程)
14. [系统依赖](#附录-d系统依赖)

---

## 1. 产品简介

### 1.1 什么是 COGTOME

COGTOME 是一个**运行时引擎**，以声明式工作流的形式执行 Agent 工具。它提供：

- **进程隔离**：每个工具在独立进程中执行，互不影响
- **DAG 编排**：通过有向无环图组织执行流程
- **状态传播**：步骤之间的数据自动传递
- **可观测性**：完整的执行追踪和错误处理

### 1.2 解决的问题

Agent 知道「做什么」，但经常在「怎么做」上出错：

- 工具调用顺序错误
- 参数传递不正确
- 错误处理不完整
- 多步操作状态丢失

### 1.3 核心特性

| 特性 | 说明 |
|------|------|
| 进程隔离 | 每个 Unit 在独立 OS 进程中执行 |
| 零重写适配 | 任何能读写 JSON 的可执行文件都可成为 Unit |
| JSON Schema 契约 | 输入输出自动校验 |
| 声明式工作流 | 支持条件分支、循环、并行执行 |

---

## 2. 安装与构建

### 2.1 环境要求

- Rust 1.70+
- npm 或 yarn（Web UI 开发需要）

### 2.2 构建步骤

```bash
# 克隆项目
git clone https://github.com/haodonLiu/cogtome.git
cd cogtome

# Debug 构建
cargo build

# Release 构建（生产环境推荐）
cargo build --release

# 运行测试
cargo test
```

### 2.3 目录结构

```
cogtome/
├── src/                    # Rust 运行时源码
├── skills/                 # 技能目录（运行时加载）
├── assemblies/             # MCP Assembly 目录
├── cogtome.toml            # 运行时配置
└── target/release/cogtome  # 编译产物
```

---

## 3. 核心概念

### 3.1 三层执行模型

COGTOME 采用三层执行模型：

```
Agent（自然语言意图）
        │
        ▼
┌─────────────────────┐
│     Structure       │  ← 顶层，chains Motifs（skills/ 或 assemblies/）
│                     │     Name, description, input/output schema
│    （SKILL.md）      │     通过 YAML front matter 定义
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif         │  ← 编排逻辑，DAG 图
│   （工作流编排）     │     定义节点和边，支持条件/循环
└─────────┬───────────┘
          │ IPC (fork+exec, stdin/stdout JSON)
          ▼
┌─────────────────────┐
│        Unit        │  ← 原子执行体
│     （可执行文件）   │     独立进程，任意语言
└─────────────────────┘
```

### 3.2 各层说明

| 层级 | 可见性 | 作用 | 定义文件 |
|------|--------|------|----------|
| **Structure** | ✅ Agent | 顶层封装，有描述 | `<name>/SKILL.md` |
| **Motif** | ❌ | DAG 编排逻辑 | `motifs/<name>.json` |
| **Unit** | ❌ | 原子执行 | `units/<name>/bin/<name>` |

### 3.3 核心纪律

1. **Unit 之间绝不直接调用** — 通过 `COGTOME_UNIT_MODE=1` 环境变量阻止
2. **跨层调用必须经过 Runtime IPC**
3. **每个边界都有 Schema 校验**

---

## 4. 快速开始

### 4.1 发现技能

```bash
./target/release/cogtome discover
```

### 4.2 运行示例

```bash
# 运行 Structure（顶层）
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# 运行 Motif
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'

# 运行 Unit（最底层）
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 4.3 启动 HTTP API

```bash
./target/release/cogtome serve --port 8080

# 测试接口
curl http://localhost:8080/health
curl http://localhost:8080/structures
```

---

## 5. CLI 命令详解

### 5.1 cogtome unit

管理原子执行体（Unit）。

```bash
# 运行 Unit
cogtome unit run <unit-name> --input '<json>'
```

**示例：**

```bash
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
# 输出: {"result":"HELLO"}
```

### 5.2 cogtome motif

管理编排逻辑（Motif）。

```bash
# 运行 Motif
cogtome motif run <motif-name> --input '<json>'
```

**示例：**

```bash
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'
```

### 5.3 cogtome run

运行 Structure（领域技能）。

```bash
cogtome run <structure-name> --input '<json>'
```

**示例：**

```bash
./target/release/cogtome run text-processing --input '{"text":"hello"}'
```

### 5.4 cogtome discover

扫描并发现所有 Structure。

```bash
cogtome discover
```

### 5.5 cogtome serve

启动 HTTP API 服务器。

```bash
cogtome serve --port <port>
```

**示例：**

```bash
./target/release/cogtome serve --port 8080
```

### 5.6 cogtome pack

打包技能到 `.cogtome` 归档文件。

```bash
cogtome pack <skill-name>
```

**示例：**

```bash
./target/release/cogtome pack text-processing
# 输出: text-processing.cogtome
```

### 5.7 cogtome install

安装 `.cogtome` 归档文件。

```bash
cogtome install <file.cogtome>
```

**示例：**

```bash
./target/release/cogtome install ./text-processing.cogtome
```

### 5.8 cogtome reload

热重载：重新加载所有 Structure 和 Motif 定义。

```bash
cogtome reload
```

### 5.9 cogtome validate

验证 Motif 或 Structure manifest 文件。

```bash
cogtome validate <path-to-manifest>
```

**示例：**

```bash
./target/release/cogtome validate ./skills/text-processing/SKILL.md
```

### 5.10 cogtome mcp-bridge

通过 MCP Bridge 运行 MCP Server 工具。

```bash
cogtome mcp-bridge --server "<server-command>" --tool <tool-name>
```

**示例：**

```bash
./target/release/cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
```

### 5.11 cogtome mcp-server

启动 MCP Server（stdio JSON-RPC 模式）。

```bash
cogtome mcp-server --assemblies <dir> --units <dir>
```

---

## 6. Unit 编写指南

### 6.1 什么是 Unit

Unit 是**原子执行体**，是 COGTOME 最底层的执行单位。任何可执行文件，只要支持 JSON stdin/stdout 协议，就可以成为 Unit。

### 6.2 协议规范

- **输入**：JSON 对象，从 stdin 读取
- **输出**：第一行 stdout 必须是有效 JSON（结果）
- **退出码**：
  - `0` — 成功
  - `1` — 输入错误（不重试）
  - `2` — 可重试错误
  - `3` — 依赖不可用

### 6.3 目录结构

```
skills/
└── units/
    └── <unit-name>/
        └── bin/
            └── <unit-name>    # 可执行文件
```

**关键规则**：Unit 名称必须与目录名和二进制文件名一致。

### 6.4 Python 示例

```python
#!/usr/bin/env python3
import sys, json

try:
    inp = json.load(sys.stdin)
    text = inp.get("text")
    if text is None:
        print(json.dumps({"error": "missing field: text"}), file=sys.stderr)
        sys.exit(1)
    print(json.dumps({"result": text.upper()}))
except Exception as e:
    print(json.dumps({"error": str(e)}), file=sys.stderr)
    sys.exit(1)
```

### 6.5 Bash 示例

```bash
#!/bin/bash
read -r input
TEXT=$(echo "$input" | jq -r '.text')
echo "{\"result\": \"${TEXT}\"}"
```

### 6.6 环境变量

运行时会自动设置：

| 变量 | 说明 |
|------|------|
| `COGTOME_UNIT_MODE=1` | 标记当前在 Unit 模式运行 |
| `COGTOME_EXEC_DIR` | 临时执行目录 |

---

## 7. Motif 编写指南

### 7.1 什么是 Motif

Motif 是**编排逻辑**，定义为一个 DAG（有向无环图），包含节点（nodes）和边（edges）。

### 7.2 文件格式

Motif 使用 **JSON** 格式存储，文件名必须与 `name` 字段一致。

**文件位置**：`skills/motifs/<name>.json`

### 7.3 节点类型

| 类型 | 说明 |
|------|------|
| `start` | 起始节点（必须有且仅有一个） |
| `return` | 结束节点，返回结果 |
| `unit` | 调用 Unit |
| `if` | 条件分支 |
| `match` | 多条件匹配 |
| `foreach` | 循环迭代 |
| `fork` | 并行执行 |
| `join` | 等待并行任务完成 |
| `motifRef` | 引用其他 Motif |

### 7.4 完整示例

```json
{
  "name": "browser-fetch",
  "type": "motif",
  "version": "2.0",
  "required_units": ["camoufox-fetch"],
  "graph": {
    "nodes": [
      {
        "id": "start",
        "type": "start",
        "position": { "x": 0, "y": 100 }
      },
      {
        "id": "fetch",
        "type": "unit",
        "unit": "camoufox-fetch",
        "input": {
          "url": "${params.url}"
        },
        "position": { "x": 200, "y": 100 }
      },
      {
        "id": "done",
        "type": "return",
        "values": {
          "url": "${steps.fetch.output.url}",
          "content": "${steps.fetch.output.content}"
        },
        "position": { "x": 400, "y": 100 }
      }
    ],
    "edges": [
      { "source": "start", "target": "fetch" },
      { "source": "fetch", "target": "done" }
    ]
  }
}
```

### 7.5 if 条件节点

```json
{
  "id": "check",
  "type": "if",
  "condition": "${params.text} != \"\"",
  "position": { "x": 200, "y": 100 }
}
```

**边标签**：`true` 或 `false`

```json
{ "source": "check", "target": "process", "label": "true" },
{ "source": "check", "target": "skip", "label": "false" }
```

### 7.6 foreach 循环节点

```json
{
  "id": "process-items",
  "type": "foreach",
  "over": "${params.items}",
  "as_var": "item",
  "max_iterations": 50,
  "error_strategy": "fail",
  "position": { "x": 200, "y": 100 }
}
```

### 7.7 变量语法

| 语法 | 说明 |
|------|------|
| `${params.x}` | 用户输入参数 |
| `${steps.<step>.output.field}` | 前置步骤的输出 |
| `${env.VAR}` | 环境变量 |
| `${arr[0]}` | 数组索引（0-based） |
| `${arr[-1]}` | 负数索引（从末尾） |
| `${arr.length}` | 数组长度 |

### 7.8 表达式函数

- `filter(arr, 'field == "value"')` — 过滤数组
- `map(arr, 'field')` — 提取字段

---

## 8. Structure（SKILL.md）编写指南

### 8.1 什么是 Structure

Structure 是**顶层封装**，是 Agent 直接交互的层。通过 `SKILL.md` 定义，包含 YAML front matter 和 Markdown 文档。

### 8.2 目录结构

```
skills/
└── <structure-name>/       # 目录名必须与 name 一致
    └── SKILL.md            # Structure 定义文件
```

### 8.3 格式

```markdown
---
name: text-processing
description: |
  文本处理领域。当任务涉及文本转换、格式化、大小写转换时使用。

motifs:
  - name: text-transform
    summary: "文本转换流水线"

units:
  - text-uppercase
  - text-lowercase

config:
  default_timeout: 10
---

# Text Processing

此处是 Markdown 文档，可以详细描述使用方式。
```

### 8.4 front matter 字段

| 字段 | 说明 |
|------|------|
| `name` | Structure 名称 |
| `description` | 描述（Agent 决策用） |
| `motifs` | 引用的 Motif 列表 |
| `units` | 依赖的 Unit 列表 |
| `config` | 配置（超时、日志等） |

### 8.5 motifs 字段

```yaml
motifs:
  - name: text-transform           # Motif 名称
    summary: "简要描述"
```

---

## 9. 配置文件

### 9.1 配置文件位置

按以下顺序查找：

1. `./cogtome.toml`
2. `$XDG_CONFIG_HOME/cogtome.toml`
3. `$HOME/.config/cogtome.toml`

### 9.2 配置项

```toml
[runtime]
max_iterations = 50           # foreach 最大迭代次数
max_iterations_hard = 500    # 硬性上限（不可覆盖）

[paths]
units = "units"                # Unit 目录（相对于 skills 根目录）
motifs = "motifs"
structures = "structures"
assemblies = "assemblies"      # MCP Server 用

[units.defaults]
timeout_secs = 30              # 默认超时（秒）

[units.concurrency.some-unit]
max_global = 3                 # 全局最大并发
resource_key = "shared_key"    # 资源键
```

### 9.3 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `COGTOME_SKILLS_DIR` | 技能根目录 | `./skills` |
| `COGTOME_TIMEOUT` | Unit 超时（秒） | `30` |
| `COGTOME_MAX_CONCURRENT` | foreach 最大并发 | `50` |
| `RUST_LOG` | 日志级别 | - |

---

## 10. HTTP API 参考

### 10.1 启动服务器

```bash
cogtome serve --port 8080
```

### 10.2 端点列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 健康检查 |
| GET | `/metrics` | 指标数据 |
| GET | `/structures` | 列出所有 Structure |
| POST | `/run` | 执行 Structure/Motif/Unit |

### 10.3 /run 请求格式

```json
{"type": "structure", "name": "text-processing", "input": {"text": "hello"}}
{"type": "motif", "name": "text-transform", "input": {"text": "hello"}}
{"type": "unit", "name": "text-uppercase", "input": {"text": "hello"}}
```

### 10.4 示例

```bash
# 健康检查
curl http://localhost:8080/health

# 列出 Structure
curl http://localhost:8080/structures

# 执行 Structure
curl -X POST http://localhost:8080/run \
  -H "Content-Type: application/json" \
  -d '{"type":"structure","name":"text-processing","input":{"text":"hello"}}'
```

---

## 11. 技能打包与安装

### 11.1 打包技能

```bash
cogtome pack <skill-name>
```

**示例**：

```bash
./target/release/cogtome pack text-processing
# 生成：text-processing.cogtome
```

### 11.2 安装技能

```bash
cogtome install <file.cogtome>
```

**示例**：

```bash
./target/release/cogtome install ./text-processing.cogtome
```

### 11.3 打包格式

`.cogtome` 文件是 `tar.gz` 归档，包含：

- `manifest.json` — 元数据
- `motifs/` — Motif 定义
- `units/` — Unit 二进制
- `SKILL.md` — Structure 定义

---

## 12. 常见问题

### 12.1 "Structure 'xxx' not found"

**原因**：SKILL.md 中的 `name` 与目录名不匹配。

**解决**：确保目录名与 SKILL.md front matter 中的 `name` 字段一致。

### 12.2 "Motif 'xxx' not found"

**原因**：文件名与 manifest 中的 `name` 不匹配。

```
motifs/fetch-web.json + name: fetch-web  ← 错误
motifs/fetch-web.json + name: fetch-web # 正确（必须与文件名一致）
```

**解决**：文件名（不含扩展名）必须与 `name` 字段一致。

### 12.3 "missing field `type`"

**解决**：在 manifest 中添加 `type: motif`。

### 12.4 Unit 执行超时

**解决**：增加超时时间

```bash
export COGTOME_TIMEOUT=60
```

或在 `cogtome.toml` 中配置：

```toml
[units.defaults]
timeout_secs = 60
```

### 12.5 变量解析失败

**语法检查**：

| 语法 | 说明 |
|------|------|
| `${params.x}` | 用户输入参数 |
| `${steps.<name>.output.field}` | 步骤输出 |
| `${env.VAR}` | 环境变量 |

---

## 附录 A：错误码

| 退出码 | 含义 | 说明 |
|--------|------|------|
| 0 | 成功 | 正常执行完成 |
| 1 | 输入错误 | 参数格式错误，不重试 |
| 2 | 可重试错误 | 临时故障，可重试 |
| 3 | 依赖不可用 | 外部依赖缺失 |

---

## 附录 B：Skills 目录布局

```
skills/                              # Structure type 1: skills directory
├── units/<name>/bin/<name>         # Executable Unit
├── motifs/<name>.json               # Filename MUST match `name` field
└── <name>/SKILL.md                  # Structure manifest with YAML front matter

assemblies/                          # Structure type 2: MCP assemblies
└── <name>/
    ├── manifest.json                # Assembly manifest
    └── workflow.json                # MotifManifestV2 DAG
```

**命名规则（违规导致 "not found" 错误）：**
- Unit: `units/<name>/bin/<name>` (must be executable)
- Motif: `motifs/<name>.json` where `<name>` matches the file
- Structure: `<name>/SKILL.md` where `<name>` matches the directory

---

## 附录 C：开发流程

本文档描述如何使用 COGTOME 构建一个完整的自动化工作流。

### C.1 三层职责划分

COGTOME 采用三层分离设计，各层职责明确：

| 层级 | 职责 | 执行者 | 产出物 |
|------|------|--------|--------|
| **Agent（规划层）** | 理解需求、拆解任务、判断流程 | Kimi / Claude Code | atomic steps 列表 |
| **COGTOME（执行层）** | 按 DAG 执行 Unit、维护状态、错误恢复 | Runtime Engine | 结构化执行结果 |
| **Unit（原子层）** | 执行单一操作（读文件、执行命令等） | 独立进程 | JSON 输出 |

### C.2 开发流程总览

```
┌──────────────────────────────────────────────────────────────┐
│  阶段 1：需求分析                                            │
│  · 确定输入输出                                               │
│  · 明确工作流的触发条件和结束条件                              │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────┐
│  阶段 2：Structure 设计（顶层）                               │
│  · 定义输入/输出 JSON Schema                                 │
│  · 命名 Structure，撰写 description（Agent 决策用）           │
│  · 确定需要哪些 Motif                                         │
│  · 位置：skills/<name>/SKILL.md                              │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────┐
│  阶段 3：Motif 设计（DAG 编排）                               │
│  · 画出 DAG 图：节点 = Unit，边 = 数据依赖                   │
│  · 确定节点类型：unit / if / match / foreach / fork / join   │
│  · 定义变量传递：${params.x}、${steps.<id>.output.field}   │
│  · 位置：skills/motifs/<name>.json                            │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────┐
│  阶段 4：Unit 实现（原子执行）                                │
│  · 每个 Unit 只做一件事                                       │
│  · 输入：JSON stdin，输出：JSON stdout                       │
│  · 退出码：0=成功，1=输入错误，2=可重试，3=依赖不可用         │
│  · 位置：skills/units/<name>/bin/<name>                      │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────┐
│  阶段 5：组装测试                                            │
│  · cogtome discover（验证结构）                              │
│  · cogtome validate <SKILL.md>（校验 manifest）             │
│  · cogtome run <name> --input '{}'（端到端测试）             │
│  · 检查 stderr 事件流输出                                    │
└────────────────────────┬─────────────────────────────────────┘
                         ↓
┌──────────────────────────────────────────────────────────────┐
│  阶段 6：打包发布                                            │
│  · cogtome pack <name> → <name>.cogtome                     │
│  · 分发归档文件或提交到 skills/ 目录                          │
└──────────────────────────────────────────────────────────────┘
```

### C.3 实战示例：代码编写 + 分析优化 + PR 提交

以下展示如何用 COGTOME 实现一个完整的代码工作流。

#### 阶段 1：需求分析

```
目标：将一个代码任务拆解为可执行步骤，并最终提交 PR
输入：{ repo_url, task, target_branch }
输出：{ pr_url, summary }
```

#### 阶段 2：Structure 设计

创建 `skills/code-workflow/SKILL.md`：

```yaml
---
name: code-workflow
description: |
  代码编写-分析优化-PR提交流水线。当需要完成代码任务、
  进行静态分析、获得优化建议、并提交PR时使用此流程。

input_schema:
  type: object
  required: [repo_url, task, target_branch]
  properties:
    repo_url: { type: string }
    task: { type: string }
    target_branch: { type: string }

output_schema:
  type: object
  properties:
    pr_url: { type: string }
    summary: { type: string }

motifs:
  - name: code-workflow-dag
    summary: "代码编写到PR提交的完整流水线"

units:
  - git-clone
  - code-write
  - static-analyze
  - git-commit
  - git-pr-create

config:
  default_timeout: 120
---
```

#### 阶段 3：Motif 设计

创建 `skills/motifs/code-workflow-dag.json`：

```json
{
  "name": "code-workflow-dag",
  "type": "motif",
  "version": "2.0",
  "required_units": ["git-clone", "code-write", "static-analyze", "git-commit", "git-pr-create"],
  "graph": {
    "nodes": [
      { "id": "start", "type": "start", "position": { "x": 0, "y": 100 } },
      {
        "id": "clone",
        "type": "unit",
        "unit": "git-clone",
        "input": { "repo_url": "${params.repo_url}", "branch": "${params.target_branch}" },
        "position": { "x": 200, "y": 100 }
      },
      {
        "id": "write",
        "type": "unit",
        "unit": "code-write",
        "input": { "task": "${params.task}", "repo_path": "${steps.clone.output.repo_path}" },
        "position": { "x": 400, "y": 100 }
      },
      {
        "id": "analyze",
        "type": "unit",
        "unit": "static-analyze",
        "input": { "repo_path": "${steps.clone.output.repo_path}", "changed_files": "${steps.write.output.files}" },
        "position": { "x": 600, "y": 100 }
      },
      {
        "id": "route_analysis",
        "type": "match",
        "on": "${steps.analyze.output.has_issues}",
        "position": { "x": 800, "y": 100 }
      },
      {
        "id": "commit",
        "type": "unit",
        "unit": "git-commit",
        "input": { "repo_path": "${steps.clone.output.repo_path}", "message": "${steps.write.output.commit_message}" },
        "position": { "x": 1000, "y": 50 }
      },
      {
        "id": "create_pr",
        "type": "unit",
        "unit": "git-pr-create",
        "input": { "repo_url": "${params.repo_url}", "branch": "${params.target_branch}" },
        "position": { "x": 1200, "y": 50 }
      },
      {
        "id": "return_success",
        "type": "return",
        "values": {
          "pr_url": "${steps.create_pr.output.pr_url}",
          "summary": "${steps.analyze.output.summary}"
        },
        "position": { "x": 1400, "y": 50 }
      }
    ],
    "edges": [
      { "source": "start", "target": "clone" },
      { "source": "clone", "target": "write" },
      { "source": "write", "target": "analyze" },
      { "source": "analyze", "target": "route_analysis" },
      { "source": "route_analysis", "target": "commit", "label": "false" },
      { "source": "route_analysis", "target": "commit", "label": "true" },
      { "source": "commit", "target": "create_pr" },
      { "source": "create_pr", "target": "return_success" }
    ]
  }
}
```

**关键设计点：**
- `route_analysis` 节点使用 `match` 根据 `has_issues` 分支（无论 true/false 都继续，示例中是简化的设计）
- 每个节点的输入通过 `${steps.<id>.output.field}` 引用前序节点的输出
- `fork/join` 可用于并行执行多个独立的分析任务

#### 阶段 4：Unit 实现

以下为各 Unit 的最小实现示例：

**git-clone** (`skills/units/git-clone/bin/git-clone`)：
```python
#!/usr/bin/env python3
import sys, json, subprocess

inp = json.load(sys.stdin)
repo_url = inp.get("repo_url")
branch = inp.get("branch", "main")

result = subprocess.run(
    ["git", "clone", "-b", branch, repo_url, "/tmp/cogtome-repo"],
    capture_output=True, text=True
)
print(json.dumps({
    "repo_path": "/tmp/cogtome-repo",
    "branch": branch,
    "success": result.returncode == 0
}))
```

**code-write**：调用外部 AI 工具写代码（封装为 Unit，保持隔离）

**static-analyze**：运行 linter、type checker 等工具

**git-commit / git-pr-create**：封装 git 操作

#### 阶段 5：组装测试

```bash
# 1. 验证结构
./target/release/cogtome discover

# 2. 验证 manifest
./target/release/cogtome validate ./skills/code-workflow/SKILL.md

# 3. 端到端测试（dry run）
./target/release/cogtome run code-workflow --input '{
  "repo_url": "https://github.com/example/repo",
  "task": "添加用户认证模块",
  "target_branch": "develop"
}'
```

#### 阶段 6：与外部 Agent 集成

COGTOME 只负责执行，任务规划由外部 Agent 负责：

```bash
# Kimi/Claude Code 规划后，调用 COGTOME 执行
./target/release/cogtome run code-workflow --input '{
  "repo_url": "...",
  "task": "...",
  "target_branch": "..."
}'

# 或通过 HTTP API
curl -X POST http://localhost:8080/run \
  -H "Content-Type: application/json" \
  -d '{"type":"structure","name":"code-workflow","input":{...}}'
```

### C.4 目录布局参考

完整项目结构：

```
skills/
└── code-workflow/                  # Structure
    └── SKILL.md                    # Structure manifest
    ├── motifs/
    │   └── code-workflow-dag.json # Motif (DAG)
    └── units/
        ├── git-clone/
        │   └── bin/git-clone      # Unit
        ├── code-write/
        │   └── bin/code-write
        ├── static-analyze/
        │   └── bin/static-analyze
        ├── git-commit/
        │   └── bin/git-commit
        └── git-pr-create/
            └── bin/git-pr-create
```

### C.5 常见陷阱

| 陷阱 | 描述 | 解决方法 |
|------|------|---------|
| Unit 过度封装 | 一个 Unit 做太多事情 | 原则：每个 Unit 只做一件事 |
| 跨 Unit 调用 | Unit 之间直接调用 | 通过 Runtime IPC，禁止直接调用 |
| 忽略错误处理 | 没有考虑失败路径 | 在 Motif 中用 `if` 节点处理异常退出码 |
| Schema 不匹配 | Unit 输出格式与下游输入不符 | 每层边界都用 JSON Schema 校验 |
| 变量引用错误 | `${steps.id.output.field}` 拼写错误 | 确保 node `id` 和 `field` 与实际输出一致 |

---

## 附录 D：系统依赖

| 工具 | 用途 | 安装 |
|------|------|------|
| `jq` | JSON 解析 | `apt install jq` / `brew install jq` |

---

*最后更新：2026-05-04*
