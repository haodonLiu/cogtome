import json, sys

raw = sys.stdin.read()
data = json.loads(raw)
mode = data.get('mode', 'review')
date = data.get('date', '')
extracted = data.get('extracted_tasks', {})
pattern = data.get('pattern_analysis', {})
completed = extracted.get('completed', [])
in_progress = extracted.get('in_progress', [])
blocked = extracted.get('blocked', [])
decisions = extracted.get('decisions', [])
next_steps = extracted.get('next_steps', [])
patterns = pattern.get('repeating_patterns', [])
abstractable = pattern.get('abstractable_modules', [])

if mode == 'review':
    lines = [f'## 📋 工作日报 — {date}', '']
    if completed:
        lines += ['### ✅ 完成'] + [f"- {t['text']}" for t in completed[:10]] + ['']
    if in_progress:
        lines += ['### 🔄 进行中'] + [f"- {t['text']}" for t in in_progress[:10]] + ['']
    if blocked:
        lines += ['### ⛔ 阻碍'] + [f"- {t['text']}" for t in blocked[:5]] + ['']
    if decisions:
        lines += ['### 💡 决策'] + [f"- {d['text']}" for d in decisions[:5]] + ['']
    if next_steps:
        lines += ['### 📝 下一步'] + [f"- [ ] {n['text']}" for n in next_steps[:10]] + ['']
    report = '\n'.join(lines)
elif mode == 'reflection':
    lines = [f'## 🌙 凌晨反思 — {date or "最近"}', '']
    if patterns:
        lines += ['### 🔁 重复模式'] + [f"- **{p['type']}**（{p['count']}次）: {p.get('description','')}" for p in patterns] + ['']
    if abstractable:
        lines += ['### 🧩 建议抽象'] + [f"- `{a['module_name']}`: {a['reason']}" for a in abstractable] + ['']
    if next_steps:
        lines += ['### 🚀 可优化'] + [f"- [ ] {n['text']}" for n in next_steps[:5]] + ['']
    report = '\n'.join(lines)
else:
    report = ''

print(json.dumps({'report': report, 'summary': {'completed_count':len(completed),'in_progress_count':len(in_progress),'blocked_count':len(blocked),'next_steps_count':len(next_steps)}, 'mode': mode}))
