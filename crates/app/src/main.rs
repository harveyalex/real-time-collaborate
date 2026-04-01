mod types;
mod state;
mod canvas;
mod ui;
mod vim;
mod commands;
mod tools;
mod export;

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
