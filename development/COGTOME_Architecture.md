# COGTOME — Agent 专用运行时架构白皮书

> **齿轮转动典籍，机械执行技艺。**
>
> COGTOME 是 Agent 的微型操作系统。
> **Cog**（齿轮）代表精确的原子执行（Unit），**Tome**（典籍）代表收录技艺的领域之门（Complex）。
> Agent 吟唱意图，COGTOME 翻找典籍、组装齿轮、驱动执行。

---

## 一、定位与核心认知

### 1.1 COGTOME 是什么

COGTOME 不是框架，不是库，而是一个**独立的进程级运行时**——Agent 的微型操作系统。

| 类比 | 对应关系 |
|------|---------|
| 操作系统内核 | COGTOME Runtime (Rust) |
| 用户进程 | Agent (LLM / 程序) |
| 系统调用 (syscall) | Unit（齿轮齿，原子执行） |
| 用户态函数 | Motif（齿轮组，Unit 编排） |
| 应用程序 | Structure（传动机构，业务封装） |
| 应用商店 / 包管理器 | Complex（典籍，领域门面） |
| shell / 交互界面 | `cogtome` CLI |

Agent 不再直接 `import` 库或 `subprocess` 调用脚本。Agent 通过 CLI 或 IPC 向 COGTOME 发送**意图**，COGTOME 完成发现、匹配、调度、执行、回收的全生命周期管理。

### 1.2 品牌语汇

| 技术术语 | COGTOME 品牌隐喻 | 说明 |
|---------|----------------|------|
| Unit | 齿 (Cog/Tooth) | 齿轮不可再分的最小单元 |
| Motif | 齿轮组 (Gear Assembly) | 齿的编排与组合 |
| Structure | 传动机构 (Drive Train) | 完成特定目标的机械结构 |
| Complex | 典籍 (Tome) | 收录传动机构的领域之书 |
| Skill | 技艺 | 用户侧仍可用 Skill 指代 Complex |
| 定义 Unit | 铸造 (Forge) | 将齿铸造出来 |
| 执行 Unit | 啮合 (Engage) | 齿与齿咬合转动 |
| 编排 Motif | 组装 (Assemble) | 将齿轮组装为传动组 |
| Agent 意图 | 指令 (Command) | 自然语言即为操作指令 |
| Execution Plan | 蓝图 (Blueprint) | 编译后的精确执行步骤 |

### 1.3 关键原则

1. **Unit 即 syscall**：每个 Unit 是独立的进程执行（fork + exec），不可再分。
2. **Motif 即用户态代码**：编排 Unit 的逻辑自由，但**必须通过 COGTOME 调度器**调用 Unit，禁止裸 `subprocess`。
3. **Structure 即应用**：完成一个具体业务目标的黑盒，对外只暴露输入输出 Schema。
4. **Complex 即门面**：Agent 唯一可见的层，持有 `description`，参与自动发现。
5. **所有跨层调用通过 COGTOME IPC**：Motif 中的 `unit.call()` 实际通过 Unix Socket / gRPC 与 Rust Runtime 通信。

---

## 二、四层架构的严格定义

### 2.1 总览

| 层级 | 名称 | Agent 可见？ | 本质 | 一句话定义 |
|------|------|-------------|------|-----------|
| **L4** | **Complex** | ✅ 唯一可见 | 领域门面 | 原 Skill，有 `description`，自动发现扫描目标 |
| **L3** | **Structure** | ❌ 不可见 | 业务结构 | 完成具体目标的内部实现，被 Complex 按需加载 |
| **L2** | **Motif** | ❌ 不可见 | 工作链 | 编排 Unit 的内部逻辑，实现方式自由 |
| **L1** | **Unit** | ❌ 不可见 | 原子执行体 | 固定 CLI，可执行的最小一步 |

### 2.2 核心纪律（Runtime 强制，不可违反）

1. **Unit 之间绝不相互调用**。Runtime 通过环境变量 `COGTOME_UNIT_MODE=1` 阻止自调用。
2. **Motif 之间不直接相互调用**（建议通过 Structure 组合）。
3. **Structure 不直接调用 Unit**，必须通过 Motif 编排。
4. **Complex 是唯一有 `description` 的层**，只有它参与 COGTOME 的自动发现。
5. **所有跨层调用必须通过 COGTOME IPC**，禁止绕过。

---

## 三、COGTOME Runtime 架构（Rust）

### 3.1 模块结构

```
cogtome/
├── Cargo.toml
└── src/
    ├── main.rs                 // CLI 入口 (clap)
    ├── commands/               // 子命令实现
    │   ├── skill.rs            // `cogtome skill *`
    │   ├── unit.rs             // `cogtome unit *`
    │   ├── motif.rs            // `cogtome motif *`
    │   ├── run.rs              // `cogtome run`
    │   ├── inspect.rs          // `cogtome inspect`
    │   ├── logs.rs             // `cogtome logs`
    │   ├── validate.rs         // `cogtome validate`
    │   └── daemon.rs           // `cogtome daemon *`
    ├── core/
    │   ├── discovery.rs        // Skill 发现与元数据缓存
    │   ├── resolver.rs         // Complex → Structure 选择器
    │   ├── scheduler.rs        // Execution Plan 编译与调度
    │   ├── unit_runner.rs      // Unit 进程管理 (tokio::process)
    │   ├── motif_engine.rs     // 多语言 Motif 执行引擎
    │   ├── resource_mgr.rs     // 资源生命周期 (RAII + WAL)
    │   ├── sandbox.rs          // 沙箱 (landlock/seccomp)
    │   └── logger.rs           // tracing-based 结构化日志
    ├── ipc/                    // 进程间通信
    │   ├── server.rs           // Daemon Unix Socket / HTTP 服务
    │   └── protocol.rs         // JSON-RPC / gRPC 定义
    ├── models/                 // 核心数据结构
    │   ├── unit.rs
    │   ├── motif.rs
    │   ├── structure.rs
    │   ├── complex.rs
    │   ├── context.rs          // ExecutionContext
    │   └── plan.rs             // ExecutionPlan / ExecutionStep
    └── utils/
        ├── schema.rs           // JSON Schema 校验
        └── paths.rs            // 路径解析 (~/.agents/skills/)
```

### 3.2 核心 Trait 定义

```rust
// models/unit.rs
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitSpec {
    pub name: String,
    pub version: semver::Version,
    pub input_schema: serde_json::Value,   // JSON Schema
    pub output_schema: serde_json::Value,  // JSON Schema
    pub exit_codes: std::collections::HashMap<u8, ExitCodeInfo>,
    pub binary_path: PathBuf,
    pub resource: Option<ResourceSpec>,    // 若为资源型 Unit
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitCodeInfo {
    pub message: String,
    pub retryable: bool,
    pub suggest: Option<String>,
    pub auto_fix: Option<String>, // 预留：自动修复脚本路径
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub side_effects: Vec<String>,      // e.g. ["spawn_daemon"]
    pub cleanup_unit: String,           // e.g. "browser-end"
    pub cleanup_on: Vec<CleanupTrigger>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanupTrigger {
    Exit,
    Error,
    Timeout,
    Panic,
}
```

```rust
// models/context.rs
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionContext {
    pub execution_id: Uuid,
    pub trace_id: Uuid,                 // 分布式追踪，嵌套调用共享
    pub span_id: Uuid,                  // 当前调用层级标识
    pub parent_span_id: Option<Uuid>,
    pub start_time: DateTime<Utc>,
    pub log_dir: PathBuf,               // ~/tmp/cogtome-logs/YYYY-MM-DD/
    pub log_level: String,
    pub timeout_remaining_ms: u64,      // 剩余超时，层层递减
    pub metadata: serde_json::Value,    // 跨层共享状态（浅拷贝传递）
}

impl ExecutionContext {
    pub fn child(&self, name: &str) -> Self {
        Self {
            execution_id: self.execution_id,
            trace_id: self.trace_id,
            span_id: Uuid::new_v4(),
            parent_span_id: Some(self.span_id),
            start_time: self.start_time,
            log_dir: self.log_dir.clone(),
            log_level: self.log_level.clone(),
            timeout_remaining_ms: self.timeout_remaining_ms, // 或由 scheduler 分配子预算
            metadata: self.metadata.clone(),
        }
    }
}
```

```rust
// models/plan.rs
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct ExecutionPlan {
    pub execution_id: Uuid,
    pub complex_name: String,
    pub structure_name: String,
    pub steps: Vec<ExecutionStep>,
}

#[derive(Debug, Serialize)]
pub enum ExecutionStep {
    UnitCall {
        name: String,
        input: serde_json::Value,
        parallel_group: Option<Uuid>,   // 同 group 的 Unit 并行执行
        timeout_ms: u64,
    },
    UnitGather {
        group_id: Uuid,                 // 等待 parallel_group 完成
    },
    MotifEnter {
        name: String,
    },
    MotifExit,
    ResourceAcquire {
        unit: String,
        params: serde_json::Value,
        handle_id: Uuid,
    },
    ResourceRelease {
        handle_id: Uuid,
    },
    ValidateOutput {
        schema: serde_json::Value,
    },
}
```

```rust
// core/scheduler.rs
use crate::models::*;

#[async_trait::async_trait]
pub trait Scheduler {
    /// 将 Agent 请求编译为执行计划
    async fn compile_plan(
        &self,
        intent: &str,
        params: serde_json::Value,
        constraints: Option<serde_json::Value>,
    ) -> Result<ExecutionPlan, ResolveError>;

    /// 按 ExecutionPlan 逐步执行，维护状态机
    async fn execute_plan(
        &self,
        plan: ExecutionPlan,
        ctx: ExecutionContext,
    ) -> Result<ExecutionResult, ExecutionError>;
}

#[derive(Debug)]
pub struct ExecutionResult {
    pub data: serde_json::Value,
    pub exit_code: u8,
    pub duration_ms: u64,
    pub logs: Vec<LogEntry>,
}
```

```rust
// core/unit_runner.rs
use crate::models::*;
use tokio::process::Command;

#[async_trait::async_trait]
pub trait UnitRunner {
    /// 调用无状态 Unit
    async fn call(
        &self,
        spec: &UnitSpec,
        input: serde_json::Value,
        ctx: &ExecutionContext,
    ) -> Result<UnitResult, UnitError>;

    /// 并行调用多个 Unit
    async fn gather(
        &self,
        calls: Vec<(&UnitSpec, serde_json::Value)>,
        ctx: &ExecutionContext,
    ) -> Result<Vec<UnitResult>, UnitError>;
}

#[derive(Debug)]
pub struct UnitResult {
    pub data: serde_json::Value,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: u8,
    pub duration_ms: u64,
}
```

```rust
// core/resource_mgr.rs
use crate::models::*;
use std::sync::Arc;

#[async_trait::async_trait]
pub trait ResourceManager: Send + Sync {
    async fn acquire(
        &self,
        spec: &UnitSpec,
        params: serde_json::Value,
        ctx: &ExecutionContext,
    ) -> Result<ResourceGuard, ResourceError>;

    /// 崩溃恢复：扫描 WAL 残留资源并强制清理
    async fn recover(&self) -> Result<u32, ResourceError>;
}

/// RAII 资源句柄，Drop 时自动触发 cleanup_unit
pub struct ResourceGuard {
    pub handle_id: Uuid,
    pub session_data: serde_json::Value,
    mgr: Arc<dyn ResourceManager>,
    ctx: ExecutionContext,
}

impl Drop for ResourceGuard {
    fn drop(&mut self) {
        // 通过 tokio Handle 在异步运行时中执行清理
        if let Ok(rt) = tokio::runtime::Handle::try_current() {
            let mgr = self.mgr.clone();
            let handle_id = self.handle_id;
            let ctx = self.ctx.clone();
            rt.spawn(async move {
                let _ = mgr.release(handle_id, &ctx).await;
            });
        }
    }
}
```

---

## 四、CLI 命令设计

### 4.1 用户层命令

```bash
# === 发现与浏览 ===
cogtome skill list                              # 列出所有 Complex（带描述）
cogtome skill show web-automation               # 查看 Complex 详情与 Structure 列表
cogtome skill search "浏览器自动化"              # 模糊搜索 description

# === 执行 ===
cogtome run web-automation \
    --input '{"url":"https://example.com"}' \
    --constraints '{"webgl":true}'

cogtome run web-automation -i                   # 交互式输入 JSON
cogtome run --dry-run web-automation --input '{}' # 编译执行计划但不执行

# === 调试层命令（开发者工具）===
cogtome unit list                               # 列出全局 + 私有 Units
cogtome unit show text-uppercase                # 查看 Unit 契约
cogtome unit run text-uppercase --input '{"text":"hello"}'
cogtome unit run text-uppercase --stdin         # 从 stdin 读入

cogtome motif list
cogtome motif run text-transform --input '{"text":"hello"}'

cogtome structure list
cogtome structure validate text-pipeline        # 校验 manifest 与依赖完整性

# === 执行检查与日志 ===
cogtome logs                                    # 列出今日执行摘要
cogtome logs --date 2026-04-23                  # 查看历史
cogtome logs --follow                           # 实时跟踪最新执行
cogtome inspect 001432                          # 查看单次执行完整链路
cogtome inspect 001432 --tree                   # 树形展示四层调用关系
cogtome inspect 001432 --unit text-uppercase    # 过滤特定 Unit 调用

# === 系统管理 ===
cogtome validate                                # 校验 ~/.agents/skills/ 下所有 Skill
cogtome validate --fix                          # 自动修复常见问题（如权限、格式）
cogtome daemon start                            # 启动常驻进程
cogtome daemon stop
cogtome daemon status                           # 查看运行状态与缓存统计
cogtome daemon reload                           # 热重载元数据缓存

# === 打包与分发（未来）===
cogtome pack ./my-skill/                        # 打包为 .cogtome 文件
cogtome install my-skill.cogtome                # 安装 Skill
cogtome registry search browser                 # 搜索中央仓库
```

### 4.2 Daemon 模式

```bash
cogtome daemon start --socket /tmp/cogtome.sock --http 127.0.0.1:9842
```

Daemon 常驻后提供：

| 能力 | 说明 |
|------|------|
| **元数据缓存** | 避免每次 CLI 调用扫描磁盘；`cogtome daemon reload` 热更新 |
| **Unit 进程池** | 高频 Unit（如文本处理）保持进程预热，避免反复 fork |
| **资源守护** | 浏览器 session 等长生命周期资源由 Daemon 持有，Client 断线不泄漏 |
| **HTTP API** | `POST /v1/run` 供远程 Agent 调用；`GET /v1/skills` 供发现 |
| **全局并发控制** | `max_concurrent` 在 Daemon 层生效，而非每个 Complex 独立 |

---

## 五、多语言 Motif 引擎

### 5.1 执行策略矩阵

| Motif 类型 | 文件扩展名 | 执行方式 | Runtime 支持 |
|-----------|-----------|---------|-------------|
| **Python Motif** | `.py` | 子进程 + IPC → COGTOME Daemon | ✅ Phase 1 |
| **Shell Motif** | `.sh` | `tokio::process::Command` 直接执行 | ✅ Phase 1 |
| **YAML Motif** | `.yaml` / `.yml` | Rust 原生解析执行（最高性能） | ✅ Phase 2 |
| **Rust Motif** | `.rs` (编译为 `.so`) | `libloading` 动态加载 | 🔮 Phase 4 |

### 5.2 Python Motif 的 IPC 机制（关键设计）

Python SDK 不再直接 `subprocess.run`，而是作为 **COGTOME Daemon 的 IPC 客户端**。

```python
# agents_sdk/unit.py (Python SDK)
import socket
import json
import os
from dataclasses import dataclass

@dataclass
class UnitResult:
    data: dict
    exit_code: int
    stdout: str
    stderr: str
    duration_ms: int

class CogtomeClient:
    """Python SDK 核心：通过 Unix Domain Socket 与 COGTOME Daemon 通信"""
    
    def __init__(self, socket_path: str = None):
        self.socket_path = socket_path or os.environ.get(
            "COGTOME_SOCKET", 
            "/tmp/cogtome.sock"
        )
    
    def _call(self, method: str, params: dict) -> dict:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.connect(self.socket_path)
        
        payload = {
            "jsonrpc": "2.0",
            "id": os.urandom(4).hex(),
            "method": method,
            "params": params
        }
        sock.send(json.dumps(payload).encode() + b"\n")
        
        response = json.loads(sock.recv(65536).decode().strip())
        sock.close()
        
        if "error" in response:
            raise RuntimeError(response["error"])
        return response["result"]

# 全局客户端实例
_default_client = CogtomeClient()

def call(name: str, input_data: dict, ctx=None) -> UnitResult:
    result = _default_client._call("unit.call", {
        "name": name,
        "input": input_data,
        "ctx": ctx.to_dict() if ctx else {}
    })
    return UnitResult(**result)

@contextmanager
def resource(name: str, params: dict, ctx=None):
    """资源型 Unit 上下文管理器"""
    handle = _default_client._call("resource.acquire", {
        "name": name,
        "params": params,
        "ctx": ctx.to_dict() if ctx else {}
    })
    try:
        yield ResourceHandle(handle)
    finally:
        _default_client._call("resource.release", {
            "handle_id": handle["handle_id"]
        })
```

```python
# agents_sdk/motif.py
from .unit import call, resource

class Motif:
    name: str = ""
    units_required: list[str] = []
    
    def run(self, ctx, **kwargs):
        raise NotImplementedError

class TextTransformMotif(Motif):
    name = "text-transform"
    units_required = ["text-uppercase", "text-reverse"]
    
    def run(self, ctx, text: str) -> dict:
        # 实际通过 IPC 调用 COGTOME Daemon
        r1 = call("text-uppercase", {"text": text}, ctx=ctx)
        r2 = call("text-reverse", {"text": text}, ctx=ctx)
        return {
            "upper": r1.data["result"],
            "reversed": r2.data["result"],
            "combined": f"{r1.data['result']} | {r2.data['result']}"
        }
```

### 5.3 YAML Motif 引擎（Rust 原生）

```yaml
# motifs/data-pipeline.yaml
name: data-pipeline
type: motif
units_required: [fetch-url, parse-json, validate-schema]

# 声明式 DAG
flow:
  - step: fetch
    unit: fetch-url
    input:
      url: "${params.url}"
    output: raw_data
    timeout: 30

  - step: parse
    unit: parse-json
    input:
      text: "${steps.fetch.output.raw_data}"
    output: json_obj

  - step: validate
    unit: validate-schema
    input:
      data: "${steps.parse.output.json_obj}"
      schema: "${params.schema}"
    output: validated

  # 条件分支
  - step: fallback
    unit: fetch-url
    condition: "${steps.validate.exit_code != 0}"
    input:
      url: "${params.fallback_url}"
    output: raw_data

  # 并行组
  - step: analyze_a
    unit: sentiment-analysis
    parallel: group_1
    input:
      text: "${steps.parse.output.json_obj.content}"
    output: sentiment

  - step: analyze_b
    unit: keyword-extract
    parallel: group_1
    input:
      text: "${steps.parse.output.json_obj.content}"
    output: keywords

  - step: merge
    unit: merge-results
    after: group_1
    input:
      sentiment: "${steps.analyze_a.output.sentiment}"
      keywords: "${steps.analyze_b.output.keywords}"
    output: final

return:
  data: "${steps.merge.output.final}"
  success: "${steps.validate.exit_code == 0}"
```

Rust 引擎解析 `${...}` 表达式，支持三个作用域：`params`、`steps`、`env`。

---

## 六、执行模型：从意图到结果

### 6.1 完整执行流程

```
Agent Query
    │
    ▼
┌──────────────────────────────────────────┐
│  1. Discovery (发现)                      │
│     扫描 ~/.agents/skills/*/SKILL.md      │
│     构建 Complex 索引 (缓存于 Daemon)      │
└────────────┬─────────────────────────────┘
             │ 返回 Complex 候选列表
             ▼
┌──────────────────────────────────────────┐
│  2. Resolution (解析)                     │
│     描述相似度 / 约束匹配 / 权重排序        │
│     Complex.select_structure()            │
└────────────┬─────────────────────────────┘
             │ 返回选中的 Structure
             ▼
┌──────────────────────────────────────────┐
│  3. Compilation (编译)                    │
│     Structure manifest → ExecutionPlan    │
│     静态检查：Unit/Motif 依赖完整性         │
│     Schema 预校验                         │
└────────────┬─────────────────────────────┘
             │ 返回 ExecutionPlan
             ▼
┌──────────────────────────────────────────┐
│  4. Scheduling (调度)                     │
│     按 ExecutionStep 顺序执行              │
│     • 串行步骤：阻塞执行                   │
│     • 并行组：tokio::spawn 并发           │
│     • 资源：acquire → use → release       │
│     • 超时：tokio::time::timeout          │
└────────────┬─────────────────────────────┘
             │ 返回 ExecutionResult
             ▼
┌──────────────────────────────────────────┐
│  5. Validation (验证)                     │
│     校验输出是否符合 output_schema        │
│     写日志索引 (index.json)               │
└────────────┬─────────────────────────────┘
             │
             ▼
        Agent 看到结果
```

### 6.2 ExecutionPlan 的 WAL（崩溃恢复）

Daemon 在执行计划前写入 WAL：

```rust
// WAL 条目
enum WalEntry {
    PlanStart { execution_id: Uuid, plan: ExecutionPlan },
    StepStart { execution_id: Uuid, step_idx: usize },
    StepComplete { execution_id: Uuid, step_idx: usize, result: serde_json::Value },
    ResourceAcquired { execution_id: Uuid, handle_id: Uuid, cleanup_unit: String },
    ResourceReleased { execution_id: Uuid, handle_id: Uuid },
    PlanComplete { execution_id: Uuid, final_result: serde_json::Value },
}
```

Daemon 启动时扫描 WAL：
- 发现 `PlanStart` 但无 `PlanComplete` → 中断的执行，根据已完成步骤决定回滚或续跑
- 发现 `ResourceAcquired` 但无 `ResourceReleased` → 残留资源，触发强制 cleanup

---

## 七、安全与沙箱

### 7.1 多层安全模型

| 层级 | 机制 | 作用 |
|------|------|------|
| **L1 环境隔离** | `COGTOME_UNIT_MODE=1` | 阻止 Unit 内部直接调用其他 Unit |
| **L2 文件系统隔离** | Linux Landlock | Unit 只能访问声明的目录（如自己的 `bin/`、`/tmp`） |
| **L3 系统调用过滤** | Linux seccomp-bpf | 阻止危险的 syscall（`execve`, `ptrace`, `mount` 等） |
| **L4 资源限制** | cgroups v2 | 限制内存、CPU、IO、网络带宽 |
| **L5 网络隔离** | network namespace | 无网络型 Unit 完全断网 |

### 7.2 资源型 Unit 的显式授权

Complex 的 `config.yaml` 必须显式声明授权：

```yaml
# ~/.agents/skills/web-automation/config.yaml
permissions:
  allow_daemon: true        # 允许启动后台进程
  allow_network: true       # 允许网络访问
  allow_gpu: false
  
  # 文件系统白名单（Landlock）
  fs_allow:
    - "~/.cache/cogtome/browser/"
    - "/tmp/"
```

未声明的权限默认 **拒绝**。

---

## 八、日志与可观测性

### 8.1 目录结构

```
~/tmp/cogtome-logs/
├── 2026-04-24/                          # 按天分组
│   ├── 143021_exec_001432/              # 单次执行目录
│   │   ├── index.json                   # 索引：关联所有层级
│   │   ├── complex_web-automation.log   # JSON Lines
│   │   ├── structure_text-pipeline.log
│   │   ├── motif_text-transform.log
│   │   ├── unit_text-uppercase_7a3f.log
│   │   └── unit_text-reverse_8b2e.log
│   └── 143022_exec_001433/
│       └── ...
├── archive/
│   └── 2026-04-24_summary.db            # SQLite 聚合（保留 30 天）
└── wal/                                 # 崩溃恢复 WAL
    └── active.wal
```

### 8.2 统一日志格式（JSON Lines）

```json
// unit_text-uppercase_7a3f.log
{"ts":"2026-04-24T14:30:21.234Z","level":"INFO","event":"start","unit":"text-uppercase","input":{"text":"hello"},"span_id":"a1b2c3d4"}
{"ts":"2026-04-24T14:30:21.235Z","level":"INFO","event":"exec","pid":12345,"cmd":"/home/user/.agents/skills/units/text-uppercase/bin/text-uppercase"}
{"ts":"2026-04-24T14:30:21.240Z","level":"INFO","event":"stdout","data":{"result":"HELLO"}}
{"ts":"2026-04-24T14:30:21.241Z","level":"INFO","event":"end","exit_code":0,"duration_ms":6}
```

```json
// index.json
{
  "execution_id": "001432",
  "trace_id": "550e8400-e29b-41d4-a716-446655440000",
  "timestamp": "2026-04-24T14:30:21.200Z",
  "complex": {"name": "web-automation", "log": "complex_web-automation.log"},
  "structure": {"name": "text-pipeline", "log": "structure_text-pipeline.log"},
  "motifs": [
    {
      "name": "text-transform",
      "log": "motif_text-transform.log",
      "units": [
        {"name": "text-uppercase", "log": "unit_text-uppercase_7a3f.log", "hash": "7a3f"},
        {"name": "text-reverse", "log": "unit_text-reverse_8b2e.log", "hash": "8b2e"}
      ]
    }
  ],
  "summary": {"status": "success", "duration_ms": 49, "error": null}
}
```

### 8.3 `cogtome inspect` 输出示例

```bash
$ cogtome inspect 001432 --tree

Execution 001432
├── [Complex] web-automation (3.2ms)
│   └── [Structure] text-pipeline (45.1ms)
│       └── [Motif] text-transform (42.0ms)
│           ├── [Unit] text-uppercase ──► {"result":"HELLO"} (6.1ms) ✅
│           ├── [Unit] text-reverse ──► {"result":"olleh"} (5.8ms) ✅
│           └── [Return] {"upper":"HELLO","reversed":"olleh",...}
└── Status: SUCCESS (49ms total)
```

---

## 九、目录结构规范

```
~/.agents/
├── skills/                              # Skill 根目录
│   ├── <complex-name>/                  # Complex = 典籍（领域门面）
│   │   ├── SKILL.md                     # 必须有 description，自动发现目标
│   │   ├── complex.py                   # Complex 类实现（可选，可用 manifest 替代）
│   │   ├── config.yaml                  # Complex 级配置 + 权限声明
│   │   ├── structures/                  # 私有 Structure
│   │   │   └── <name>/
│   │   │       ├── manifest.yaml
│   │   │       └── structure.py         # 可选自定义执行器
│   │   └── motifs/                      # 私有 Motif
│   │       └── <name>.{py,sh,yaml}
│   │
│   ├── structures/                      # 全局共享 Structure（可选）
│   │   └── <name>/
│   │       └── manifest.yaml
│   │
│   ├── motifs/                          # 全局共享 Motif（可选）
│   │   └── <name>.{py,sh,yaml}
│   │
│   └── units/                           # 全局 Unit（齿轮齿）
│       └── <unit-name>/
│           ├── SKILL.md                 # CLI 契约（无 description）
│           ├── errors.py                # 错误模式库（可选）
│           └── bin/<unit-name>          # 可执行入口（chmod +x）
│
└── .sdk/                                # Python SDK（pip install 时安装）
    └── agents_sdk/
        ├── __init__.py
        ├── unit.py                      # IPC 客户端
        ├── motif.py
        ├── context.py
        └── errors.py

~/tmp/cogtome-logs/                      # 运行时日志
├── YYYY-MM-DD/
│   ├── HHMMSS_exec_<id>/
│   │   ├── index.json
│   │   ├── *_complex_*.log
│   │   ├── *_structure_*.log
│   │   ├── *_motif_*.log
│   │   └── *_unit_*_<hash>.log
│   └── ...
├── archive/
│   └── YYYY-MM-DD_summary.db
└── wal/
    └── active.wal
```

---

## 十、实施路线图

### Phase 1：COGTOME Core MVP（3 周）

**目标**：`cogtome run` 能执行文档中的 text-processing 示例。

- [x] Cargo workspace 搭建
- [x] CLI 框架（clap）：`cogtome skill list`, `cogtome run`, `cogtome unit run`
- [x] Discovery：扫描 `~/.agents/skills/*/SKILL.md`
- [x] UnitRunner：`tokio::process` + stdin/stdout JSON + 超时控制
- [x] Logger：tracing subscriber + JSON Lines 写入 `~/tmp/cogtome-logs/`
- [x] Python SDK IPC 客户端（Unix Socket）
- [x] Python Motif 引擎（子进程 + IPC）
- [x] Structure manifest 解析 + 默认执行器
- [x] Complex 描述匹配（关键词 BM25）

**验证命令**：
```bash
cogtome skill list
cogtome run text-processing --input '{"text":"hello"}'
cogtome unit run text-uppercase --input '{"text":"hello"}'
cogtome inspect <execution-id> --tree
```

### Phase 2：Daemon 与并发（2 周）

- [x] `cogtome daemon start/stop/status`
- [x] Unix Socket + HTTP API (`/v1/run`, `/v1/skills`)
- [x] 元数据缓存与热重载
- [x] Unit 进程预热池
- [x] 并行 Unit 调用（`unit.gather()`）
- [x] YAML Motif 原生引擎（Rust 实现）
- [x] `cogtome validate` + `--fix`

### Phase 3：资源管理与安全（2 周）

- [x] 资源型 Unit：`resource.acquire/release` + RAII Guard
- [x] WAL 崩溃恢复机制
- [x] Linux Landlock 文件系统隔离
- [x] seccomp-bpf 系统调用过滤
- [x] cgroups v2 资源限制（内存、CPU）
- [x] `cogtome logs --follow`

### Phase 4：生态与优化（持续）

- [x] `cogtome pack/install` 打包分发
- [x] Registry / 中央仓库协议
- [x] Rust Motif 动态加载（`.so`）
- [x] Web UI 监控面板
- [x] 性能基准测试与优化

---

**文档版本**：v1.0-COGTOME
**适用架构**：Unit-Motif-Structure-Complex 四层模型 + COGTOME Rust Runtime
**核心原则**：Unit 原子化、Motif 自由编排、Structure 黑盒复用、Complex 唯一门面、COGTOME Runtime 强约束跨层调用、日志日清复盘。
