# COGTOME 实践问题与挑战

本文档记录 COGTOME 在实际开发和使用中可能遇到的问题、风险和建议解决方案。

---

## 一、Runtime 层问题

### 1.1 进程管理与资源泄漏

**问题描述：**
Unit 执行后可能产生僵尸进程或资源未释放，特别是在以下场景：
- Unit 内部调用了后台进程未正确清理
- Unit 执行超时被强制 kill，但子进程残留
- 网络请求占用文件描述符未关闭

**当前缓解：**
- `COGTOME_TIMEOUT_MS` 环境变量用于超时控制
- `COGTOME_UNIT_MODE=1` 阻止 Unit 内部再调 Unit

**未解决：**
- [ ] 缺乏进程树（process tree）完整 kill 机制
- [ ] 缺乏文件描述符泄漏检测
- [ ] 缺乏网络连接池清理

**建议：**
```
Phase 3 实现 cgroups v2 资源限制
Phase 3 实现 Linux Landlock 文件系统隔离
```

---

### 1.2 stdin/stdout JSON 契约的脆弱性

**问题描述：**
- 如果 Unit 输出非 UTF-8 编码的文本，JSON 解析失败
- 如果 Unit 输出多行 JSON（部分工具会输出日志到 stdout），解析失败
- 如果 Unit 输出 JSON 但包含注释，解析失败
- 调试时难以区分"输出是错误信息"还是"输出是业务数据"

**当前缓解：**
- stderr 用于人类可读诊断信息

**未解决：**
- [ ] 缺乏 stdout 清晰分离机制（数据流 vs 日志流）
- [ ] 缺乏 JSON 解析错误的具体位置信息
- [ ] 缺乏二进制数据支持

**建议：**
```
stdout: 结构化 JSON 数据
stderr: 人类可读日志
增加 --debug 模式输出原始字节流
```

---

### 1.3 变量解析的边界情况

**当前支持的变量：**
- `${params.xxx}` — 原始参数
- `${steps.<name>.output.xxx}` — 步骤输出
- `${steps.<name>.exit_code}` — 退出码
- `${env.xxx}` — 环境变量

**未处理的边界情况：**
- [ ] `steps.xxx.output` 为 `null` 时的默认值为空字符串
- [ ] 数组访问 `${steps.fetch.output.items[0].name}` 不支持
- [ ] 条件表达式 `${params.x > 5 ? "big" : "small"}` 不支持
- [ ] 嵌套引用 `${steps.a.output.${params.key}}` 不支持
- [ ] JSON 路径表达式 `${steps.fetch.output..value}` 不支持（双重点）

**建议：**
```
引入 JSONPath 或 JMESPath 支持
增加 ${default(value, fallback)} 语法
```

---

## 二、Discovery 层问题

### 2.1 多版本 Unit 的冲突

**问题描述：**
如果两个 Complex 下各有同名的 `text-uppercase` Unit：
```
~/.agents/skills/
├── complex-a/
│   └── units/text-uppercase/bin/text-uppercase
└── complex-b/
    └── units/text-uppercase/bin/text-uppercase  # 冲突！
```

**当前缓解：**
Discovery 文档提到"全局 → Complex 私有，优先级查找"，但具体优先级规则未实现。

**未解决：**
- [ ] 多版本 Unit 的优先级策略
- [ ] 版本号机制（如何声明 `text-uppercase@1.2`）
- [ ] 依赖锁定机制

**建议：**
```
采用语义版本（SemVer）
Complex 声明依赖时指定版本范围
Runtime 选择策略：最近者优先 / 显式声明优先
```

---

### 2.2 SKILL.md 描述的语义漂移

**问题描述：**
`description` 字段是自由文本，Agent 对它的理解可能与作者意图产生偏差：
- 作者写："文本处理"
- Agent 理解：包括"情感分析"
- 实际场景：不需要情感分析

**当前缓解：**
- `scenarios` 字段提供关键词扩展

**未解决：**
- [ ] 缺乏描述与实际能力的验证机制
- [ ] 缺乏描述一致性检查（描述中提到的 capability 是否都有对应 Motif）
- [ ] 缺乏模糊匹配的调试工具

**建议：**
```
增加 cogtome validate --check-description 命令
对比 description 与实际 Structure/Motif 的输入输出 Schema
```

---

### 2.3 目录扫描性能

**问题描述：**
每次 `cogtome discover` 都扫描整个目录树，在以下场景变慢：
- `~/.agents/skills/` 包含大量 Complex
- Unit 二进制文件很大（Go/Rust 编译产物）
- 网络存储（NAS）延迟

**未解决：**
- [ ] 元数据缓存与热重载（Phase 2）
- [ ] 增量扫描（只检查变更文件）
- [ ] 后台守护进程预扫描

**建议：**
```
启动 daemon 模式后，在空闲时预扫描并缓存
写 ~/.cogtome/cache/index.json
```

---

## 三、执行层问题

### 3.1 并行执行的状态竞争

**问题描述：**
当 Motif 的多个 Unit 并行执行时：
```yaml
flow:
  - name: fetch_a
    unit: fetch-url
    parallel: group1
    input: { url: "${params.url_a}" }
  - name: fetch_b
    unit: fetch-url
    parallel: group1
    input: { url: "${params.url_b}" }
  - name: merge
    unit: merge-results
    after: group1
    input:
      data: "${steps.fetch_a.output.data} + ${steps.fetch_b.output.data}"  # 竞争！
```

**未解决：**
- [ ] 并行步骤的输出合并顺序不确定
- [ ] `steps.fetch_a.output` 在 `fetch_a` 未完成时的值为空/错误
- [ ] 缺乏并行安全的数组操作（如 append）

**建议：**
```
并行步骤的输出按数组顺序返回，不按名字
${steps.group1.output[0]} 对应第一个完成的
引入 ${steps.group1.results} 作为完整结果数组
```

---

### 3.2 错误恢复与重试的复杂性

**问题描述：**
当前重试策略（exit code 2/3 可重试）过于简单：
- 网络请求失败：立即重试可能浪费资源
- 数据库连接失败：需要指数退避
- API 限流：需要遵循 `Retry-After` 响应头
- 第三方服务宕机：需要熔断机制

**未解决：**
- [ ] 缺乏退避策略配置（linear/exponential/fibonacci）
- [ ] 缺乏熔断机制
- [ ] 缺乏重试次数限制
- [ ] 缺乏部分成功处理（部分 Unit 成功，部分失败）

**建议：**
```yaml
- name: fetch
  unit: fetch-url
  retry:
    max_attempts: 3
    backoff: exponential
    initial_delay_ms: 1000
    max_delay_ms: 30000
    retry_on: [2, 3, 429]
```

---

### 3.3 超时控制的粗糙性

**问题描述：**
当前超时是"从 Unit 启动到结束的总时间"，无法实现：
- "等待某个条件出现，最多 10 秒"
- "每个步骤最多 5 秒，但总流程不限"
- "直到成功为止，不限时间"

**未解决：**
- [ ] 分层超时（步骤级 / Motif 级 / Structure 级）
- [ ] 条件等待（poll until condition）
- [ ] 取消令牌（cancellation token）

**建议：**
```yaml
- name: wait_for_ready
  unit: poll-health
  timeout_ms: 30000
  poll_interval_ms: 1000
  condition: "${steps.poll.output.status == 'ready'}"
```

---

## 四、安全层问题

### 4.1 Unit 的权限控制

**问题描述：**
Unit 执行时拥有 Runtime 进程的完整权限：
- 可以访问所有文件
- 可以发起网络请求
- 可以执行任意系统命令
- 可以 fork 新进程

**当前缓解：**
- `COGTOME_UNIT_MODE=1` 阻止 Unit 嵌套调用

**未解决：**
- [ ] 缺乏最小权限原则
- [ ] 缺乏文件系统访问控制（只读 / 只写特定目录）
- [ ] 缺乏网络访问控制（只允许特定域名）
- [ ] 缺乏系统命令黑名单

**建议：**
```
Phase 3 实现 Linux Landlock
Unit 声明所需权限：
capabilities:
  filesystem: ["read:/tmp", "write:/output"]
  network: ["https://api.example.com"]
```

---

### 4.2 输入验证的完整性

**问题描述：**
`input_schema` 只在 Structure 层验证，Motif 和 Unit 层缺乏验证：
- Motif 可能收到未声明的额外字段
- Unit 可能收到类型错误的输入（如期望 string 但收到 array）
- 嵌套对象的深度验证不完整

**未解决：**
- [ ] 全链路输入验证
- [ ] 验证失败的具体字段报告
- [ ] 验证策略配置（strict / permissive）

**建议：**
```
所有层统一使用 JSON Schema draft-07
增加 --strict 模式严格校验类型
```

---

### 4.3 日志泄露敏感信息

**问题描述：**
执行日志可能包含：
- API 密钥（环境变量中的 Bearer token）
- 用户隐私数据（手机号、邮箱）
- 数据库查询结果

**当前缓解：**
无

**未解决：**
- [ ] 敏感字段自动脱敏
- [ ] 日志级别控制（DEBUG 包含敏感信息，INFO 不包含）
- [ ] 日志审计（谁在什么时间看了什么日志）

**建议：**
```bash
COGTOME_LOG_LEVEL=info  # 默认不输出敏感字段
COGTOME_SENSITIVE_FIELDS=token,password,secret  # 声明敏感字段
```

---

## 五、生态问题

### 5.1 多语言 Unit 的维护负担

**问题描述：**
Agent 可以用任意语言编写 Unit，但：
- 每种语言需要独立的运行时环境
- Python Unit 依赖的包版本冲突
- 不同 OS 下的二进制兼容性问题

**当前缓解：**
无

**未解决：**
- [ ] 官方推荐的 Unit 模板
- [ ] 依赖声明格式（requirements.txt / go.mod / Cargo.toml）
- [ ] 跨平台构建支持

**建议：**
```
只支持三种 Unit 语言：Shell（轻量）、Python（通用）、Rust（性能）
提供官方镜像包含标准依赖
```

---

### 5.2 Complex 的分发与安装

**问题描述：**
当前 Complex 只能通过文件系统分发：
- 复制整个 skills/ 目录
- 手动放置到正确位置

**当前缓解：**
无

**未解决：**
- [ ] `.cogtome` 打包格式
- [ ] Registry / 中央仓库
- [ ] `cogtome install <name>` 一键安装
- [ ] 版本管理与升级

**Phase 4 计划：**
```
cogtome pack ./my-complex/ → my-complex.cogtome
cogtome install my-complex.cogtome
cogtome registry search <keyword>
```

---

### 5.3 Python Motif 的性能开销

**问题描述：**
Python Motif 通过子进程 + IPC 调用 Runtime：
- 每次 `unit.call()` 需要进程创建 + Socket 通信
- 大量小步骤时开销显著
- Python 启动时间 ~100ms

**当前缓解：**
无

**未解决：**
- [ ] Python Motif 的 IPC 优化
- [ ] Python 进程池预热
- [ ] gRPC 替代 JSON-RPC over Unix Socket

**建议：**
```
Phase 2 实现 Python 进程池（预热 3 个进程复用）
避免频繁创建销毁
```

---

## 六、测试与调试问题

### 6.1 缺乏单元测试框架

**问题描述：**
- Unit 如何写测试？
- Motif 如何写测试？
- 如何在不执行外部依赖的情况下测试？

**未解决：**
- [ ] 官方测试框架建议
- [ ] Mock/Stub 机制
- [ ] `cogtome test <unit>` 命令

**建议：**
```
Unit 测试：独立进程 + mock stdin
Motif 测试：加载 YAML + mock Unit 结果
Structure 测试：加载 manifest + mock Motif 结果
```

---

### 6.2 执行追踪的可读性

**问题描述：**
`cogtome inspect <execution-id> --tree` 输出的 JSON Lines 难以阅读：
- 大量技术细节
- 缺乏人类友好的摘要
- 缺乏问题根因分析

**未解决：**
- [ ] 树形可视化输出
- [ ] 执行瓶颈分析（哪个步骤最慢）
- [ ] 错误传播路径追踪

**建议：**
```
增加 --format tree/text/json/yaml 选项
增加 --verbose 和 --quiet 级别
增加执行摘要：总耗时、成功/失败步骤数、重试次数
```

---

## 七、OpenClaw 集成问题

### 7.1 Architecture Decision: COGTOME is Pure Execution Backend (2026-04-24)

**Established Principle:**
- COGTOME = pure execution backend (Option A)
- OpenClaw = decision layer (intent matching, Complex selection)
- Discovery = capability catalog (`ls` + `cat`), NOT auto-router (`grep --smart`)

```
OpenClaw (decision): Understand intent → Select Complex → Construct parameters
     ↓
COGTOME (execution): Receive → Parse Structure → Orchestrate Motif → Schedule Unit
     ↓
OS: Process, file, network
```

**Key rules:**
- "Matching" must happen at upper layer
- "Execution" must happen at COGTOME
- If COGTOME also matches → dual Agent problem, debugging nightmare
- Agent only sees Complex layer (10-20 items), never Unit (100+ items)

### 7.2 Auto-Complex Mechanism (Solution to Simple Task Problem)

**Problem:** Simple task like "extract numbers" requires 4 files (Complex + Structure + Motif + Unit)

**Solution:** `auto-complex` registration:

```bash
cogtome register unit extract_numbers.py \
  --name "extract_numbers" \
  --description "从文本中提取所有数字" \
  --input "text: string" \
  --output "numbers: list[int]" \
  --auto-complex  # Runtime auto-generates Complex + Structure + Motif
```

Runtime auto-generates:
- Complex: SKILL.md with description
- Structure: manifest.yaml with schema
- Motif: YAML with direct_run flow

Result: Developer writes 1 file, Agent sees 1 Complex - architecture stays unified.

---

### 7.1 Skill 发现机制的差异

**问题描述：**
- OpenClaw Skills 位于 `~/.agents/skills/`
- COGTOME Complex 位于 `~/cogtome-demo/skills/`
- 两个系统的 SKILL.md 格式不完全兼容

**当前状态：**
- OpenClaw 的 SKILL.md 有 `description` 字段
- COGTOME 的 SKILL.md 有 `description` 字段
- 但目录结构和元数据格式不同

**建议：**
```
统一 SKILL.md 前端格式（description、scenarios、examples）
后端实现分离：
  - OpenClaw → AI 意图匹配
  - COGTOME → 进程级执行
```

---

### 7.2 执行结果的传递

**问题描述：**
COGTOME 执行完成后，结果如何传递给 OpenClaw？
- JSON 输出作为字符串返回
- 文件路径引用需要额外处理
- 流式输出（大量日志）如何处理

**当前状态：**
无标准协议

**建议：**
```
标准化结果格式：
{
  "status": "success|error",
  "output": { ... },
  "metadata": {
    "execution_id": "xxx",
    "duration_ms": 123,
    "units_executed": ["a", "b"]
  },
  "errors": []
}
```

---

## 八、优先级建议

### 立即解决（P0）

1. **stdin/stdout 清晰分离** — 防止日志污染数据流
2. **变量解析的 null 处理** — 防止空指针
3. **输入验证完整性** — 防止类型错误传播

### 高优先级（P1）

4. **进程树 kill** — 防止资源泄漏
5. **分层超时控制** — 精确控制执行时间
6. **敏感信息脱敏** — 防止日志泄露

### 中优先级（P2）

7. **元数据缓存** — 提升发现速度
8. **并行安全** — 正确处理并行输出
9. **错误恢复策略** — 指数退避 + 熔断

### 低优先级（P3）

10. **多语言 Unit 优化** — Python 进程池
11. **Registry 分发** — `.cogtome` 打包
12. **Web UI** — 可视化监控

---

## 附录：已知 Bug

| ID | 描述 | 严重性 | 状态 |
|----|------|--------|------|
| B001 | 变量 `${steps.a.output[0]}` 数组访问不支持 | 高 | 未修复 |
| B002 | 并行步骤输出顺序不确定 | 中 | 未修复 |
| B003 | 超时被 kill 的进程可能留下僵尸 | 中 | 未修复 |
| B004 | stderr 日志可能干扰 JSON 解析 | 低 | 已知 |

