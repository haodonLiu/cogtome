import json, sys
from collections import Counter

raw = sys.stdin.read()
data = json.loads(raw)
extracted = data.get('extracted_tasks', {})
completed = extracted.get('completed', [])
in_progress = extracted.get('in_progress', [])
next_steps = extracted.get('next_steps', [])
projects = extracted.get('projects', [])

KW = {
    'git': ['git','commit','push','pull'],
    'bug_fix': ['bug','fix','修复'],
    'feature': ['feat','功能','新增'],
    'refactor': ['refactor','重构'],
    'api': ['api','接口'],
    'web_fetch': ['fetch','抓取'],
    'skill': ['skill','unit','motif'],
    'test': ['test','测试'],
    'doc': ['doc','文档']
}
counter = Counter()
for t in completed + in_progress + next_steps:
    text = t.get('text', '').lower()
    for kw, words in KW.items():
        if any(w in text for w in words):
            counter[kw] += 1

repeating = [{'type': k, 'count': c, 'description': f'出现 {c} 次'} for k, c in counter.most_common(10) if c >= 2]

abstractable = []
for k, c in counter.most_common(5):
    if c >= 3:
        tasks = [t['text'][:80] for t in (completed + in_progress + next_steps)
                 if any(w in t.get('text', '').lower() for w in KW[k])][:5]
        abstractable.append({'module_name': f'auto-{k}', 'reason': f'出现 {c} 次，可封装', 'tasks': tasks})

project_status = {}
for p in projects:
    project_status[p] = {'tasks_count': len(completed + in_progress), 'status': 'active'}

print(json.dumps({
    'repeating_patterns': repeating,
    'abstractable_modules': abstractable,
    'project_status': project_status
}))
