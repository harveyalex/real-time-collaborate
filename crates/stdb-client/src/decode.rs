//! BSATN row decoding for SpacetimeDB server events.
//!
//! The SpacetimeDB wire protocol sends rows as packed BSATN bytes. Each row in
//! a `BsatnRowList` is a BSATN-encoded product (struct) where fields are
//! serialized in the exact order they appear in the server's table definition.
//!
//! We use `bsatn::from_reader` with `&mut &[u8]` (which implements `BufReader`)
//! to read fields sequentially, advancing the cursor after each field.

use shared::point::Point;
use shared::ElementKind;
use spacetimedb_lib::{bsatn, Identity};

use crate::signals::{CursorData, ElementData};

/// Decode a full Element row from BSATN bytes.
///
/// Returns `Ok(None)` if the element has `deleted == true`.
///
/// **Element table field order** (from `crates/server/src/tables.rs`):
/// ```text
/// id: u64, room_id: u64, kind: ElementKind, x: f64, y: f64, width: f64,
/// height: f64, rotation: f64, points: Vec<u8>, stroke_color: u32,
/// fill_color: u32, stroke_width: f32, opacity: f32, font_size: f32,
/// text_content: String, z_index: i32, version: u64, updated_by: Identity,
/// deleted: bool
/// ```
pub fn decode_element(row_bytes: &[u8]) -> Result<Option<ElementData>, String> {
    let reader = &mut &row_bytes[..];

    let id: u64 = read_field(reader, "id")?;
    let room_id: u64 = read_field(reader, "room_id")?;
    let kind: ElementKind = read_field(reader, "kind")?;
    let x: f64 = read_field(reader, "x")?;
    let y: f64 = read_field(reader, "y")?;
    let width: f64 = read_field(reader, "width")?;
    let height: f64 = read_field(reader, "height")?;
    let rotation: f64 = read_field(reader, "rotation")?;
    let points_raw: Vec<u8> = read_field(reader, "points")?;
    let stroke_color: u32 = read_field(reader, "stroke_color")?;
    let fill_color: u32 = read_field(reader, "fill_color")?;
    let stroke_width: f32 = read_field(reader, "stroke_width")?;
    let opacity: f32 = read_field(reader, "opacity")?;
    let font_size: f32 = read_field(reader, "font_size")?;
    let text_content: String = read_field(reader, "text_content")?;
    let z_index: i32 = read_field(reader, "z_index")?;
    let _version: u64 = read_field(reader, "version")?;
    let _updated_by: Identity = read_field(reader, "updated_by")?;
    let deleted: bool = read_field(reader, "deleted")?;

    if deleted {
        return Ok(None);
    }

    // Decode the compact point encoding used by shared::point
    let points: Vec<Point> = if points_raw.is_empty() {
        Vec::new()
    } else {
        shared::decode_points(&points_raw).map_err(|e| format!("decode_points: {e}"))?
    };

    Ok(Some(ElementData {
        id,
        room_id,
        kind,
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
        z_index,
    }))
}

/// Decode a full Cursor row from BSATN bytes.
///
/// Returns `(identity_hex_key, CursorData)`.
///
/// **Cursor table field order:**
/// ```text
/// user_id: Identity, room_id: u64, x: f64, y: f64, name: String, color: u32
/// ```
pub fn decode_cursor(row_bytes: &[u8]) -> Result<(String, CursorData), String> {
    let reader = &mut &row_bytes[..];

    let user_id: Identity = read_field(reader, "user_id")?;
    let _room_id: u64 = read_field(reader, "room_id")?;
    let x: f64 = read_field(reader, "x")?;
    let y: f64 = read_field(reader, "y")?;
    let name: String = read_field(reader, "name")?;
    let color: u32 = read_field(reader, "color")?;

    let identity_hex = user_id.to_hex().to_string();

    Ok((identity_hex, CursorData { x, y, name, color }))
}

/// Decode just the element `id` (first field) from BSATN row bytes.
/// Used for processing deletes where we only need the primary key.
pub fn decode_element_id(row_bytes: &[u8]) -> Result<u64, String> {
    let reader = &mut &row_bytes[..];
    read_field(reader, "id")
}

/// Decode just the cursor `user_id` (first field) from BSATN row bytes.
/// Used for processing deletes where we only need the primary key.
pub fn decode_cursor_key(row_bytes: &[u8]) -> Result<String, String> {
    let reader = &mut &row_bytes[..];
    let user_id: Identity = read_field(reader, "user_id")?;
    Ok(user_id.to_hex().to_string())
}

/// Helper: read a single BSATN-deserializable field from a reader, with a
/// field name for error context.
fn read_field<'de, T: spacetimedb_lib::de::Deserialize<'de>>(
    reader: &mut &'de [u8],
    field_name: &str,
) -> Result<T, String> {
    bsatn::from_reader(reader).map_err(|e| format!("failed to decode field '{field_name}': {e}"))
}
