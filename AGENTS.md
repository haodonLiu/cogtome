# COGTOME — Agent Coding Guide

> **TL;DR**: COGTOME is a Rust-based micro-OS and execution runtime for AI Agents. It uses a four-layer execution model (Complex → Structure → Motif → Unit) where Units are external processes communicating via stdin/stdout JSON, Motifs are DAG graphs (JSON), and the Runtime orchestrates them with process isolation, expression evaluation, and structured errors. Read this before modifying any code.

---

## 1. Project Overview

COGTOME ("Gears turn the tome, mechanics execute the craft") is **not** a framework or library — it is an **independent process-level runtime**: a micro operating system for Agents.

| OS Concept | COGTOME Equivalent |
|-----------|-------------------|
| Kernel | COGTOME Runtime (Rust + Tokio) |
| User Process | Agent (LLM / Program) |
| System Call | Unit (atomic execution) |
| User-space Function | Motif (orchestration logic) |
| Application | Structure (business encapsulation) |
| App Store | Complex (domain facade) |
| Shell | `cogtome` CLI |

**Core problem it solves**: Agents need to call external tools, but direct `subprocess` calls cause process management chaos, no type safety, no versioning/discovery, and no execution trace tracking. COGTOME provides disciplined execution infrastructure so Agents can focus on business logic.

**Key discipline**: The Runtime has zero business logic. The binary ships with no built-in Units.

---

## 2. Technology Stack

| Component | Choice | Version |
|-----------|--------|---------|
| Language | Rust | 2021 edition, 1.70+ |
| Async runtime | Tokio | 1.37 (full features) |
| CLI framework | clap | 4.5 (derive features) |
| Serialization | serde + serde_json + serde_yaml | 1.0 / 0.9 |
| JSON Schema | jsonschema | 0.26 |
| Config parsing | toml | 0.8 |
| HTTP API | axum + tower + tower-http | 0.7 / 0.4 / 0.5 |
| Error handling | anyhow + thiserror | 1.0 |
| Concurrency | tokio::sync (Semaphore, Mutex), futures | — |
| Packaging | tar + flate2 | 0.4 / 1.0 |
| Observability | tracing + tracing-subscriber | 0.1 / 0.3 |
| Web UI | React + TypeScript + Vite + Zustand + React Flow | 18 / 5 / 5 / 4 / 12 |

**No database, no ORM, no web framework beyond the API server.**

---

## 3. Build and Test Commands

```bash
# Build debug binary
cargo build

# Build release binary
cargo build --release

# Run tests (54 unit tests across expression, variables, discovery, graph, config, error, validation, assembly, mcp_bridge, shutdown)
cargo test

# Run the CLI
./target/release/cogtome --help

# Example execution flows
./target/release/cogtome discover
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'
./target/release/cogtome structure run text-pipeline --input '{"text":"hello"}'
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# Start HTTP API server
./target/release/cogtome serve --port 8080

# Pack / install skills
./target/release/cogtome pack text-processing
./target/release/cogtome install ./text-processing.cogtome

# Hot reload (re-discover all skills)
./target/release/cogtome reload

# Validate a manifest file
./target/release/cogtome validate ./skills/structures/text-pipeline/manifest.json

# MCP Bridge: run an MCP Server tool
./target/release/cogtome mcp-bridge --server "npx -y @modelcontextprotocol/server-filesystem /tmp" --tool list_allowed_directories

# MCP Server (stdio JSON-RPC mode)
./target/release/cogtome mcp-server --assemblies ./assemblies --units ./units

# Web UI (development)
cd webui && npm install && npm run dev
```

---

## 4. Code Organization

```
src/
├── main.rs              # CLI entry point (clap subcommands), command dispatch
├── config.rs            # cogtome.toml parsing (RuntimeConfig, PathsConfig, UnitsConfig)
├── discovery.rs         # SkillsDir scanning, Complex discovery, SKILL.md front-matter parsing
├── validation.rs        # JSON Schema input validation + manifest validation (Motif/Structure)
├── pack.rs              # .cogtome archive pack/install (tar.gz), with zip-slip protection
├── api.rs               # HTTP API server (axum): /health, /metrics, /complexes, /run, CRUD APIs
├── error.rs             # Structured error types: CogtomeError, ErrorLayer, ErrorCode, exit-code mapping
├── metrics.rs           # In-memory metrics (unit executions, durations, requests, foreach counters)
├── assembly.rs          # Assembly manifest parsing and AssemblyRegistry (for MCP server)
├── mcp_server.rs        # MCP Server implementation (JSON-RPC 2.0 over stdio)
├── shutdown.rs          # Graceful shutdown manager (SIGINT/SIGTERM handling, CancellationToken)
├── context/
│   ├── mod.rs           # Module re-exports
│   ├── variables.rs     # ExecContext: variable resolution, step storage, snapshot semantics, ternary/filter/map
│   └── expression.rs    # Expression engine: eval_condition, eval_expression, is_truthy, comparisons
├── engine/
│   ├── mod.rs           # GraphMotifEngine + StructureExecutor orchestration
│   ├── graph.rs         # MotifManifestV2, Graph, Node, Edge, graph validation (cycles, reachability, edges)
│   ├── unit_runner.rs   # UnitRunner: fork+exec, timeout, concurrency semaphore, temp sandbox, metrics
│   └── mcp_bridge.rs    # McpBridgeUnit: run external MCP Servers as COGTOME Units
└── services/
    ├── mod.rs           # Service module re-exports
    ├── discovery_service.rs # List/query structures, motifs, units for HTTP API
    └── validation_service.rs # Validate motifs and structures by name for HTTP API
```

### Four-Layer Model (L1–L4)

```
Agent (natural language intent)
        │
        ▼
┌─────────────────────┐
│      Complex        │  ← Only layer visible to Agent. Has description, auto-discovered.
│   (Domain Tome)     │     Defined by <complex>/SKILL.md with YAML front matter.
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│     Structure       │  ← Business black box. manifest.json defines input/output schema.
│   (Drive Train)     │     Chains motifs sequentially.
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif         │  ← Orchestration logic. JSON graph with nodes and edges.
│   (Gear Assembly)   │     Supports start, unit, if, match, foreach, fork, join, return, motifRef.
└─────────┬───────────┘
          │ IPC (fork+exec, stdin/stdout JSON)
          ▼
┌─────────────────────┐
│        Unit         │  ← Atomic execution. Independent process.
│       (Cog)         │     Any language. Reads JSON from stdin, prints JSON to stdout.
└─────────────────────┘
```

**Core discipline (hard rules)**:
1. **Units never call each other** — Runtime blocks via `COGTOME_UNIT_MODE=1`
2. **Motifs don't directly call each other** — composed via Structure
3. **Structure doesn't directly call Unit** — must go through Motif
4. **Complex is the only layer with `description`**
5. **All cross-layer calls go through Runtime IPC**

---

## 5. Skills Directory Layout

The Runtime scans a `skills/` directory at runtime. Default path: `./skills`, overridable via `COGTOME_SKILLS_DIR`.

```
skills/
├── units/
│   └── <unit-name>/
│       └── bin/
│           └── <unit-name>          # Must be executable. Any language.
├── motifs/
│   └── <motif-name>.json            # Filename MUST match `name` field inside
├── structures/
│   └── <structure-name>/            # Directory name MUST match `name` field inside
│       └── manifest.json
└── <complex-name>/                  # Complex directory
    └── SKILL.md                     # YAML front matter + markdown docs
```

**Critical naming rules** (violations cause "not found" errors):

| Element | Rule |
|---------|------|
| Unit binary | `units/<name>/bin/<name>` |
| Motif file | `motifs/<name>.json` where `<name>` == manifest `name` field |
| Structure dir | `structures/<name>/` where `<name>` == manifest `name` field |

Unit lookup is **global-first, then Complex-private**. Motif/Structure lookup follows the same pattern.

**Note**: Legacy `.yaml` motif and structure files exist in the repository but the **current engine only loads JSON manifests**. The YAML files are documentation artifacts from an earlier serial-flow engine.

---

## 6. Unit Execution Protocol

Units are external executables. COGTOME runs them via `tokio::process::Command` with these guarantees:

- **Input**: JSON object written to stdin
- **Output**: First line of stdout MUST be valid JSON (the result). Remaining stdout is ignored.
- **Errors**: Non-zero exit code → error, stderr included in error message
- **Timeout**: Configurable, default 30s. Kills child on timeout.
- **Sandbox**: Each execution gets a unique temp dir (`/tmp/cogtome-exec-<uuid>/`) as CWD
- **Environment**: `COGTOME_UNIT_MODE=1` is always set. Only whitelisted env vars are inherited.
- **Concurrency**: Semaphore-based rate limiting per Unit (configurable in `cogtome.toml`)
- **Exit code semantics**:
  - `0` — Success
  - `1` — Input error (do not retry)
  - `2` — Retryable error
  - `3` — Dependency unavailable

Example Unit (Python):
```python
#!/usr/bin/env python3
import sys, json
inp = json.load(sys.stdin)
print(json.dumps({"result": inp["text"].upper()}))
```

---

## 7. Configuration (`cogtome.toml`)

Config resolution order: `./cogtome.toml` → `$XDG_CONFIG_HOME/cogtome.toml` → `$HOME/.config/cogtome.toml` → defaults.

```toml
[runtime]
max_iterations = 50        # Default foreach iteration limit
max_iterations_hard = 500  # Absolute hard limit (cannot be overridden per-motif)

[paths]
units = "units"            # Subdir under skills root (or absolute path)
motifs = "motifs"
structures = "structures"
assemblies = "assemblies"  # Used by MCP server

[units.defaults]
timeout_secs = 30

[units.concurrency.some-unit]
max_global = 3
max_per_host = 1
resource_key = "openai_api"
```

Environment variable overrides:
- `COGTOME_SKILLS_DIR` — skills root directory
- `COGTOME_TIMEOUT` — unit timeout in seconds
- `COGTOME_MAX_CONCURRENT` — max parallel foreach concurrency (default 50)
- `COGTOME_LOG_FORMAT` — `pretty` or `json`
- `RUST_LOG` — tracing log level

---

## 8. Code Style Guidelines

- **All I/O must be async** using `tokio` APIs. Do NOT use `std::fs` or `std::process` in async contexts (except during startup config loading).
- **Error handling**: Use `anyhow::Result` at boundaries, `anyhow::Context` for rich errors. Prefer `?` over unwrap. Use `CogtomeError` for structured errors crossing API boundaries.
- **JSON handling**: Use `serde_json::Value` for dynamic schemas, strongly-typed structs for manifests.
- **Comments**: Code comments and module doc comments are acceptable in either English or Chinese, matching surrounding context. Keep them concise.
- **Naming**: Follow Rust conventions (`snake_case` for functions/variables, `PascalCase` for types). The four-layer metaphor names (Complex, Structure, Motif, Unit) are proper nouns and use `PascalCase`.
- **Clone cost**: `ExecContext` uses `Arc<HashMap>` for steps to make fork/snapshot O(1). If you modify step storage, preserve this pattern.
- **Dead code**: The codebase has `#[allow(dead_code)]` on some fields intentionally (planned features). Only remove if you are certain the feature is cancelled.
- **Tracing**: Use `tracing::info!`, `tracing::error!`, etc. with structured fields (`key = %value`) rather than string interpolation for observable logs.

---

## 9. Testing Instructions

### Existing Tests
Run `cargo test`. Currently **54 tests** covering:
- `context::expression` — truthiness, comparisons, array/object equality, eval_expression, eval_condition
- `context::variables` — variable resolution, steps, locals, fork/snapshot, ternary, filter, map, array indexing, length
- `discovery` — SKILL.md front matter parsing, description extraction, structure extraction
- `engine::graph` — graph validation, cycle detection, if-node edge labels, foreach subgraph validation
- `config` — default config, load config, minimal config
- `error` — error display, exit-code mapping, serialization
- `validation` — motif validation, structure validation, empty motifs
- `assembly` — manifest parsing
- `engine::mcp_bridge` — MCP filesystem bridge integration tests
- `shutdown` — graceful shutdown creation, cancellation tokens

### Manual Integration Tests
After `cargo build --release`:

```bash
# Unit layer
./target/release/cogtome unit run text-uppercase --input '{"text":"hello"}'
# Expected: {"result":"HELLO"}

# Motif layer
./target/release/cogtome motif run text-transform --input '{"text":"hello"}'
# Expected: {"upper":"HELLO","reversed":"olleh","combined":"HELLO | olleh"}

# Structure layer
./target/release/cogtome structure run text-pipeline --input '{"text":"hello"}'

# Complex layer
./target/release/cogtome run text-processing --input '{"text":"hello"}'

# Discovery
./target/release/cogtome discover

# HTTP API (in one terminal)
./target/release/cogtome serve --port 8080
# In another:
curl http://localhost:8080/complexes
curl http://localhost:8080/metrics
curl -X POST http://localhost:8080/run \
  -H "Content-Type: application/json" \
  -d '{"type":"complex","name":"text-processing","input":{"text":"hello"}}'
```

### Adding New Tests
- Add unit tests in the same file under `#[cfg(test)] mod tests` (existing pattern).
- For integration tests involving actual Units, add them to `test_suite/COMPREHENSIVE_TESTS.md` and update `test_suite/README.md`.
- There is no dedicated `tests/` directory yet; integration tests are currently manual CLI invocations documented in `test_suite/`.

---

## 10. Security Considerations

### Current Protections
1. **Process isolation**: Units run in separate fork+exec processes, not threads.
2. **Unit mode lock**: `COGTOME_UNIT_MODE=1` prevents Units from recursively invoking COGTOME.
3. **Temp directory sandbox**: Each Unit execution gets a unique `/tmp/cogtome-exec-<uuid>/` working directory.
4. **Env var whitelist**: Units do NOT inherit the parent environment unless variables are explicitly whitelisted in the Motif step (`env_whitelist`).
5. **Archive path traversal protection**: `pack::install` validates extracted paths to prevent zip-slip attacks.
6. **Name validation**: API endpoints validate names reject empty strings, `..`, `/`, and `\`.
7. **Structured errors**: Unit exit codes are mapped to specific `ErrorCode` values to prevent information leakage.

### Known Gaps (Documented in `feedback/`)
- No memory/CPU cgroups or seccomp on Units yet.
- No network namespace isolation.
- Unit and Runtime run as the same OS user.
- These are acknowledged in `feedback/2026-04-25-architecture-review.md` as P1 items.

---

## 11. Development Conventions

### When Adding a New CLI Command
1. Add the command variant to `Commands` enum in `src/main.rs` (follow existing Chinese doc-comment style for subcommand help text).
2. Implement command logic in the `match cli.command` block.
3. Reuse `UnitRunner`, `GraphMotifEngine`, `StructureExecutor` — do not bypass the engine layers.
4. Update `README.md` CLI Reference section.

### When Adding a New Engine Feature
1. Define graph node types in `src/engine/graph.rs`.
2. Implement execution logic in the appropriate engine file (`engine/mod.rs` for motif/structure execution, `engine/unit_runner.rs` for process spawning).
3. Update `ExecContext` in `src/context/variables.rs` if you need new variable resolution rules.
4. Update `src/context/expression.rs` if you need new expression syntax.

### When Adding a Skill (Unit/Motif/Structure/Complex)
1. Follow the naming rules in Section 5 exactly.
2. Refer to `development/SKILL_AUTHORING_GUIDE.md` for complete manifest templates.
3. Ensure `SKILL.md` has YAML front matter delimited by `---`.
4. Ensure Motif manifests are **JSON** with a `graph` object containing `nodes` and `edges`.
5. Test with the actual CLI before committing.

### Documentation Updates
- `README.md` — Human-facing quick start and project description (English).
- `development/TECHNICAL_SPEC.md` — Architecture spec (Chinese).
- `development/IMPLEMENTATION_GUIDE.md` — Implementation pseudocode and sequencing (Chinese).
- `development/SKILL_AUTHORING_GUIDE.md` — Skill developer guide (English).
- `test_suite/` — Test cases and execution logs.
- `feedback/` — Architecture review feedback and daily use issues (dated files).

---

## 12. HTTP API Endpoints

Implemented in `src/api.rs`:

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Returns `"OK"` |
| GET | `/metrics` | Returns JSON metrics snapshot (units, durations, requests, foreach) |
| GET | `/complexes` | List all discovered Complexes |
| GET | `/complexes/:name` | Get Complex metadata from SKILL.md front matter |
| POST | `/run` | Execute Complex/Motif/Structure/Unit via JSON body |
| GET | `/api/structures` | List all structures |
| GET | `/api/structures/:name` | Get structure manifest JSON |
| PUT | `/api/structures/:name` | Create or update a structure |
| DELETE | `/api/structures/:name` | Delete a structure |
| GET | `/api/motifs` | List all motifs |
| GET | `/api/motifs/:name` | Get motif JSON content |
| PUT | `/api/motifs/:name` | Create or update a motif |
| GET | `/api/units` | List all units |
| GET | `/api/units/:name` | Get unit metadata (stub) |
| PUT | `/api/units/:name` | Create a unit (generates bash stub) |
| POST | `/api/validate/:type/:name` | Validate a motif or structure by name |

POST `/run` body format (tagged union):
```json
{"type": "complex", "name": "text-processing", "input": {"text": "hello"}}
{"type": "motif", "name": "text-transform", "input": {"text": "hello"}}
{"type": "structure", "name": "text-pipeline", "input": {"text": "hello"}}
{"type": "unit", "name": "text-uppercase", "input": {"text": "hello"}}
```

The server also serves the Web UI static files from `webui/dist/` if present.

---

## 13. Key Files to Read Before Major Changes

| If you want to change... | Read these files first |
|--------------------------|------------------------|
| CLI / command dispatch | `src/main.rs` |
| Config format | `src/config.rs`, `cogtome.toml` |
| Discovery / path resolution | `src/discovery.rs` |
| Unit process spawning | `src/engine/unit_runner.rs` |
| Motif execution / flow control | `src/engine/mod.rs`, `src/engine/graph.rs` |
| Variable resolution / snapshots | `src/context/variables.rs` |
| Expression syntax | `src/context/expression.rs` |
| Manifest schemas | `src/engine/graph.rs` |
| JSON Schema validation | `src/validation.rs` |
| HTTP API | `src/api.rs` |
| Skill packaging | `src/pack.rs` |
| MCP integration | `src/mcp_server.rs`, `src/engine/mcp_bridge.rs` |
| Assembly registry | `src/assembly.rs` |
| Metrics | `src/metrics.rs` |
| Full architecture spec | `development/TECHNICAL_SPEC.md` |
| Skill authoring rules | `development/SKILL_AUTHORING_GUIDE.md` |

---

## 14. Project Status (Phase 1 Complete)

**Implemented (Phase 1):**
- CLI framework with discover, run, unit run, motif run, structure run, serve, pack, install, reload, validate, mcp-bridge, mcp-server
- Unit execution (fork+exec, stdin/stdout JSON, timeout, temp sandbox, structured exit codes)
- JSON Graph Motif engine with graph validation (cycles, reachability, edge constraints)
- Structure → Motif → Unit chain
- Complex discovery (SKILL.md front-matter parsing)
- Graph node types: start, unit, if, match, foreach, fork, join, return, motifRef
- foreach loops with subgraphs
- Expression engine (variable resolution, array indexing, negative indices, length, ternary, filter/map)
- if conditional execution with true/false labeled edges
- max_iterations hard limit
- Error strategies (fail, continue, fallback) at unit node level
- HTTP API server with CORS
- Pack/install with tar.gz
- Hot reload command
- In-memory metrics with `/metrics` endpoint
- MCP Bridge Unit (run external MCP Servers)
- MCP Server mode (stdio JSON-RPC)
- Assembly registry for publishable skills
- Graceful shutdown (SIGINT/SIGTERM)
- Web UI with React Flow graph editor

**Planned (Phase 2+):**
- Python Motif engine via Unix Socket
- File-system event-based auto-reload (notify crate)
- Execution logs and inspect API
- Docker Unit Runner for untrusted tools

---

*Last updated: 2026-04-29*
*Version: 0.2.0*
