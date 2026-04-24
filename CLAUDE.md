# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run

```bash
cargo build --release                    # Build
cargo test                               # Run unit tests
./target/release/cogtome discover        # Scan all Complexes
./target/release/cogtome run <complex>   # Execute Complex → Structure → Motif → Unit
./target/release/cogtome unit run <name> # Run Unit directly
./target/release/cogtome motif run <name> # Run Motif
./target/release/cogtome structure run <name> # Run Structure
```

**Environment variables:**
- `COGTOME_SKILLS_DIR` — skills directory (default: `$(cargo manifest_dir)/skills`)
- `COGTOME_TIMEOUT` — unit timeout in seconds (default: 30)

## Architecture

COGTOME is a micro OS for AI Agents with four execution layers:

```
Agent → Complex (L4) → Structure (L3) → Motif (L2) → Unit (L1)
```

- **L4 Complex**: Domain facade with `SKILL.md` (only layer with `description`)
- **L3 Structure**: Business structure with `manifest.yaml`
- **L2 Motif**: Orchestration logic (YAML declarative, serial flow)
- **L1 Unit**: Atomic executor — fork+exec CLI, stdin/stdout JSON

**Core discipline**: Units never call each other. All cross-layer calls go through Runtime IPC.

## Source Modules (src/)

| File | Responsibility |
|------|----------------|
| `main.rs` | CLI entry via clap. Routes to Unit/Motif/Structure/Run/Discover commands |
| `engine.rs` | `UnitRunner` (tokio::process), `YamlMotifEngine` (serial execution), `StructureExecutor` |
| `context.rs` | `ExecCtx` holds params + steps; `resolve_var()` handles `${params.x}`, `${steps.name.output.field}`, `${env.VAR}` |
| `discovery.rs` | `SkillsDir` — two-phase lookup: global paths first, then Complex-private paths |

**Unit contract**: stdin/stdout JSON, exit codes 0=success, 1=input error, 2=retryable, 3=dependency unavailable. Runtime injects `COGTOME_UNIT_MODE=1`, `COGTOME_EXECUTION_ID`, `COGTOME_TRACE_ID`.

## Skills Directory

```
skills/
├── units/<name>/bin/<name>          # Executable Unit
├── motifs/<name>.yaml               # YAML Motif
├── structures/<name>/manifest.yaml  # Structure manifest
└── <complex>/SKILL.md               # Complex (must have description field)
```

Resolution order: global paths (`skills/units/` etc.) → Complex-private paths (`skills/<complex>/units/` etc.).

## Key Implementation Notes

- **Variable resolution**: `${params.x}`, `${steps.name.output.field}`, `${env.VAR}`
- **Array index access**: `${steps.a.output.field[0]}`, `${steps.a.output[-1]}` (negative from end)
- **Length property**: `${steps.a.output.array.length}`
- **Ternary expressions**: `${a > 5 ? 'big' : 'small'}`
- **if condition**: YAML step field `if: "${params.flag}"` skips step when false
- **foreach loop**: YAML block with `over`, `as_var`, `max_iterations`, `on_error`, `flow`, `aggregate`
- **Aggregate modes**: `array`, `object`, `sum`, `join`
- **Error strategies**: `fail_fast` (default), `continue`
- **Snapshot semantics**: foreach iterations start from snapshot of outer steps (read-only)
- **Process model**: Units are forked as independent processes via tokio::process::Command
