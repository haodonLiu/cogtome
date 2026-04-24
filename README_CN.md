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
2. [核心架构：四层模型](#核心架构四层模型)
3. [快速开始](#快速开始)
4. [项目结构](#项目结构)
5. [CLI 参考](#cli-参考)
6. [路线图](#路线图)
7. [设计原则](#设计原则)

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
│  (齿轮组 Assembly)   │     YAML 声明式
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
│   ├── context.rs          # 执行上下文 + 变量解析
│   ├── discovery.rs        # 目录扫描
│   └── engine.rs           # UnitRunner + MotifEngine + StructureExecutor
├── skills/                 # Skills 目录
│   ├── units/              # 原子执行体
│   ├── motifs/             # 编排逻辑 (YAML)
│   ├── structures/          # 业务结构
│   └── <complex>/          # 领域 Complex
│       └── SKILL.md        # Complex 定义（必须有 description）
├── test_suite/             # 测试用例
├── development/            # 技术文档
└── Cargo.toml
```

### Skills 目录结构

```
skills/
├── units/<name>/bin/<name>     # 可执行 Unit（任意语言）
├── motifs/<name>.yaml          # YAML Motif
├── structures/<name>/
│   └── manifest.yaml           # Structure manifest
└── <complex>/SKILL.md          # Complex（必须有 description）
```

---

## CLI 参考

### 已实现 ✅

```bash
# 发现
cogtome discover                              # 扫描所有 Complex

# 执行
cogtome run <complex> --input <json>        # 运行 Complex
cogtome unit run <name> --input <json>     # 直接运行 Unit
cogtome motif run <name> --input <json>    # 运行 Motif
cogtome structure run <name> --input <json> # 运行 Structure

# 帮助
cogtome help                                  # 显示所有命令
```

### 计划中（尚未实现）🔮

```bash
cogtome unit list                            # 列出所有 Unit
cogtome motif list                           # 列出所有 Motif
cogtome structure list                       # 列出所有 Structure
cogtome validate                             # 校验所有 Skills
cogtome logs                                 # 显示执行日志
cogtome inspect <id>                         # 检查执行树
cogtome daemon start/stop                   # 守护进程模式
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

### Phase 2: 核心编排 🔮

- [ ] `foreach` 循环 + `aggregate` 聚合
- [ ] 表达式引擎（变量索引、数组访问）
- [ ] `if` 条件执行
- [ ] `max_iterations` 安全限制
- [ ] 错误分层（`runtime` / `motif` / `unit`）
- [ ] 快照语义（只读外部状态）

### Phase 3: 并发 🔮

- [ ] 并行 `foreach`（`parallel: true`）
- [ ] Unit 并发声明（`max_global`, `resource_key`）
- [ ] Runtime 资源限制器

### Phase 4: 生态 🔮

- [ ] Python Motif（Unix Socket IPC）
- [ ] HTTP API Server
- [ ] Discovery API（`GET /complexes`）
- [ ] `auto-complex` 快捷注册
- [ ] `cogtome pack/install`

---

## 设计原则

1. **Runtime 零业务逻辑** — COGTOME 二进制不内置任何 Unit
2. **Agent 创作自由** — Unit 可用任意语言，Motifs 用 YAML/Python/Shell
3. **强契约** — 每层 JSON Schema 校验
4. **进程隔离** — Unit 之间绝不相互调用
5. **可观测性** — 完整执行链路日志

---

## 相关链接

- [技术规格](./development/TECHNICAL_SPEC.md) — 详细架构
- [操作系统隐喻](./development/OS_METAPHORS.md) — 概念基础
- [OpenClaw 集成](./development/OPENCLAW_INTEGRATION.md) — 集成协议

---

## 许可证

MIT
