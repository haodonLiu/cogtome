---
name: text-processing
description: |
  文本处理领域。当任务涉及文本转换、格式化、大写/小写、反转、拼接、
  简单字符串操作时，自动调用此 Skill。

structures:
  - name: text-pipeline
    path: structures/text-pipeline
    summary: "标准文本处理流水线"
    scenarios: ["文本大写", "文本反转", "字符串变换"]
    weight: 1.0

config:
  default_timeout: 10
  log_retention: "1d"
---
