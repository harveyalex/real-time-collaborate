use spacetimedb::{Identity, Timestamp};
use shared::types::{ElementKind, UndoAction};

#[spacetimedb::table(accessor = room, public)]
pub struct Room {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    pub name: String,
    pub created_by: Identity,
    pub created_at: Timestamp,
}

#[spacetimedb::table(accessor = element, public)]
pub struct Element {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub room_id: u64,
    pub kind: ElementKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub points: Vec<u8>,
    pub stroke_color: u32,
    pub fill_color: u32,
    pub stroke_width: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub text_content: String,
    pub z_index: i32,
    pub version: u64,
    pub updated_by: Identity,
    pub deleted: bool,
}

#[spacetimedb::table(accessor = cursor, public)]
pub struct Cursor {
    #[primary_key]
    pub user_id: Identity,
    pub room_id: u64,
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub color: u32,
}

#[spacetimedb::table(accessor = undo_entry, public)]
pub struct UndoEntry {
    #[primary_key]
    #[auto_inc]
    pub id: u64,
    #[index(btree)]
    pub room_id: u64,
    pub user_id: Identity,
    pub action: UndoAction,
    pub element_id: u64,
    pub prev_state: Vec<u8>,
    pub next_state: Vec<u8>,
    pub timestamp: Timestamp,
    pub undone: bool,
}
