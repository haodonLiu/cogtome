# COGTOME 架构决策记录

> 决策时间：2026-04-24
> 来源：与"聪明人"的讨论

---

## 一、Scope 边界（已确立）

**核心原则：COGTOME 不是 Agent，但必须是"带编排能力的执行引擎"**

| 能力 | 归属 | 原因 |
|------|------|------|
| 进程管理（fork/exec、JSON 契约） | ✅ COGTOME | 基础运行时 |
| 循环、条件分支、并行（控制流） | ✅ COGTOME | 执行纪律，不是智能决策 |
| 结果聚合、错误重试 | ✅ COGTOME | 执行可靠性 |
| 意图匹配、Complex 选择 | ❌ OpenClaw | Agent 的智能 |

**关键区分：**
- **编排（Orchestration）** = 确定性控制流。`for each file in list` 不需要智能，只需要执行纪律。
- **决策（Decision）** = 非确定性选择。`哪个 Complex 最适合这个任务` 需要理解上下文，是 Agent 的事。

---

## 二、P0 问题解决方案

### P0-1：Motif 循环语法 — `foreach` + `aggregate`

```yaml
# motifs/code-review.yaml
flow:
 - name: status
   unit: git-status

 - name: review_loop
   foreach:
     over: "${steps.status.output.files}"
     as: file              # 迭代变量名
     max_iterations: 50   # 安全上限
   flow:
   - name: diff
     unit: git-diff
     input:
       file: "${item}"     # 引用当前迭代项

   - name: review
     unit: ai-review
     input:
       diff: "${steps.diff.output.diff}"
       language: "${params.language}"

   aggregate:
     mode: array           # 收集为数组
     map:
       file: "${item}"
       diff: "${steps.diff.output.diff}"
       review: "${steps.review.output.comment}"

 - name: save
   unit: save-note
   input:
     path: "${params.save_path}"
     content: "${steps.review_loop.aggregate.reviews}"
```

**关键设计：**
- `foreach` 是容器节点，内部有自己的子 flow
- `aggregate` 定义如何收集结果，避免手动写 `return`
- `max_iterations` 防止无限循环

**aggregate 模式：**

| 模式 | 用途 | 示例 |
|------|------|------|
| `array` | 收集所有结果为数组 | 5 个文件 → 5 个 review |
| `object` | 按键聚合 | `{"a.py": review1, "b.py": review2}` |
| `sum` | 数值累加 | 统计总行数 |
| `join` | 字符串拼接 | 合并所有 diff |

---

### P0-2：变量引用增强 — 轻量表达式引擎

引入表达式引擎（Rust `evalexpr` 或类似），支持：

```yaml
# 基础操作
count: "${steps.extract.output.numbers.length}" # 数组长度
first: "${steps.extract.output.numbers[0]}"      # 正索引（已有）
last: "${steps.extract.output.numbers[-1]}"       # 负索引（新增）
slice: "${steps.extract.output.numbers[0:3]}"     # 切片（新增）

# 过滤
text_files: "${steps.files.output.files.filter(f => !f.is_binary)}"

# 条件
- name: maybe_review
  if: "${steps.diff.output.is_binary} == false"
  unit: ai-review
  input:
    diff: "${steps.diff.output.diff}"
```

**实现建议：** 表达式引擎只读，不修改状态。复杂逻辑（如 `filter`）映射到内置函数，而非通用 JavaScript。

---

### P0-3：Unit 路径解析规则 — 三级查找

```rust
enum UnitResolution {
  // 1. 当前 Complex 的私有 Unit（最优先）
  ComplexLocal { path: "skills/complex/{complex_name}/units/{unit_name}" },
  
  // 2. 全局注册表（次优先）
  GlobalRegistry { path: "~/.cogtome/units/{unit_name}" },
  
  // 3. 系统 PATH（兜底）
  SystemPath { name: "{unit_name}" },
}
```

**注册机制：**

```bash
# 注册到全局
cogtome register unit ./web-search.py --name web-search --global

# 注册到特定 Complex 私有
cogtome register unit ./web-search.py --name web-search --complex web-research
```

**运行时查找顺序：** Complex 本地 → 全局注册表 → `$PATH`

---

### P0-4：动态结果累积 — 配合 foreach 解决

```yaml
# 完整示例：Git 代码审查
flow:
 - name: status
   unit: git-status

 - name: reviews
   foreach:
     over: "${steps.status.output.files}"
     as: file
     flow:
     - name: diff
       unit: git-diff
       input: { file: "${item}" }
     - name: review
       if: "${steps.diff.output.is_binary} == false"  # 条件跳过
       unit: ai-review
       input: { diff: "${steps.diff.output.diff}" }
     aggregate:
       mode: array
       map:
         file: "${item}"
         is_binary: "${steps.diff.output.is_binary}"
         review: "${steps.review.output.comment}"  # 跳过时为 null

 - name: report
   unit: generate-report
   input:
     reviews: "${steps.reviews.aggregate}"  # 完整数组

return:
   report: "${steps.report.output}"
   file_count: "${steps.status.output.files.length}"
   reviewed_count: "${steps.reviews.aggregate.filter(r => r.review != null).length}"
```

---

## 三、P1/P2 问题解决方案

| ID | 问题 | 解决方案 |
|----|------|----------|
| P1-1 | 目录创建/覆盖策略 | Schema 增加 `constraints: {auto_mkdir: true, on_exists: overwrite\|fail\|append}` |
| P1-2 | 搜索引擎参数来源 | Structure manifest `defaults: {engine: duckduckgo}`，Motif 用 `${params.engine \|\| defaults.engine}` |
| P1-3 | AI token 限制 | Unit 层处理（分块），或 Motif 增加 `preprocess` 步骤拆分 diff |
| P1-4 | 二进制文件 | `if` 条件跳过（见 P0-4 示例） |
| P2-1 | 简单任务 4 文件 | `auto-complex` 快捷注册（之前讨论） |
| P2-2 | 负索引 | 表达式引擎支持（见 P0-2） |
| P2-3 | 数字格式 | Unit 职责，通过单元测试约束，不在 Motif 层处理 |
| P2-4 | 相对路径基准 | 约定：Unit cwd = Complex 根目录；支持 `~/` 和 `./` |

---

## 四、实施优先级

```
Phase 1（本周，阻塞所有场景）：
├── P0-3 Unit 路径解析（没有它连场景1都跑不通）
├── P0-1 foreach 循环语法（场景3的核心）
└── P0-4 aggregate 聚合（配合循环使用）

Phase 2（下周，解锁复杂场景）：
├── P0-2 表达式引擎（length, filter, if）
├── P1-1 Schema 约束扩展
└── P1-2 默认值机制

Phase 3（后续，体验优化）：
├── P2-1 auto-complex 快捷注册
└── P1-3 AI token 分块
```

---

## 五、待验证问题

**foreach 内部是否允许引用外部 steps 的变量？**

```yaml
foreach:
  over: "${steps.a.output.list}"
  as: item
  flow:
  - unit: process
    input:
      data: "${item}"                  # 当前迭代项
      config: "${steps.b.output.config}"  # 外部步骤的结果？
```

**如果允许：** COGTOME 需要在每次迭代时冻结外部 steps 的状态。

**如果不允许：** 所有需要的变量必须在 `over` 中传入，外部变量不可引用。

**建议：** 允许引用，但声明为"快照"语义——迭代开始前的值，迭代过程中不变。
