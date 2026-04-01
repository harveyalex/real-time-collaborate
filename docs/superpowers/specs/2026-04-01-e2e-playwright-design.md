# Playwright E2E Tests for Two-Browser Collaboration — Design Spec

## Overview

Automated Playwright tests that verify real-time collaboration between two browser contexts. The test harness manages the full stack: SpacetimeDB server, server module publishing, and Trunk dev server.

**Goal:** Prove that drawing operations in one browser appear in another via SpacetimeDB sync.

**Non-goals:** Visual regression testing, performance benchmarking, offline mode testing.

## Test Infrastructure

### Directory Structure

```
tests/e2e/
├── package.json          # Playwright + ts-node deps
├── playwright.config.ts  # Playwright config (baseURL, global setup/teardown)
├── global-setup.ts       # Start SpacetimeDB + publish module + start Trunk
├── global-teardown.ts    # Kill child processes
├── helpers.ts            # Shared utilities (wait for element count, etc.)
└── specs/
    ├── canvas-loads.spec.ts
    ├── two-browser-sync.spec.ts
    ├── cursor-presence.spec.ts
    └── delete-sync.spec.ts
```

### Global Setup (global-setup.ts)

1. Start `spacetime start` as a child process, wait for it to be listening (poll HTTP health endpoint)
2. Run `spacetime publish collaborate crates/server` and wait for success
3. Start `trunk serve` in `crates/app/`, wait for HTTP 200 on `http://localhost:8080`
4. Store child process PIDs in a temp file for teardown

### Global Teardown (global-teardown.ts)

1. Read PIDs from temp file
2. Kill Trunk and SpacetimeDB processes
3. Clean up temp file

### Playwright Config

- Two browser contexts per test (not tabs — isolated like incognito)
- Base URL: `http://localhost:8080`
- Timeout: 30s per test (SpacetimeDB sync can take a moment)
- Single worker (tests share the server)

## Test Hooks

A debug-only block in the Leptos app that exposes reactive state to `window` for Playwright to query.

### App-side (Rust)

In `crates/app/src/main.rs`, add a debug-only hook after `provide_context`:

```rust
#[cfg(debug_assertions)]
{
    use wasm_bindgen::prelude::*;

    #[wasm_bindgen]
    extern "C" {
        #[wasm_bindgen(js_namespace = window, js_name = "__TEST_SET_HOOK")]
        fn test_set_hook(name: &str, value: &JsValue);
    }

    // Expose element count and element IDs to window for Playwright
    Effect::new({
        let store = state.store.clone();
        move || {
            let count = store.elements.with(|e| e.len());
            let ids: Vec<u64> = store.elements.with(|e| e.keys().copied().collect());
            // Set on window object
            let js_count = JsValue::from_f64(count as f64);
            js_sys::Reflect::set(
                &js_sys::global(),
                &JsValue::from_str("__TEST_ELEMENT_COUNT"),
                &js_count,
            ).ok();
            let js_ids = serde_wasm_bindgen::to_value(&ids).unwrap_or(JsValue::NULL);
            js_sys::Reflect::set(
                &js_sys::global(),
                &JsValue::from_str("__TEST_ELEMENT_IDS"),
                &js_ids,
            ).ok();
        }
    });
}
```

### Playwright-side (TypeScript)

```typescript
async function getElementCount(page: Page): Promise<number> {
    return page.evaluate(() => (window as any).__TEST_ELEMENT_COUNT ?? 0);
}

async function waitForElementCount(page: Page, count: number, timeout = 10000) {
    await page.waitForFunction(
        (expected) => (window as any).__TEST_ELEMENT_COUNT === expected,
        count,
        { timeout }
    );
}
```

## Test Cases

### 1. canvas-loads.spec.ts

- Open page in a single context
- Verify canvas element exists
- Verify mode indicator shows "NORMAL"
- Verify app renders without errors (no console errors)

### 2. two-browser-sync.spec.ts

- Open two browser contexts (A and B), both navigate to the app
- Wait for both to show "Connected" in the top bar
- In context A: press `r` to enter rectangle mode, click+drag to draw a rectangle
- Wait for context A to show element count = 1
- Wait for context B to show element count = 1 (synced via SpacetimeDB)
- Verify both contexts have the same element

### 3. cursor-presence.spec.ts

- Open two browser contexts, both connected
- In context A: move mouse to a known canvas position
- In context B: verify a cursor label appears (check for the cursor dot via test hook or screenshot)

### 4. delete-sync.spec.ts

- Open two contexts, draw a shape in context A
- Wait for element count = 1 in both contexts
- In context A: click the shape to select, press `dd` to delete
- Wait for element count = 0 in context A
- Wait for element count = 0 in context B (delete synced)

## Dependencies

- `@playwright/test` — test runner
- `ts-node` / `typescript` — TypeScript execution
- SpacetimeDB CLI (`spacetime`) — must be on PATH
- Trunk (`trunk`) — must be on PATH
- `serde-wasm-bindgen` — added to app crate for test hook serialization (debug only)

## Build Considerations

- Test hooks are `#[cfg(debug_assertions)]` only — zero cost in release builds
- Trunk dev server builds in debug mode by default, so hooks are active during `trunk serve`
- `serde-wasm-bindgen` is a dev-dependency only
