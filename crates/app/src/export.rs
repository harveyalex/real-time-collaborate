use std::f64::consts::PI;

use js_sys::Array;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{
    Blob, BlobPropertyBag, HtmlAnchorElement, HtmlCanvasElement, Url,
};

use shared::ElementKind;
use stdb_client::ElementData;

use crate::state::AppState;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/// Convert a packed RGBA u32 to a 6-digit hex string suitable for SVG.
pub fn color_u32_to_hex(color: u32) -> String {
    let r = (color >> 24) & 0xFF;
    let g = (color >> 16) & 0xFF;
    let b = (color >> 8) & 0xFF;
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

/// Alpha channel (0-1) from a packed RGBA u32.
fn color_u32_alpha(color: u32) -> f64 {
    let a = color & 0xFF;
    a as f64 / 255.0
}

/// HTML-escape a string for safe embedding in SVG/XML.
pub fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Compute the bounding box (min_x, min_y, max_x, max_y) of a slice of elements.
/// Falls back to (0, 0, 100, 100) when the slice is empty.
pub fn bounding_box(elements: &[ElementData]) -> (f64, f64, f64, f64) {
    if elements.is_empty() {
        return (0.0, 0.0, 100.0, 100.0);
    }

    let mut min_x = f64::MAX;
    let mut min_y = f64::MAX;
    let mut max_x = f64::MIN;
    let mut max_y = f64::MIN;

    for elem in elements {
        // Expand by x/y/width/height for box-shaped elements.
        let x2 = elem.x + elem.width;
        let y2 = elem.y + elem.height;
        min_x = min_x.min(elem.x).min(x2);
        min_y = min_y.min(elem.y).min(y2);
        max_x = max_x.max(elem.x).max(x2);
        max_y = max_y.max(elem.y).max(y2);

        // Also expand by any explicit points (lines, arrows, freehand).
        for p in &elem.points {
            min_x = min_x.min(p.x);
            min_y = min_y.min(p.y);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }
    }

    (min_x, min_y, max_x, max_y)
}

/// Create a hidden `<a>` element, set its href + download attribute, click it,
/// then revoke the object URL.
pub fn trigger_download(object_url: &str, filename: &str) {
    let window = match web_sys::window() {
        Some(w) => w,
        None => return,
    };
    let document = match window.document() {
        Some(d) => d,
        None => return,
    };

    let a: HtmlAnchorElement = document
        .create_element("a")
        .expect("create <a>")
        .dyn_into()
        .expect("HtmlAnchorElement");

    a.set_href(object_url);
    a.set_download(filename);
    a.set_attribute("style", "display:none").unwrap_or(());

    document
        .body()
        .expect("body")
        .append_child(&a)
        .expect("append");

    a.click();

    document
        .body()
        .expect("body")
        .remove_child(&a)
        .expect("remove");

    Url::revoke_object_url(object_url).unwrap_or(());
}

// ---------------------------------------------------------------------------
// SVG element serialisation
// ---------------------------------------------------------------------------

pub fn element_to_svg(elem: &ElementData) -> String {
    let stroke = color_u32_to_hex(elem.stroke_color);
    let stroke_alpha = color_u32_alpha(elem.stroke_color);
    let fill = color_u32_to_hex(elem.fill_color);
    let fill_alpha = color_u32_alpha(elem.fill_color);
    let sw = elem.stroke_width;
    let opacity = elem.opacity;

    // Build a transform="" string for rotation if needed.
    let transform = if elem.rotation != 0.0 {
        let cx = elem.x + elem.width / 2.0;
        let cy = elem.y + elem.height / 2.0;
        let deg = elem.rotation * 180.0 / PI;
        format!(r#" transform="rotate({:.4},{:.4},{:.4})""#, deg, cx, cy)
    } else {
        String::new()
    };

    // fill-opacity / stroke-opacity attrs
    let fill_opacity_attr = format!(r#" fill-opacity="{:.4}""#, fill_alpha);
    let stroke_opacity_attr = format!(r#" stroke-opacity="{:.4}""#, stroke_alpha);

    match elem.kind {
        ElementKind::Rectangle => {
            format!(
                r#"<rect x="{x:.4}" y="{y:.4}" width="{w:.4}" height="{h:.4}" fill="{fill}"{fo} stroke="{stroke}"{so} stroke-width="{sw}" opacity="{opacity}"{transform}/>"#,
                x = elem.x, y = elem.y, w = elem.width, h = elem.height,
                fill = fill, fo = fill_opacity_attr,
                stroke = stroke, so = stroke_opacity_attr,
                sw = sw, opacity = opacity, transform = transform,
            )
        }
        ElementKind::Ellipse => {
            let cx = elem.x + elem.width / 2.0;
            let cy = elem.y + elem.height / 2.0;
            let rx = elem.width.abs() / 2.0;
            let ry = elem.height.abs() / 2.0;
            format!(
                r#"<ellipse cx="{cx:.4}" cy="{cy:.4}" rx="{rx:.4}" ry="{ry:.4}" fill="{fill}"{fo} stroke="{stroke}"{so} stroke-width="{sw}" opacity="{opacity}"{transform}/>"#,
                cx = cx, cy = cy, rx = rx, ry = ry,
                fill = fill, fo = fill_opacity_attr,
                stroke = stroke, so = stroke_opacity_attr,
                sw = sw, opacity = opacity, transform = transform,
            )
        }
        ElementKind::Line | ElementKind::Arrow | ElementKind::Freehand => {
            if elem.points.len() < 2 {
                return String::new();
            }
            let mut d = format!("M {:.4} {:.4}", elem.points[0].x, elem.points[0].y);
            for p in &elem.points[1..] {
                d.push_str(&format!(" L {:.4} {:.4}", p.x, p.y));
            }
            // For arrows, append the arrowhead lines.
            if elem.kind == ElementKind::Arrow {
                let last = &elem.points[elem.points.len() - 1];
                let prev = &elem.points[elem.points.len() - 2];
                let angle = (last.y - prev.y).atan2(last.x - prev.x);
                let head_len = 12.0;
                let ax1 = last.x - head_len * (angle - PI / 6.0).cos();
                let ay1 = last.y - head_len * (angle - PI / 6.0).sin();
                let ax2 = last.x - head_len * (angle + PI / 6.0).cos();
                let ay2 = last.y - head_len * (angle + PI / 6.0).sin();
                d.push_str(&format!(
                    " M {:.4} {:.4} L {:.4} {:.4} M {:.4} {:.4} L {:.4} {:.4}",
                    last.x, last.y, ax1, ay1,
                    last.x, last.y, ax2, ay2,
                ));
            }
            format!(
                r#"<path d="{d}" fill="none" stroke="{stroke}"{so} stroke-width="{sw}" opacity="{opacity}"{transform}/>"#,
                d = d, stroke = stroke, so = stroke_opacity_attr,
                sw = sw, opacity = opacity, transform = transform,
            )
        }
        ElementKind::Text => {
            let escaped = html_escape(&elem.text_content);
            format!(
                r#"<text x="{x:.4}" y="{y:.4}" font-family="monospace" font-size="{fs}" fill="{stroke}"{so} opacity="{opacity}"{transform}>{text}</text>"#,
                x = elem.x, y = elem.y + elem.font_size as f64,
                fs = elem.font_size,
                stroke = stroke, so = stroke_opacity_attr,
                opacity = opacity, transform = transform,
                text = escaped,
            )
        }
    }
}

// ---------------------------------------------------------------------------
// PNG export
// ---------------------------------------------------------------------------

pub fn export_png(state: &AppState) {
    let elements = state.store.sorted_elements();
    let (min_x, min_y, max_x, max_y) = bounding_box(&elements);
    let pad = 20.0;
    let canvas_w = (max_x - min_x + pad * 2.0).max(1.0);
    let canvas_h = (max_y - min_y + pad * 2.0).max(1.0);

    let window = match web_sys::window() {
        Some(w) => w,
        None => {
            log::error!("export_png: no window");
            return;
        }
    };
    let document = match window.document() {
        Some(d) => d,
        None => {
            log::error!("export_png: no document");
            return;
        }
    };

    let canvas: HtmlCanvasElement = document
        .create_element("canvas")
        .expect("create canvas")
        .dyn_into()
        .expect("HtmlCanvasElement");

    canvas.set_width(canvas_w as u32);
    canvas.set_height(canvas_h as u32);

    let ctx: web_sys::CanvasRenderingContext2d = canvas
        .get_context("2d")
        .ok()
        .flatten()
        .expect("2d context")
        .dyn_into()
        .expect("CanvasRenderingContext2d");

    // White background.
    ctx.set_fill_style_str("#ffffff");
    ctx.fill_rect(0.0, 0.0, canvas_w, canvas_h);

    // Translate so elements are offset into view with padding.
    ctx.translate(-min_x + pad, -min_y + pad).unwrap_or(());

    for elem in &elements {
        crate::canvas::render_element(&ctx, elem);
    }

    // Use canvas.toBlob() via a JS closure.
    let canvas_clone = canvas.clone();
    let closure = Closure::once(move |blob_val: JsValue| {
        if blob_val.is_null() || blob_val.is_undefined() {
            log::error!("export_png: toBlob returned null");
            return;
        }
        let blob: Blob = match blob_val.dyn_into() {
            Ok(b) => b,
            Err(_) => {
                log::error!("export_png: could not cast to Blob");
                return;
            }
        };
        let url = match Url::create_object_url_with_blob(&blob) {
            Ok(u) => u,
            Err(_) => {
                log::error!("export_png: createObjectURL failed");
                return;
            }
        };
        trigger_download(&url, "canvas.png");
        // canvas_clone kept alive until here
        let _ = &canvas_clone;
    });

    canvas
        .to_blob(closure.as_ref().unchecked_ref())
        .unwrap_or(());

    closure.forget();
}

// ---------------------------------------------------------------------------
// SVG export
// ---------------------------------------------------------------------------

pub fn export_svg(state: &AppState) {
    let elements = state.store.sorted_elements();
    let (min_x, min_y, max_x, max_y) = bounding_box(&elements);
    let pad = 20.0;
    let vb_x = min_x - pad;
    let vb_y = min_y - pad;
    let vb_w = (max_x - min_x + pad * 2.0).max(1.0);
    let vb_h = (max_y - min_y + pad * 2.0).max(1.0);

    let mut svg = format!(
        r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="{x:.4} {y:.4} {w:.4} {h:.4}" width="{w:.4}" height="{h:.4}">"#,
        x = vb_x, y = vb_y, w = vb_w, h = vb_h,
    );
    svg.push_str("\n  <rect x=\"");
    svg.push_str(&format!("{:.4}", vb_x));
    svg.push_str("\" y=\"");
    svg.push_str(&format!("{:.4}", vb_y));
    svg.push_str("\" width=\"");
    svg.push_str(&format!("{:.4}", vb_w));
    svg.push_str("\" height=\"");
    svg.push_str(&format!("{:.4}", vb_h));
    svg.push_str("\" fill=\"white\"/>\n");

    for elem in &elements {
        let s = element_to_svg(elem);
        if !s.is_empty() {
            svg.push_str("  ");
            svg.push_str(&s);
            svg.push('\n');
        }
    }

    svg.push_str("</svg>\n");

    // Build a Blob from the SVG string.
    let parts = Array::new();
    parts.push(&JsValue::from_str(&svg));

    let opts = BlobPropertyBag::new();
    opts.set_type("image/svg+xml");

    let blob = match Blob::new_with_str_sequence_and_options(&parts, &opts) {
        Ok(b) => b,
        Err(_) => {
            log::error!("export_svg: failed to create Blob");
            return;
        }
    };

    let url = match Url::create_object_url_with_blob(&blob) {
        Ok(u) => u,
        Err(_) => {
            log::error!("export_svg: createObjectURL failed");
            return;
        }
    };

    trigger_download(&url, "canvas.svg");
}
