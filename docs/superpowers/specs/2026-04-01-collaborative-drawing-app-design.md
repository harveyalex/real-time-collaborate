# Collaborative Drawing App (Excalidraw Clone) — Design Spec

## Overview

A proof-of-concept Excalidraw clone built with a pure Rust stack: SpacetimeDB for real-time state sync, Leptos for the browser UI, a custom WASM-compatible SpacetimeDB client, and a Rust service worker for offline support. Includes vim-style keybindings for keyboard-driven drawing.

**Goal:** Demonstrate that SpacetimeDB + Leptos + Rust WASM can power a collaborative drawing application with real-time sync, offline support, and a vim-inspired interaction model.

**Non-goals:** Production polish, full Excalidraw feature parity, CRDT-based conflict resolution, authentication beyond SpacetimeDB Identity.

## Feature Scope

- Drawing primitives: rectangle, ellipse, arrow, line, text, freehand
- Selection, move, resize with handles
- Undo/redo (per-user, server-side stack)
- Styling: stroke color, fill color, stroke width, opacity
- Freehand drawing
- Export: PNG and SVG
- Clipboard: copy/paste elements
- Real-time collaboration: live element sync, cursor presence
- Conflict resolution: last-write-wins (inherent in SpacetimeDB's serialized reducers)
- Vim keybindings: modal interaction (Normal, Insert, Visual, Command)

## Architecture

### System Overview

```
Browser (WASM)                          SpacetimeDB Server
┌─────────────────────┐                 ┌─────────────────────┐
│  Leptos App          │                │  Server Module       │
│  (UI, Canvas, Vim)   │                │  (Rust → WASM)       │
│         ↕            │   WebSocket    │  Tables, Reducers,   │
│  stdb-client         │◄──────────────►│  Subscriptions       │
│  (gloo-net + BSATN)  │   (BSATN)     │         ↕            │
│         ↕            │                │  SQLite Storage      │
│  Service Worker      │                └─────────────────────┘
│  (offline + sync)    │
└─────────────────────┘
```

### Cargo Workspace

```
real-time-collaborate/
├── Cargo.toml                 # workspace root
├── crates/
│   ├── shared/                # types, BSATN codecs, shared logic
│   │   └── src/lib.rs         # Element, Room, Cursor, enums
│   ├── server/                # SpacetimeDB server module
│   │   └── src/lib.rs         # tables, reducers
│   ├── stdb-client/           # WASM-compatible SpacetimeDB client
│   │   └── src/
│   │       ├── lib.rs         # public API: connect(), subscribe(), call_reducer()
│   │       ├── ws.rs          # gloo-net WebSocket wrapper
│   │       ├── protocol.rs    # SpacetimeDB wire protocol messages
│   │       ├── codec.rs       # BSATN serialization (re-exports spacetimedb-lib)
│   │       ├── cache.rs       # client-side row cache (HashMap per table)
│   │       └── signals.rs     # Leptos signal integration layer
│   ├── app/                   # Leptos frontend
│   │   └── src/
│   │       ├── main.rs
│   │       ├── canvas.rs      # HTML5 Canvas rendering
│   │       ├── vim.rs         # Vim mode state machine
│   │       ├── tools.rs       # Drawing tools (rect, arrow, etc.)
│   │       └── state.rs       # App state, undo/redo
│   └── service-worker/        # Rust WASM service worker
│       └── src/lib.rs         # offline caching, background sync
```

### Key Architectural Decisions

- **HTML5 Canvas** (not WebGL/wgpu) — sufficient for 2D drawing PoC, simpler API
- **Shared crate** — types compiled into both server module and client, zero deserialization mismatch
- **BSATN protocol** — `spacetimedb-lib` provides the codec; we wrap it for WASM WebSocket transport
- **Leptos CSR only** — no SSR needed for a drawing app
- **Trunk** for building the Leptos app, **wasm-pack** for the service worker
- **Pure Rust** — no JavaScript in application code; the stdb-client implements the SpacetimeDB WebSocket protocol directly

## Data Model

### Tables

#### Room
| Field | Type | Notes |
|-------|------|-------|
| id | u64 | PK, auto_inc |
| name | String | |
| created_by | Identity | |
| created_at | u64 | |

#### Element
| Field | Type | Notes |
|-------|------|-------|
| id | u64 | PK, auto_inc |
| room_id | u64 | indexed |
| kind | ElementKind | enum u8 |
| x | f64 | |
| y | f64 | |
| width | f64 | |
| height | f64 | |
| rotation | f64 | |
| points | Vec\<u8\> | encoded point data (freehand paths, arrow waypoints) |
| stroke_color | u32 | RGBA packed |
| fill_color | u32 | RGBA packed |
| stroke_width | f32 | |
| opacity | f32 | |
| font_size | f32 | |
| text_content | String | |
| z_index | i32 | |
| version | u64 | for optimistic concurrency |
| updated_by | Identity | |
| deleted | bool | soft delete for undo |

#### Cursor
| Field | Type | Notes |
|-------|------|-------|
| user_id | Identity | PK |
| room_id | u64 | |
| x | f64 | |
| y | f64 | |
| name | String | |
| color | u32 | |

#### UndoEntry
| Field | Type | Notes |
|-------|------|-------|
| id | u64 | PK, auto_inc |
| room_id | u64 | |
| user_id | Identity | |
| action | UndoAction | enum |
| element_id | u64 | |
| prev_state | Vec\<u8\> | BSATN-encoded previous Element |
| timestamp | u64 | |

### Enums

```rust
enum ElementKind {
    Rectangle = 0,
    Ellipse = 1,
    Arrow = 2,
    Line = 3,
    Text = 4,
    Freehand = 5,
}

enum UndoAction {
    Create = 0,  // undo → soft delete
    Update = 1,  // undo → restore prev_state
    Delete = 2,  // undo → undelete
}
```

### Reducers

```
create_room(name) → Room
join_room(room_id) → sets cursor
create_element(room_id, kind, x, y, w, h, ...) → Element + UndoEntry
update_element(id, ..fields) → Element + UndoEntry
delete_element(id) → soft delete + UndoEntry
move_element(id, dx, dy) → update position + UndoEntry
resize_element(id, w, h) → update size + UndoEntry
update_cursor(room_id, x, y) → Cursor
undo(room_id) → pop user's last UndoEntry, apply inverse
redo(room_id) → re-apply last undone entry
```

### Subscriptions

Clients subscribe on room join:
```sql
SELECT * FROM elements WHERE room_id = :id AND deleted = false
SELECT * FROM cursors WHERE room_id = :id
SELECT * FROM undo_entries WHERE room_id = :id AND user_id = :me
```

SpacetimeDB pushes row-level diffs automatically. The client cache updates Leptos signals on each delta.

## Canvas Rendering

### Render Pipeline

1. SpacetimeDB delta arrives → update Leptos signal (`RwSignal<ElementStore>`)
2. Signal change triggers `create_effect` → `request_animation_frame`
3. On each frame:
   - Clear canvas
   - Apply camera transform (pan + zoom)
   - Sort elements by z_index
   - For each visible element: render based on kind (stroke_rect, ellipse, bezier curves, fill_text, line_to paths)
   - Draw selection handles (if any selected)
   - Draw other users' cursors + names
   - Draw vim mode indicator + command buffer

### UI Layout

- **Top bar:** App name, room info, user count, vim mode indicator badge
- **Canvas:** Full-bleed, handles all mouse/keyboard events
- **Bottom bar:** Vim command line (`:` prefix), zoom level, cursor position

## Vim Mode

### State Machine

Four modes with transitions:

**NORMAL** (default)
- `h/j/k/l` — move selected element (10px steps, shift = 1px)
- `r` — enter INSERT: rectangle creation
- `e` — enter INSERT: ellipse creation
- `a` — enter INSERT: arrow creation
- `L` (shift+l) — enter INSERT: line creation
- `t` — enter INSERT: text creation
- `f` — enter INSERT: freehand mode
- `dd` — delete selected
- `yy` — copy selected
- `p` — paste
- `u` — undo
- `Ctrl+r` — redo
- `/` — enter COMMAND mode
- `v` — enter VISUAL mode
- `Esc` — deselect all

**INSERT** (drawing)
- Shapes (r/e): click+drag to size → creates element → back to NORMAL
- Arrow/Line (a/L): click start → click end → back to NORMAL
- Text (t): click position → type text → Esc commits and returns to NORMAL
- Freehand (f): click+drag to draw → release commits stroke, stays in INSERT for more strokes → Esc returns to NORMAL

**VISUAL** (multi-select)
- `h/j/k/l` — expand selection box
- Click elements to toggle selection
- `d` — delete all selected
- `y` — copy all selected
- `Esc` — back to NORMAL

**COMMAND** (`:` prefix)
- `:w` — export PNG
- `:ws` — export SVG
- `:wq` — export and close
- `:color #hex` — set stroke color
- `:fill #hex` — set fill color
- `:stroke N` — set stroke width
- `Esc` — back to NORMAL

## stdb-client Crate

### Purpose

A minimal, WASM-compatible SpacetimeDB client replacing the official Rust SDK (which depends on tokio and cannot compile to wasm32-unknown-unknown).

### Modules

- **ws.rs** — gloo-net WebSocket wrapper with reconnection logic
- **protocol.rs** — SpacetimeDB wire protocol message types (Subscribe, CallReducer, InitialSubscription, TransactionUpdate, IdentityToken, SubscriptionError)
- **codec.rs** — BSATN serialization via spacetimedb-lib re-exports
- **cache.rs** — client-side row cache (HashMap per table), emits insert/update/delete callbacks
- **signals.rs** — Leptos signal integration: cache changes trigger RwSignal updates

### Dependencies

- `gloo-net` — WASM WebSocket
- `spacetimedb-lib` — BSATN codec + types
- `spacetimedb-sats` — algebraic type system
- `serde` — struct serialization
- `wasm-bindgen-futures` — async bridge
- `leptos` — signal types (feature-gated)

### Key Risk

The BSATN wire protocol is not formally documented as a public API. We depend on spacetimedb-lib for encoding/decoding, but message framing must be reverse-engineered from the TypeScript SDK or Rust SDK source.

**Mitigation:** Pin to a specific SpacetimeDB version. The protocol has been stable since 1.0.

## Service Worker

### Responsibilities

- **App shell caching:** Cache HTML, WASM, CSS on install. Cache-first for static assets, network-first for API/WS.
- **Operation queue:** When offline, queue reducer calls in IndexedDB. On reconnect, replay via Background Sync API.
- **Canvas snapshot:** Periodically cache last-known element state to IndexedDB. On offline load, render from cached snapshot.
- **Communication:** postMessage with main thread via web-sys. Reports online/offline status, queue depth, sync progress.

### Build

Separate WASM binary built with `wasm-pack --target no-modules`. Thin JS loader (`sw-loader.js`) registers and instantiates the WASM module.

## Error Handling

- **Connection loss:** Status bar shows OFFLINE. Drawing continues locally with ops queued. Exponential backoff reconnect (1s → 30s max). On reconnect: replay queue, re-subscribe, reconcile state.
- **Reducer errors:** Rollback optimistic local change. Toast notification. No auto-retry.
- **WASM panics:** `console_error_panic_hook` for stack traces. Corrupted element data skipped in render with warning logged.
- **Version mismatch:** BSATN decode failure triggers full page reload. Service worker updates cached assets on activate.

## Testing Strategy

### Unit Tests (cargo test)
- **shared:** BSATN round-trip encoding, element type conversions, point data encode/decode
- **app:** Vim state machine transitions, undo/redo stack logic, hit testing (point-in-shape), selection box intersection
- **stdb-client:** Protocol message parsing, row cache insert/update/delete

### Integration Tests
- **server:** Spin up local SpacetimeDB, call reducers, verify table state, test undo/redo sequences, test concurrent updates, verify subscription filtering

### WASM / E2E Tests (stretch goal)
- **wasm-bindgen-test:** stdb-client WebSocket connect, signal update propagation
- **Playwright:** Two-tab sync test, vim keybindings, export produces valid PNG

## Export & Clipboard

### PNG Export (`:w`)
1. Create offscreen canvas sized to bounding box + padding
2. Render all elements (same pipeline)
3. `canvas.to_blob("image/png")` via web-sys
4. Download via `URL.createObjectURL`

### SVG Export (`:ws`)
1. Generate SVG markup per element (rect → `<rect>`, ellipse → `<ellipse>`, freehand → `<path>`)
2. Wrap in `<svg>` with viewBox
3. Blob + download

### Clipboard (`yy` / `p`)
- **Copy:** Serialize selected elements to JSON, write to `navigator.clipboard` + local signal
- **Paste:** Read from local signal (fast path) or clipboard API (cross-tab), deserialize, call create_element reducers, offset to avoid overlap

## Build Pipeline

```bash
# 1. Build SpacetimeDB server module
spacetime build crates/server

# 2. Build Leptos app (includes stdb-client)
trunk build crates/app/index.html --release

# 3. Build service worker
wasm-pack build crates/service-worker --target no-modules --out-dir ../../dist/sw

# 4. Publish server module
spacetime publish collaborate crates/server

# 5. Serve dist/ (or trunk serve for dev)
```
