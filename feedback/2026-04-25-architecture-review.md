# COGTOME 架构评审 (2026-04-25)

> 来源：Easton Liu
> 评估：架构方向正确，但有 10 个具体缺陷会在第一次接入真实 Agent 时暴露

---

## 核心定位确认

- 内核极简、入口多样、MCP 不侵入
- COGTOME 是 Agent 的原生执行语言，不是协议桥接

---

## 10 个具体缺陷（按严重程度排序）

### 1. Registry 职责不纯，SKILL.md 与 manifest.yaml 无一致性校验

**问题**：manifest.yaml 改了字段名，SKILL.md 不同步，Agent 用过时的文档调用，导致运行时错误

**建议**：
- 取消 SKILL.md 作为独立真相源
- manifest.yaml 的 `input` 同时生成机器校验规则和人类可读文档
- Registry 只认 manifest.yaml，SKILL.md 只做渲染用途

---

### 2. Runtime 没有输入校验层

**问题**：Agent 传 `{"url": 123}` 或 `{"uri": "..."}`，Runtime 直接喂给 Unit，不拦截

**建议**：
- manifest.yaml 的 `input` 必须是 JSON Schema（或类型声明）
- Runtime 执行前先校验输入，失败返回结构化错误
- 不要启动进程

---

### 3. 错误处理模型完全缺失

**问题**：step 失败时 Runtime 行为未定义。终止？跳过？重试？引用什么？

**建议**：manifest.yaml 必须显式声明：
```yaml
steps:
  - unit: http-get
    retry: { max: 3, backoff: exponential }
    on_error: fail  # 或 continue / fallback
```

---

### 4. Unit 进程通信协议没有严格约定

**问题**：
- stdout 加调试日志 → Runtime 解析 panic
- NDJSON 多行输出 → 不知道哪行是结果

**建议**：必须约定边界协议：
- stdout 只输出一行 JSON（最终结果）
- 日志必须走 stderr
- 或用 NDJSON 带 `type` 字段区分 `result/log/progress`
- 或长度前缀帧 `Content-Length: 123\n\n{...}`

---

### 5. 安全沙箱为零

**问题**：Unit 和 Runtime 同用户权限，可以：
- `rm -rf $HOME/`
- 任意网络请求
- 读取其他 Unit 的 API Key

**建议**（最小安全三层）：
1. **工作目录隔离**：每次执行独立 tmpdir
2. **环境变量白名单**：Unit 只看到显式注入的变量
3. **资源上限**：超时、内存/CPU 限制、可选网络命名空间

---

### 6. Registry 纯内存态，无热重载

**问题**：改了 manifest.yaml 必须重启 Runtime 才能生效

**建议**：
- `cogtome reload` 命令
- 或基于 `notify` crate 的文件系统事件自动热重载
- 重载时清理已缓存的 Structure AST

---

### 7. MCP 层过度简化，忽略协议状态机

**问题**：MCP 不是无状态 JSON-RPC，有：
- `initialize` 握手和能力协商
- `notifications/progress` 长任务推送
- stdio vs SSE 传输差异

**建议**：
- 实现完整 MCP 协议状态机
- 或使用成熟 SDK（如 Rust `rmcp`）
- `tools/list` 的 `inputSchema` 从 manifest.yaml 自动生成

---

### 8. 并发模型与执行上下文隔离未定义

**问题**：两个请求同时写 `/tmp/cogtome_output.json` 会互相覆盖

**建议**：
- 每次 `run` 生成独立 Execution Context
- 包含 UUID、独立 tmpdir、独立变量作用域
- Runtime 是"执行器工厂"，不是单例

---

### 9. 聚合逻辑过于简单

**问题**：
- step 1 失败但 step 2 有降级数据，怎么组装？
- 数组过滤/映射 `steps[0].output.files.filter(f => f.size > 0)` 怎么写？

**建议**：
- 明确聚合是 Motif 的责任，不是 Unit 的责任
- manifest.yaml 的 `output` 支持轻量表达式
- 或引用 `aggregate` motif，避免逻辑下沉到 Unit

---

### 10. 缺少执行可观测性

**问题**：10 步执行，第 8 步慢，Agent 长时间无响应，不知道是卡死还是出错

**建议**：输出执行事件流（Execution Events）：
- `step_start { step_id, unit, timestamp }`
- `step_end { step_id, duration, status }`
- `variable_resolved { name, source }`
- CLI → stderr，HTTP/MCP → SSE 或 `notifications/progress`

---

## 总结

**最大风险**：不是"概念不清"，而是"执行细节缺位"。

必须优先锁定三件事：
1. **输入校验规则**
2. **Unit 通信协议格式**
3. **步骤级错误处理策略**

这三件事定了，Rust 端的类型系统才能写对。

---

## 优先级排序

| 优先级 | 问题 | 理由 |
|--------|------|------|
| P0 | 输入校验 | 一个字段传错就崩溃 |
| P0 | 错误处理策略 | 任何网络问题让系统挂住 |
| P0 | Unit 通信协议 | 第三方 Unit 让系统不可调试 |
| P1 | 安全沙箱 | 生产环境必须有隔离 |
| P1 | 并发隔离 | 多请求场景必现 |
| P2 | 热重载 | 开发迭代体验 |
| P2 | 可观测性 | 调试困难 |
| P3 | SKILL.md 一致性 | 文档问题 |
| P3 | 聚合表达式 | 功能完整性 |
| P3 | MCP 状态机 | 如果不用 MCP 可以不做 |
