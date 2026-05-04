#!/usr/bin/env python3
"""Task tracker unit: detect newly completed tasks vs last heartbeat check."""
import json, sys, os, re
from pathlib import Path

HEARTBEAT_PATH = "/home/haodont/.openclaw/workspace/HEARTBEAT.md"
STATE_PATH = "/home/haodont/.openclaw/workspace/.task-tracker-state.json"

raw = sys.stdin.read()
data = json.loads(raw)

# Load current heartbeat state
def parse_heartbeat(path):
    """Extract current tasks with their status from HEARTBEAT.md"""
    if not os.path.exists(path):
        return {}, []
    content = open(path).read()
    
    in_progress = []
    completed = []
    current_section = None
    
    for line in content.split('\n'):
        # Section header
        m = re.match(r'^### (.+)$', line)
        if m:
            current_section = m.group(1).strip()
            continue
        
        # Check for task status
        if re.search(r'✅', line):
            m = re.match(r'^[-*]\s+(?:✅|\[x\])\s+(.+)$', line, re.I) or re.match(r'^#{1,3}\s+(.+?)\s+✅\s*$', line)
            if m:
                task_text = m.group(1).strip() if m.lastindex else re.sub(r'^[-*]\s+(?:✅|\[x\])\s+', '', line).strip()
                completed.append({'text': task_text, 'section': current_section})
        elif re.search(r'🔄', line) or re.search(r'- \[ \]', line):
            m = re.match(r'^[-*]\s+\[ \]\s+(.+)$', line, re.I)
            if m:
                in_progress.append({'text': m.group(1).strip(), 'section': current_section})
    
    return completed, in_progress

def load_state():
    if os.path.exists(STATE_PATH):
        with open(STATE_PATH) as f:
            return json.load(f)
    return {'last_completed': [], 'last_in_progress': []}

def save_state(state):
    with open(STATE_PATH, 'w') as f:
        json.dump(state, f)

# Parse current heartbeat
current_completed, current_in_progress = parse_heartbeat(HEARTBEAT_PATH)
prev_state = load_state()

# Detect newly completed tasks
prev_completed_texts = {t['text'] for t in prev_state.get('last_completed', [])}
newly_completed = [t for t in current_completed if t['text'] not in prev_completed_texts]

# Update state
new_state = {
    'last_completed': current_completed,
    'last_in_progress': current_in_progress
}
save_state(new_state)

# Build suggestions for newly completed tasks
suggestions = []
for task in newly_completed:
    text = task['text'].lower()
    section = task.get('section', '')
    
    # Analyze task type and suggest what could be abstracted
    if any(k in text for k in ['安装', 'skill', '安装完成']):
        suggestions.append({
            'task': task['text'],
            'section': section,
            'suggestion': '可封装成 Skill 安装的自动化流程',
            'cogtome_type': 'structure'
        })
    elif any(k in text for k in ['修复', 'bug', 'fix']):
        suggestions.append({
            'task': task['text'],
            'section': section,
            'suggestion': 'Bug 修复流程可记录到知识库',
            'cogtome_type': 'structure'
        })
    elif any(k in text for k in ['实现', 'feat', '新增', '添加']):
        suggestions.append({
            'task': task['text'],
            'section': section,
            'suggestion': '新功能实现可形成可复用的工作流模板',
            'cogtome_type': 'structure'
        })
    elif any(k in text for k in ['测试', 'test']):
        suggestions.append({
            'task': task['text'],
            'section': section,
            'suggestion': '测试流程可封装成标准 Unit',
            'cogtome_type': 'unit'
        })

print(json.dumps({
    'newly_completed': newly_completed,
    'suggestions': suggestions,
    'current_in_progress': current_in_progress,
    'has_new_completions': len(newly_completed) > 0
}, ensure_ascii=False))
