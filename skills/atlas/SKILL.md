---
name: atlas
description: |
  方案审查与辩论 Skill。通过 Atlas（AI critic subagent）进行结构化审查，
  多轮辩论后收敛到双方共识的建议。COGTOME Motif 定义审查流程的 DAG，
  Atlas subagent 作为执行体，实际调用通过 sessions_send 发起。
structures:
  - name: atlas-review
    path: ./motifs/atlas-review.json
    summary: "方案审查与辩论 — 3轮收敛到共识的 DAG 工作流"
    scenarios: ["审查技术方案", "评估决策风险", "设计review", "第二意见"]
---

# Atlas Review — 方案审查与辩论

## 架构概述

```
SKILL.md (定义)
    ↓
motifs/atlas-review.json (DAG 流程)
    ↓
Milos 通过 sessions_send 调用 Atlas subagent (执行体)
    ↓
Atlas 返回结构化审查结果
    ↓
收敛到共识结论
```

Atlas 是一个 **persistent subagent**，由 Milos 在需要时通过 `sessions_send` 驱动。
Motif 文件定义了审查流程的协议——不是 COGTOME 直接执行，而是 Milos 作为中间层。

## 工作流（3轮收敛）

```
submit → initial_review → respond → synthesize → [若未共识则循环] → verdict
```

- **轮次**：最多 3 轮，3 轮后强制收敛
- **输入**：`plan`（方案）+ `context`（可选背景）
- **输出**：`{ verdict, recommendation, remaining_concerns }`

## Motif：DAG 结构

```
[start]
   │
   ▼
[atlas-initial-review]  ◄── Atlas 初始审查
   │
   ▼
[gate: verdict?]  ── HOLD ──► [output]  （直接通过）
   │
 FLAGGED
   │
   ▼
[planner-respond]  ◄── Planner 逐一回应每个 FLAGGED 点
   │
   ▼
[atlas-synthesize]  ◄── Atlas 综合回应，给出新 verdict
   │
   ▼
[gate: round >= 3 or agreed?] ──► [output]
   │
 (继续循环，最多3轮)
```

## 调用方式

```bash
# Milos 通过 sessions_send 驱动 Atlas
sessions_send("atlas", {
  plan: "...",
  context: "...",
  depth: "full"
})
```

## 输入格式

```json
{
  "plan": "string — 待审查的方案",
  "context": "string — 背景信息（可选）",
  "depth": "light | full  — 审查深度，默认 full"
}
```

## Atlas 角色定义

见 `units/atlas-role.md`

## Units

| Unit | 状态 | 说明 |
|------|------|------|
| `atlas-initial-review` | 协议定义 | Atlas 初始审查（通过 subagent 执行） |
| `atlas-synthesize` | 协议定义 | Atlas 综合回应（通过 subagent 执行） |

实际执行体：persistent Atlas subagent（通过 `sessions_send` 驱动）

## 适用场景

- 技术方案设计 review
- 新项目立项评估
- 重大依赖升级决策
- 任何"我想这样做但想听听第二意见"的时刻
