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

/// Help overlay shown when user presses `?`.
#[component]
pub fn HelpOverlay() -> impl IntoView {
    let state = expect_context::<AppState>();

    let visible = move || state.show_help.get();

    view! {
        <Show when=visible>
            <div
                on:click=move |_| state.show_help.set(false)
                style="
                    position: fixed; inset: 0; z-index: 1000;
                    background: rgba(0,0,0,0.7);
                    display: flex; align-items: center; justify-content: center;
                    cursor: pointer;
                "
            >
                <div
                    on:click=move |ev: web_sys::MouseEvent| ev.stop_propagation()
                    style="
                        background: #1a1a2e; border: 1px solid #0f3460;
                        border-radius: 12px; padding: 24px 32px;
                        max-width: 600px; width: 90%; max-height: 80vh;
                        overflow-y: auto; cursor: default;
                        font-family: monospace; font-size: 13px; color: #eee;
                    "
                >
                    <div style="display: flex; justify-content: space-between; align-items: center; margin-bottom: 16px;">
                        <h2 style="margin: 0; font-size: 18px; color: #6ee7b7;">"Keyboard Shortcuts"</h2>
                        <span style="color: #666; font-size: 11px;">"press ? or click outside to close"</span>
                    </div>

                    <div style="display: grid; grid-template-columns: 1fr 1fr; gap: 20px;">
                        <div>
                            <h3 style="color: #22c55e; font-size: 13px; margin: 0 0 8px 0;">"Normal Mode"</h3>
                            <table style="width: 100%; border-collapse: collapse;">
                                <tbody>
                                    {key_row("r", "Draw rectangle")}
                                    {key_row("e", "Draw ellipse")}
                                    {key_row("a", "Draw arrow")}
                                    {key_row("L", "Draw line")}
                                    {key_row("t", "Draw text")}
                                    {key_row("f", "Freehand draw")}
                                    {key_row("h j k l", "Move selected")}
                                    {key_row("dd", "Delete selected")}
                                    {key_row("yy", "Copy selected")}
                                    {key_row("p", "Paste")}
                                    {key_row("u", "Undo")}
                                    {key_row("Ctrl+r", "Redo")}
                                    {key_row("v", "Visual mode")}
                                    {key_row(":", "Command mode")}
                                    {key_row("?", "Toggle help")}
                                </tbody>
                            </table>
                        </div>
                        <div>
                            <h3 style="color: #f59e0b; font-size: 13px; margin: 0 0 8px 0;">"Insert Mode"</h3>
                            <table style="width: 100%; border-collapse: collapse;">
                                <tbody>
                                    {key_row("click+drag", "Create shape")}
                                    {key_row("click, click", "Place arrow/line")}
                                    {key_row("type", "Enter text")}
                                    {key_row("Esc", "Back to Normal")}
                                </tbody>
                            </table>

                            <h3 style="color: #ef4444; font-size: 13px; margin: 16px 0 8px 0;">"Commands"</h3>
                            <table style="width: 100%; border-collapse: collapse;">
                                <tbody>
                                    {key_row(":w", "Export PNG")}
                                    {key_row(":ws", "Export SVG")}
                                    {key_row(":color #hex", "Stroke color")}
                                    {key_row(":fill #hex", "Fill color")}
                                    {key_row(":stroke N", "Stroke width")}
                                </tbody>
                            </table>

                            <h3 style="color: #888; font-size: 13px; margin: 16px 0 8px 0;">"Canvas"</h3>
                            <table style="width: 100%; border-collapse: collapse;">
                                <tbody>
                                    {key_row("click", "Select element")}
                                    {key_row("scroll", "Zoom in/out")}
                                </tbody>
                            </table>
                        </div>
                    </div>
                </div>
            </div>
        </Show>
    }
}

fn key_row(key: &str, desc: &str) -> impl IntoView {
    let key = key.to_string();
    let desc = desc.to_string();
    view! {
        <tr>
            <td style="padding: 2px 8px 2px 0;">
                <kbd style="
                    background: #0f3460; padding: 1px 6px; border-radius: 3px;
                    font-size: 11px; color: #6ee7b7; white-space: nowrap;
                ">{key}</kbd>
            </td>
            <td style="padding: 2px 0; color: #aaa;">{desc}</td>
        </tr>
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
                <span style="color: #555;">"? help"</span>
                <span>{zoom_pct}</span>
                <span>{mouse_world}</span>
            </div>
        </div>
    }
}
