---
name: core-tools
description: |
  COGTOME 核心工具集。封装 OpenClaw 内置工具为标准化 Unit，
  提供文件操作、Shell 执行等基础能力。

structures:
  - name: shell-executor
    path: ../structures/shell-executor
    summary: "执行 Shell 命令"
    scenarios: ["运行命令", "系统交互", "脚本执行"]
    weight: 1.0
  - name: file-operations
    path: ../structures/file-operations
    summary: "文件读写操作"
    scenarios: ["读取文件", "写入文件", "创建目录"]
    weight: 1.0
---

# Core Tools

OpenClaw 内置工具的 COGTOME 封装层。

## Units

| Unit | 封装 | 功能 |
|------|------|------|
| `shell-run` | exec | 执行 shell 命令 |
| `file-read` | read | 读取文件 |
| `file-write` | write | 写入文件 |

## 使用方式

```bash
# 通过 Complex + Structure 调用
cogtome run core-tools --input '{"command": "pwd"}'
```
