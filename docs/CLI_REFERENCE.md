# CLI Reference

> 完整命令参考。人类日常只需 `just` 命令，以下内容供 AI agent 和高级用户参考。

## 核心命令

### `cogtome run`

运行 Skill / Unit / Motif（自动检测类型）。

```bash
cogtome run <name> --input '<json>'

# 示例
cogtome run daily-summary --input '{"date":"2026-05-05"}'
```

### `cogtome discover`

列出所有已注册的 Structure。

```bash
cogtome discover
```

### `cogtome serve`

启动 WebUI + HTTP API 服务。

```bash
cogtome serve --port 3334    # 默认端口 3334
```

### `cogtome pack`

打包 Skill 为 `.cogtome` 归档文件。

```bash
cogtome pack <name> --output <path>
```

### `cogtome install`

安装 `.cogtome` 归档。

```bash
cogtome install <file.cogtome>
```

---

## 高级命令

### `cogtome unit run`

直接运行 Unit（跳过 Structure/Motif 层）。

```bash
cogtome unit run <name> --input '<json>'

# 示例
cogtome unit run text-uppercase --input '{"text":"hello"}'
```

### `cogtome motif run`

直接运行 Motif（JSON DAG 编排）。

```bash
cogtome motif run <name> --input '<json>'

# 示例
cogtome motif run daily-summary --input '{}'
```

### `cogtome structure run`

直接运行 Structure。

```bash
cogtome structure run <name> --input '<json>'
```

### `cogtome mcp-server`

以 MCP Server 模式运行（stdio JSON-RPC 2.0）。

```bash
cogtome mcp-server --assemblies ./assemblies --units ./units --timeout 30
```

### `cogtome mcp-bridge`

将 MCP Server 工具作为 COGTOME Unit 使用。

```bash
cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool read_text_file \
  --args '{"path":"/tmp/test.txt"}' \
  --init-timeout 30 \
  --request-timeout 60
```

### `cogtome reload`

热重载所有 Structure 和 Motif 定义。

```bash
cogtome reload
```

### `cogtome validate`

验证 Motif 或 Structure manifest 文件。

```bash
cogtome validate <path-to-manifest>
```

### `cogtome stats`

显示 Assembly 调用热力图。

```bash
cogtome stats           # 基本统计
cogtome stats --heat    # 详细热力图
cogtome stats --gc      # 归档僵尸 Assembly
```

### `cogtome trace-dashboard`

启动 Trace 可视化面板。

```bash
cogtome trace-dashboard --port 4321
```

---

## 环境变量

| 变量 | 默认值 | 说明 |
|------|--------|------|
| `COGTOME_SKILLS_DIR` | `./skills` | Skills 根目录 |
| `COGTOME_TIMEOUT` | `30` | Unit 超时（秒） |
| `COGTOME_MAX_CONCURRENT` | `50` | foreach 最大并行数 |

## Unit 退出码

| 退出码 | 含义 |
|--------|------|
| `0` | 成功 |
| `1` | 输入错误（不重试） |
| `2` | 可重试错误 |
| `3` | 依赖不可用 |
