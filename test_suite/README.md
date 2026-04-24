# COGTOME 测试用例集

> 创建时间：2026-04-24
> 基于：TECHNICAL_SPEC.md
> 目的：验证 Phase 1 功能可用性
> 最新更新：2026-04-24 20:42

---

## 一、测试环境

```bash
# 工作目录
cd ~/cogtome

# Rust 环境
rustc 1.95.0
cargo 1.95.0

# 编译
cargo build --release

# 可执行文件
./target/release/cogtome
```

---

## 二、执行结果（2026-04-24）

### ✅ 已通过测试

| 用例 ID | 描述 | 结果 | 输出 |
|---------|------|------|------|
| T-UR-01 | 全局 Unit 查找 | ✅ PASS | `{"result":"HELLO"}` |
| T-UR-03 | 不存在的 Unit | ✅ PASS | `Error: Unit 'nonexistent' not found` |
| T-DS-01 | discover 命令 | ✅ PASS | `Found 1 Complex: text-processing` |
| T-CX-01 | run Complex | ✅ PASS | `{"reversed":"dlrow olleh","upper":"HELLO WORLD"}` |

### ⏳ 待测试

| 用例 ID | 描述 | 优先级 |
|---------|------|--------|
| T-EX-01~06 | 表达式引擎 | P0~P1 |
| T-FE-01~06 | foreach 循环 | P0~P1 |
| T-AG-01~05 | aggregate 聚合 | P0~P1 |
| T-ER-01~05 | 错误处理 | P0~P1 |
| T-SS-01~03 | 快照语义 | P0~P1 |
| T-PC-01~03 | 并行安全 | P2 |

---

## 三、测试命令

```bash
cd ~/cogtome

# Unit 测试
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
./target/release/cogtome unit run text-reverse --input '{"text":"hello"}'
./target/release/cogtome unit run nonexistent --input '{}'

# Motif 测试
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'

# Structure 测试
./target/release/cogtome structure run text-pipeline --input '{"text":"hello"}'

# Complex 测试
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# 发现测试
./target/release/cogtome discover
```

---

## 四、待测试用例详情

### 4.1 表达式引擎（P0-2）

| 用例 ID | 描述 | 测试命令 |
|---------|------|----------|
| T-EX-01 | 变量引用 | `${params.text}` |
| T-EX-02 | 索引访问 | `${steps.items.output[0]}` |
| T-EX-03 | 负索引 | `${steps.items.output[-1]}` |
| T-EX-04 | 长度属性 | `${steps.items.output.length}` |
| T-EX-05 | 缺失变量 | `${steps.nonexistent.output}` |
| T-EX-06 | 三目运算 | `${a > 5 ? 'big' : 'small'}` |

### 4.2 foreach 循环（P0-1）

| 用例 ID | 描述 | 优先级 |
|---------|------|--------|
| T-FE-01 | 空数组 `over: []` | P0 |
| T-FE-02 | 正常迭代 3 元素 | P0 |
| T-FE-03 | max_iterations 超限 | P0 |
| T-FE-04 | item 引用 | P0 |
| T-FE-05 | __index 引用 | P1 |
| T-FE-06 | 嵌套 foreach | P2 |

### 4.3 aggregate 聚合（P0-4）

| 用例 ID | 描述 | 模式 |
|---------|------|------|
| T-AG-01 | array 模式 | `mode: array` |
| T-AG-02 | object 模式 | `mode: object` |
| T-AG-03 | sum 模式 | `sum: "${item.value}"` |
| T-AG-04 | join 模式 | `join` + `separator` |
| T-AG-05 | 引用跳过 step | null + 警告日志 |

### 4.4 错误处理

| 用例 ID | 描述 | 期望 |
|---------|------|------|
| T-ER-01 | fail_fast | 不产出 aggregate，直接报错 |
| T-ER-02 | continue | 记录 `__error`，继续迭代 |
| T-ER-03 | if 解析失败 | 视为 false（跳过） |
| T-ER-04 | return 引用缺失 | **报错** |
| T-ER-05 | 层级错误码 | `layer: "unit/motif/runtime"` |

### 4.5 快照语义

| 用例 ID | 描述 | 期望 |
|---------|------|------|
| T-SS-01 | 变量遮蔽 | 内部优先，外部不受影响 |
| T-SS-02 | 外部只读 | foreach 内禁止修改外部 |
| T-SS-03 | 并行一致性 | snapshot 在迭代开始前捕获 |

---

## 五、测试覆盖检查

| TECHNICAL_SPEC 章节 | 对应测试 | 状态 |
|---------------------|----------|------|
| 3.1 foreach 循环 | T-FE-* | ⏳ 待测 |
| 3.2 表达式引擎 | T-EX-* | ⏳ 待测 |
| 4. 路径解析 | T-UR-* | ✅ 已通过 |
| 5. 快照语义 | T-SS-* | ⏳ 待测 |
| 6. 错误处理 | T-ER-* | ⏳ 待测 |
| 7. 并行安全 | T-PC-* | ⏳ Phase 2 |

---

## 六、测试执行日志

```
2026-04-24 20:42 GMT+8

$ ./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
{"result":"HELLO"}
[exit code: 0] ✅

$ ./target/release/cogtome unit run nonexistent --input '{}'
Error: Unit 'nonexistent' not found
[exit code: 1] ✅

$ ./target/release/cogtome discover
Found 1 Complex(es):
  text-processing      文本处理领域...
[exit code: 0] ✅

$ ./target/release/cogtome run text-processing --input '{"text":"hello world"}'
{
  "reversed": "dlrow olleh",
  "upper": "HELLO WORLD"
}
[exit code: 0] ✅
```
