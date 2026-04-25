# Skill Authoring Guide

## File Layout

```
skills/
├── units/
│   └── <unit-name>/
│       └── bin/
│           └── <unit-name>          # Executable (exit 0=ok, 1=error, 2=retry, 3=dep unavailable)
├── motifs/
│   └── <motif-name>.yaml            # Filename MUST match motif name
└── structures/
    └── <structure-name>/             # Directory name MUST match structure name
        ├── manifest.yaml
        └── SKILL.md                 # Optional Complex-level docs

<complex>/
└── SKILL.md                         # Complex facade (L4)
```

**Critical naming rules:**

| Element | Rule | Error if violated |
|---------|------|-------------------|
| Structure directory | `structures/<name>/` where `name` matches `manifest.yaml`'s `name` | `Structure 'xxx' not found` |
| Motif file | `motifs/<name>.yaml` where `name` matches the `name` field | `Motif 'xxx' not found` |
| Unit directory | `units/<name>/bin/<name>` | Unit not found |

**Path resolution in SKILL.md:**

Paths in `structures[].path` are relative to SKILL.md's location, NOT the skills root.

```
# WRONG (if SKILL.md is at skills/web-fetch/SKILL.md)
structures:
  - path: ../structures/web-fetch

# CORRECT
structures:
  - path: ../structures/fetch      # matches directory name
```

---

## Structure Manifest

**File:** `structures/<name>/manifest.yaml`

```yaml
name: <structure-name>              # Required; must match parent directory name
description: What this structure does
type: structure

input_schema:                       # Required
  type: object
  properties:
    <param-name>:
      type: string|number|boolean|array|object
      description: Parameter description
      default: <default-value>      # Optional
  required: [<param-name>, ...]     # Optional

output_schema:                     # Required
  type: object
  properties:
    <field-name>:
      type: <type>
      description: Output field description

units_required:                     # Required (can be empty list)
  - <unit-name-1>
  - <unit-name-2>

steps:                              # Required
  - name: <step-name>               # Unique within this structure
    unit: <unit-name>
    input:                          # Static or dynamic via ${params.x}
      <field>: <value>
    condition: <expression>        # Optional: skip if false
```

**Example:**

```yaml
name: fetch
description: Fetch and parse web content
type: structure

input_schema:
  type: object
  properties:
    url:
      type: string
      description: URL to fetch
    selector:
      type: string
      description: CSS selector for content extraction
      default: body
  required: [url]

output_schema:
  type: object
  properties:
    content:
      type: string
      description: Extracted text content
    status:
      type: number
      description: HTTP status code

units_required:
  - curl-unit
  - jq-unit

steps:
  - name: fetch_page
    unit: curl-unit
    input:
      url: "${params.url}"
      timeout: 10

  - name: extract_content
    unit: jq-unit
    input:
      json: "${steps.fetch_page.output.content}"
      query: "${params.selector}"
```

---

## Motif Manifest

**File:** `motifs/<name>.yaml`

**Filename MUST match `name` field exactly.**

```yaml
name: <motif-name>                  # Required; must match filename (without .yaml)
description: What this motif orchestrates
type: motif

input_schema:                       # Optional
  type: object
  properties:
    <param-name>:
      type: <type>
  required: [...]

output_schema:                      # Optional
  type: object

flow:                               # Required; array of steps
  - name: <step-name>               # Unique within this motif
    structure: <structure-name>     # Must exist in skills dir
    input:                          # Static or ${params.x} or ${steps.s.output.field}
      <field>: <value>
    condition: <expression>        # Optional: skip if false
    on_error: fail_fast|continue   # Default: fail_fast

  - name: <next-step>
    structure: <another-structure>
    input:
      data: "${steps.previous.output.field}"

# foreach:                          # NOT YET IMPLEMENTED (Phase 2)
#   over: "${params.items}"
#   as_var: "item"
#   parallel: false
#   aggregate:
#     mode: array
```

**Example (serial):**

```yaml
name: fetch-web
description: Fetch URL and extract content
type: motif

input_schema:
  type: object
  properties:
    url:
      type: string
  required: [url]

output_schema:
  type: object
  properties:
    result:
      type: string

flow:
  - name: fetch
    structure: fetch
    input:
      url: "${params.url}"

  - name: parse
    structure: parse
    input:
      html: "${steps.fetch.output.content}"
```

---

## Variable Resolution

| Syntax | Meaning |
|--------|---------|
| `${params.x}` | User input parameter |
| `${steps.name.output.field}` | Output from a previous step |
| `${env.VAR}` | Environment variable |
| `${arr[0]}` | Array index (0-based) |
| `${arr[-1]}` | Array index (negative = from end) |
| `${arr.length}` | Array length |

**Expression functions:**

- `filter(arr, 'field == "value"')` — filter array
- `map(arr, 'field')` — extract field from array

---

## Common Errors

### "Structure 'xxx' not found"

**Cause:** Directory name mismatch.

```
structures/web-fetch/manifest.yaml  ← wrong
structures/fetch/manifest.yaml      ← correct
```

The runtime looks for `structures/<name>/manifest.yaml` where `<name>` is what you specified.

**Fix:** Ensure directory name matches the structure's `name` field in manifest.

---

### "Motif 'xxx' not found"

**Cause:** Filename mismatch.

```
motifs/fetch-web.yaml + name: fetch-web  ← wrong
motifs/fetch-web.yaml + name: fetch     ← wrong
motifs/fetch-web.yaml + name: fetch-web # must match filename without .yaml
```

**Fix:** The motif filename (without `.yaml`) must exactly match the `name` field.

---

### "missing field `type`"

**Cause:** Manifest missing required `type` field.

**Fix:** Add `type: structure` or `type: motif` to your manifest.

---

### "missing field `units_required`"

**Cause:** Structure manifest missing `units_required`.

**Fix:** Add `units_required: []` (empty list if no units needed).

---

### "jq: command not found"

**Cause:** System dependency `jq` not installed.

**Fix:** Install jq:

```bash
# Ubuntu/Debian
sudo apt install jq

# macOS
brew install jq
```

---

### Path resolution in SKILL.md

**Cause:** Paths in `structures[].path` are relative to SKILL.md location.

If SKILL.md is at `skills/web-fetch/SKILL.md`:

```yaml
# WRONG
structures:
  - path: ../structures/web-fetch

# CORRECT
structures:
  - path: ../structures/fetch     # matches directory name, not path
```

---

## System Dependencies

| Tool | Purpose | Installation |
|------|---------|--------------|
| `jq` | JSON parsing in units | `apt install jq` / `brew install jq` |

Units communicate via stdin/stdout JSON. Any tool that can read JSON input and emit JSON output works as a unit.

---

## SKILL.md Template

**File:** `<complex>/SKILL.md` (for Complex-level skills)

```yaml
---
name: <complex-name>
description: <what this complex does>
structures:
  - path: ../structures/<structure-name>   # relative to SKILL.md location
  - path: ../structures/<another-structure>
units:
  - path: ../units/<unit-name>
---

# <Complex Name>

Optional markdown documentation for the Complex layer.
```

**Remember:** `path` values are relative to SKILL.md's directory.
