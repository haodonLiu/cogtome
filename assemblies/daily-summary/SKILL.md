# Daily Summary

AI Agent 的自我记忆整理工具——每日生成工作总结，识别可复用模式。

## 两种运行模式

### 1. Daily Review（日报）

每天结束前运行，生成当日工作总结：

```bash
cogtome run daily-summary --input '{
  "mode": "review",
  "date": "2026-04-30",
  "memory_base_path": "/home/haodont/.openclaw/workspace/memory"
}'
```

**输出内容：**
- ✅ 今日完成的任务
- 🔄 进行中的任务
- ⛔ 遇到的阻碍
- 💡 值得注意的决策
- 📝 下一步待办

### 2. Midnight Reflection（凌晨反思）

凌晨 3AM 自动运行（通过 cron 触发），分析近期记忆识别可抽象模式：

```bash
cogtome run daily-summary --input '{
  "mode": "reflection",
  "days": 7,
  "memory_base_path": "/home/haodont/.openclaw/workspace/memory"
}'
```

**输出内容：**
- 🔁 重复出现的工作模式
- 🧩 可封装成通用模块的建议
- 📌 当前项目的整体状态
- 🚀 优化建议

## Units

| Unit | 功能 |
|------|------|
| `memory-indexer` | 读取指定日期或日期范围的 memory 文件 |
| `task-extractor` | 从 memory 内容中抽取任务状态（完成/进行中/阻碍） |
| `pattern-finder` | 跨日分析，识别重复模式和可抽象的工作流 |
| `report-writer` | 生成结构化的总结报告 |

## 工作流

```
daily-summary (Match 路由)
├── review 分支:
│   └── memory-indexer → task-extractor → report-writer → return
└── reflection 分支:
    └── memory-indexer → task-extractor → pattern-finder → report-writer → return
```
