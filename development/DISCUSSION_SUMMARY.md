# COGTOME 讨论总结

> 整理时间：2026-04-24
> 整理人：Milos

---

## 一、COGTOME 项目现状

### 1.1 项目定位

COGTOME 是面向 AI Agent 的微型操作系统与执行运行时，核心思想是：

- **四层模型**：Complex → Structure → Motif → Unit
- **执行纪律**：每层只调用下一层，禁止跨层直接调用
- **发现机制**：Complex 持有 description，被 Runtime 自动发现

### 1.2 当前进展

- ✅ 四层模型设计完成
- ✅ Rust Runtime MVP 实现
- ✅ CLI 框架（unit/motif/structure/run/discover）
- ✅ 文本处理示例 Complex
- ✅ 中英双语 README + cover.jpg
- ✅ 开发文档目录建立
- ✅ 实践问题文档编写
- ⚠️ 尚未实际运行验证（Windows 环境无 Rust）

### 1.3 项目结构

```
cogtome/
├── src/                    # Rust Runtime
├── skills/                 # 示例 Skill
│   ├── units/              # 原子执行体
│   ├── motifs/             # 编排逻辑
│   ├── structures/         # 业务封装
│   └── text-processing/    # 示例 Complex
└── development/            # 开发文档
    ├── COGTOME_Architecture.md
    ├── MILOS_COGTOME_INTEGRATION_PLAN.md
    ├── COGTOME_IMAGE_PROMPTS.md
    └── PRACTICAL_ISSUES.md
```

---

## 二、核心疑问（待解决）

### 疑问 1：简单任务与复杂任务的分层入口

**问题描述：**

当前设计中，Complex 是唯一对 Agent 暴露的业务封装层。但对于简单任务（只需要一个 Unit），写一个 Complex 显得过于繁琐：

| 层级 | 最小工作量 |
|------|-----------|
| Complex | 写 SKILL.md（description + structures） |
| Structure | 写 manifest.yaml（input/output schema） |
| Motif | 写 YAML（flow + return） |
| Unit | 写 Python 脚本 + chmod +x |

**简单任务示例**："从文本提取所有数字"

```
# 需要 4 个文件
Complex SKILL.md + Structure manifest.yaml + Motif yaml + Unit script
```

如果 Agent 只需要跑一个简单的 Python 脚本，这个开销是否合理？

**待明确：**
- 简单任务是否应该跳过 Complex/Structure/Motif 层？
- COGTOME 是否应该提供多级入口（inline script / unit / motif / complex）？

---

### 疑问 2：Unit 接口的暴露问题

**问题描述：**

如果 Complex 暴露给 Agent，那么 Agent 看到的是 10-20 个 Complex。

但如果 Agent 需要直接调用 Unit（简单任务），那可能面临 100+ 个 Unit 接口，这会占据大量上下文。

**两种方案对比：**

| 方案 | Agent 看到什么 | 问题 |
|------|---------------|------|
| 直接暴露 Unit | 100+ 个 Unit | 上下文爆炸 |
| 只暴露 Complex | 10-20 个 Complex | 简单任务过重 |

**待明确：**
- Agent 应该看到哪一层？
- 简单任务如何处理？

---

### 疑问 3：COGTOME 的智能边界

**问题描述：**

我在描述 OpenClaw + COGTOME 集成时，无意中把 COGTOME 描述成了"带意图匹配的运行时"：

```
OpenClaw 扫描 SKILL.md → 匹配 Complex → COGTOME 执行
        ↑
   OpenClaw 在做匹配
```

这与 COGTOME 的原始设计"纯执行后端"产生了偏差。

**原始设计思路：**
- COGTOME 是**执行引擎**，不是 Agent
- Agent 自己在 SKILL.md 里读 description，自己决定用什么
- COGTOME 只负责忠实执行指定的 Complex

**但实践中：**
- 如果 Agent 需要"发现可用技能"，需要有人扫描 SKILL.md
- Discovery 机制提供了扫描能力，但"匹配"应该由谁做？

**待明确：**

```
COGTOME 的职责边界：
- Option A：纯执行后端（Agent 决定 → COGTOME 执行）
- Option B：带一定智能的运行时（能根据描述自动路由）
```

如果是 A，Discovery 只是给 Agent 提供**可用技能列表**，不是自动匹配。
如果是 B，COGTOME 在扮演 Agent 的角色，需要重新审视定位。

---

## 三、讨论要点

### 3.1 关于分层入口

- 是否需要为简单任务提供轻量入口？
- Complex 是否应该是"可选层"而非"必填层"？
- Motif 和 Structure 的区别是否清晰？它们在什么场景下是必需的？

### 3.2 关于 Agent 与 COGTOME 的边界

- COGTOME 是工具还是 Agent？
- 如果是工具，它的"发现"能力是否必要？
- 如果是 Agent，它的"执行"能力是否足够独立？

### 3.3 关于上下文占用

- Unit 级别的接口数量是否会成为瓶颈？
- 如何在不暴露所有细节的前提下让 Agent 知道能力边界？
- OpenClaw 的 Skill 发现机制与 COGTOME 的 Discovery 是否需要统一？

---

## 四、结论（已明确）

### 架构决策：COGTOME = 纯执行后端（方案 A）

> "Agent 选 Complex，COGTOME 跑 Unit；Agent 做决策，COGTOME 做纪律。"

**确立的原则：**
- OpenClaw 负责"聪明"（理解、选择、规划）
- COGTOME 负责"可靠"（执行、隔离、编排、日志）
- Discovery = 能力目录（`ls` + `cat`），不是自动路由（`grep --smart`）
- 如果 COGTOME 也做匹配 → 双重 Agent 问题，调试地狱

```
OpenClaw (决策层)：理解意图 → 选择 Complex → 构造参数
     ↓
COGTOME (执行层)：接收指令 → 解析 Structure → 编排 Motif → 调度 Unit
     ↓
操作系统：进程、文件、网络
```

### 简单任务问题：auto-complex 机制

**方案：** 提供 `--auto-complex` 注册选项

```bash
cogtome register unit extract_numbers.py \
  --name "extract_numbers" \
  --description "从文本中提取所有数字" \
  --auto-complex
```

- 开发者写 **1 个文件**（Unit 脚本）
- Runtime 自动生成其余 3 层
- Agent 看到 **1 个 Complex**
- 架构统一性保持

### Unit 暴露问题：Agent 永远只看 Complex

- Agent 上下文只包含 10-20 个 Complex
- 不暴露 Unit（100+ 个）
- 通用 Unit 如需暴露 → 提升为独立 Complex

---

## 五、想请教的问题

1. ✅ **已解决**：COGTOME 是纯执行后端，不做匹配
2. ✅ **已解决**：auto-complex 机制解决简单任务过重
3. ✅ **已解决**：Agent 只看 Complex 层
4. **待跟进**：auto-complex 的实现优先级？是否纳入 Phase 1？

---

## 六、参考资料

- `COGTOME_Architecture.md` — 原始架构文档
- `MILOS_COGTOME_INTEGRATION_PLAN.md` — 与 OpenClaw 的集成思考
- `PRACTICAL_ISSUES.md` — 实践中可能遇到的问题

---

如果你觉得这个总结有遗漏或偏差，欢迎补充。
