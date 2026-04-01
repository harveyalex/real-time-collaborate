use leptos::prelude::{Set, Update};

use crate::state::AppState;
use crate::vim::VimAction;

/// Process a VimAction and apply it to the app state.
pub fn handle_vim_action(state: &AppState, action: VimAction) {
    match action {
        VimAction::None => {}
        VimAction::SetTool(tool) => {
            state.tool.set(tool);
        }
        VimAction::MoveSelected(dx, dy) => {
            // TODO: call update_element reducer for each selected element
            log::debug!("Move selected by ({}, {})", dx, dy);
        }
        VimAction::DeleteSelected => {
            // TODO: call delete_element reducer for each selected element
            log::debug!("Delete selected");
        }
        VimAction::CopySelected => {
            // TODO: copy selected elements to clipboard signal
            log::debug!("Copy selected");
        }
        VimAction::Paste => {
            // TODO: paste from clipboard signal
            log::debug!("Paste");
        }
        VimAction::Undo => {
            // TODO: call undo reducer
            log::debug!("Undo");
        }
        VimAction::Redo => {
            // TODO: call redo reducer
            log::debug!("Redo");
        }
        VimAction::EnterCommand => {
            state.mode.set(crate::state::VimMode::Command);
        }
        VimAction::EnterVisual => {
            state.mode.set(crate::state::VimMode::Visual);
        }
        VimAction::ExitToNormal => {
            state.mode.set(crate::state::VimMode::Normal);
            state.selected_ids.update(|ids: &mut Vec<u64>| ids.clear());
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
