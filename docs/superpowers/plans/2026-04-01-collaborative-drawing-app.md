# Collaborative Drawing App Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a proof-of-concept Excalidraw clone with real-time collaboration, vim keybindings, and offline support using a pure Rust stack.

**Architecture:** Cargo workspace with 5 crates — `shared` (types), `server` (SpacetimeDB module), `stdb-client` (WASM-compatible SpacetimeDB client wrapping the official SDK with `browser` feature), `app` (Leptos CSR frontend), and `service-worker` (Rust WASM). HTML5 Canvas for rendering, SpacetimeDB for state sync.

**Tech Stack:** Rust, SpacetimeDB 2.x, Leptos 0.7 (CSR), Trunk, wasm-pack, web-sys, gloo-net

**Spec:** `docs/superpowers/specs/2026-04-01-collaborative-drawing-app-design.md`

---

## Phase 1: Project Scaffolding & Shared Types

### Task 1.1: Initialize Cargo Workspace

**Files:**
- Create: `Cargo.toml` (workspace root)
- Create: `crates/shared/Cargo.toml`
- Create: `crates/shared/src/lib.rs`
- Create: `crates/server/Cargo.toml`
- Create: `crates/server/src/lib.rs`
- Create: `crates/app/Cargo.toml`
- Create: `crates/app/src/main.rs`
- Create: `crates/app/index.html`
- Create: `crates/stdb-client/Cargo.toml`
- Create: `crates/stdb-client/src/lib.rs`
- Create: `crates/service-worker/Cargo.toml`
- Create: `crates/service-worker/src/lib.rs`
- Create: `.gitignore`
- Create: `rust-toolchain.toml`

- [ ] **Step 1: Create workspace root Cargo.toml**

```toml
[workspace]
resolver = "2"
members = [
    "crates/shared",
    "crates/server",
    "crates/stdb-client",
    "crates/app",
    "crates/service-worker",
]

[workspace.dependencies]
shared = { path = "crates/shared" }
spacetimedb = "2.0"
spacetimedb-lib = { version = "2.0", default-features = false }
spacetimedb-sdk = { version = "2.0", default-features = false }
leptos = { version = "0.7", features = ["csr"] }
web-sys = "0.3"
wasm-bindgen = "0.2"
js-sys = "0.3"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
log = "0.4"
```

- [ ] **Step 2: Create .gitignore and rust-toolchain.toml**

`.gitignore`:
```
/target
/dist
*.wasm
.superpowers/
```

`rust-toolchain.toml`:
```toml
[toolchain]
channel = "stable"
targets = ["wasm32-unknown-unknown"]
```

- [ ] **Step 3: Create shared crate with stub**

`crates/shared/Cargo.toml`:
```toml
[package]
name = "shared"
version = "0.1.0"
edition = "2021"

[dependencies]
spacetimedb-lib = { workspace = true, features = ["serde"] }
serde = { workspace = true }
```

`crates/shared/src/lib.rs`:
```rust
pub mod types;
```

Create `crates/shared/src/types.rs` as empty file for now.

- [ ] **Step 4: Create server crate with stub**

`crates/server/Cargo.toml`:
```toml
[package]
name = "server"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
spacetimedb = { workspace = true }
shared = { workspace = true }
log = { workspace = true }
```

`crates/server/src/lib.rs`:
```rust
use spacetimedb::ReducerContext;
```

- [ ] **Step 5: Create stdb-client crate with stub**

`crates/stdb-client/Cargo.toml`:
```toml
[package]
name = "stdb-client"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { workspace = true }
spacetimedb-sdk = { workspace = true, features = ["browser"] }
spacetimedb-lib = { workspace = true }
leptos = { workspace = true }
web-sys = { workspace = true }
wasm-bindgen = { workspace = true }
wasm-bindgen-futures = "0.4"
log = { workspace = true }
```

`crates/stdb-client/src/lib.rs`:
```rust
pub mod connection;
```

Create `crates/stdb-client/src/connection.rs` as empty file.

- [ ] **Step 6: Create app crate with stub**

`crates/app/Cargo.toml`:
```toml
[package]
name = "app"
version = "0.1.0"
edition = "2021"

[dependencies]
shared = { workspace = true }
stdb-client = { path = "../stdb-client" }
leptos = { workspace = true }
web-sys = { workspace = true, features = [
    "HtmlCanvasElement",
    "CanvasRenderingContext2d",
    "MouseEvent",
    "KeyboardEvent",
    "WheelEvent",
    "Window",
    "Document",
    "DomRect",
] }
wasm-bindgen = { workspace = true }
js-sys = { workspace = true }
console_error_panic_hook = "0.1"
log = { workspace = true }
wasm-logger = "0.2"
```

`crates/app/src/main.rs`:
```rust
use leptos::prelude::*;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <h1>"Collaborate"</h1>
    }
}
```

`crates/app/index.html`:
```html
<!DOCTYPE html>
<html>
  <head>
    <meta charset="utf-8" />
    <title>Collaborate</title>
    <link data-trunk rel="rust" data-wasm-opt="z" />
    <style>
      * { margin: 0; padding: 0; box-sizing: border-box; }
      body { background: #1a1a2e; color: #eee; font-family: monospace; overflow: hidden; }
    </style>
  </head>
  <body></body>
</html>
```

- [ ] **Step 7: Create service-worker crate with stub**

`crates/service-worker/Cargo.toml`:
```toml
[package]
name = "service-worker"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
shared = { workspace = true }
wasm-bindgen = { workspace = true }
web-sys = { workspace = true, features = [
    "ServiceWorkerGlobalScope",
    "Cache",
    "CacheStorage",
    "Request",
    "Response",
    "FetchEvent",
] }
js-sys = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

`crates/service-worker/src/lib.rs`:
```rust
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() {
    // Service worker entry point
}
```

- [ ] **Step 8: Verify workspace compiles**

Run: `cargo check --workspace`
Expected: Successful compilation (warnings OK, no errors)

- [ ] **Step 9: Commit**

```bash
git add -A
git commit -m "feat: initialize cargo workspace with 5 crates"
```

---

### Task 1.2: Define Shared Types

**Files:**
- Create: `crates/shared/src/types.rs`
- Create: `crates/shared/src/point.rs`
- Test: `crates/shared/src/types.rs` (inline tests)

- [ ] **Step 1: Write tests for ElementKind and UndoAction serialization**

Add to `crates/shared/src/types.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use spacetimedb_lib::bsatn;

    #[test]
    fn element_kind_round_trip() {
        let kinds = [
            ElementKind::Rectangle,
            ElementKind::Ellipse,
            ElementKind::Arrow,
            ElementKind::Line,
            ElementKind::Text,
            ElementKind::Freehand,
        ];
        for kind in kinds {
            let encoded = bsatn::to_vec(&kind).unwrap();
            let decoded: ElementKind = bsatn::from_slice(&encoded).unwrap();
            assert_eq!(kind, decoded);
        }
    }

    #[test]
    fn undo_action_round_trip() {
        let actions = [
            UndoAction::Create,
            UndoAction::Update,
            UndoAction::Delete,
        ];
        for action in actions {
            let encoded = bsatn::to_vec(&action).unwrap();
            let decoded: UndoAction = bsatn::from_slice(&encoded).unwrap();
            assert_eq!(action, decoded);
        }
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p shared`
Expected: FAIL — types not defined yet

- [ ] **Step 3: Implement shared types**

`crates/shared/src/types.rs`:
```rust
use spacetimedb_lib::SpacetimeType;
use serde::{Serialize, Deserialize};

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElementKind {
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Text,
    Freehand,
}

#[derive(SpacetimeType, Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum UndoAction {
    Create,
    Update,
    Delete,
}

/// Default styling for new elements.
pub const DEFAULT_STROKE_COLOR: u32 = 0xFFFFFFFF; // white
pub const DEFAULT_FILL_COLOR: u32 = 0x00000000;   // transparent
pub const DEFAULT_STROKE_WIDTH: f32 = 2.0;
pub const DEFAULT_OPACITY: f32 = 1.0;
pub const DEFAULT_FONT_SIZE: f32 = 20.0;
```

- [ ] **Step 4: Implement point encoding module**

`crates/shared/src/point.rs`:
```rust
use serde::{Serialize, Deserialize};

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct Point {
    pub x: f64,
    pub y: f64,
}

/// Encode a list of points into a compact byte representation.
/// Format: [count: u32 LE] [x0: f64 LE] [y0: f64 LE] [x1: f64 LE] [y1: f64 LE] ...
pub fn encode_points(points: &[Point]) -> Vec<u8> {
    let mut buf = Vec::with_capacity(4 + points.len() * 16);
    buf.extend_from_slice(&(points.len() as u32).to_le_bytes());
    for p in points {
        buf.extend_from_slice(&p.x.to_le_bytes());
        buf.extend_from_slice(&p.y.to_le_bytes());
    }
    buf
}

/// Decode points from the compact byte representation.
pub fn decode_points(data: &[u8]) -> Result<Vec<Point>, &'static str> {
    if data.len() < 4 {
        return Err("data too short for point count");
    }
    let count = u32::from_le_bytes(data[0..4].try_into().unwrap()) as usize;
    let expected_len = 4 + count * 16;
    if data.len() < expected_len {
        return Err("data too short for declared point count");
    }
    let mut points = Vec::with_capacity(count);
    for i in 0..count {
        let offset = 4 + i * 16;
        let x = f64::from_le_bytes(data[offset..offset + 8].try_into().unwrap());
        let y = f64::from_le_bytes(data[offset + 8..offset + 16].try_into().unwrap());
        points.push(Point { x, y });
    }
    Ok(points)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_decode_empty() {
        let points: Vec<Point> = vec![];
        let encoded = encode_points(&points);
        let decoded = decode_points(&encoded).unwrap();
        assert_eq!(decoded, points);
    }

    #[test]
    fn encode_decode_multiple_points() {
        let points = vec![
            Point { x: 1.5, y: 2.5 },
            Point { x: -3.0, y: 100.0 },
            Point { x: 0.0, y: 0.0 },
        ];
        let encoded = encode_points(&points);
        let decoded = decode_points(&encoded).unwrap();
        assert_eq!(decoded, points);
    }

    #[test]
    fn decode_too_short_fails() {
        assert!(decode_points(&[]).is_err());
        assert!(decode_points(&[1, 0, 0, 0]).is_err()); // claims 1 point but no data
    }
}
```

- [ ] **Step 5: Update lib.rs**

`crates/shared/src/lib.rs`:
```rust
pub mod types;
pub mod point;

pub use types::*;
pub use point::{Point, encode_points, decode_points};
```

- [ ] **Step 6: Run tests to verify they pass**

Run: `cargo test -p shared`
Expected: All tests pass

- [ ] **Step 7: Commit**

```bash
git add crates/shared/
git commit -m "feat: define shared types — ElementKind, UndoAction, Point encoding"
```

---

## Phase 2: SpacetimeDB Server Module

### Task 2.1: Define Tables

**Files:**
- Create: `crates/server/src/tables.rs`
- Modify: `crates/server/src/lib.rs`

- [ ] **Step 1: Define all 4 tables**

`crates/server/src/tables.rs`:
```rust
use spacetimedb::{Identity, Timestamp, SpacetimeType};
use shared::types::{ElementKind, UndoAction};

#[spacetimedb::table(accessor = room, public)]
pub struct Room {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub name: String,
    pub created_by: Identity,
    pub created_at: Timestamp,
}

#[spacetimedb::table(accessor = element, public)]
pub struct Element {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub room_id: u64,
    pub kind: ElementKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub points: Vec<u8>,
    pub stroke_color: u32,
    pub fill_color: u32,
    pub stroke_width: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub text_content: String,
    pub z_index: i32,
    pub version: u64,
    pub updated_by: Identity,
    pub deleted: bool,
}

#[spacetimedb::table(accessor = cursor, public)]
pub struct Cursor {
    #[primary_key]
    pub user_id: Identity,
    pub room_id: u64,
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub color: u32,
}

#[spacetimedb::table(accessor = undo_entry, public)]
pub struct UndoEntry {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub room_id: u64,
    pub user_id: Identity,
    pub action: UndoAction,
    pub element_id: u64,
    pub prev_state: Vec<u8>,
    pub next_state: Vec<u8>,
    pub timestamp: Timestamp,
    pub undone: bool,
}
```

- [ ] **Step 2: Update server lib.rs**

`crates/server/src/lib.rs`:
```rust
mod tables;

use spacetimedb::{ReducerContext, Table, Identity, Timestamp};
use tables::*;
use shared::types::*;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p server`
Expected: Compiles (may need to adjust SpacetimeType derives based on actual spacetimedb version)

- [ ] **Step 4: Commit**

```bash
git add crates/server/
git commit -m "feat: define SpacetimeDB tables — Room, Element, Cursor, UndoEntry"
```

---

### Task 2.2: Implement Reducers

**Files:**
- Create: `crates/server/src/reducers.rs`
- Modify: `crates/server/src/lib.rs`

- [ ] **Step 1: Implement room and lifecycle reducers**

`crates/server/src/reducers.rs`:
```rust
use spacetimedb::{ReducerContext, Table};
use crate::tables::*;
use shared::types::*;

#[spacetimedb::reducer]
pub fn create_room(ctx: &ReducerContext, name: String) -> Result<(), String> {
    if name.is_empty() {
        return Err("Room name cannot be empty".into());
    }
    ctx.db.room().insert(Room {
        id: 0,
        name,
        created_by: ctx.sender,
        created_at: ctx.timestamp,
    });
    Ok(())
}

#[spacetimedb::reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    log::info!("Client connected: {:?}", ctx.sender);
}

#[spacetimedb::reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    // Remove cursor on disconnect
    if ctx.db.cursor().user_id().find(ctx.sender).is_some() {
        ctx.db.cursor().user_id().delete(&ctx.sender);
    }
    log::info!("Client disconnected: {:?}", ctx.sender);
}

#[spacetimedb::reducer]
pub fn join_room(ctx: &ReducerContext, room_id: u64, name: String) -> Result<(), String> {
    if ctx.db.room().id().find(room_id).is_none() {
        return Err("Room not found".into());
    }
    // Assign a color based on identity bytes
    let color = cursor_color_from_identity(ctx.sender);
    // Upsert cursor
    if let Some(existing) = ctx.db.cursor().user_id().find(ctx.sender) {
        ctx.db.cursor().user_id().update(Cursor {
            room_id,
            name,
            color,
            x: 0.0,
            y: 0.0,
            ..existing
        });
    } else {
        ctx.db.cursor().insert(Cursor {
            user_id: ctx.sender,
            room_id,
            x: 0.0,
            y: 0.0,
            name,
            color,
        });
    }
    Ok(())
}

fn cursor_color_from_identity(id: spacetimedb::Identity) -> u32 {
    let bytes = id.to_byte_array();
    let r = bytes[0];
    let g = bytes[1];
    let b = bytes[2];
    u32::from_be_bytes([r, g, b, 0xFF])
}
```

- [ ] **Step 2: Implement element CRUD reducers**

Append to `crates/server/src/reducers.rs`:
```rust
#[spacetimedb::reducer]
pub fn create_element(
    ctx: &ReducerContext,
    room_id: u64,
    kind: ElementKind,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    stroke_color: u32,
    fill_color: u32,
    stroke_width: f32,
    opacity: f32,
    font_size: f32,
    text_content: String,
    points: Vec<u8>,
) -> Result<(), String> {
    if ctx.db.room().id().find(room_id).is_none() {
        return Err("Room not found".into());
    }
    let next_z = next_z_index(ctx, room_id);
    let elem = ctx.db.element().insert(Element {
        id: 0,
        room_id,
        kind,
        x,
        y,
        width,
        height,
        rotation: 0.0,
        points,
        stroke_color,
        fill_color,
        stroke_width,
        opacity,
        font_size,
        text_content,
        z_index: next_z,
        version: 1,
        updated_by: ctx.sender,
        deleted: false,
    });
    // Record undo entry
    ctx.db.undo_entry().insert(UndoEntry {
        id: 0,
        room_id,
        user_id: ctx.sender,
        action: UndoAction::Create,
        element_id: elem.id,
        prev_state: vec![],
        next_state: vec![],
        timestamp: ctx.timestamp,
        undone: false,
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn update_element(
    ctx: &ReducerContext,
    element_id: u64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    stroke_color: u32,
    fill_color: u32,
    stroke_width: f32,
    opacity: f32,
    font_size: f32,
    text_content: String,
    points: Vec<u8>,
) -> Result<(), String> {
    let Some(elem) = ctx.db.element().id().find(element_id) else {
        return Err("Element not found".into());
    };
    // Serialize previous state for undo
    let prev_state = serialize_element(&elem);
    // Build the next state snapshot for redo
    let next_snapshot = ElementSnapshot {
        x, y, width, height, rotation,
        stroke_color, fill_color, stroke_width, opacity, font_size,
        text_content: text_content.clone(), points: points.clone(),
    };
    let next_state = serde_json::to_vec(&next_snapshot).unwrap_or_default();
    ctx.db.undo_entry().insert(UndoEntry {
        id: 0,
        room_id: elem.room_id,
        user_id: ctx.sender,
        action: UndoAction::Update,
        element_id,
        prev_state,
        next_state,
        timestamp: ctx.timestamp,
        undone: false,
    });
    ctx.db.element().id().update(Element {
        x,
        y,
        width,
        height,
        rotation,
        stroke_color,
        fill_color,
        stroke_width,
        opacity,
        font_size,
        text_content,
        points,
        version: elem.version + 1,
        updated_by: ctx.sender,
        ..elem
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn delete_element(ctx: &ReducerContext, element_id: u64) -> Result<(), String> {
    let Some(elem) = ctx.db.element().id().find(element_id) else {
        return Err("Element not found".into());
    };
    let prev_state = serialize_element(&elem);
    ctx.db.undo_entry().insert(UndoEntry {
        id: 0,
        room_id: elem.room_id,
        user_id: ctx.sender,
        action: UndoAction::Delete,
        element_id,
        prev_state,
        next_state: vec![],
        timestamp: ctx.timestamp,
        undone: false,
    });
    ctx.db.element().id().update(Element {
        deleted: true,
        version: elem.version + 1,
        updated_by: ctx.sender,
        ..elem
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn update_cursor(ctx: &ReducerContext, room_id: u64, x: f64, y: f64) {
    if let Some(c) = ctx.db.cursor().user_id().find(ctx.sender) {
        ctx.db.cursor().user_id().update(Cursor { x, y, room_id, ..c });
    }
}

fn next_z_index(ctx: &ReducerContext, room_id: u64) -> i32 {
    ctx.db.element()
        .room_id()
        .filter(&room_id)
        .map(|e| e.z_index)
        .max()
        .unwrap_or(0) + 1
}

fn serialize_element(elem: &Element) -> Vec<u8> {
    // Simple binary serialization of element fields for undo storage.
    // Using serde_json for simplicity in PoC; could use BSATN for efficiency.
    serde_json::to_vec(elem).unwrap_or_default()
}
```

Note: The `serialize_element` function needs `Element` to derive `serde::Serialize`. We may need to add serde derives to the table struct or use a separate serialization approach. Adjust based on what SpacetimeDB's table macro allows.

- [ ] **Step 3: Implement undo/redo reducers**

Append to `crates/server/src/reducers.rs`:
```rust
#[spacetimedb::reducer]
pub fn undo(ctx: &ReducerContext, room_id: u64) -> Result<(), String> {
    // Find the most recent non-undone entry for this user in this room
    let entry = ctx.db.undo_entry()
        .room_id()
        .filter(&room_id)
        .filter(|e| e.user_id == ctx.sender && !e.undone)
        .max_by_key(|e| e.id);

    let Some(entry) = entry else {
        return Err("Nothing to undo".into());
    };

    match entry.action {
        UndoAction::Create => {
            // Undo create = soft delete
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: true,
                    version: elem.version + 1,
                    updated_by: ctx.sender,
                    ..elem
                });
            }
        }
        UndoAction::Update => {
            // Undo update = restore previous state
            if let Ok(prev) = serde_json::from_slice::<ElementSnapshot>(&entry.prev_state) {
                if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                    ctx.db.element().id().update(Element {
                        x: prev.x,
                        y: prev.y,
                        width: prev.width,
                        height: prev.height,
                        rotation: prev.rotation,
                        stroke_color: prev.stroke_color,
                        fill_color: prev.fill_color,
                        stroke_width: prev.stroke_width,
                        opacity: prev.opacity,
                        font_size: prev.font_size,
                        text_content: prev.text_content,
                        points: prev.points,
                        version: elem.version + 1,
                        updated_by: ctx.sender,
                        ..elem
                    });
                }
            }
        }
        UndoAction::Delete => {
            // Undo delete = undelete
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: false,
                    version: elem.version + 1,
                    updated_by: ctx.sender,
                    ..elem
                });
            }
        }
    }

    // Mark entry as undone
    ctx.db.undo_entry().id().update(UndoEntry {
        undone: true,
        ..entry
    });

    Ok(())
}

#[spacetimedb::reducer]
pub fn redo(ctx: &ReducerContext, room_id: u64) -> Result<(), String> {
    // Find the most recent undone entry for this user
    let entry = ctx.db.undo_entry()
        .room_id()
        .filter(&room_id)
        .filter(|e| e.user_id == ctx.sender && e.undone)
        .max_by_key(|e| e.id);

    let Some(entry) = entry else {
        return Err("Nothing to redo".into());
    };

    match entry.action {
        UndoAction::Create => {
            // Redo create = undelete
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: false,
                    version: elem.version + 1,
                    updated_by: ctx.sender,
                    ..elem
                });
            }
        }
        UndoAction::Update => {
            // Redo update = re-apply the next_state snapshot
            if let Ok(next) = serde_json::from_slice::<ElementSnapshot>(&entry.next_state) {
                if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                    ctx.db.element().id().update(Element {
                        x: next.x,
                        y: next.y,
                        width: next.width,
                        height: next.height,
                        rotation: next.rotation,
                        stroke_color: next.stroke_color,
                        fill_color: next.fill_color,
                        stroke_width: next.stroke_width,
                        opacity: next.opacity,
                        font_size: next.font_size,
                        text_content: next.text_content,
                        points: next.points,
                        version: elem.version + 1,
                        updated_by: ctx.sender,
                        ..elem
                    });
                }
            }
        }
        UndoAction::Delete => {
            // Redo delete = re-delete
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: true,
                    version: elem.version + 1,
                    updated_by: ctx.sender,
                    ..elem
                });
            }
        }
    }

    ctx.db.undo_entry().id().update(UndoEntry {
        undone: false,
        ..entry
    });

    Ok(())
}

/// Snapshot of element fields for undo storage.
/// Separate from the table struct to avoid SpacetimeDB macro conflicts with serde.
#[derive(serde::Serialize, serde::Deserialize)]
struct ElementSnapshot {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    points: Vec<u8>,
    stroke_color: u32,
    fill_color: u32,
    stroke_width: f32,
    opacity: f32,
    font_size: f32,
    text_content: String,
}

impl From<&Element> for ElementSnapshot {
    fn from(e: &Element) -> Self {
        Self {
            x: e.x,
            y: e.y,
            width: e.width,
            height: e.height,
            rotation: e.rotation,
            points: e.points.clone(),
            stroke_color: e.stroke_color,
            fill_color: e.fill_color,
            stroke_width: e.stroke_width,
            opacity: e.opacity,
            font_size: e.font_size,
            text_content: e.text_content.clone(),
        }
    }
}
```

Update `serialize_element` to use `ElementSnapshot`:
```rust
fn serialize_element(elem: &Element) -> Vec<u8> {
    let snapshot = ElementSnapshot::from(elem);
    serde_json::to_vec(&snapshot).unwrap_or_default()
}
```

- [ ] **Step 4: Update lib.rs to include reducers module**

`crates/server/src/lib.rs`:
```rust
mod tables;
mod reducers;

use spacetimedb::{ReducerContext, Table};
```

- [ ] **Step 5: Add serde_json dependency to server**

Add to `crates/server/Cargo.toml`:
```toml
serde = { workspace = true }
serde_json = { workspace = true }
```

- [ ] **Step 6: Verify it compiles**

Run: `cargo check -p server`
Expected: Compiles successfully

- [ ] **Step 7: Commit**

```bash
git add crates/server/
git commit -m "feat: implement SpacetimeDB reducers — CRUD, undo/redo, cursors"
```

---

## Phase 3: SpacetimeDB WASM Client

### Task 3.1: Evaluate Official SDK Browser Feature

**Files:**
- Modify: `crates/stdb-client/Cargo.toml`
- Modify: `crates/stdb-client/src/lib.rs`
- Create: `crates/stdb-client/src/connection.rs`

- [ ] **Step 1: Test if spacetimedb-sdk compiles with browser feature for wasm32**

Run: `cargo check -p stdb-client --target wasm32-unknown-unknown`

If this **succeeds**, the official SDK works in WASM and we wrap it. If it **fails**, we need to implement a custom client using `spacetimedb-client-api-messages` + `gloo-net`. Document which path was taken.

- [ ] **Step 2a (SDK works): Implement connection wrapper**

`crates/stdb-client/src/connection.rs`:
```rust
use leptos::prelude::*;
use std::sync::Arc;

/// Wrapper around SpacetimeDB SDK connection for Leptos integration.
/// Provides reactive signals that update when SpacetimeDB pushes deltas.
pub struct StdbConnection {
    // SDK connection handle — exact type depends on spacetimedb-sdk API
    // Will be filled in during implementation based on SDK's public API
}

impl StdbConnection {
    pub async fn connect(host: &str, db_name: &str) -> Result<Self, String> {
        // Use spacetimedb-sdk's browser WebSocket connection
        todo!("Implement based on SDK API discovery")
    }
}
```

Note: The exact API depends on what `spacetimedb-sdk` with `browser` feature exposes. The implementer should read the SDK source to discover the connection API, then wrap it here.

- [ ] **Step 2b (SDK fails): Implement custom WebSocket client**

If the SDK doesn't compile for WASM, create a custom client using:
- `gloo-net` for WebSocket
- `spacetimedb-client-api-messages` for message types
- `spacetimedb-lib::bsatn` for serialization

Replace `stdb-client` dependencies in Cargo.toml:
```toml
[dependencies]
shared = { workspace = true }
spacetimedb-client-api-messages = { git = "https://github.com/clockworklabs/SpacetimeDB", tag = "v2.0.5" }
spacetimedb-lib = { workspace = true }
gloo-net = { version = "0.6", features = ["websocket"] }
leptos = { workspace = true }
wasm-bindgen = { workspace = true }
wasm-bindgen-futures = "0.4"
flate2 = "1"  # for gzip decompression
log = { workspace = true }
```

Then implement:
- `ws.rs` — WebSocket connection with reconnection
- `protocol.rs` — encode ClientMessage, decode ServerMessage with compression
- `cache.rs` — client-side row cache

This is substantial work. Consult the wire protocol details in the spec research.

- [ ] **Step 3: Verify stdb-client compiles for WASM target**

Run: `cargo check -p stdb-client --target wasm32-unknown-unknown`
Expected: Compiles successfully

- [ ] **Step 4: Commit**

```bash
git add crates/stdb-client/
git commit -m "feat: implement SpacetimeDB WASM client"
```

---

### Task 3.2: Leptos Signal Integration

**Files:**
- Create: `crates/stdb-client/src/signals.rs`
- Modify: `crates/stdb-client/src/lib.rs`

- [ ] **Step 1: Define reactive store types**

`crates/stdb-client/src/signals.rs`:
```rust
use leptos::prelude::*;
use shared::point::Point;
use std::collections::HashMap;

/// Client-side representation of an element (mirrors server Element table).
#[derive(Clone, Debug)]
pub struct ElementData {
    pub id: u64,
    pub room_id: u64,
    pub kind: shared::ElementKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub points: Vec<Point>,
    pub stroke_color: u32,
    pub fill_color: u32,
    pub stroke_width: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub text_content: String,
    pub z_index: i32,
}

/// Client-side cursor representation.
#[derive(Clone, Debug)]
pub struct CursorData {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub color: u32,
}

/// Reactive store for all SpacetimeDB state.
#[derive(Clone)]
pub struct StdbStore {
    pub elements: RwSignal<HashMap<u64, ElementData>>,
    pub cursors: RwSignal<HashMap<String, CursorData>>, // keyed by identity hex
    pub connected: RwSignal<bool>,
    pub my_identity: RwSignal<Option<String>>,
    pub current_room: RwSignal<Option<u64>>,
}

impl StdbStore {
    pub fn new() -> Self {
        Self {
            elements: RwSignal::new(HashMap::new()),
            cursors: RwSignal::new(HashMap::new()),
            connected: RwSignal::new(false),
            my_identity: RwSignal::new(None),
            current_room: RwSignal::new(None),
        }
    }

    /// Get elements sorted by z_index for rendering.
    pub fn sorted_elements(&self) -> Vec<ElementData> {
        self.elements.with(|elems| {
            let mut sorted: Vec<_> = elems.values().cloned().collect();
            sorted.sort_by_key(|e| e.z_index);
            sorted
        })
    }
}
```

- [ ] **Step 2: Update lib.rs exports**

`crates/stdb-client/src/lib.rs`:
```rust
pub mod connection;
pub mod signals;

pub use signals::{StdbStore, ElementData, CursorData};
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo check -p stdb-client --target wasm32-unknown-unknown`
Expected: Compiles

- [ ] **Step 4: Commit**

```bash
git add crates/stdb-client/
git commit -m "feat: add Leptos signal integration for SpacetimeDB store"
```

---

## Phase 4: Canvas Rendering

### Task 4.1: App Shell & Canvas Setup

**Files:**
- Modify: `crates/app/src/main.rs`
- Create: `crates/app/src/state.rs`
- Create: `crates/app/src/canvas.rs`
- Create: `crates/app/src/ui.rs`

- [ ] **Step 1: Define app state**

`crates/app/src/state.rs`:
```rust
use leptos::prelude::*;
use stdb_client::StdbStore;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum VimMode {
    Normal,
    Insert,
    Visual,
    Command,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Tool {
    Select,
    Rectangle,
    Ellipse,
    Arrow,
    Line,
    Text,
    Freehand,
}

#[derive(Clone)]
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self { x: 0.0, y: 0.0, zoom: 1.0 }
    }
}

/// Global application state, provided via Leptos context.
#[derive(Clone)]
pub struct AppState {
    pub store: StdbStore,
    pub mode: RwSignal<VimMode>,
    pub tool: RwSignal<Tool>,
    pub camera: RwSignal<Camera>,
    pub selected_ids: RwSignal<Vec<u64>>,
    pub command_buffer: RwSignal<String>,
    pub key_buffer: RwSignal<String>,
    pub stroke_color: RwSignal<u32>,
    pub fill_color: RwSignal<u32>,
    pub stroke_width: RwSignal<f32>,
    pub mouse_pos: RwSignal<(f64, f64)>,
    pub canvas_size: RwSignal<(f64, f64)>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: StdbStore::new(),
            mode: RwSignal::new(VimMode::Normal),
            tool: RwSignal::new(Tool::Select),
            camera: RwSignal::new(Camera::default()),
            selected_ids: RwSignal::new(vec![]),
            command_buffer: RwSignal::new(String::new()),
            key_buffer: RwSignal::new(String::new()),
            stroke_color: RwSignal::new(shared::DEFAULT_STROKE_COLOR),
            fill_color: RwSignal::new(shared::DEFAULT_FILL_COLOR),
            stroke_width: RwSignal::new(shared::DEFAULT_STROKE_WIDTH),
            mouse_pos: RwSignal::new((0.0, 0.0)),
            canvas_size: RwSignal::new((0.0, 0.0)),
        }
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        let cam = self.camera.get_untracked();
        let wx = (sx / cam.zoom) + cam.x;
        let wy = (sy / cam.zoom) + cam.y;
        (wx, wy)
    }

    /// Convert world coordinates to screen coordinates.
    pub fn world_to_screen(&self, wx: f64, wy: f64) -> (f64, f64) {
        let cam = self.camera.get_untracked();
        let sx = (wx - cam.x) * cam.zoom;
        let sy = (wy - cam.y) * cam.zoom;
        (sx, sy)
    }
}
```

- [ ] **Step 2: Create canvas component**

`crates/app/src/canvas.rs`:
```rust
use leptos::prelude::*;
use leptos::html::Canvas;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::CanvasRenderingContext2d;
use std::cell::RefCell;
use std::rc::Rc;
use std::f64::consts::TAU;
use crate::state::AppState;
use stdb_client::ElementData;
use shared::ElementKind;

#[component]
pub fn DrawCanvas() -> impl IntoView {
    let state = expect_context::<AppState>();
    let canvas_ref: NodeRef<Canvas> = NodeRef::new();

    // Render loop
    Effect::new(move || {
        let Some(canvas) = canvas_ref.get() else { return };

        // Size canvas to window
        let window = web_sys::window().unwrap();
        let w = window.inner_width().unwrap().as_f64().unwrap();
        let h = window.inner_height().unwrap().as_f64().unwrap() - 64.0; // top + bottom bars
        canvas.set_width(w as u32);
        canvas.set_height(h as u32);
        state.canvas_size.set((w, h));

        let ctx: CanvasRenderingContext2d = canvas
            .get_context("2d")
            .unwrap()
            .unwrap()
            .dyn_into()
            .unwrap();

        let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
        let g = f.clone();
        let win = window.clone();

        *g.borrow_mut() = Some(Closure::new(move || {
            render_frame(&ctx, &state);
            win.request_animation_frame(
                f.borrow().as_ref().unwrap().as_ref().unchecked_ref()
            ).unwrap();
        }));

        window.request_animation_frame(
            g.borrow().as_ref().unwrap().as_ref().unchecked_ref()
        ).unwrap();
    });

    // Mouse handlers
    let state2 = state.clone();
    let on_mousedown = move |ev: web_sys::MouseEvent| {
        let (x, y) = canvas_mouse_pos(&canvas_ref, &ev);
        let (wx, wy) = state2.screen_to_world(x, y);
        // Will be handled by tools module
        log::debug!("mousedown at world ({}, {})", wx, wy);
    };

    let state3 = state.clone();
    let on_mousemove = move |ev: web_sys::MouseEvent| {
        let (x, y) = canvas_mouse_pos(&canvas_ref, &ev);
        state3.mouse_pos.set((x, y));
    };

    let on_mouseup = move |_ev: web_sys::MouseEvent| {
        // Will be handled by tools module
    };

    let state4 = state.clone();
    let on_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        let delta = ev.delta_y();
        state4.camera.update(|cam| {
            let factor = if delta > 0.0 { 0.9 } else { 1.1 };
            cam.zoom = (cam.zoom * factor).clamp(0.1, 10.0);
        });
    };

    view! {
        <canvas
            node_ref=canvas_ref
            on:mousedown=on_mousedown
            on:mousemove=on_mousemove
            on:mouseup=on_mouseup
            on:wheel=on_wheel
            on:contextmenu=move |ev: web_sys::MouseEvent| { ev.prevent_default(); }
            style="display:block; cursor:crosshair;"
        />
    }
}

fn canvas_mouse_pos(
    canvas_ref: &NodeRef<Canvas>,
    ev: &web_sys::MouseEvent,
) -> (f64, f64) {
    if let Some(canvas) = canvas_ref.get_untracked() {
        let rect = canvas.get_bounding_client_rect();
        (
            ev.client_x() as f64 - rect.left(),
            ev.client_y() as f64 - rect.top(),
        )
    } else {
        (0.0, 0.0)
    }
}

fn render_frame(ctx: &CanvasRenderingContext2d, state: &AppState) {
    let (w, h) = state.canvas_size.get_untracked();
    let cam = state.camera.get_untracked();

    // Clear
    ctx.set_fill_style_str("#1a1a2e");
    ctx.fill_rect(0.0, 0.0, w, h);

    // Apply camera transform
    ctx.save();
    ctx.scale(cam.zoom, cam.zoom).unwrap();
    ctx.translate(-cam.x, -cam.y).unwrap();

    // Render elements
    let elements = state.store.sorted_elements();
    let selected = state.selected_ids.get_untracked();

    for elem in &elements {
        render_element(ctx, elem);
        if selected.contains(&elem.id) {
            render_selection_handles(ctx, elem);
        }
    }

    // Render cursors
    let cursors = state.store.cursors.get_untracked();
    for cursor in cursors.values() {
        render_cursor(ctx, cursor);
    }

    ctx.restore();
}

fn render_element(ctx: &CanvasRenderingContext2d, elem: &ElementData) {
    let stroke = color_u32_to_css(elem.stroke_color);
    let fill = color_u32_to_css(elem.fill_color);

    ctx.save();
    ctx.set_global_alpha(elem.opacity as f64);
    ctx.set_stroke_style_str(&stroke);
    ctx.set_fill_style_str(&fill);
    ctx.set_line_width(elem.stroke_width as f64);

    if elem.rotation != 0.0 {
        let cx = elem.x + elem.width / 2.0;
        let cy = elem.y + elem.height / 2.0;
        ctx.translate(cx, cy).unwrap();
        ctx.rotate(elem.rotation).unwrap();
        ctx.translate(-cx, -cy).unwrap();
    }

    match elem.kind {
        ElementKind::Rectangle => {
            if elem.fill_color & 0xFF != 0 {
                ctx.fill_rect(elem.x, elem.y, elem.width, elem.height);
            }
            ctx.stroke_rect(elem.x, elem.y, elem.width, elem.height);
        }
        ElementKind::Ellipse => {
            ctx.begin_path();
            ctx.ellipse(
                elem.x + elem.width / 2.0,
                elem.y + elem.height / 2.0,
                (elem.width / 2.0).abs(),
                (elem.height / 2.0).abs(),
                0.0, 0.0, TAU,
            ).unwrap();
            if elem.fill_color & 0xFF != 0 {
                ctx.fill();
            }
            ctx.stroke();
        }
        ElementKind::Arrow | ElementKind::Line => {
            if elem.points.len() >= 2 {
                ctx.begin_path();
                ctx.move_to(elem.points[0].x, elem.points[0].y);
                for p in &elem.points[1..] {
                    ctx.line_to(p.x, p.y);
                }
                ctx.stroke();

                // Arrowhead for Arrow kind
                if matches!(elem.kind, ElementKind::Arrow) && elem.points.len() >= 2 {
                    let end = elem.points.last().unwrap();
                    let prev = &elem.points[elem.points.len() - 2];
                    let angle = (end.y - prev.y).atan2(end.x - prev.x);
                    let arrow_len = 15.0;
                    ctx.begin_path();
                    ctx.move_to(end.x, end.y);
                    ctx.line_to(
                        end.x - arrow_len * (angle - 0.4).cos(),
                        end.y - arrow_len * (angle - 0.4).sin(),
                    );
                    ctx.move_to(end.x, end.y);
                    ctx.line_to(
                        end.x - arrow_len * (angle + 0.4).cos(),
                        end.y - arrow_len * (angle + 0.4).sin(),
                    );
                    ctx.stroke();
                }
            }
        }
        ElementKind::Text => {
            ctx.set_font(&format!("{}px monospace", elem.font_size));
            ctx.set_fill_style_str(&stroke); // text uses stroke color
            for (i, line) in elem.text_content.lines().enumerate() {
                ctx.fill_text(
                    line,
                    elem.x,
                    elem.y + (i as f64 + 1.0) * elem.font_size as f64,
                ).unwrap();
            }
        }
        ElementKind::Freehand => {
            if elem.points.len() >= 2 {
                ctx.begin_path();
                ctx.move_to(elem.points[0].x, elem.points[0].y);
                for p in &elem.points[1..] {
                    ctx.line_to(p.x, p.y);
                }
                ctx.stroke();
            }
        }
    }

    ctx.restore();
}

fn render_selection_handles(ctx: &CanvasRenderingContext2d, elem: &ElementData) {
    let handle_size = 8.0;
    let half = handle_size / 2.0;
    ctx.set_fill_style_str("#6ee7b7");
    ctx.set_stroke_style_str("#1a1a2e");
    ctx.set_line_width(1.0);

    let x = elem.x;
    let y = elem.y;
    let w = elem.width;
    let h = elem.height;

    // 8 handles: corners + midpoints
    let handles = [
        (x, y), (x + w / 2.0, y), (x + w, y),
        (x, y + h / 2.0), (x + w, y + h / 2.0),
        (x, y + h), (x + w / 2.0, y + h), (x + w, y + h),
    ];

    for (hx, hy) in handles {
        ctx.fill_rect(hx - half, hy - half, handle_size, handle_size);
        ctx.stroke_rect(hx - half, hy - half, handle_size, handle_size);
    }
}

fn render_cursor(ctx: &CanvasRenderingContext2d, cursor: &stdb_client::CursorData) {
    let color = color_u32_to_css(cursor.color);
    ctx.set_fill_style_str(&color);

    // Cursor dot
    ctx.begin_path();
    ctx.arc(cursor.x, cursor.y, 5.0, 0.0, TAU).unwrap();
    ctx.fill();

    // Name label
    ctx.set_font("12px monospace");
    ctx.fill_text(&cursor.name, cursor.x + 8.0, cursor.y - 8.0).unwrap();
}

fn color_u32_to_css(color: u32) -> String {
    let r = (color >> 24) & 0xFF;
    let g = (color >> 16) & 0xFF;
    let b = (color >> 8) & 0xFF;
    let a = color & 0xFF;
    format!("rgba({},{},{},{})", r, g, b, a as f64 / 255.0)
}
```

- [ ] **Step 3: Create UI shell (top bar + bottom bar)**

`crates/app/src/ui.rs`:
```rust
use leptos::prelude::*;
use crate::state::{AppState, VimMode};

#[component]
pub fn TopBar() -> impl IntoView {
    let state = expect_context::<AppState>();

    let mode_display = move || {
        let mode = state.mode.get();
        let (label, color) = match mode {
            VimMode::Normal => ("NORMAL", "#6ee7b7"),
            VimMode::Insert => ("INSERT", "#f59e0b"),
            VimMode::Visual => ("VISUAL", "#9382dc"),
            VimMode::Command => ("COMMAND", "#ef4444"),
        };
        view! {
            <span style=format!(
                "padding: 2px 8px; border-radius: 3px; background: {}22; color: {};",
                color, color
            )>{label}</span>
        }
    };

    let room_display = move || {
        state.store.current_room.get()
            .map(|id| format!("Room #{}", id))
            .unwrap_or_else(|| "No room".into())
    };

    let connection_display = move || {
        if state.store.connected.get() { "Connected" } else { "Offline" }
    };

    view! {
        <div style="height: 40px; background: #16213e; display: flex; align-items: center; padding: 0 1rem; font-size: 0.85rem; gap: 1rem; border-bottom: 1px solid #1a1a4e;">
            <span style="color: #6ee7b7; font-weight: bold;">"collaborate"</span>
            <span style="color: #444;">"|"</span>
            <span style="color: #888;">{room_display}</span>
            <span style="color: #444;">"|"</span>
            <span style="color: #888;">{connection_display}</span>
            <span style="flex: 1;" />
            {mode_display}
        </div>
    }
}

#[component]
pub fn BottomBar() -> impl IntoView {
    let state = expect_context::<AppState>();

    let command_display = move || {
        let mode = state.mode.get();
        let cmd = state.command_buffer.get();
        if mode == VimMode::Command {
            format!(":{}", cmd)
        } else {
            let keys = state.key_buffer.get();
            keys
        }
    };

    let position_display = move || {
        let (x, y) = state.mouse_pos.get();
        let cam = state.camera.get();
        let wx = (x / cam.zoom) + cam.x;
        let wy = (y / cam.zoom) + cam.y;
        format!("{:.0}% | ({:.0}, {:.0})", cam.zoom * 100.0, wx, wy)
    };

    view! {
        <div style="height: 24px; background: #16213e; display: flex; align-items: center; padding: 0 1rem; font-size: 0.75rem; border-top: 1px solid #1a1a4e;">
            <span style="color: #888; min-width: 200px;">{command_display}</span>
            <span style="flex: 1;" />
            <span style="color: #666;">{position_display}</span>
        </div>
    }
}
```

- [ ] **Step 4: Wire up main.rs**

`crates/app/src/main.rs`:
```rust
mod state;
mod canvas;
mod ui;

use leptos::prelude::*;
use state::AppState;
use canvas::DrawCanvas;
use ui::{TopBar, BottomBar};

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state);

    view! {
        <div style="display: flex; flex-direction: column; height: 100vh;">
            <TopBar />
            <div style="flex: 1; overflow: hidden;">
                <DrawCanvas />
            </div>
            <BottomBar />
        </div>
    }
}
```

- [ ] **Step 5: Verify it builds with Trunk**

Run: `cd crates/app && trunk build`
Expected: Builds successfully, produces dist/ with WASM + HTML

- [ ] **Step 6: Commit**

```bash
git add crates/app/
git commit -m "feat: app shell with canvas rendering, top bar, bottom bar"
```

---

## Phase 5: Vim Mode

### Task 5.1: Vim State Machine

**Files:**
- Create: `crates/app/src/vim.rs`
- Test: inline tests in `crates/app/src/vim.rs`

- [ ] **Step 1: Write tests for vim state transitions**

`crates/app/src/vim.rs`:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normal_to_insert_via_r() {
        let mut vim = VimStateMachine::new();
        assert_eq!(vim.mode(), VimMode::Normal);
        let action = vim.handle_key("r", false, false);
        assert_eq!(vim.mode(), VimMode::Insert);
        assert!(matches!(action, VimAction::SetTool(Tool::Rectangle)));
    }

    #[test]
    fn insert_to_normal_via_escape() {
        let mut vim = VimStateMachine::new();
        vim.handle_key("r", false, false);
        assert_eq!(vim.mode(), VimMode::Insert);
        let action = vim.handle_key("Escape", false, false);
        assert_eq!(vim.mode(), VimMode::Normal);
        assert!(matches!(action, VimAction::SetTool(Tool::Select)));
    }

    #[test]
    fn normal_dd_deletes() {
        let mut vim = VimStateMachine::new();
        let a1 = vim.handle_key("d", false, false);
        assert!(matches!(a1, VimAction::None)); // buffered
        let a2 = vim.handle_key("d", false, false);
        assert!(matches!(a2, VimAction::DeleteSelected));
    }

    #[test]
    fn normal_to_command_mode() {
        let mut vim = VimStateMachine::new();
        let action = vim.handle_key(":", false, false);
        assert_eq!(vim.mode(), VimMode::Command);
        assert!(matches!(action, VimAction::EnterCommand));
    }

    #[test]
    fn hjkl_movement() {
        let mut vim = VimStateMachine::new();
        let a = vim.handle_key("h", false, false);
        assert!(matches!(a, VimAction::MoveSelected(-10.0, 0.0)));
        let a = vim.handle_key("j", false, false);
        assert!(matches!(a, VimAction::MoveSelected(0.0, 10.0)));
        let a = vim.handle_key("k", false, false);
        assert!(matches!(a, VimAction::MoveSelected(0.0, -10.0)));
        let a = vim.handle_key("l", false, false);
        assert!(matches!(a, VimAction::MoveSelected(10.0, 0.0)));
    }

    #[test]
    fn shift_hjkl_fine_movement() {
        let mut vim = VimStateMachine::new();
        let a = vim.handle_key("h", true, false);
        assert!(matches!(a, VimAction::MoveSelected(-1.0, 0.0)));
    }

    #[test]
    fn undo_redo() {
        let mut vim = VimStateMachine::new();
        let a = vim.handle_key("u", false, false);
        assert!(matches!(a, VimAction::Undo));
        let a = vim.handle_key("r", false, true); // Ctrl+r
        assert!(matches!(a, VimAction::Redo));
    }

    #[test]
    fn visual_mode() {
        let mut vim = VimStateMachine::new();
        let a = vim.handle_key("v", false, false);
        assert_eq!(vim.mode(), VimMode::Visual);
        assert!(matches!(a, VimAction::EnterVisual));
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test -p app --lib vim::tests`
Expected: FAIL — VimStateMachine not defined

- [ ] **Step 3: Implement VimStateMachine**

`crates/app/src/vim.rs` (above the tests module):
```rust
use crate::state::{VimMode, Tool};

#[derive(Debug, PartialEq)]
pub enum VimAction {
    None,
    SetTool(Tool),
    MoveSelected(f64, f64),
    DeleteSelected,
    CopySelected,
    Paste,
    Undo,
    Redo,
    EnterCommand,
    EnterVisual,
    ExitToNormal,
    CommandChar(char),
    CommandSubmit(String),
    CommandBackspace,
}

pub struct VimStateMachine {
    mode: VimMode,
    key_buffer: String,
    command_buffer: String,
}

impl VimStateMachine {
    pub fn new() -> Self {
        Self {
            mode: VimMode::Normal,
            key_buffer: String::new(),
            command_buffer: String::new(),
        }
    }

    pub fn mode(&self) -> VimMode {
        self.mode
    }

    pub fn key_buffer(&self) -> &str {
        &self.key_buffer
    }

    pub fn command_buffer(&self) -> &str {
        &self.command_buffer
    }

    /// Handle a key press. Returns the action to take.
    /// `shift` and `ctrl` indicate modifier keys.
    pub fn handle_key(&mut self, key: &str, shift: bool, ctrl: bool) -> VimAction {
        match self.mode {
            VimMode::Normal => self.handle_normal(key, shift, ctrl),
            VimMode::Insert => self.handle_insert(key, shift, ctrl),
            VimMode::Visual => self.handle_visual(key, shift, ctrl),
            VimMode::Command => self.handle_command(key, shift, ctrl),
        }
    }

    fn handle_normal(&mut self, key: &str, shift: bool, ctrl: bool) -> VimAction {
        // Check for buffered key sequences (like "dd", "yy")
        if !self.key_buffer.is_empty() {
            let combined = format!("{}{}", self.key_buffer, key);
            self.key_buffer.clear();
            return match combined.as_str() {
                "dd" => VimAction::DeleteSelected,
                "yy" => VimAction::CopySelected,
                _ => VimAction::None,
            };
        }

        let step = if shift { 1.0 } else { 10.0 };

        match key {
            "h" => VimAction::MoveSelected(-step, 0.0),
            "j" => VimAction::MoveSelected(0.0, step),
            "k" => VimAction::MoveSelected(0.0, -step),
            "l" if !shift => VimAction::MoveSelected(step, 0.0),
            "r" if ctrl => VimAction::Redo,
            "r" => {
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Rectangle)
            }
            "e" => {
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Ellipse)
            }
            "a" => {
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Arrow)
            }
            "L" | "l" if shift => {
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Line)
            }
            "t" => {
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Text)
            }
            "f" => {
                self.mode = VimMode::Insert;
                VimAction::SetTool(Tool::Freehand)
            }
            "d" | "y" => {
                self.key_buffer = key.to_string();
                VimAction::None
            }
            "p" => VimAction::Paste,
            "u" => VimAction::Undo,
            "v" => {
                self.mode = VimMode::Visual;
                VimAction::EnterVisual
            }
            ":" => {
                self.mode = VimMode::Command;
                self.command_buffer.clear();
                VimAction::EnterCommand
            }
            "Escape" => {
                self.key_buffer.clear();
                VimAction::ExitToNormal
            }
            _ => VimAction::None,
        }
    }

    fn handle_insert(&mut self, key: &str, _shift: bool, _ctrl: bool) -> VimAction {
        match key {
            "Escape" => {
                self.mode = VimMode::Normal;
                VimAction::SetTool(Tool::Select)
            }
            _ => VimAction::None, // Insert mode keys handled by tools
        }
    }

    fn handle_visual(&mut self, key: &str, shift: bool, _ctrl: bool) -> VimAction {
        let step = if shift { 1.0 } else { 10.0 };
        match key {
            "Escape" => {
                self.mode = VimMode::Normal;
                VimAction::ExitToNormal
            }
            "h" => VimAction::MoveSelected(-step, 0.0),
            "j" => VimAction::MoveSelected(0.0, step),
            "k" => VimAction::MoveSelected(0.0, -step),
            "l" => VimAction::MoveSelected(step, 0.0),
            "d" => {
                self.mode = VimMode::Normal;
                VimAction::DeleteSelected
            }
            "y" => {
                self.mode = VimMode::Normal;
                VimAction::CopySelected
            }
            _ => VimAction::None,
        }
    }

    fn handle_command(&mut self, key: &str, _shift: bool, _ctrl: bool) -> VimAction {
        match key {
            "Escape" => {
                self.mode = VimMode::Normal;
                self.command_buffer.clear();
                VimAction::ExitToNormal
            }
            "Enter" => {
                self.mode = VimMode::Normal;
                let cmd = self.command_buffer.clone();
                self.command_buffer.clear();
                VimAction::CommandSubmit(cmd)
            }
            "Backspace" => {
                self.command_buffer.pop();
                VimAction::CommandBackspace
            }
            k if k.len() == 1 => {
                let ch = k.chars().next().unwrap();
                self.command_buffer.push(ch);
                VimAction::CommandChar(ch)
            }
            _ => VimAction::None,
        }
    }
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test -p app --lib vim::tests`
Expected: All tests pass

- [ ] **Step 5: Commit**

```bash
git add crates/app/src/vim.rs
git commit -m "feat: implement vim state machine with Normal/Insert/Visual/Command modes"
```

---

### Task 5.2: Wire Vim to App

**Files:**
- Create: `crates/app/src/commands.rs`
- Modify: `crates/app/src/main.rs`

- [ ] **Step 1: Create command handler for vim actions**

`crates/app/src/commands.rs`:
```rust
use crate::state::AppState;
use crate::vim::VimAction;

/// Process a VimAction and apply it to the app state.
pub fn handle_vim_action(state: &AppState, action: VimAction) {
    match action {
        VimAction::None => {}
        VimAction::SetTool(tool) => {
            state.tool.set(tool);
        }
        VimAction::MoveSelected(dx, dy) => {
            // TODO: call update_element reducer for each selected element
            log::debug!("Move selected by ({}, {})", dx, dy);
        }
        VimAction::DeleteSelected => {
            // TODO: call delete_element reducer for each selected element
            log::debug!("Delete selected");
        }
        VimAction::CopySelected => {
            // TODO: copy selected elements to clipboard signal
            log::debug!("Copy selected");
        }
        VimAction::Paste => {
            // TODO: paste from clipboard signal
            log::debug!("Paste");
        }
        VimAction::Undo => {
            // TODO: call undo reducer
            log::debug!("Undo");
        }
        VimAction::Redo => {
            // TODO: call redo reducer
            log::debug!("Redo");
        }
        VimAction::EnterCommand => {
            state.mode.set(crate::state::VimMode::Command);
        }
        VimAction::EnterVisual => {
            state.mode.set(crate::state::VimMode::Visual);
        }
        VimAction::ExitToNormal => {
            state.mode.set(crate::state::VimMode::Normal);
            state.selected_ids.update(|ids| ids.clear());
        }
        VimAction::CommandChar(_) | VimAction::CommandBackspace => {
            // Command buffer is managed by VimStateMachine
        }
        VimAction::CommandSubmit(cmd) => {
            handle_command(state, &cmd);
        }
    }
}

fn handle_command(state: &AppState, cmd: &str) {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    match parts.as_slice() {
        ["w"] => {
            // TODO: export PNG
            log::info!("Export PNG");
        }
        ["ws"] => {
            // TODO: export SVG
            log::info!("Export SVG");
        }
        ["wq"] => {
            // TODO: export and close
            log::info!("Export and close");
        }
        ["color", hex] => {
            if let Some(color) = parse_hex_color(hex) {
                state.stroke_color.set(color);
                log::info!("Set stroke color to {}", hex);
            }
        }
        ["fill", hex] => {
            if let Some(color) = parse_hex_color(hex) {
                state.fill_color.set(color);
                log::info!("Set fill color to {}", hex);
            }
        }
        ["stroke", width] => {
            if let Ok(w) = width.parse::<f32>() {
                state.stroke_width.set(w);
                log::info!("Set stroke width to {}", w);
            }
        }
        _ => {
            log::warn!("Unknown command: {}", cmd);
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<u32> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            // Short form: #rgb → #rrggbb
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(u32::from_be_bytes([r, g, b, 0xFF]))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(u32::from_be_bytes([r, g, b, 0xFF]))
        }
        _ => None,
    }
}
```

- [ ] **Step 2: Add global keyboard listener to main.rs**

Add to `App` component in `crates/app/src/main.rs`, inside the component function before the `view!` macro:

```rust
use crate::vim::VimStateMachine;
use crate::commands::handle_vim_action;
use std::cell::RefCell;
use std::rc::Rc;

// Inside App component:
let vim = Rc::new(RefCell::new(VimStateMachine::new()));

// Global keyboard handler
{
    let vim = vim.clone();
    let state2 = state.clone();
    Effect::new(move || {
        let vim = vim.clone();
        let state2 = state2.clone();
        let closure = wasm_bindgen::closure::Closure::<dyn FnMut(_)>::new(
            move |ev: web_sys::KeyboardEvent| {
                // Don't capture if we're in text input mode and typing text
                let action = {
                    let mut vm = vim.borrow_mut();
                    let action = vm.handle_key(
                        &ev.key(),
                        ev.shift_key(),
                        ev.ctrl_key(),
                    );
                    state2.mode.set(vm.mode());
                    state2.key_buffer.set(vm.key_buffer().to_string());
                    state2.command_buffer.set(vm.command_buffer().to_string());
                    action
                };
                handle_vim_action(&state2, action);

                // Prevent default for keys we handle
                ev.prevent_default();
            },
        );
        let doc = leptos::prelude::document();
        doc.add_event_listener_with_callback(
            "keydown",
            closure.as_ref().unchecked_ref(),
        ).unwrap();
        closure.forget();
    });
}
```

- [ ] **Step 3: Update main.rs module declarations**

```rust
mod state;
mod canvas;
mod ui;
mod vim;
mod commands;
```

- [ ] **Step 4: Verify it builds**

Run: `cd crates/app && trunk build`
Expected: Builds successfully

- [ ] **Step 5: Commit**

```bash
git add crates/app/
git commit -m "feat: wire vim keybindings to global keyboard listener"
```

---

## Phase 6: Drawing Tools & Features

### Task 6.1: Drawing Tools (Shape Creation)

**Files:**
- Create: `crates/app/src/tools.rs`
- Modify: `crates/app/src/canvas.rs` (wire up mouse handlers to tools)

- [ ] **Step 1: Implement tool handler trait and shape tools**

`crates/app/src/tools.rs`:
```rust
use crate::state::{AppState, Tool, VimMode};
use shared::{ElementKind, Point};
use stdb_client::ElementData;
use std::collections::HashMap;

/// Tracks in-progress drawing state.
#[derive(Clone, Debug)]
pub enum DrawingState {
    None,
    /// Dragging to create a shape (rectangle, ellipse).
    ShapeDrag { start_x: f64, start_y: f64 },
    /// Placing arrow/line endpoints.
    LinePlacement { points: Vec<Point> },
    /// Typing text at a position.
    TextInput { x: f64, y: f64, text: String },
    /// Drawing freehand.
    FreehandDraw { points: Vec<Point> },
}

pub struct ToolHandler {
    pub drawing: DrawingState,
}

impl ToolHandler {
    pub fn new() -> Self {
        Self { drawing: DrawingState::None }
    }

    pub fn on_mouse_down(&mut self, state: &AppState, wx: f64, wy: f64) {
        let tool = state.tool.get_untracked();
        let mode = state.mode.get_untracked();

        match (mode, tool) {
            (VimMode::Normal, Tool::Select) => {
                // Hit test to select an element
                let elements = state.store.sorted_elements();
                if let Some(elem) = hit_test(&elements, wx, wy) {
                    state.selected_ids.set(vec![elem.id]);
                } else {
                    state.selected_ids.set(vec![]);
                }
            }
            (VimMode::Insert, Tool::Rectangle | Tool::Ellipse) => {
                self.drawing = DrawingState::ShapeDrag { start_x: wx, start_y: wy };
            }
            (VimMode::Insert, Tool::Arrow | Tool::Line) => {
                match &mut self.drawing {
                    DrawingState::LinePlacement { points } => {
                        points.push(Point { x: wx, y: wy });
                        if points.len() >= 2 {
                            // Create the element
                            let tool = state.tool.get_untracked();
                            let kind = if matches!(tool, Tool::Arrow) {
                                ElementKind::Arrow
                            } else {
                                ElementKind::Line
                            };
                            create_line_element(state, kind, points.clone());
                            self.drawing = DrawingState::None;
                            // Stay in insert mode for more lines
                        }
                    }
                    _ => {
                        self.drawing = DrawingState::LinePlacement {
                            points: vec![Point { x: wx, y: wy }],
                        };
                    }
                }
            }
            (VimMode::Insert, Tool::Text) => {
                self.drawing = DrawingState::TextInput {
                    x: wx, y: wy, text: String::new(),
                };
            }
            (VimMode::Insert, Tool::Freehand) => {
                self.drawing = DrawingState::FreehandDraw {
                    points: vec![Point { x: wx, y: wy }],
                };
            }
            _ => {}
        }
    }

    pub fn on_mouse_move(&mut self, _state: &AppState, wx: f64, wy: f64) {
        match &mut self.drawing {
            DrawingState::FreehandDraw { points } => {
                points.push(Point { x: wx, y: wy });
            }
            _ => {}
        }
    }

    pub fn on_mouse_up(&mut self, state: &AppState, wx: f64, wy: f64) {
        match &self.drawing {
            DrawingState::ShapeDrag { start_x, start_y } => {
                let tool = state.tool.get_untracked();
                let kind = if matches!(tool, Tool::Rectangle) {
                    ElementKind::Rectangle
                } else {
                    ElementKind::Ellipse
                };
                let x = start_x.min(wx);
                let y = start_y.min(wy);
                let w = (wx - start_x).abs();
                let h = (wy - start_y).abs();

                if w > 2.0 && h > 2.0 {
                    create_shape_element(state, kind, x, y, w, h);
                }
                self.drawing = DrawingState::None;
            }
            DrawingState::FreehandDraw { points } => {
                if points.len() >= 2 {
                    create_freehand_element(state, points.clone());
                }
                self.drawing = DrawingState::None;
            }
            _ => {}
        }
    }

    pub fn on_text_key(&mut self, state: &AppState, key: &str) {
        if let DrawingState::TextInput { x, y, text } = &mut self.drawing {
            match key {
                "Escape" => {
                    if !text.is_empty() {
                        create_text_element(state, *x, *y, text.clone());
                    }
                    self.drawing = DrawingState::None;
                }
                "Enter" => {
                    text.push('\n');
                }
                "Backspace" => {
                    text.pop();
                }
                k if k.len() == 1 => {
                    text.push_str(k);
                }
                _ => {}
            }
        }
    }

    /// Render in-progress drawing preview (ghost shape).
    pub fn render_preview(
        &self,
        ctx: &web_sys::CanvasRenderingContext2d,
        state: &AppState,
    ) {
        let (mx, my) = state.mouse_pos.get_untracked();
        let (wmx, wmy) = state.screen_to_world(mx, my);

        ctx.set_stroke_style_str("rgba(110, 231, 183, 0.5)");
        ctx.set_line_width(2.0);
        ctx.set_line_dash(&js_sys::Array::of2(
            &wasm_bindgen::JsValue::from_f64(5.0),
            &wasm_bindgen::JsValue::from_f64(5.0),
        )).unwrap();

        match &self.drawing {
            DrawingState::ShapeDrag { start_x, start_y } => {
                let x = start_x.min(wmx);
                let y = start_y.min(wmy);
                let w = (wmx - start_x).abs();
                let h = (wmy - start_y).abs();
                ctx.stroke_rect(x, y, w, h);
            }
            DrawingState::LinePlacement { points } if !points.is_empty() => {
                ctx.begin_path();
                ctx.move_to(points[0].x, points[0].y);
                for p in &points[1..] {
                    ctx.line_to(p.x, p.y);
                }
                ctx.line_to(wmx, wmy);
                ctx.stroke();
            }
            DrawingState::FreehandDraw { points } if points.len() >= 2 => {
                ctx.begin_path();
                ctx.move_to(points[0].x, points[0].y);
                for p in &points[1..] {
                    ctx.line_to(p.x, p.y);
                }
                ctx.stroke();
            }
            _ => {}
        }

        // Reset line dash
        ctx.set_line_dash(&js_sys::Array::new()).unwrap();
    }
}

fn hit_test(elements: &[ElementData], wx: f64, wy: f64) -> Option<&ElementData> {
    // Iterate in reverse z-order (top-most first)
    for elem in elements.iter().rev() {
        match elem.kind {
            ElementKind::Rectangle | ElementKind::Text => {
                if wx >= elem.x && wx <= elem.x + elem.width
                    && wy >= elem.y && wy <= elem.y + elem.height
                {
                    return Some(elem);
                }
            }
            ElementKind::Ellipse => {
                let cx = elem.x + elem.width / 2.0;
                let cy = elem.y + elem.height / 2.0;
                let rx = elem.width / 2.0;
                let ry = elem.height / 2.0;
                if rx > 0.0 && ry > 0.0 {
                    let dx = (wx - cx) / rx;
                    let dy = (wy - cy) / ry;
                    if dx * dx + dy * dy <= 1.0 {
                        return Some(elem);
                    }
                }
            }
            ElementKind::Arrow | ElementKind::Line | ElementKind::Freehand => {
                // Distance to polyline
                let threshold = 8.0;
                for window in elem.points.windows(2) {
                    let dist = point_to_segment_dist(
                        wx, wy,
                        window[0].x, window[0].y,
                        window[1].x, window[1].y,
                    );
                    if dist <= threshold {
                        return Some(elem);
                    }
                }
            }
        }
    }
    None
}

fn point_to_segment_dist(px: f64, py: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> f64 {
    let dx = x2 - x1;
    let dy = y2 - y1;
    let len_sq = dx * dx + dy * dy;
    if len_sq == 0.0 {
        return ((px - x1).powi(2) + (py - y1).powi(2)).sqrt();
    }
    let t = ((px - x1) * dx + (py - y1) * dy) / len_sq;
    let t = t.clamp(0.0, 1.0);
    let proj_x = x1 + t * dx;
    let proj_y = y1 + t * dy;
    ((px - proj_x).powi(2) + (py - proj_y).powi(2)).sqrt()
}

// Placeholder functions — these will call SpacetimeDB reducers once stdb-client is wired up.
// For now, they create elements locally.
fn create_shape_element(state: &AppState, kind: ElementKind, x: f64, y: f64, w: f64, h: f64) {
    let id = generate_local_id();
    state.store.elements.update(|elems| {
        elems.insert(id, ElementData {
            id,
            room_id: state.store.current_room.get_untracked().unwrap_or(0),
            kind,
            x, y, width: w, height: h,
            rotation: 0.0,
            points: vec![],
            stroke_color: state.stroke_color.get_untracked(),
            fill_color: state.fill_color.get_untracked(),
            stroke_width: state.stroke_width.get_untracked(),
            opacity: shared::DEFAULT_OPACITY,
            font_size: shared::DEFAULT_FONT_SIZE,
            text_content: String::new(),
            z_index: elems.len() as i32,
        });
    });
    state.selected_ids.set(vec![id]);
}

fn create_line_element(state: &AppState, kind: ElementKind, points: Vec<Point>) {
    let id = generate_local_id();
    state.store.elements.update(|elems| {
        elems.insert(id, ElementData {
            id,
            room_id: state.store.current_room.get_untracked().unwrap_or(0),
            kind,
            x: 0.0, y: 0.0, width: 0.0, height: 0.0,
            rotation: 0.0,
            points,
            stroke_color: state.stroke_color.get_untracked(),
            fill_color: 0,
            stroke_width: state.stroke_width.get_untracked(),
            opacity: shared::DEFAULT_OPACITY,
            font_size: shared::DEFAULT_FONT_SIZE,
            text_content: String::new(),
            z_index: elems.len() as i32,
        });
    });
}

fn create_text_element(state: &AppState, x: f64, y: f64, text: String) {
    let id = generate_local_id();
    let font_size = shared::DEFAULT_FONT_SIZE;
    state.store.elements.update(|elems| {
        elems.insert(id, ElementData {
            id,
            room_id: state.store.current_room.get_untracked().unwrap_or(0),
            kind: ElementKind::Text,
            x, y,
            width: text.len() as f64 * font_size as f64 * 0.6,
            height: font_size as f64 * 1.2,
            rotation: 0.0,
            points: vec![],
            stroke_color: state.stroke_color.get_untracked(),
            fill_color: 0,
            stroke_width: state.stroke_width.get_untracked(),
            opacity: shared::DEFAULT_OPACITY,
            font_size,
            text_content: text,
            z_index: elems.len() as i32,
        });
    });
}

fn create_freehand_element(state: &AppState, points: Vec<Point>) {
    let id = generate_local_id();
    state.store.elements.update(|elems| {
        elems.insert(id, ElementData {
            id,
            room_id: state.store.current_room.get_untracked().unwrap_or(0),
            kind: ElementKind::Freehand,
            x: 0.0, y: 0.0, width: 0.0, height: 0.0,
            rotation: 0.0,
            points,
            stroke_color: state.stroke_color.get_untracked(),
            fill_color: 0,
            stroke_width: state.stroke_width.get_untracked(),
            opacity: shared::DEFAULT_OPACITY,
            font_size: shared::DEFAULT_FONT_SIZE,
            text_content: String::new(),
            z_index: elems.len() as i32,
        });
    });
}

fn generate_local_id() -> u64 {
    // Temporary local ID — will be replaced by server-assigned ID.
    // Use timestamp + random bits to avoid collisions.
    let window = web_sys::window().unwrap();
    let now = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * 1_000_000.0) as u64;
    now * 1_000_000 + random
}
```

- [ ] **Step 2: Wire tool handler into canvas mouse events**

Update `crates/app/src/canvas.rs` to use `ToolHandler`:
- Store a `Rc<RefCell<ToolHandler>>` in the canvas component
- Call `tool_handler.on_mouse_down/move/up` from mouse event handlers
- Call `tool_handler.render_preview` at the end of each frame

- [ ] **Step 3: Verify it builds**

Run: `cd crates/app && trunk build`
Expected: Builds successfully

- [ ] **Step 4: Commit**

```bash
git add crates/app/
git commit -m "feat: implement drawing tools — shape, line, text, freehand creation"
```

---

### Task 6.2: Export (PNG & SVG)

**Files:**
- Create: `crates/app/src/export.rs`
- Modify: `crates/app/src/commands.rs` (wire up `:w` and `:ws`)

- [ ] **Step 1: Implement PNG export**

`crates/app/src/export.rs`:
```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlCanvasElement, CanvasRenderingContext2d, Blob, Url};
use crate::state::AppState;
use crate::canvas::render_element;
use stdb_client::ElementData;

pub fn export_png(state: &AppState) {
    let elements = state.store.sorted_elements();
    if elements.is_empty() { return; }

    let (min_x, min_y, max_x, max_y) = bounding_box(&elements);
    let padding = 20.0;
    let w = (max_x - min_x + padding * 2.0) as u32;
    let h = (max_y - min_y + padding * 2.0) as u32;

    let document = web_sys::window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .create_element("canvas").unwrap()
        .dyn_into().unwrap();
    canvas.set_width(w);
    canvas.set_height(h);

    let ctx: CanvasRenderingContext2d = canvas
        .get_context("2d").unwrap().unwrap()
        .dyn_into().unwrap();

    // White background
    ctx.set_fill_style_str("white");
    ctx.fill_rect(0.0, 0.0, w as f64, h as f64);

    // Offset to fit all elements
    ctx.translate(padding - min_x, padding - min_y).unwrap();

    for elem in &elements {
        render_element(&ctx, elem);
    }

    // Trigger download via blob
    let closure = Closure::once(move |blob: JsValue| {
        let blob: Blob = blob.dyn_into().unwrap();
        let url = Url::create_object_url_with_blob(&blob).unwrap();
        trigger_download(&url, "drawing.png");
        Url::revoke_object_url(&url).unwrap();
    });

    canvas.to_blob(closure.as_ref().unchecked_ref()).unwrap();
    closure.forget();
}

pub fn export_svg(state: &AppState) {
    let elements = state.store.sorted_elements();
    if elements.is_empty() { return; }

    let (min_x, min_y, max_x, max_y) = bounding_box(&elements);
    let padding = 20.0;
    let w = max_x - min_x + padding * 2.0;
    let h = max_y - min_y + padding * 2.0;

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{} {} {} {}" width="{}" height="{}">"#,
        min_x - padding, min_y - padding, w, h, w, h
    );
    svg.push_str(r#"<rect width="100%" height="100%" fill="white"/>"#);

    for elem in &elements {
        svg.push_str(&element_to_svg(elem));
    }

    svg.push_str("</svg>");

    let blob = Blob::new_with_str_sequence_and_options(
        &js_sys::Array::of1(&JsValue::from_str(&svg)),
        web_sys::BlobPropertyBag::new().type_("image/svg+xml"),
    ).unwrap();

    let url = Url::create_object_url_with_blob(&blob).unwrap();
    trigger_download(&url, "drawing.svg");
    Url::revoke_object_url(&url).unwrap();
}

fn element_to_svg(elem: &ElementData) -> String {
    let stroke = color_u32_to_hex(elem.stroke_color);
    let fill = color_u32_to_hex(elem.fill_color);
    let sw = elem.stroke_width;

    match elem.kind {
        shared::ElementKind::Rectangle => {
            format!(
                r#"<rect x="{}" y="{}" width="{}" height="{}" stroke="{}" fill="{}" stroke-width="{}" opacity="{}"/>"#,
                elem.x, elem.y, elem.width, elem.height, stroke, fill, sw, elem.opacity
            )
        }
        shared::ElementKind::Ellipse => {
            format!(
                r#"<ellipse cx="{}" cy="{}" rx="{}" ry="{}" stroke="{}" fill="{}" stroke-width="{}" opacity="{}"/>"#,
                elem.x + elem.width / 2.0, elem.y + elem.height / 2.0,
                elem.width / 2.0, elem.height / 2.0,
                stroke, fill, sw, elem.opacity
            )
        }
        shared::ElementKind::Arrow | shared::ElementKind::Line | shared::ElementKind::Freehand => {
            if elem.points.len() < 2 { return String::new(); }
            let mut d = format!("M{},{}", elem.points[0].x, elem.points[0].y);
            for p in &elem.points[1..] {
                d.push_str(&format!(" L{},{}", p.x, p.y));
            }
            format!(
                r#"<path d="{}" stroke="{}" fill="none" stroke-width="{}" opacity="{}"/>"#,
                d, stroke, sw, elem.opacity
            )
        }
        shared::ElementKind::Text => {
            format!(
                r#"<text x="{}" y="{}" font-family="monospace" font-size="{}" fill="{}" opacity="{}">{}</text>"#,
                elem.x, elem.y + elem.font_size as f64,
                elem.font_size, stroke, elem.opacity,
                html_escape(&elem.text_content)
            )
        }
    }
}

fn bounding_box(elements: &[ElementData]) -> (f64, f64, f64, f64) {
    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for elem in elements {
        min_x = min_x.min(elem.x);
        min_y = min_y.min(elem.y);
        max_x = max_x.max(elem.x + elem.width);
        max_y = max_y.max(elem.y + elem.height);
        for p in &elem.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
    }
    (min_x, min_y, max_x, max_y)
}

fn trigger_download(url: &str, filename: &str) {
    let document = web_sys::window().unwrap().document().unwrap();
    let a = document.create_element("a").unwrap();
    a.set_attribute("href", url).unwrap();
    a.set_attribute("download", filename).unwrap();
    a.dyn_ref::<web_sys::HtmlElement>().unwrap().click();
}

fn color_u32_to_hex(color: u32) -> String {
    let r = (color >> 24) & 0xFF;
    let g = (color >> 16) & 0xFF;
    let b = (color >> 8) & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}
```

- [ ] **Step 2: Wire exports into command handler**

Update `crates/app/src/commands.rs`:
```rust
["w"] => export::export_png(state),
["ws"] => export::export_svg(state),
```

- [ ] **Step 3: Verify it builds**

Run: `cd crates/app && trunk build`
Expected: Builds

- [ ] **Step 4: Commit**

```bash
git add crates/app/
git commit -m "feat: implement PNG and SVG export via :w and :ws commands"
```

---

## Phase 7: Service Worker

### Task 7.1: Implement Service Worker

**Files:**
- Modify: `crates/service-worker/src/lib.rs`
- Create: `crates/service-worker/src/cache.rs`
- Create: `sw-loader.js` (minimal JS required to register the WASM service worker — this is browser infrastructure, not application code)

- [ ] **Step 1: Implement app shell caching service worker**

`crates/service-worker/src/lib.rs`:
```rust
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{ServiceWorkerGlobalScope, FetchEvent, Cache, Request, Response};
use js_sys::Promise;

const CACHE_NAME: &str = "collaborate-v1";
const STATIC_ASSETS: &[&str] = &[
    "/",
    "/index.html",
    "/app.wasm",
    "/app.js",
];

#[wasm_bindgen(start)]
pub fn start() {
    let global: ServiceWorkerGlobalScope = js_sys::global().unchecked_into();

    // Install event — cache static assets
    let install_cb = Closure::wrap(Box::new(move |event: web_sys::ExtendableEvent| {
        let promise = wasm_bindgen_futures::future_to_promise(async {
            let cache = open_cache(CACHE_NAME).await?;
            for url in STATIC_ASSETS {
                let request = Request::new_with_str(url)
                    .map_err(|e| JsValue::from_str(&format!("{:?}", e)))?;
                let response = fetch_request(&request).await?;
                cache_put(&cache, &request, &response).await?;
            }
            Ok(JsValue::undefined())
        });
        event.wait_until(&promise).unwrap();
    }) as Box<dyn FnMut(web_sys::ExtendableEvent)>);
    global.add_event_listener_with_callback("install", install_cb.as_ref().unchecked_ref()).unwrap();
    install_cb.forget();

    // Fetch event — cache-first for static, network-first for API
    let fetch_cb = Closure::wrap(Box::new(move |event: FetchEvent| {
        let request = event.request();
        let url = request.url();

        if url.contains("/ws") || url.contains("/v1/") {
            // Network-only for WebSocket and API
            return;
        }

        let promise = wasm_bindgen_futures::future_to_promise(async move {
            // Try cache first
            if let Ok(cached) = cache_match(&request).await {
                return Ok(cached);
            }
            // Fallback to network
            let response = fetch_request(&request).await?;
            // Cache the response for next time
            let cache = open_cache(CACHE_NAME).await?;
            cache_put(&cache, &request, &response.clone().unwrap()).await?;
            Ok(response.into())
        });
        event.respond_with(&promise).unwrap();
    }) as Box<dyn FnMut(FetchEvent)>);
    global.add_event_listener_with_callback("fetch", fetch_cb.as_ref().unchecked_ref()).unwrap();
    fetch_cb.forget();
}

async fn open_cache(name: &str) -> Result<Cache, JsValue> {
    let global: ServiceWorkerGlobalScope = js_sys::global().unchecked_into();
    let caches = global.caches()?;
    let promise = caches.open(name);
    let cache = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(cache.unchecked_into())
}

async fn cache_match(request: &Request) -> Result<JsValue, JsValue> {
    let global: ServiceWorkerGlobalScope = js_sys::global().unchecked_into();
    let caches = global.caches()?;
    let promise = caches.match_with_request(request);
    wasm_bindgen_futures::JsFuture::from(promise).await
}

async fn cache_put(cache: &Cache, request: &Request, response: &Response) -> Result<(), JsValue> {
    let promise = cache.put_with_request(request, response);
    wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(())
}

async fn fetch_request(request: &Request) -> Result<Response, JsValue> {
    let global: ServiceWorkerGlobalScope = js_sys::global().unchecked_into();
    let promise = global.fetch_with_request(request);
    let response = wasm_bindgen_futures::JsFuture::from(promise).await?;
    Ok(response.unchecked_into())
}
```

- [ ] **Step 2: Create sw-loader.js (minimal browser registration)**

`sw-loader.js` — this is not application code, it's browser infrastructure required to load a WASM service worker:
```javascript
// Minimal service worker registration — loads the Rust WASM service worker.
// This is required by browser APIs (service workers must be JS entry points).
importScripts('./sw/service_worker.js');
wasm_bindgen('./sw/service_worker_bg.wasm');
```

- [ ] **Step 3: Add service worker registration to app**

Add to `crates/app/src/main.rs` at the end of `main()`:
```rust
// Register service worker
wasm_bindgen_futures::spawn_local(async {
    let window = web_sys::window().unwrap();
    if let Ok(sw) = window.navigator().service_worker() {
        if let Ok(promise) = sw.register("/sw-loader.js") {
            let _ = wasm_bindgen_futures::JsFuture::from(promise).await;
            log::info!("Service worker registered");
        }
    }
});
```

- [ ] **Step 4: Verify service worker crate builds**

Run: `wasm-pack build crates/service-worker --target no-modules --out-dir ../../dist/sw`
Expected: Produces `dist/sw/service_worker.js` and `service_worker_bg.wasm`

- [ ] **Step 5: Commit**

```bash
git add crates/service-worker/ sw-loader.js
git commit -m "feat: implement Rust WASM service worker with app shell caching"
```

---

## Phase 8: Integration & Polish

### Task 8.1: Wire SpacetimeDB Connection to App

**Files:**
- Modify: `crates/stdb-client/src/connection.rs`
- Modify: `crates/app/src/main.rs`
- Modify: `crates/app/src/commands.rs`
- Modify: `crates/app/src/tools.rs`

- [ ] **Step 1: Implement the stdb-client connection module**

`crates/stdb-client/src/connection.rs`:
```rust
use leptos::prelude::*;
use crate::signals::{StdbStore, ElementData, CursorData};
use shared::{ElementKind, Point, encode_points, decode_points};
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

/// Handle to an active SpacetimeDB connection.
/// Wraps the WebSocket and provides methods to call reducers.
#[derive(Clone)]
pub struct StdbConnection {
    ws: Rc<RefCell<Option<web_sys::WebSocket>>>,
    store: StdbStore,
}

impl StdbConnection {
    /// Connect to SpacetimeDB and bind row-change callbacks to the store's signals.
    ///
    /// If `spacetimedb-sdk` with `browser` feature compiles for wasm32, use it:
    /// ```rust
    /// use spacetimedb_sdk::{DbConnection, subscribe_owned};
    /// let conn = DbConnection::builder()
    ///     .with_uri(host)
    ///     .with_module_name(db_name)
    ///     .on_connect(|_conn, identity, token| { /* save identity */ })
    ///     .on_disconnect(|_conn, _err| { store.connected.set(false); })
    ///     .build()
    ///     .await?;
    /// conn.subscribe(&["SELECT * FROM element WHERE room_id = ..."]);
    /// ```
    ///
    /// If the SDK does NOT compile for wasm32, use gloo-net raw WebSocket:
    pub async fn connect(host: &str, db_name: &str, store: StdbStore) -> Result<Self, String> {
        let url = format!("{}/v1/database/{}/subscribe?compression=None", host, db_name);

        let ws = web_sys::WebSocket::new_with_str(&url, "v2.bsatn.spacetimedb")
            .map_err(|e| format!("WebSocket creation failed: {:?}", e))?;
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        let ws_rc = Rc::new(RefCell::new(Some(ws.clone())));
        let conn = Self { ws: ws_rc, store: store.clone() };

        // On open: mark connected, subscribe to room data
        let store_open = store.clone();
        let ws_open = ws.clone();
        let onopen = Closure::<dyn FnMut()>::new(move || {
            store_open.connected.set(true);
            log::info!("SpacetimeDB connected");
            // Send Subscribe message for the current room
            // The actual BSATN encoding depends on whether we use the SDK or raw protocol.
            // With raw protocol: encode ClientMessage::Subscribe { request_id, query_set_id, query_strings }
            // using spacetimedb_client_api_messages and bsatn::to_vec, then ws.send_with_u8_array(&bytes).
        });
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        onopen.forget();

        // On message: decode ServerMessage, update store signals
        let store_msg = store.clone();
        let onmessage = Closure::<dyn FnMut(_)>::new(move |ev: web_sys::MessageEvent| {
            if let Ok(buf) = ev.data().dyn_into::<js_sys::ArrayBuffer>() {
                let bytes = js_sys::Uint8Array::new(&buf).to_vec();
                // First byte is compression tag (0x00 = None)
                if bytes.is_empty() { return; }
                let _compression_tag = bytes[0];
                let payload = &bytes[1..];
                // Decode: bsatn::from_slice::<ServerMessage>(payload)
                // Then match on the ServerMessage variant:
                // - InitialConnection: save identity + token
                // - SubscribeApplied: bulk insert rows into store.elements
                // - TransactionUpdate: apply inserts/deletes to store.elements
                // - ReducerResult: check for errors, apply embedded TransactionUpdate
                //
                // For each inserted/updated element row, decode the BSATN row bytes
                // into the Element table struct fields, convert to ElementData,
                // and update store.elements signal:
                //
                // store_msg.elements.update(|elems| {
                //     elems.insert(elem_data.id, elem_data);
                // });
                //
                // For deleted rows, remove from the HashMap:
                // store_msg.elements.update(|elems| { elems.remove(&id); });
                log::debug!("Received {} bytes from SpacetimeDB", bytes.len());
            }
        });
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        onmessage.forget();

        // On close: mark disconnected, attempt reconnect
        let store_close = store.clone();
        let onclose = Closure::<dyn FnMut()>::new(move || {
            store_close.connected.set(false);
            log::warn!("SpacetimeDB disconnected");
            // TODO: exponential backoff reconnect
        });
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));
        onclose.forget();

        Ok(conn)
    }

    /// Call a SpacetimeDB reducer by name with BSATN-encoded arguments.
    pub fn call_reducer(&self, name: &str, args: Vec<u8>) {
        if let Some(ws) = self.ws.borrow().as_ref() {
            // Encode ClientMessage::CallReducer { request_id, flags: 0, reducer: name, args }
            // let msg = ClientMessage::CallReducer(CallReducer { ... });
            // let bytes = bsatn::to_vec(&msg).unwrap();
            // ws.send_with_u8_array(&bytes).unwrap();
            log::debug!("Calling reducer: {}", name);
        }
    }

    /// Subscribe to queries for a room.
    pub fn subscribe_room(&self, room_id: u64) {
        let queries = [
            format!("SELECT * FROM element WHERE room_id = {} AND deleted = false", room_id),
            format!("SELECT * FROM cursor WHERE room_id = {}", room_id),
        ];
        // Encode ClientMessage::Subscribe { request_id: 1, query_set_id: { id: 1 }, query_strings: queries }
        // Send over WebSocket
        log::info!("Subscribing to room {}", room_id);
    }
}
```

The implementer should:
1. First try `cargo check -p stdb-client --target wasm32-unknown-unknown` with `spacetimedb-sdk = { workspace = true, features = ["browser"] }`.
2. If the SDK compiles, replace the raw WebSocket code above with the SDK's `DbConnection::builder()` API and register `on_insert`/`on_delete` callbacks per table that update the `StdbStore` signals.
3. If the SDK does NOT compile, use the raw WebSocket approach above with `spacetimedb-client-api-messages` for message types and `spacetimedb-lib::bsatn` for serialization. The comments in the code above describe exactly which message types to encode/decode and how to update the store.

- [ ] **Step 2: Replace local element creation with reducer calls**

Update `crates/app/src/tools.rs` — change `create_shape_element` and similar functions to use optimistic local insert + reducer call:

```rust
use stdb_client::connection::StdbConnection;

fn create_shape_element(state: &AppState, kind: ElementKind, x: f64, y: f64, w: f64, h: f64) {
    // 1. Optimistic local insert with a temporary ID
    let temp_id = generate_local_id();
    let elem = ElementData {
        id: temp_id,
        room_id: state.store.current_room.get_untracked().unwrap_or(0),
        kind,
        x, y, width: w, height: h,
        rotation: 0.0,
        points: vec![],
        stroke_color: state.stroke_color.get_untracked(),
        fill_color: state.fill_color.get_untracked(),
        stroke_width: state.stroke_width.get_untracked(),
        opacity: shared::DEFAULT_OPACITY,
        font_size: shared::DEFAULT_FONT_SIZE,
        text_content: String::new(),
        z_index: state.store.elements.with_untracked(|e| e.len() as i32),
    };
    state.store.elements.update(|elems| { elems.insert(temp_id, elem); });
    state.selected_ids.set(vec![temp_id]);

    // 2. Call reducer — when the server confirms, the subscription callback
    //    will insert the real element (with server-assigned ID) into the store.
    //    We reconcile by: the subscription callback checks if an element with
    //    matching (room_id, kind, x, y, width, height) and a temp_id exists,
    //    removes the temp entry, and inserts the server-assigned one.
    if let Some(conn) = state.connection.get_untracked() {
        conn.call_reducer("create_element", encode_create_element_args(
            state.store.current_room.get_untracked().unwrap_or(0),
            kind, x, y, w, h,
            state.stroke_color.get_untracked(),
            state.fill_color.get_untracked(),
            state.stroke_width.get_untracked(),
            shared::DEFAULT_OPACITY,
            shared::DEFAULT_FONT_SIZE,
            String::new(),
            vec![],
        ));
    }
}

// Similar pattern for create_line_element, create_text_element, create_freehand_element.
// Each: (1) insert optimistic local element, (2) call reducer, (3) subscription callback reconciles.
```

Add a `connection: RwSignal<Option<StdbConnection>>` field to `AppState` in `state.rs`:
```rust
pub connection: RwSignal<Option<stdb_client::connection::StdbConnection>>,
```

- [ ] **Step 3: Wire connection in main.rs**

Add SpacetimeDB connection on app startup in `crates/app/src/main.rs`, inside the `App` component:
```rust
// Connect to SpacetimeDB
{
    let state = state.clone();
    wasm_bindgen_futures::spawn_local(async move {
        match stdb_client::connection::StdbConnection::connect(
            "ws://localhost:3000",
            "collaborate",
            state.store.clone(),
        ).await {
            Ok(conn) => {
                state.connection.set(Some(conn.clone()));
                // Auto-join or create a room
                conn.call_reducer("create_room", /* encode "default" room name */);
                conn.subscribe_room(1); // subscribe to room 1
                state.store.current_room.set(Some(1));
                log::info!("Connected to SpacetimeDB");
            }
            Err(e) => log::error!("Failed to connect to SpacetimeDB: {}", e),
        }
    });
}
```

- [ ] **Step 4: Verify full app builds and runs**

Run: `cd crates/app && trunk serve --open`
Expected: App loads in browser, shows canvas with vim mode indicator

- [ ] **Step 5: Commit**

```bash
git add -A
git commit -m "feat: wire SpacetimeDB connection, replace local state with server sync"
```

---

### Task 8.2: End-to-End Test

**Files:**
- Create: `tests/e2e.md` (manual test script)

- [ ] **Step 1: Write manual E2E test script**

`tests/e2e.md`:
```markdown
# E2E Test Script

## Prerequisites
1. SpacetimeDB running locally: `spacetime start`
2. Server module published: `spacetime publish collaborate crates/server`
3. App running: `cd crates/app && trunk serve`

## Test Cases

### 1. Canvas Loads
- [ ] Open http://localhost:8080
- [ ] See "NORMAL" mode indicator in top bar
- [ ] See "Offline" or "Connected" in top bar
- [ ] Canvas fills the viewport

### 2. Vim Mode - Shape Creation
- [ ] Press `r` — mode changes to "INSERT"
- [ ] Click and drag on canvas — rectangle appears
- [ ] Press `Escape` — mode returns to "NORMAL"
- [ ] Press `e` then click+drag — ellipse appears

### 3. Vim Mode - Navigation
- [ ] Select a shape (click it)
- [ ] Press `h` — shape moves left
- [ ] Press `j` — shape moves down
- [ ] Press `k` — shape moves up
- [ ] Press `l` — shape moves right

### 4. Vim Mode - Delete/Copy/Paste
- [ ] Select a shape, press `dd` — shape deleted
- [ ] Press `u` — shape restored (undo)
- [ ] Select a shape, press `yy` then `p` — shape duplicated

### 5. Vim Mode - Commands
- [ ] Press `:` — command bar activates
- [ ] Type `color #ff0000` + Enter — stroke color changes to red
- [ ] Draw a new shape — it appears red
- [ ] Press `:w` + Enter — PNG downloads

### 6. Multi-user (if SpacetimeDB connected)
- [ ] Open two browser tabs
- [ ] Draw in tab 1 — appears in tab 2
- [ ] Move mouse in tab 1 — cursor visible in tab 2

### 7. Freehand Drawing
- [ ] Press `f` — enter freehand mode
- [ ] Click and drag — freehand stroke appears
- [ ] Press `Escape` — return to normal

### 8. Text
- [ ] Press `t` — enter text mode
- [ ] Click canvas — cursor appears
- [ ] Type text — text appears on canvas
- [ ] Press `Escape` — text committed

### 9. Zoom
- [ ] Scroll wheel up — canvas zooms in
- [ ] Scroll wheel down — canvas zooms out
- [ ] Bottom bar shows zoom percentage
```

- [ ] **Step 2: Commit**

```bash
git add tests/
git commit -m "docs: add manual E2E test script"
```

---

### Task 8.3: Build Script & README

**Files:**
- Create: `Makefile`
- Create: `README.md`

- [ ] **Step 1: Create Makefile**

```makefile
.PHONY: dev build test server publish clean

# Development
dev:
	cd crates/app && trunk serve --open

# Build all
build: build-server build-app build-sw

build-server:
	spacetime build crates/server

build-app:
	cd crates/app && trunk build --release

build-sw:
	wasm-pack build crates/service-worker --target no-modules --out-dir ../../dist/sw

# Tests
test:
	cargo test --workspace

test-shared:
	cargo test -p shared

# SpacetimeDB
server:
	spacetime start

publish:
	spacetime publish collaborate crates/server

# Clean
clean:
	cargo clean
	rm -rf dist/
	rm -rf crates/app/dist/
```

- [ ] **Step 2: Create README.md**

Brief README with: project description, prerequisites (Rust, Trunk, wasm-pack, SpacetimeDB CLI), quickstart commands, vim keybinding reference table.

- [ ] **Step 3: Commit**

```bash
git add Makefile README.md
git commit -m "docs: add Makefile and README with quickstart guide"
```
