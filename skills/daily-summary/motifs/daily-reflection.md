# Daily Reflection Motif

合并版 reflection 路径：memory-indexer + task-extractor + pattern-finder + report-writer

## 优化目标
- 进程启动：从 4 次 → 1 次
- 加缓存：相同参数不重跑

## 输入
```json
{
  "memory_base_path": "/path/to/memory",
  "days": 7,
  "date": "2026-05-04",
  "mode": "reflection"
}
```

## 输出
```json
{
  "report": "## 🌙 凌晨反思...",
  "summary": {...},
  "from_cache": true
}
```
