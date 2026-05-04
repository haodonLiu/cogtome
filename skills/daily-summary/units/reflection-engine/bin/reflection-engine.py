#!/usr/bin/env python3
"""
reflection-engine: LLM-powered reflection analysis.

Input (stdin JSON):
  {
    "tasks": [...],           # from task-extractor
    "patterns": {...},        # raw pattern-finder output (for context)
    "traces": {...},          # from trace-analyzer
    "mode": "reflection",
    "date_range": {"start": "YYYY-MM-DD", "end": "YYYY-MM-DD"}
  }

Output (stdout JSON):
  {
    "summary": {...},
    "patterns": [...],
    "abstractions": [...],
    "risks": [...],
    "reflection": "...",
    "report": "..."   # rendered markdown
  }
"""

import json
import sys
import os
import requests
import re
from datetime import datetime

# ---------------------------------------------------------------------------
# Config
# ---------------------------------------------------------------------------
API_KEY = os.environ.get("MINIMAX_API_KEY", "")
if not API_KEY:
    raise RuntimeError("MINIMAX_API_KEY environment variable not set")
BASE_URL = "https://api.minimaxi.com/v1"
MODEL = "MiniMax-M2.7"

TIMEOUT_SECONDS = 60

# ---------------------------------------------------------------------------
# LLM call
# ---------------------------------------------------------------------------

def llm_complete(prompt: str, system: str = "") -> str:
    messages = []
    if system:
        messages.append({"role": "system", "content": system})
    messages.append({"role": "user", "content": prompt})

    resp = requests.post(
        f"{BASE_URL}/chat/completions",
        headers={"Authorization": f"Bearer {API_KEY}", "Content-Type": "application/json"},
        json={"model": MODEL, "messages": messages, "temperature": 0.3},
        timeout=TIMEOUT_SECONDS,
    )
    resp.raise_for_status()
    return resp.json()["choices"][0]["message"]["content"]


# ---------------------------------------------------------------------------
# Prompt builder
# ---------------------------------------------------------------------------

SYSTEM_PROMPT = """You are Midnight Reflection's analysis engine. Your role is to:
1. Identify genuine behavioral patterns from task and trace data
2. Diagnose root causes using 5-Whys style reasoning
3. Assess which repeated work is worth abstracting into automation/templates
4. Predict下周 risks based on current blocked tasks and patterns

Rules:
- Do NOT describe "what happened" — analyze "what it means" and "why it matters"
- Every judgment must cite evidence (task ID or trace location)
- Distinguish facts from inferences; use "may", "suggests" for inferences
- If data is insufficient, say "Insufficient data to determine" — do not fabricate
- Work in Chinese when analyzing Chinese user data
- Output valid JSON matching the schema exactly
- Do not add extra fields or commentary outside the JSON
- The "report" field should be Chinese Markdown
- completion_rate should be "X%" string format
- If a completed task has no trace evidence, mark it as suspicious in fake_completed
- abstractable work should have: name, trigger, current_cost, proposed_solution, roi
- Each pattern needs: type, category, evidence (task IDs), root_cause, actionable
- Each risk needs: what, why, mitigation
"""


USER_PROMPT_TEMPLATE = """# Input Data

## Tasks (from task-extractor)
```json
{TASKS_JSON}
```

## Raw Patterns (from pattern-finder, for context)
```json
{PATTERNS_JSON}
```

## Trace Analysis (from trace-analyzer)
```json
{TRACES_JSON}
```

## Date Range
{START_DATE} → {END_DATE}

---

# Your Analysis

Analyze the data above following these steps:

## Step 1: Task Authenticity Check
- For each completed task, check if there's trace evidence of actual completion
- If ✅ but no trace → fake_completed += 1
- Calculate completion_rate = completed / (completed + in_progress + blocked + uncertain)

## Step 2: Semantic Clustering
- Group tasks by work type: coding, research, infra, communication, writing, debugging
- Within each cluster, look for repeated structures (not just word frequency)
- A pattern needs 2+ occurrences with similar context to be real

## Step 3: Pattern Diagnosis (5 Whys)
For each genuine repeated pattern, ask why it keeps happening.
Stop when you reach a changeable root cause (habit, environment, toolchain).

## Step 4: Abstraction Assessment
For any work repeated 3+ times:
- automation cost vs. benefit
- template/automation feasibility
- ROI: 高/中/低

## Step 5: Risk Prediction
Based on blocked tasks and recent patterns, predict top 3 blockers for next week.
Each risk needs: what, why (root cause), mitigation.

---

# Output Schema

Return ONLY valid JSON (no markdown code fences, no explanation outside the JSON):

{{
  "summary": {{
    "completed": N,
    "in_progress": N,
    "blocked": N,
    "uncertain": N,
    "fake_completed": N,
    "completion_rate": "X%"
  }},
  "patterns": [
    {{
      "type": "模式类型简短描述",
      "category": "时间分配|工作类型|阻塞原因|工具使用",
      "evidence": ["task_id_1", "task_id_2"],
      "root_cause": "5 Whys 推到可改变的根本原因",
      "actionable": "具体的、可执行的改进建议"
    }}
  ],
  "abstractions": [
    {{
      "name": "automation-name",
      "trigger": "什么情况触发",
      "current_cost": "目前平均耗时/频率",
      "proposed_solution": "具体解决方案",
      "roi": "高|中|低"
    }}
  ],
  "risks": [
    {{
      "what": "预测的下周卡点",
      "why": "为什么预测这个",
      "mitigation": "提前准备的对策"
    }}
  ],
  "reflection": "一段自然语言总结，评估本周整体效率和主要发现",
  "report": "## 🌙 凌晨反思 — {END_DATE}\\n\\n（渲染好的 Markdown 报告，包含 summary、patterns、abstractions、risks，用中文，含 emoji）"
}}

Constraints:
- Return ONLY the JSON object, nothing else
- report must be valid Markdown with Chinese content
- patterns 和 abstractions 至少为空数组，不要省略
- 所有中文字符必须保留，不要转义
"""


def build_prompt(tasks: list, patterns: dict, traces: dict,
                 start_date: str, end_date: str) -> str:
    return USER_PROMPT_TEMPLATE.format(
        TASKS_JSON=json.dumps(tasks, ensure_ascii=False, indent=2),
        PATTERNS_JSON=json.dumps(patterns, ensure_ascii=False, indent=2),
        TRACES_JSON=json.dumps(traces, ensure_ascii=False, indent=2),
        START_DATE=start_date,
        END_DATE=end_date,
    )


# ---------------------------------------------------------------------------
# Output schema for rendering markdown from LLM JSON
# ---------------------------------------------------------------------------

def render_report(data: dict, end_date: str) -> str:
    """Render a markdown report from the structured JSON."""
    summary = data.get("summary", {})
    patterns = data.get("patterns", [])
    abstractions = data.get("abstractions", [])
    risks = data.get("risks", [])
    reflection = data.get("reflection", "")

    lines = [f"## 🌙 凌晨反思 — {end_date}", ""]

    # Summary
    total = (summary.get("completed", 0) + summary.get("in_progress", 0) +
             summary.get("blocked", 0) + summary.get("uncertain", 0))
    lines += [
        "### 📊 执行摘要",
        f"- ✅ 完成: {summary.get('completed', 0)}",
        f"- 🔄 进行中: {summary.get('in_progress', 0)}",
        f"- ⛔ 阻塞: {summary.get('blocked', 0)}",
        f"- ❓ 不确定: {summary.get('uncertain', 0)}",
        f"- �假完成: {summary.get('fake_completed', 0)}",
        f"- 完整率: {summary.get('completion_rate', 'N/A')}",
        "",
    ]

    # Reflection
    if reflection:
        lines += ["### 💭 反思", f"{reflection}", ""]

    # Patterns
    if patterns:
        lines += ["### 🔁 行为模式"]
        for p in patterns:
            lines += [
                f"- **{p.get('type', 'unknown')}**（{p.get('category', '')}）",
                f"  - 证据: {', '.join(p.get('evidence', []))}",
                f"  - 根因: {p.get('root_cause', 'unknown')}",
                f"  - 建议: {p.get('actionable', 'none')}",
            ]
        lines += [""]
    else:
        lines += ["### 🔁 行为模式\n*未发现明显的重复模式*\n"]

    # Abstractions
    if abstractions:
        lines += ["### 🧩 抽象建议"]
        for a in abstractions:
            lines += [
                f"- **{a.get('name', 'unknown')}**（ROI: {a.get('roi', '?')}）",
                f"  - 触发: {a.get('trigger', 'unknown')}",
                f"  - 当前成本: {a.get('current_cost', 'unknown')}",
                f"  - 方案: {a.get('proposed_solution', 'unknown')}",
            ]
        lines += [""]
    else:
        lines += ["### 🧩 抽象建议\n*无明显值得抽象的工作*\n"]

    # Risks
    if risks:
        lines += ["### ⚠️ 下周风险预警"]
        for r in risks:
            lines += [
                f"- **{r.get('what', 'unknown')}**",
                f"  - 原因: {r.get('why', 'unknown')}",
                f"  - 对策: {r.get('mitigation', 'unknown')}",
            ]
        lines += [""]
    else:
        lines += ["### ⚠️ 下周风险预警\n*无明显风险*\n"]

    return "\n".join(lines)


# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

def main():
    raw = sys.stdin.read()
    if not raw.strip():
        # Empty input — return empty structure
        result = {"summary": {}, "patterns": [], "abstractions": [], "risks": [], "reflection": "", "report": ""}
        print(json.dumps(result, ensure_ascii=False))
        return

    data = json.loads(raw)

    tasks = data.get("tasks", [])
    patterns = data.get("patterns", {})
    traces = data.get("traces", {})
    mode = data.get("mode", "reflection")
    date_range = data.get("date_range", {})
    start_date = date_range.get("start", "")
    end_date = date_range.get("end", datetime.now().strftime("%Y-%m-%d"))

    if mode != "reflection":
        # Non-reflection mode: just pass through
        result = {"summary": {}, "patterns": [], "abstractions": [], "risks": [], "reflection": "", "report": ""}
        print(json.dumps(result, ensure_ascii=False))
        return

    if not tasks:
        result = {
            "summary": {"completed": 0, "in_progress": 0, "blocked": 0, "uncertain": 0, "fake_completed": 0, "completion_rate": "0%"},
            "patterns": [],
            "abstractions": [],
            "risks": [],
            "reflection": "本周无任务记录，数据不足无法进行有效反思。",
            "report": f"## 🌙 凌晨反思 — {end_date}\n\n*本周无任务记录，数据不足*\n"
        }
        print(json.dumps(result, ensure_ascii=False))
        return

    prompt = build_prompt(tasks, patterns, traces, start_date, end_date)

    try:
        raw_response = llm_complete(prompt, SYSTEM_PROMPT)
    except Exception as e:
        result = {
            "summary": {"error": str(e)},
            "patterns": [],
            "abstractions": [],
            "risks": [],
            "reflection": f"LLM 调用失败: {e}",
            "report": f"## 🌙 凌晨反思 — {end_date}\n\n*LLM 调用失败: {e}*\n"
        }
        print(json.dumps(result, ensure_ascii=False))
        return

    # Parse JSON from response (strip markdown fences and thinking tags)
    raw_response = raw_response.strip()

    # Strip triple-backtick code fences
    if raw_response.startswith("```"):
        lines = raw_response.split("\n")
        if lines[0].startswith("```"):
            lines = lines[1:]
        if lines and lines[-1].startswith("```"):
            lines = lines[:-1]
        raw_response = "\n".join(lines).strip()

    # Strip think tags (模型输出的思考内容)
    import re
    raw_response = re.sub(r'<think>[\s\S]*?</think>', '', raw_response)

    try:
        parsed = json.loads(raw_response)
    except json.JSONDecodeError as e:
        result = {
            "summary": {"error": f"JSON parse failed: {e}", "raw": raw_response[:500]},
            "patterns": [],
            "abstractions": [],
            "risks": [],
            "reflection": f"JSON 解析失败: {e}",
            "report": f"## 🌙 凌晨反思 — {end_date}\n\n*JSON 解析失败，请检查 LLM 输出*\n{raw_response[:500]}\n"
        }
        print(json.dumps(result, ensure_ascii=False))
        return

    # Ensure report field is populated
    if not parsed.get("report"):
        parsed["report"] = render_report(parsed, end_date)

    print(json.dumps(parsed, ensure_ascii=False))


if __name__ == "__main__":
    main()
