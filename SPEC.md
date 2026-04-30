# COGTOME Self-Evolution Mode — Simplified Design

> 用 JSONL 日志记录执行轨迹，Agent 自己分析改进点。不依赖数据库。

**Version**: 0.2-simplified
**Date**: 2026-04-30

---

## 核心理念

**越简单越好。** 日志只是数据，不做任何预分析。Agent 自己读日志，自己想清楚哪里可以改进。

---

## 1. Trace Logger（新增 Unit）

每次 Motif 执行完毕后，自动追加一条 JSONL 记录到 `~/.cogtome/traces/<skill-name>/<YYYY-MM.jsonl>`。

**Unit: `trace-logger`**

- 输入：执行结果（skill_name, duration_ms, status, node_traces, error 等）
- 输出：`{"stored": true, "trace_id": "xxx"}`
- 实现：bash 脚本，`echo "$json" >> ~/.cogtome/traces/<skill>/<date>.jsonl`

**注入点**：在 `engine/mod.rs` 的 `GraphMotifEngine::execute` 完成后调用一次 `trace-logger`。

---

## 2. Trace Log Format

每行一条 JSON：

```jsonl
{"trace_id":"uuid","skill":"daily-summary","started_at":"2026-04-30T10:00:00Z","duration_ms":12000,"status":"success","nodes":[{"id":"route","type":"match","branch":"review","ok":true,"ms":1},{"id":"report","type":"unit","unit":"report-writer","ok":true,"ms":320},{"id":"report","type":"unit","ok":false,"error":"timeout","ms":30001}]}
{"trace_id":"uuid2","skill":"ima","started_at":"2026-04-30T10:05:00Z","duration_ms":5000,"status":"error","nodes":[{"id":"fetch","type":"unit","ok":false,"error":"E_UNIT_NOT_FOUND"}]}
```

关键字段：
- `skill` — 哪个 skill
- `status` — success / error / timeout
- `duration_ms` — 总耗时
- `nodes[].id` — 节点 ID
- `nodes[].ok` — 是否成功
- `nodes[].ms` — 该节点耗时
- `nodes[].error` — 错误信息（如果有）

---

## 3. 分析方式（Agent 自主分析）

**不需要专门的分析引擎。** 直接在现有的 `midnight-reflection` 流程里加一个步骤：

```
凌晨反思流程：
1. memory-indexer → 读最近 N 天 memory
2. task-extractor → 抽取任务状态
3. pattern-finder → 跨日识别重复模式（现有逻辑）
4. trace-analyzer → 新增：读 ~/.cogtome/traces/，让 Agent 分析改进点   ← 关键！
5. report-writer → 生成报告
```

**`trace-analyzer` Unit**：读取 trace 日志文件，把关键信息提取出来给 Agent：

```bash
# 输入
{"skill_name": "daily-summary", "days": 7, "trace_dir": "~/.cogtome/traces"}

# 输出（Agent 可读的摘要）
{
  "summary": "7 天内 daily-summary 执行了 3 次，1 次 error（memory-indexer timeout），平均耗时 12s，p95=15s。route 节点 3 次都走 review 分支，reflection 分支从未命中。建议：1) memory-indexer 加 timeout 重试 2) 考虑 reflection 分支预热"
}
```

Agent 根据这个 summary + memory 内容，自己给出改进建议，写入报告的 `可优化` 章节。

---

## 4. 具体改动清单

### 4.1 `engine/mod.rs` — 注入 trace hook

在 `GraphMotifEngine::execute` 的 `Ok(result)` 和 `Err(e)` 分支各加一行调用 `trace-logger`。

参考位置：`src/engine/mod.rs` 的 `execute` 函数末尾。

### 4.2 新增 Unit: `trace-logger`

```
units/trace-logger/
├── unit.json
└── bin/trace-logger     # bash 脚本
```

```bash
#!/bin/bash
# 读取 stdin JSON，追加到 ~/.cogtome/traces/<skill>/<date>.jsonl
# 自动创建目录
```

### 4.3 新增 Unit: `trace-analyzer`

```
units/trace-analyzer/
├── unit.json
└── bin/trace-analyzer   # python 脚本，读取 JSONL，生成摘要
```

功能：
1. 扫描 `~/.cogtome/traces/` 下最近 N 天的 .jsonl 文件
2. 解析每行，按 skill 分组
3. 计算：执行次数、成功率、平均/p95 耗时、错误类型分布、节点命中率
4. 生成 Agent 可读的 summary JSON

### 4.4 motif 更新：`midnight-reflection.json`

在现有反射流程中插入 `trace-analyzer` 节点。

---

## 5. 安全边界

**什么不能自动改：**
- `units/` 下的可执行文件
- `manifest.json` 的 `input_schema` 字段（破坏契约）
- `risk_level: critical` 的 assembly

**自动改进的范围：**
- 在 `if`/`match` 节点从未命中的分支上加注释（提示 Agent 为什么没走到）
- 在 `foreach` 步骤前加输入大小校验（避免空循环）
- 在 `timeout` 前标记高风险步骤（建议增加 timeout 配置）
- 在 motif 注释里记录"上次这个步骤失败了 N 次"

---

## 6. 实现顺序

**Step 1（今天）：**
- 实现 `trace-logger` Unit（bash，< 20 行）
- 在 `engine/mod.rs` 注入 hook
- 测试：跑一次 `daily-summary`，验证 `~/.cogtome/traces/` 有日志

**Step 2（明天）：**
- 实现 `trace-analyzer` Unit（Python，解析 JSONL + 统计）
- 在 midnight-reflection motif 里加 trace-analyzer 节点
- 测试：凌晨 cron 验证输出包含 trace 分析

**Step 3（后续）：**
- 凌晨反思报告里看 trace 分析的质量
- 根据实际体验调整 trace 字段（加或减）
- 如果需要，再加自动改进逻辑

---

## 7. 文件结构

```
cogtome/
├── src/engine/mod.rs          # 修改：execute() 末尾加 trace-logger 调用
├── units/trace-logger/       # 新增
│   ├── unit.json
│   └── bin/trace-logger
├── units/trace-analyzer/      # 新增
│   ├── unit.json
│   └── bin/trace-analyzer
└── skills/daily-summary/
    └── motifs/
        └── midnight-reflection.json   # 修改：加 trace-analyzer 节点
```
