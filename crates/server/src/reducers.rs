use spacetimedb::{ReducerContext, Identity, Table};
use crate::tables::*;
use shared::types::UndoAction;

// ---------------------------------------------------------------------------
// Element snapshot — used for undo/redo state storage
// ---------------------------------------------------------------------------

#[derive(serde::Serialize, serde::Deserialize)]
struct ElementSnapshot {
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    points: Vec<u8>,
    stroke_color: u32,
    fill_color: u32,
    stroke_width: f32,
    opacity: f32,
    font_size: f32,
    text_content: String,
}

// ---------------------------------------------------------------------------
// Helper functions
// ---------------------------------------------------------------------------

fn cursor_color_from_identity(id: Identity) -> u32 {
    let bytes = id.to_byte_array();
    let r = bytes[0] as u32;
    let g = bytes[1] as u32;
    let b = bytes[2] as u32;
    // Bright colours only: ensure each channel is >= 0x80
    let r = (r | 0x80) & 0xFF;
    let g = (g | 0x80) & 0xFF;
    let b = (b | 0x80) & 0xFF;
    0xFF000000 | (r << 16) | (g << 8) | b
}

fn next_z_index(ctx: &ReducerContext, room_id: u64) -> i32 {
    ctx.db
        .element()
        .room_id()
        .filter(&room_id)
        .filter_map(|e| if !e.deleted { Some(e.z_index) } else { None })
        .max()
        .map(|m| m + 1)
        .unwrap_or(0)
}

fn serialize_element(elem: &Element) -> Vec<u8> {
    let snap = ElementSnapshot {
        x: elem.x,
        y: elem.y,
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
    };
    serde_json::to_vec(&snap).unwrap_or_default()
}

// ---------------------------------------------------------------------------
// Lifecycle reducers
// ---------------------------------------------------------------------------

#[spacetimedb::reducer(client_connected)]
pub fn client_connected(ctx: &ReducerContext) {
    log::info!("Client connected: {:?}", ctx.sender());
}

#[spacetimedb::reducer(client_disconnected)]
pub fn client_disconnected(ctx: &ReducerContext) {
    log::info!("Client disconnected: {:?}", ctx.sender());
    ctx.db.cursor().user_id().delete(&ctx.sender());
}

// ---------------------------------------------------------------------------
// Room management
// ---------------------------------------------------------------------------

#[spacetimedb::reducer]
pub fn create_room(ctx: &ReducerContext, name: String) -> Result<(), String> {
    if name.trim().is_empty() {
        return Err("Room name must not be empty".to_string());
    }
    ctx.db.room().insert(Room {
        id: 0, // auto_inc
        name,
        created_by: ctx.sender(),
        created_at: ctx.timestamp,
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Cursor / presence
// ---------------------------------------------------------------------------

#[spacetimedb::reducer]
pub fn join_room(ctx: &ReducerContext, room_id: u64, name: String) -> Result<(), String> {
    if ctx.db.room().id().find(room_id).is_none() {
        return Err(format!("Room {} does not exist", room_id));
    }
    let color = cursor_color_from_identity(ctx.sender());
    // Upsert: delete existing cursor for this user, then insert fresh one.
    ctx.db.cursor().user_id().delete(&ctx.sender());
    ctx.db.cursor().insert(Cursor {
        user_id: ctx.sender(),
        room_id,
        x: 0.0,
        y: 0.0,
        name,
        color,
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn update_cursor(ctx: &ReducerContext, room_id: u64, x: f64, y: f64) -> Result<(), String> {
    match ctx.db.cursor().user_id().find(ctx.sender()) {
        Some(existing) => {
            ctx.db.cursor().user_id().update(Cursor {
                room_id,
                x,
                y,
                ..existing
            });
            Ok(())
        }
        None => Err("Not in any room; call join_room first".to_string()),
    }
}

// ---------------------------------------------------------------------------
// Element CRUD
// ---------------------------------------------------------------------------

#[spacetimedb::reducer]
#[allow(clippy::too_many_arguments)]
pub fn create_element(
    ctx: &ReducerContext,
    room_id: u64,
    kind: shared::types::ElementKind,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    stroke_color: u32,
    fill_color: u32,
    stroke_width: f32,
    opacity: f32,
    font_size: f32,
    text_content: String,
    points: Vec<u8>,
) -> Result<(), String> {
    if ctx.db.room().id().find(room_id).is_none() {
        return Err(format!("Room {} does not exist", room_id));
    }
    let z_index = next_z_index(ctx, room_id);
    let inserted = ctx.db.element().insert(Element {
        id: 0, // auto_inc
        room_id,
        kind,
        x,
        y,
        width,
        height,
        rotation: 0.0,
        points,
        stroke_color,
        fill_color,
        stroke_width,
        opacity,
        font_size,
        text_content,
        z_index,
        version: 1,
        updated_by: ctx.sender(),
        deleted: false,
    });
    let next_state = serialize_element(&inserted);
    ctx.db.undo_entry().insert(UndoEntry {
        id: 0,
        room_id,
        user_id: ctx.sender(),
        action: UndoAction::Create,
        element_id: inserted.id,
        prev_state: vec![],
        next_state,
        timestamp: ctx.timestamp,
        undone: false,
    });
    Ok(())
}

#[spacetimedb::reducer]
#[allow(clippy::too_many_arguments)]
pub fn update_element(
    ctx: &ReducerContext,
    element_id: u64,
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    rotation: f64,
    stroke_color: u32,
    fill_color: u32,
    stroke_width: f32,
    opacity: f32,
    font_size: f32,
    text_content: String,
    points: Vec<u8>,
) -> Result<(), String> {
    let existing = ctx
        .db
        .element()
        .id()
        .find(element_id)
        .ok_or_else(|| format!("Element {} not found", element_id))?;
    if existing.deleted {
        return Err(format!("Element {} has been deleted", element_id));
    }
    let prev_state = serialize_element(&existing);
    let updated = Element {
        x,
        y,
        width,
        height,
        rotation,
        points,
        stroke_color,
        fill_color,
        stroke_width,
        opacity,
        font_size,
        text_content,
        version: existing.version + 1,
        updated_by: ctx.sender(),
        ..existing
    };
    let next_state = serialize_element(&updated);
    ctx.db.undo_entry().insert(UndoEntry {
        id: 0,
        room_id: existing.room_id,
        user_id: ctx.sender(),
        action: UndoAction::Update,
        element_id,
        prev_state,
        next_state,
        timestamp: ctx.timestamp,
        undone: false,
    });
    ctx.db.element().id().update(updated);
    Ok(())
}

#[spacetimedb::reducer]
pub fn delete_element(ctx: &ReducerContext, element_id: u64) -> Result<(), String> {
    let existing = ctx
        .db
        .element()
        .id()
        .find(element_id)
        .ok_or_else(|| format!("Element {} not found", element_id))?;
    if existing.deleted {
        return Err(format!("Element {} is already deleted", element_id));
    }
    let prev_state = serialize_element(&existing);
    ctx.db.undo_entry().insert(UndoEntry {
        id: 0,
        room_id: existing.room_id,
        user_id: ctx.sender(),
        action: UndoAction::Delete,
        element_id,
        prev_state,
        next_state: vec![],
        timestamp: ctx.timestamp,
        undone: false,
    });
    ctx.db.element().id().update(Element {
        deleted: true,
        updated_by: ctx.sender(),
        version: existing.version + 1,
        ..existing
    });
    Ok(())
}

// ---------------------------------------------------------------------------
// Undo / Redo
// ---------------------------------------------------------------------------

#[spacetimedb::reducer]
pub fn undo(ctx: &ReducerContext, room_id: u64) -> Result<(), String> {
    // Most recent non-undone entry for this user in this room
    let entry = ctx
        .db
        .undo_entry()
        .room_id()
        .filter(&room_id)
        .filter(|e| e.user_id == ctx.sender() && !e.undone)
        .max_by_key(|e| e.timestamp);
    let entry = entry.ok_or("Nothing to undo")?;

    match entry.action {
        UndoAction::Create => {
            // Undo create → soft-delete
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: true,
                    updated_by: ctx.sender(),
                    version: elem.version + 1,
                    ..elem
                });
            }
        }
        UndoAction::Update => {
            // Undo update → restore prev_state
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                if let Ok(snap) =
                    serde_json::from_slice::<ElementSnapshot>(&entry.prev_state)
                {
                    ctx.db.element().id().update(Element {
                        x: snap.x,
                        y: snap.y,
                        width: snap.width,
                        height: snap.height,
                        rotation: snap.rotation,
                        points: snap.points,
                        stroke_color: snap.stroke_color,
                        fill_color: snap.fill_color,
                        stroke_width: snap.stroke_width,
                        opacity: snap.opacity,
                        font_size: snap.font_size,
                        text_content: snap.text_content,
                        version: elem.version + 1,
                        updated_by: ctx.sender(),
                        ..elem
                    });
                }
            }
        }
        UndoAction::Delete => {
            // Undo delete → undelete
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: false,
                    updated_by: ctx.sender(),
                    version: elem.version + 1,
                    ..elem
                });
            }
        }
    }

    // Mark the entry as undone
    ctx.db.undo_entry().id().update(UndoEntry {
        undone: true,
        ..entry
    });
    Ok(())
}

#[spacetimedb::reducer]
pub fn redo(ctx: &ReducerContext, room_id: u64) -> Result<(), String> {
    // Most recent undone entry for this user in this room
    let entry = ctx
        .db
        .undo_entry()
        .room_id()
        .filter(&room_id)
        .filter(|e| e.user_id == ctx.sender() && e.undone)
        .max_by_key(|e| e.timestamp);
    let entry = entry.ok_or("Nothing to redo")?;

    match entry.action {
        UndoAction::Create => {
            // Redo create → undelete
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: false,
                    updated_by: ctx.sender(),
                    version: elem.version + 1,
                    ..elem
                });
            }
        }
        UndoAction::Update => {
            // Redo update → apply next_state
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                if let Ok(snap) =
                    serde_json::from_slice::<ElementSnapshot>(&entry.next_state)
                {
                    ctx.db.element().id().update(Element {
                        x: snap.x,
                        y: snap.y,
                        width: snap.width,
                        height: snap.height,
                        rotation: snap.rotation,
                        points: snap.points,
                        stroke_color: snap.stroke_color,
                        fill_color: snap.fill_color,
                        stroke_width: snap.stroke_width,
                        opacity: snap.opacity,
                        font_size: snap.font_size,
                        text_content: snap.text_content,
                        version: elem.version + 1,
                        updated_by: ctx.sender(),
                        ..elem
                    });
                }
            }
        }
        UndoAction::Delete => {
            // Redo delete → soft-delete again
            if let Some(elem) = ctx.db.element().id().find(entry.element_id) {
                ctx.db.element().id().update(Element {
                    deleted: true,
                    updated_by: ctx.sender(),
                    version: elem.version + 1,
                    ..elem
                });
            }
        }
    }

    // Mark the entry as no longer undone
    ctx.db.undo_entry().id().update(UndoEntry {
        undone: false,
        ..entry
    });
    Ok(())
}
