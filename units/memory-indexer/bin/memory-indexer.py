import json, sys, os
from datetime import datetime, timedelta

raw = sys.stdin.read()
data = json.loads(raw)

memory_base = data.get('memory_base_path', '/home/haodont/.openclaw/workspace/memory')
date = data.get('date')
days = data.get('days')
start_date = data.get('start_date')
end_date = data.get('end_date')
include_special = data.get('include_special', [])
scan_mode = data.get('scan_mode', 'full')

files = []
range_start = range_end = ''

if date:
    date_file = os.path.join(memory_base, f'{date}.md')
    if os.path.exists(date_file):
        files.append(date_file)
        range_start = range_end = date
    else:
        print(json.dumps({'error': f'not found: {date_file}', 'files_read': [], 'contents': [], 'date_range': None}))
        sys.exit(1)
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
elif start_date and end_date:
    range_start, range_end = start_date, end_date
    current = datetime.strptime(start_date, '%Y-%m-%d')
    end = datetime.strptime(end_date, '%Y-%m-%d')
    while current <= end:
        f = os.path.join(memory_base, f"{current.strftime('%Y-%m-%d')}.md")
        if os.path.exists(f):
            files.append(f)
        current += timedelta(days=1)
else:
    print(json.dumps({'error': 'need date, days, or start_date+end_date', 'files_read': [], 'contents': [], 'date_range': None}))
    sys.exit(1)

contents = []

# Fast path: single file, skip heavier processing
if scan_mode == 'single_day' and len(files) == 1:
    with open(files[0]) as f:
        contents.append({'path': files[0], 'date': os.path.basename(files[0]).replace('.md',''), 'content': f.read()})
else:
    for fpath in files:
        with open(fpath) as f:
            contents.append({'path': fpath, 'date': os.path.basename(fpath).replace('.md',''), 'content': f.read()})

for sf in include_special:
    if os.path.isfile(sf):
        with open(sf) as f:
            contents.append({'path': sf, 'date': os.path.basename(sf), 'content': f.read()})

print(json.dumps({'files_read': files, 'contents': contents, 'date_range': {'start': range_start, 'end': range_end}}))
