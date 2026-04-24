# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build and Run

```bash
# Build
cargo build --release

# Run CLI
cargo run -- run text-processing --input '{"text":"hello"}'
cargo run -- discover

# Run a single Unit directly
cargo run -- unit run text-uppercase --input '{"text":"hello"}'

# Run a Motif
cargo run -- motif run text-transform --input '{"text":"hello"}'
```

## Architecture: Four-Layer Model

COGTOME is a micro operating system for AI Agents with four execution layers:

```
Agent → Complex (L4) → Structure (L3) → Motif (L2) → Unit (L1)
```

| Layer | Name | Visibility | Definition |
|-------|------|------------|------------|
| L4 | Complex | Agent-visible (only layer with `description`) | Domain facade with SKILL.md |
| L3 | Structure | Hidden | Business structure with manifest.yaml |
| L2 | Motif | Hidden | Orchestration logic (.yaml, .py, .sh) |
| L1 | Unit | Hidden | Atomic executor (fork+exec CLI with stdin/stdout JSON) |

**Core discipline**: Units never call each other. All cross-layer calls go through Runtime IPC.

## Source Modules (src/)

- **main.rs**: CLI entry point using clap. Routes to `Commands::Unit|Motif|Structure|Run|Discover`
- **engine.rs**: Core execution - `UnitRunner` (tokio::process), `YamlMotifEngine` (serial flow execution), `StructureExecutor` (motif chain)
- **context.rs**: `ExecContext` holds params + steps. `resolve_var()` handles `${params.x}`, `${steps.name.output.field}`, `${env.VAR}`
- **discovery.rs**: `SkillsDir` handles path resolution. Three-level lookup: global → Complex-private → system PATH

## Skills Directory Structure

```
skills/
├── units/<name>/bin/<name>     # Executable Unit (any language)
├── motifs/<name>.yaml          # YAML Motif definition
├── structures/<name>/manifest.yaml
└── <complex>/SKILL.md          # Complex definition (must have description)
```

**Unit contract**: stdin/stdout JSON, exit codes 0=success, 1=input error, 2=retryable, 3=dependency unavailable. Runtime injects `COGTOME_UNIT_MODE=1`, `COGTOME_EXECUTION_ID`, `COGTOME_TRACE_ID`.

## Discovery Path

The Runtime scans `$HOME/cogtome-demo/skills/` (configurable in main.rs:79). Unit/Motif/Structure are resolved by searching:
1. Global paths (`skills/units/`, `skills/motifs/`, `skills/structures/`)
2. Complex-private paths (each `skills/<complex>/units/` etc.)

## CLI Commands

```bash
cogtome discover              # Scan all Complexes
cogtome run <complex>          # Execute via Complex → Structure → Motif → Unit
cogtome unit run <name>       # Run Unit directly
cogtome motif run <name>      # Run Motif
cogtome structure run <name>   # Run Structure
```

## Key Implementation Notes

- **Variable resolution**: `${params.x}` for input, `${steps.name.output.field}` for step outputs, `${env.VAR}` for environment
- **YAML Motifs**: Currently only serial execution. Parallel/conditional flows are Phase 2
- **Process model**: Units are forked as independent processes via tokio::process::Command
- **Python Motif SDK**: Phase 2 - communicates via Unix Domain Socket IPC, not bare subprocess
