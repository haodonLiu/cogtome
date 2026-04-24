# COGTOME 综合测试用例集

> 创建时间：2026-04-24
> 目的：覆盖所有可能的使用场景，包括架构讨论反馈流程

---

## 一、基础执行测试

### 1.1 Unit 执行

| ID | 场景 | 命令 | 期望 |
|----|------|------|------|
| U-01 | 正常执行 | `unit run text-uppercase --input '{"text":"hello"}'` | `{"result":"HELLO"}` |
| U-02 | 多 Unit 组合 | `unit run text-reverse --input '{"text":"hello"}'` | `{"result":"olleh"}` |
| U-03 | Unit 不存在 | `unit run nonexistent --input '{}'` | `Error: Unit 'nonexistent' not found` |
| U-04 | JSON 格式错误 | `unit run text-uppercase --input 'not-json'` | JSON 解析错误 |
| U-05 | 缺少必需参数 | `unit run text-uppercase --input '{}'` | 根据 Unit 实现决定 |
| U-06 | 超长输入 | `unit run text-uppercase --input '{"text":"x"*10000}'` | 正常处理或超时 |
| U-07 | 特殊字符 | `unit run text-uppercase --input '{"text":"中文测试!@#$"}'` | 正确处理 |

### 1.2 Motif 执行

| ID | 场景 | 命令 | 期望 |
|----|------|------|------|
| M-01 | 正常 Motif | `motif run text-transform --input '{"text":"hello"}'` | 返回多个字段 |
| M-02 | Motif 不存在 | `motif run nonexistent --input '{}'` | 错误 |
| M-03 | 引用不存在的 Unit | (在 YAML 中配置) | 运行时错误 |
| M-04 | 变量解析成功 | `${params.text}` | 正确替换 |
| M-05 | 变量引用嵌套 | `${steps.upper.output.result}` | 正确解析 |

### 1.3 Structure 执行

| ID | 场景 | 命令 | 期望 |
|----|------|------|------|
| S-01 | 正常 Structure | `structure run text-pipeline --input '{"text":"hello"}'` | 返回结果 |
| S-02 | Structure 不存在 | `structure run nonexistent --input '{}'` | 错误 |
| S-03 | 校验失败 | input 缺少必需字段 | Schema 验证错误 |

### 1.4 Complex 执行

| ID | 场景 | 命令 | 期望 |
|----|------|------|------|
| C-01 | 正常 Complex | `run text-processing --input '{"text":"hello"}'` | 返回结果 |
| C-02 | Complex 不存在 | `run nonexistent --input '{}'` | 错误 |
| C-03 | 空输入 | `run text-processing --input '{}'` | 根据 Schema 验证 |
| C-04 | 复杂输入 | `run text-processing --input '{"text":"Hello World 123!"}'` | 正确处理 |

---

## 二、发现与索引测试

### 2.1 Discovery

| ID | 场景 | 命令 | 期望 |
|----|------|------|------|
| D-01 | 发现所有 Complex | `discover` | 列出所有 Complex |
| D-02 | 无 Complex | (清空 skills 目录) | `Found 0 Complex(es)` |
| D-03 | 重复 Complex 名 | (创建两个同名的) | 报错或覆盖 |
| D-04 | Complex 路径不存在 | `COGTOME_SKILLS_DIR=/nonexistent` | 错误或空结果 |

### 2.2 路径解析

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| P-01 | 默认路径 | 无环境变量 | 找到 skills |
| P-02 | 自定义路径 | `COGTOME_SKILLS_DIR=/custom` | 从自定义路径加载 |
| P-03 | 相对路径 | `COGTOME_SKILLS_DIR=./relative` | 正确解析 |
| P-04 | 嵌套 Complex | `skills/<complex>/units/<unit>/bin/<unit>` | 正确查找 |

---

## 三、表达式引擎测试

### 3.1 变量解析

| ID | 表达式 | 输入 context | 期望 |
|----|--------|-------------|------|
| E-01 | `${params.text}` | `params={"text":"hello"}` | `"hello"` |
| E-02 | `${steps.a.output.x}` | `steps={"a":{"output":{"x":1}}}` | `1` |
| E-03 | `${env.HOME}` | `env={"HOME":"/home/user"}` | `"/home/user"` |
| E-04 | `${params.missing}` | `params={}` | null 或错误 |
| E-05 | `${steps.nonexistent.output}` | `steps={}` | null 或错误 |

### 3.2 索引与属性

| ID | 表达式 | 输入 | 期望 |
|----|--------|------|------|
| E-10 | `${list[0]}` | `list=["a","b"]` | `"a"` |
| E-11 | `${list[-1]}` | `list=["a","b"]` | `"b"` |
| E-12 | `${obj.field}` | `obj={"field":"value"}` | `"value"` |
| E-13 | `${nested.a.b.c}` | `nested={"a":{"b":{"c":1}}}` | `1` |
| E-14 | `${list[99]}` | `list=["a"]` | null 或越界错误 |
| E-15 | `${list.length}` | `list=["a","b","c"]` | `3` |

### 3.3 运算

| ID | 表达式 | 输入 | 期望 |
|----|--------|------|------|
| E-20 | `${a + b}` | `a=1,b=2` | `3` |
| E-21 | `${a > b}` | `a=5,b=3` | `true` |
| E-22 | `${a == b}` | `a=1,b=1` | `true` |
| E-23 | `${a ? 'yes' : 'no'}` | `a=true` | `"yes"` |
| E-24 | `${'hello' ++ ' ' ++ 'world'}` | (空) | `"hello world"` |

---

## 四、Foreach 循环测试

### 4.1 基本循环

| ID | 场景 | YAML 配置 | 期望 |
|----|------|----------|------|
| F-01 | 空数组 | `over: []` | 0 次迭代，返回空 |
| F-02 | 单元素 | `over: [1]` | 1 次迭代 |
| F-03 | 多元素 | `over: [1,2,3]` | 3 次迭代 |
| F-04 | item 引用 | `${item}` 在 input 中 | 正确替换 |
| F-05 | __index 引用 | `${__index}` | 0,1,2,... |

### 4.2 安全限制

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| F-10 | max_iterations 正常 | `over: [1..5], max: 10` | 正常完成 |
| F-11 | max_iterations 超限 | `over: [1..100], max: 50` | `MaxIterationsExceeded` |
| F-12 | 超大数组 | `over: [1..10000]` | 根据 max_iterations 决定 |

### 4.3 条件执行

| ID | 场景 | if 条件 | 期望 |
|----|------|---------|------|
| F-20 | if 跳过 | `if: "${item} > 5"`，item=3 | 跳过此迭代 |
| F-21 | if 执行 | `if: "${item} > 5"`，item=7 | 执行此迭代 |
| F-22 | if 表达式错误 | `if: "${nonexistent}"` | 视为 false |
| F-23 | if 条件部分无效 | 数组中部分满足条件 | 只执行满足的 |

### 4.4 嵌套循环

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| F-30 | 嵌套 foreach | 外层 2 项，内层 3 项 | 6 次内层执行 |
| F-31 | 嵌套 item 重名 | 内外都用 `${item}` | 内层遮蔽外层 |

---

## 五、Aggregate 聚合测试

### 5.1 array 模式

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| A-01 | 正常收集 | 3 次迭代，每次返回 `x` | `[x1,x2,x3]` |
| A-02 | 空数组 | 0 次迭代 | `[]` |
| A-03 | map 字段 | `map: {v: "${item}"}` | `[{v:1},{v:2},{v:3}]` |
| A-04 | 引用跳过步骤 | 步骤被 if 跳过 | `null` + 警告日志 |

### 5.2 object 模式

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| A-10 | 正常对象 | `key: "${item.k}", value: "${item.v}"` | `{k1:v1, k2:v2}` |
| A-11 | 重复 key | key 重复 | 覆盖或报错 |
| A-12 | 缺少 key | key 表达式求值失败 | null 或跳过 |

### 5.3 sum/join 模式

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| A-20 | sum | `sum: "${item.count}"` | 总和数值 |
| A-21 | join | `join: "${item}", separator: "\n"` | 拼接字符串 |
| A-22 | sum 非数值 | sum 字符串 | 错误或 NaN |

---

## 六、错误处理测试

### 6.1 foreach 错误策略

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| H-01 | fail_fast 单次失败 | 迭代 2 失败 | 不返回 aggregate，直接报错 |
| H-02 | continue 单次失败 | 迭代 2 失败 | 继续 3,4,5，记录 `__error` |
| H-03 | 全部失败 | 3 次迭代全部失败 | fail_fast 直接停，continue 返回部分结果 |

### 6.2 边界情况

| ID | 场景 | 期望行为 |
|----|------|----------|
| H-10 | `foreach.over` 不存在 | 视为空数组，0 次迭代 |
| H-11 | `foreach.over` 非数组 | 类型错误，报错 |
| H-12 | `return` 引用缺失 | **报错**，不静默 |
| H-13 | `aggregate.map` 引用缺失 | 静默 null + 警告日志 |

### 6.3 层级错误码

| ID | 场景 | 期望 layer |
|----|------|------------|
| H-20 | Unit 执行失败 | `layer: "unit"` |
| H-21 | Motif 表达式解析失败 | `layer: "motif"` |
| H-22 | Runtime 路径错误 | `layer: "runtime"` |

---

## 七、快照语义测试

### 7.1 变量遮蔽

| ID | 场景 | YAML | 期望 |
|----|------|------|------|
| S-01 | 同名内部优先 | 内部 step 与外部同名 | 内部值 |
| S-02 | 外部不受影响 | 内部修改（不可写） | 外部值不变 |
| S-03 | 遮蔽与引用 | 内部引用内部，外部引用外部 | 各自独立 |

### 7.2 快照一致性

| ID | 场景 | 期望 |
|----|------|------|
| S-10 | 并行 foreach 快照 | 迭代开始前快照，过程中不变 |
| S-11 | 大状态快照 | Arc 引用，O(1) 克隆 |

---

## 八、并发安全测试（Phase 2）

### 8.1 并行 foreach

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| P-01 | parallel: true | 5 个迭代 | 并发执行 |
| P-02 | max_concurrency | `max: 2`，5 个迭代 | 最多 2 并发 |
| P-03 | 未声明并发安全 | Unit 无 concurrency 字段 | 串行化执行 |

### 8.2 资源限流

| ID | 场景 | 配置 | 期望 |
|----|------|------|------|
| P-10 | max_global | `max_global: 3` | 全局最多 3 并发 |
| P-11 | resource_key | `resource_key: "api"` | 同 key 共享配额 |
| P-12 | 超出限制 | 10 个迭代，max: 3 | 排队等待 |

---

## 九、集成与工作流测试

### 9.1 完整工作流：文本处理

```
输入："Hello World 123!"
  ↓
Complex: text-processing
  ↓
Structure: text-pipeline
  ↓
Motif: text-transform
  ├→ Unit: text-uppercase → "HELLO WORLD 123!"
  └→ Unit: text-reverse → "!321 DLROW OLLEH"
  ↓
输出：{upper: "...", reversed: "..."}
```

### 9.2 完整工作流：文档数字提取（模拟）

```
输入："订单号12345，总价¥1999.99元"
  ↓
Complex: document-processing
  ↓
Structure: number-extractor
  ↓
Motif: extract-numbers
  └→ Unit: regex-extract → ["12345", "1999.99"]
  ↓
输出：{numbers: [...], count: 2}
```

### 9.3 完整工作流：Git 代码审查（模拟）

```
输入：{repo_path: "/project", save_path: "review.md"}
  ↓
Complex: code-review
  ↓
Structure: git-audit
  ↓
Motif: review-loop (foreach)
  ├→ [文件1] git-diff → review → save-note
  ├→ [文件2] git-diff → review → save-note
  └→ [文件3] git-diff → review → save-note
  ↓
输出：{file_count: 3, reviews: [...]}
```

---

## 十、架构讨论反馈流程测试（元测试）

> 这是测试"如何测试"——捕获架构决策的迭代流程

### 10.1 讨论 → 文档流程

```
初始问题
  ↓
[讨论] "简单任务是否需要 4 层封装？"
  ↓
[文档] DISCUSSION_SUMMARY.md
  ↓
[反馈] "auto-complex 机制"
  ↓
[决策] ARCHITECTURE_DECISIONS.md
  ↓
[修正] v2 版本修正 7 类问题
  ↓
[合并] TECHNICAL_SPEC.md
```

### 10.2 发现问题流程

```
场景模拟：提取数字
  ↓
问题1：4 个文件太繁琐
  ↓
问题2：100+ Unit 上下文爆炸
  ↓
问题3：COGTOME 边界不清
  ↓
模拟讨论：应该怎么做？
  ↓
输出：PRACTICAL_ISSUES.md
  ↓
给"聪明人"请教
  ↓
获得反馈，更新决策
```

### 10.3 迭代验证流程

```
问题：示例代码笔误
  ↓
[发现] "steps.report 不存在"
  ↓
[报告] "这是笔误，需要修正"
  ↓
[确认] "同意，这是实现歧义"
  ↓
[修复] 更新文档
  ↓
[验证] 代码可实际运行
```

### 10.4 测试覆盖验证

| 阶段 | 验证内容 |
|------|----------|
| 讨论阶段 | 问题是否清晰定义？ |
| 文档阶段 | 决策是否有依据？ |
| 实现阶段 | 代码是否符合文档？ |
| 测试阶段 | 测试是否覆盖边界？ |
| 反馈阶段 | 发现的问题是否闭环？ |

---

## 十一、测试执行脚本

```bash
#!/bin/bash
set -e

COGTOME="./target/release/cogtome"
SKILLS="$COGTOME_SKILLS_DIR"

echo "=== COGTOME 综合测试 ==="
echo ""

echo "[1/11] 基础执行测试"
$COGTOME run text-processing --input '{"text":"hello"}' || echo "FAIL: basic run"
echo ""

echo "[2/11] Unit 执行测试"
$COGTOME unit run text-uppercase --input '{"text":"test"}' || echo "FAIL: unit"
echo ""

echo "[3/11] Discovery 测试"
$COGTOME discover || echo "FAIL: discover"
echo ""

echo "[4/11] 路径解析测试"
COGTOME_SKILLS_DIR=/tmp/nonexistent $COGTOME discover 2>&1 || echo "EXPECTED: path error"
echo ""

echo "[5/11] 表达式引擎测试"
# 需要构造包含表达式的 Motif
echo "TODO: 表达式引擎测试"
echo ""

echo "[6/11] Foreach 测试"
# 需要构造 foreach Motif
echo "TODO: foreach 测试"
echo ""

echo "[7/11] Aggregate 测试"
echo "TODO: aggregate 测试"
echo ""

echo "[8/11] 错误处理测试"
$COGTOME unit run nonexistent --input '{}' 2>&1 || echo "EXPECTED: not found"
echo ""

echo "[9/11] 快照语义测试"
echo "TODO: 快照测试"
echo ""

echo "[10/11] 并发安全测试"
echo "TODO: Phase 2 测试"
echo ""

echo "[11/11] 工作流测试"
echo "TODO: 完整工作流测试"
echo ""

echo "=== 测试完成 ==="
```

---

## 十二、测试矩阵

| 功能 | 正常 | 边界 | 错误 | 性能 |
|------|------|------|------|------|
| Unit 执行 | U-01 | U-06,07 | U-03,04,05 | - |
| Motif 执行 | M-01 | M-04,05 | M-02,03 | - |
| 表达式 | E-01~05 | E-10~15 | E-04,05 | - |
| foreach | F-01~05 | F-10~12 | F-22,23 | - |
| aggregate | A-01~04 | A-10~12 | A-22 | - |
| 错误处理 | H-01 | H-10~13 | H-20~22 | - |
| 快照 | S-01~03 | S-10,11 | - | - |
| 并发 | P-01~03 | P-10~12 | - | - |
| 工作流 | 9.1~9.3 | - | - | - |
