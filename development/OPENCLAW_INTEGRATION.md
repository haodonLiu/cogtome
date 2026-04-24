# COGTOME × OpenClaw 集成协议

> 状态：Phase 0.5 待补充
> 锁定时间：2026-04-24

---

## 一、为什么现在就要锁定三个耦合点

COGTOME Phase 1 可以独立跑通（CLI 直接指定 Complex 路径），不需要等 OpenClaw。

但如果不提前锁定以下三点，后期会出现**"两边都以为对方会处理"的灰色地带**：

| 锁定项 | 风险 |
|--------|------|
| Complex 元数据格式 | OpenClaw 的 intent matching 没有素材 |
| 调用入口 | Agent 直接 exec 还是走 API？ |
| 错误码分层 | Agent 无法判断是重试还是换 Complex |

---

## 二、锁定契约

### 2.1 Complex 元数据格式

**SKILL.md 前置 YAML frontmatter 必须包含：**

```yaml
---
name: text-processing
description: |
  文本处理领域。当任务涉及文本转换、格式化、大写/小写、
  反转、拼接、简单字符串操作时，自动调用此 Skill。
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
---
```

**字段说明：**

| 字段 | 必须 | 说明 |
|------|------|------|
| `name` | ✅ | Complex 唯一标识 |
| `description` | ✅ | Agent intent matching 的素材 |
| `input_schema` | ✅ | JSON Schema，Agent 构造参数 |
| `output_schema` | ✅ | JSON Schema，Agent 解析结果 |

**发现接口：**

Agent 不需要理解文件系统布局。通过 COGTOME discovery API 获取：

```bash
GET /complexes
```

```json
{
  "complexes": [
    {
      "name": "text-processing",
      "description": "文本处理领域...",
      "input_schema": {...},
      "output_schema": {...}
    }
  ]
}
```

---

### 2.2 调用入口

**HTTP POST 或 Unix Socket，body 格式：**

```json
POST /invoke
{
  "complex": "text-processing",
  "params": {
    "text": "hello"
  },
  "request_id": "uuid-v4-generated-by-agent"
}
```

**说明：**
- `request_id` 由调用方（Agent）生成，原样返回，方便 Agent 关联日志
- 响应里 `execution_id` 由 COGTOME 生成，用于后续 `inspect` 追溯

**响应格式：**

```json
{
  "status": "success",
  "execution_id": "uuid-v4-by-cogtome",
  "request_id": "uuid-v4-by-agent",
  "output": {
    "upper": "HELLO",
    "reversed": "olleh"
  }
}
```

**错误响应：**

```json
{
  "status": "error",
  "execution_id": "uuid-v4-by-cogtome",
  "request_id": "uuid-v4-by-agent",
  "error": {
    "layer": "unit",
    "code": "E_EXEC",
    "message": "Unit 'text-uppercase' failed with exit code 1",
    "hint": "Check Unit binary exists and JSON output is valid"
  }
}
```

---

### 2.3 错误码分层

**三层错误模型：**

| layer | 含义 | Agent 行为 |
|-------|------|-----------|
| `runtime` | 超时、路径不存在、配置错误 | 降级或告警，不重试 |
| `motif` | 表达式解析失败、循环配置错误 | 直接报给开发者 |
| `unit` | Unit 执行失败（exit code != 0） | 可重试或换 Complex |

**Agent 决策逻辑：**

```python
if error.layer == "unit":
    # Unit 执行失败，可能是临时性问题
    # → 重试或选择其他 Complex
elif error.layer == "motif":
    # 编排逻辑自身出错，必须修复 YAML
    # → 报告给开发者
else:
    # Runtime 错误，环境问题
    # → 降级或告警
```

---

## 三、Expression Engine AST 设计

### 3.1 设计原则

Phase 1 的表达式子集足够窄，不需要完整 AST。

直接用简单 enum 建模：

```rust
enum Expression {
    // 变量引用
    Variable(String),
    
    // 索引访问
    Index(Box<Expression>, i64),  // i64 支持负索引
    
    // 属性访问
    Field(Box<Expression>, String),
    
    // 基础运算
    BinaryOp {
        op: BinOp,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    
    // 三目运算
    Conditional {
        cond: Box<Expression>,
        then: Box<Expression>,
        else_: Box<Expression>,
    },
    
    // Phase 2: 内置函数
    // Builtin(String, Vec<Expression>),
}
```

**关键设计：**
- 解析（Parse）和求值（Eval）分开
- 解析 → Expression（只检查语法）
- 求值 → 传入 Expression + 上下文，返回 Value
- Phase 2 扩展只需加 variant，不需要重构

---

## 四、Foreach 状态机设计

### 4.1 核心状态

```rust
enum ExecutionState {
    Init,           // 初始化，等待输入
    Snapshotting,   // 快照外部状态
    Iterating,     // 执行迭代
    Aggregating,    // 收集结果
    Done,           // 完成
    Failed(String), // 失败（含错误信息）
}

enum AggregatingSubState {
    Waiting,           // 等待迭代完成
    Collecting(Vec<Value>),  // 收集中
    Finalizing,        // 执行 aggregate
}
```

### 4.2 状态转移（串行）

```
Init
  │
  │ 解析 over 表达式
  ▼
Snapshotting ───────────────────────────────────┐
  │                                                  │
  │ Arc::clone(&ctx.global_steps) → O(1) 快照        │
  ▼                                                  │
Iterating ──────────────────────────────────────────►│
  │                                                  │
  │ 创建 child_ctx（snapshot + item + __index）        │
  │ 执行子 flow                                       │
  │                                                   │
  ├─► Aggregating                                    │
  │    │                                             │
  │    │ apply aggregate                              │
  ▼    ▼                                             │
  │  有更多迭代? ──► Yes ─────────────────────────────┘
  │                    │
  │                    No
  │                    ▼
  │               Done
  │
  │ (if on_error: continue)
  │ 遇到错误 → Aggregating（记录 __error）→ 继续迭代
  │
  │ (if on_error: fail_fast)
  └─► Failed("iteration N failed")
```

### 4.3 Phase 2 并行扩展

加并行时，只需要改 `Iterating` 状态的调度层：

```rust
match block.parallel {
    true => {
        // tokio::spawn 并发执行
        // 结果按索引排序
        // 状态机本身不动
    }
    false => {
        // 逐个迭代（现有逻辑）
    }
}
```

**状态机不变，调度层变** — 这就是显式状态建模的好处。

---

## 五、Phase 1 实施建议

### 5.1 顺序

```
1. Unit 路径解析（P0-3）
   └── 基础设施，让 CLI 能跑通

2. 表达式引擎基础（P0-2）
   └── 独立 crate/module，写单元测试
   └── 边界情况：空路径、负索引、缺失变量

3. foreach/aggregate 状态机
   └── 5 个状态 + aggregate 子状态
   └── 串行优先，测试覆盖 fail_fast/continue

4. max_iterations 硬限制
   └── 带 Hint 的结构化错误
```

### 5.2 测试策略

| 测试项 | 输入 | 期望 |
|--------|------|------|
| `foreach` 空数组 | `over: []` | 0 次迭代，返回空 aggregate |
| `foreach` 100 项 | `over: [1..100]`, `max: 50` | 报错 `MaxIterationsExceeded` |
| 负索引 | `${steps.a.output[-1]}` | 正常解析 |
| 变量遮蔽 | 内部 `config` vs 外部 `config` | 内部优先 |
| `fail_fast` | 迭代 3 失败 | 不产出 aggregate，直接报错 |
| `continue` | 迭代 3 失败 | 记录 `__error`，继续迭代 4/5 |

---

## 六、待补充

- [ ] COGTOME HTTP/Unix Socket Server 实现
- [ ] Discovery API `GET /complexes`
- [ ] Invoke API `POST /invoke`
- [ ] `inspect` API `GET /execution/{id}`
