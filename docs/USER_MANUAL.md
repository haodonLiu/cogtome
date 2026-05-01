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
8. [Structure 编写指南](#8-structure-编写指南)
9. [Complex 编写指南](#9-complex-skillmd-编写指南)
10. [配置文件](#10-配置文件)
11. [HTTP API 参考](#11-http-api-参考)
12. [Web UI 使用指南](#12-web-ui-使用指南)
13. [技能打包与安装](#13-技能打包与安装)
14. [常见问题](#14-常见问题)

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
| 可视化编辑器 | Web UI 拖拽式图形编辑器 |

---

## 2. 安装与构建

### 2.1 环境要求

- Rust 1.70+
- Node.js 18+（Web UI 开发需要）
- npm 或 yarn

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
├── webui/                  # Web UI (React + TypeScript)
├── skills/                 # 技能目录（运行时加载）
├── cogtome.toml            # 运行时配置
└── target/release/cogtome  # 编译产物
```

---

## 3. 核心概念

### 3.1 四层执行模型

COGTOME 采用四层执行模型：

```
Agent（自然语言意图）
        │
        ▼
┌─────────────────────┐
│      Complex        │  ← 领域 Tome，Agent 可见层
│    （领域编排）      │     定义描述，通过 SKILL.md 组织
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│     Structure       │  ← 业务结构，业务黑盒
│   （业务封装）       │     定义输入输出 Schema，链式调用 Motif
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
| **Complex** | ✅ Agent | 领域能力封装，有描述 | `SKILL.md` |
| **Structure** | ❌ | 业务链式调用 | `manifest.yaml` |
| **Motif** | ❌ | DAG 编排逻辑 | `*.json` |
| **Unit** | ❌ | 原子执行 | `bin/<name>` |

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
# 运行 Complex（顶层）
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# 运行 Structure
./target/release/cogtome structure run text-pipeline --input '{"text":"hello"}'

# 运行 Motif
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'

# 运行 Unit（最底层）
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### 4.3 启动 HTTP API

```bash
./target/release/cogtome serve --port 8080

# 测试接口
curl http://localhost:8080/health
curl http://localhost:8080/complexes
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

### 5.3 cogtome structure

管理业务结构（Structure）。

```bash
# 运行 Structure
cogtome structure run <structure-name> --input '<json>'
```

**示例：**

```bash
./target/release/cogtome structure run shell-executor --input '{"command":"pwd"}'
```

### 5.4 cogtome run

运行 Complex（领域技能）。

```bash
cogtome run <complex-name> --input '<json>'
```

**示例：**

```bash
./target/release/cogtome run ima --input '{"action":"list"}'
```

### 5.5 cogtome discover

扫描并发现所有 Complex。

```bash
cogtome discover
```

### 5.6 cogtome serve

启动 HTTP API 服务器。

```bash
cogtome serve --port <port>
```

**示例：**

```bash
./target/release/cogtome serve --port 8080
```

### 5.7 cogtome pack

打包技能到 `.cogtome` 归档文件。

```bash
cogtome pack <skill-name>
```

**示例：**

```bash
./target/release/cogtome pack text-processing
# 输出: text-processing.cogtome
```

### 5.8 cogtome install

安装 `.cogtome` 归档文件。

```bash
cogtome install <file.cogtome>
```

**示例：**

```bash
./target/release/cogtome install ./text-processing.cogtome
```

### 5.9 cogtome reload

热重载：重新加载所有 Structure 和 Motif 定义。

```bash
cogtome reload
```

### 5.10 cogtome validate

验证 Motif 或 Structure manifest 文件。

```bash
cogtome validate <path-to-manifest>
```

**示例：**

```bash
./target/release/cogtome validate ./skills/structures/shell-executor/manifest.yaml
```

### 5.11 cogtome mcp-bridge

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

### 5.12 cogtome mcp-server

启动 MCP Server（stdio JSON-RPC 模式）。

```bash
cogtome mcp-server --assemblies <dir> --units <dir>
```

### 5.13 cogtome stats

显示 Assembly 调用热力图。

```bash
cogtome stats
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
skills/units/<unit-name>/
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

## 8. Structure 编写指南

### 8.1 什么是 Structure

Structure 是**业务结构**，定义输入输出 Schema，并按顺序链式调用 Motif。

### 8.2 目录结构

```
skills/structures/<structure-name>/
├── manifest.yaml           # 必须
└── SKILL.md               # 可选
```

**关键规则**：目录名必须与 `manifest.yaml` 中的 `name` 字段一致。

### 8.3 manifest.yaml 格式

```yaml
name: shell-executor
type: structure

motifs:
  - name: shell-executor

input_schema:
  type: object
  required: [command]
  properties:
    command:
      type: string
      description: Shell command to execute
    timeout:
      type: number
      description: Timeout in seconds (optional)

output_schema:
  type: object
  properties:
    stdout:
      type: string
    stderr:
      type: string
    exit_code:
      type: number

resources:
  units:
    - shell-run
```

### 8.4 字段说明

| 字段 | 必填 | 说明 |
|------|------|------|
| `name` | ✅ | 结构名称，必须与目录名一致 |
| `type` | ✅ | 固定值 `structure` |
| `input_schema` | ✅ | 输入 JSON Schema |
| `output_schema` | ✅ | 输出 JSON Schema |
| `resources.units` | ✅ | 依赖的 Unit 列表 |

---

## 9. Complex（SKILL.md）编写指南

### 9.1 什么是 Complex

Complex 是**领域技能**，是 Agent 直接交互的层。通过 `SKILL.md` 定义，包含 YAML front matter 和 Markdown 文档。

### 9.2 文件位置

```
skills/<complex-name>/
└── SKILL.md
```

### 9.3 格式

```markdown
---
name: text-processing
description: |
  文本处理领域。当任务涉及文本转换、格式化、大小写转换时使用。

structures:
  - name: text-pipeline
    path: structures/text-pipeline
    summary: "标准文本处理流水线"
    scenarios: ["文本大写", "文本反转", "字符串变换"]
    weight: 1.0

config:
  default_timeout: 10
---

# Text Processing

此处是 Markdown 文档，可以详细描述使用方式。
```

### 9.4 front matter 字段

| 字段 | 说明 |
|------|------|
| `name` | Complex 名称 |
| `description` | 描述（Agent 决策用） |
| `structures` | 引用的 Structure 列表 |
| `config` | 配置（超时、日志等） |

### 9.5 structures 字段

```yaml
structures:
  - name: text-pipeline           # Structure 名称
    path: structures/text-pipeline  # 相对路径（相对于 SKILL.md）
    summary: "简要描述"
    scenarios: ["使用场景1", "场景2"]
    weight: 1.0                   # 权重（用于 Agent 决策）
```

**重要**：`path` 是相对于 SKILL.md 位置的路径，不是 skills 根目录。

---

## 10. 配置文件

### 10.1 配置文件位置

按以下顺序查找：

1. `./cogtome.toml`
2. `$XDG_CONFIG_HOME/cogtome.toml`
3. `$HOME/.config/cogtome.toml`

### 10.2 配置项

```toml
[runtime]
max_iterations = 100           # foreach 最大迭代次数
max_iterations_hard = 500      # 硬性上限（不可覆盖）

[paths]
units = "units"                # Unit 目录（相对于 skills 根目录）
motifs = "motifs"
structures = "structures"
assemblies = "assemblies"      # MCP Server 用

[units.defaults]
timeout_secs = 30              # 默认超时（秒）

[units.concurrency.some-unit]
max_global = 3                 # 全局最大并发
max_per_host = 1               # 每主机最大并发
resource_key = "openai_api"    # 资源键
```

### 10.3 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `COGTOME_SKILLS_DIR` | 技能根目录 | `./skills` |
| `COGTOME_TIMEOUT` | Unit 超时（秒） | `30` |
| `COGTOME_MAX_CONCURRENT` | foreach 最大并发 | `50` |
| `COGTOME_LOG_FORMAT` | 日志格式 | `pretty` |
| `RUST_LOG` | 日志级别 | - |

---

## 11. HTTP API 参考

### 11.1 启动服务器

```bash
cogtome serve --port 8080
```

### 11.2 端点列表

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/health` | 健康检查 |
| GET | `/metrics` | 指标数据 |
| GET | `/complexes` | 列出所有 Complex |
| GET | `/complexes/:name` | 获取 Complex 元数据 |
| POST | `/run` | 执行 Complex/Motif/Structure/Unit |
| GET | `/api/structures` | 列出所有 Structure |
| GET | `/api/structures/:name` | 获取 Structure manifest |
| PUT | `/api/structures/:name` | 创建/更新 Structure |
| DELETE | `/api/structures/:name` | 删除 Structure |
| GET | `/api/motifs` | 列出所有 Motif |
| GET | `/api/motifs/:name` | 获取 Motif JSON |
| PUT | `/api/motifs/:name` | 创建/更新 Motif |
| GET | `/api/units` | 列出所有 Unit |
| GET | `/api/units/:name` | 获取 Unit 元数据 |
| PUT | `/api/units/:name` | 创建 Unit（生成 bash 桩代码） |
| POST | `/api/validate/:type/:name` | 验证 Motif 或 Structure |

### 11.3 /run 请求格式

```json
{"type": "complex", "name": "text-processing", "input": {"text": "hello"}}
{"type": "motif", "name": "text-transform", "input": {"text": "hello"}}
{"type": "structure", "name": "text-pipeline", "input": {"text": "hello"}}
{"type": "unit", "name": "text-uppercase", "input": {"text": "hello"}}
```

### 11.4 示例

```bash
# 健康检查
curl http://localhost:8080/health

# 列出 Complex
curl http://localhost:8080/complexes

# 执行 Complex
curl -X POST http://localhost:8080/run \
  -H "Content-Type: application/json" \
  -d '{"type":"complex","name":"text-processing","input":{"text":"hello"}}'
```

---

## 12. Web UI 使用指南

### 12.1 启动方式

```bash
# 方式1：一键启动
./start-webui.sh

# 方式2：手动启动
cargo build --release
./target/release/cogtome serve --port 3334 &
cd webui && npm install && npm run dev
```

访问 **http://localhost:3333**

### 12.2 功能概览

- **图形编辑器**：拖拽式编排 Motif
- **节点类型**：支持 9 种节点类型
- **实时预览**：Graph ↔ JSON 同步
- **执行调试**：查看执行链路和数据流

---

## 13. 技能打包与安装

### 13.1 打包技能

```bash
cogtome pack <skill-name>
```

**示例**：

```bash
./target/release/cogtome pack text-processing
# 生成：text-processing.cogtome
```

### 13.2 安装技能

```bash
cogtome install <file.cogtome>
```

**示例**：

```bash
./target/release/cogtome install ./text-processing.cogtome
```

### 13.3 打包格式

`.cogtome` 文件是 `tar.gz` 归档，包含：

- `manifest.json` — 元数据
- `structures/` — Structure 定义
- `motifs/` — Motif 定义
- `units/` — Unit 二进制
- `SKILL.md` — Complex 定义

---

## 14. 常见问题

### 14.1 "Structure 'xxx' not found"

**原因**：目录名与 manifest 中的 `name` 不匹配。

```
structures/web-fetch/manifest.yaml  ← 错误
structures/fetch/manifest.yaml      ← 正确
```

**解决**：确保目录名与 `manifest.yaml` 中的 `name` 字段一致。

### 14.2 "Motif 'xxx' not found"

**原因**：文件名与 manifest 中的 `name` 不匹配。

```
motifs/fetch-web.yaml + name: fetch-web  ← 错误
motifs/fetch-web.yaml + name: fetch-web # 正确（必须与文件名一致）
```

**解决**：文件名（不含扩展名）必须与 `name` 字段一致。

### 14.3 "missing field `type`"

**解决**：在 manifest 中添加 `type: structure` 或 `type: motif`。

### 14.4 Unit 执行超时

**解决**：增加超时时间

```bash
export COGTOME_TIMEOUT=60
```

或在 `cogtome.toml` 中配置：

```toml
[units.defaults]
timeout_secs = 60
```

### 14.5 变量解析失败

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
skills/
├── units/
│   └── <unit-name>/
│       └── bin/
│           └── <unit-name>          # 可执行文件
├── motifs/
│   └── <motif-name>.json            # 文件名 = name 字段
├── structures/
│   └── <structure-name>/             # 目录名 = name 字段
│       └── manifest.yaml
└── <complex-name>/
    └── SKILL.md                     # Complex 定义
```

---

## 附录 C：系统依赖

| 工具 | 用途 | 安装 |
|------|------|------|
| `jq` | JSON 解析 | `apt install jq` / `brew install jq` |

---

## 附录 D：Trace Dashboard（可观测性）

### D.1 概述

COGTOME 内置 trace 可视化 dashboard，读取 `COGTOME_TRACE_DIR` 下的 JSONL 执行记录，以 HTML 页面展示执行历史、成功率、耗时分布。

### D.2 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `COGTOME_TRACE_DIR` | trace 日志目录 | `~/.cogtome/traces` |

### D.3 Trace 文件格式

每次执行完成后，COGTOME 将结构化记录写入 trace 日志：

```json
{
  "trace_id": "uuid",
  "skill": "daily-summary",
  "date": "2026-05-01",
  "started_at": "2026-05-01T18:00:00Z",
  "completed_at": "2026-05-01T18:00:05Z",
  "duration_ms": 5200,
  "status": "success",
  "nodes": [
    { "id": "index-memory", "type": "unit", "ok": true, "ms": 1200 },
    { "id": "extract-tasks", "type": "unit", "ok": true, "ms": 800 }
  ]
}
```

### D.4 启动 Trace Dashboard

```bash
# 需要先构建
cargo build --release

# 启动 dashboard（默认端口 4321）
cargo run --release -- trace-dashboard

# 或指定端口
cargo run --release -- trace-dashboard --port 8080
```

### D.5 Dashboard 功能

- **执行统计**：总执行次数、成功率、平均耗时
- **按技能筛选**：支持按 skill name 和 date 过滤
- **执行详情**：每次执行的节点耗时、状态、错误信息
- **自动刷新**：每 30 秒自动刷新数据

### D.6 REST API

| 方法 | 路径 | 说明 |
|------|------|------|
| GET | `/` | HTML dashboard 页面 |
| GET | `/api/traces` | 列出 trace 记录 |
| GET | `/api/traces/stats` | 统计汇总 |
| GET | `/api/traces/:trace_id` | 单条 trace 详情 |

查询参数：
- `skill` — 按技能名筛选
- `date` — 按日期筛选（YYYY-MM-DD）
- `limit` — 返回条数（默认 50）

---

## 附录 E：Benchmark 基准测试

### E.1 概述

COGTOME 内置基准测试套件，测量核心指标：执行成功率、平均耗时、Motif 复用率。

### E.2 运行基准测试

```bash
cargo test --release -- --nocapture benchmark
```

### E.3 环境变量

| 变量 | 说明 | 默认值 |
|------|------|--------|
| `BENCHMARK_ITERATIONS` | 迭代次数 | `100` |
| `BENCHMARK_SKILL` | 测试用技能名 | `text-uppercase` |

### E.4 输出指标

| 指标 | 说明 |
|------|------|
| **Success Rate** | 成功次数 / 总次数 |
| **Avg Duration** | 平均执行耗时 |
| **P50 / P95 Duration** | 中位数和 95 分位耗时 |
| **Motif Reuse Rate** | 共享底层 Unit 的技能比例 |

### E.5 示例输出

```
══════════════════════════════════════════════════════════════
  COGTOME Benchmark Results
══════════════════════════════════════════════════════════════

  Success Rate:              100 / 100 (100.0%)
  Failures:                       0

  Duration Statistics:
  Average:                    52.3 ms
  Min:                         31 ms
  Max:                        187 ms
  P50:                        48 ms
  P95:                        89 ms

  Motif Reuse Rate:           60.0%

══════════════════════════════════════════════════════════════
```

---

*最后更新：2026-05-01*
