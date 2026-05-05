# COGTOME — Agent Coding Guide

> **TL;DR**: COGTOME is a Rust-based execution runtime for AI Agents. It uses a three-layer execution model (Structure → Motif → Unit) where Units are external processes communicating via stdin/stdout JSON, Motifs are DAG graphs (JSON), and the Runtime orchestrates them with process isolation, expression evaluation, and structured errors. Read this before modifying code.

---

## Build and Test Commands

```bash
# Build
cargo build              # debug
cargo build --release    # release

# Test (54 unit tests)
cargo test

# CLI
./target/release/cogtome --help
./target/release/cogtome discover

# Run examples
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# HTTP API
./target/release/cogtome serve --port 8080

# MCP
./target/release/cogtome mcp-bridge \
  --server "npx -y @modelcontextprotocol/server-filesystem /tmp" \
  --tool list_allowed_directories
./target/release/cogtome mcp-server --assemblies ./assemblies --units ./units

# Pack/install structures
./target/release/cogtome pack text-processing
./target/release/cogtome install ./text-processing.cogtome

# Hot reload
./target/release/cogtome reload
```

---

## Architecture: Three-Layer Model

```
Agent (natural language intent)
        |
        v
+---------------------+
|     Structure       |  <- Chains Motifs (skills/ or assemblies/)
|                     |     Name, description, input/output schema
+-------------+-------+
              |
              v
+---------------------+
|       Motif         |  <- Orchestration logic (JSON DAG)
|                     |     Nodes: start, unit, if, match, foreach, fork, join, return
+-------------+-------+
              | IPC (fork+exec, stdin/stdout JSON)
              v
+---------------------+
|        Unit         |  <- Atomic execution (independent process)
|                     |     Any language, JSON stdin/stdout
+---------------------+
```

**Core discipline:**
1. Units never call each other — Runtime blocks via `COGTOME_UNIT_MODE=1`
2. All cross-layer calls go through Runtime IPC
3. Schema validation at every boundary

---

## Code Organization

```
src/
├── main.rs              # CLI entry point (clap), command dispatch
├── config.rs            # cogtome.toml parsing
├── discovery.rs         # SkillsDir scanning, Structure discovery
├── validation.rs        # JSON Schema validation
├── pack.rs              # .cogtome archive pack/install
├── api.rs               # HTTP API server (axum)
├── error.rs             # CogtomeError, ErrorCode, exit-code mapping
├── metrics.rs           # In-memory metrics
├── assembly.rs          # Assembly manifest and AssemblyRegistry
├── mcp_server.rs        # MCP Server (JSON-RPC 2.0 over stdio)
├── shutdown.rs          # Graceful shutdown (SIGINT/SIGTERM)
├── context/
│   ├── variables.rs    # ExecContext: variable resolution, Arc-snapshot
│   └── expression.rs   # Expression engine: filter, map, conditions
├── engine/
│   ├── mod.rs          # GraphMotifEngine + StructureExecutor
│   ├── graph.rs        # MotifManifestV2, Graph validation
│   ├── unit_runner.rs  # UnitRunner: fork+exec, timeout, sandbox
│   └── mcp_bridge.rs   # McpBridgeUnit: run MCP Servers as Units
└── services/
    ├── discovery_service.rs
    └── validation_service.rs
```

---

## Structures Directory Layout

Two types of Structures chain Motifs:

```
skills/                              # Structure type 1: skills directory
├── units/<name>/bin/<name>         # Executable Unit
├── motifs/<name>.json               # Filename MUST match `name` field
└── <name>/SKILL.md                  # Structure manifest with YAML front matter

assemblies/                          # Structure type 2: MCP assemblies
└── <name>/
    ├── manifest.json                # Assembly manifest
    └── workflow.json                # MotifManifestV2 DAG
```

**Naming rules (violations cause "not found" errors):**
- Unit: `units/<name>/bin/<name>` (must be executable)
- Motif: `motifs/<name>.json` where `<name>` matches the file
- Assembly: `assemblies/<name>/` directory

**Note**: Current engine loads **JSON** manifests. Legacy `.yaml` files exist but are not loaded.

---

## Unit Execution Protocol

- **Input**: JSON object via stdin
- **Output**: First line of stdout MUST be valid JSON
- **Exit codes**: `0`=success, `1`=input error, `2`=retryable, `3`=dep unavailable
- **Timeout**: Default 30s, kills child on timeout
- **Sandbox**: Unique temp dir `/tmp/cogtome-exec-<uuid>/` as CWD
- **Env**: `COGTOME_UNIT_MODE=1` always set

Example (Python):
```python
#!/usr/bin/env python3
import sys, json
inp = json.load(sys.stdin)
print(json.dumps({"result": inp["text"].upper()}))
```

---

## Configuration (`cogtome.toml`)

Config order: `./cogtome.toml` → `$XDG_CONFIG_HOME/cogtome.toml` → `$HOME/.config/cogtome.toml`

```toml
[runtime]
max_iterations = 50        # foreach iteration limit
max_iterations_hard = 500  # absolute hard limit

[paths]
units = "units"
motifs = "motifs"
structures = "structures"
assemblies = "assemblies"

[units.defaults]
timeout_secs = 30

[units.concurrency.some-unit]
max_global = 3             # -1 = unlimited
resource_key = "shared_key"
```

**Environment variables:**
- `COGTOME_SKILLS_DIR` — skills root (default: `./skills`)
- `COGTOME_TIMEOUT` — unit timeout in seconds
- `COGTOME_MAX_CONCURRENT` — max parallel foreach (default: 50)
- `RUST_LOG` — tracing log level

---

## Code Style

- **All I/O must be async** using `tokio` APIs. No `std::fs`/`std::process` in async contexts (except startup config loading).
- **Error handling**: `anyhow::Result` at boundaries, `anyhow::Context` for rich errors, prefer `?` over unwrap.
- **JSON**: `serde_json::Value` for dynamic schemas, structs for manifests.
- **Naming**: `snake_case` functions/variables, `PascalCase` types. Layer names (Structure, Motif, Unit) are proper nouns.
- **ExecContext**: Uses `Arc<HashMap>` for O(1) snapshot. Preserve this pattern.
- **Tracing**: Use structured fields (`key = %value`) over string interpolation.

---

## Testing

```bash
cargo test              # all 54 tests
cargo test <test_name>  # run specific test

# Manual integration (after cargo build --release)
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
./target/release/cogtome motif run browser-fetch --input '{"url":"https://example.com"}'
./target/release/cogtome discover
```

Unit tests: `#[cfg(test)] mod tests` in each source file.
Integration tests: documented in `test_suite/` (manual CLI invocations).

---

## Security

**Current protections:**
- Process isolation (fork+exec, not threads)
- `COGTOME_UNIT_MODE=1` blocks recursive invocation
- Temp directory sandbox per execution
- Env var whitelist (not inherited by default)
- zip-slip protection in pack/install
- Name validation (rejects `..`, `/`, `\`)

**Known gaps:**
- No cgroups/seccomp yet
- No network namespace isolation
- Same OS user as Runtime

---

## HTTP API

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Returns `"OK"` |
| GET | `/metrics` | Metrics snapshot |
| GET | `/structures` | List all Structures |
| POST | `/run` | Execute via tagged union |

POST `/run` body:
```json
{"type": "structure", "name": "text-processing", "input": {"text": "hello"}}
{"type": "motif", "name": "browser-fetch", "input": {"url": "https://example.com"}}
{"type": "unit", "name": "text-uppercase", "input": {"text": "hello"}}
```

---

## Key Files to Read Before Changes

| Change | Read |
|--------|------|
| CLI / dispatch | `src/main.rs` |
| Config format | `src/config.rs`, `cogtome.toml` |
| Discovery | `src/discovery.rs` |
| Unit spawning | `src/engine/unit_runner.rs` |
| Motif execution | `src/engine/mod.rs`, `src/engine/graph.rs` |
| Variable resolution | `src/context/variables.rs` |
| Expression engine | `src/context/expression.rs` |
| HTTP API | `src/api.rs` |
| MCP integration | `src/mcp_server.rs`, `src/engine/mcp_bridge.rs` |
| Structure authoring | `development/SKILL_AUTHORING_GUIDE.md` |

---

*Last updated: 2026-05-04*