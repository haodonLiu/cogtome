# COGTOME 技术规格文档

> 版本：v1.0
> 更新：2026-04-24
> 状态：Phase 1 实施准备

---

## 一、项目概述

### 1.1 定位

COGTOME 是面向 AI Agent 的微型操作系统与执行运行时。

**核心原则：** COGTOME 不是 Agent，但必须是"带编排能力的执行引擎"。

| 能力 | 归属 | 原因 |
|------|------|------|
| 进程管理（fork/exec、JSON 契约） | ✅ COGTOME | 基础运行时 |
| 循环、条件分支、并行（控制流） | ✅ COGTOME | 执行纪律，不是智能决策 |
| 结果聚合、错误重试 | ✅ COGTOME | 执行可靠性 |
| 意图匹配、Complex 选择 | ❌ OpenClaw | Agent 的智能 |

### 1.2 操作系统隐喻

> "COGTOME = Agent 的微型操作系统"
> "历史不会重复，但会押韵。"

| OS 概念 | Agent 对应 |
|---------|-----------|
| 进程 | Complex（资源边界） |
| 线程 | Unit（原子执行） |
| 系统调用 | Function Calling（受控接口） |
| 内核/调度器 | Runtime（权限、调度、IPC） |
| 多核调度 | 多 Agent 协作 |

---

## 二、架构设计

### 2.1 四层模型

```
Agent (自然语言意图)
        │
        ▼
┌─────────────────────┐
│      Complex        │  ← Agent 唯一可见的层
│   (领域典籍 Tome)   │     持有 description，参与自动发现
│                     │
│  select_structure() │
└─────────┬───────────┘
          │ 加载 Structure
          ▼
┌─────────────────────┐
│     Structure       │  ← 业务黑盒
│  (传动机构 Drive)    │     manifest.yaml 定义契约
│                     │
│  execute(motifs)   │
└─────────┬───────────┘
          │ 加载 Motif
          ▼
┌─────────────────────┐
│       Motif         │  ← 编排逻辑
│  (齿轮组 Assembly)   │     YAML / Python / Shell
│                     │
│  unit.call()       │
└─────────┬───────────┘
          │ IPC 调用
          ▼
┌─────────────────────┐
│        Unit         │  ← 原子执行
│      (齿 Cog)       │     独立进程，stdin/stdout JSON
│                     │
│  fork + exec        │
└─────────────────────┘
```

### 2.2 层级职责

| 层级 | 名称 | Agent 可见？ | 本质 | 一句话定义 |
|------|------|-------------|------|-----------|
| **L4** | **Complex** | ✅ 唯一可见 | 领域门面 | 有 `description`，被自动发现 |
| **L3** | **Structure** | ❌ 不可见 | 业务结构 | 完成具体目标的内部实现 |
| **L2** | **Motif** | ❌ 不可见 | 工作链 | 编排 Unit 的内部逻辑 |
| **L1** | **Unit** | ❌ 不可见 | 原子执行体 | 固定 CLI，最小执行一步 |

### 2.3 核心纪律

1. **Unit 之间绝不相互调用**（Runtime 通过 `COGTOME_UNIT_MODE=1` 阻止）
2. **Motif 之间不直接相互调用**（通过 Structure 组合）
3. **Structure 不直接调用 Unit**（必须通过 Motif 编排）
4. **Complex 是唯一有 `description` 的层**
5. **所有跨层调用通过 Runtime IPC**（禁止裸 `subprocess`）

---

## 三、语法规格

### 3.1 foreach 循环

```yaml
foreach:
  over: "${steps.status.output.files}"   # 要迭代的数组表达式
  as: file                               # 迭代变量名
  max_iterations: 50                     # 安全上限（默认 50）
  on_error: fail_fast                    # fail_fast | continue
flow:
  - name: diff
    unit: git-diff
    input:
      file: "${item}"                   # 引用当前迭代项
  - name: review
    if: "${steps.diff.output.is_binary} == false"
    unit: ai-review
aggregate:
  mode: array                           # array | object | sum | join
  map:
    file: "${item}"
    review: "${steps.review.output.comment}"  # 跳过时为 null
```

**内置变量：**

| 变量 | 作用域 | 说明 |
|------|--------|------|
| `item` | foreach 子 flow | 当前迭代项 |
| `__index` | foreach 子 flow | 当前迭代索引（0 开始） |
| `__error` | 仅在 aggregate.map 中 | 当前迭代错误信息 |

**aggregate 模式：**

| 模式 | 用途 | 语法 |
|------|------|------|
| `array` | 收集为数组 | `map` 定义每个元素 |
| `object` | 按键聚合 | `key: "${item.filename}"` |
| `sum` | 数值累加 | `sum: "${steps.diff.output.lines}"` |
| `join` | 字符串拼接 | `join` + `separator` |

### 3.2 表达式引擎（Phase 1）

**Phase 1 支持：**
```yaml
"${steps.a.output.field[0]}"          # 变量 + 索引
"${steps.a.output.numbers.length}"    # 长度属性
"${steps.a.output.numbers[-1]}"       # 负索引
"${a > 5 ? 'big' : 'small'}"        # 三目运算
```

**Phase 2（后续）：**
- `filter()`, `map()`, `length()`, `join()` 内置函数
- 简单 lambda

### 3.3 if 条件

```yaml
- name: review
  if: "${steps.diff.output.is_binary} == false"
  unit: ai-review
```

**条件分类：**

| 类型 | 示例 | 归属 |
|------|------|------|
| 确定性条件 | `is_binary == false` | ✅ Motif 编排 |
| AI 输出条件 | `quality > 0.8` | ⚠️ 封装为 Unit |

---

## 四、路径解析

### 4.1 三级查找

```rust
enum UnitResolution {
  ComplexLocal {},    // 1. Complex 私有 Unit（最优先）
  GlobalRegistry {},  // 2. 全局注册表
  SystemPath {},      // 3. 系统 PATH（兜底）
}
```

### 4.2 平台默认路径

| 平台 | 默认路径 |
|------|----------|
| Linux | `~/.local/share/cogtome/` |
| macOS | `~/Library/Application Support/cogtome/` |
| Windows | `%APPDATA%/cogtome/` |

**目录结构：**
```
~/.local/share/cogtome/
├── units/              # Unit 查找路径
└── complexes/          # Complex 查找路径
```

**环境变量覆盖：**
```bash
COGTOME_UNITS_PATH=/custom/path
COGTOME_COMPLEXES_PATH=/custom/complexes
```

---

## 五、快照语义

### 5.1 实现约束

```rust
// StepState 内部使用 Arc<Value>，clone 为 O(1)
struct StepState {
    data: Arc<serde_json::Value>,
}

// global_steps 本身是 Arc<HashMap>，clone 时只复制指针
let snapshot = Arc::clone(&ctx.global_steps); // O(1)
```

### 5.2 变量遮蔽

- ✅ 允许内部 step 与外部同名（遮蔽）
- ❌ 禁止在 foreach 内部修改外部状态

**解析优先级：**
```
1. 迭代变量（item, __index）
2. 当前局部 steps（foreach 内部）
3. 快照中的外部 steps（只读）
4. 用户输入 params
```

---

## 六、错误处理

### 6.1 foreach 错误策略

| 策略 | 行为 |
|------|------|
| `fail_fast`（默认） | 任一迭代失败，不产出 aggregate，直接抛错 |
| `continue` | 跳过失败迭代，`__error` 记录，继续执行 |

### 6.2 边界情况处理

| 场景 | 处理策略 |
|------|----------|
| `if` 表达式解析失败 | 视为 `false`（跳过） |
| `foreach.over` 不存在 | 视为空数组 |
| `return` 引用缺失 | **抛错** |
| aggregate.map 引用不存在 | 静默 null + 警告日志 |
| `on_error: fail_fast` 迭代失败 | 不产出 aggregate，向上抛错 |

---

## 七、并行安全（Phase 2）

### 7.1 核心原则

**未声明并发安全的 Unit，在并行 foreach 中默认串行化。**

### 7.2 Unit 并发声明

```yaml
name: ai-review
concurrency:
  max_global: 3           # 全局最多 3 并发
  max_per_host: 1         # 单机最多 1
  resource_key: "openai_api"  # 共享资源标识
```

---

## 八、配置项

```toml
# cogtome.toml
[runtime]
max_iterations = 50        # 默认上限
max_iterations_hard = 500 # 绝对上限

[paths]
# 默认值因平台而异，通过环境变量覆盖
```

**错误信息示例：**
```
Error: MaxIterationsExceeded
  foreach 'review_loop' attempted 51 iterations (limit: 50).
  Hint: Increase max_iterations in cogtome.toml or ask Agent to batch process.
```

---

## 九、实施优先级

```
Phase 1（立即，阻塞所有场景）：
├── Unit/Complex 路径解析（三级查找 + dirs crate）
├── foreach 循环语法（as, over, flow, aggregate）
├── aggregate 聚合（array/object/sum/join）
├── 表达式引擎 Phase 1 子集
├── 变量遮蔽规则
├── 边界情况处理规则
└── max_iterations 硬限制

Phase 2（后续，解锁复杂场景）：
├── foreach parallel: true
├── Unit concurrency 声明 + Runtime 限流
├── 表达式引擎增强（filter, join）
├── Schema 约束扩展
└── 默认值机制

Phase 3（体验优化）：
├── auto-complex 快捷注册
└── AI token 分块
```

---

## 十、OpenClaw 集成

### 10.1 分层边界

```
OpenClaw（决策层）：理解意图 → 选择 Complex → 构造参数
     ↓
COGTOME（执行层）：接收指令 → 解析 Structure → 编排 Motif → 调度 Unit
```

**⚠️ 关键约束：**
- OpenClaw 不做执行，只做决策
- COGTOME 不做匹配，只做执行
- Discovery = 能力目录（`ls` + `cat`），不是自动路由

### 10.2 目录结构

```
~/.milos/                          # Milos 执行引擎根目录
├── structures/                    # Structure 层
├── motifs/                       # Motif 层
├── units/                        # Unit 层
└── logs/                         # 执行日志

~/.agents/skills/                  # OpenClaw 技能发现层
└── cogtome/                      # COGTOME Complex
    └── SKILL.md
```

### 10.3 实施阶段

| 阶段 | 任务 |
|------|------|
| Phase 0 | 基础设施（Rust 环境、CLI 可运行） |
| Phase 1 | Unit 层构建（fetch-url, read-file, git-status 等） |
| Phase 2 | Motif 层构建（web-research, git-audit 等） |
| Phase 3 | Structure 层构建（code-review 等） |
| Phase 4 | Milos 执行层集成 |

---

*文档版本：v1.0 | 日期：2026-04-24*
