# COGTOME MCP Server 设计文档

> COGTOME 作为本地 MCP Server，对外暴露 Assembly（Skill）作为可调用工具。

---

## 1. 核心理念

```
Agent (Claude/Cursor)
    ↓  MCP stdio (JSON-RPC 2.0)
COGTOME MCP Server
    ↓  复用现有引擎
Assembly → Motif → Unit
```

**对外无状态，内部可增强。**

---

## 2. 目录结构

### 2.1 改名：skills/ → assemblies/

```
assemblies/                          # 对外暴露的复合能力
└── text-processing/
    ├── manifest.json                # 机器权威：name/description/schema
    ├── SKILL.md                     # 人类文档（无 front-matter）
    └── workflow.json                # 可选：Motif DAG 定义

units/                               # 全局可复用原子执行器
├── text-uppercase/
│   ├── bin/
│   │   └── text-uppercase          # 可执行文件
│   └── unit.json                   # 元数据
└── file-read/
    ├── bin/
    └── unit.json
```

### 2.2 单元解析规则

`unit.json` 定义 Unit 元数据：

```json
{
  "name": "text-uppercase",
  "entry": "./bin/text-uppercase",
  "input_schema": { ... },
  "output_schema": { ... }
}
```

**解析规则：**
- Unit 名称 → `units/<name>/unit.json`
- 可执行文件路径 → `units/<name>/<entry>`
- `entry` 是相对 `unit.json` 所在目录的路径

**Motif 中的引用：**
```json
{
  "type": "unit",
  "id": "step1",
  "unit": "text-uppercase",
  "input": { "text": "${params.text}" }
}
```

---

## 3. manifest.json 结构

```json
{
  "name": "text-processing",
  "description": "Process and transform text content",
  "version": "1.0.0",
  "tags": ["text", "productivity"],
  "category": "productivity",
  "input_schema": {
    "type": "object",
    "properties": {
      "text": { "type": "string" }
    },
    "required": ["text"]
  },
  "output_schema": {
    "type": "object",
    "properties": {
      "result": { "type": "string" }
    }
  },
  "units": ["text-uppercase"],
  "workflow": "./workflow.json"
}
```

**字段说明：**
- `name`: 工具名称（唯一标识）
- `description`: 工具描述（LLM 理解用）
- `input_schema`: JSON Schema（用于 `tools/list` 响应）
- `output_schema`: JSON Schema（内部校验用，不暴露给 Host）
- `units`: 依赖的 Unit 列表（用于 discovery 和校验）
- `workflow`: Motif DAG 定义路径，支持：
  - `"./workflow.json"` — 同目录下的文件
  - `"builtin:linear"` — 内置线性模板（未来）
- `tags`/`category`: 预留字段，用于未来 Host 按标签过滤

---

## 4. MCP 接口设计

### 4.1 tools/list

**请求：**
```json
{ "method": "tools/list", "params": {} }
```

**响应：**
```json
{
  "tools": [
    {
      "name": "text-processing",
      "description": "Process and transform text content",
      "inputSchema": {
        "type": "object",
        "properties": {
          "text": { "type": "string" }
        },
        "required": ["text"]
      }
    }
  ]
}
```

**注意：** 不暴露 `outputSchema`（MCP 标准不支持，Host 会忽略或报错）

### 4.2 tools/call

**请求：**
```json
{
  "method": "tools/call",
  "params": {
    "name": "text-processing",
    "arguments": { "text": "hello" }
  }
}
```

**成功响应：**
```json
{
  "content": [{ "type": "text", "text": "{\"result\":\"HELLO\"}" }]
}
```

**业务错误响应：**
```json
{
  "content": [{ "type": "text", "text": "Error: file not found: /tmp/data.csv" }],
  "isError": true
}
```

**协议错误响应：**
```json
{
  "error": {
    "code": -32602,
    "message": "Invalid params",
    "data": { "detail": "Assembly 'nonexistent' not found" }
  }
}
```

### 4.3 错误分层

| 场景 | 响应方式 | Host 行为 |
|------|----------|----------|
| Assembly 不存在 | JSON-RPC `error` (`-32602`) | 报"工具调用失败" |
| 参数格式错误 | JSON-RPC `error` (`-32602`) | 报"参数错误" |
| 执行失败（文件不存在等） | `result` + `isError: true` | 把错误喂回 LLM 决策 |
| 超时 | `result` + `isError: true` | 把错误喂回 LLM 决策 |

---

## 5. Motif 引擎增强

### 5.1 节点纠错（Error Correction）

在 `workflow.json` 的节点级别配置纠错策略，Motif 引擎按优先级执行：

```json
{
  "id": "fetch_data",
  "type": "unit",
  "unit": "web-fetch",
  "input": { "url": "${params.url}" },
  "config": {
    "retry": {
      "max_attempts": 3,
      "backoff": "exponential",
      "delay_ms": 1000,
      "on": ["timeout", "connection_error"]
    },
    "fallback": {
      "node": "fetch_cache",
      "on": ["timeout"]
    },
    "compensate": "log_failure",
    "circuit_breaker": {
      "failure_threshold": 5,
      "recovery_timeout_ms": 30000
    }
  }
}
```

**纠错策略优先级（引擎自动处理）：**

| 策略 | 触发条件 | 行为 |
|------|----------|------|
| 输入消毒 | 每次执行前 | Schema 校验 + 数据清洗（防注入、截断） |
| 重试 (Retry) | 执行失败且错误码在 `on` 列表 | 指数退避重试，成功则继续 DAG |
| 降级 (Fallback) | 重试耗尽 | 跳转到备用节点，输出兼容原节点 schema |
| 补偿 (Compensate) | 最终仍失败 | 执行补偿节点（如删除脏文件、回滚事务），然后抛错 |
| 断路器 (Circuit Breaker) | 单位时间失败 N 次 | 后续调用直接走 fallback，避免雪崩 |

**关键设计：所有纠错动作对下游节点透明。** 下游节点只看到"成功 + 干净输出"，不知道上游经历过重试或降级。

### 5.2 节点自净化（Self-Cleaning）

自净化分三个阶段，由 Runtime 强制执行，Unit 代码无需关心：

**阶段 1：执行前净化（Pre-clean）**

Runtime 在 `fork` 之后、`exec` 之前完成：

```rust
fn pre_clean(node: &Node, ctx: &ExecutionContext) {
    // 1. 隔离工作目录
    let workdir = format!("/tmp/cogtome/{}/{}", ctx.execution_id, node.id);
    fs::create_dir_all(&workdir);
    env::set_current_dir(&workdir);

    // 2. 输入消毒
    let sanitized = sanitize_json(node.input);  // 去除控制字符、路径遍历防护
    env::set_var("COGTOME_INPUT", sanitized.to_string());

    // 3. 资源预检
    assert_disk_space(min_mb: 100);
    assert_memory_limit(max_mb: node.config.memory_limit);
}
```

**阶段 2：执行后净化（Post-clean）**

Unit 进程退出后，Runtime 立即执行：

```rust
fn post_clean(node: &Node, result: &ExecutionResult) {
    // 1. 输出校验（必须匹配 output_schema）
    validate_schema(result.output, node.output_schema);

    // 2. 敏感数据脱敏（日志、输出中的密钥打码）
    let scrubbed = scrub_secrets(result.output);

    // 3. 临时目录清理（默认行为）
    if !node.config.preserve_artifacts {
        fs::remove_dir_all(&workdir);
    }

    // 4. 句柄/连接泄漏检查（通过 /proc/<pid>/fd 统计）
    assert_no_fd_leak(node.pid);
}
```

**阶段 3：副作用回滚（Rollback）**

如果节点成功但下游 DAG 最终失败，需要回滚该节点的副作用：

```json
{
  "id": "reserve_stock",
  "type": "unit",
  "unit": "inventory-reserve",
  "config": {
    "idempotent": true,
    "rollback": "release_stock",
    "transactional": true
  }
}
```

**回滚触发条件：**
- 同一 `execution_id` 的 Motif 最终状态为 `failed`
- 或下游节点显式声明 `on_failure: rollback_upstream`

### 5.3 与 MCP 无状态约束的兼容

所有增强机制发生在 COGTOME 内部，对外 MCP 接口仍然无状态：

```
Host ──tools/call──► COGTOME MCP Server
                           │
                           ▼
                    ┌──────────────┐
                    │  Motif Engine │
                    │  ├─ Pre-clean │
                    │  ├─ Retry x3  │
                    │  ├─ Fallback  │
                    │  ├─ Post-clean│
                    │  └─ Checkpoint│
                    └──────────────┘
                           │
                    ┌──────▼──────┐
                    │   Unit/MCP   │
                    │  (隔离执行)  │
                    └─────────────┘
```

Host 只发一次 `tools/call`，内部重试 3 次 + 降级 + 补偿，Host 完全无感知。如果最终失败，Host 收到的是标准 MCP 错误响应，内部 trace 写入 SQLite 供人工排查。

---

## 6. 状态管理

### 6.1 对外无状态

- 每次 `tools/call` 完全独立
- 无 session 概念
- Host 完全无感知

### 6.2 内部 Checkpoint（可选增强）

**execution_id 生成：**
```
输入 JSON → 序列化 → SHA256 → execution_id
```

**SQLite 表结构：**
```sql
CREATE TABLE executions (
    id TEXT PRIMARY KEY,           -- SHA256(input)
    assembly_name TEXT NOT NULL,
    status TEXT NOT NULL,          -- pending/running/completed/failed
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE execution_nodes (
    execution_id TEXT,
    node_id TEXT,
    status TEXT,                   -- pending/running/completed/failed
    output TEXT,                   -- JSON 输出
    error TEXT,
    PRIMARY KEY (execution_id, node_id),
    FOREIGN KEY (execution_id) REFERENCES executions(id)
);
```

**重试逻辑：**
- 相同输入哈希 → 相同 `execution_id`
- 检查 `execution_nodes` 表，跳过已完成节点
- Host 完全无感知

### 6.3 长任务进度（未来）

- 首次调用返回 `task_id`
- Host 用 `resources/read` 轮询进度
- 符合 MCP 标准

---

## 7. 启动与配置

### 7.1 启动命令

```bash
# 直接启动（stdio 模式）
cogtome mcp-server

# 或指定 skills 目录
cogtome mcp-server --assemblies ./my-assemblies --units ./my-units
```

### 7.2 Claude Desktop 配置

```json
{
  "mcpServers": {
    "cogtome": {
      "command": "cogtome",
      "args": ["mcp-server"]
    }
  }
}
```

### 7.3 环境变量

```bash
COGTOME_ASSEMBLIES_DIR=./assemblies
COGTOME_UNITS_DIR=./units
COGTOME_TIMEOUT=30
```

---

## 7. Discovery 逻辑

```rust
fn discover_assemblies(dir: &Path) -> Vec<Assembly> {
    WalkDir::new(dir)
        .max_depth(2)
        .filter(|e| e.file_name() == "manifest.json")
        .map(|e| parse_manifest(e.path()))
        .collect()
}

fn discover_units(dir: &Path) -> Vec<Unit> {
    WalkDir::new(dir)
        .max_depth(2)
        .filter(|e| e.file_name() == "unit.json")
        .map(|e| parse_unit(e.path()))
        .collect()
}
```

**启动时：**
1. 扫描 `assemblies/*/manifest.json`
2. 扫描 `units/*/unit.json`
3. 校验引用完整性（每个 `units` 中的 Unit 存在）
4. 加载到内存，生成 `tools/list` 响应

---

## 8. 校验命令

```bash
cogtome validate
```

**检查项：**
- `manifest.json` schema 合法
- `workflow.json` 语法正确（DAG 无环、有 start/return）
- `units` 引用的每个 Unit 在 `units/` 目录下存在
- `entry` 路径可执行

**输出示例：**
```
✓ assemblies/text-processing/manifest.json
✓ assemblies/text-processing/workflow.json
✓ units/text-uppercase/unit.json
✗ assemblies/broken-skill/manifest.json
  Error: Missing required field 'description'
```

---

## 9. 与现有代码的关系

### 9.1 复用的模块

- `src/engine/mod.rs` — GraphMotifEngine, StructureExecutor
- `src/engine/motif_manifest.rs` — Graph/Node/Edge 类型
- `src/engine/unit_runner.rs` — UnitRunner 执行器
- `src/context/` — 变量解析、表达式求值

### 9.2 需要新增的模块

- `src/mcp_server.rs` — MCP Server 实现（stdio JSON-RPC）
- `src/assembly.rs` — Assembly 加载和 discovery
- `src/unit_registry.rs` — Unit 注册和查找

### 9.3 需要修改的模块

- `src/main.rs` — 添加 `mcp-server` 子命令
- `src/discovery.rs` — 改为扫描 `assemblies/` 和 `units/`
- `src/config.rs` — 添加 assemblies/units 路径配置

---

## 10. 向后兼容

- `skills/` 目录继续支持（作为 `assemblies/` 的别名）
- 现有 `cogtome run <name>` 命令不变
- `SKILL.md` front-matter 继续解析（deprecated，但不立即删除）

---

## 11. 测试策略

- **单元测试：**
  - `manifest.json` 解析
  - `unit.json` 解析
  - Discovery 逻辑
  - 错误分层（协议错误 vs 业务错误）

- **集成测试：**
  - 启动 MCP Server
  - 发送 `tools/list` 请求
  - 发送 `tools/call` 请求
  - 模拟超时和错误场景

- **端到端测试：**
  - 创建测试 Assembly
  - 启动 Server
  - 用 MCP 客户端调用
  - 验证输出

---

## 12. 实现优先级

### Phase 1: 核心 MCP Server（2-3 天）

1. 创建 `assemblies/` 目录结构
2. 实现 `manifest.json` 解析
3. 实现 `unit.json` 解析
4. 实现 MCP Server（stdio）
5. 实现 `tools/list`
6. 实现 `tools/call`（复用 GraphMotifEngine）
7. 添加 `cogtome mcp-server` 命令

### Phase 2: 校验与发现（1-2 天）

8. 实现 `cogtome validate` 命令
9. 实现热重载
10. 完善错误处理（协议错误 vs 业务错误）

### Phase 3: Motif 引擎增强（2-3 天）

11. 节点纠错（retry/fallback/compensate）
12. 节点自净化（pre-clean/post-clean）
13. 断路器模式
14. 副作用回滚

### Phase 4: 增强功能（未来）

15. SQLite checkpoint
16. 长任务进度
17. `builtin:linear` 模板
18. 按标签过滤

---

## 13. 设计决策记录

| 决策 | 理由 |
|------|------|
| `assemblies/` 重命名 | "assembly" 强调组装/编排，比 "skills" 更精确 |
| `manifest.json` 为唯一权威 | 机器零解析成本，避免 front-matter 不同步 |
| `SKILL.md` 降级为人类文档 | 分离机器可读与人类可读 |
| `units/` 全局共享 | 解耦复用，单点更新 |
| `tools/list` 不暴露 `outputSchema` | MCP 标准不支持，Host 会忽略 |
| 对外无状态 | 符合 MCP 标准，Host 完全无感知 |
| 内部可做 checkpoint | 长任务断点续跑，输入哈希决定 execution_id |
| 错误分两层 | 协议错误 vs 业务错误，Host 行为不同 |

