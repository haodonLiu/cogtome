# COGTOME — Agent Coding Guide

> **TL;DR**: COGTOME is a Rust-based micro-OS and execution runtime for AI Agents. It uses a four-layer execution model (Complex → Structure → Motif → Unit) where Units are external processes communicating via stdin/stdout JSON. Read this before modifying any code.

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
| Error handling | anyhow | 1.0 |
| Concurrency | tokio::sync (Semaphore, Mutex), futures | — |
| Packaging | tar + flate2 | 0.4 / 1.0 |

**No database, no ORM, no web framework beyond the API server.**

---

## 3. Build and Test Commands

```bash
# Build debug binary
cargo build

# Build release binary
cargo build --release

# Run tests (18 unit tests in expression engine and discovery)
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
```

---

## 4. Code Organization

```
src/
├── main.rs              # CLI entry point (clap subcommands), command dispatch
├── config.rs            # cogtome.toml parsing (RuntimeConfig, PathsConfig, UnitsConfig)
├── discovery.rs         # SkillsDir scanning, Complex discovery, SKILL.md front-matter parsing
├── validation.rs        # JSON Schema input validation (thin wrapper around jsonschema crate)
├── pack.rs              # .cogtome archive pack/install (tar.gz), with zip-slip protection
├── api.rs               # HTTP API server (axum): /health, /complexes, /run
├── python_motif.rs      # Python Motif engine stub (Unix Socket JSON-RPC, NOT YET ACTIVE)
├── context/
│   ├── mod.rs           # Module re-exports
│   ├── variables.rs     # ExecContext: variable resolution, step storage, snapshot semantics
│   └── expression.rs    # Expression engine: eval_condition, eval_expression, is_truthy, filter/map
└── engine/
    ├── mod.rs           # YamlMotifEngine + StructureExecutor orchestration
    ├── motif_manifest.rs# MotifManifest, StructureManifest, FlowStep, ForeachBlock, AggregateBlock serde types
    ├── unit_runner.rs   # UnitRunner: fork+exec, timeout, concurrency semaphore, temp sandbox
    └── foreach.rs       # execute_foreach_serial / execute_foreach_parallel, retry logic, events
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
│     Structure       │  ← Business black box. manifest.yaml defines input/output schema.
│   (Drive Train)     │     Chains motifs sequentially.
└─────────┬───────────┘
          │
          ▼
┌─────────────────────┐
│       Motif         │  ← Orchestration logic. YAML declarative flow.
│   (Gear Assembly)   │     Steps reference Units. Supports foreach, if, retry, on_error.
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
│   └── <motif-name>.yaml            # Filename MUST match `name` field inside
├── structures/
│   └── <structure-name>/            # Directory name MUST match `name` field inside
│       └── manifest.yaml
└── <complex-name>/                  # Complex directory
    └── SKILL.md                     # YAML front matter + markdown docs
```

**Critical naming rules** (violations cause "not found" errors):

| Element | Rule |
|---------|------|
| Unit binary | `units/<name>/bin/<name>` |
| Motif file | `motifs/<name>.yaml` where `<name>` == manifest `name` field |
| Structure dir | `structures/<name>/` where `<name>` == manifest `name` field |

Unit lookup is **global-first, then Complex-private**. Motif/Structure lookup follows the same pattern.

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

---

## 8. Code Style Guidelines

- **All I/O must be async** using `tokio` APIs. Do NOT use `std::fs` or `std::process` in async contexts (except during startup config loading).
- **Error handling**: Use `anyhow::Result` at boundaries, `anyhow::Context` for rich errors. Prefer `?` over unwrap.
- **JSON handling**: Use `serde_json::Value` for dynamic schemas, strongly-typed structs for manifests.
- **Comments**: Code comments and module doc comments are acceptable in either English or Chinese, matching surrounding context. Keep them concise.
- **Naming**: Follow Rust conventions (`snake_case` for functions/variables, `PascalCase` for types). The four-layer metaphor names (Complex, Structure, Motif, Unit) are proper nouns and use `PascalCase`.
- **Clone cost**: `ExecContext` uses `Arc<HashMap>` for steps to make fork/snapshot O(1). If you modify step storage, preserve this pattern.
- **Dead code**: The codebase has `#[allow(dead_code)]` on some fields intentionally (planned features). Only remove if you are certain the feature is cancelled.

---

## 9. Testing Instructions

### Existing Tests
Run `cargo test`. Currently 18 tests covering:
- `context::expression` — truthiness, comparisons, array/object equality, eval_expression, eval_condition
- `discovery` — SKILL.md front matter parsing, description extraction, structure extraction

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
curl -X POST http://localhost:8080/run \
  -H "Content-Type: application/json" \
  -d '{"type":"complex","name":"text-processing","input":{"text":"hello"}}'
```

### Adding New Tests
- Add unit tests in the same file under `#[cfg(test)] mod tests` (existing pattern in `expression.rs` and `discovery.rs`).
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
3. Reuse `UnitRunner`, `YamlMotifEngine`, `StructureExecutor` — do not bypass the engine layers.
4. Update `README.md` CLI Reference section.

### When Adding a New Engine Feature
1. Define manifest types in `src/engine/motif_manifest.rs`.
2. Implement execution logic in the appropriate engine file (`engine/mod.rs` for motifs/structures, `engine/foreach.rs` for loops, `engine/unit_runner.rs` for process spawning).
3. Update `ExecContext` in `src/context/variables.rs` if you need new variable resolution rules.
4. Update `src/context/expression.rs` if you need new expression syntax.

### When Adding a Skill (Unit/Motif/Structure/Complex)
1. Follow the naming rules in Section 5 exactly.
2. Refer to `development/SKILL_AUTHORING_GUIDE.md` for complete manifest templates.
3. Ensure `SKILL.md` has YAML front matter delimited by `---`.
4. Test with the actual CLI before committing.

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
| GET | `/health` | Returns "OK" |
| GET | `/complexes` | List all discovered Complexes |
| GET | `/complexes/{name}` | Get Complex metadata from SKILL.md front matter |
| POST | `/run` | Execute Complex/Motif/Structure/Unit via JSON body |

POST `/run` body format (tagged union):
```json
{"type": "complex", "name": "text-processing", "input": {"text": "hello"}}
{"type": "motif", "name": "text-transform", "input": {"text": "hello"}}
{"type": "structure", "name": "text-pipeline", "input": {"text": "hello"}}
{"type": "unit", "name": "text-uppercase", "input": {"text": "hello"}}
```

---

## 13. Key Files to Read Before Major Changes

| If you want to change... | Read these files first |
|--------------------------|------------------------|
| CLI / command dispatch | `src/main.rs` |
| Config format | `src/config.rs`, `cogtome.toml` |
| Discovery / path resolution | `src/discovery.rs` |
| Unit process spawning | `src/engine/unit_runner.rs` |
| Motif execution / flow control | `src/engine/mod.rs`, `src/engine/foreach.rs` |
| Variable resolution / snapshots | `src/context/variables.rs` |
| Expression syntax | `src/context/expression.rs` |
| Manifest schemas | `src/engine/motif_manifest.rs` |
| JSON Schema validation | `src/validation.rs` |
| HTTP API | `src/api.rs` |
| Skill packaging | `src/pack.rs` |
| Full architecture spec | `development/TECHNICAL_SPEC.md` |
| Skill authoring rules | `development/SKILL_AUTHORING_GUIDE.md` |

---

## 14. Project Status (Phase 1 Complete)

**Implemented (Phase 1):**
- CLI framework with discover, run, unit run, motif run, structure run, serve, pack, install, reload
- Unit execution (fork+exec, stdin/stdout JSON, timeout, temp sandbox)
- YAML Motif parsing and serial flow execution
- Structure → Motif → Unit chain
- Complex discovery (SKILL.md front-matter parsing)
- foreach loops with aggregate (array, object, sum, join)
- Expression engine (variable resolution, array indexing, negative indices, length, ternary, filter/map)
- if conditional execution
- max_iterations hard limit
- Retry with exponential/linear backoff
- Error strategies (fail, continue, fallback) at step and foreach level
- Parallel foreach with concurrency limiter and cancellation token
- HTTP API server
- Pack/install with tar.gz
- Hot reload command

**Planned (Phase 2+):**
- Python Motif engine via Unix Socket (stub exists in `src/python_motif.rs`)
- Full MCP protocol state machine
- auto-complex registration
- File-system event-based auto-reload (notify crate)
- Execution logs and inspect API

---

*Last updated: 2026-04-25*
*Version: 0.2.0*
