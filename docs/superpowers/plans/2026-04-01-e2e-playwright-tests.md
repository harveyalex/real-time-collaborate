# Playwright E2E Tests Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Automated Playwright tests that verify two browsers can draw and see each other's elements in real-time via SpacetimeDB.

**Architecture:** Three tasks: (1) implement BSATN row decoding in sync.rs so server-pushed elements populate the store, (2) add debug-only test hooks exposing element/cursor counts to `window`, (3) create Playwright test infrastructure with global setup/teardown and 4 test specs.

**Tech Stack:** Playwright (TypeScript), SpacetimeDB CLI, Trunk, spacetimedb-lib BSATN

**Spec:** `docs/superpowers/specs/2026-04-01-e2e-playwright-design.md`

---

## Task 1: BSATN Row Decoding in sync.rs

**Files:**
- Create: `crates/stdb-client/src/decode.rs`
- Modify: `crates/stdb-client/src/lib.rs`
- Modify: `crates/app/src/sync.rs`
- Modify: `crates/stdb-client/Cargo.toml`

This is the prerequisite — without it, elements created in one browser never appear in another's store.

- [ ] **Step 1: Create decode module with row decoding functions**

`crates/stdb-client/src/decode.rs`:

The BSATN row data for each table is a flat byte buffer where each row is encoded as a BSATN `ProductValue` — the fields are serialized in order matching the table struct definition. We decode by manually reading each field in order using `spacetimedb_lib::bsatn::Deserializer`.

```rust
//! Decode BSATN row bytes from SpacetimeDB wire protocol into client-side types.

use crate::signals::{ElementData, CursorData};
use shared::{ElementKind, Point, decode_points};
use spacetimedb_lib::bsatn;
use spacetimedb_sats::de::Deserialize;

/// Decode a single Element row from BSATN bytes.
///
/// The fields must be decoded in the exact order they appear in the server's
/// `#[spacetimedb::table]` struct definition:
///   id, room_id, kind, x, y, width, height, rotation, points,
///   stroke_color, fill_color, stroke_width, opacity, font_size,
///   text_content, z_index, version, updated_by, deleted
pub fn decode_element(row_bytes: &[u8]) -> Result<Option<ElementData>, String> {
    let mut reader = &row_bytes[..];

    let id: u64 = bsatn::from_reader(&mut reader).map_err(|e| format!("id: {e}"))?;
    let room_id: u64 = bsatn::from_reader(&mut reader).map_err(|e| format!("room_id: {e}"))?;
    let kind: ElementKind = bsatn::from_reader(&mut reader).map_err(|e| format!("kind: {e}"))?;
    let x: f64 = bsatn::from_reader(&mut reader).map_err(|e| format!("x: {e}"))?;
    let y: f64 = bsatn::from_reader(&mut reader).map_err(|e| format!("y: {e}"))?;
    let width: f64 = bsatn::from_reader(&mut reader).map_err(|e| format!("width: {e}"))?;
    let height: f64 = bsatn::from_reader(&mut reader).map_err(|e| format!("height: {e}"))?;
    let rotation: f64 = bsatn::from_reader(&mut reader).map_err(|e| format!("rotation: {e}"))?;
    let points_raw: Vec<u8> = bsatn::from_reader(&mut reader).map_err(|e| format!("points: {e}"))?;
    let stroke_color: u32 = bsatn::from_reader(&mut reader).map_err(|e| format!("stroke_color: {e}"))?;
    let fill_color: u32 = bsatn::from_reader(&mut reader).map_err(|e| format!("fill_color: {e}"))?;
    let stroke_width: f32 = bsatn::from_reader(&mut reader).map_err(|e| format!("stroke_width: {e}"))?;
    let opacity: f32 = bsatn::from_reader(&mut reader).map_err(|e| format!("opacity: {e}"))?;
    let font_size: f32 = bsatn::from_reader(&mut reader).map_err(|e| format!("font_size: {e}"))?;
    let text_content: String = bsatn::from_reader(&mut reader).map_err(|e| format!("text_content: {e}"))?;
    let z_index: i32 = bsatn::from_reader(&mut reader).map_err(|e| format!("z_index: {e}"))?;
    let _version: u64 = bsatn::from_reader(&mut reader).map_err(|e| format!("version: {e}"))?;
    // updated_by is an Identity = 32-byte value (U256 internally)
    let _updated_by_bytes: [u8; 32] = bsatn::from_reader(&mut reader).map_err(|e| format!("updated_by: {e}"))?;
    let deleted: bool = bsatn::from_reader(&mut reader).map_err(|e| format!("deleted: {e}"))?;

    // Skip deleted elements
    if deleted {
        return Ok(None);
    }

    let points = if points_raw.is_empty() {
        vec![]
    } else {
        decode_points(&points_raw).unwrap_or_default()
    };

    Ok(Some(ElementData {
        id,
        room_id,
        kind,
        x,
        y,
        width,
        height,
        rotation,
        points,
        stroke_color,
        fill_color,
        stroke_width,
        opacity,
        font_size,
        text_content,
        z_index,
    }))
}

/// Decode a single Cursor row from BSATN bytes.
///
/// Field order: user_id (Identity), room_id, x, y, name, color
pub fn decode_cursor(row_bytes: &[u8]) -> Result<(String, CursorData), String> {
    let mut reader = &row_bytes[..];

    let user_id_bytes: [u8; 32] = bsatn::from_reader(&mut reader).map_err(|e| format!("user_id: {e}"))?;
    let user_id_hex = hex::encode(user_id_bytes);
    let _room_id: u64 = bsatn::from_reader(&mut reader).map_err(|e| format!("room_id: {e}"))?;
    let x: f64 = bsatn::from_reader(&mut reader).map_err(|e| format!("x: {e}"))?;
    let y: f64 = bsatn::from_reader(&mut reader).map_err(|e| format!("y: {e}"))?;
    let name: String = bsatn::from_reader(&mut reader).map_err(|e| format!("name: {e}"))?;
    let color: u32 = bsatn::from_reader(&mut reader).map_err(|e| format!("color: {e}"))?;

    Ok((user_id_hex, CursorData { x, y, name, color }))
}

/// Extract the primary key (id: u64) from an Element row's BSATN bytes.
/// Only reads the first 8 bytes.
pub fn decode_element_id(row_bytes: &[u8]) -> Result<u64, String> {
    let mut reader = &row_bytes[..];
    bsatn::from_reader(&mut reader).map_err(|e| format!("element id: {e}"))
}

/// Extract the primary key (user_id: Identity) from a Cursor row's BSATN bytes.
pub fn decode_cursor_key(row_bytes: &[u8]) -> Result<String, String> {
    let mut reader = &row_bytes[..];
    let bytes: [u8; 32] = bsatn::from_reader(&mut reader).map_err(|e| format!("cursor key: {e}"))?;
    Ok(hex::encode(bytes))
}
```

- [ ] **Step 2: Add hex dependency to stdb-client Cargo.toml**

Add to `crates/stdb-client/Cargo.toml`:
```toml
hex = "0.4"
```

- [ ] **Step 3: Update stdb-client lib.rs**

```rust
pub mod connection;
pub mod signals;
pub mod decode;

pub use signals::{StdbStore, ElementData, CursorData};
```

- [ ] **Step 4: Verify decode module compiles**

Run: `cargo check -p stdb-client --target wasm32-unknown-unknown`
Expected: Compiles

Note: `bsatn::from_reader` takes a `&mut impl Read`. `&[u8]` implements `Read`. This should work, but if `bsatn::from_reader` doesn't exist in the public API, use `bsatn::from_slice` on sub-slices instead (requiring manual offset tracking). The implementer should check what `spacetimedb_lib::bsatn` exports and adapt.

If `Identity` is not simply `[u8; 32]` in BSATN, the implementer should check how Identity is serialized (it may be a wrapper struct with a `__identity__` field wrapping a `U256`). Adjust deserialization accordingly.

- [ ] **Step 5: Implement event handler with row decoding in sync.rs**

Replace the stub TODOs in `crates/app/src/sync.rs`:

```rust
use leptos::prelude::{Set, Update};
use stdb_client::StdbStore;
use stdb_client::connection::{StdbConnection, ServerEvent};
use stdb_client::decode;
use spacetimedb_client_api_messages::websocket::common::RowListLen;

pub fn setup_event_handler(conn: &StdbConnection, store: StdbStore) {
    conn.on_event(move |event| {
        match event {
            ServerEvent::Connected { identity, .. } => {
                store.connected.set(true);
                store.my_identity.set(Some(identity.to_string()));
                log::info!("Connected as {}", identity);
            }
            ServerEvent::Disconnected { reason } => {
                store.connected.set(false);
                log::warn!("Disconnected: {}", reason);
            }
            ServerEvent::SubscribeApplied { rows, .. } => {
                for table_rows in rows.tables.iter() {
                    let table_name = table_rows.table.as_ref();
                    let row_list = &table_rows.rows;
                    let count = row_list.len();
                    log::info!("SubscribeApplied: table={}, rows={}", table_name, count);

                    match table_name {
                        "element" => {
                            store.elements.update(|elems| {
                                for i in 0..count {
                                    if let Some(bytes) = row_list.get(i) {
                                        match decode::decode_element(&bytes) {
                                            Ok(Some(elem)) => { elems.insert(elem.id, elem); }
                                            Ok(None) => {} // deleted
                                            Err(e) => log::warn!("decode element: {}", e),
                                        }
                                    }
                                }
                            });
                        }
                        "cursor" => {
                            store.cursors.update(|cursors| {
                                for i in 0..count {
                                    if let Some(bytes) = row_list.get(i) {
                                        match decode::decode_cursor(&bytes) {
                                            Ok((key, cursor)) => { cursors.insert(key, cursor); }
                                            Err(e) => log::warn!("decode cursor: {}", e),
                                        }
                                    }
                                }
                            });
                        }
                        _ => log::debug!("Ignoring table: {}", table_name),
                    }
                }
            }
            ServerEvent::TransactionUpdate { updates } => {
                for qs_update in updates.query_sets.iter() {
                    for table_update in qs_update.tables.iter() {
                        let table_name = table_update.table_name.as_ref();
                        for update_rows in table_update.rows.iter() {
                            use spacetimedb_client_api_messages::websocket::v2::TableUpdateRows;
                            if let TableUpdateRows::PersistentTable(persistent) = update_rows {
                                match table_name {
                                    "element" => {
                                        // Process deletes first
                                        let del_count = persistent.deletes.len();
                                        if del_count > 0 {
                                            store.elements.update(|elems| {
                                                for i in 0..del_count {
                                                    if let Some(bytes) = persistent.deletes.get(i) {
                                                        if let Ok(id) = decode::decode_element_id(&bytes) {
                                                            elems.remove(&id);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        // Then inserts
                                        let ins_count = persistent.inserts.len();
                                        if ins_count > 0 {
                                            store.elements.update(|elems| {
                                                for i in 0..ins_count {
                                                    if let Some(bytes) = persistent.inserts.get(i) {
                                                        match decode::decode_element(&bytes) {
                                                            Ok(Some(elem)) => { elems.insert(elem.id, elem); }
                                                            Ok(None) => {} // deleted flag
                                                            Err(e) => log::warn!("decode element insert: {}", e),
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    "cursor" => {
                                        let del_count = persistent.deletes.len();
                                        if del_count > 0 {
                                            store.cursors.update(|cursors| {
                                                for i in 0..del_count {
                                                    if let Some(bytes) = persistent.deletes.get(i) {
                                                        if let Ok(key) = decode::decode_cursor_key(&bytes) {
                                                            cursors.remove(&key);
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                        let ins_count = persistent.inserts.len();
                                        if ins_count > 0 {
                                            store.cursors.update(|cursors| {
                                                for i in 0..ins_count {
                                                    if let Some(bytes) = persistent.inserts.get(i) {
                                                        match decode::decode_cursor(&bytes) {
                                                            Ok((key, cursor)) => { cursors.insert(key, cursor); }
                                                            Err(e) => log::warn!("decode cursor insert: {}", e),
                                                        }
                                                    }
                                                }
                                            });
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                }
            }
            ServerEvent::ReducerResult { request_id, .. } => {
                log::debug!("Reducer result for request {}", request_id);
            }
            ServerEvent::SubscriptionError { error, .. } => {
                log::error!("Subscription error: {}", error);
            }
            _ => {}
        }
    });
}
```

- [ ] **Step 6: Add spacetimedb-client-api-messages to app Cargo.toml**

The `sync.rs` now uses `RowListLen` trait and `TableUpdateRows` from the messages crate. Add to `crates/app/Cargo.toml`:
```toml
spacetimedb-client-api-messages = "2.1"
```

- [ ] **Step 7: Verify it compiles**

Run: `cargo check -p app --target wasm32-unknown-unknown`
Expected: Compiles

- [ ] **Step 8: Commit**

```bash
git add crates/stdb-client/ crates/app/
git commit -m "feat: implement BSATN row decoding for SpacetimeDB sync"
```

---

## Task 2: Debug Test Hooks

**Files:**
- Modify: `crates/app/src/main.rs`
- Modify: `crates/app/Cargo.toml`

- [ ] **Step 1: Add serde-wasm-bindgen as a dev dependency**

- [ ] **Step 1: Add test hooks to main.rs**

In the `App` component in `crates/app/src/main.rs`, after the `provide_context(state)` line, add:

```rust
// Debug-only test hooks: expose element/cursor counts on window for Playwright
#[cfg(debug_assertions)]
{
    let store = state.store.clone();
    Effect::new(move |_| {
        let elem_count = store.elements.with(|e| e.len());
        let cursor_count = store.cursors.with(|c| c.len());

        let global = js_sys::global();
        js_sys::Reflect::set(
            &global,
            &wasm_bindgen::JsValue::from_str("__TEST_ELEMENT_COUNT"),
            &wasm_bindgen::JsValue::from_f64(elem_count as f64),
        ).ok();
        js_sys::Reflect::set(
            &global,
            &wasm_bindgen::JsValue::from_str("__TEST_CURSOR_COUNT"),
            &wasm_bindgen::JsValue::from_f64(cursor_count as f64),
        ).ok();
    });
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo check -p app --target wasm32-unknown-unknown`
Expected: Compiles

- [ ] **Step 3: Commit**

```bash
git add crates/app/
git commit -m "feat: add debug-only test hooks for Playwright E2E"
```

---

## Task 3: Playwright Test Infrastructure & Specs

**Files:**
- Create: `tests/e2e/package.json`
- Create: `tests/e2e/tsconfig.json`
- Create: `tests/e2e/playwright.config.ts`
- Create: `tests/e2e/global-setup.ts`
- Create: `tests/e2e/global-teardown.ts`
- Create: `tests/e2e/helpers.ts`
- Create: `tests/e2e/specs/canvas-loads.spec.ts`
- Create: `tests/e2e/specs/two-browser-sync.spec.ts`
- Create: `tests/e2e/specs/cursor-presence.spec.ts`
- Create: `tests/e2e/specs/delete-sync.spec.ts`
- Modify: `.gitignore`

- [ ] **Step 1: Create package.json**

`tests/e2e/package.json`:
```json
{
  "name": "collaborate-e2e",
  "private": true,
  "scripts": {
    "test": "npx playwright test",
    "test:headed": "npx playwright test --headed"
  },
  "devDependencies": {
    "@playwright/test": "^1.45.0",
    "typescript": "^5.5.0"
  }
}
```

- [ ] **Step 2: Create tsconfig.json**

`tests/e2e/tsconfig.json`:
```json
{
  "compilerOptions": {
    "target": "ES2020",
    "module": "commonjs",
    "strict": true,
    "esModuleInterop": true,
    "outDir": "./dist",
    "rootDir": "."
  },
  "include": ["**/*.ts"]
}
```

- [ ] **Step 3: Create playwright.config.ts**

`tests/e2e/playwright.config.ts`:
```typescript
import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './specs',
  timeout: 30_000,
  retries: 1,
  workers: 1, // tests share the server
  globalSetup: './global-setup.ts',
  globalTeardown: './global-teardown.ts',
  use: {
    baseURL: 'http://localhost:8080',
    trace: 'on-first-retry',
  },
  projects: [
    {
      name: 'chromium',
      use: { browserName: 'chromium' },
    },
  ],
});
```

- [ ] **Step 4: Create global-setup.ts**

`tests/e2e/global-setup.ts`:
```typescript
import { spawn, execSync, ChildProcess } from 'child_process';
import * as fs from 'fs';
import * as path from 'path';

const PID_FILE = path.join(__dirname, '.test-pids.json');
const PROJECT_ROOT = path.resolve(__dirname, '../..');

async function waitForUrl(url: string, timeoutMs = 60_000): Promise<void> {
  const start = Date.now();
  while (Date.now() - start < timeoutMs) {
    try {
      const res = await fetch(url);
      if (res.ok) return;
    } catch {
      // not ready yet
    }
    await new Promise(r => setTimeout(r, 1000));
  }
  throw new Error(`Timed out waiting for ${url}`);
}

async function globalSetup() {
  console.log('Starting SpacetimeDB...');
  const spacetime = spawn('spacetime', ['start'], {
    cwd: PROJECT_ROOT,
    stdio: 'pipe',
    detached: true,
  });
  spacetime.unref();

  // Wait for SpacetimeDB to be ready
  await waitForUrl('http://localhost:3000/database/ping', 30_000).catch(() => {
    // SpacetimeDB might not have a /ping endpoint — wait a fixed time
    return new Promise(r => setTimeout(r, 5000));
  });

  console.log('Publishing server module...');
  execSync('spacetime publish collaborate crates/server --yes', {
    cwd: PROJECT_ROOT,
    stdio: 'inherit',
  });

  console.log('Starting Trunk dev server...');
  const trunk = spawn('trunk', ['serve', '--port', '8080'], {
    cwd: path.join(PROJECT_ROOT, 'crates/app'),
    stdio: 'pipe',
    detached: true,
  });
  trunk.unref();

  await waitForUrl('http://localhost:8080', 60_000);

  // Save PIDs for teardown
  fs.writeFileSync(PID_FILE, JSON.stringify({
    spacetime: spacetime.pid,
    trunk: trunk.pid,
  }));

  console.log('Test infrastructure ready.');
}

export default globalSetup;
```

- [ ] **Step 5: Create global-teardown.ts**

`tests/e2e/global-teardown.ts`:
```typescript
import * as fs from 'fs';
import * as path from 'path';

const PID_FILE = path.join(__dirname, '.test-pids.json');

function killProcess(pid: number | undefined) {
  if (!pid) return;
  try {
    // Kill the process group (negative PID)
    process.kill(-pid, 'SIGTERM');
  } catch {
    try {
      process.kill(pid, 'SIGTERM');
    } catch {
      // already dead
    }
  }
}

async function globalTeardown() {
  console.log('Tearing down test infrastructure...');
  try {
    const pids = JSON.parse(fs.readFileSync(PID_FILE, 'utf-8'));
    killProcess(pids.trunk);
    killProcess(pids.spacetime);
    fs.unlinkSync(PID_FILE);
  } catch {
    // PID file may not exist if setup failed
  }
}

export default globalTeardown;
```

- [ ] **Step 6: Create helpers.ts**

`tests/e2e/helpers.ts`:
```typescript
import { Page } from '@playwright/test';

export async function getElementCount(page: Page): Promise<number> {
  return page.evaluate(() => (window as any).__TEST_ELEMENT_COUNT ?? 0);
}

export async function getCursorCount(page: Page): Promise<number> {
  return page.evaluate(() => (window as any).__TEST_CURSOR_COUNT ?? 0);
}

export async function waitForElementCount(
  page: Page,
  count: number,
  timeout = 10_000,
): Promise<void> {
  await page.waitForFunction(
    (expected) => (window as any).__TEST_ELEMENT_COUNT === expected,
    count,
    { timeout },
  );
}

export async function waitForCursorCount(
  page: Page,
  minCount: number,
  timeout = 10_000,
): Promise<void> {
  await page.waitForFunction(
    (min) => ((window as any).__TEST_CURSOR_COUNT ?? 0) >= min,
    minCount,
    { timeout },
  );
}

export async function waitForConnected(page: Page, timeout = 15_000): Promise<void> {
  await page.waitForFunction(
    () => document.body?.innerText?.toLowerCase().includes('connected'),
    undefined,
    { timeout },
  );
}

/**
 * Draw a rectangle on the canvas by pressing 'r' then click-dragging.
 */
export async function drawRectangle(
  page: Page,
  x: number,
  y: number,
  width: number,
  height: number,
): Promise<void> {
  // Press 'r' to enter rectangle insert mode
  await page.keyboard.press('r');
  // Small delay for mode change
  await page.waitForTimeout(100);

  const canvas = page.locator('canvas');
  const box = await canvas.boundingBox();
  if (!box) throw new Error('Canvas not found');

  const startX = box.x + x;
  const startY = box.y + y;

  await page.mouse.move(startX, startY);
  await page.mouse.down();
  await page.mouse.move(startX + width, startY + height, { steps: 5 });
  await page.mouse.up();

  // Wait a moment for the element to be created
  await page.waitForTimeout(200);
}
```

- [ ] **Step 7: Create canvas-loads.spec.ts**

`tests/e2e/specs/canvas-loads.spec.ts`:
```typescript
import { test, expect } from '@playwright/test';

test.describe('Canvas Loads', () => {
  test('page loads with canvas and mode indicator', async ({ page }) => {
    // Register console listener before navigation to catch all errors
    const errors: string[] = [];
    page.on('console', msg => {
      if (msg.type() === 'error') errors.push(msg.text());
    });

    await page.goto('/');

    // Canvas element exists
    const canvas = page.locator('canvas');
    await expect(canvas).toBeVisible();

    // Mode indicator shows NORMAL
    await expect(page.locator('text=NORMAL')).toBeVisible();

    await page.waitForTimeout(2000);
    // Filter out expected connection errors (SpacetimeDB may not be ready)
    const realErrors = errors.filter(e => !e.includes('SpacetimeDB') && !e.includes('WebSocket'));
    expect(realErrors).toEqual([]);
  });
});
```

- [ ] **Step 8: Create two-browser-sync.spec.ts**

`tests/e2e/specs/two-browser-sync.spec.ts`:
```typescript
import { test, expect } from '@playwright/test';
import { waitForConnected, waitForElementCount, drawRectangle } from '../helpers';

test.describe('Two-Browser Sync', () => {
  test('rectangle drawn in browser A appears in browser B', async ({ browser }) => {
    // Create two isolated browser contexts
    const contextA = await browser.newContext();
    const contextB = await browser.newContext();
    const pageA = await contextA.newPage();
    const pageB = await contextB.newPage();

    try {
      // Navigate both to the app
      await pageA.goto('/');
      await pageB.goto('/');

      // Wait for both to connect to SpacetimeDB
      await waitForConnected(pageA);
      await waitForConnected(pageB);

      // Draw a rectangle in page A
      await drawRectangle(pageA, 100, 100, 200, 150);

      // Verify element count = 1 in page A
      await waitForElementCount(pageA, 1);

      // Verify element syncs to page B
      await waitForElementCount(pageB, 1, 15_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
```

- [ ] **Step 9: Create cursor-presence.spec.ts**

`tests/e2e/specs/cursor-presence.spec.ts`:
```typescript
import { test, expect } from '@playwright/test';
import { waitForConnected, waitForCursorCount } from '../helpers';

test.describe('Cursor Presence', () => {
  test('cursor movement in browser A is visible in browser B', async ({ browser }) => {
    const contextA = await browser.newContext();
    const contextB = await browser.newContext();
    const pageA = await contextA.newPage();
    const pageB = await contextB.newPage();

    try {
      await pageA.goto('/');
      await pageB.goto('/');
      await waitForConnected(pageA);
      await waitForConnected(pageB);

      // Move mouse in page A to trigger cursor update
      const canvas = pageA.locator('canvas');
      const box = await canvas.boundingBox();
      if (!box) throw new Error('Canvas not found');
      await pageA.mouse.move(box.x + 200, box.y + 200);

      // Wait a moment for cursor update reducer to fire
      await pageA.waitForTimeout(500);

      // Page B should see at least 1 remote cursor
      await waitForCursorCount(pageB, 1, 10_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
```

- [ ] **Step 10: Create delete-sync.spec.ts**

`tests/e2e/specs/delete-sync.spec.ts`:
```typescript
import { test, expect } from '@playwright/test';
import { waitForConnected, waitForElementCount, drawRectangle, getElementCount } from '../helpers';

test.describe('Delete Sync', () => {
  test('deleting an element in browser A removes it from browser B', async ({ browser }) => {
    const contextA = await browser.newContext();
    const contextB = await browser.newContext();
    const pageA = await contextA.newPage();
    const pageB = await contextB.newPage();

    try {
      await pageA.goto('/');
      await pageB.goto('/');
      await waitForConnected(pageA);
      await waitForConnected(pageB);

      // Draw rectangle in A
      await drawRectangle(pageA, 100, 100, 200, 150);
      await waitForElementCount(pageA, 1);
      await waitForElementCount(pageB, 1, 15_000);

      // Select the element in A by clicking on it
      const canvasA = pageA.locator('canvas');
      const boxA = await canvasA.boundingBox();
      if (!boxA) throw new Error('Canvas not found');
      // Click in the center of where we drew the rectangle
      await pageA.mouse.click(boxA.x + 200, boxA.y + 175);
      await pageA.waitForTimeout(200);

      // Press Escape first to ensure we're in Normal mode, then select and delete
      await pageA.keyboard.press('Escape');
      await pageA.waitForTimeout(100);

      // Click to select
      await pageA.mouse.click(boxA.x + 200, boxA.y + 175);
      await pageA.waitForTimeout(200);

      // Delete with dd
      await pageA.keyboard.press('d');
      await pageA.keyboard.press('d');
      await pageA.waitForTimeout(200);

      // Element count should drop to 0 in both browsers
      await waitForElementCount(pageA, 0, 5_000);
      await waitForElementCount(pageB, 0, 15_000);
    } finally {
      await contextA.close();
      await contextB.close();
    }
  });
});
```

- [ ] **Step 11: Update .gitignore**

Add to project root `.gitignore`:
```
tests/e2e/node_modules/
tests/e2e/dist/
tests/e2e/.test-pids.json
tests/e2e/test-results/
tests/e2e/playwright-report/
```

- [ ] **Step 12: Install Playwright dependencies**

Run:
```bash
cd tests/e2e && npm install && npx playwright install chromium
```

- [ ] **Step 13: Verify Playwright setup**

Run:
```bash
cd tests/e2e && npx playwright test --list
```
Expected: Lists all 4 test specs

- [ ] **Step 14: Commit**

```bash
git add tests/e2e/ .gitignore
git commit -m "feat: add Playwright E2E tests for two-browser collaboration"
```

---

## Running the Tests

After all tasks are complete:

```bash
cd tests/e2e && npm test
```

This will:
1. Start SpacetimeDB and publish the server module
2. Start Trunk dev server
3. Run all 4 test specs
4. Tear down servers

For debugging:
```bash
cd tests/e2e && npm run test:headed
```
