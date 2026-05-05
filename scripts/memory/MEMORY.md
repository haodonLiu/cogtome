# COGTOME Memory — Long-Term

## Architecture Decisions

- **skills/ vs assemblies/** — parallel systems: `skills/` for `cogtome run` CLI, `assemblies/` for MCP Server
- **Units exit codes**: `0`=success, `1`=input error, `2`=retryable, `3`=dep unavailable
- **Rust-only tools** — trace-analyzer, all Units; no Python in COGTOME itself
- **JSONL over SQLite** — trace logs at `~/.cogtome/traces/<skill>/<date>.jsonl`, Agent analyzes directly

## Phase 2 Update (2026-04-30) — SandboxBackend Trait

### Direction: 分离隔离层和编排层
- 不要自己造 Docker runner
- 集成成熟沙箱方案（E2B / Bubblewrap / QuickJS）
- 在 `unit_runner.rs` 抽象 `SandboxBackend` trait

### SandboxBackend Trait Design
```rust
trait SandboxBackend {
    fn execute(&self, unit: &Unit, input: Value, timeout: Duration) -> Result<Value>;
}
struct BubblewrapBackend;   // 本地默认，毫秒级，零成本
struct E2bBackend { client: E2bClient };  // 远程，强隔离，按调用计费
struct NullBackend;  // 现有 fork+exec，保留
```

### Isolation 分层策略
| 场景 | 后端 | 原因 |
|------|------|------|
| 本地开发 / 自有脚本 | bubblewrap | 毫秒级，零网络依赖，零成本 |
| CI / 不可信第三方 | e2b | Firecracker microVM，执行完即销毁 |
| JS/TS 轻量片段 | quickjs | 亚毫秒级，Wasm 线性内存隔离 |
| 回退 / macOS-Windows | none | 降级到现有 fork+exec |

### Unit manifest isolation 字段
```yaml
# cogtome.toml 全局默认
[sandbox]
default = "bubblewrap"

# Unit manifest 可覆盖
isolation: e2b  # 必须联网的 Unit
isolation: bubblewrap  # 本地处理
```

### MCP Bridge 沙箱注意
- MCP Server 通常通过 `npx`/`uvx` spawn 子进程
- Bubblewrap 需要 `--proc /proc` 挂载
- E2B microVM 内无限制，推荐 MCP Bridge 用 e2b

### CC 研究清单
1. E2B Python SDK `Sandbox` 类：`start_and_wait` 同步阻塞调用
2. 文件系统 API 延迟 + 批量写入支持
3. 环境变量注入（`COGTOME_UNIT_MODE=1`）
4. Hobby plan 并发 sandbox 数量限制
5. 成本模型：按秒还是按 API 调用？20 Unit/Motif 成本

## Self-Evolution

### Phase 1 ✅
- `emit_trace()` hook in `GraphMotifEngine::execute()` — Rust direct file write
- `trace-logger` Unit (bash) — append-only JSONL

### Phase 2 ✅ (进行中)
- `trace-analyzer` Unit (Rust binary) — reads JSONL, outputs suggestions ✅
- Midnight-reflection motif 接入 trace-analyzer ✅
- SandboxBackend trait 设计完成 ⬜
- BubblewrapBackend 实现 ⬜
- E2bBackend 实现 ⬜

### Phase 3 ⬜
- Agent reads suggestions → modifies Assemblies/Units via `DagMutator` API
- Gate: human reviews changes before apply

## Key Files

| File | Purpose |
|------|---------|
| `src/engine/mod.rs` | execute(), emit_trace(), format_time(), format_date() |
| `SPEC.md` | Self-evolution design doc |
| `tools/trace-analyzer/` | Rust trace analyzer |
| `units/trace-logger/` | bash JSONL appender |
| `units/trace-analyzer/` | Rust binary copy |

## Traced Skills

- `daily-summary` → `~/.cogtome/traces/daily-summary/`

## Known Issues

- ~~`avg_ms` per node is 0~~ — FIXED: per-node Instant timing added in `87a2ed4`
