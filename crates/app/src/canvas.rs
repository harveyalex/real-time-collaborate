use std::cell::RefCell;
use std::f64::consts::PI;
use std::rc::Rc;

use leptos::prelude::*;
use leptos::html;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CanvasRenderingContext2d, HtmlCanvasElement};

use shared::ElementKind;
use stdb_client::ElementData;

use crate::state::AppState;

/// Convert a packed RGBA u32 to a CSS `rgba()` string.
pub fn color_u32_to_css(color: u32) -> String {
    let r = (color >> 24) & 0xFF;
    let g = (color >> 16) & 0xFF;
    let b = (color >> 8) & 0xFF;
    let a = color & 0xFF;
    format!("rgba({},{},{},{:.2})", r, g, b, a as f64 / 255.0)
}

/// Render a single element onto the given 2D context.
/// The caller is expected to have already applied the camera transform.
pub fn render_element(ctx: &CanvasRenderingContext2d, elem: &ElementData) {
    let stroke_css = color_u32_to_css(elem.stroke_color);
    let fill_css = color_u32_to_css(elem.fill_color);

    ctx.set_global_alpha(elem.opacity as f64);
    ctx.set_stroke_style_str(&stroke_css);
    ctx.set_fill_style_str(&fill_css);
    ctx.set_line_width(elem.stroke_width as f64);

    ctx.save();

    // Apply per-element rotation around the element center.
    if elem.rotation != 0.0 {
        let cx = elem.x + elem.width / 2.0;
        let cy = elem.y + elem.height / 2.0;
        ctx.translate(cx, cy).unwrap_or(());
        ctx.rotate(elem.rotation).unwrap_or(());
        ctx.translate(-cx, -cy).unwrap_or(());
    }

    match elem.kind {
        ElementKind::Rectangle => {
            if (elem.fill_color & 0xFF) > 0 {
                ctx.fill_rect(elem.x, elem.y, elem.width, elem.height);
            }
            ctx.stroke_rect(elem.x, elem.y, elem.width, elem.height);
        }
        ElementKind::Ellipse => {
            let cx = elem.x + elem.width / 2.0;
            let cy = elem.y + elem.height / 2.0;
            let rx = elem.width.abs() / 2.0;
            let ry = elem.height.abs() / 2.0;
            ctx.begin_path();
            ctx.ellipse(cx, cy, rx, ry, 0.0, 0.0, PI * 2.0).unwrap_or(());
            if (elem.fill_color & 0xFF) > 0 {
                ctx.fill();
            }
            ctx.stroke();
        }
        ElementKind::Line => {
            if elem.points.len() >= 2 {
                ctx.begin_path();
                ctx.move_to(elem.points[0].x, elem.points[0].y);
                for p in &elem.points[1..] {
                    ctx.line_to(p.x, p.y);
                }
                ctx.stroke();
            }
        }
        ElementKind::Arrow => {
            if elem.points.len() >= 2 {
                ctx.begin_path();
                ctx.move_to(elem.points[0].x, elem.points[0].y);
                for p in &elem.points[1..] {
                    ctx.line_to(p.x, p.y);
                }
                ctx.stroke();

                // Draw arrowhead at the last point.
                let last = &elem.points[elem.points.len() - 1];
                let prev = &elem.points[elem.points.len() - 2];
                let angle = (last.y - prev.y).atan2(last.x - prev.x);
                let head_len = 12.0;
                ctx.begin_path();
                ctx.move_to(last.x, last.y);
                ctx.line_to(
                    last.x - head_len * (angle - PI / 6.0).cos(),
                    last.y - head_len * (angle - PI / 6.0).sin(),
                );
                ctx.move_to(last.x, last.y);
                ctx.line_to(
                    last.x - head_len * (angle + PI / 6.0).cos(),
                    last.y - head_len * (angle + PI / 6.0).sin(),
                );
                ctx.stroke();
            }
        }
        ElementKind::Text => {
            ctx.set_font(&format!("{}px monospace", elem.font_size));
            ctx.set_fill_style_str(&stroke_css);
            ctx.fill_text(&elem.text_content, elem.x, elem.y + elem.font_size as f64)
                .unwrap_or(());
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
    ctx.set_global_alpha(1.0);
}

/// Render selection handles (8 handles: 4 corners + 4 midpoints) for a selected element.
fn render_selection_handles(ctx: &CanvasRenderingContext2d, elem: &ElementData) {
    let handle_size = 6.0;
    let half = handle_size / 2.0;
    ctx.set_fill_style_str("#00ff00");
    ctx.set_stroke_style_str("#005500");
    ctx.set_line_width(1.0);

    let x = elem.x;
    let y = elem.y;
    let w = elem.width;
    let h = elem.height;

    let positions = [
        (x, y),                         // top-left
        (x + w / 2.0, y),               // top-mid
        (x + w, y),                     // top-right
        (x + w, y + h / 2.0),           // mid-right
        (x + w, y + h),                 // bottom-right
        (x + w / 2.0, y + h),           // bottom-mid
        (x, y + h),                     // bottom-left
        (x, y + h / 2.0),               // mid-left
    ];

    for (hx, hy) in positions {
        ctx.fill_rect(hx - half, hy - half, handle_size, handle_size);
        ctx.stroke_rect(hx - half, hy - half, handle_size, handle_size);
    }

    // Selection bounding box.
    ctx.set_stroke_style_str("#00ff00");
    ctx.set_line_width(1.0);
    let dashes = js_sys::Array::new();
    dashes.push(&JsValue::from(4.0));
    dashes.push(&JsValue::from(4.0));
    ctx.set_line_dash(&dashes).unwrap_or(());
    ctx.stroke_rect(x, y, w, h);
    ctx.set_line_dash(&js_sys::Array::new()).unwrap_or(());
}

/// Render a remote user cursor.
fn render_cursor(ctx: &CanvasRenderingContext2d, x: f64, y: f64, name: &str, color: u32) {
    let css = color_u32_to_css(color);
    ctx.set_fill_style_str(&css);
    ctx.begin_path();
    ctx.arc(x, y, 5.0, 0.0, PI * 2.0).unwrap_or(());
    ctx.fill();

    ctx.set_font("11px monospace");
    ctx.set_fill_style_str(&css);
    ctx.fill_text(name, x + 8.0, y - 4.0).unwrap_or(());
}

/// Perform a full frame render.
fn render_frame(state: &AppState, ctx: &CanvasRenderingContext2d, width: f64, height: f64) {
    // Clear with dark background.
    ctx.set_fill_style_str("#1a1a2e");
    ctx.fill_rect(0.0, 0.0, width, height);

    ctx.save();

    // Apply camera transform.
    let cam = state.camera.get_untracked();
    ctx.scale(cam.zoom, cam.zoom).unwrap_or(());
    ctx.translate(-cam.x, -cam.y).unwrap_or(());

    // Render elements sorted by z_index.
    let elements = state.store.sorted_elements();
    let selected = state.selected_ids.get_untracked();

    for elem in &elements {
        render_element(ctx, elem);
    }

    // Render selection handles on selected elements.
    for elem in &elements {
        if selected.contains(&elem.id) {
            render_selection_handles(ctx, elem);
        }
    }

    // Render other users' cursors.
    state.store.cursors.with_untracked(|cursors| {
        for cursor in cursors.values() {
            render_cursor(ctx, cursor.x, cursor.y, &cursor.name, cursor.color);
        }
    });

    ctx.restore();
}

/// The main canvas drawing component.
#[component]
pub fn DrawCanvas() -> impl IntoView {
    let state = expect_context::<AppState>();
    let canvas_ref = NodeRef::<html::Canvas>::new();

    // Set up the render loop and event handlers once the canvas is mounted.
    Effect::new({
        let state = state.clone();
        move |_| {
            let Some(canvas_el) = canvas_ref.get() else {
                return;
            };
            let canvas: HtmlCanvasElement = canvas_el.into();

            // Size the canvas to fill its container.
            let rect = canvas.get_bounding_client_rect();
            let dpr = web_sys::window()
                .map(|w| w.device_pixel_ratio())
                .unwrap_or(1.0);
            let w = rect.width();
            let h = rect.height();
            canvas.set_width((w * dpr) as u32);
            canvas.set_height((h * dpr) as u32);
            state.canvas_size.set((w, h));

            let ctx: CanvasRenderingContext2d = canvas
                .get_context("2d")
                .ok()
                .flatten()
                .expect("canvas 2d context")
                .dyn_into()
                .expect("CanvasRenderingContext2d");
            ctx.scale(dpr, dpr).unwrap_or(());

            // requestAnimationFrame render loop using Rc<RefCell<Option<Closure>>> pattern.
            let f: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
            let g = f.clone();
            let state_inner = state.clone();

            *g.borrow_mut() = Some(Closure::wrap(Box::new(move || {
                render_frame(&state_inner, &ctx, w, h);

                // Request next frame.
                if let Some(win) = web_sys::window() {
                    if let Some(ref cb) = *f.borrow() {
                        win.request_animation_frame(cb.as_ref().unchecked_ref())
                            .unwrap_or(0);
                    }
                }
            }) as Box<dyn FnMut()>));

            // Kick off the first frame.
            if let Some(win) = web_sys::window() {
                if let Some(ref cb) = *g.borrow() {
                    win.request_animation_frame(cb.as_ref().unchecked_ref())
                        .unwrap_or(0);
                }
            }
        }
    });

    // Mouse event handlers.
    let state_mouse = state.clone();
    let on_mousedown = move |ev: web_sys::MouseEvent| {
        let (wx, wy) = state_mouse.screen_to_world(ev.offset_x() as f64, ev.offset_y() as f64);
        log::debug!("mousedown at world ({:.1}, {:.1})", wx, wy);
    };

    let state_move = state.clone();
    let on_mousemove = move |ev: web_sys::MouseEvent| {
        let sx = ev.offset_x() as f64;
        let sy = ev.offset_y() as f64;
        state_move.mouse_pos.set((sx, sy));
    };

    let on_mouseup = move |_ev: web_sys::MouseEvent| {
        log::debug!("mouseup");
    };

    let state_wheel = state.clone();
    let on_wheel = move |ev: web_sys::WheelEvent| {
        ev.prevent_default();
        let delta = ev.delta_y();
        let zoom_factor = if delta < 0.0 { 1.1 } else { 1.0 / 1.1 };
        state_wheel.camera.update(|cam| {
            let new_zoom = (cam.zoom * zoom_factor).clamp(0.1, 10.0);
            // Zoom toward cursor position.
            let sx = ev.offset_x() as f64;
            let sy = ev.offset_y() as f64;
            let wx = (sx / cam.zoom) + cam.x;
            let wy = (sy / cam.zoom) + cam.y;
            cam.zoom = new_zoom;
            cam.x = wx - sx / new_zoom;
            cam.y = wy - sy / new_zoom;
        });
    };

    view! {
        <canvas
            node_ref=canvas_ref
            style="width: 100%; height: 100%; display: block;"
            on:mousedown=on_mousedown
            on:mousemove=on_mousemove
            on:mouseup=on_mouseup
            on:wheel=on_wheel
        />
    }
}
