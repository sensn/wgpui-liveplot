use wgpui::{
    App, BorderStyle, Bounds, ContentMask, Corners, Edges, Hsla, PathBuilder, Pixels, Rgba,
    TextRun, Window, font, point, px, quad,
};

use crate::geom::{ScreenPoint, ScreenRect};
use crate::render::{
    LineSegment, LineStyle, MarkerShape, MarkerStyle, RectStyle, RenderCommand, TextStyle,
};

use super::frame::PlotFrame;

pub(crate) fn paint_frame(frame: &PlotFrame, window: &mut Window, cx: &mut App) {
    let mut clip_stack: Vec<ContentMask<Pixels>> = Vec::new();
    
    for command in frame.render.commands() {
        match command {
            crate::render::RenderCommand::ClipRect(rect) => {
                clip_stack.push(ContentMask {
                    bounds: to_bounds(*rect),
                });
            }
            crate::render::RenderCommand::ClipEnd => {
                clip_stack.pop();
            }
            crate::render::RenderCommand::LineSegments { segments, style } => {
                with_clip(window, &clip_stack, |window| {
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
            // Hier nutzen wir den gesamten Enum-Pfad und fangen die Text-Variante ab.
            // Falls der Makro-Konflikt im Match-Arm selbst liegt, nutzen wir den Fallback unten!
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

fn paint_lines(window: &mut Window, segments: &[LineSegment], style: LineStyle) {
    if segments.is_empty() {
        return;
    }
    let thickness = style.width.max(1.0);
    let half_thick = thickness * 0.5;

    for segment in segments {
        let dx = segment.end.x - segment.start.x;
        let dy = segment.end.y - segment.start.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.1 { continue; }

        // 1. Wenn die Linie exakt horizontal oder vertikal verläuft, 
        // nutzen wir die performante achsensymmetrische Quad-Abkürzung
        if dy.abs() < 0.1 {
            let min_x = segment.start.x.min(segment.end.x);
            let bounds = Bounds::from_corners(
                point(px(min_x), px(segment.start.y - half_thick)),
                point(px(min_x + len), px(segment.start.y + half_thick)),
            );
            window.paint_quad(quad(
                bounds,
                Corners::all(px(0.0)),
                style.color,
                Edges::all(px(0.0)),
                style.color,
                BorderStyle::default(),
            ));
            continue;
        } else if dx.abs() < 0.1 {
            let min_y = segment.start.y.min(segment.end.y);
            let bounds = Bounds::from_corners(
                point(px(segment.start.x - half_thick), px(min_y)),
                point(px(segment.start.x + half_thick), px(min_y + len)),
            );
            window.paint_quad(quad(
                bounds,
                Corners::all(px(0.0)),
                style.color,
                Edges::all(px(0.0)),
                style.color,
                BorderStyle::default(),
            ));
            continue;
        }

        // 2. FÜR DIAGONALE LINIEN (Die eigentliche Sinuskurve):
        // Um Treppenstufen zu verhindern, zerlegen wir das Segment in 
        // 4 sehr feine, aneinandergereihte Sub-Quads. Das emuliert eine
        // stufenlose, glatte Kurve perfekt im stabilen Quad-Rasterizer!
        let steps = 4;
        let step_x = dx / steps as f32;
        let step_y = dy / steps as f32;

        for i in 0..steps {
            let start_x = segment.start.x + i as f32 * step_x;
            let start_y = segment.start.y + i as f32 * step_y;
            let end_x = start_x + step_x;
            let end_y = start_y + step_y;

            let min_x = start_x.min(end_x);
            let min_y = start_y.min(end_y);
            let max_x = start_x.max(end_x);
            let max_y = start_y.max(end_y);

            let bounds = Bounds::from_corners(
                point(px(min_x - 0.2), px(min_y - 0.2)), // Winziger Überlass verhindert Lücken
                point(px(max_x + 0.2), px(max_y + 0.2)),
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
    if let Some(mask) = stack.last() {
        window.with_content_mask(Some(mask.clone()), f);
    } else {
        f(window);
    }
}
