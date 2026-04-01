use leptos::prelude::*;
use shared::point::Point;
use std::collections::HashMap;

/// Client-side representation of an element (mirrors server Element table).
#[derive(Clone, Debug)]
pub struct ElementData {
    pub id: u64,
    pub room_id: u64,
    pub kind: shared::ElementKind,
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
    pub rotation: f64,
    pub points: Vec<Point>,
    pub stroke_color: u32,
    pub fill_color: u32,
    pub stroke_width: f32,
    pub opacity: f32,
    pub font_size: f32,
    pub text_content: String,
    pub z_index: i32,
}

/// Client-side cursor representation.
#[derive(Clone, Debug)]
pub struct CursorData {
    pub x: f64,
    pub y: f64,
    pub name: String,
    pub color: u32,
}

/// Reactive store for all SpacetimeDB state.
#[derive(Clone)]
pub struct StdbStore {
    pub elements: RwSignal<HashMap<u64, ElementData>>,
    pub cursors: RwSignal<HashMap<String, CursorData>>, // keyed by identity hex
    pub connected: RwSignal<bool>,
    pub my_identity: RwSignal<Option<String>>,
    pub current_room: RwSignal<Option<u64>>,
}

impl StdbStore {
    pub fn new() -> Self {
        Self {
            elements: RwSignal::new(HashMap::new()),
            cursors: RwSignal::new(HashMap::new()),
            connected: RwSignal::new(false),
            my_identity: RwSignal::new(None),
            current_room: RwSignal::new(None),
        }
    }

    /// Get elements sorted by z_index for rendering.
    pub fn sorted_elements(&self) -> Vec<ElementData> {
        self.elements.with(|elems| {
            let mut sorted: Vec<_> = elems.values().cloned().collect();
            sorted.sort_by_key(|e| e.z_index);
            sorted
        })
    }
}
