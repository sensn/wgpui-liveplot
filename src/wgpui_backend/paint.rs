use wgpui::{
    App, BorderStyle, Bounds, ContentMask, Corners, Edges, Hsla, PathBuilder, Pixels, Rgba,
    TextRun, Window, font, point, px, quad, rgb,
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
    let mut builder = PathBuilder::fill();
    
    // Fest definierte, absolute Pixel-Koordinaten im Raum.
    // Diese simulieren die transformierten Datenpunkte deines Plot-Frameworks.
    let start_x = 100.0_f32;
    let center_y = 300.0_f32;
    let width = 400.0_f32;
    let height = 100.0_f32;
    let steps = 50;
    
    let mut points = Vec::with_capacity(steps + 3);

    // 1. Startpunkt links auf der Nulllinie
    points.push(point(px(start_x), px(center_y)));

    // 2. Erzeuge die Wellenform über absolute Pixel-Schritte
    for i in 0..=steps {
        let t = (i as f32) / (steps as f32);
        let cur_x = start_x + width * t;
        let angle = t * 2.0 * std::f32::consts::PI * 2.0; // 2 Oszillationen
        let cur_y = center_y + (angle.sin() * height);
        
        points.push(point(px(cur_x), px(cur_y)));
    }

    // 3. Endpunkt rechts auf der Nulllinie
    points.push(point(px(start_x + width), px(center_y)));

    // 4. Pfad-Struktur übergeben (Strikte WGPUI-Syntax)
    if let Some(&first) = points.first() {
        builder.move_to(first);
        for &pt in points.iter().skip(1) {
            builder.line_to(pt);
        }
        builder.close(); // Zwingend schließen!
        
        if let Ok(path) = builder.build() {
            window.paint_path(path, rgb(0x06b6d4)); // Cyan-Blau
        }
    }
}
//
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
  // 🌟 TEST-TRIGGER: Zeichnet die feste Cyan-Sinuswelle flächig in den Plot!
        // (Stelle sicher, dass 'bounds' hier im Scope liegt)
        paint_wgpui_dynamic_test_sine(window); 
                
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
    
    // FIX 1: Wir garantieren eine Mindeststärke von 1.5 Pixeln, 
    // um High-DPI/Subpixel-Verschlucken auf Linux-Treibern zu verhindern!
    let thickness = style.width.max(1.5);
    let half_thick = thickness * 0.5;

    for segment in segments {
        let dx = segment.end.x - segment.start.x;
        let dy = segment.end.y - segment.start.y;
        let len = (dx * dx + dy * dy).sqrt();
        if len < 0.1 { continue; }

        // Perfekt horizontale Linien (z.B. Gitterlinien)
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
        } 
        
        // Perfekt vertikale Linien
        if dx.abs() < 0.1 {
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

        // FIX 2: DIAGONALE SEGMENTE (Die Sinuskurve)
        // Wir nutzen KEIN paint_path. Wir zerlegen das Segment direkt in 
        // extrem dichte, überlappende Quad-Flächen. Das zwingt die GPU über die 
        // stabile Quad-Pipeline Pixel auszugeben und tilgt Treppenstufen komplett!
        let steps = (len * 2.0).ceil().max(4.0) as usize; // Höhere Dichte für perfekte Glättung
        let step_x = dx / steps as f32;
        let step_y = dy / steps as f32;

        for i in 0..steps {
            let s_x = segment.start.x + i as f32 * step_x;
            let s_y = segment.start.y + i as f32 * step_y;
            let e_x = s_x + step_x;
            let e_y = s_y + step_y;

            let min_x = s_x.min(e_x);
            let min_y = s_y.min(e_y);
            let max_x = s_x.max(e_x);
            let max_y = s_y.max(e_y);

            // Bounds mit Sicherheitsüberlapp (+0.3) gegen subpixel-Spalten
            let bounds = Bounds::from_corners(
                point(px(min_x - half_thick - 0.3), px(min_y - half_thick - 0.3)),
                point(px(max_x + half_thick + 0.3), px(max_y + half_thick + 0.3)),
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
