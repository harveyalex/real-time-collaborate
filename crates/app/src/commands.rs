use leptos::prelude::{GetUntracked, Set, Update, WithUntracked};

use shared::point::Point;
use shared::ElementKind;
use stdb_client::ElementData;

use crate::state::{AppState, Tool};
use crate::tools::{
    create_shape_element, create_line_element, create_freehand_element,
    generate_local_id, next_z_index, sync_element_to_server, DrawingState,
};
use crate::vim::VimAction;

/// Process a VimAction and apply it to the app state.
pub fn handle_vim_action(state: &AppState, action: VimAction) {
    match action {
        VimAction::None => {}
        VimAction::SetTool(tool) => {
            // Commit any in-progress freehand before switching tools
            {
                let mut th = state.tool_handler.lock().unwrap();
                if let DrawingState::FreehandDraw { points } = &th.drawing {
                    if points.len() >= 2 {
                        let pts = points.clone();
                        th.drawing = DrawingState::None;
                        drop(th);
                        create_freehand_element(state, &pts);
                    } else {
                        th.drawing = DrawingState::None;
                    }
                }
            }
            state.tool.set(tool);
        }
        VimAction::MoveSelected(dx, dy) => {
            // TODO: call update_element reducer for each selected element
            log::debug!("Move selected by ({}, {})", dx, dy);
        }
        VimAction::MoveCursor(dx, dy) => {
            state.vim_cursor.update(|pos| {
                pos.0 += dx;
                pos.1 += dy;
            });
            // If freehand drawing, append new cursor position to path
            let mut th = state.tool_handler.lock().unwrap();
            if let DrawingState::FreehandDraw { ref mut points } = th.drawing {
                let (nx, ny) = state.vim_cursor.get_untracked();
                points.push(Point { x: nx, y: ny });
            }
        }
        VimAction::SelectAtCursor => {
            let (cx, cy) = state.vim_cursor.get_untracked();
            let hit = crate::tools::hit_test(state, cx, cy);
            if let Some(id) = hit {
                state.selected_ids.set(vec![id]);
            } else {
                state.selected_ids.set(vec![]);
            }
        }
        VimAction::DeleteSelected => {
            let selected = state.selected_ids.get_untracked();
            // Remove locally
            state.store.elements.update(|elems| {
                for id in &selected {
                    elems.remove(id);
                }
            });
            // Call server reducer
            if let Some(conn) = state.connection.get_untracked() {
                for id in &selected {
                    let args = spacetimedb_lib::bsatn::to_vec(&(*id,)).unwrap();
                    conn.call_reducer("delete_element", args);
                }
            }
            state.selected_ids.update(|ids: &mut Vec<u64>| ids.clear());
        }
        VimAction::CopySelected => {
            let selected = state.selected_ids.get_untracked();
            let copied: Vec<ElementData> = state.store.elements.with_untracked(|elems| {
                selected.iter().filter_map(|id| elems.get(id).cloned()).collect()
            });
            state.clipboard.set(copied);
        }
        VimAction::Paste => {
            let clipboard = state.clipboard.get_untracked();
            let mut new_ids = vec![];
            for elem in &clipboard {
                let id = generate_local_id();
                let z = next_z_index(state);
                let new_elem = ElementData {
                    id,
                    room_id: elem.room_id,
                    kind: elem.kind,
                    x: elem.x + 20.0,
                    y: elem.y + 20.0,
                    width: elem.width,
                    height: elem.height,
                    rotation: elem.rotation,
                    points: elem.points.clone(),
                    stroke_color: elem.stroke_color,
                    fill_color: elem.fill_color,
                    stroke_width: elem.stroke_width,
                    opacity: elem.opacity,
                    font_size: elem.font_size,
                    text_content: elem.text_content.clone(),
                    z_index: z,
                };
                sync_element_to_server(state, &new_elem);
                state.store.elements.update(|elems| { elems.insert(id, new_elem); });
                new_ids.push(id);
            }
            if !new_ids.is_empty() {
                state.selected_ids.set(new_ids);
            }
        }
        VimAction::Undo => {
            if let Some(conn) = state.connection.get_untracked() {
                let room_id = state.store.current_room.get_untracked().unwrap_or(1);
                let args = spacetimedb_lib::bsatn::to_vec(&(room_id,)).unwrap();
                conn.call_reducer("undo", args);
            }
        }
        VimAction::Redo => {
            if let Some(conn) = state.connection.get_untracked() {
                let room_id = state.store.current_room.get_untracked().unwrap_or(1);
                let args = spacetimedb_lib::bsatn::to_vec(&(room_id,)).unwrap();
                conn.call_reducer("redo", args);
            }
        }
        VimAction::EnterCommand => {
            state.mode.set(crate::state::VimMode::Command);
        }
        VimAction::EnterVisual => {
            state.mode.set(crate::state::VimMode::Visual);
        }
        VimAction::ExitToNormal => {
            // Commit any in-progress drawing before exiting
            {
                let mut th = state.tool_handler.lock().unwrap();
                match &th.drawing {
                    DrawingState::FreehandDraw { points } if points.len() >= 2 => {
                        let pts = points.clone();
                        th.drawing = DrawingState::None;
                        drop(th);
                        create_freehand_element(state, &pts);
                    }
                    DrawingState::LinePlacement { points } if !points.is_empty() => {
                        // Discard incomplete line on Escape
                        th.drawing = DrawingState::None;
                    }
                    _ => {
                        th.drawing = DrawingState::None;
                    }
                }
            }
            state.mode.set(crate::state::VimMode::Normal);
            state.selected_ids.update(|ids: &mut Vec<u64>| ids.clear());
        }
        VimAction::ToggleHelp => {
            state.show_help.update(|v| *v = !*v);
        }
        VimAction::CreateAtCenter => {
            let tool = state.tool.get_untracked();
            let (cx, cy) = state.vim_cursor.get_untracked();
            match tool {
                Tool::Rectangle => {
                    create_shape_element(state, ElementKind::Rectangle, cx - 50.0, cy - 30.0, 100.0, 60.0);
                }
                Tool::Ellipse => {
                    create_shape_element(state, ElementKind::Ellipse, cx - 50.0, cy - 30.0, 100.0, 60.0);
                }
                Tool::Text => {
                    let mut th = state.tool_handler.lock().unwrap();
                    th.drawing = DrawingState::TextInput { x: cx, y: cy, text: String::new() };
                }
                Tool::Arrow | Tool::Line => {
                    let mut th = state.tool_handler.lock().unwrap();
                    match &th.drawing {
                        DrawingState::LinePlacement { points } if !points.is_empty() => {
                            // Second Enter: finish at vim cursor
                            let mut pts = points.clone();
                            pts.push(Point { x: cx, y: cy });
                            let kind = if tool == Tool::Arrow { ElementKind::Arrow } else { ElementKind::Line };
                            th.drawing = DrawingState::None;
                            drop(th); // release lock before calling create
                            create_line_element(state, kind, &pts);
                        }
                        _ => {
                            // First Enter: start at vim cursor
                            th.drawing = DrawingState::LinePlacement {
                                points: vec![Point { x: cx, y: cy }],
                            };
                        }
                    }
                }
                Tool::Freehand => {
                    let mut th = state.tool_handler.lock().unwrap();
                    match &th.drawing {
                        DrawingState::FreehandDraw { points } if points.len() >= 2 => {
                            // Enter while drawing: commit current stroke
                            let pts = points.clone();
                            th.drawing = DrawingState::None;
                            drop(th);
                            create_freehand_element(state, &pts);
                        }
                        _ => {
                            // First Enter: start recording freehand at vim cursor
                            th.drawing = DrawingState::FreehandDraw {
                                points: vec![Point { x: cx, y: cy }],
                            };
                        }
                    }
                }
                _ => {}
            }
        }
        VimAction::SelectNext => {
            let current = state.selected_ids.get_untracked();
            let elements = state.store.sorted_elements();
            if !elements.is_empty() {
                let current_idx = if let Some(&id) = current.first() {
                    elements.iter().position(|e| e.id == id).map(|i| i + 1).unwrap_or(0)
                } else {
                    0
                };
                let next_idx = current_idx % elements.len();
                state.selected_ids.set(vec![elements[next_idx].id]);
            }
        }
        VimAction::ZoomIn => {
            state.camera.update(|cam| { cam.zoom = (cam.zoom * 1.2).clamp(0.1, 10.0); });
        }
        VimAction::ZoomOut => {
            state.camera.update(|cam| { cam.zoom = (cam.zoom / 1.2).clamp(0.1, 10.0); });
        }
        VimAction::ResizeSelected(dw, dh) => {
            let selected = state.selected_ids.get_untracked();
            state.store.elements.update(|elems| {
                for &id in &selected {
                    if let Some(elem) = elems.get_mut(&id) {
                        elem.width = (elem.width + dw).max(10.0);
                        elem.height = (elem.height + dh).max(10.0);
                    }
                }
            });
        }
        VimAction::RotateSelected(angle) => {
            let selected = state.selected_ids.get_untracked();
            state.store.elements.update(|elems| {
                for &id in &selected {
                    if let Some(elem) = elems.get_mut(&id) {
                        elem.rotation += angle;
                    }
                }
            });
        }
        VimAction::PanCamera(dx, dy) => {
            state.camera.update(|cam| {
                cam.x += dx;
                cam.y += dy;
            });
        }
        VimAction::CommandChar(_) | VimAction::CommandBackspace => {
            // Command buffer is managed by VimStateMachine
        }
        VimAction::CommandSubmit(cmd) => {
            handle_command(state, &cmd);
        }
    }
}

fn handle_command(state: &AppState, cmd: &str) {
    let parts: Vec<&str> = cmd.trim().split_whitespace().collect();
    match parts.as_slice() {
        ["w"] => {
            crate::export::export_png(state);
        }
        ["ws"] => {
            crate::export::export_svg(state);
        }
        ["wq"] => {
            // TODO: export and close
            log::info!("Export and close");
        }
        ["color", hex] => {
            if let Some(color) = parse_hex_color(hex) {
                state.stroke_color.set(color);
            }
        }
        ["fill", hex] => {
            if let Some(color) = parse_hex_color(hex) {
                state.fill_color.set(color);
            }
        }
        ["stroke", width] => {
            if let Ok(w) = width.parse::<f32>() {
                state.stroke_width.set(w);
            }
        }
        _ => {
            log::warn!("Unknown command: {}", cmd);
        }
    }
}

fn parse_hex_color(hex: &str) -> Option<u32> {
    let hex = hex.trim_start_matches('#');
    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1].repeat(2), 16).ok()?;
            let g = u8::from_str_radix(&hex[1..2].repeat(2), 16).ok()?;
            let b = u8::from_str_radix(&hex[2..3].repeat(2), 16).ok()?;
            Some(u32::from_be_bytes([r, g, b, 0xFF]))
        }
        6 => {
            let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some(u32::from_be_bytes([r, g, b, 0xFF]))
        }
        _ => None,
    }
}
