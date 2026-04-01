use std::f64::consts::PI;

use leptos::prelude::*;
use wasm_bindgen::JsValue;
use web_sys::CanvasRenderingContext2d;

use shared::point::Point;
use shared::ElementKind;
use stdb_client::ElementData;
use spacetimedb_lib::bsatn;

use crate::state::AppState;
use crate::types::{Tool, VimMode};

/// Generate a locally-unique element ID from timestamp + random.
fn generate_local_id() -> u64 {
    let now = js_sys::Date::now() as u64;
    let random = (js_sys::Math::random() * 1_000_000.0) as u64;
    now * 1_000_000 + random
}

/// Minimum distance from a point to a line segment.
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

/// Current drawing interaction state.
#[derive(Clone, Debug)]
pub enum DrawingState {
    None,
    ShapeDrag { start_x: f64, start_y: f64 },
    LinePlacement { points: Vec<Point> },
    TextInput { x: f64, y: f64, text: String },
    FreehandDraw { points: Vec<Point> },
}

/// Handles tool interactions on the canvas.
pub struct ToolHandler {
    pub drawing: DrawingState,
}

impl ToolHandler {
    pub fn new() -> Self {
        Self {
            drawing: DrawingState::None,
        }
    }

    /// Handle mouse-down at world coordinates (wx, wy).
    pub fn on_mouse_down(&mut self, state: &AppState, wx: f64, wy: f64) {
        let mode = state.mode.get_untracked();
        let tool = state.tool.get_untracked();

        match mode {
            VimMode::Normal => {
                if tool == Tool::Select {
                    // Hit test: select topmost element under cursor.
                    let hit = hit_test(state, wx, wy);
                    if let Some(id) = hit {
                        state.selected_ids.set(vec![id]);
                    } else {
                        state.selected_ids.set(vec![]);
                    }
                }
            }
            VimMode::Insert => match tool {
                Tool::Rectangle | Tool::Ellipse => {
                    self.drawing = DrawingState::ShapeDrag {
                        start_x: wx,
                        start_y: wy,
                    };
                }
                Tool::Arrow | Tool::Line => {
                    match &mut self.drawing {
                        DrawingState::LinePlacement { points } => {
                            // Second click: finish the line.
                            points.push(Point { x: wx, y: wy });
                            let pts = points.clone();
                            let kind = if tool == Tool::Arrow {
                                ElementKind::Arrow
                            } else {
                                ElementKind::Line
                            };
                            create_line_element(state, kind, &pts);
                            self.drawing = DrawingState::None;
                        }
                        _ => {
                            // First click: start placement.
                            self.drawing = DrawingState::LinePlacement {
                                points: vec![Point { x: wx, y: wy }],
                            };
                        }
                    }
                }
                Tool::Text => {
                    // If already in text input, commit the current text first.
                    if let DrawingState::TextInput { x, y, ref text } = self.drawing {
                        if !text.is_empty() {
                            let t = text.clone();
                            create_text_element(state, x, y, &t);
                        }
                    }
                    self.drawing = DrawingState::TextInput {
                        x: wx,
                        y: wy,
                        text: String::new(),
                    };
                }
                Tool::Freehand => {
                    self.drawing = DrawingState::FreehandDraw {
                        points: vec![Point { x: wx, y: wy }],
                    };
                }
                Tool::Select => {
                    let hit = hit_test(state, wx, wy);
                    if let Some(id) = hit {
                        state.selected_ids.set(vec![id]);
                    } else {
                        state.selected_ids.set(vec![]);
                    }
                }
            },
            _ => {}
        }
    }

    /// Handle mouse-move at world coordinates.
    pub fn on_mouse_move(&mut self, _state: &AppState, wx: f64, wy: f64) {
        match &mut self.drawing {
            DrawingState::FreehandDraw { points } => {
                points.push(Point { x: wx, y: wy });
            }
            _ => {}
        }
    }

    /// Handle mouse-up at world coordinates. Finalizes shapes.
    pub fn on_mouse_up(&mut self, state: &AppState, wx: f64, wy: f64) {
        match &self.drawing {
            DrawingState::ShapeDrag { start_x, start_y } => {
                let sx = *start_x;
                let sy = *start_y;
                let tool = state.tool.get_untracked();
                let kind = if tool == Tool::Ellipse {
                    ElementKind::Ellipse
                } else {
                    ElementKind::Rectangle
                };
                let x = sx.min(wx);
                let y = sy.min(wy);
                let w = (wx - sx).abs();
                let h = (wy - sy).abs();
                if w > 1.0 || h > 1.0 {
                    create_shape_element(state, kind, x, y, w, h);
                }
                self.drawing = DrawingState::None;
            }
            DrawingState::FreehandDraw { points } => {
                if points.len() >= 2 {
                    create_freehand_element(state, points);
                }
                self.drawing = DrawingState::None;
            }
            _ => {
                // LinePlacement and TextInput are not finalized on mouse-up.
            }
        }
    }

    /// Handle keyboard input during text mode.
    pub fn on_text_key(&mut self, state: &AppState, key: &str) {
        if let DrawingState::TextInput { x, y, ref mut text } = self.drawing {
            match key {
                "Escape" | "Enter" => {
                    if !text.is_empty() {
                        let t = text.clone();
                        create_text_element(state, x, y, &t);
                    }
                    self.drawing = DrawingState::None;
                }
                "Backspace" => {
                    text.pop();
                }
                _ if key.len() == 1 => {
                    text.push_str(key);
                }
                _ => {}
            }
        }
    }

    /// Draw a ghost preview of the shape currently being drawn.
    pub fn render_preview(&self, ctx: &CanvasRenderingContext2d, state: &AppState) {
        let dash_array = js_sys::Array::new();
        dash_array.push(&JsValue::from(6.0));
        dash_array.push(&JsValue::from(4.0));

        match &self.drawing {
            DrawingState::ShapeDrag { start_x, start_y } => {
                let (mx, my) = state.mouse_pos.get_untracked();
                let (wx, wy) = state.screen_to_world(mx, my);
                let x = start_x.min(wx);
                let y = start_y.min(wy);
                let w = (wx - start_x).abs();
                let h = (wy - start_y).abs();

                ctx.set_stroke_style_str("rgba(255,255,255,0.6)");
                ctx.set_line_width(1.5);
                let _ = ctx.set_line_dash(&dash_array);

                let tool = state.tool.get_untracked();
                if tool == Tool::Ellipse {
                    let cx = x + w / 2.0;
                    let cy = y + h / 2.0;
                    let rx = w / 2.0;
                    let ry = h / 2.0;
                    ctx.begin_path();
                    let _ = ctx.ellipse(cx, cy, rx.max(0.1), ry.max(0.1), 0.0, 0.0, PI * 2.0);
                    ctx.stroke();
                } else {
                    ctx.stroke_rect(x, y, w, h);
                }

                let _ = ctx.set_line_dash(&js_sys::Array::new());
            }
            DrawingState::LinePlacement { points } => {
                if let Some(start) = points.first() {
                    let (mx, my) = state.mouse_pos.get_untracked();
                    let (wx, wy) = state.screen_to_world(mx, my);

                    ctx.set_stroke_style_str("rgba(255,255,255,0.6)");
                    ctx.set_line_width(1.5);
                    let _ = ctx.set_line_dash(&dash_array);

                    ctx.begin_path();
                    ctx.move_to(start.x, start.y);
                    ctx.line_to(wx, wy);
                    ctx.stroke();

                    let _ = ctx.set_line_dash(&js_sys::Array::new());
                }
            }
            DrawingState::TextInput { x, y, text } => {
                let font_size = shared::DEFAULT_FONT_SIZE;
                ctx.set_font(&format!("{}px monospace", font_size));
                ctx.set_fill_style_str("rgba(255,255,255,0.8)");
                let display = if text.is_empty() {
                    "|".to_string()
                } else {
                    format!("{}|", text)
                };
                let _ = ctx.fill_text(&display, *x, *y + font_size as f64);
            }
            DrawingState::FreehandDraw { points } => {
                if points.len() >= 2 {
                    ctx.set_stroke_style_str("rgba(255,255,255,0.6)");
                    ctx.set_line_width(1.5);
                    let _ = ctx.set_line_dash(&dash_array);

                    ctx.begin_path();
                    ctx.move_to(points[0].x, points[0].y);
                    for p in &points[1..] {
                        ctx.line_to(p.x, p.y);
                    }
                    ctx.stroke();

                    let _ = ctx.set_line_dash(&js_sys::Array::new());
                }
            }
            DrawingState::None => {}
        }
    }
}

// ---------------------------------------------------------------------------
// Hit testing
// ---------------------------------------------------------------------------

/// Find the topmost element under (wx, wy). Returns its ID if found.
fn hit_test(state: &AppState, wx: f64, wy: f64) -> Option<u64> {
    let elements = state.store.sorted_elements();
    // Iterate in reverse (topmost z-index first).
    for elem in elements.iter().rev() {
        if hit_test_element(elem, wx, wy) {
            return Some(elem.id);
        }
    }
    None
}

fn hit_test_element(elem: &ElementData, wx: f64, wy: f64) -> bool {
    match elem.kind {
        ElementKind::Rectangle | ElementKind::Text => {
            wx >= elem.x
                && wx <= elem.x + elem.width
                && wy >= elem.y
                && wy <= elem.y + elem.height
        }
        ElementKind::Ellipse => {
            let cx = elem.x + elem.width / 2.0;
            let cy = elem.y + elem.height / 2.0;
            let rx = elem.width / 2.0;
            let ry = elem.height / 2.0;
            if rx.abs() < 0.01 || ry.abs() < 0.01 {
                return false;
            }
            let dx = (wx - cx) / rx;
            let dy = (wy - cy) / ry;
            dx * dx + dy * dy <= 1.0
        }
        ElementKind::Arrow | ElementKind::Line | ElementKind::Freehand => {
            let threshold = 8.0;
            let pts = &elem.points;
            if pts.len() < 2 {
                return false;
            }
            for i in 0..pts.len() - 1 {
                let dist = point_to_segment_dist(
                    wx, wy, pts[i].x, pts[i].y, pts[i + 1].x, pts[i + 1].y,
                );
                if dist <= threshold {
                    return true;
                }
            }
            false
        }
    }
}

// ---------------------------------------------------------------------------
// Sync helper: call the create_element reducer on the server
// ---------------------------------------------------------------------------

fn sync_element_to_server(state: &AppState, elem: &ElementData) {
    let conn = state.connection.get_untracked();
    if let Some(conn) = conn {
        let points_bytes = shared::encode_points(&elem.points);
        // BSATN product encoding: concatenate each field's serialization.
        // The tuple Serialize impl only supports up to 12 elements, so we
        // manually concatenate for the 13-arg create_element reducer.
        let mut args = Vec::new();
        bsatn::to_writer(&mut args, &elem.room_id).unwrap();
        bsatn::to_writer(&mut args, &elem.kind).unwrap();
        bsatn::to_writer(&mut args, &elem.x).unwrap();
        bsatn::to_writer(&mut args, &elem.y).unwrap();
        bsatn::to_writer(&mut args, &elem.width).unwrap();
        bsatn::to_writer(&mut args, &elem.height).unwrap();
        bsatn::to_writer(&mut args, &elem.stroke_color).unwrap();
        bsatn::to_writer(&mut args, &elem.fill_color).unwrap();
        bsatn::to_writer(&mut args, &elem.stroke_width).unwrap();
        bsatn::to_writer(&mut args, &elem.opacity).unwrap();
        bsatn::to_writer(&mut args, &elem.font_size).unwrap();
        bsatn::to_writer(&mut args, &elem.text_content.clone()).unwrap();
        bsatn::to_writer(&mut args, &points_bytes).unwrap();
        conn.call_reducer("create_element", args);
    }
}

// ---------------------------------------------------------------------------
// Element creation helpers
// ---------------------------------------------------------------------------

fn next_z_index(state: &AppState) -> i32 {
    state
        .store
        .elements
        .with_untracked(|elems| elems.values().map(|e| e.z_index).max().unwrap_or(0) + 1)
}

fn create_shape_element(state: &AppState, kind: ElementKind, x: f64, y: f64, w: f64, h: f64) {
    let id = generate_local_id();
    let room_id = state.store.current_room.get_untracked().unwrap_or(0);
    let z = next_z_index(state);
    let elem = ElementData {
        id,
        room_id,
        kind,
        x,
        y,
        width: w,
        height: h,
        rotation: 0.0,
        points: vec![],
        stroke_color: state.stroke_color.get_untracked(),
        fill_color: state.fill_color.get_untracked(),
        stroke_width: state.stroke_width.get_untracked(),
        opacity: shared::DEFAULT_OPACITY,
        font_size: shared::DEFAULT_FONT_SIZE,
        text_content: String::new(),
        z_index: z,
    };
    sync_element_to_server(state, &elem);
    state.store.elements.update(|elems| {
        elems.insert(id, elem);
    });
    state.selected_ids.set(vec![id]);
}

fn create_line_element(state: &AppState, kind: ElementKind, points: &[Point]) {
    let id = generate_local_id();
    let room_id = state.store.current_room.get_untracked().unwrap_or(0);
    let z = next_z_index(state);
    let elem = ElementData {
        id,
        room_id,
        kind,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        rotation: 0.0,
        points: points.to_vec(),
        stroke_color: state.stroke_color.get_untracked(),
        fill_color: state.fill_color.get_untracked(),
        stroke_width: state.stroke_width.get_untracked(),
        opacity: shared::DEFAULT_OPACITY,
        font_size: shared::DEFAULT_FONT_SIZE,
        text_content: String::new(),
        z_index: z,
    };
    sync_element_to_server(state, &elem);
    state.store.elements.update(|elems| {
        elems.insert(id, elem);
    });
    state.selected_ids.set(vec![id]);
}

fn create_text_element(state: &AppState, x: f64, y: f64, text: &str) {
    let id = generate_local_id();
    let room_id = state.store.current_room.get_untracked().unwrap_or(0);
    let z = next_z_index(state);
    let font_size = shared::DEFAULT_FONT_SIZE;
    // Rough bounding box estimate: width ~ chars * font_size * 0.6, height ~ font_size * 1.2
    let w = text.len() as f64 * font_size as f64 * 0.6;
    let h = font_size as f64 * 1.2;
    let elem = ElementData {
        id,
        room_id,
        kind: ElementKind::Text,
        x,
        y,
        width: w,
        height: h,
        rotation: 0.0,
        points: vec![],
        stroke_color: state.stroke_color.get_untracked(),
        fill_color: state.fill_color.get_untracked(),
        stroke_width: state.stroke_width.get_untracked(),
        opacity: shared::DEFAULT_OPACITY,
        font_size,
        text_content: text.to_string(),
        z_index: z,
    };
    sync_element_to_server(state, &elem);
    state.store.elements.update(|elems| {
        elems.insert(id, elem);
    });
    state.selected_ids.set(vec![id]);
}

fn create_freehand_element(state: &AppState, points: &[Point]) {
    let id = generate_local_id();
    let room_id = state.store.current_room.get_untracked().unwrap_or(0);
    let z = next_z_index(state);
    let elem = ElementData {
        id,
        room_id,
        kind: ElementKind::Freehand,
        x: 0.0,
        y: 0.0,
        width: 0.0,
        height: 0.0,
        rotation: 0.0,
        points: points.to_vec(),
        stroke_color: state.stroke_color.get_untracked(),
        fill_color: state.fill_color.get_untracked(),
        stroke_width: state.stroke_width.get_untracked(),
        opacity: shared::DEFAULT_OPACITY,
        font_size: shared::DEFAULT_FONT_SIZE,
        text_content: String::new(),
        z_index: z,
    };
    sync_element_to_server(state, &elem);
    state.store.elements.update(|elems| {
        elems.insert(id, elem);
    });
    state.selected_ids.set(vec![id]);
}
