import json, sys, re

raw = sys.stdin.read()
data = json.loads(raw)
contents = data.get('contents', [])
mode = data.get('mode', 'review')

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

print(json.dumps({
    'completed': dedup(completed), 'in_progress': dedup(in_progress),
    'blocked': dedup(blocked), 'decisions': dedup(decisions),
    'next_steps': dedup(next_steps), 'projects': projects[:20]
}))
