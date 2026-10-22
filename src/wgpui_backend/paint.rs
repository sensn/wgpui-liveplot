use wgpui::{
    App, BorderStyle, Bounds, ContentMask, Corners, Edges, Hsla, PathBuilder, Pixels, Rgba,
    TextRun, Window, font, point, px, quad, rgb,Point
};

use crate::geom::{ScreenPoint, ScreenRect};
use crate::render::{
    LineSegment, LineStyle, MarkerShape, MarkerStyle, RectStyle, RenderCommand, TextStyle,
};

use super::frame::PlotFrame;
//
//use wgpui::{Window, PathBuilder, Pixels, Bounds, point, px, rgb};
// Zeichnet eine Test-Sinuswelle über die WGPUI-Pfad-Pipeline.
/// Reagiert dynamisch auf Zoom und Pan, da sie KEINE festen Canvas-Bounds nutzt!
pub fn paint_wgpui_dynamic_test_sine(window: &mut Window) {
    // 1. Hole die aktuell gültige Maske, die GPUI für diesen Zeichenschritt erlaubt
    let current_mask = window.content_mask();
    let bounds = current_mask.bounds;

    // Falls die Maske kollabiert ist (Breite oder Höhe <= 0), abbrechen
    if bounds.size.width <= px(0.0) || bounds.size.height <= px(0.0) {
        return;
    }

    // 2. Nutze die echten Dimensionen des erlaubten Clip-Bereichs
    let start_x: f32 = bounds.origin.x.into();
    let width: f32 = bounds.size.width.into();
    
    let min_y: f32 = bounds.origin.y.into();
    let height_box: f32 = bounds.size.height.into();
    
    // Berechne die Mitte und die maximale Amplitude innerhalb der Maske
    let center_y = min_y + (height_box / 2.0);
    let amplitude = height_box * 0.4; // Nutze 80% des vertikalen Platzes (40% hoch, 40% runter)

    let mut builder = PathBuilder::fill();
    let steps = 60;
    let mut points = Vec::with_capacity(steps + 3);

    // Startpunkt links auf der dynamischen Nulllinie
    points.push(point(px(start_x), px(center_y)));

    // Erzeuge die Wellenform angepasst an die echten Bounds
    for i in 0..=steps {
        let t = (i as f32) / (steps as f32);
        let cur_x = start_x + width * t;
        let angle = t * 2.0 * std::f32::consts::PI * 3.0; // 3 schöne Oszillationen
        let cur_y = center_y + (angle.sin() * amplitude);
        
        points.push(point(px(cur_x), px(cur_y)));
    }

    // Endpunkt rechts auf der dynamischen Nulllinie
    points.push(point(px(start_x + width), px(center_y)));

    // Pfad-Struktur aufbauen und füllen
    if let Some(&first) = points.first() {
        builder.move_to(first);
        for &pt in points.iter().skip(1) {
            builder.line_to(pt);
        }
        builder.close();
        
        if let Ok(path) = builder.build() {
            window.paint_path(path, rgb(0x06b6d4)); // Cyan-Blau
        }
    }
    }
//
pub(crate) fn paint_frame(frame: &PlotFrame, window: &mut Window, cx: &mut App) {
    let mut clip_stack: Vec<ContentMask<Pixels>> = Vec::new();
    
    // 1. Die echte absolute Maske des Canvas holen
    let initial_mask = window.content_mask(); 
    clip_stack.push(initial_mask.clone());
    
    // Der absolute Startpunkt unseres Canvas auf dem Bildschirm
    let canvas_origin = initial_mask.bounds.origin;

    for command in frame.render.commands() {
        match command {
            crate::render::RenderCommand::ClipRect(rect) => {
                // 2. Lokale Ecken berechnen
                let local_min = point(px(rect.min.x), px(rect.min.y));
                let local_max = point(px(rect.max.x), px(rect.max.y));
                
                // 3. Absolute Ecken durch Aufaddieren des Canvas-Ursprungs bestimmen
                let abs_min = point(canvas_origin.x + local_min.x, canvas_origin.y + local_min.y);
                let abs_max = point(canvas_origin.x + local_max.x, canvas_origin.y + local_max.y);
                
                let absolute_bounds = Bounds::from_corners(abs_min, abs_max);

                // 4. Mit der übergeordneten Maske schneiden, um sauberes Verschachteln zu garantieren
                if let Some(parent_mask) = clip_stack.last() {
                    clip_stack.push(ContentMask {
                        bounds: parent_mask.bounds.intersect(&absolute_bounds),
                    });
                } else {
                    clip_stack.push(ContentMask {
                        bounds: absolute_bounds,
                    });
                }
            }
            crate::render::RenderCommand::ClipEnd => {
                if clip_stack.len() > 1 {
                    clip_stack.pop();
                }
            }
            crate::render::RenderCommand::LineSegments { segments, style } => {
                with_clip(window, &clip_stack, |window| {
                                      //paint_wgpui_dynamic_test_sine(window); 
                      paint_lines(window, segments, *style); 

                });
            }
            crate::render::RenderCommand::Points { points, style } => {
                with_clip(window, &clip_stack, |window| {
                    paint_points(window, points, *style);
                });
            }
            crate::render::RenderCommand::Rect { rect, style } => {
                with_clip(window, &clip_stack, |window| {
                    paint_rect(window, *rect, *style);
                });
            }
            cmd => {
                if let crate::render::RenderCommand::Text { position, text, style } = cmd {
                    if let Some(mask) = clip_stack.last() {
                        window.with_content_mask(Some(mask.clone()), |window| {
                            paint_text(window, cx, *position, text, style);
                        });
                    } else {
                        paint_text(window, cx, *position, text, style);
                    }
                }
            }
        }
    }
}


fn paint_lines(
    window: &mut Window, 
    segments: &[LineSegment], 
    style: LineStyle
) {
//_________
// 1. Hole die aktuell gültige Maske, die GPUI für diesen Zeichenschritt erlaubt
let current_mask = window.content_mask();
let bounds = current_mask.bounds;

// Falls die Maske kollabiert ist (Breite oder Höhe <= 0), abbrechen
if bounds.size.width <= px(0.0) || bounds.size.height <= px(0.0) {
    return;
}

// 2. Nutze die echten Dimensionen des erlaubten Clip-Bereichs
let start_x: f32 = bounds.origin.x.into();
let width: f32 = bounds.size.width.into();

let min_y: f32 = bounds.origin.y.into();
let height_box: f32 = bounds.size.height.into();

// 3. Zeichne ein komplett unsichtbares Rechteck (Alpha = 0.0)
let mut builder = PathBuilder::fill();

builder.move_to(point(px(start_x), px(min_y)));                  // Oben links
builder.line_to(point(px(start_x + width), px(min_y)));          // Oben rechts
builder.line_to(point(px(start_x + width), px(min_y + height_box))); // Unten rechts
builder.line_to(point(px(start_x), px(min_y + height_box)));     // Unten links
builder.close(); 

if let Ok(path) = builder.build() {
    let mut transparent_color = rgb(0x06b6d4);
    transparent_color.a = 0.0; // 🌟 100% Unsichtbar!
    window.paint_path(path, transparent_color);
}

    
//_______


    if segments.is_empty() {
        return;
    }
    
    // 1. Maske abrufen und Bounds korrekt zuweisen!
    let current_mask = window.content_mask();
    let bounds = current_mask.bounds;

    // Canvas-Offset extrahieren (Der absolute Startpunkt auf dem Bildschirm)
    let offset_x: f32 = bounds.origin.x.into();
    let offset_y: f32 = bounds.origin.y.into();

    // 2. Linienstärke definieren
    let thickness = style.width.max(3.0);
    let mut builder = PathBuilder::stroke(px(thickness));

    // 3. Jedes Segment einzeln zeichnen (Move + Line für diskrete Abschnitte)
    for segment in segments {
        let sx = offset_x + (segment.start.x as f32);
        let sy = offset_y + (segment.start.y as f32);
        let ex = offset_x + (segment.end.x as f32);
        let ey = offset_y + (segment.end.y as f32);
        
        builder.move_to(point(px(sx), px(sy)));
        builder.line_to(point(px(ex), px(ey)));
    }

    // 4. Pfad erzeugen und im Window einreichen
    if let Ok(path) = builder.build() {
        let mut opaque_color = style.color;
        opaque_color.a = 1.0; // Verhindert Shader-Transparenzfehler
        window.paint_path(path, opaque_color);
    }
}

fn paint_points(window: &mut Window, points: &[ScreenPoint], style: MarkerStyle) {
    if points.is_empty() {
        return;
    }

    let size = style.size.max(2.0);
    match style.shape {
        MarkerShape::Circle => {
            let radius = size * 0.5;
            for pt in points {
                let bounds = Bounds::from_corners(
                    point(px(pt.x - radius), px(pt.y - radius)),
                    point(px(pt.x + radius), px(pt.y + radius)),
                );
                window.paint_quad(quad(
                    bounds,
                    Corners::all(px(radius)),
                    style.color,
                    Edges::all(px(0.0)),
                    style.color,
                    BorderStyle::default(),
                ));
            }
        }
        MarkerShape::Square => {
            let half = size * 0.5;
            for pt in points {
                let bounds = Bounds::from_corners(
                    point(px(pt.x - half), px(pt.y - half)),
                    point(px(pt.x + half), px(pt.y + half)),
                );
                window.paint_quad(quad(
                    bounds,
                    Corners::all(px(0.0)),
                    style.color,
                    Edges::all(px(0.0)),
                    style.color,
                    BorderStyle::default(),
                ));
            }
        }
        MarkerShape::Cross => {
            let half = size * 0.5;
            let thickness = 1.0_f32;
            for pt in points {
                // Horizontale Linie des Kreuzes als Quad zeichnen
                let h_bounds = Bounds::from_corners(
                    point(px(pt.x - half), px(pt.y - thickness * 0.5)),
                    point(px(pt.x + half), px(pt.y + thickness * 0.5)),
                );
                window.paint_quad(quad(
                    h_bounds,
                    Corners::all(px(0.0)),
                    style.color,
                    Edges::all(px(0.0)),
                    style.color,
                    BorderStyle::default(),
                ));
                
                // Vertikale Linie des Kreuzes als Quad zeichnen
                let v_bounds = Bounds::from_corners(
                    point(px(pt.x - thickness * 0.5), px(pt.y - half)),
                    point(px(pt.x + thickness * 0.5), px(pt.y + half)),
                );
                window.paint_quad(quad(
                    v_bounds,
                    Corners::all(px(0.0)),
                    style.color,
                    Edges::all(px(0.0)),
                    style.color,
                    BorderStyle::default(),
                ));
            }
        }
    }
}

fn paint_rect(window: &mut Window, rect: ScreenRect, style: RectStyle) {
    let bounds = to_bounds(rect);
    let quad = quad(
        bounds,
        Corners::all(px(0.0)),
        style.fill,
        Edges::all(px(style.stroke_width)),
        style.stroke,
        BorderStyle::default(),
    );
    window.paint_quad(quad);
}

fn paint_text(
    window: &mut Window,
    cx: &mut App, // cx wieder hinzugefügt für das 6-Argumente-paint von wgpui 0.3.6
    position: ScreenPoint,
    text: &str,
    style: &TextStyle,
) {
    if text.is_empty() {
        return;
    }
    let font_size = px(style.size);
    let run = TextRun {
        len: text.len(),
        font: font(".SystemUIFont"),
        color: to_hsla(style.color),
        background_color: None,
        underline: None,
        strikethrough: None,
    };
    
    let shaped = window
        .text_system()
        .shape_line(text.to_string().into(), font_size, &[run], None);
    let line_height = shaped.ascent + shaped.descent;
    let origin = point(px(position.x), px(position.y));
    
    // WGPUI v0.3.6 benötigt exakt diese 6 Argumente:
    let _ = shaped.paint(
        origin,
        line_height,
        wgpui::TextAlign::Left,
        None,
        window,
        cx,
    );
}
pub(crate) fn to_hsla(color: Rgba) -> Hsla {
    Hsla::from(color)
}

fn to_bounds(rect: ScreenRect) -> Bounds<Pixels> {
    Bounds::from_corners(
        point(px(rect.min.x), px(rect.min.y)),
        point(px(rect.max.x), px(rect.max.y)),
    )
}
fn with_clip(window: &mut Window, stack: &[ContentMask<Pixels>], f: impl FnOnce(&mut Window)) {
    // 🛡️ DIAGNOSE: Wir ignorieren die gepushten Sub-Masken und nutzen IMMER 
    // das allererste Element (die originale, funktionierende Canvas-Maske)
    if let Some(initial_mask) = stack.first() {
        window.with_content_mask(Some(initial_mask.clone()), f);
    } else {
        f(window);
    }
}
/*
fn with_clip(window: &mut Window, stack: &[ContentMask<Pixels>], f: impl FnOnce(&mut Window)) {
    if let Some(mask) = stack.last() {
        window.with_content_mask(Some(mask.clone()), f);
    } else {
        f(window);
    }
}
*/