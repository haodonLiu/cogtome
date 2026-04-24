# COGTOME 架构决策记录

> 决策时间：2026-04-24
> 来源：与"聪明人"的讨论
> 版本：v2（修正 7 类具体问题）

---

## 一、Scope 边界（已确立）

**核心原则：COGTOME 不是 Agent，但必须是"带编排能力的执行引擎"**

| 能力 | 归属 | 原因 |
|------|------|------|
| 进程管理（fork/exec、JSON 契约） | ✅ COGTOME | 基础运行时 |
| 循环、条件分支、并行（控制流） | ✅ COGTOME | 执行纪律，不是智能决策 |
| 结果聚合、错误重试 | ✅ COGTOME | 执行可靠性 |
| 意图匹配、Complex 选择 | ❌ OpenClaw | Agent 的智能 |

---

## 二、P0 问题解决方案

### P0-1：Motif 循环语法 — `foreach` + `aggregate`

#### foreach 语法定义

```yaml
foreach:
  over: "${steps.status.output.files}"   # 要迭代的数组表达式（必填）
  as: file                               # 迭代变量名（必填）
  var_name: file                         # 别名，等同于 as
  max_iterations: 50                     # 安全上限（默认 50）
  on_error: fail_fast                    # fail_fast | continue（默认 fail_fast）
flow:
  - name: diff
    unit: git-diff
    input:
      file: "${item}"                   # 引用当前迭代项
  - name: review
    if: "${steps.diff.output.is_binary} == false"
    unit: ai-review
    input:
      diff: "${steps.diff.output.diff}"
aggregate:
  mode: array                           # array | object | sum | join
  map:                                  # 仅 array/object 模式需要
    file: "${item}"
    is_binary: "${steps.diff.output.is_binary}"
    review: "${steps.review.output.comment}"  # 被跳过时为 null
```

#### 内置变量

| 变量 | 作用域 | 说明 |
|------|--------|------|
| `item` | foreach 子 flow 全局 | 当前迭代项 |
| `__index` | foreach 子 flow 全局 | 当前迭代索引（从 0 开始） |
| `__error` | 仅在 `on_error: continue` 的 aggregate.map 中 | 当前迭代的错误信息（如有） |

**注意：`__error` 仅在 aggregate 阶段可用，不可在 foreach 内部的后续 step 中使用。**

#### foreach 完整示例（修正笔误）

```yaml
# motifs/code-review.yaml
flow:
  - name: status
    unit: git-status

  - name: review_loop
    foreach:
      over: "${steps.status.output.files}"
      as: file
      max_iterations: 50
      on_error: fail_fast
    flow:
      - name: diff
        unit: git-diff
        input:
          file: "${item}"

      - name: review
        if: "${steps.diff.output.is_binary} == false"
        unit: ai-review
        input:
          diff: "${steps.diff.output.diff}"
          language: "${params.language}"

    aggregate:
      mode: array
      map:
        file: "${item}"
        is_binary: "${steps.diff.output.is_binary}"
        review: "${steps.review.output.comment}"

  - name: save
    unit: save-note
    input:
      path: "${params.save_path}"
      content: "${steps.review_loop.aggregate}"

return:
  file_count: "${steps.status.output.files.length}"
  reviews: "${steps.review_loop.aggregate}"
  # Phase 2 支持：reviewed_count: "${length(filter(steps.review_loop.aggregate, 'review != null'))}"
```

**⚠️ 修正说明（v2）：**
- 原文档 `steps.report` → 不存在，已移除
- 原文档 `steps.reviews.aggregate` → 应为 `steps.review_loop.aggregate`，已修正
- Phase 1 `return` 不支持 `filter` 函数，仅输出原始数组

#### aggregate 模式

| 模式 | 用途 | 语法 |
|------|------|------|
| `array` | 收集所有结果为数组 | `map` 定义每个元素的字段 |
| `object` | 按键聚合 | `key: "${item.filename}"`，`value: "${steps.review.output}"` |
| `sum` | 数值累加 | `sum: "${steps.diff.output.lines_added}"` |
| `join` | 字符串拼接 | `join: "${steps.review.output.comment}"`，`separator: "\n\n"` |

---

### P0-2：变量引用增强 — 轻量表达式引擎

#### Phase 1 vs Phase 2 边界（已修正矛盾）

**Phase 1（立即实现）：**
- 变量引用：`${steps.a.output}`, `${steps.a.output.field[0]}`
- 索引访问：`[0]`, `[-1]`
- 长度属性：`.length`
- 比较运算：`==`, `!=`, `>`, `<`, `>=`, `<=`
- 逻辑运算：`&&`, `||`, `!`
- 三目运算：`condition ? a : b`

**Phase 2（后续实现）：**
- 内置函数：`filter()`, `map()`, `length()`, `join()`
- 简单 lambda：`arr.filter(x => x > 5)`
- 管道操作

#### Phase 1 return 示例（可实际运行）

```yaml
return:
  file_count: "${steps.status.output.files.length}"
  reviews: "${steps.review_loop.aggregate}"
  first_file: "${steps.status.output.files[0]}"
  last_file: "${steps.status.output.files[-1]}"
  has_binary: "${steps.status.output.files.length > 0}"
```

**⚠️ 修正说明（v2）：**
- 原文档 `filter()` 出现在 Phase 1 示例中，但属于 Phase 2
- 已将 `reviewed_count` 改为 Phase 1 可运行的形式
- 明确 Phase 边界，避免实现歧义

---

### P0-3：Unit 路径解析规则 — 三级查找

```rust
enum UnitResolution {
  // 1. 当前 Complex 的私有 Unit（最优先）
  ComplexLocal {
    base: PathBuf,  // Complex 根目录
    unit_name: &str,
  },

  // 2. 全局注册表（次优先）— 使用 dirs crate，跨平台
  GlobalRegistry {
    path: PathBuf,  // 因平台而异（见下方）
  },

  // 3. 系统 PATH（兜底）
  SystemPath { name: &str },
}
```

**平台默认路径：**

| 平台 | 默认路径 | 说明 |
|------|----------|------|
| Linux | `~/.local/share/cogtome/units/` | XDG Data Dir |
| macOS | `~/Library/Application Support/cogtome/units/` | Cocoa NSApplicationSupport |
| Windows | `C:\Users\<User>\AppData\Roaming\cogtome\units\` | Win CSIDL_APPDATA |

**环境变量覆盖：**
```bash
export COGTOME_UNITS_PATH=/custom/path
```

### Complex 路径解析（新增）

Complex 与 Unit 共用同一根目录：

```
~/.local/share/cogtome/
├── units/              # Unit 查找路径
│   ├── web-search/
│   └── git-diff/
└── complexes/          # Complex 查找路径
    ├── text-processing/
    └── web-research/
```

**环境变量覆盖：**
```bash
export COGTOME_COMPLEXES_PATH=/custom/complexes
```

Agent 调用时：`cogtome run <complex_name>` → 在 complexes 目录下查找 `<complex_name>/SKILL.md`。

---

## 三、快照语义实现约束

### 实现细节（已修正 O(1) 注释）

```rust
// StepState 内部使用 Arc<Value>，clone 为 O(1)
struct StepState {
    data: Arc<serde_json::Value>,
}

// global_steps 本身是 Arc<HashMap>，clone 时只复制指针
let snapshot: Arc<HashMap<String, StepState>> = Arc::new(ctx.global_steps.clone());
//                                          ^
//                                          只有指针复制，是 O(1)

for item in list {
    let config = snapshot.get("b"); // 永远一致，无深拷贝
}
```

**⚠️ 修正说明（v2）：**
- 原文档 `Arc::new(steps.clone())` 实际是 O(n)（先深拷贝 HashMap 再包 Arc）
- 正确做法：`ctx.global_steps` 本身就是 `Arc<HashMap>`，直接 `Arc::clone(&ctx.global_steps)` 是 O(1)
- 已修正代码示例

### 三个边界情况

**1. 并发一致性** — 快照在迭代开始前一次性捕获（Arc 引用，O(1)）

**2. 禁止修改外部状态** — foreach 内部维护局部 scope，迭代结束后通过 aggregate 显式输出

**3. 内存优化** — `StepState.data` 是 `Arc<serde_json::Value>`，内部 clone 也是 O(1)

---

## 四、变量遮蔽规则

**允许遮蔽，禁止写入：**

```yaml
- name: config          # 外部 step
  unit: load_config

- name: loop
  foreach:
  flow:
  - name: config        # ✅ 允许：遮蔽外部 config
    unit: some_other
```

**解析优先级：**
```
1. 迭代变量（item, __index）
2. 当前局部 steps（foreach 内部产生的）
3. 快照中的外部 steps（只读）
4. 用户输入 params
```

---

## 五、边界情况处理汇总（新增）

### 统一规则

| 场景 | 处理策略 | 理由 |
|------|----------|------|
| `if` 表达式解析失败 | 视为 `false`（跳过） | 保守策略：不确定时跳过，避免执行不该执行的 step |
| `foreach.over` 不存在 | 视为**空数组**，继续执行（生成 0 次迭代） | `fail_fast` 对空数组无意义 |
| `foreach.over` 求值非数组 | 抛错 | 类型错误，必须修复 |
| `return` 引用缺失的 step | **抛错** | return 是最终输出，静默 null 会隐藏 bug |
| aggregate.map 引用不存在的 step | 静默返回 `null` + 警告日志 | 被 if 跳过的 step 语义上就是未执行 |
| `on_error: continue` 迭代失败 | 记录 `__error`，继续下一迭代 | 部分成功是有价值的信息 |
| `on_error: fail_fast` 任一迭代失败 | **不产出 aggregate**，直接向上抛错 | 语义干净，避免脏数据传播 |

### Aggregate object 模式语法

```yaml
aggregate:
  mode: object
  key: "${item.filename}"           # 必填：作为键的表达式
  value: "${steps.review.output}"    # 作为值的表达式
```

**示例：**
```yaml
aggregate:
  mode: object
  key: "${item}"
  value: "${steps.review.output.comment}"
# 结果：{"a.py": "LGTM", "b.py": "needs fix"}
```

---

## 六、错误处理策略

### foreach 错误策略

| 策略 | 行为 |
|------|------|
| `fail_fast`（默认） | 任一迭代失败，**不产出 aggregate**，直接向上传播错误 |
| `continue` | 跳过失败迭代，继续处理，`__error` 记录错误信息 |

```yaml
foreach:
  over: "${files}"
  as: file
  on_error: continue
  flow:
  - unit: risky_operation
  aggregate:
    mode: array
    map:
      file: "${item}"
      result: "${steps.risky.output}"  # 失败时为 null
      error: "${__error}"              # 仅在失败时有值
```

---

## 七、并行 foreach（Phase 2）

### 核心原则：默认保守，显式放行

**未声明并发安全的 Unit，在并行 foreach 中默认串行化。**

### Unit 级别并发声明

```yaml
# units/ai-review/manifest.yaml
name: ai-review
concurrency:
  max_global: 3      # 全局最多 3 个并发实例
  max_per_host: 1    # 单主机最多 1 个
  resource_key: "openai_api"  # 共享资源标识
```

**如果不声明 `concurrency` 字段：**

| 场景 | 行为 |
|------|------|
| 串行 foreach | 正常执行 |
| 并行 foreach | 该 Unit 被强制串行化 |

### Phase 2 语法（预留）

```yaml
foreach:
  over: "${files}"
  as: file
  parallel: true
  max_concurrency: 5
  flow:
  - unit: git-diff
  - unit: ai-review
```

---

## 八、if 条件分类

| 类型 | 定义 | 示例 | 归属 |
|------|------|------|------|
| 确定性条件 | 基于结构化数据的布尔判断 | `is_binary == false`, `file_size > 1000` | ✅ Motif 编排 |
| AI 输出条件 | 基于 AI 生成内容的阈值判断 | `quality > 0.8` | ⚠️ 谨慎使用 |

**最佳实践：** Motif 的 `if` 只接收布尔值，不解释数值含义。数值到布尔的转换应该在 Unit 内完成。

---

## 九、实施优先级（最终版）

```
Phase 1（立即，阻塞所有场景）：
├── P0-3 Unit/Complex 路径解析（三级查找 + dirs crate）
├── P0-1 foreach 循环语法（as, over, flow, aggregate）
├── P0-4 aggregate 聚合（array/object/sum/join）
├── P0-2 表达式引擎 Phase 1 子集
├── 变量遮蔽规则
├── 边界情况处理规则
└── max_iterations 硬限制

Phase 2（后续，解锁复杂场景）：
├── foreach parallel: true
├── Unit concurrency 声明 + Runtime 限流
├── P0-2 表达式引擎增强（filter, join, length）
├── P1-1 Schema 约束扩展
├── P1-2 默认值机制
└── P1-3 AI token 分块

Phase 3（体验优化）：
├── P2-1 auto-complex 快捷注册
└── P1-4 AI 输出条件分类规范
```

---

## 十、配置项（已修正路径示例）

```toml
# cogtome.toml
[runtime]
max_iterations = 50           # 默认上限
max_iterations_hard = 500     # 绝对上限

[paths]
# 默认值因平台而异，无需显式配置：
# Linux:   ~/.local/share/cogtome/
# macOS:   ~/Library/Application Support/cogtome/
# Windows: %APPDATA%/cogtome/
# 如需自定义，通过环境变量覆盖：
# COGTOME_UNITS_PATH
# COGTOME_COMPLEXES_PATH
```

**⚠️ 修正说明（v2）：**
- 原文档 `units = "~/.cogtome/units"` 与实际平台默认值不符
- 已修正为"默认值因平台而异"，示例改为说明文字
- 明确 `~` 展开由 cogtome 自行处理

---

## 十一、错误信息示例

```
Error: MaxIterationsExceeded
  foreach 'review_loop' attempted 51 iterations (limit: 50).
  Hint: Increase max_iterations in cogtome.toml or ask Agent to batch process.

Error: InvalidAggregateExpression
  Aggregate map expression '${steps.nonexistent.output}' resolved to null.
  (This is a warning, not an error. Check your if conditions.)

Error: TypeMismatch
  'over' expression resolved to string, expected array.
```
