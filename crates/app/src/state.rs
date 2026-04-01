use leptos::prelude::*;
use leptos::reactive::owner::LocalStorage;
use stdb_client::StdbStore;
use stdb_client::connection::StdbConnection;

pub use crate::types::{VimMode, Tool};

#[derive(Clone, Debug)]
pub struct Camera {
    pub x: f64,
    pub y: f64,
    pub zoom: f64,
}

impl Default for Camera {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            zoom: 1.0,
        }
    }
}

/// Global application state, provided via Leptos context.
#[derive(Clone)]
pub struct AppState {
    pub store: StdbStore,
    pub connection: RwSignal<Option<StdbConnection>, LocalStorage>,
    pub mode: RwSignal<VimMode>,
    pub tool: RwSignal<Tool>,
    pub camera: RwSignal<Camera>,
    pub selected_ids: RwSignal<Vec<u64>>,
    pub command_buffer: RwSignal<String>,
    pub key_buffer: RwSignal<String>,
    pub stroke_color: RwSignal<u32>,
    pub fill_color: RwSignal<u32>,
    pub stroke_width: RwSignal<f32>,
    pub mouse_pos: RwSignal<(f64, f64)>,
    pub canvas_size: RwSignal<(f64, f64)>,
    pub show_help: RwSignal<bool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            store: StdbStore::new(),
            connection: RwSignal::new_local(None),
            mode: RwSignal::new(VimMode::Normal),
            tool: RwSignal::new(Tool::Select),
            camera: RwSignal::new(Camera::default()),
            selected_ids: RwSignal::new(vec![]),
            command_buffer: RwSignal::new(String::new()),
            key_buffer: RwSignal::new(String::new()),
            stroke_color: RwSignal::new(shared::DEFAULT_STROKE_COLOR),
            fill_color: RwSignal::new(shared::DEFAULT_FILL_COLOR),
            stroke_width: RwSignal::new(shared::DEFAULT_STROKE_WIDTH),
            mouse_pos: RwSignal::new((0.0, 0.0)),
            canvas_size: RwSignal::new((0.0, 0.0)),
            show_help: RwSignal::new(false),
        }
    }

    /// Convert screen coordinates to world coordinates.
    pub fn screen_to_world(&self, sx: f64, sy: f64) -> (f64, f64) {
        let cam = self.camera.get_untracked();
        let wx = (sx / cam.zoom) + cam.x;
        let wy = (sy / cam.zoom) + cam.y;
        (wx, wy)
    }

    /// Convert world coordinates to screen coordinates.
    pub fn world_to_screen(&self, wx: f64, wy: f64) -> (f64, f64) {
        let cam = self.camera.get_untracked();
        let sx = (wx - cam.x) * cam.zoom;
        let sy = (wy - cam.y) * cam.zoom;
        (sx, sy)
    }
}
