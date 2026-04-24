> English | [中文版本](README_CN.md)

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
2. [核心架构：四层模型](#核心架构四层模型)
3. [快速开始](#快速开始)
4. [完整教程：从零创建一个 Skill](#完整教程从零创建一个-skill)
5. [接口规格](#接口规格)
6. [CLI 参考](#cli-参考)
7. [技术实现](#技术实现)
8. [路线图](#路线图)
9. [设计原则](#设计原则)

---

## 什么是 COGTOME

### 定位

COGTOME 不是框架，不是库，而是一个**独立的进程级运行时**——Agent 的微型操作系统。

| 操作系统概念 | COGTOME 对应 |
|-----------|------------|
| 内核 | COGTOME Runtime (Rust) |
| 用户进程 | Agent (LLM / 程序) |
| 系统调用 | Unit (原子执行) |
| 用户态函数 | Motif (编排逻辑) |
| 应用程序 | Structure (业务封装) |
| 应用商店 | Complex (领域门面) |
| Shell | `cogtome` CLI |

### 核心问题

Agent 需要调用外部工具（浏览器、数据库、API、文件处理），但直接 `subprocess` 会导致：
- 进程管理混乱（泄漏、僵尸进程）
- 无类型安全（输入输出无契约）
- 无版本与发现机制
- 无执行链路追踪

COGTOME 解决以上全部问题：Agent 只负责**编写业务逻辑**（Unit/Motif/Structure），Runtime 负责**一切基础设施**（进程、调度、日志、安全、发现）。

### 品牌隐喻

| 技术术语 | 品牌隐喻 | 含义 |
|---------|---------|------|
| Unit | 齿 (Cog) | 不可再分的原子执行体 |
| Motif | 齿轮组 (Gear Assembly) | 齿的编排与组合 |
| Structure | 传动机构 (Drive Train) | 完成业务目标的结构 |
| Complex | 典籍 (Tome) | 收录传动机构的领域之书 |
| 执行 | 啮合 (Engage) | 齿与齿咬合转动 |

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
          │ 加载 Structure
          ▼
┌─────────────────────┐
│     Structure       │  ← 业务黑盒
│  (传动机构 Drive)    │     manifest.yaml 定义契约
│                     │
│  execute(motifs)    │
└─────────┬───────────┘
          │ 加载 Motif
          ▼
┌─────────────────────┐
│       Motif          │  ← 编排逻辑
│  (齿轮组 Assembly)   │     YAML / Python / Shell
│                     │
│  unit.call()        │
└─────────┬───────────┘
          │ IPC 调用
          ▼
┌─────────────────────┐
│        Unit          │  ← 原子执行
│      (齿 Cog)       │     独立进程，stdin/stdout JSON
│                     │
│  fork + exec        │
└─────────────────────┘
```

### 层级总览

| 层级 | 名称 | Agent 可见？ | 本质 | 一句话定义 |
|------|------|-------------|------|-----------|
| **L4** | **Complex** | ✅ 唯一可见 | 领域门面 | 有 `description`，被自动发现 |
| **L3** | **Structure** | ❌ 不可见 | 业务结构 | 完成具体目标的内部实现 |
| **L2** | **Motif** | ❌ 不可见 | 工作链 | 编排 Unit 的内部逻辑 |
| **L1** | **Unit** | ❌ 不可见 | 原子执行体 | 固定 CLI，最小执行一步 |

### 核心纪律

1. **Unit 之间绝不相互调用**（Runtime 通过 `COGTOME_UNIT_MODE=1` 阻止）
2. **Motif 之间不直接相互调用**（通过 Structure 组合）
3. **Structure 不直接调用 Unit**（必须通过 Motif 编排）
4. **Complex 是唯一有 `description` 的层**
5. **所有跨层调用通过 Runtime IPC**（禁止裸 `subprocess`）

---

## 快速开始

### 1. 安装

```bash
# 克隆仓库
git clone https://github.com/haodonLiu/cogtome.git
cd cogtome

# 编译
cargo build --release

# 可选：安装到 PATH
cp target/release/cogtome /usr/local/bin/
```

### 2. 运行内置示例

```bash
# 发现所有 Complex
cogtome discover

# 直接运行 Unit（原子能力）
cogtome unit run text-uppercase --input '{"text":"hello"}'
# {"result": "HELLO"}

# 运行 Motif（编排逻辑）
cogtome motif run text-transform --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}

# 运行 Structure（业务封装）
cogtome structure run text-pipeline --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}

# 运行 Complex（完整领域 Skill）
cogtome run text-processing --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}
```

### 3. 项目结构

```
cogtome/
├── src/                    # Runtime 源码（Rust）
│   ├── main.rs             # CLI 入口
│   ├── context.rs          # 执行上下文 + 变量解析
│   ├── discovery.rs        # 目录扫描与发现
│   └── engine.rs           # Unit 运行器 + Motif 引擎 + Structure 执行器
├── skills/                 # Agent 创作目录（Runtime 不内置业务逻辑）
│   ├── units/              # 原子执行体
│   ├── motifs/             # 编排逻辑
│   ├── structures/         # 业务结构
│   └── <complex>/          # 领域典籍
└── Cargo.toml
```

---

## 完整教程：从零创建一个 Skill

本教程演示 Agent 如何从零开始创建一个可运行的 Skill：文本处理流水线。

### Step 1：铸造 Unit（编写原子能力）

Unit 是**任意 CLI 可执行文件**。Agent 用 Python、Bash、Go、Rust 都可以，只要遵守 stdin/stdout JSON 契约。

创建目录和文件：

```bash
mkdir -p skills/units/text-uppercase/bin
cat > skills/units/text-uppercase/bin/text-uppercase << 'EOF'
#!/usr/bin/env python3
import sys, json
inp = json.load(sys.stdin)
print(json.dumps({"result": inp["text"].upper()}))
EOF
chmod +x skills/units/text-uppercase/bin/text-uppercase
```

再创建第二个 Unit：

```bash
mkdir -p skills/units/text-reverse/bin
cat > skills/units/text-reverse/bin/text-reverse << 'EOF'
#!/usr/bin/env python3
import sys, json
inp = json.load(sys.stdin)
print(json.dumps({"result": inp["text"][::-1]}))
EOF
chmod +x skills/units/text-reverse/bin/text-reverse
```

**Unit 接口契约**：
- **输入**：stdin 接收 UTF-8 JSON
- **输出**：stdout 输出 UTF-8 JSON
- **错误**：exit code 非 0，stderr 输出可读信息
- **环境变量**：Runtime 自动注入 `COGTOME_EXECUTION_ID`、`COGTOME_TRACE_ID`、`COGTOME_UNIT_MODE=1`

### Step 2：编织 Motif（编排 Unit）

Motif 回答一个问题：**"给定输入，按什么顺序调用哪些 Unit？"**

Agent 编写 YAML 声明式 Motif：

```yaml
# skills/motifs/text-transform.yaml
name: text-transform
type: motif
units_required: [text-uppercase, text-reverse]

flow:
  - name: upper
    unit: text-uppercase
    input:
      text: "${params.text}"

  - name: rev
    unit: text-reverse
    input:
      text: "${params.text}"

return:
  upper: "${steps.upper.output.result}"
  reversed: "${steps.rev.output.result}"
  combined: "${steps.upper.output.result} | ${steps.rev.output.result}"
```

**变量作用域**：
- `${params.xxx}` —— Structure/Agent 传入的原始参数
- `${steps.<name>.output.xxx}` —— 某一步的 stdout JSON 字段
- `${steps.<name>.exit_code}` —— 某一步的退出码
- `${env.xxx}` —— 环境变量

**控制流**（当前 Demo 支持串行，完整版支持并行/条件）：
- 默认串行执行
- `parallel: <group>` —— 同组并发
- `after: <group>` —— 等待组完成
- `condition: "${expr}"` —— 条件执行
- `on_error: <label>` —— 错误跳转

### Step 3：组建 Structure（封装业务目标）

Structure 是完成**一个具体业务目标**的黑盒。对外只暴露输入输出 Schema。

```yaml
# skills/structures/text-pipeline/manifest.yaml
name: text-pipeline
type: structure

motifs:
  - name: text-transform

input_schema:
  type: object
  required: [text]
  properties:
    text: { type: string }

output_schema:
  type: object
  properties:
    upper: { type: string }
    reversed: { type: string }
    combined: { type: string }

resources: {}
```

**字段说明**：
- `motifs`：该 Structure 使用的 Motif 列表，按顺序执行
- `input_schema` / `output_schema`：JSON Schema，Runtime 自动校验
- `resources`：资源需求（内存、网络、GPU），供调度器参考
- `constraints`：约束条件（如 `webgl: true`），供 Complex 选择时参考

### Step 4：编纂 Complex（领域门面）

Complex 是**Agent 唯一可见的层**。它持有 `description`，被 Runtime 自动发现。

```yaml
# skills/text-processing/SKILL.md
---
name: text-processing
description: |
  文本处理领域。当任务涉及文本转换、格式化、大写/小写、反转、拼接、
  简单字符串操作时，自动调用此 Skill。

structures:
  - name: text-pipeline
    path: structures/text-pipeline
    summary: "标准文本处理流水线"
    scenarios: ["文本大写", "文本反转", "字符串变换"]
    weight: 1.0

config:
  default_timeout: 10
  log_retention: "1d"
---
```

**关键规则**：
- 必须包含 `description`，否则不参与自动发现
- `structures` 列表定义下辖的所有 Structure
- `weight` 用于冲突解决（多个 Structure 匹配时的优先级）
- `scenarios` 用于意图匹配的关键词扩展

### Step 5：验证与运行

```bash
# 验证发现
cogtome discover
# Found 1 Complex(es):
#   text-processing  文本处理领域...

# 运行
cogtome run text-processing --input '{"text":"hello"}'
# {"upper": "HELLO", "reversed": "olleh"}
```

---

## 接口规格

### Unit 契约

#### 进程模型

```
输入：stdin 接收 UTF-8 JSON
输出：stdout 输出 UTF-8 JSON
诊断：stderr 输出人类可读文本
状态：exit code 0 = 成功，非 0 = 失败
```

#### 退出码标准

| Code | 含义 | Runtime 行为 |
|------|------|-------------|
| 0 | 成功 | 解析 stdout JSON，返回给上层 |
| 1 | 输入错误 | 不可重试，报告输入问题 |
| 2 | 处理异常 | 可重试 |
| 3 | 依赖不可用 | 可重试，指数退避 |
| 126 | 命令不可执行 | 不可重试，报告权限问题 |
| 127 | 命令未找到 | 不可重试，报告 Unit 未安装 |
| 130 | SIGINT | 可重试，用户中断或超时 |
| 137 | SIGKILL | 可重试，OOM 或强制终止 |

#### 环境变量

```bash
COGTOME_UNIT_MODE=1       # 禁止 Unit 内部再调 Unit
COGTOME_EXECUTION_ID=xxx  # 本次执行唯一 ID
COGTOME_TRACE_ID=xxx      # 分布式追踪 ID
COGTOME_LOG_LEVEL=info    # 日志级别
COGTOME_TIMEOUT_MS=30000  # 剩余超时毫秒数
```

#### 目录模板

```
units/<unit-name>/
├── SKILL.md          # CLI 契约声明（无 description）
├── errors.yaml       # 错误模式库（可选）
└── bin/<unit-name>   # 可执行入口（chmod +x）
```

### Motif 契约

#### YAML 声明式 Motif

```yaml
name: data-pipeline
type: motif
units_required: [fetch-url, parse-json]

flow:
  - name: fetch
    unit: fetch-url
    input:
      url: "${params.url}"
    output: raw_data

  - name: parse
    unit: parse-json
    input:
      text: "${steps.fetch.output.raw_data}"
    output: json_obj

return:
  data: "${steps.parse.output.json_obj}"
```

#### Python Motif（复杂逻辑）

```python
from agents_sdk import Motif, unit, Context

class DataPipelineMotif(Motif):
    name = "data-pipeline"
    units_required = ["fetch-url", "parse-json"]

    def run(self, ctx: Context, url: str):
        raw = unit.call("fetch-url", {"url": url}, ctx=ctx)
        parsed = unit.call("parse-json", {"text": raw["result"]}, ctx=ctx)
        return parsed.data
```

**关键**：Python Motif 中的 `unit.call()` 通过 Unix Socket IPC 与 Runtime 通信，**不直接创建进程**。

### Structure 契约

#### manifest.yaml

```yaml
name: text-pipeline
type: structure

motifs:
  - name: text-transform

input_schema:
  type: object
  required: [text]
  properties:
    text: { type: string }

output_schema:
  type: object
  properties:
    upper: { type: string }

resources:
  memory: "512m"
  network: true
  gpu: false

constraints:
  webgl: false
```

#### 自定义执行器（可选）

若 `structures/<name>/structure.py` 存在，Runtime 加载并执行：

```python
from agents_sdk import Structure, motif, Context

class TextPipelineStructure(Structure):
    def execute(self, params: dict, ctx: Context) -> dict:
        m = motif.load("text-transform")
        return m.run(ctx, **params)
```

若无 `structure.py`，Runtime 使用**默认执行器**：按 `manifest.motifs` 顺序加载并执行 Motif。

### Complex 契约

#### SKILL.md

```yaml
---
name: web-automation
description: |
  浏览器自动化领域...

structures:
  - name: lightpanda
    path: structures/lightpanda
    summary: "基于原生引擎的高速无头浏览器"
    scenarios: ["静态页面抓取", "表单自动化"]
    constraints: { webgl: false }
    weight: 0.8

config:
  default_timeout: 30
  max_concurrent: 3
  log_retention: "1d"
---
```

#### 自定义选择器（可选）

```python
from agents_sdk import Complex, Structure

class WebAutomationComplex(Complex):
    def select_structure(self, intent: str, constraints: dict) -> Structure:
        if constraints.get("webgl"):
            return self.load_structure("playwright")
        return self.load_structure("lightpanda")
```

---

## CLI 参考

```bash
# 发现与浏览
cogtome discover                              # 扫描所有 Complex
cogtome skill list                            # 列出 Complex
cogtome skill show <name>                     # 查看 Complex 详情
cogtome skill search <keyword>                 # 模糊搜索

# 调试层（开发者工具）
cogtome unit list                             # 列出所有 Unit
cogtome unit show <name>                      # 查看 Unit 契约
cogtome unit run <name> --input <json>       # 直接运行 Unit
cogtome unit run <name> --stdin               # 从 stdin 读入

cogtome motif list                            # 列出所有 Motif
cogtome motif run <name> --input <json>      # 运行 Motif

cogtome structure list                       # 列出所有 Structure
cogtome structure validate <name>             # 校验 manifest
cogtome structure run <name> --input <json> # 运行 Structure

# 执行层（Agent 使用）
cogtome run <complex> --input <json>         # 运行 Complex
cogtome run <complex> --input <json> --dry-run # 编译计划但不执行

# 日志与检查
cogtome logs                                  # 列出今日执行
cogtome logs --date 2026-04-24              # 查看历史
cogtome inspect <execution-id> --tree         # 树形展示四层调用

# 系统管理
cogtome validate                              # 校验所有 Skill
cogtome validate --fix                        # 自动修复常见问题
cogtome daemon start                          # 启动常驻进程
cogtome daemon stop
cogtome daemon status

# 打包与分发（未来）
cogtome pack ./my-skill/                      # 打包为 .cogtome 文件
cogtome install my-skill.cogtome             # 安装 Skill
```

---

## 技术实现

### Runtime 模块（Rust）

```
src/
├── main.rs                 # CLI 入口 (clap)
│   ├── unit run            # 直接调用 UnitRunner
│   ├── motif run           # 调用 YamlMotifEngine
│   ├── structure run       # 调用 StructureExecutor
│   └── run                 # Complex → Structure → Motif → Unit
├── context.rs              # 执行上下文 + 变量解析
│   ├── ExecContext         # params + steps HashMap
│   └── resolve_var()       # ${params} / ${steps} / ${env}
├── discovery.rs            # 目录扫描与元数据发现
│   ├── find_unit()         # 全局 → Complex 私有，优先级查找
│   ├── find_motif()        # .yaml / .py / .sh
│   ├── find_structure()    # manifest.yaml
│   └── discover_complexes() # 扫描 SKILL.md
└── engine.rs               # 核心执行引擎
    ├── UnitRunner          # tokio::process fork/exec
    ├── YamlMotifEngine     # YAML 解析 + 串行调度
    └── StructureExecutor   # manifest 加载 + Motif 链执行
```

### 执行流程

```
Agent Query
    │
    ▼
┌────────────────────────────────┐
│ 1. Discovery                   │  扫描 ~/.agents/skills/*/SKILL.md
│    构建 Complex 索引           │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 2. Resolution                  │  描述相似度 / 约束匹配 / 权重排序
│    Complex.select_structure()  │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 3. Compilation                 │  Structure manifest → ExecutionPlan
│    静态检查依赖完整性          │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 4. Scheduling                  │  按 ExecutionStep 顺序执行
│    • 串行：阻塞执行            │
│    • 并行：tokio::spawn        │
│    • 资源：acquire → release   │
│    • 超时：tokio::timeout      │
└────────┬───────────────────────┘
         │
         ▼
┌────────────────────────────────┐
│ 5. Validation                  │  校验 output_schema
│    写日志索引 index.json       │
└────────────────────────────────┘
```

### 多语言 Motif 策略

| 类型 | 扩展名 | 执行方式 | 状态 |
|------|--------|---------|------|
| YAML Motif | `.yaml` | Rust 原生解析 | ✅ 已实现 |
| Python Motif | `.py` | 子进程 + IPC → Runtime | 🔮 Phase 2 |
| Shell Motif | `.sh` | `tokio::process::Command` | 🔮 Phase 2 |
| Rust Motif | `.so` | `libloading` 动态加载 | 🔮 Phase 4 |

### Python SDK IPC（未来）

Python Motif 不直接 `subprocess.run`，而是通过 Unix Domain Socket 与 Runtime 通信：

```python
# agents_sdk/unit.py
class CogtomeClient:
    def __init__(self, socket_path="/tmp/cogtome.sock"):
        self.sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        self.sock.connect(socket_path)

    def unit_call(self, name, input, ctx):
        # 发送 JSON-RPC
        ...
```

这样既保留 Python 的灵活性，又获得 Rust 的进程管理性能。

---

## 路线图

### Phase 1：Core MVP ✅（当前）

- [x] CLI 框架（unit / motif / structure / run / discover）
- [x] UnitRunner：`tokio::process` + stdin/stdout JSON + 超时
- [x] Discovery：扫描 `skills/` 目录树
- [x] YamlMotifEngine：变量解析 + 串行执行
- [x] StructureExecutor：manifest 加载 + Motif 链
- [x] Complex 发现：SKILL.md 解析

### Phase 2：Daemon 与并发

- [ ] `cogtome daemon`（Unix Socket + HTTP API）
- [ ] 元数据缓存与热重载
- [ ] Unit 进程预热池
- [ ] 并行 Unit 调用（`unit.gather()`）
- [ ] Python SDK IPC 客户端
- [ ] YAML Motif：并行组 + 条件分支

### Phase 3：资源管理与安全

- [ ] 资源型 Unit：`resource.acquire/release` + RAII Guard
- [ ] WAL 崩溃恢复机制
- [ ] Linux Landlock 文件系统隔离
- [ ] seccomp-bpf 系统调用过滤
- [ ] cgroups v2 资源限制

### Phase 4：生态与优化

- [ ] `cogtome pack/install` 打包分发
- [ ] Registry / 中央仓库协议
- [ ] Rust Motif 动态加载（`.so`）
- [ ] Web UI 监控面板
- [ ] 性能基准测试

---

## 设计原则

1. **Runtime 零业务逻辑**：COGTOME 二进制不内置任何 Unit。Agent 根据需求自行铸造。
2. **Agent 创作自由**：Unit 可用任何语言编写；Motif 可用 YAML/Python/Shell；Structure 可纯声明或自定义执行器。
3. **强契约**：所有跨层调用通过 Schema 校验（JSON Schema），输入输出类型安全。
4. **进程隔离**：Unit 之间绝不相互调用，每个 Unit 是独立的 fork + exec。
5. **可观测性**：每次执行生成完整的四层链路日志（JSON Lines），支持 `cogtome inspect --tree`。

---

## 许可证

MIT
