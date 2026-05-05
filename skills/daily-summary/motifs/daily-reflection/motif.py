#!/usr/bin/env python3
"""Daily Reflection Motif — merged: memory-indexer + task-extractor + pattern-finder + report-writer"""
import json, sys, os, hashlib
from datetime import datetime, timedelta

CACHE_DIR = os.path.expanduser("~/.cogtome/cache/daily-reflection")
os.makedirs(CACHE_DIR, exist_ok=True)

def cache_key(params):
    m = hashlib.md5()
    for k in sorted(params):
        m.update(f"{k}={params[k]}".encode())
    return m.hexdigest()

def read_cache(key):
    path = os.path.join(CACHE_DIR, f"{key}.json")
    if os.path.exists(path):
        with open(path) as f:
            return json.load(f)
    return None

def write_cache(key, data):
    path = os.path.join(CACHE_DIR, f"{key}.json")
    with open(path, 'w') as f:
        json.dump(data, f)

# ── Unit 1: memory-indexer ──────────────────────────────────────────────
def memory_indexer(params):
    memory_base = params.get('memory_base_path', '/home/haodont/.openclaw/workspace/memory')
    date = params.get('date')
    days = params.get('days')
    scan_mode = params.get('scan_mode', 'full')

    files = []
    range_start = range_end = ''

    if date:
        date_file = os.path.join(memory_base, f'{date}.md')
        if os.path.exists(date_file):
            files.append(date_file)
            range_start = range_end = date
        else:
            return None, {'error': f'not found: {date_file}'}
    elif days:
        end_str = datetime.now().strftime('%Y-%m-%d')
        start = datetime.strptime(end_str, '%Y-%m-%d') - timedelta(days=days-1)
        start_str = start.strftime('%Y-%m-%d')
        range_start, range_end = start_str, end_str
        current = start
        end = datetime.strptime(end_str, '%Y-%m-%d')
        while current <= end:
            f = os.path.join(memory_base, f"{current.strftime('%Y-%m-%d')}.md")
            if os.path.exists(f):
                files.append(f)
            current += timedelta(days=1)

    contents = []
    for fpath in files:
        with open(fpath) as f:
            contents.append({'path': fpath, 'date': os.path.basename(fpath).replace('.md',''), 'content': f.read()})

    return {'files_read': files, 'contents': contents, 'date_range': {'start': range_start, 'end': range_end}}, None

# ── Unit 2: task-extractor ─────────────────────────────────────────────
def task_extractor(contents, mode):
    import re
    completed, in_progress, blocked, decisions, next_steps, projects = [], [], [], [], [], []

    for item in contents:
        text = item.get('content', '')
        date = item.get('date', '')
        for line in text.split('\n'):
            line = line.strip()
            m_proj = re.match(r'^## (.+)$', line)
            if m_proj:
                proj = m_proj.group(1).strip()
                if proj not in projects and not proj.startswith(('待', '20')):
                    projects.append(proj)
            if '✅' in line:
                m = re.match(r'^#{1,3}\s+(.+?)\s+✅\s*$', line)
                if m: completed.append({'text': m.group(1).strip(), 'source_date': date})
                else:
                    m = re.match(r'^[-*]\s+(?:✅|\[x\])\s+(.+)$', line, re.I)
                    if m: completed.append({'text': m.group(1).strip(), 'source_date': date})
            if '🔄' in line:
                m = re.match(r'^#{1,3}\s+(.+?)\s+🔄\s*$', line)
                if m: in_progress.append({'text': m.group(1).strip(), 'source_date': date})
                else:
                    m = re.match(r'^[-*]\s+(?:🔄|\[ \])\s+(.+)$', line, re.I)
                    if m: in_progress.append({'text': m.group(1).strip(), 'source_date': date})
            if re.match(r'^⛔', line):
                blocked.append({'text': re.sub(r'^⛔\s*', '', line).strip(), 'source_date': date})
            m = re.match(r'^[-*]\s+\[ \]\s+(.+)$', line, re.I)
            if m:
                step = m.group(1).strip()
                if step and len(step) > 2: next_steps.append({'text': step, 'source_date': date})
            if '**决定**' in line or '**决策**' in line:
                decisions.append({'text': line, 'source_date': date})

    def dedup(items):
        seen, result = set(), []
        for i in items:
            key = i['text'][:60]
            if key not in seen:
                seen.add(key); result.append(i)
        return result

    return {
        'completed': dedup(completed), 'in_progress': dedup(in_progress),
        'blocked': dedup(blocked), 'decisions': dedup(decisions),
        'next_steps': dedup(next_steps), 'projects': projects[:20]
    }

# ── Unit 3: pattern-finder ──────────────────────────────────────────────
def pattern_finder(extracted):
    from collections import Counter
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

    return {
        'repeating_patterns': repeating,
        'abstractable_modules': abstractable,
        'project_status': project_status
    }

# ── Unit 4: report-writer ───────────────────────────────────────────────
def report_writer(extracted, pattern, mode, date):
    completed = extracted.get('completed', [])
    in_progress = extracted.get('in_progress', [])
    blocked = extracted.get('blocked', [])
    decisions = extracted.get('decisions', [])
    next_steps = extracted.get('next_steps', [])
    patterns = pattern.get('repeating_patterns', [])
    abstractable = pattern.get('abstractable_modules', [])

    lines = [f'## 🌙 凌晨反思 — {date or "最近"}', '']
    if patterns:
        lines += ['### 🔁 重复模式'] + [f"- **{p['type']}**（{p['count']}次）: {p.get('description','')}" for p in patterns] + ['']
    if abstractable:
        lines += ['### 🧩 建议抽象'] + [f"- `{a['module_name']}`: {a['reason']}" for a in abstractable] + ['']
    if next_steps:
        lines += ['### 🚀 可优化'] + [f"- [ ] {n['text']}" for n in next_steps[:5]] + ['']
    report = '\n'.join(lines)

    return {
        'report': report,
        'summary': {'completed_count':len(completed),'in_progress_count':len(in_progress),'blocked_count':len(blocked),'next_steps_count':len(next_steps)},
        'mode': mode
    }

# ── Main ─────────────────────────────────────────────────────────────────
raw = sys.stdin.read()
params = json.loads(raw)

# Cache check
key = cache_key(params)
cached = read_cache(key)
if cached:
    cached['from_cache'] = True
    print(json.dumps(cached))
    sys.exit(0)

# Step 1: memory-indexer
idx_result, idx_err = memory_indexer(params)
if idx_err:
    print(json.dumps(idx_err))
    sys.exit(1)

# Step 2: task-extractor
extracted = task_extractor(idx_result['contents'], params.get('mode', 'reflection'))

# Step 3: pattern-finder
patterns = pattern_finder(extracted)

# Step 4: report-writer
date_range = idx_result.get('date_range', {}).get('end', params.get('date', ''))
report = report_writer(extracted, patterns, params.get('mode', 'reflection'), date_range)

result = {
    'report': report['report'],
    'summary': report['summary'],
    'date_range': idx_result.get('date_range'),
    'from_cache': False
}

# Write cache
write_cache(key, result)

print(json.dumps(result))
