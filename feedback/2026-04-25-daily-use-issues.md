# COGTOME 日常使用反馈 (2026-04-25)

## 概述

用 COGTOME 构建了一个 web-fetch Skill，用于抓取网页内容。过程中遇到多个坑。

---

## 问题清单

### 1. 目录命名必须与 Structure name 匹配

**现象**：
```
structures/web-fetch/manifest.yaml
→ Error: Structure 'fetch' not found
```

**原因**：`find_structure()` 查找的是 `structures/<name>/manifest.yaml`，而不是 `structures/<dir-name>/manifest.yaml`。

**建议**：
- 文档中明确说明目录名 = structure name
- 或者自动扫描所有 structures 子目录

---

### 2. SKILL.md 的 path 路径是相对于 SKILL.md 本身

**现象**：
```yaml
structures:
  - path: ../structures/web-fetch  # 错误
  - path: ../structures/fetch     # 正确
```

**原因**：路径从 SKILL.md 所在目录计算，不是从 skills 根目录。

**建议**：
- 文档中用图示说明路径计算方式
- 或支持绝对路径（如 `skills/structures/fetch`）

---

### 3. Motif 文件名必须与 Motif name 一致

**现象**：
```
motifs/web-fetch.yaml + name: fetch-web
→ Error: Motif 'fetch-web' not found
```

**原因**：`find_motif(name)` 查找 `motifs/<name>.yaml`，而不是扫描所有 yaml 文件。

**建议**：
- 文档明确说明文件名 = motif name
- 或在 manifest 内声明 `name` 时也用文件名索引

---

### 4. Manifest 格式要求严格

**现象**：
```
missing field `type`
missing field `units_required`
```

**原因**：Structure 需要 `type: structure` + `input_schema`/`output_schema`，Motif 需要 `type: motif` + `flow`。

**建议**：
- 提供完整的 manifest 模板
- 启动时给出更友好的错误提示

---

### 5. Unit 依赖 jq 但系统未安装

**现象**：
```
jq: command not found
```

**建议**：
- 在文档中列出系统依赖
- 或使用更通用的 JSON 解析方式（如 grep/sed）

---

### 6. 缺少 foreach/aggregate

**场景**：想对抓取的多个 URL 内容分别处理并汇总，但 Phase 1 未实现。

**影响**：很多实际工作流程无法建模。

---

## 总结

**核心痛点**：文档不够明确，新手需要反复试错。

建议在 `IMPLEMENTATION_GUIDE.md` 旁边新增一个 `SKILL_AUTHORING_GUIDE.md`，包含：
1. 完整的文件布局示例（图）
2. 每种 manifest 的完整模板
3. 常见错误及解决方案
4. 系统依赖清单
