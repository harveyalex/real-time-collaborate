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
