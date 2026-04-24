# COGTOME 操作系统隐喻

> 整理时间：2026-04-24
> 灵感来源：学习 Agent 有感（@林中月）

---

## 核心洞察

> "操作系统花了几十年才把这些问题想清楚。
> Agent 时代正在把同样的问题重新答一遍——
> 只不过资源从 CPU/内存变成了 token/推理时间，
> '程序'从机器指令变成了自然语言。"

**历史不会重复，但会押韵。**

---

## OS → Agent 概念映射

| OS 概念 | Agent 对应 | COGTOME 实现 |
|---------|-----------|--------------|
| 进程 | Sub-Agent | Complex / Structure（资源边界） |
| 线程 | 执行单元 | Unit（原子执行体） |
| 系统调用 | Tool Use / Function Calling | Unit 的 stdin/stdout JSON 契约 |
| Cache / 虚拟内存 | Context Window | 上下文管理 + 压缩策略 |
| 文件系统挂载 | RAG | Discovery 机制（挂载 Skills 目录） |
| 内核 | Harness / Orchestrator | COGTOME Runtime |
| 调度器 | Orchestrator | Motif 引擎 |

---

## 1. 进程与线程 → Complex 与 Unit

### OS 类比

```
进程（Process）
├── 资源边界：独立的内存空间、文件句柄、网络连接
├── 线程（Thread）：进程内的执行单元，共享进程资源
└── 进程间通信（IPC）：管道、Socket、消息队列
```

### Agent 类比

```
Complex（领域典籍）
├── 资源边界：独立的 Skill 目录、Unit 集合、配置
├── Unit（齿）：Complex 内的原子执行体
└── 跨 Complex 调用：通过 Runtime IPC，禁止裸 subprocess
```

### COGTOME 实现

```rust
// Complex = 进程
struct Complex {
    name: String,
    path: PathBuf,              // 独立的资源边界
    structures: Vec<Structure>,  // 内部结构
    config: Config,
}

// Unit = 线程
// 同一个 Complex 内的 Unit 共享 discovery 缓存和配置
// 但执行是完全隔离的（fork + exec）
```

**纪律：Unit 之间绝不相互调用** — 就像同一进程的线程绝不直接访问对方栈，必须通过 IPC。

---

## 2. 系统调用 → Tool Use / Function Calling

### OS 类比

```
用户程序（User Space）
       │
       │ 想访问硬件？不行！
       ▼
  系统调用接口（System Call Interface）
       │ 陷入内核
       ▼
内核（Kernel）→ 代为执行磁盘读写、网络发送等
       │
       │ 结果返回
       ▼
用户程序（继续执行）
```

### Agent 类比

```
Agent（Model）
       │
       │ 想搜网页？不能直接动！
       ▼
  Function Calling 接口
       │ 权限检查 + 参数校验
       ▼
Harness（Runtime）→ 代为执行 fetch-url、git-diff 等
       │
       │ JSON 结果返回
       ▼
Agent（继续推理）
```

### COGTOME 实现

```
Agent
  │
  │ cogtome run text-processing --input '{"text":"hello"}'
  ▼
Runtime（内核）
  │ 权限检查：COGTOME_UNIT_MODE=1（禁止嵌套）
  │ 参数校验：input_schema 验证
  ▼
Unit 执行（fork + exec）
  │ stdin: JSON 输入
  │ stdout: JSON 输出
  ▼
结果返回
```

**本质：** 在权限边界上打一个受控的洞，能力从这个洞里流进来，风险也从这个洞里被隔住。

---

## 3. Cache / 虚拟内存 → Context Window

### OS 类比

```
寄存器（Register）：极快，容量极小，当前正在执行的指令
     │
     ▼
Cache（L1/L2/L3）：快，容量小，近期使用的数据
     │
     ▼
内存（RAM）：较慢，容量大，当前运行的程序
     │
     ▼
磁盘（Disk）：慢，容量大，不常用的数据（交换区）
```

### Agent 类比

```
当前推理（Current Token）：正在处理的上下文
     │
     ▼
近期对话（Recent Context）：当前会话的上下文窗口
     │
     ▼
压缩摘要（Compressed Summary）：历史信息的语义压缩
     │
     ▼
外部知识库（RAG）：按需检索，不占窗口
```

### COGTOME 的 Context 管理

```rust
struct ExecutionContext {
    // 寄存器级：当前正在执行的步骤
    current_step: Option<&'a Step>,

    // Cache 级：当前 Motif 的 steps（热数据）
    steps: Arc<HashMap<String, StepState>>,  // 快照，O(1) 访问

    // 内存级：当前 Structure 的 params（温数据）
    params: Value,

    // 磁盘级：历史执行日志（冷数据，按需加载）
    // ~/.cogtome/logs/YYYY-MM-DD/{execution_id}.json
}
```

**COGTOME 不直接管理 Context Window**，但它的设计遵循同样的分层原则：
- `local_steps`：热数据，执行时可零延迟访问
- `snapshot`：快照，迭代间共享但不修改
- `logs/`：冷数据，问题追溯时按需读取

---

## 4. 文件系统挂载 → RAG

### OS 类比

```
根文件系统（/）
├── /home/user/           # 本地存储
├── /mnt/external/        # 挂载的外部磁盘
├── /proc/                # 虚拟文件系统（内核状态）
└── /dev/                # 设备文件
```

### Agent 类比

```
Agent 的"文件树"
├── /context/            # 当前对话上下文
├── /memory/             # 长期记忆
├── /skills/             # 挂载的 Skills（按需加载）
└── /rag/               # RAG 知识库（检索式挂载）
```

### COGTOME 的 Discovery 机制

```rust
// Skills 目录就像挂载点
enum SkillMount {
    ComplexLocal(PathBuf),     // Complex 私有的 Skills
    Global(PathBuf),           // 全局注册的 Skills
    Remote(Registry),          // 远程 Registry（未来）
}

// 挂载点发现
fn discover_complexes(root: &Path) -> Vec<Complex> {
    // 扫描所有 SKILL.md，就像 mount -t 检查文件系统类型
    // 解析 description，建立索引
    // 供 Agent 按需"检索"
}
```

**RAG = 知识库的按需挂载，Discovery = Skills 的按需发现。**

---

## 5. 内核 / 调度器 → Harness / Orchestrator

### OS 类比

```
内核（Kernel）
├── 进程调度：决定哪个进程先跑、跑多久
├── 资源分配：CPU 时间片、内存页、文件句柄
├── 权限管理：系统调用权限检查
└── 进程间通信：信号、管道、Socket
```

### Agent 类比

```
Harness / Runtime
├── 执行调度：决定哪个 Unit 先跑、并行还是串行
├── 资源管理：超时控制、并发限制
├── 权限管理：COGTOME_UNIT_MODE=1（禁止嵌套）
└── IPC：Unit 间的结果传递
```

### COGTOME Runtime 架构

```rust
// Runtime = Agent 的操作系统内核
struct CogtomeRuntime {
    // 调度器
    scheduler: MotifEngine,

    // 资源管理
    resource_limits: ConcurrencyLimiter,

    // 权限管理
    unit_mode: UnitMode,

    // IPC 机制
    ipc: UnixSocketIPC,
}

// 调度器负责：
// - foreach 串行/并行执行
// - 错误策略（fail_fast / continue）
// - 超时控制
```

**"Agent = Model + Harness，Model 是计算本身，Harness 是操作系统内核。"**

---

## 6. 多 Agent 调度 → 分布式调度

### OS 类比

```
单机调度：
  调度器 → CPU0（进程A）
              CPU1（进程B）
              CPU2（进程C）

多核调度：
  调度器 → 核心A（线程1）
           核心B（线程2）
           ...
```

### Agent 类比

```
单 Agent：
  Orchestrator → Agent（处理任务）

多 Agent（COGTOME）：
  Runtime → Complex A（web-research）
           → Complex B（code-review）
           → Complex C（data-processing）
```

### COGTOME 的并行 foreach（Phase 2）

```rust
// Phase 2: 并行迭代
foreach:
  over: "${files}"
  as: file
  parallel: true           // 并行执行
  max_concurrency: 5      // 资源限制
  flow:
    - unit: git-diff
    - unit: ai-review

// 调度器决定：
// - 哪些迭代可以并行
// - 哪些 Unit 必须串行（未声明并发安全）
// - 资源配额如何分配
```

---

## 7. 开发启示

### OS 设计原则 → COGTOME 设计原则

| OS 原则 | COGTOME 实现 |
|---------|--------------|
| **最小权限原则** | COGTOME_UNIT_MODE=1，Unit 不能调 Unit |
| **资源边界清晰** | Complex 独立目录，Unit 独立进程 |
| **受控接口** | stdin/stdout JSON 契约，系统调用式权限控制 |
| **可观测性** | 每步执行写入 logs/，支持 inspect |
| **容错与恢复** | fail_fast / continue 策略 |
| **资源限制** | max_iterations、concurrency 声明 |

### 历史教训 → COGTOME 避免踩坑

| OS 历史教训 | COGTOME 对策 |
|------------|--------------|
| 竞争条件 | foreach 内部局部 scope，禁止修改外部状态 |
| 死锁 | 禁止嵌套调用，单向依赖链 |
| 内存泄漏 | 进程隔离，执行后回收 |
| 权限 escalation | Unit 模式锁定，不允许特权升级 |
| 资源耗尽 | max_iterations、concurrency 硬限制 |

---

## 8. 一句话总结

> **COGTOME = Agent 的微型操作系统**
>
> - **进程** = Complex（资源边界）
> - **线程** = Unit（原子执行）
> - **系统调用** = Function Calling（受控能力）
> - **上下文窗口** = Memory Hierarchy（分层管理）
> - **RAG** = 文件系统挂载（按需加载）
> - **Runtime** = 内核（权限、调度、IPC）
> - **Motif** = 调度策略（串行/并行/条件）

---

*文档版本：v1 | 日期：2026-04-24*
