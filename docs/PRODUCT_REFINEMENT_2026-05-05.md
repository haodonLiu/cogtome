# COGTOME 产品定位梳理 (2026-05-05)

> 来源：Hermes × COGTOME 讨论。目标：明确 COGTOME 的核心价值、产品定位、以及下一步实施方向。

---

## 一、核心定位

**COGTOME = Hermes Agent 的执行引擎**

```
Hermes（Agent 框架）
    ├── 理解用户意图
    ├── 决策和规划
    ├── 调用工具
    └── 调度 COGTOME
            │
            ▼
      COGTOME（执行引擎）
            ├── 可靠的 DAG 执行
            ├── 进程隔离
            ├── 步骤间契约
            └── 返回结构化结果
```

- **Hermes 是脑子** — 做意图理解、决策、规划
- **COGTOME 是手** — 稳定执行多步骤工作流

---

## 二、COGTOME 的核心价值（差异化）

| 能力 | 解决的问题 |
|------|-----------|
| **精确错误定位** | 任何一步出错，Agent 和用户都能直接看到是哪个 Unit 失败，不是黑盒猜测 |
| **避免死循环** | Exit code (0=成功, 1=输入错误, 2=重试, 3=依赖不可用) 驱动重试策略，Hermes 根据错误类型决定下一步 |
| **进程隔离** | 一个步骤崩溃不影响其他步骤 |
| **步骤间契约** | stdin/stdout JSON Schema 验证，输入输出是确定性的 |
| **可见性** | 每一步的输入输出都记录，Agent 和用户都能查 |
| **上下文高效** | 结构化 Unit 调用比反复传对话历史省 token |

**对比传统 LLM Agent：**

| | 传统 LLM Agent | COGTOME |
|---|---|---|
| 错误定位 | 模糊（"出错了"） | 精确到哪个 Unit |
| 死循环 | 容易 | Exit code 兜底 |
| 用户可见性 | 低（黑盒） | 高（每步可查） |
| 重试策略 | LLM 自己猜 | 错误类型驱动 |

---

## 三、目标用户场景

**帮助普通人把工作流迁移到 AI**

- 不是所有任务都有创造性，生活中大量是固定流程 + 变量输入
- 传统 AI Agent 门槛高（要会写提示词、会调试）
- COGTOME 的价值：**自然语言描述意图 → 结构化工作流 → 稳定执行**
- 用户可以看到每一步，发现问题可以精确纠错

**典型场景示例：**

```
用户："我每天要把客户发来的 Excel 订单整理到 Notion，再通知 Slack"
    ↓ Hermes 理解意图，拆解为 DAG
[read-email] → [extract-order] → [write-notion] → [send-slack]
    ↓ COGTOME 稳定执行
    - 某步骤失败？精确知道是哪步
    - 邮件服务挂了？通知用户，不死循环重试
    - 每天固定时间跑，过程完全可观测
```

---

## 四、技术架构

### 4.1 Hermes → COGTOME 调用链

```
用户输入（自然语言）
    ↓
Hermes Agent（理解意图）
    ↓
拆解为步骤 + 确定工具需求
    ↓
调用 COGTOME Assembly（如 hermes-daily-review）
    ↓
COGTOME DAG 执行（进程隔离）
    ↓
结构化结果（JSON）
    ↓
Hermes 包装结果 → 推送用户
```

### 4.2 每日复盘场景（近期实施目标）

```
凌晨 3:00 cron 触发
    ↓
Hermes 调用 COGTOME Assembly: hermes-daily-review
    ↓
DAG 执行（单元化）：
  session-indexer     → 扫描 ~/.hermes/sessions/ 按日期找 .jsonl
      ↓
  message-extractor   → 解析 JSONL，提取 user/assistant 消息
      ↓
  memory-detector     → 检测偏好/规则关键词
      ↓              → memory-patcher → 写入 ~/.hermes/memories/
      ↓
  skill-detector      → 检测报错/流程变化
      ↓              → skill-recorder → 写入 ~/.hermes/auto_skill_issues.md
      ↓
  daily-log-writer    → 写入 ~/.hermes/daily/YYYY-MM-DD.md
      ↓
  weekly-log-writer   → 追加到 ~/.hermes/weekly/YYYY-WXX.md
      ↓
Return（完整报告 JSON）
    ↓
Hermes 推送微信
```

### 4.3 Exit Code 策略

| Exit Code | 含义 | Hermes 处理策略 |
|-----------|------|----------------|
| 0 | 成功 | 继续下一步 |
| 1 | 输入错误 | 记录，不重试，直接终止 DAG |
| 2 | 重试 | 等一等重跑一次，超过次数终止 |
| 3 | 依赖不可用 | 跳过该步骤，通知用户，继续其他步骤 |

---

## 五、下一步实施计划

### Phase 1: 每日复盘 Assembly（最小闭环）

新建 `assemblies/hermes-daily-review/`，把现有 `daily_review.py` 的能力迁移过去：

```
assemblies/hermes-daily-review/
├── manifest.json          # Assembly 定义
├── workflow.json          # DAG 编排
└── units/
    ├── session-indexer/   # Unit: 扫描 session JSONL 文件
    ├── message-extractor/ # Unit: 解析 JSONL，提取消息
    ├── memory-detector/   # Unit: 检测偏好/规则关键词
    ├── skill-detector/    # Unit: 检测报错/流程变化
    ├── memory-patcher/    # Unit: 写入 memory
    ├── skill-recorder/    # Unit: 写入 skill issues
    ├── daily-log-writer/  # Unit: 写每日日志
    └── weekly-log-writer/ # Unit: 写周报
```

### Phase 2: Hermes 集成

- Hermes 的 cron job 从 `python3 daily_review.py` 改为调用 COGTOME Assembly
- 设计 Hermes → COGTOME 的调用接口（命令行？HTTP API？）
- 让 Hermes 能读取 Assembly 的返回结果

### Phase 3: 错误可视化

- 当 DAG 中某个 Unit 失败时，Hermes 能展示：
  - 哪个步骤失败了
  - 失败原因（exit code + stdout 输出）
  - 用户可以决定：重试、跳过、终止

### Phase 4: Units 生态（长期）

- 沉淀常用的 Unit：read-email、write-notion、send-slack、read-file、http-request 等
- 形成 Units 库，方便快速搭工作流

---

## 六、待讨论问题

1. Hermes → COGTOME 的调用方式：命令行（`cogtome run`）还是 HTTP API？
2. Exit code 策略是否需要支持配置化（不同 Assembly 可以自定义错误处理）？
3. Units 库的建设方式：COGTOME 官方维护还是社区贡献？

---

## 七、参考资料

- COGTOME 源码：`~/cogtome/`
- 现有每日复盘脚本：`~/.hermes/scripts/daily_review.py`
- 现有 Assembly 示例：`~/cogtome/assemblies/daily-summary/`
