mod types;
mod state;
mod canvas;
mod ui;
mod vim;
mod commands;
mod tools;
mod export;
mod sync;

use leptos::prelude::*;
use state::AppState;
use canvas::DrawCanvas;
use ui::{TopBar, BottomBar};
use crate::vim::VimStateMachine;
use crate::commands::handle_vim_action;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

fn main() {
    console_error_panic_hook::set_once();
    wasm_logger::init(wasm_logger::Config::default());
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let state = AppState::new();
    provide_context(state.clone());

    // Connect to SpacetimeDB in the background.
    {
        let state = state.clone();
        wasm_bindgen_futures::spawn_local(async move {
            match stdb_client::connection::StdbConnection::connect(
                "ws://localhost:3000",
                "collaborate",
            )
            .await
            {
                Ok(conn) => {
                    crate::sync::setup_event_handler(&conn, state.store.clone());
                    state.connection.set(Some(conn.clone()));
                    // Attempt to create the default room (fails gracefully if it exists).
                    let room_args = spacetimedb_lib::bsatn::to_vec(&("default".to_string(),)).unwrap();
                    conn.call_reducer("create_room", room_args);
                    // Join the default room so the server creates a cursor row for us.
                    let join_args = spacetimedb_lib::bsatn::to_vec(&(1u64, "User".to_string())).unwrap();
                    conn.call_reducer("join_room", join_args);
                    // Subscribe to all elements/cursors (no room filter) to
                    // avoid a race where room_id isn't known yet.
                    conn.subscribe_all();
                    state.store.current_room.set(Some(1));
                    log::info!("Connected to SpacetimeDB");
                }
                Err(e) => {
                    log::error!(
                        "SpacetimeDB connection failed: {} — running in offline mode",
                        e
                    );
                    // App still works locally without server.
                }
            }
        });
    }

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
            )
            .ok();
            js_sys::Reflect::set(
                &global,
                &wasm_bindgen::JsValue::from_str("__TEST_CURSOR_COUNT"),
                &wasm_bindgen::JsValue::from_f64(cursor_count as f64),
            )
            .ok();
        });
    }

    let vim = Rc::new(RefCell::new(VimStateMachine::new()));
    let state_for_keys = state.clone();
    let vim_clone = vim.clone();

    Effect::new(move || {
        let vim = vim_clone.clone();
        let state = state_for_keys.clone();
        let closure = Closure::<dyn FnMut(_)>::new(move |ev: web_sys::KeyboardEvent| {
            let action = {
                let mut vm = vim.borrow_mut();
                let action = vm.handle_key(&ev.key(), ev.shift_key(), ev.ctrl_key());
                state.mode.set(vm.mode());
                state.key_buffer.set(vm.key_buffer().to_string());
                state.command_buffer.set(vm.command_buffer().to_string());
                action
            };
            handle_vim_action(&state, action);
            ev.prevent_default();
        });
        let doc = leptos::prelude::document();
        doc.add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref()).unwrap();
        closure.forget();
    });

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
