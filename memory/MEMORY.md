# COGTOME Memory — Long-Term

## Architecture Decisions

- **skills/ vs assemblies/** — parallel systems: `skills/` for `cogtome run` CLI, `assemblies/` for MCP Server
- **Units exit codes**: `0`=success, `1`=input error, `2`=retryable, `3`=dep unavailable
- **Rust-only tools** — trace-analyzer, all Units; no Python in COGTOME itself
- **JSONL over SQLite** — trace logs at `~/.cogtome/traces/<skill>/<date>.jsonl`, Agent analyzes directly

## Self-Evolution

### Phase 1 ✅
- `emit_trace()` hook in `GraphMotifEngine::execute()` — Rust direct file write
- `trace-logger` Unit (bash) — append-only JSONL

### Phase 2 ⬜
- `trace-analyzer` Unit (Rust binary) — reads JSONL, outputs suggestions
- Insert into midnight-reflection motif

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

- `avg_ms` per node is 0 — `emit_trace` doesn't capture per-node wall-clock time (Engine lacks node-level timing). Total `duration_ms` is accurate.
