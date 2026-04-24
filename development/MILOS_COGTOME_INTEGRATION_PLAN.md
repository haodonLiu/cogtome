# Milos × COGTOME 整合计划书

> 将 COGTOME 四层执行架构融入 Milos 工作流，消除幻觉，实现可审计的每步执行。

---

## 一、背景与目标

### 1.1 现状问题

Milos（我）目前执行任务的方式：

```
用户请求 → 隐性推理 → 调用工具 → 结果输出
```

**问题：**
- 推理过程不可见、无记录
- 中间状态存在"脑子里"，不落在文件
- 幻觉产生的空间大
- 无法追踪每步的输入输出
- 多步任务无结构化编排，重度依赖 prompt engineering

### 1.2 目标

将 COGTOME 的四层执行纪律融入 Milos：

```
Complex（入口）→ Structure（业务编排）→ Motif（工作链）→ Unit（原子执行）
```

每层只调用下一层，所有执行步骤必须：
1. 有 manifest 声明
2. 有执行结果记录
3. 有 exit_code 校验
4. 结果通过变量引用，不靠"我记得"

---

## 二、整合架构

### 2.1 目录结构

```
~/.milos/                          # Milos 执行引擎根目录
├── structures/                    # Structure 层（业务结构）
│   ├── web-research/
│   │   └── manifest.yaml
│   ├── code-review/
│   │   └── manifest.yaml
│   ├── file-operations/
│   │   └── manifest.yaml
│   └── note-taking/
│       └── manifest.yaml
├── motifs/                       # Motif 层（工作链）
│   ├── fetch-and-extract.yaml
│   ├── git-status-check.yaml
│   ├── context-summarize.yaml
│   └── web-search-collect.yaml
├── units/                        # Unit 层（原子执行）
│   ├── fetch-url/
│   │   ├── bin/fetch-url        # Shell 脚本
│   │   └── unit.yaml            # Unit 契约定义
│   ├── extract-markdown/
│   │   ├── bin/extract-markdown
│   │   └── unit.yaml
│   ├── save-note/
│   │   ├── bin/save-note
│   │   └── unit.yaml
│   └── git-diff/
│       ├── bin/git-diff
│       └── unit.yaml
└── logs/                         # 执行日志
    └── YYYY-MM-DD/
        └── {execution_id}.json   # 每步执行记录

~/.agents/skills/                  # OpenClaw 技能发现层
└── cogtome/                      # COGTOME Complex（作为 OpenClaw Skill）
    └── SKILL.md
```

### 2.2 各层职责

| 层级 | 位置 | 职责 | 调用方式 |
|------|------|------|----------|
| **Complex** | `~/.agents/skills/cogtome/SKILL.md` | 唯一对外入口，AI 自动发现 | OpenClaw 语义匹配 |
| **Structure** | `~/.milos/structures/*/manifest.yaml` | 编排多个 Motif | `cogtome structure run <name>` |
| **Motif** | `~/.milos/motifs/*.yaml` | 编排多个 Unit（串行/并行） | `cogtome motif run <name>` |
| **Unit** | `~/.milos/units/*/bin/*` | 原子执行，不可再分 | `cogtome unit run <name>` |

### 2.3 与 OpenClaw 的关系（已确立边界）

```
OpenClaw (Milos)
    │
    ├── 发现 Complex（~/.agents/skills/*/SKILL.md）
    │       ↓
    │   匹配到 cogtome skill（OpenClaw 做决策）
    │       ↓
    └── 调用 COGTOME CLI（cogtome structure run xxx）
            │
            ├── Structure manifest.yaml
            │       ↓
            ├── Motif *.yaml
            │       ↓
            └── Unit bin/*（生成 JSON 结果）
                    ↓
                写入 ~/.milos/logs/
```

**⚠️ 关键边界约束（2026-04-24 确立）：**

| 层级 | 职责 | 禁止 |
|------|------|------|
| **OpenClaw** | 意图理解、Complex 匹配、参数构造 | 不做执行 |
| **COGTOME** | 执行指定 Complex忠实执行，不做路由 | 不做意图匹配 |

**原则：**
- "Agent 选 Complex，COGTOME 跑 Unit；Agent 做决策，COGTOME 做纪律"
- Discovery = 能力目录（`ls` + `cat`），不是自动路由（`grep --smart`）
- COGTOME 绝对不做匹配，否则产生双重 Agent 问题

**简单任务解决方案：**
- 提供 `--auto-complex` 注册机制
- 写 1 个 Unit 脚本，Runtime 自动生成 Complex + Structure + Motif

---

## 三、实施计划

### Phase 0：基础设施（1天）

**目标：** COGTOME CLI 在 WSL 里可运行，`cogtome` 命令生效

**任务：**
- [ ] 确认 Rust 环境（`rustc --version`）
- [ ] 编译 `cogtome-demo` 项目：`cd ~/cogtome-demo && cargo build --release`
- [ ] 将 `target/release/cogtome` 链接到 `~/.milos/cogtome` 或 `~/.local/bin/cogtome`
- [ ] 初始化目录结构：`~/.milos/{structures,motifs,units,logs}`
- [ ] 创建 COGTOME Complex SKILL.md

### Phase 1：Unit 层构建（2-3天）

**目标：** 构建最常用的原子执行 Unit，覆盖 80% 日常操作

**Unit 清单：**

| Unit 名称 | 输入 | 输出 | 实现方式 |
|-----------|------|------|----------|
| `fetch-url` | `{ url, selector? }` | `{ html, markdown, status }` | Shell + curl/wget |
| `read-file` | `{ path, offset?, limit? }` | `{ content, lines }` | Shell |
| `write-file` | `{ path, content, append? }` | `{ bytes_written }` | Shell |
| `git-status` | `{ repo_path }` | `{ files, branch, clean }` | Shell |
| `git-diff` | `{ repo_path, file? }` | `{ diff, staged }` | Shell |
| `save-note` | `{ filename, content }` | `{ path, size }` | Shell |
| `list-files` | `{ path, pattern?, recursive? }` | `{ files[] }` | Shell + find |
| `extract-markdown` | `{ html, query? }` | `{ content, links[] }` | Python 脚本 |
| `run-command` | `{ cmd, cwd?, timeout? }` | `{ stdout, stderr, exit_code }` | Shell |

### Phase 2：Motif 层构建（2-3天）

**目标：** 将常用操作序列写成 YAML Motif

**Motif 清单：**

```yaml
# motifs/web-research.yaml
name: web-research
type: motif
units_required: [fetch-url, extract-markdown]
flow:
  - name: fetch
    unit: fetch-url
    input: { url: "${params.url}" }
  - name: extract
    unit: extract-markdown
    input: { html: "${steps.fetch.output.html}" }
return:
  content: "${steps.extract.output.content}"
  links: "${steps.extract.output.links}"
```

**Motif 场景：**
- `web-research` — 抓取 + 提取页面内容
- `git-audit` — git status + diff 组合
- `file-batch-read` — 读取多个文件内容

### Phase 3：Structure 层构建（1-2天）

**目标：** Structure 编排多个 Motif，形成完整业务操作

**Structure 清单：**

```yaml
# structures/code-review.yaml
name: code-review
type: structure
motifs:
  - name: git-status-check  # 检查变更文件
  - name: git-diff-extract  # 提取变更内容
  - name: ai-review-generate # 生成 review 意见（调用 AI）
input_schema:
  type: object
  required: [repo_path]
  properties:
    repo_path: { type: string }
output_schema:
  type: object
```

### Phase 4：Milos 执行层集成（持续）

**目标：** Milos 在执行任务时遵循 COGTOME 执行纪律

**执行流程（修订后）：**

```
用户请求
    ↓
解析意图 → 匹配 Complex（~/.agents/skills/*/SKILL.md）
    ↓
选择 Structure（~/.milos/structures/*/manifest.yaml）
    ↓
加载 Motif YAML（~/.milos/motifs/*.yaml）
    ↓
逐条执行 Unit（~/.milos/units/*/bin/*）
    ↓
每步结果写入 ~/.milos/logs/YYYY-MM-DD/{execution_id}.json
    ↓
返回最终结果 + execution trace
```

**关键约束：**
- 不在脑子里存中间结果，全部写文件
- Unit 输出必须为 JSON，不接受纯文本
- 变量引用全部用 `${steps.xxx.output.yyy}` 格式
- 多步任务必须有 Motif YAML 作为执行计划

---

## 四、幻觉消除机制

### 4.1 强制记录

每次执行：
```bash
cogtome unit run fetch-url --input '{"url":"..."}' \
  | tee ~/.milos/logs/$(date +%Y-%m-%d)/$(uuidgen).json
```

### 4.2 变量追溯

Milos 不说"我记得 X"，而是说：
> `${steps.fetch.output.markdown}` 中包含...

所有信息必须有 `steps` 来源。

### 4.3 Exit Code 检查

Unit 执行后必须检查：
```rust
if exit_code != 0 {
    // 不继续，报告错误
    eprintln!("Unit '{}' failed: {}", name, stderr);
}
```

---

## 五、预期效果

| 指标 | 现状 | 目标 |
|------|------|------|
| 中间步骤可见性 | 隐性，存在脑子里 | 全量记录在文件 |
| 多步任务编排 | Prompt 拼凑 | YAML Motif 声明 |
| 执行结果可追溯 | 不可 | execution_id 全链路 |
| 幻觉空间 | 大 | 压缩到最小 |
| 新任务接入速度 | 每次重新设计 | 复用 Motif 组合 |

---

## 六、风险与对策

| 风险 | 对策 |
|------|------|
| Rust 编译环境不稳定 | 用 Python 写 Prototype，证明后再 Rust 重写 |
| Unit 过多难以维护 | 严格 description + 分类标签 |
| YAML 维护成本高 | 编写 Motif 生成器，配合 AI 生成 YAML |
| 执行速度下降 | Unit 预热（Daemon 模式保持进程池） |

---

*Plan version: 1.0 | Date: 2026-04-24*
