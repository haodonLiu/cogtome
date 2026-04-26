# Visual Block Editor Design

## Overview

Three new editor pages for COGTOME's WebUI:

1. **Unit Editor** — edit Unit metadata, test execution, AI chat assistant
2. **Motif Editor** — visual block editor with draggable Unit blocks and control flow containers
3. **Structure Editor** — visual block editor with Unit + Motif blocks, Motif blocks expandable

## COGTOME Layer Architecture

```
Structure (L3) ────► Motif (L2) ────► Unit (L1)
   blocks              blocks            atomic
 Units + Motifs       Units only        CLI executor
```

---

## 1. Unit Editor Page

**Route:** `/units/:name` (edit existing) | `/units/new`

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  [←] Unit Editor: file-read                         [Save] [×] │
├──────────────────────────────┬───────────────────────────────-─┤
│  CONFIGURATION               │  TEST PANEL                     │
│                              │                                 │
│  Name: [file-read        ]   │  Input JSON:                    │
│  Path: skills/units/...       │  ┌─────────────────────────┐   │
│                              │  │ { "path": "test.txt",    │   │
│  Timeout: ────●───── 30s     │  │   "offset": 0,           │   │
│  Concurrency: [1     ▼]       │  │   "limit": 100 }         │   │
│                              │  └─────────────────────────┘   │
│  Description:                │                                 │
│  ┌──────────────────────┐    │  [▶ Run Test]                  │
│  │ Reads file content   │    │                                 │
│  │ from specified path  │    │  Output:                        │
│  └──────────────────────┘    │  ┌─────────────────────────┐   │
│                              │  │ { "content": "...",      │   │
│                              │  │   "lines": 42 }           │   │
│                              │  └─────────────────────────┘   │
│                              │  ✓ Success (exit 0, 23ms)       │
├──────────────────────────────┴───────────────────────────────-─┤
│  🤖 Assistant                                        [−] [×]   │
│  ─────────────────────────────────────────────────────────────│
│  You: How do I add offset support?                            │
│  Bot: You can use the ${params.offset} variable to skip...    │
│  ─────────────────────────────────────────────────────────────│
│  [Ask the assistant about this Unit...]                [Send]  │
└─────────────────────────────────────────────────────────────────┘
```

### Sections

**Configuration Panel (left, ~300px)**
- Name field (read-only for existing Units)
- Path display (read-only)
- Timeout slider (1-300s, default 30)
- Concurrency dropdown (1-10, or "unlimited")
- Description textarea

**Test Panel (right, ~400px)**
- Input JSON textarea (with syntax highlighting via CodeMirror or textarea)
- "Run Test" button → calls `POST /run` with `{"type":"unit", "name":"...", "input":{...}}`
- Output display area (JSON, with status badge: success/error/exit code)
- Execution time display

**Chat Assistant (bottom, collapsible, ~300px default)**
- Message history (scrollable)
- Input field with Send button
- AI analyzes current Unit and context to answer questions
- Powered by Claude API

---

## 2. Motif Editor Page

**Route:** `/motifs/:name/edit`

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  [←] Motif: fetch-web                              [Validate]   │
│                                                     [Save] [×] │
├────────────┬─────────────────────────────────────┬──────────────┤
│ PALETTE    │  CANVAS (horizontal pipeline)       │ PROPERTIES   │
│            │                                     │              │
│ ▸ UNITS    │  [fetch-text]───────►[return]      │ Step: fetch  │
│   fetch    │                                     │              │
│   filter   │                                     │ Input:       │
│   map      │                                     │   url: ${    │
│ ▸ CONTROL  │                                     │     params   │
│   if       │                                     │     .url}    │
│   foreach  │                                     │              │
│ ▸ LOGIC    │                                     │              │
│   == != >  │                                     │              │
│   && ||    │                                     │              │
│            │                                     │              │
│            │  [+ Add Step]  [+ Add Branch]      │              │
├────────────┴─────────────────────────────────────┴──────────────┤
│  [YAML Preview]  [Block View] (toggle)                         │
└─────────────────────────────────────────────────────────────────┘
```

### Block Types

**Unit Block**
```
┌─────────────────────┐
│ ● fetch-text        │
│ input | output      │
└─────────────────────┘
```
- Draggable from palette to canvas
- Click to select → properties on right

**Control Flow Container (foreach)**
```
┌─ foreach ──────────────────┐
│  over: [expression    ]    │
│  max:  [50            ]    │
│  ──────────────────────────│
│  [Unit block here]         │
│  [Unit block here]         │
│  ──────────────────────────│
│  [+ Drop unit here]        │
└─────────────────────────────┘
```
- C-block shape (Scratch style)
- Expression input for `over` field
- max_iterations input
- Nested drop zone for unit blocks
- Collapsible

**Control Flow Container (if)**
```
┌─ if ───────────────────────┐
│  condition: [expression]    │
│  ──────────────────────────│
│  [Unit block here]          │
│  ──────────────────────────│
│  [+ Drop unit here]         │
└─────────────────────────────┘
┌─ else ─────────────────────┐
│  ──────────────────────────│
│  [Unit block here]          │
│  ──────────────────────────│
│  [+ Drop unit here]         │
└─────────────────────────────┘
```
- if/else pair
- Condition expression input
- Collapsible

**Return Block**
```
┌──── return ─────────────────┐
│  url: ${steps.fetch...}     │
│  content: ${steps...}       │
└─────────────────────────────┘
```
- Terminal block (no output port)
- Key-value pairs for return expressions

### Interactions

- **Drag from palette to canvas** → adds new block
- **Drag unit into foreach/if container** → nests inside
- **Click block** → selects it, shows properties
- **Delete key** → removes selected block
- **Canvas scroll** → horizontal scroll for wide motifs
- **Block collapse** → toggle to hide/show nested content
- **View toggle** → switch between Block View and YAML Preview

### Properties Panel

When a block is selected:
- **Unit block**: shows input field mappings (key → expression)
- **foreach block**: shows `over` expression, `max_iterations`, `parallel` toggle
- **if block**: shows `condition` expression
- **return block**: shows key-value editor

---

## 3. Structure Editor Page (Redesign)

**Route:** `/structures/:name` (existing) | `/structures/new`

### Layout

```
┌─────────────────────────────────────────────────────────────────┐
│  [←] Structure: fetch                               [Validate]  │
│                                                     [Save] [×]  │
├────────────┬─────────────────────────────────────┬──────────────┤
│ PALETTE    │  CANVAS (horizontal pipeline)       │ PROPERTIES   │
│            │                                     │              │
│ ▸ MOTIFS   │  [fetch-web ▼]─────►[return]       │ Motif:       │
│   fetch    │       │                              │   fetch-web  │
│   filter   │   (expanded)                        │              │
│   map      │  [fetch-text]─────►[return]         │ Input:       │
│ ▸ UNITS    │                                     │   url: ${...}│
│   fetch    │  [file-write]─────►                 │              │
│ ▸ CONTROL  │                                     │ Output:      │
│   if       │                                     │   content    │
│   foreach  │                                     │              │
│            │                                     │              │
│            │  [+ Add Motif] [+ Add Unit]         │              │
└────────────┴─────────────────────────────────────┴──────────────┘
```

### Block Types

**Motif Block**
```
┌─────────────────────┐
│ ◆ fetch-web    [▼]  │  ← ◆ = Motif indicator, ▼ = expandable
│ motifs | output     │
└─────────────────────┘
```
- Draggable from palette
- Click ▼ to expand/collapse inline Motif editor
- Expanded shows Motif's internal unit blocks
- Right panel shows Motif's input/output schema

**Unit Block**
```
┌─────────────────────┐
│ ● fetch-text        │  ← ● = Unit indicator
│ input | output      │
└─────────────────────┘
```
- Same as Motif editor Unit block

**Control Flow Containers** — same as Motif editor

### Key Differences from Motif Editor

1. Palette includes Motif blocks (not just Units)
2. Motif blocks are expandable inline (see internal flow)
3. Clicking expanded Motif shows its blocks + allows editing
4. Structure-level blocks (Motifs/Units) are top-level pipeline steps

---

## Technical Approach

### Libraries

- **React Flow** — canvas, nodes, edges, pan/zoom, drag-drop
- **@dnd-kit** — drag from palette to canvas (existing, keep)
- **Zustand** — state management (existing, extend)
- **CodeMirror 6** — JSON input/output editing in Unit editor
- **react-markdown** — chat messages rendering

### API Changes

**New Unit endpoints needed:**
```
GET  /api/units/:name           → read Unit metadata + content
PUT  /api/units/:name           → save Unit
POST /api/units/:name/test      → run test execution
```

**Modified endpoints:**
```
POST /run { type: "unit", name, input }  → already exists, use for test
```

### Data Flow

1. **Palette** → drag → **Canvas** → creates new block node
2. Block node → stores `{ id, type, config }`
3. Canvas → serialize to YAML → `PUT /api/motifs/:name` or `PUT /api/structures/:name`
4. Load → `GET /api/motifs/:name` → parse YAML → reconstruct nodes

### File Structure

```
webui/src/
├── components/
│   ├── editors/
│   │   ├── UnitEditor.tsx
│   │   ├── MotifEditor.tsx
│   │   ├── StructureEditor.tsx      (redesign)
│   │   ├── ChatAssistant.tsx
│   │   ├── TestPanel.tsx
│   │   └── blocks/
│   │       ├── UnitBlock.tsx
│   │       ├── ForeachBlock.tsx
│   │       ├── IfBlock.tsx
│   │       ├── ReturnBlock.tsx
│   │       ├── MotifBlock.tsx
│   │       └── BlockPalette.tsx
│   ├── canvas/
│   │   ├── PipelineCanvas.tsx
│   │   ├── CanvasNode.tsx
│   │   └── CanvasEdge.tsx
│   └── PropertyPanel.tsx
├── store/
│   ├── editorStore.ts               (Zustand, block/canvas state)
│   └── structureStore.ts            (existing)
├── api/
│   └── client.ts                    (add Unit CRUD)
└── types/
    └── index.ts                     (add BlockNode, BlockEdge types)
```

### Chat Assistant

- Uses Claude API (`/v1/messages`)
- System prompt includes: current Unit/Motif/Structure YAML, COGTOME documentation
- Streaming responses
- Persists in `localStorage` per session

---

## Implementation Order

1. **Unit Editor** — simplest, no canvas
2. **Block palette + canvas infrastructure** — React Flow integration
3. **Motif Editor** — Unit blocks + control flow containers
4. **Structure Editor redesign** — Motif blocks + expansion
5. **Chat Assistant** — Claude API integration
6. **Polish** — animations, keyboard shortcuts, undo/redo
