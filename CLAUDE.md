# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run

```bash
cargo build --release                    # Build
cargo test                               # Run all tests
cargo test discovery::tests              # Run specific test module
./target/release/cogtome discover       # Scan all Complexes
./target/release/cogtome run <complex>  # Execute Complex → Structure → Motif → Unit
./target/release/cogtome unit run <name> --input '<json>'  # Run Unit directly
./target/release/cogtome motif run <name> --input '<json>'  # Run Motif
./target/release/cogtome structure run <name> --input '<json>' # Run Structure
```

**Environment variables:**
- `COGTOME_SKILLS_DIR` — skills directory (default: `$(cargo manifest_dir)/skills`)
- `COGTOME_TIMEOUT` — unit timeout in seconds (default: 30)
- `COGTOME_MAX_CONCURRENT` — max parallel iterations in foreach (default: 50, min: 1)

## Architecture

COGTOME is a micro OS for AI Agents with four execution layers:

```
Agent → Complex (L4) → Structure (L3) → Motif (L2) → Unit (L1)
```

- **L4 Complex**: Domain facade with `SKILL.md` (only layer with `description`)
- **L3 Structure**: Business structure with `manifest.yaml`
- **L2 Motif**: Orchestration logic (YAML declarative, serial or parallel flow)
- **L1 Unit**: Atomic executor — fork+exec CLI, stdin/stdout JSON

**Core discipline**: Units never call each other. All cross-layer calls go through Runtime IPC.

## Source Modules (src/)

| File | Responsibility |
|------|----------------|
| `main.rs` | CLI entry via clap. Routes to Unit/Motif/Structure/Run/Discover commands. Loads `cogtome.toml` config |
| `engine.rs` | `UnitRunner` (tokio::process + semaphore rate limiting), `YamlMotifEngine` (serial/parallel execution), `StructureExecutor` |
| `context.rs` | `ExecContext` holds params + steps; `resolve_var()` handles `${params.x}`, `${steps.name.output.field}`, expressions, functions |
| `discovery.rs` | `SkillsDir` — two-phase lookup: global paths first, then Complex-private paths. `parse_skill_front_matter()` parses SKILL.md YAML front matter |
| `config.rs` | `CogtomeConfig` loads `cogtome.toml` with runtime, paths, and units concurrency settings |

**Unit contract**: stdin/stdout JSON, exit codes 0=success, 1=input error, 2=retryable, 3=dependency unavailable. Runtime injects `COGTOME_UNIT_MODE=1`, `COGTOME_EXECUTION_ID`, `COGTOME_TRACE_ID`.

## Skills Directory

```
skills/
├── units/<name>/bin/<name>          # Executable Unit
├── motifs/<name>.yaml              # YAML Motif
├── structures/<name>/manifest.yaml  # Structure manifest
└── <complex>/SKILL.md              # Complex with YAML front matter
```

Resolution order: global paths (`skills/units/` etc.) → Complex-private paths (`skills/<complex>/units/` etc.).

## Config File (cogtome.toml)

```toml
[runtime]
max_iterations = 50           # default per-foreach limit
max_iterations_hard = 500    # absolute hard limit

[paths]
units = "./skills/units"      # root for units (also COGTOME_SKILLS_DIR)
motifs = "./skills/motifs"   # motifs subdirectory
structures = "./skills/structures"  # structures subdirectory

[units.defaults]
timeout_secs = 30

[units.concurrency.<name>]   # per-unit concurrency config
max_global = 3               # semaphore permits
resource_key = "api_quota"   # share quota across units
```

## Key Implementation Notes

### Variable Resolution
- `${params.x}` — user input parameters
- `${steps.name.output.field}` — step outputs (Arc-cloned, O(1) snapshot)
- `${env.VAR}` — environment variables
- Array index: `${arr[0]}`, `${arr[-1]}` (negative from end)
- Length: `${arr.length}`

### Expression Functions
- `filter(arr, 'condition')` — filter array by condition expression
- `map(arr, 'expression')` — transform array elements
- Condition supports: `==`, `!=`, `>`, `<`, `>=`, `<=`, `&&`, `||`

### Foreach Loop
```yaml
foreach:
  over: "${items}"           # array expression
  as_var: "item"             # iteration variable
  max_iterations: 50         # per-foreach limit (capped by config)
  parallel: false            # true for concurrent execution
  on_error: fail_fast        # fail_fast | continue
  flow:
    - name: step1
      unit: my-unit
      input:
        x: "${item.value}"
  aggregate:
    mode: array             # array | object | sum | join
```

### Snapshot Semantics
Foreach iterations start from a snapshot of outer steps (read-only). Each iteration's step writes are isolated via copy-on-write `Arc<HashMap>`.

### Concurrency Control
- Parallel foreach uses `Semaphore` to limit concurrent iterations
- `COGTOME_MAX_CONCURRENT` env var controls max concurrency (default 50, min 1)
- Undeclared units get per-unit `Semaphore(1)` (serialized by default)
- Units with `resource_key` share a semaphore across the group

### Error Handling
- Exit codes: 0=success, 1=input error, 2=retryable, 3=dependency unavailable
- Timeout kill uses `Arc<Mutex<Option<Child>>>` pattern to avoid zombies
- Fail-fast cancels other iterations on first error
- Continue mode collects errors in `__error` field
