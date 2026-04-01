# Collaborate

A proof-of-concept Excalidraw clone with real-time collaboration and vim keybindings, built with a pure Rust stack.

## Tech Stack

- **SpacetimeDB** — real-time state synchronization
- **Leptos 0.7** — reactive UI framework (CSR)
- **HTML5 Canvas** — 2D rendering via web-sys
- **Rust/WASM** — service worker for offline support
- **Custom SpacetimeDB client** — WASM-compatible WebSocket client

## Quick Start

### Prerequisites

- Rust (stable) with `wasm32-unknown-unknown` target
- [Trunk](https://trunkrs.dev/) — `cargo install trunk`
- [wasm-pack](https://rustwasm.github.io/wasm-pack/) — `cargo install wasm-pack`
- [SpacetimeDB CLI](https://spacetimedb.com/install) — for server module

### Run (local only, no server)

```bash
make dev
```

The app works offline with local state. Shapes, vim commands, and export all work without SpacetimeDB.

### Run (with collaboration)

```bash
# Terminal 1: Start SpacetimeDB
make server

# Terminal 2: Publish the server module
make publish

# Terminal 3: Start the app
make dev
```

Open multiple tabs to test real-time collaboration.

## Vim Keybindings

### Normal Mode

| Key | Action |
|-----|--------|
| `r` | Create rectangle |
| `e` | Create ellipse |
| `a` | Create arrow |
| `L` | Create line |
| `t` | Create text |
| `f` | Freehand draw |
| `h/j/k/l` | Move selected (10px) |
| `Shift+h/j/k/l` | Move selected (1px) |
| `dd` | Delete selected |
| `yy` | Copy selected |
| `p` | Paste |
| `u` | Undo |
| `Ctrl+r` | Redo |
| `v` | Visual (multi-select) mode |
| `:` | Command mode |
| `Esc` | Deselect |

### Command Mode

| Command | Action |
|---------|--------|
| `:w` | Export PNG |
| `:ws` | Export SVG |
| `:color #hex` | Set stroke color |
| `:fill #hex` | Set fill color |
| `:stroke N` | Set stroke width |

## Project Structure

```
crates/
  shared/          — Types shared between server and client
  server/          — SpacetimeDB server module (tables + reducers)
  stdb-client/     — Custom WASM-compatible SpacetimeDB client
  app/             — Leptos frontend (canvas, vim mode, tools)
  service-worker/  — Rust WASM service worker (offline caching)
```

## Tests

```bash
make test
```
