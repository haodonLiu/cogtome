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
     if: "${steps.diff.output.is_binary} == false"
     unit: ai-review
     input:
       diff: "${steps.diff.output.diff}"
       language: "${params.language}"

   aggregate:
     mode: array           # 收集为数组
     map:
       file: "${item}"
       diff: "${steps.diff.output.diff}"
       review: "${steps.review.output.comment}"  # 跳过时为 null

 - name: save
   unit: save-note
   input:
     path: "${params.save_path}"
     content: "${steps.review_loop.aggregate.reviews}"

return:
   report: "${steps.report.output}"
   file_count: "${steps.status.output.files.length}"
   reviewed_count: "${length(filter(steps.reviews.aggregate, 'review != null'))}"
```

**关键设计：**
- `foreach` 是容器节点，内部有自己的子 flow
- `aggregate` 定义如何收集结果，避免手动写 `return`
- `max_iterations` 防止无限循环
- **默认 `fail_fast`**，可选 `on_error: continue`

**aggregate 模式：**

| 模式 | 用途 | 示例 |
|------|------|------|
| `array` | 收集所有结果为数组 | 5 个文件 → 5 个 review |
| `object` | 按键聚合 | `{"a.py": review1, "b.py": review2}` |
| `sum` | 数值累加 | 统计总行数 |
| `join` | 字符串拼接 | 合并所有 diff |

---

### P0-2：变量引用增强 — 轻量表达式引擎

**Phase 1 支持的表达式子集：**

```yaml
# 基础
"${steps.a.output}"                         # 变量
"${steps.a.output.numbers[0]}"             # 索引
"${steps.a.output.numbers.length}"          # 长度属性
"${steps.a.output.numbers[-1]}"             # 负索引

# 内置函数（Motif 引擎提供）
"${filter(steps.reviews, 'review != null')}" # 过滤
"${length(steps.reviews)}"                  # 计数
"${join(steps.diffs, '\n\n')}"              # 拼接
```

**不支持（Phase 1）：**
- Method chaining：`steps.a.filter()`
- Lambda：`arr.filter(x => x > 5)`

**Phase 2 再考虑：**
- 简单 lambda
- 管道操作

---

### P0-3：Unit 路径解析规则 — 三级查找

```rust
enum UnitResolution {
  // 1. 当前 Complex 的私有 Unit（最优先）
  ComplexLocal { path: "skills/complex/{complex_name}/units/{unit_name}" },
  
  // 2. 全局注册表（次优先）— 使用 dirs crate，跨平台
  GlobalRegistry { path: data_dir().join("cogtome").join("units") },
  
  // 3. 系统 PATH（兜底）
  SystemPath { name: "{unit_name}" },
}
```

**平台路径：**
- Linux: `~/.local/share/cogtome/units/`
- macOS: `~/Library/Application Support/cogtome/units/`
- Windows: `C:\Users\<User>\AppData\Roaming\cogtome\units\`

**环境变量覆盖：**
```bash
export COGTOME_UNITS_PATH=/custom/path
```

---

### P0-4：动态结果累积 — 配合 foreach 解决

见上方 P0-1 示例，aggregate 自动收集迭代结果，无需手动列举。

---

## 三、快照语义实现约束

### 三个边界情况

**1. 并发一致性** — 快照在迭代开始前一次性捕获
```rust
let snapshot = Arc::new(steps.clone()); // Arc::clone = O(1)
for item in list {
    let config = snapshot.get("b"); // 永远一致
}
```

**2. 禁止修改外部状态** — foreach 内部维护局部 scope，迭代结束后通过 aggregate 显式输出

**3. 内存优化**
```rust
struct StepState {
    data: Arc<serde_json::Value>, // Arc::clone = O(1)
}
```

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
    # 这个 config 的输出只存在于 foreach 局部 scope
```

**解析优先级：**
```
1. 迭代变量（item, __index）
2. 当前局部 steps（foreach 内部产生的）
3. 快照中的外部 steps（只读）
4. 用户输入 params
```

---

## 五、错误处理策略

### foreach 错误策略

| 策略 | 行为 |
|------|------|
| `fail_fast`（默认） | 任一迭代失败，**不产出 aggregate**，直接向上传播错误 |
| `continue` | 跳过失败迭代，继续处理，记录错误到 aggregate |

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
      error: "${__error}"              # 内置变量：错误信息
```

---

## 六、并行 foreach（Phase 2）

### 核心原则：默认保守，显式放行

**未声明并发安全的 Unit，在并行 foreach 中默认串行化。**

### Unit 级别并发声明

```yaml
# units/ai-review/manifest.yaml
name: ai-review
concurrency:
  max_global: 3      # 全局最多 3 个并发实例（跨所有 foreach）
  max_per_host: 1    # 单主机最多 1 个（防止 GPU 内存耗尽）
  resource_key: "openai_api"  # 共享资源标识，同 key 的 Unit 共享配额
```

**如果不声明 `concurrency` 字段：**

| 场景 | 行为 |
|------|------|
| 串行 foreach | 正常执行 |
| 并行 foreach | 该 Unit 被强制串行化，其他 Unit 仍可并行 |

### Phase 2 并行设计

```yaml
# Phase 2 语法（预留）
foreach:
  over: "${files}"
  as: file
  parallel: true
  max_concurrency: 5  # 并发上限
  flow:
  - unit: git-diff
  - unit: ai-review
```

**Runtime 调度：**
- Motif 作者写 `parallel: true`，不感知底层限流
- Runtime 根据 Unit `concurrency` 声明自动调度
- 有 Unit 未声明安全 → 整个迭代串行化

---

## 七、if 条件分类

**Motif 层的 `if` 是确定性编排，文档必须区分：**

| 类型 | 定义 | 示例 | 归属 |
|------|------|------|------|
| 确定性条件 | 基于结构化数据的布尔判断，结果可预期 | `is_binary == false`, `file_size > 1000` | ✅ Motif 编排 |
| AI 输出条件 | 基于 AI 生成内容的阈值判断，结果不确定 | `quality > 0.8`, `sentiment == "positive"` | ⚠️ 谨慎使用 |

**最佳实践：**

```
Motif 的 if 只接收布尔值，不解释数值含义。
数值到布尔的转换应该在 Unit 内完成。
```

| ❌ 不推荐 | ✅ 推荐 |
|----------|--------|
| `if: "${steps.review.output.quality} > 0.8"` | `if: "${steps.quality_check.output.passed}"` |
| Motif 直接解析 AI 输出 | AI 判断封装在 Unit 内，输出明确的布尔字段 |

---

## 八、Windows 路径支持

使用 `dirs` crate，而非硬编码 `~/.cogtome`：

```rust
use dirs::data_dir;

fn global_units_path() -> PathBuf {
    data_dir()
        .expect("无法确定数据目录")
        .join("cogtome")
        .join("units")
}
```

同时支持环境变量覆盖：`COGTOME_UNITS_PATH`

---

## 九、Aggregate null 处理规则

**聚合阶段引用不存在的 step 时：静默返回 `null`，记录警告日志。**

```rust
fn resolve_for_aggregate(expr: &str, ctx: &IterationContext) -> Value {
    match resolve_variable(expr, ctx) {
        Some(v) => v,
        None => {
            log::warn!("Aggregate map 中 '{}' 解析失败，使用 null", expr);
            Value::Null
        }
    }
}
```

**文档明确：**
> 聚合阶段引用不存在的 step 时，返回 `null` 并记录警告。如需严格检查，在 `return` 阶段用表达式引擎做非空校验。

---

## 十、实施优先级

```
Phase 1（本周，阻塞所有场景）：
├── P0-3 Unit 路径解析（没有它连场景1都跑不通）
├── P0-1 foreach 循环语法（场景3的核心）
├── P0-4 aggregate 聚合（配合循环使用）
└── P0-2 表达式引擎基础（变量 + 索引 + length）

Phase 2（下周，解锁复杂场景）：
├── P0-2 表达式引擎增强（filter, length, join）
├── P1-1 Schema 约束扩展
├── P1-2 默认值机制
├── foreach parallel: true（并发迭代）
└── Unit concurrency 声明 + Runtime 限流（并行安全的前提）

Phase 3（后续，体验优化）：
├── P2-1 auto-complex 快捷注册
├── P1-3 AI token 分块
└── P2-2 负索引（已在 P0-2 覆盖）
```

---

## 十一、配置项

```toml
# cogtome.toml
[runtime]
max_iterations = 50           # 默认上限
max_iterations_hard = 500    # 绝对上限，防止配置错误

[paths]
units = "~/.cogtome/units"    # 可通过 COGTOME_UNITS_PATH 覆盖
```

**错误信息示例：**
```
Error: MaxIterationsExceeded
 foreach 'review_loop' attempted 51 iterations (limit: 50).
 Hint: Increase max_iterations in cogtome.toml or ask Agent to batch process.
```
