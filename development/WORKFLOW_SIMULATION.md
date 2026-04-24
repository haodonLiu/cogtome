# COGTOME 工作流模拟与问题发现

> 模拟时间：2026-04-24
> 模拟人：Milos

---

## 模拟场景总览

| 场景 | 复杂度 | 目的 |
|------|--------|------|
| 场景1：提取文本中的数字 | 简单（1 Unit） | 发现简单任务的层间开销 |
| 场景2：网页研究并保存笔记 | 中等（3 Unit） | 发现跨 Motif 协作问题 |
| 场景3：Git 代码审查 | 复杂（5 Unit + AI） | 发现工具链集成问题 |

---

## 场景1：提取文本中的数字

**任务**："从文本'订单号12345，总价¥1999.99元'中提取所有数字"

### 1.1 当前需要的文件

```
skills/
├── units/text-numbers-extract/
│   └── bin/text-numbers-extract      # Python 脚本
├── motifs/numbers-extract.yaml        # Motif YAML
├── structures/numbers-extract/
│   └── manifest.yaml                  # Structure manifest
└── complex/numbers-processing/
    └── SKILL.md                      # Complex
```

**4 个文件**，即使这个任务只需要一个 Unit。

### 1.2 模拟执行

```bash
$ cogtome run numbers-processing --input '{"text":"订单号12345，总价¥1999.99元"}'
```

### 1.3 发现的问题

**问题 1.3.1：Unit 输出格式不确定**

```python
# Unit 脚本可能的输出：
# 选项A：返回字符串数组
{"numbers": ["12345", "1999.99"]}

# 选项B：返回数字类型
{"numbers": [12345, 1999.99]}

# 选项C：返回带单位的结构
{"numbers": [{"value": "12345", "type": "integer"}, {"value": "1999.99", "type": "float"}]}
```

Motif 引用时：
```yaml
return:
  first_number: "${steps.extract.output.numbers[0]}"
  count: "${steps.extract.output.numbers.length}"  # YAML 不支持 .length！
```

**问题 1.3.2：变量表达式不支持数组操作**

```yaml
# 想做：
count: "${steps.extract.output.numbers.length}"      # ❌ 不支持
first: "${steps.extract.output.numbers[0]}"         # ✅ 支持
last: "${steps.extract.output.numbers[-1]}"         # ❌ 不支持 Python 负索引
filter: "${steps.extract.output.numbers.filter(x => x > 1000)}"  # ❌ 不支持
```

**问题 1.3.3：数字格式多样性**

```
1999.99    → 正常提取
1,234.56   → 逗号分隔
1.23e-4    → 科学计数法
０１２３    → 全角数字
ⅣⅦⅢ       → 罗马数字（算不算？）
```

**实际单元测试可能失败，但 SKILL.md 的 description 不会说这些边界。**

---

## 场景2：网页研究并保存笔记

**任务**："搜索 Rust 并发编程，把结果保存到 notes/rust-concurrency.md"

### 2.1 执行流程

```
User Input: "搜索 Rust 并发编程，把结果保存到 notes/rust-concurrency.md"
                    │
                    ▼
        ┌───────────────────────┐
        │ OpenClaw 做意图匹配    │
        │ Complex: web-research │
        └───────────────────────┘
                    │
                    ▼
        ┌───────────────────────┐
        │ COGTOME Structure     │
        │ web-research         │
        │ 输入: {query, save_path} │
        └───────────────────────┘
                    │
                    ▼
        ┌───────────────────────┐
        │ Motif: web-research   │
        │ flow: search → fetch → extract → save │
        └───────────────────────┘
                    │
                    ▼
        ┌───────────────────────┐
        │ Unit: web-search      │
        │ 问题：这个 Unit 不存在！│
        └───────────────────────┘
```

### 2.2 发现的问题

**问题 2.2.1：Motif 需要一个不存在的 Unit**

```yaml
# motifs/web-research.yaml
flow:
  - name: search
    unit: web-search              # ❌ 没有这个 Unit！
    input:
      query: "${params.query}"
```

COGTOME 不知道去哪里找 `web-search`：
- 它应该在 `skills/units/web-search/` 吗？
- 还是应该在全局 `~/.cogtome/units/`？
- 还是需要完整路径 `skills/units/web-search/bin/web-search`？

**当前设计说"全局优先 → Complex 私有次之"，但这个优先级没有在代码里实现。**

---

**问题 2.2.2：搜索引擎选择没有声明**

```yaml
# 当前 Motif
- name: search
  unit: web-search
  input:
    query: "${params.query}"
    engine: ???  # 用百度？Google？DuckDuckGo？
```

`engine` 参数从哪里来？params 里没有声明。

---

**问题 2.2.3：网页内容提取的质量问题**

```
Motif flow:
  search → fetch → extract → save

问题：
1. search 返回的是 URL 列表
2. fetch 获取 HTML
3. extract 提取正文
4. save 保存

但 extract 的质量完全依赖第三方库（ readability、trafilatura 等），
Motif 无法声明"我要提取什么"，只能碰运气。
```

---

**问题 2.2.4：保存路径的目录创建**

```yaml
# Unit save-note 期望：
input:
  path: "notes/rust-concurrency.md"
  content: "..."

# 问题：
# 1. notes/ 目录不存在 → Unit 应该自动创建还是报错？
# 2. 文件已存在 → 应该覆盖、追加、还是报错？
# 3. 路径是相对路径 → 相对于谁？cwd？cogtome root？Home？
```

**Schema 没有声明这些约束。**

---

## 场景3：Git 代码审查

**任务**："审查 ~/project/src 的代码变更，给出审查意见"

### 3.1 执行流程

```
Structure: code-review
  ├── Motif: git-status-check
  │     └── Unit: git-status → {files: ["a.py", "b.py"]}
  │
  ├── Motif: git-diff-extract (对每个文件)
  │     └── Unit: git-diff → {diff: "..."}
  │
  ├── Motif: ai-review-generate
  │     └── Unit: ai-review → {comment: "..."}
  │
  └── Motif: save-review
        └── Unit: save-note
```

### 3.2 发现的问题

**问题 3.2.1：Motif 动态加载文件列表**

```yaml
# git-diff-extract Motif
flow:
  - name: diff
    unit: git-diff
    input:
      file: "${steps.status.output.files[0]}"  # ❌ 硬编码索引
```

**问题：如何对每个文件执行 git diff？**

Motif 没有"遍历"机制：
```yaml
# 想做：
for each file in ${steps.status.output.files}:
  - name: diff_${file}
    unit: git-diff
    input: {file: file}
```

当前 YAML 不支持循环。

---

**问题 3.2.2：AI review Unit 的 API 设计**

```yaml
# ai-review Unit
input:
  diff: "..."        # 大量文本，可能很长
  language: "zh"     # 审查语言
  style: "strict"    # 严格程度

output:
  comment: "..."     # AI 返回的意见
```

**问题：**
1. diff 可能很大（1000行+），AI 模型有 token 限制
2. diff 作为字符串放 YAML 里会很长，变量引用能处理吗？
3. AI review 服务用什么？OpenAI？Claude？本地模型？

---

**问题 3.2.3：执行结果累积**

```
git-status → 5 个文件
git-diff × 5 → 5 个 diff 结果
ai-review × 5 → 5 个 review 意见
save-note → 1 个文件

中间结果存在哪里？
- ${steps.diff_1.output.diff}
- ${steps.diff_2.output.diff}
- ...
- ${steps.review_1.output.comment}
- ${steps.review_2.output.comment}
- ...
```

**Motif 的 return 语句要写多少行？**

```yaml
return:
  file1_diff: "${steps.diff_1.output.diff}"
  file1_review: "${steps.review_1.output.comment}"
  file2_diff: "${steps.diff_2.output.diff}"
  file2_review: "${steps.review_2.output.comment}"
  # ... 手动列出所有文件！
```

**如果文件数量是动态的，这个 YAML 根本写不出来。**

---

**问题 3.2.4：二进制文件处理**

```bash
$ git diff image/logo.png
warning: Cannot binary files differ.

# Unit git-diff 输出：
{
  "diff": "warning: Cannot binary files differ.\nBinary files a/image/logo.png and b/image/logo.png differ",
  "is_binary": true
}
```

**问题：**
1. AI review 收到二进制文件的 diff，直接传给 AI 会出问题
2. Motif 需要判断 `is_binary` 来跳过 AI review
3. 但 YAML 条件判断 `${steps.diff.output.is_binary}` 支持吗？

---

## 发现的问题汇总

### P0 - 立即会失败

| ID | 问题 | 影响 |
|----|------|------|
| P0-1 | Motif 不支持循环（遍历文件列表） | 任何"对每个文件做 X"的任务都无法表达 |
| P0-2 | 变量引用不支持数组操作（length, filter, map） | 无法统计数量、过滤结果 |
| P0-3 | Unit 路径解析规则未实现 | 无法确定使用哪个 Unit |
| P0-4 | 动态数量结果无法累积 | "5个文件 → 5个review"这种模式无法表达 |

### P1 - 会导致错误行为

| ID | 问题 | 影响 |
|----|------|------|
| P1-1 | Schema 不声明目录创建/文件覆盖策略 | 行为不确定 |
| P1-2 | engine/search params 等参数没有来源 | Motif 写死或缺失 |
| P1-3 | AI review 的 token 限制未处理 | 大 diff 会导致模型失败 |
| P1-4 | 二进制文件未特殊处理 | 会传给 AI 或破坏 JSON |

### P2 - 工程体验问题

| ID | 问题 | 影响 |
|----|------|------|
| P2-1 | 简单任务 4 个文件 | 开发成本高 |
| P2-2 | 数组负索引不支持 | 不符合直觉 |
| P2-3 | 数字格式多样性（科学计数法、全角） | Unit 行为不一致 |
| P2-4 | 相对路径解析基准不明 | 保存位置不确定 |

---

## 待向"聪明人"请教的问题

1. **循环问题**：Motif YAML 如何支持"对列表中每个元素执行"这种模式？

2. **动态结果**：当文件数量是动态的时候，return 语句如何写？还是说这种场景不该用 YAML Motif？

3. **AI 集成**：AI review 这种"AI as Unit"的场景，是应该：
   - 把 AI 包装成 Unit（方案 A）
   - 还是 Motif 层直接调用 AI API（方案 B）
   - 还是 Structure 层做 AI 编排（方案 C）

4. **边界设计**：COGTOME 的 Scope 应该有多宽？
   - 只管"进程管理"（fork/exec、JSON契约）
   - 还是也管"业务逻辑"（循环、条件分支）？
