use leptos::prelude::*;

use crate::state::{AppState, VimMode};

/// Top bar component: app name, room info, user count, mode indicator.
#[component]
pub fn TopBar() -> impl IntoView {
    let state = expect_context::<AppState>();

    let mode_label = move || {
        match state.mode.get() {
            VimMode::Normal => "NORMAL",
            VimMode::Insert => "INSERT",
            VimMode::Visual => "VISUAL",
            VimMode::Command => "COMMAND",
        }
    };

    let mode_color = move || {
        match state.mode.get() {
            VimMode::Normal => "#22c55e",  // green
            VimMode::Insert => "#f59e0b",  // amber
            VimMode::Visual => "#a855f7",  // purple
            VimMode::Command => "#ef4444", // red
        }
    };

    let connected = move || state.store.connected.get();

    let user_count = move || {
        state.store.cursors.with(|c| c.len()) + 1
    };

    view! {
        <div style="
            display: flex;
            align-items: center;
            justify-content: space-between;
            height: 40px;
            padding: 0 16px;
            background: #16213e;
            border-bottom: 1px solid #0f3460;
            font-family: monospace;
            font-size: 13px;
            color: #eee;
            user-select: none;
        ">
            <div style="display: flex; align-items: center; gap: 12px;">
                <span style="font-weight: bold; font-size: 15px;">"collaborate"</span>
                <span style="color: #888;">
                    {move || if connected() { "connected" } else { "disconnected" }}
                </span>
            </div>

            <div style="display: flex; align-items: center; gap: 12px;">
                <span style="color: #888;">
                    {move || format!("{} user(s)", user_count())}
                </span>
                <span style:background=mode_color
                      style="
                          padding: 2px 10px;
                          border-radius: 4px;
                          font-weight: bold;
                          font-size: 11px;
                          color: #000;
                      ">
                    {mode_label}
                </span>
            </div>
        </div>
    }
}

/// Bottom bar component: command/key buffer, zoom percentage, mouse world coordinates.
#[component]
pub fn BottomBar() -> impl IntoView {
    let state = expect_context::<AppState>();

    let buffer_display = move || {
        match state.mode.get() {
            VimMode::Command => format!(":{}", state.command_buffer.get()),
            _ => state.key_buffer.get(),
        }
    };

    let zoom_pct = move || {
        let cam = state.camera.get();
        format!("{:.0}%", cam.zoom * 100.0)
    };

    let mouse_world = move || {
        let (sx, sy) = state.mouse_pos.get();
        let (wx, wy) = state.screen_to_world(sx, sy);
        format!("{:.0}, {:.0}", wx, wy)
    };

    view! {
        <div style="
            display: flex;
            align-items: center;
            justify-content: space-between;
            height: 24px;
            padding: 0 12px;
            background: #16213e;
            border-top: 1px solid #0f3460;
            font-family: monospace;
            font-size: 11px;
            color: #aaa;
            user-select: none;
        ">
            <span style="min-width: 200px;">{buffer_display}</span>
            <div style="display: flex; gap: 16px;">
                <span>{zoom_pct}</span>
                <span>{mouse_world}</span>
            </div>
        </div>
    }
}
